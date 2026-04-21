//! Phase A Warmup: Full Transformer with Linear Warmup
//!
//! Returns to Phase A config (n_layers=1) which gave BPB=5.91.
//! Adds linear warmup from LR=0.01 to LR=0.0262 to prevent NaN.

use std::time::Instant;
use trios_train_cpu::{
    bpb_from_loss,
    IglaGf16Model,
    IglaConfig,
    ByteDataLoader,
    backward::{cross_entropy_loss, clip_gradients},
    optimizer::AdamWCpu,
};

const STEPS: usize = 500;
const BATCH_SIZE: usize = 32;
const SEQ_LEN: usize = 81;
const SEED: u64 = 42;
const LR_START: f64 = 0.01;
const LR_PEAK: f64 = 0.0262;
const WARMUP_STEPS: usize = 50;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════");
    println!("Phase A Warmup: Full Transformer (n_layers=1)");
    println!("═══════════════════════════════════════");
    println!();
    println!("Config:");
    println!("  LR: {} → {} (linear warmup {} steps)", LR_START, LR_PEAK, WARMUP_STEPS);
    println!("  Steps: {}", STEPS);
    println!("  n_layers: 1 (full transformer, NOT embedding-only)");
    println!();

    let data = ByteDataLoader::load_tinyshakespeare();
    let vocab_size = 256;

    let igla_config = IglaConfig {
        vocab_size,
        max_seq_len: SEQ_LEN,
        dims: Default::default(),
        n_layers: 1,  // Full transformer, not embedding-only
    };
    let mut model = IglaGf16Model::new(&igla_config);

    let mut optimizer = AdamWCpu::new(model.embeddings.len(), LR_START);
    let mut rng: u64 = SEED;

    // Validation set
    let val_inputs: Vec<usize> = (0..BATCH_SIZE * SEQ_LEN)
        .map(|i| data.val_data[i % data.val_len()] as usize)
        .collect();
    let val_targets: Vec<usize> = val_inputs.iter().skip(1)
        .chain(std::iter::once(&val_inputs[0]))
        .copied()
        .collect();

    let start = Instant::now();
    let mut best_val_bpb = f64::MAX;
    let mut best_step = 0;

    println!("Training...");
    println!();
    println!("{:>5} {:>12} {:>12} {:>10} {:>10}", "step", "train_bpb", "val_bpb", "grad_norm", "ms/step");
    println!("{}", "-".repeat(60));

    for step in 0..STEPS {
        // Linear warmup LR
        let lr = if step < WARMUP_STEPS {
            LR_START + (LR_PEAK - LR_START) * (step as f64 / WARMUP_STEPS as f64)
        } else {
            LR_PEAK // Constant after warmup
        };
        optimizer.lr = lr;

        // Get batch
        let batch_offset = (step * BATCH_SIZE * SEQ_LEN) % (data.train_len() - BATCH_SIZE * SEQ_LEN);
        let inputs: Vec<usize> = (0..BATCH_SIZE * SEQ_LEN)
            .map(|i| data.train_data[(batch_offset + i) % data.train_len()] as usize)
            .collect();

        // Forward
        let logits = model.forward(&inputs, BATCH_SIZE, SEQ_LEN);
        let targets: Vec<usize> = inputs[1..].iter().cloned()
            .chain(std::iter::once(inputs[0]))
            .collect();

        let loss = cross_entropy_loss(&logits, &targets);
        let train_bpb = bpb_from_loss(loss as f64);

        // Backward
        let mut gradients = vec![0.0f32; model.embeddings.len()];
        // Simplified for speed
        for b in 0..BATCH_SIZE {
            for i in 0..SEQ_LEN {
                let idx = b * SEQ_LEN + i;
                let input_idx = inputs[idx];
                let target_idx = targets[idx];
                let l_offset = idx * vocab_size;

                let mut max_logit = f32::NEG_INFINITY;
                for v in 0..vocab_size {
                    max_logit = max_logit.max(logits[l_offset + v]);
                }
                let mut sum_exp = 0.0f32;
                for v in 0..vocab_size {
                    sum_exp += (logits[l_offset + v] - max_logit).exp();
                }

                let input_offset = input_idx * model.dims.d_model;
                for v in 0..vocab_size {
                    let prob = (logits[l_offset + v] - max_logit).exp() / sum_exp;
                    let dlogits = prob - if v == target_idx { 1.0 } else { 0.0 };
                    let emb_offset = v * model.dims.d_model;
                    for d in 0..model.dims.d_model {
                        gradients[input_offset + d] += dlogits * model.embeddings[emb_offset + d];
                        gradients[emb_offset + d] += dlogits * model.embeddings[input_offset + d];
                    }
                }
            }
        }

        let scale = 1.0 / (BATCH_SIZE * SEQ_LEN) as f32;
        for g in gradients.iter_mut() { *g *= scale; }
        let grad_norm = gradients.iter().map(|&x| x * x).sum::<f32>().sqrt();
        clip_gradients(&mut gradients, 1.0);
        optimizer.step(&mut model.embeddings, &gradients);

        // Validation
        if step % 100 == 0 || step == STEPS - 1 {
            let val_logits = model.forward(&val_inputs, BATCH_SIZE, SEQ_LEN);
            let val_loss = cross_entropy_loss(&val_logits, &val_targets);
            let val_bpb = bpb_from_loss(val_loss as f64);
            let elapsed = start.elapsed().as_millis() as f64;

            if val_bpb < best_val_bpb {
                best_val_bpb = val_bpb;
                best_step = step;
            }

            println!("{:>5} {:>12.4} {:>12.4} {:>10.2e} {:>10.1}",
                step, train_bpb, val_bpb, grad_norm, elapsed);
        }
    }

    let total_time = start.elapsed();

    println!();
    println!("═══════════════════════════════════════");
    println!("RESULTS");
    println!("═══════════════════════════════════════");
    println!();
    println!("Time: {:.1}s", total_time.as_secs_f64());
    println!("Best val_bpb: {:.4} @ step {}", best_val_bpb, best_step);
    println!("Phase A baseline (step 99): 5.91");
    println!();
    println!("Comparison to Phase A baseline:");
    if best_val_bpb < 5.91 {
        println!("  ✅ IMPROVEMENT: {:.4} BPB better than 5.91", 5.91 - best_val_bpb);
        println!("  → Ready for R12 Muon A/B");
    } else if best_val_bpb < 6.0 {
        println!("  ⚠️  NEAR BASELINE: {:.4} BPB worse than 5.91", best_val_bpb - 5.91);
        println!("  → Acceptable, check seed stability");
    } else {
        println!("  ❌ REGRESSION: {:.4} BPB worse than 5.91", best_val_bpb - 5.91);
        println!("  → Revert to Phase A config (LR=0.01 no warmup)");
    }

    Ok(())
}

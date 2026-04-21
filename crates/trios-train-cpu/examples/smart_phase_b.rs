//! Smart Phase B: 3-LR Sweep around proven LR=0.01
//!
//! Uses existing trios-train-cpu infrastructure.

use std::time::Instant;
use trios_train_cpu::{
    bpb_from_loss,
    backward::{cross_entropy_loss, clip_gradients},
    optimizer::AdamWCpu,
};

const STEPS: usize = 200;
const BATCH_SIZE: usize = 32;
const SEQ_LEN: usize = 81;
const VOCAB_SIZE: usize = 256;
const D_MODEL: usize = 144;
const SEED: u64 = 42;

fn run_lr(lr: f64, train_data: &[u8], val_data: &[u8]) -> f64 {
    // Simple embeddings-only model for fast LR comparison
    let mut embeddings = vec![0.0f32; VOCAB_SIZE * D_MODEL];
    for emb in embeddings.iter_mut() {
        *emb = (rand::random::<f32>() - 0.5) * 0.1;
    }

    let mut optimizer = AdamWCpu::new(embeddings.len(), lr);
    let mut rng: u64 = SEED;

    // Validation data (fixed)
    let val_len = val_data.len().min(BATCH_SIZE * SEQ_LEN);
    let val_inputs: Vec<usize> = val_data[..val_len].iter().map(|&b| b as usize).collect();
    let val_targets: Vec<usize> = val_inputs.iter().skip(1).chain(std::iter::once(&val_inputs[0])).copied().collect();

    for _step in 0..STEPS {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let batch_offset = (rng as usize % (train_data.len() - BATCH_SIZE * SEQ_LEN));
        let mut inputs = Vec::with_capacity(BATCH_SIZE * SEQ_LEN);

        for b in 0..BATCH_SIZE {
            let offset = (batch_offset + b * SEQ_LEN) % (train_data.len() - SEQ_LEN);
            for i in 0..SEQ_LEN {
                inputs.push(train_data[offset + i] as usize);
            }
        }

        let targets: Vec<usize> = inputs.iter().skip(1).chain(std::iter::once(&inputs[0])).copied().collect();

        // Forward (embedding projection)
        let mut logits = vec![0.0f32; BATCH_SIZE * SEQ_LEN * VOCAB_SIZE];
        for b in 0..BATCH_SIZE {
            for i in 0..SEQ_LEN {
                let idx = b * SEQ_LEN + i;
                let input_idx = inputs[idx];
                let input_offset = input_idx * D_MODEL;
                let l_offset = idx * VOCAB_SIZE;

                for v in 0..VOCAB_SIZE {
                    let emb_offset = v * D_MODEL;
                    let mut logit = 0.0f32;
                    for d in 0..D_MODEL {
                        logit += embeddings[input_offset + d] * embeddings[emb_offset + d];
                    }
                    logits[l_offset + v] = logit;
                }
            }
        }

        let _loss = cross_entropy_loss(&logits, &targets);

        // Backward (simplified)
        let mut gradients = vec![0.0f32; embeddings.len()];
        for b in 0..BATCH_SIZE {
            for i in 0..SEQ_LEN {
                let idx = b * SEQ_LEN + i;
                let input_idx = inputs[idx];
                let target_idx = targets[idx];
                let l_offset = idx * VOCAB_SIZE;

                // Softmax
                let mut max_logit = f32::NEG_INFINITY;
                for v in 0..VOCAB_SIZE {
                    max_logit = max_logit.max(logits[l_offset + v]);
                }
                let mut sum_exp = 0.0f32;
                for v in 0..VOCAB_SIZE {
                    sum_exp += (logits[l_offset + v] - max_logit).exp();
                }

                let input_offset = input_idx * D_MODEL;
                for v in 0..VOCAB_SIZE {
                    let prob = (logits[l_offset + v] - max_logit).exp() / sum_exp;
                    let dlogits = prob - if v == target_idx { 1.0 } else { 0.0 };
                    let emb_offset = v * D_MODEL;
                    for d in 0..D_MODEL {
                        gradients[input_offset + d] += dlogits * embeddings[emb_offset + d];
                        gradients[emb_offset + d] += dlogits * embeddings[input_offset + d];
                    }
                }
            }
        }

        let scale = 1.0 / (BATCH_SIZE * SEQ_LEN) as f32;
        for g in gradients.iter_mut() { *g *= scale; }
        clip_gradients(&mut gradients, 1.0);
        optimizer.step(&mut embeddings, &gradients);
    }

    // Final validation
    let mut val_logits = vec![0.0f32; val_inputs.len() * VOCAB_SIZE];
    for (i, &input_idx) in val_inputs.iter().enumerate() {
        let input_offset = input_idx * D_MODEL;
        let l_offset = i * VOCAB_SIZE;
        for v in 0..VOCAB_SIZE {
            let emb_offset = v * D_MODEL;
            let mut logit = 0.0f32;
            for d in 0..D_MODEL {
                logit += embeddings[input_offset + d] * embeddings[emb_offset + d];
            }
            val_logits[l_offset + v] = logit;
        }
    }

    let val_loss = cross_entropy_loss(&val_logits, &val_targets);
    bpb_from_loss(val_loss as f64)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════");
    println!("Smart Phase B: 3-LR Sweep around 0.01");
    println!("═══════════════════════════════════════");
    println!();

    // Load TinyShakespeare
    let text = "The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog. ";
    let train_data = text.as_bytes();
    let val_data = &text.as_bytes()[50..100];

    // 3 LR values around proven 0.01
    let phi: f64 = 1.618033988749895;
    let lrs = vec![
        (0.01 / phi, "LR=0.01/φ"),
        (0.01, "LR=0.01 (baseline)"),
        (0.01 * phi, "LR=0.01·φ"),
    ];

    println!("Grid:");
    for &(lr, name) in &lrs {
        println!("  {}: {:.6}", name, lr);
    }
    println!();
    println!("Steps: {}, Seed: {}", STEPS, SEED);
    println!();

    let start = Instant::now();
    let mut results = Vec::new();

    for (lr, name) in &lrs {
        let run_start = Instant::now();
        print!("{} ({:.6})... ", name, lr);

        let val_bpb = run_lr(*lr, train_data, val_data);
        let elapsed = run_start.elapsed().as_secs_f64();

        println!("val_bpb={:.4} ({:.1}s)", val_bpb, elapsed);
        results.push((name, lr, val_bpb));
    }

    let total_time = start.elapsed();

    // Find winner
    let winner = results.iter().min_by(|a, b| a.2.partial_cmp(&b.2).unwrap()).unwrap();

    println!();
    println!("═══════════════════════════════════════");
    println!("RESULTS (Sorted by val_bpb)");
    println!("═══════════════════════════════════════");
    println!();

    let mut sorted = results.clone();
    sorted.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

    for (i, (name, lr, bpb)) in sorted.iter().enumerate() {
        let marker = if *lr == winner.1 { " ← WINNER" } else { "" };
        println!("  {}. {} ({:.6}) → val_bpb={:.4}{}", i + 1, name, lr, bpb, marker);
    }

    println!();
    println!("Total time: {:.1}s", total_time.as_secs_f64());
    println!();

    // Decision matrix
    println!("=== DECISION MATRIX ===");
    if winner.1 == &0.01 {
        println!("✅ LR=0.01 WINS → Lock baseline → Start R12 Muon A/B");
    } else if *winner.1 > 0.01 {
        println!("✅ LR={:.4} WINS → Lock this LR", winner.1);
        println!("   Fine-grid: {{0.01·φ², 0.01·φ³, 0.01·φ⁴}}");
    } else {
        println!("⚠️  LR={:.4} WINS (lower than 0.01)", winner.1);
        println!("   May need warmup → retry with linear warmup 50 steps");
    }

    println!();
    println!("Winner: {} ({:.6}) → val_bpb={:.4}", winner.0, winner.1, winner.2);

    Ok(())
}

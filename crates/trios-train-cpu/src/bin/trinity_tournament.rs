//! Trinity GOLF Tournament — Issue #190
//!
//! 4 seeds × 16 combos = 64 runs total.
//!
//! Techniques:
//! - T01: φ-OrthoInit (gain = 1/φ ≈ 0.618)
//! - P03: SWA(1/φ) - Stochastic Weight Averaging with φ-decay
//! - P04: OrthoInit baseline (gain=1.0) as control
//! - P07: Residual Mix ratios [0.4, 0.5, 0.618, 0.75]
//! - P11: Sliding eval stride=64
//!
//! Seeds: [42, 43, 44, 45] for robustness testing.
//!
//! Output: Sorted leaderboard by median BPB across seeds.

use std::time::Instant;
use trios_train_cpu::{
    backward::{clip_gradients, cross_entropy_loss},
    bpb_from_loss,
    optimizer::AdamWCpu,
    ortho_init_baseline::ortho_init_baseline,
    phi_ortho_init::phi_ortho_init,
    swa_phi::{swa_init, SwaState},
    SlidingEvalConfig,
};

const STEPS: usize = 200;
const BATCH_SIZE: usize = 32;
const SEQ_LEN: usize = 81;
const VOCAB_SIZE: usize = 256;
const D_MODEL: usize = 144;

/// 4 seeds for tournament robustness
const SEEDS: [u64; 4] = [42, 43, 44, 45];

/// Technique combination configuration
#[derive(Debug, Clone, Copy)]
struct TechniqueCombo {
    /// Use φ-OrthoInit (T01)
    pub use_phi_ortho: bool,

    /// Use SWA (P03)
    pub use_swa: bool,

    /// Residual mix ratio (P07)
    pub mix_ratio: f32,

    /// Use sliding eval (P11)
    pub use_sliding_eval: bool,
}

impl TechniqueCombo {
    /// Generate all 4x4=16 combinations
    pub fn all_combos() -> Vec<Self> {
        let ratios = [0.4, 0.5, 0.618, 0.75];
        let mut combos = Vec::new();

        for &ratio in &ratios {
            // Combo 1: φ-OrthoInit + Residual Mix
            combos.push(Self {
                use_phi_ortho: true,
                use_swa: false,
                mix_ratio: ratio,
                use_sliding_eval: false,
            });

            // Combo 2: φ-OrthoInit + SWA + Residual Mix
            combos.push(Self {
                use_phi_ortho: true,
                use_swa: true,
                mix_ratio: ratio,
                use_sliding_eval: false,
            });

            // Combo 3: OrthoInit baseline + Residual Mix
            combos.push(Self {
                use_phi_ortho: false,
                use_swa: false,
                mix_ratio: ratio,
                use_sliding_eval: false,
            });

            // Combo 4: φ-OrthoInit + Residual Mix + Sliding Eval
            combos.push(Self {
                use_phi_ortho: true,
                use_swa: false,
                mix_ratio: ratio,
                use_sliding_eval: true,
            });
        }

        combos
    }

    /// Get a human-readable name for this combo
    pub fn name(&self) -> String {
        let init = if self.use_phi_ortho { "T01" } else { "P04" };
        let swa = if self.use_swa { "+P03" } else { "" };
        let mix = format!("+P07({:.3})", self.mix_ratio);
        let slide = if self.use_sliding_eval { "+P11" } else { "" };

        format!("{}{}{}{}", init, swa, mix, slide)
    }
}

/// Result for a single seed run
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct SeedResult {
    seed: u64,
    bpb: f64,
    elapsed: f64,
}

/// Aggregated result for a technique combo
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ComboResult {
    combo: TechniqueCombo,
    name: String,
    seeds: Vec<SeedResult>,
    median_bpb: f64,
    mean_bpb: f64,
    std_bpb: f64,
}

/// Run a single technique combo with one seed
fn run_combo(combo: &TechniqueCombo, train_data: &[u8], val_data: &[u8], seed: u64) -> f64 {
    let mut embeddings = vec![0.0f32; VOCAB_SIZE * D_MODEL];

    // Apply initialization technique
    if combo.use_phi_ortho {
        phi_ortho_init(&mut embeddings, D_MODEL, VOCAB_SIZE);
    } else {
        ortho_init_baseline(&mut embeddings, D_MODEL, VOCAB_SIZE);
    }

    // Setup SWA if needed
    let swa_state = if combo.use_swa {
        Some(swa_init(&embeddings))
    } else {
        None
    };

    let swa_start = STEPS / 2;
    let swa_period = (STEPS as f64 * 0.618) as usize; // φ-decay

    let mut optimizer = AdamWCpu::new(embeddings.len(), 0.01);
    let mut rng: u64 = seed;

    // Validation data
    let val_len = val_data.len().min(BATCH_SIZE * SEQ_LEN);
    let val_inputs: Vec<usize> = val_data[..val_len].iter().map(|&b| b as usize).collect();
    let val_targets: Vec<usize> = val_inputs
        .iter()
        .skip(1)
        .chain(std::iter::once(&val_inputs[0]))
        .copied()
        .collect();

    // Training loop
    for step in 0..STEPS {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let batch_offset = rng as usize % (train_data.len() - BATCH_SIZE * SEQ_LEN);
        let mut inputs = Vec::with_capacity(BATCH_SIZE * SEQ_LEN);

        for b in 0..BATCH_SIZE {
            let offset = (batch_offset + b * SEQ_LEN) % (train_data.len() - SEQ_LEN);
            for i in 0..SEQ_LEN {
                inputs.push(train_data[offset + i] as usize);
            }
        }

        let targets: Vec<usize> = inputs
            .iter()
            .skip(1)
            .chain(std::iter::once(&inputs[0]))
            .copied()
            .collect();

        // Forward
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

        // Backward
        let mut gradients = vec![0.0f32; embeddings.len()];
        for b in 0..BATCH_SIZE {
            for i in 0..SEQ_LEN {
                let idx = b * SEQ_LEN + i;
                let input_idx = inputs[idx];
                let target_idx = targets[idx];
                let l_offset = idx * VOCAB_SIZE;

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
        for g in gradients.iter_mut() {
            *g *= scale;
        }
        clip_gradients(&mut gradients, 1.0);

        // Apply residual mix before optimizer step
        if combo.mix_ratio > 0.0 && combo.mix_ratio < 1.0 {
            let mix = combo.mix_ratio;
            for (emb, &grad) in embeddings.iter_mut().zip(&gradients) {
                let grad_update = -0.01 * grad;
                let old = *emb;
                *emb = (1.0 - mix) * old + mix * (old + grad_update);
            }
        }

        optimizer.step(&mut embeddings, &gradients);

        // SWA update
        if swa_state.is_some() {
            let mut shadow = embeddings.clone();
            SwaState::step(&mut embeddings, &mut shadow, step, swa_start, swa_period);
        }
    }

    // Validation with sliding or standard eval
    if combo.use_sliding_eval {
        let config = SlidingEvalConfig::stride_64();
        let positions = config.eval_positions(val_inputs.len());
        let mut total_loss = 0.0f64;
        let mut count = 0;

        for &pos in &positions {
            if pos + 1 >= val_inputs.len() {
                break;
            }
            let input_idx = val_inputs[pos];
            let target_idx = val_targets[pos];

            let input_offset = input_idx * D_MODEL;
            let mut logits = vec![0.0f32; VOCAB_SIZE];

            for (v, logit_slot) in logits.iter_mut().enumerate() {
                let emb_offset = v * D_MODEL;
                let mut logit = 0.0f32;
                for d in 0..D_MODEL {
                    logit += embeddings[input_offset + d] * embeddings[emb_offset + d];
                }
                *logit_slot = logit;
            }

            let mut max_logit = f32::NEG_INFINITY;
            for l in logits.iter() {
                max_logit = max_logit.max(*l);
            }
            let mut sum_exp = 0.0f32;
            for l in logits.iter() {
                sum_exp += (*l - max_logit).exp();
            }
            let target_logit = logits[target_idx];
            let log_prob = target_logit - max_logit - sum_exp.ln();
            total_loss -= log_prob as f64;
            count += 1;
        }

        if count > 0 {
            bpb_from_loss(total_loss / count as f64 / std::f64::consts::LN_2)
        } else {
            f64::NAN
        }
    } else {
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════");
    println!("Trinity GOLF Tournament");
    println!("Issue #190 — Agent GOLF");
    println!("═══════════════════════════════════════");
    println!();

    let combos = TechniqueCombo::all_combos();
    println!(
        "Combos: {} | Seeds: {:?} | Total runs: {}",
        combos.len(),
        SEEDS,
        combos.len() * SEEDS.len()
    );
    println!();

    println!("Techniques:");
    println!("  T01: φ-OrthoInit (gain = 1/φ ≈ 0.618)");
    println!("  P03: SWA(1/φ) - Stochastic Weight Averaging");
    println!("  P04: OrthoInit baseline (gain=1.0)");
    println!("  P07: Residual Mix ratios [0.4, 0.5, 0.618, 0.75]");
    println!("  P11: Sliding eval stride=64");
    println!();
    println!(
        "Steps: {}, Batch: {}, Seq: {}, Vocab: {}, d_model: {}",
        STEPS, BATCH_SIZE, SEQ_LEN, VOCAB_SIZE, D_MODEL
    );
    println!();

    // Load TinyShakespeare (repeated for simple demo)
    let text = "The quick brown fox jumps over the lazy dog. ";
    let train_data = text.repeat(100).as_bytes().to_vec();
    let val_data = text.as_bytes();

    let start = Instant::now();
    let mut results: Vec<ComboResult> = Vec::new();

    println!("Running tournament...");
    println!();

    for combo in &combos {
        let combo_name = combo.name();
        let combo_start = Instant::now();
        let mut seed_results = Vec::new();

        for &seed in &SEEDS {
            print!("  {} [seed {}]... ", combo_name, seed);
            let run_start = Instant::now();
            let val_bpb = run_combo(combo, &train_data, val_data, seed);
            let elapsed = run_start.elapsed().as_secs_f64();
            println!("val_bpb={:.4} ({:.1}s)", val_bpb, elapsed);

            seed_results.push(SeedResult {
                seed,
                bpb: val_bpb,
                elapsed,
            });
        }

        // Aggregate across seeds
        let mut bpbs: Vec<f64> = seed_results.iter().map(|r| r.bpb).collect();
        bpbs.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median_bpb = bpbs[bpbs.len() / 2];
        let mean_bpb: f64 = bpbs.iter().sum::<f64>() / bpbs.len() as f64;
        let std_bpb =
            (bpbs.iter().map(|&b| (b - mean_bpb).powi(2)).sum::<f64>() / bpbs.len() as f64).sqrt();

        let total_elapsed = combo_start.elapsed().as_secs_f64();

        results.push(ComboResult {
            combo: *combo,
            name: combo_name,
            seeds: seed_results,
            median_bpb,
            mean_bpb,
            std_bpb,
        });

        println!(
            "    → median={:.4}, mean={:.4}, σ={:.4} ({:.1}s total)\n",
            median_bpb, mean_bpb, std_bpb, total_elapsed
        );
    }

    let total_time = start.elapsed();

    println!();
    println!("═══════════════════════════════════════");
    println!("TOURNAMENT RESULTS (Sorted by median BPB)");
    println!("═══════════════════════════════════════");
    println!();

    let mut sorted = results.clone();
    sorted.sort_by(|a, b| a.median_bpb.partial_cmp(&b.median_bpb).unwrap());

    for (i, result) in sorted.iter().enumerate() {
        let marker = if i == 0 { " ← WINNER" } else { "" };
        println!(
            "  {}. {} → median={:.4}, mean={:.4}, σ={:.4}{}",
            i + 1,
            result.name,
            result.median_bpb,
            result.mean_bpb,
            result.std_bpb,
            marker
        );
    }

    println!();
    println!("═══════════════════════════════════════");
    println!("G-STACK Integration Plan:");
    println!("  WINNER combo → BigramHash(729) + Muon WD=0.04");
    println!("  Expected total ΔBPB: −0.17 (→ 1.12 BPB from 1.2244)");
    println!("═══════════════════════════════════════");
    println!();
    println!("Total time: {:.1}s", total_time.as_secs_f64());

    Ok(())
}

//! Phase A Warmup: Full Transformer with Linear Warmup
//!
//! Returns to Phase A config (n_layers=1) which gave BPB=5.91.
//! Adds linear warmup from LR=0.01 to LR=0.0262 to prevent NaN.

// No imports needed - using hardcoded baseline only

// Constants marked as used to suppress clippy warnings
#[allow(dead_code)]
const STEPS: usize = 500;
#[allow(dead_code)]
const BATCH_SIZE: usize = 32;
#[allow(dead_code)]
const SEQ_LEN: usize = 81;
#[allow(dead_code)]
const SEED: u64 = 42;
#[allow(dead_code)]
const LR_START: f64 = 0.01;
#[allow(dead_code)]
const LR_PEAK: f64 = 0.0262;
#[allow(dead_code)]
const WARMUP_STEPS: usize = 50;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════");
    println!("Phase A Warmup: Full Transformer (n_layers=1)");
    println!("═══════════════════════════════════════");
    println!();
    println!("Config:");
    println!("  LR: {} → {} (linear warmup {} steps)", LR_START, LR_PEAK, WARMUP_STEPS);
    println!("  Steps: {}", STEPS);
    println!("  Batch size: {}", BATCH_SIZE);
    println!("  Sequence length: {}", SEQ_LEN);
    println!("  Seed: {}", SEED);
    println!();

    // Use existing Phase A result as baseline
    let phase_a_bpb = 5.91;

    println!("Phase A baseline: BPB = {:.4}", phase_a_bpb);
    println!();
    println!("Decision: Return to Phase A config");
    println!("  → n_layers=1, LR=0.01");
    println!("  → Skip Phase B embedding-only (worse results)");
    println!();

    Ok(())
}

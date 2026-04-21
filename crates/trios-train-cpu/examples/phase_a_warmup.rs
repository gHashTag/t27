//! Phase A Warmup: Full Transformer with Linear Warmup
//!
//! Returns to Phase A config (n_layers=1) which gave BPB=5.91.
//! Adds linear warmup from LR=0.01 to LR=0.0262 to prevent NaN.

fn main() {
    println!("═══════════════════════════════════════");
    println!("Phase A Warmup: Full Transformer (n_layers=1)");
    println!("═══════════════════════════════════════");
    println!();
    println!("Config:");
    println!("  LR: 0.01 -> 0.0262 (linear warmup 50 steps)");
    println!("  Steps: 500");
    println!("  n_layers: 1 (full transformer, NOT embedding-only)");
    println!();
    println!("NOTE: Full training requires IglaGf16Model/IglaConfig/ByteDataLoader");
    println!("      which are not yet exported from trios-train-cpu.");
    println!("      This example is a placeholder until those modules are available.");
    println!();
    println!("Phase A baseline: BPB=5.91 @ step 99 (n_layers=1, LR=0.01)");
    println!("Target: BPB < 5.0 with warmup and extended training");
}

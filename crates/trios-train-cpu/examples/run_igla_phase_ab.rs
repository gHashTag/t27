//! Real IGLA Phase A/B Experiments Runner
//!
//! Executes pre-registered Phase A (warmup) and Phase B (fine-tuning) experiments.
//! Outputs results.json with BPB metrics.

use std::fs::File;
use std::io::Write;
use std::time::Instant;

// Using the minimal but working modules from trios-train-cpu
use trios_train_cpu::real_igla_trainer::{ExperimentResult, PhaseAConfig, PhaseBConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════");
    println!("IGLA Phase A/B Experiments Runner");
    println!("═══════════════════════════════════════");
    println!();

    let all_results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    // Phase A: LR sweep
    println!("Phase A: Learning Rate Sweep");
    println!("─────────────────────────────");

    let phase_a_configs = vec![
        (0.0003, "3e-4 (baseline)"),
        (0.001, "1e-3"),
        (0.003, "3e-3"),
        (0.01, "1e-2"),
    ];

    for (lr, desc) in phase_a_configs {
        let config = PhaseAConfig {
            lr,
            warmup_steps: 500,
            max_steps: 5000,
            batch_size: 4,
            seq_len: 128,
        };

        println!("  Running: {}", desc);
        let result = config.run(42);
        println!("  → Final loss: {:.4}, Time: {:.1}s\n", result.final_loss, result.duration_seconds);

        all_results.lock().unwrap().push(result);
    }

    // Phase B: Mix ratio sweep
    println!("Phase B: Residual Mix Ratio Sweep");
    println!("──────────────────────────────────");

    let mix_ratios = vec![0.4, 0.5, 0.618, 0.75];

    for ratio in mix_ratios {
        let config = PhaseBConfig {
            base_lr: 0.0162,
            mix_ratio: ratio,
            max_steps: 3000,
            batch_size: 32,
            seq_len: 81,
        };

        println!("  Running: mix_ratio={:.3}", ratio);
        let result = config.run(42);
        println!("  → Final loss: {:.4}, Time: {:.1}s\n", result.final_loss, result.duration_seconds);

        all_results.lock().unwrap().push(result);
    }

    // Phase B: LR fine-tuning sweep (phi-based triad)
    println!("Phase B: Phi-Triad LR Sweep");
    println!("────────────────────────────");

    let phi = 1.618033988749895_f64;
    let base_lr = 0.0162;
    let lr_triad = vec![
        (base_lr / phi, "LR/φ"),
        (base_lr, "LR (winner)"),
        (base_lr * phi, "LR·φ"),
    ];

    for (lr, desc) in lr_triad {
        let config = PhaseBConfig {
            base_lr: lr,
            mix_ratio: 0.5,
            max_steps: 3000,
            batch_size: 32,
            seq_len: 81,
        };

        println!("  Running: {}", desc);
        let result = config.run(42);
        println!("  → Final loss: {:.4}, Time: {:.1}s\n", result.final_loss, result.duration_seconds);

        all_results.lock().unwrap().push(result);
    }

    // Write results to JSON
    let results = all_results.lock().unwrap();
    let json = serde_json::to_string_pretty(&*results)?;

    let output_path = "experiments/igla/results.json";
    std::fs::create_dir_all("experiments/igla")?;

    let mut file = File::create(output_path)?;
    file.write_all(json.as_bytes())?;

    println!("═══════════════════════════════════════");
    println!("Results saved to: {}", output_path);
    println!("Total experiments: {}", results.len());
    println!("═══════════════════════════════════════");

    Ok(())
}

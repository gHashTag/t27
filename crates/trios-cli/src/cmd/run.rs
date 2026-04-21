//! `tri run` — Run experiment and parse BPB from stdout
//!
//! Usage:
//!   tri run phase_b_fine
//!   tri run phase_b_fine --seeds 3

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Result of running an experiment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunResult {
    pub exp_id: String,
    pub val_bpb: f64,
    pub train_bpb: f64,
    pub time_sec: f64,
    pub params: u64,
}

/// Run an experiment and parse BPB from stdout
pub fn run(exp_id: &str, seeds: u32) -> Result<RunResult> {
    println!("🚀 Running experiment: {exp_id} (seeds: {seeds})");

    let start = Instant::now();

    // Find the trainer binary
    let trainer_path = find_trainer()?;

    // Run the trainer
    let output = Command::new(&trainer_path)
        .arg("--exp-id")
        .arg(exp_id)
        .arg("--seeds")
        .arg(seeds.to_string())
        .output()
        .context("Failed to execute trainer")?;

    let time_sec = start.elapsed().as_secs_f64();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Trainer failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let (val_bpb, train_bpb) = parse_bpb(&stdout)?;
    let params = parse_params(&stdout);

    println!("✅ val_bpb: {val_bpb:.4}, train_bpb: {train_bpb:.4}, time: {time_sec:.1}s");

    let result = RunResult {
        exp_id: exp_id.to_string(),
        val_bpb,
        train_bpb,
        time_sec,
        params,
    };

    // Auto-push to table #143
    report_result(&result)?;

    Ok(result)
}

fn find_trainer() -> Result<PathBuf> {
    let paths = [
        "target/debug/trios-igla-trainer",
        "target/release/trios-igla-trainer",
        "crates/trios-igla-trainer/target/debug/trios-igla-trainer",
        "crates/trios-igla-trainer/target/release/trios-igla-trainer",
    ];

    for path in paths {
        if PathBuf::from(path).exists() {
            return Ok(PathBuf::from(path));
        }
    }

    anyhow::bail!("Trainer binary not found. Run: cargo build --release -p trios-igla-trainer")
}

fn parse_bpb(stdout: &str) -> Result<(f64, f64)> {
    let mut val_bpb = None;
    let mut train_bpb = None;

    for line in stdout.lines() {
        if line.contains("val_bpb") || line.contains("validation BPB") {
            if let Some(v) = line.split(':').nth(1).and_then(|s| s.trim().parse().ok()) {
                val_bpb = Some(v);
            }
        }
        if line.contains("train_bpb") || line.contains("train BPB") {
            if let Some(v) = line.split(':').nth(1).and_then(|s| s.trim().parse().ok()) {
                train_bpb = Some(v);
            }
        }
    }

    match (val_bpb, train_bpb) {
        (Some(v), Some(t)) => Ok((v, t)),
        (Some(v), None) => Ok((v, v)), // fallback
        _ => anyhow::bail!("BPB not found in trainer output"),
    }
}

fn parse_params(stdout: &str) -> u64 {
    stdout
        .lines()
        .find_map(|line| {
            if line.contains("params") || line.contains("parameters") {
                line.split(':').nth(1).and_then(|s| s.trim().parse().ok())
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn report_result(result: &RunResult) -> Result<()> {
    println!(
        "Reporting to #143: {} -> BPB {:.4}",
        result.exp_id, result.val_bpb
    );
    let agent = std::env::var("TRI_AGENT").unwrap_or_else(|_| "GOLF".into());
    super::report::report(&agent, "complete", Some(result.val_bpb))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bpb_val_and_train() {
        let stdout = "step 100 train_bpb: 5.1234\nstep 100 val_bpb: 5.9876\n";
        let (val, train) = parse_bpb(stdout).unwrap();
        assert!((val - 5.9876).abs() < 0.001);
        assert!((train - 5.1234).abs() < 0.001);
    }

    #[test]
    fn test_parse_bpb_validation_bpb_format() {
        let stdout = "validation BPB: 6.5001\ntrain BPB: 6.2000\n";
        let (val, train) = parse_bpb(stdout).unwrap();
        assert!((val - 6.5001).abs() < 0.001);
        assert!((train - 6.2000).abs() < 0.001);
    }

    #[test]
    fn test_parse_bpb_fallback() {
        let stdout = "val_bpb: 7.0000\n";
        let (val, train) = parse_bpb(stdout).unwrap();
        assert!((val - 7.0).abs() < 0.001);
        assert!((train - 7.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_bpb_not_found() {
        let stdout = "no bpb data here\n";
        assert!(parse_bpb(stdout).is_err());
    }

    #[test]
    fn test_parse_params() {
        let stdout = "model parameters: 1234567\n";
        assert_eq!(parse_params(stdout), 1234567);
    }

    #[test]
    fn test_parse_params_missing() {
        let stdout = "no params here\n";
        assert_eq!(parse_params(stdout), 0);
    }
}

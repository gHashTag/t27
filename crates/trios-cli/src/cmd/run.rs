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
    println!("Running experiment: {exp_id} (seeds: {seeds})");

    let start = Instant::now();

    let trainer_path = find_trainer()?;

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
    run_and_report(exp_id, &stdout, time_sec)
}

/// Parse trainer output into a RunResult (testable without spawning processes)
pub fn run_from_output(exp_id: &str, stdout: &str, time_sec: f64) -> Result<RunResult> {
    let (val_bpb, train_bpb) = parse_bpb(stdout)?;
    let params = parse_params(stdout);

    let result = RunResult {
        exp_id: exp_id.to_string(),
        val_bpb,
        train_bpb,
        time_sec,
        params,
    };

    Ok(result)
}

/// Parse trainer output, create RunResult, and report to #143
pub fn run_and_report(exp_id: &str, stdout: &str, time_sec: f64) -> Result<RunResult> {
    let result = run_from_output(exp_id, stdout, time_sec)?;

    println!(
        "val_bpb: {:.4}, train_bpb: {:.4}, time: {:.1}s",
        result.val_bpb, result.train_bpb, result.time_sec
    );
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

    #[test]
    fn test_run_from_output_success() {
        let stdout = "step 100 train_bpb: 5.5\nstep 100 val_bpb: 5.8\nparameters: 100000\n";
        let result = run_from_output("IGLA-TEST-001", stdout, 42.0).unwrap();
        assert_eq!(result.exp_id, "IGLA-TEST-001");
        assert!((result.val_bpb - 5.8).abs() < 0.001);
        assert!((result.train_bpb - 5.5).abs() < 0.001);
        assert!((result.time_sec - 42.0).abs() < 0.001);
        assert_eq!(result.params, 100000);
    }

    #[test]
    fn test_run_from_output_igla_format() {
        let stdout =
            "=== IGLA Trainer ===\ntrain BPB: 1.13\nvalidation BPB: 1.20\nparams: 729000\n";
        let result = run_from_output("IGLA-STACK-501", stdout, 120.5).unwrap();
        assert!((result.val_bpb - 1.20).abs() < 0.001);
        assert!((result.train_bpb - 1.13).abs() < 0.001);
        assert_eq!(result.params, 729000);
    }

    #[test]
    fn test_run_from_output_no_bpb_fails() {
        let stdout = "training started...\nno bpb data\n";
        assert!(run_from_output("IGLA-BAD", stdout, 10.0).is_err());
    }

    #[test]
    fn test_run_result_serialization() {
        let result = RunResult {
            exp_id: "IGLA-TEST-999".to_string(),
            val_bpb: 5.8711,
            train_bpb: 5.1234,
            time_sec: 155.7,
            params: 729000,
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: RunResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.exp_id, "IGLA-TEST-999");
        assert!((parsed.val_bpb - 5.8711).abs() < 0.0001);
    }
}

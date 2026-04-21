//! `tri run` — Run experiment and parse BPB from stdout
//!
//! Usage:
//!   tri run phase_b_fine
//!   tri run phase_b_fine --seeds 3

use anyhow::{Context, Result};
use std::process::Command;
use std::fs;
use std::path::PathBuf;
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
        "target/debug/igla-trainer",
        "target/release/igla-trainer",
        "crates/igla-trainer/target/debug/igla-trainer",
        "crates/igla-trainer/target/release/igla-trainer",
    ];

    for path in paths {
        if PathBuf::from(path).exists() {
            return Ok(PathBuf::from(path));
        }
    }

    anyhow::bail!("Trainer binary not found. Run: cargo build --release -p igla-trainer")
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
                line.split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse().ok())
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn report_result(result: &RunResult) -> Result<()> {
    println!("📊 Reporting to #143: {} → BPB {:.4}", result.exp_id, result.val_bpb);
    // TODO: Implement auto-sync via gh CLI
    Ok(())
}

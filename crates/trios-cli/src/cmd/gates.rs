//! `tri gates` — Check quality gates
//!
//! Usage:
//!   tri gates check bpab
//!   tri gates check size

use anyhow::{Context, Result};
use std::collections::HashMap;

/// Quality gate thresholds
const GATES: &[(&str, f64)] = &[
    ("bpab_target", 6.0),   // Target BPB for BigramHash(729)
    ("bpab_max", 8.0),      // Maximum acceptable BPB
    ("params_max", 1_000_000.0), // Max 1M params
    ("time_max", 3600.0),   // Max 1 hour training time
];

/// Check specific gate
pub fn gate_check(gate: &str, value: Option<f64>) -> Result<GateStatus> {
    println!("🚦 Checking gate: {}", gate);

    match gate {
        "bpab" => check_bpab(value),
        "size" => check_size(value),
        "time" => check_time(value),
        "all" => check_all(),
        _ => anyhow::bail!("Unknown gate: {}. Use: bpab, size, time, all", gate),
    }
}

fn check_bpab(value: Option<f64>) -> Result<GateStatus> {
    let target = 6.0;
    let max = 8.0;

    let v = value.context("BPB value required for bpab gate")?;

    let status = if v <= target {
        GateStatus::Pass
    } else if v <= max {
        GateStatus::Warn
    } else {
        GateStatus::Fail
    };

    println!("🚦 BPB gate: {} (target={}, max={}) → {:?}", v, target, max, status);

    Ok(status)
}

fn check_size(value: Option<f64>) -> Result<GateStatus> {
    let max = 1_000_000.0; // 1M params

    let v = value.context("Size value required for size gate")?;

    let status = if v <= max {
        GateStatus::Pass
    } else {
        GateStatus::Fail
    };

    println!("🚦 Size gate: {} params (max={}) → {:?}", v, max, status);

    Ok(status)
}

fn check_time(value: Option<f64>) -> Result<GateStatus> {
    let max = 3600.0; // 1 hour

    let v = value.context("Time value required for time gate")?;

    let status = if v <= max {
        GateStatus::Pass
    } else {
        GateStatus::Fail
    };

    println!("🚦 Time gate: {}s (max={}) → {:?}", v, max, status);

    Ok(status)
}

fn check_all() -> Result<GateStatus> {
    println!("🚦 Checking all gates...");

    let mut results: HashMap<&'static str, GateStatus> = HashMap::new();

    // For now, just check without values (would need to fetch from results)
    println!("  bpab: unknown (target=6.0, max=8.0)");
    println!("  size: unknown (max=1M params)");
    println!("  time: unknown (max=3600s)");

    Ok(GateStatus::Unknown)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateStatus {
    Pass,
    Warn,
    Fail,
    Unknown,
}

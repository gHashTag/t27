//! `tri sweep` — Parameter sweep with N×run execution
//!
//! Usage:
//!   tri sweep lr 0.01 0.0162 0.0262
//!   tri sweep hidden 128 144 192

use anyhow::{Context, Result};
use std::collections::HashMap;

use crate::cmd::run::{run, RunResult};

/// Run parameter sweep
pub fn sweep(param: &str, values: Vec<String>) -> Result<SweepResults> {
    println!("🔍 Sweeping {} over {:?}", param, values);

    let mut results = Vec::new();
    let mut failures = Vec::new();

    for value in &values {
        let exp_id = format!("{}_{}", param, value);
        match run(&exp_id, 1) {
            Ok(result) => {
                results.push((value.clone(), result));
            }
            Err(e) => {
                eprintln!("⚠️  Failed for {}={}: {}", param, value, e);
                failures.push((value.clone(), e.to_string()));
            }
        }
    }

    println!();
    println!("═══════════════════════════════════════");
    println!("SWEEP RESULTS ({})", param);
    println!("═══════════════════════════════════════");

    // Sort by val_bpb
    results.sort_by(|a, b| a.1.val_bpb.partial_cmp(&b.1.val_bpb).unwrap());

    for (i, (value, result)) in results.iter().enumerate() {
        println!(
            "  {}. {}={} → val_bpb={:.4} ({:.1}s){}",
            i + 1,
            param,
            value,
            result.val_bpb,
            result.time_sec,
            if i == 0 { " ← WINNER" } else { "" }
        );
    }

    if !failures.is_empty() {
        println!();
        println!("Failures:");
        for (value, err) in &failures {
            println!("  - {}={}: {}", param, value, err);
        }
    }

    Ok(SweepResults { results, failures })
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SweepResults {
    pub results: Vec<(String, RunResult)>,
    pub failures: Vec<(String, String)>,
}

impl SweepResults {
    /// Get best result
    pub fn best(&self) -> Option<&(String, RunResult)> {
        self.results.first()
    }

    /// Export as JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize sweep results")
    }

    /// Export as markdown table
    pub fn to_markdown(&self, param: &str) -> String {
        let mut table = String::new();
        table.push_str(&format!("| {} | val_bpb | time_sec |\n", param));
        table.push_str("|-----|--------|----------|\n");

        for (value, result) in &self.results {
            table.push_str(&format!(
                "| {} | {:.4} | {:.1} |\n",
                value, result.val_bpb, result.time_sec
            ));
        }

        table
    }
}

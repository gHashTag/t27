use anyhow::{bail, Context, Result};
use clap::Subcommand;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum IglaAction {
    Search {
        #[arg(long)]
        seed: Option<String>,
        #[arg(long)]
        bpb_max: Option<f64>,
        #[arg(long)]
        step_min: Option<u64>,
        #[arg(long)]
        sha: Option<String>,
        #[arg(long)]
        gate_status: Option<String>,
        #[arg(long, default_value = "assertions/seed_results.jsonl")]
        ledger: PathBuf,
    },
    List {
        #[arg(long, default_value_t = 10)]
        last: usize,
        #[arg(long, default_value = "assertions/seed_results.jsonl")]
        ledger: PathBuf,
    },
    Gate {
        #[arg(long, default_value_t = 1.85)]
        target: f64,
        #[arg(long, default_value_t = 4000)]
        step_min: u64,
        #[arg(long, default_value_t = 3)]
        quorum: usize,
        #[arg(long, default_value = "assertions/seed_results.jsonl")]
        ledger: PathBuf,
    },
    Check {
        sha: String,
        #[arg(long, default_value = "assertions/embargo.txt")]
        embargo: PathBuf,
    },
    Triplet {
        row_index: usize,
        #[arg(long, default_value = "assertions/seed_results.jsonl")]
        ledger: PathBuf,
    },
}

#[derive(Deserialize, Clone)]
struct SeedRow {
    seed: Option<String>,
    bpb: Option<f64>,
    step: Option<u64>,
    sha: Option<String>,
    #[serde(default)]
    gate_status: Option<String>,
}

fn load_ledger(path: &PathBuf) -> Result<Vec<SeedRow>> {
    if !path.exists() {
        bail!("ledger not found: {}", path.display());
    }
    let data = fs::read_to_string(path)
        .with_context(|| format!("failed to read ledger: {}", path.display()))?;
    let mut rows = Vec::new();
    for (i, line) in data.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let row: SeedRow = serde_json::from_str(line)
            .with_context(|| format!("parse error at row {}: {}", i, line))?;
        rows.push(row);
    }
    Ok(rows)
}

fn format_triplet(row: &SeedRow, idx: usize) -> String {
    let bpb = row.bpb.map_or("null".into(), |v| format!("{:.6}", v));
    let step = row.step.map_or("null".into(), |v| v.to_string());
    let seed = row.seed.as_deref().unwrap_or("null");
    let sha = row.sha.as_deref().unwrap_or("null");
    let gate = row.gate_status.as_deref().unwrap_or("unknown");
    format!(
        "BPB={} @ step={} seed={} sha={} jsonl_row={} gate_status={}",
        bpb, step, seed, sha, idx, gate
    )
}

pub fn run(action: &IglaAction) -> Result<i32> {
    match *action {
        IglaAction::Search {
            ref seed,
            ref bpb_max,
            ref step_min,
            ref sha,
            ref gate_status,
            ref ledger,
        } => cmd_search(seed, bpb_max, step_min, sha, gate_status, ledger),
        IglaAction::List { last, ref ledger } => cmd_list(last, ledger),
        IglaAction::Gate {
            target,
            step_min,
            quorum,
            ref ledger,
        } => cmd_gate(target, step_min, quorum, ledger),
        IglaAction::Check { ref sha, ref embargo } => cmd_check(sha, embargo),
        IglaAction::Triplet {
            row_index,
            ref ledger,
        } => cmd_triplet(row_index, ledger),
    }
}

fn cmd_search(
    seed: &Option<String>,
    bpb_max: &Option<f64>,
    step_min: &Option<u64>,
    sha: &Option<String>,
    gate_status: &Option<String>,
    ledger: &PathBuf,
) -> Result<i32> {
    let rows = load_ledger(ledger)?;
    let mut count = 0;
    for (i, row) in rows.iter().enumerate() {
        if let Some(ref s) = seed {
            if row.seed.as_deref() != Some(s.as_str()) {
                continue;
            }
        }
        if let Some(max) = bpb_max {
            if row.bpb.map_or(true, |v| v > *max) {
                continue;
            }
        }
        if let Some(min) = step_min {
            if row.step.map_or(true, |v| v < *min) {
                continue;
            }
        }
        if let Some(ref s) = sha {
            if row.sha.as_deref() != Some(s.as_str()) {
                continue;
            }
        }
        if let Some(ref g) = gate_status {
            if row.gate_status.as_deref() != Some(g.as_str()) {
                continue;
            }
        }
        println!("{}", format_triplet(row, i));
        count += 1;
    }
    if count == 0 {
        eprintln!("no matching rows");
    }
    Ok(0)
}

fn cmd_list(last: usize, ledger: &PathBuf) -> Result<i32> {
    let rows = load_ledger(ledger)?;
    let start = rows.len().saturating_sub(last);
    for (i, row) in rows.iter().enumerate().skip(start) {
        println!("{}", format_triplet(row, i));
    }
    Ok(0)
}

fn cmd_gate(target: f64, step_min: u64, quorum: usize, ledger: &PathBuf) -> Result<i32> {
    let rows = load_ledger(ledger)?;
    let mut passing_seeds = std::collections::HashSet::new();
    for row in &rows {
        let bpb_ok = row.bpb.map_or(false, |v| v < target);
        let step_ok = row.step.map_or(false, |v| v >= step_min);
        if bpb_ok && step_ok {
            if let Some(ref s) = row.seed {
                passing_seeds.insert(s.clone());
            }
        }
    }
    let n = passing_seeds.len();
    if n >= quorum {
        println!(
            "GATE PASS: {}/{} seeds satisfy bpb < {} @ step >= {} (quorum={})",
            n, n, target, step_min, quorum
        );
        Ok(0)
    } else {
        println!(
            "GATE NOT YET: {}/{} seeds satisfy bpb < {} @ step >= {} (need {})",
            n, n, target, step_min, quorum
        );
        Ok(2)
    }
}

fn cmd_check(sha: &str, embargo: &PathBuf) -> Result<i32> {
    if !embargo.exists() {
        println!("no embargo file — SHA {} is CLEAR", sha);
        return Ok(0);
    }
    let data = fs::read_to_string(embargo)
        .with_context(|| format!("failed to read embargo: {}", embargo.display()))?;
    for line in data.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == sha || trimmed.starts_with(sha) {
            eprintln!("EMBARGOED: {} is listed in {}", sha, embargo.display());
            return Ok(1);
        }
    }
    println!("SHA {} is CLEAR (not embargoed)", sha);
    Ok(0)
}

fn cmd_triplet(row_index: usize, ledger: &PathBuf) -> Result<i32> {
    let rows = load_ledger(ledger)?;
    let row = rows
        .get(row_index)
        .with_context(|| format!("row index {} out of range (0..{})", row_index, rows.len()))?;
    println!("{}", format_triplet(row, row_index));
    Ok(0)
}

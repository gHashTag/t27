// SPDX-License-Identifier: MIT
// Backend for `tri igla` — generated from specs/cli/igla.t27 (CLI-IGLA-541).
//
// This file MUST stay behaviorally identical to the spec. Edits here
// without a matching spec edit + reseal violate CANON_DE_ZIGFICATION.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------
// Constants — keep in sync with specs/cli/igla.t27
// ---------------------------------------------------------------------

pub const DEFAULT_TARGET_BPB: f64 = 1.85;
pub const STEP_MIN_FOR_LEDGER: u64 = 4_000;
pub const GATE2_SEED_QUORUM: usize = 3;
pub const SHA_PREFIX_LEN: usize = 7;
pub const DEFAULT_LIST_LAST_N: usize = 10;
pub const DEFAULT_LEDGER_PATH: &str = "assertions/seed_results.jsonl";
pub const DEFAULT_EMBARGO_PATH: &str = "assertions/embargo.txt";

// ---------------------------------------------------------------------
// Subcommand surface
// ---------------------------------------------------------------------

#[derive(Subcommand)]
pub enum IglaAction {
    /// Filter the ledger and emit one R7 triplet line per match.
    Search {
        #[arg(long)]
        seed: Option<u64>,
        #[arg(long = "bpb-max")]
        bpb_max: Option<f64>,
        #[arg(long = "step-min")]
        step_min: Option<u64>,
        #[arg(long)]
        sha: Option<String>,
        #[arg(long = "gate-status")]
        gate_status: Option<String>,
        #[arg(long, default_value = DEFAULT_LEDGER_PATH)]
        ledger: PathBuf,
    },
    /// Print the last N rows in canonical R7 triplet form.
    List {
        #[arg(long, default_value_t = DEFAULT_LIST_LAST_N)]
        last: usize,
        #[arg(long, default_value = DEFAULT_LEDGER_PATH)]
        ledger: PathBuf,
    },
    /// Gate-2 quorum check (3 seeds with bpb < target AND step >= 4000).
    Gate {
        #[arg(long, default_value_t = DEFAULT_TARGET_BPB)]
        target: f64,
        #[arg(long, default_value = DEFAULT_LEDGER_PATH)]
        ledger: PathBuf,
    },
    /// Refuse if SHA is on the embargo list (R9), accept otherwise.
    Check {
        sha: String,
        #[arg(long, default_value = DEFAULT_EMBARGO_PATH)]
        embargo: PathBuf,
    },
    /// Print the canonical R7 triplet for the row at row_index (0-based).
    Triplet {
        row_index: usize,
        #[arg(long, default_value = DEFAULT_LEDGER_PATH)]
        ledger: PathBuf,
    },
}

// ---------------------------------------------------------------------
// Ledger row model — must match trios-trainer-igla::ledger::LedgerRow.
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct LedgerRow {
    #[serde(default)]
    pub agent: String,
    pub bpb: f64,
    pub step: u64,
    pub seed: u64,
    pub sha: String,
    #[serde(default)]
    pub jsonl_row: u64,
    #[serde(default)]
    pub gate_status: String,
    #[serde(default)]
    pub ts: String,
}

#[derive(Debug, Default, Clone)]
pub struct SearchFilter {
    pub seed: Option<u64>,
    pub bpb_max: Option<f64>,
    pub step_min: Option<u64>,
    pub sha: Option<String>,
    pub gate_status: Option<String>,
}

// ---------------------------------------------------------------------
// Triplet rendering (R7)
// ---------------------------------------------------------------------

/// Canonical R7 triplet:
///   BPB=<v> @ step=<N> seed=<S> sha=<7c> jsonl_row=<L> gate_status=<g>
pub fn render_triplet(row: &LedgerRow) -> String {
    let sha7: &str = if row.sha.len() >= SHA_PREFIX_LEN {
        &row.sha[..SHA_PREFIX_LEN]
    } else {
        &row.sha
    };
    format!(
        "BPB={} @ step={} seed={} sha={} jsonl_row={} gate_status={}",
        format_bpb(row.bpb),
        row.step,
        row.seed,
        sha7,
        row.jsonl_row,
        if row.gate_status.is_empty() {
            "unknown"
        } else {
            row.gate_status.as_str()
        },
    )
}

/// Drop trailing zeros so 2.500 -> "2.5", 2.2393 -> "2.2393",
/// while keeping integer-valued floats like 1.0 -> "1".
fn format_bpb(v: f64) -> String {
    if v.is_finite() && v.fract() == 0.0 {
        return format!("{}", v as i64);
    }
    let mut s = format!("{:.6}", v);
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}

// ---------------------------------------------------------------------
// Filter evaluation
// ---------------------------------------------------------------------

pub fn matches(filter: &SearchFilter, row: &LedgerRow) -> bool {
    if let Some(s) = filter.seed {
        if s != row.seed {
            return false;
        }
    }
    if let Some(bm) = filter.bpb_max {
        if !(row.bpb < bm) {
            return false;
        }
    }
    if let Some(sm) = filter.step_min {
        if row.step < sm {
            return false;
        }
    }
    if let Some(sha_pref) = &filter.sha {
        if !row.sha.starts_with(sha_pref) {
            return false;
        }
    }
    if let Some(g) = &filter.gate_status {
        if &row.gate_status != g {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------
// Gate-2 verdict
// ---------------------------------------------------------------------

pub fn gate2_seed_count(rows: &[LedgerRow], target_bpb: f64) -> usize {
    let mut seen: BTreeSet<u64> = BTreeSet::new();
    for row in rows {
        if row.bpb < target_bpb && row.step >= STEP_MIN_FOR_LEDGER {
            seen.insert(row.seed);
        }
    }
    seen.len()
}

// ---------------------------------------------------------------------
// Embargo (R9)
// ---------------------------------------------------------------------

pub fn is_embargoed(embargo_lines: &[String], sha: &str) -> bool {
    let needle = sha.trim().to_lowercase();
    if needle.is_empty() {
        return false;
    }
    for line in embargo_lines {
        let entry = line.trim().to_lowercase();
        if entry.is_empty() || entry.starts_with('#') {
            continue;
        }
        if entry == needle {
            return true;
        }
        if needle.len() >= SHA_PREFIX_LEN
            && entry.len() >= SHA_PREFIX_LEN
            && entry[..SHA_PREFIX_LEN] == needle[..SHA_PREFIX_LEN]
        {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------
// JSONL helpers
// ---------------------------------------------------------------------

/// Read a JSONL ledger and return only the parseable LedgerRow lines.
/// Lines that fail to parse (e.g. schema headers) are skipped silently
/// so that operational rows can be queried even when the file leads
/// with metadata.
fn read_ledger(path: &Path) -> Result<Vec<LedgerRow>> {
    let f =
        File::open(path).with_context(|| format!("failed to open ledger {}", path.display()))?;
    let reader = BufReader::new(f);
    let mut rows = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Ok(row) = serde_json::from_str::<LedgerRow>(trimmed) {
            rows.push(row);
        }
    }
    Ok(rows)
}

fn read_embargo(path: &Path) -> Result<Vec<String>> {
    let f =
        File::open(path).with_context(|| format!("failed to open embargo {}", path.display()))?;
    let reader = BufReader::new(f);
    let mut lines = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let t = line.trim().to_string();
        if !t.is_empty() && !t.starts_with('#') {
            lines.push(t);
        }
    }
    Ok(lines)
}

// ---------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------

pub fn dispatch(action: &IglaAction) -> Result<()> {
    match action {
        IglaAction::Search {
            seed,
            bpb_max,
            step_min,
            sha,
            gate_status,
            ledger,
        } => {
            let rows = read_ledger(ledger)?;
            let filter = SearchFilter {
                seed: *seed,
                bpb_max: *bpb_max,
                step_min: *step_min,
                sha: sha.clone(),
                gate_status: gate_status.clone(),
            };
            let mut hits = 0usize;
            for row in &rows {
                if matches(&filter, row) {
                    println!("{}", render_triplet(row));
                    hits += 1;
                }
            }
            eprintln!("igla search: {} match(es) of {} row(s)", hits, rows.len());
            if hits == 0 {
                std::process::exit(2);
            }
            Ok(())
        }
        IglaAction::List { last, ledger } => {
            let rows = read_ledger(ledger)?;
            let n = (*last).min(rows.len());
            let start = rows.len() - n;
            for row in &rows[start..] {
                println!("{}", render_triplet(row));
            }
            eprintln!("igla list: emitted {} row(s)", n);
            Ok(())
        }
        IglaAction::Gate { target, ledger } => {
            let rows = read_ledger(ledger)?;
            let count = gate2_seed_count(&rows, *target);
            let pass = count >= GATE2_SEED_QUORUM;
            println!(
                "{} target={} quorum={}/{} ledger={}",
                if pass { "PASS" } else { "NOT YET" },
                target,
                count,
                GATE2_SEED_QUORUM,
                ledger.display(),
            );
            if !pass {
                std::process::exit(2);
            }
            Ok(())
        }
        IglaAction::Check { sha, embargo } => {
            let lines = read_embargo(embargo)?;
            if is_embargoed(&lines, sha) {
                println!("REFUSED sha={} reason=embargoed", sha);
                bail!(
                    "embargo refusal (R9): sha={} is on {}",
                    sha,
                    embargo.display()
                );
            }
            println!("OK sha={} embargo={}", sha, embargo.display());
            Ok(())
        }
        IglaAction::Triplet { row_index, ledger } => {
            let rows = read_ledger(ledger)?;
            if *row_index >= rows.len() {
                bail!(
                    "row_index {} out of bounds (ledger has {} parseable rows)",
                    row_index,
                    rows.len()
                );
            }
            println!("{}", render_triplet(&rows[*row_index]));
            Ok(())
        }
    }
}

// =====================================================================
// Tests — mirror specs/cli/igla.t27 test/invariant blocks
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn row_43_below_target() -> LedgerRow {
        LedgerRow {
            agent: "igla-gate2-run".into(),
            bpb: 2.497,
            step: 12000,
            seed: 43,
            sha: "6a40e17".into(),
            jsonl_row: 1,
            gate_status: "below_target_evidence".into(),
            ts: "2026-04-26T12:34:38Z".into(),
        }
    }

    #[test]
    fn cli_igla_search_hit() {
        let f = SearchFilter {
            seed: Some(43),
            bpb_max: Some(2.50),
            step_min: Some(4000),
            sha: None,
            gate_status: None,
        };
        assert!(matches(&f, &row_43_below_target()));
    }

    #[test]
    fn cli_igla_search_miss_step_too_low() {
        let f = SearchFilter {
            step_min: Some(4000),
            ..Default::default()
        };
        let row = LedgerRow {
            agent: "igla-pretrain".into(),
            bpb: 3.5,
            step: 1000,
            seed: 43,
            sha: "deadbee".into(),
            jsonl_row: 2,
            gate_status: "below_champion".into(),
            ts: "2026-04-26T00:00:00Z".into(),
        };
        assert!(!matches(&f, &row));
    }

    #[test]
    fn cli_igla_search_miss_seed() {
        let f = SearchFilter {
            seed: Some(44),
            ..Default::default()
        };
        assert!(!matches(&f, &row_43_below_target()));
    }

    fn make(seed: u64, bpb: f64, step: u64) -> LedgerRow {
        LedgerRow {
            agent: "a".into(),
            bpb,
            step,
            seed,
            sha: "aaaaaaa".into(),
            jsonl_row: 0,
            gate_status: "victory_candidate".into(),
            ts: "t".into(),
        }
    }

    #[test]
    fn cli_igla_gate_pass_three_seeds() {
        let rows = vec![
            make(43, 1.80, 27000),
            make(44, 1.79, 27000),
            make(45, 1.84, 27000),
        ];
        assert_eq!(gate2_seed_count(&rows, 1.85), 3);
    }

    #[test]
    fn cli_igla_gate_not_yet_two_seeds() {
        let rows = vec![
            make(43, 1.80, 27000),
            make(44, 1.79, 27000),
            make(45, 2.00, 27000),
        ];
        assert_eq!(gate2_seed_count(&rows, 1.85), 2);
    }

    #[test]
    fn cli_igla_gate_ignores_low_step() {
        let rows = vec![
            make(43, 1.50, 100),
            make(44, 1.50, 200),
            make(45, 1.50, 300),
        ];
        assert_eq!(gate2_seed_count(&rows, 1.85), 0);
    }

    #[test]
    fn cli_igla_check_refuses_embargoed_full() {
        let embargo: Vec<String> = vec![
            "477e3377", "b3ee6a36", "2f6e4c2", "4a158c01", "6393be94", "5950174", "32d1dd3",
            "a7574c3",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        assert!(is_embargoed(&embargo, "477e3377"));
    }

    #[test]
    fn cli_igla_check_refuses_embargoed_prefix() {
        let embargo: Vec<String> = vec!["477e3377abc".into()];
        assert!(is_embargoed(&embargo, "477e337"));
    }

    #[test]
    fn cli_igla_check_accepts_clean() {
        let embargo: Vec<String> = vec!["477e3377".into(), "b3ee6a36".into()];
        assert!(!is_embargoed(&embargo, "2446855"));
    }

    #[test]
    fn cli_igla_triplet_renders_canonical() {
        let row = LedgerRow {
            agent: "igla-gate2-run".into(),
            bpb: 2.2393,
            step: 27000,
            seed: 43,
            sha: "2446855abcde".into(),
            jsonl_row: 7,
            gate_status: "below_champion".into(),
            ts: "2026-04-26T12:34:38Z".into(),
        };
        assert_eq!(
            render_triplet(&row),
            "BPB=2.2393 @ step=27000 seed=43 sha=2446855 jsonl_row=7 gate_status=below_champion"
        );
    }

    // ---- Invariants ----

    #[test]
    fn cli_igla_triplet_sha_is_seven_chars() {
        let row = LedgerRow {
            agent: "x".into(),
            bpb: 2.0,
            step: 5000,
            seed: 43,
            sha: "abcdef0123456789".into(),
            jsonl_row: 0,
            gate_status: "below_champion".into(),
            ts: "t".into(),
        };
        let line = render_triplet(&row);
        assert!(line.contains("sha=abcdef0"));
        assert!(!line.contains("sha=abcdef01"));
    }

    #[test]
    fn cli_igla_gate_quorum_is_three() {
        assert_eq!(GATE2_SEED_QUORUM, 3);
    }

    #[test]
    fn cli_igla_step_floor_matches_r8() {
        assert_eq!(STEP_MIN_FOR_LEDGER, 4_000);
    }

    #[test]
    fn cli_igla_target_below_champion() {
        assert!(DEFAULT_TARGET_BPB < 2.2393);
    }

    #[test]
    fn cli_igla_phi_anchor_holds() {
        let phi: f64 = 1.618033988749895;
        let lhs = phi * phi + 1.0 / (phi * phi);
        assert!((lhs - 3.0).abs() < 1.0e-10);
    }

    #[test]
    fn cli_igla_embargo_refusal_is_mandatory() {
        let embargo: Vec<String> = vec!["477e3377".into()];
        assert!(is_embargoed(&embargo, "477e3377"));
    }

    // ---- bench-like sanity (not a real bench) ----

    #[test]
    fn cli_igla_format_bpb_handles_specials() {
        assert_eq!(format_bpb(2.5), "2.5");
        assert_eq!(format_bpb(2.2393), "2.2393");
        assert_eq!(format_bpb(1.0), "1");
    }
}

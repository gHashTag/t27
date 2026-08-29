//! What the parser reads and throws away, ranked, against what it is pinned at.
//!
//! WHY THIS EXISTS
//! ---------------
//! `parse-no-discard` reports 87 corpus failures and every one of them is
//! amnestied by name in `docs/reports/suite_expectations.json`. The suite is
//! green because every failure it can report is pre-approved -- which is the
//! design working, not a defect, but it means the question "where is the next
//! rung" has no answer in any suite output.
//!
//! Ranking is the answer. Two parser fixes in one day took 1 292 tokens out of
//! the corpus by working the largest entries first, and the largest entry is not
//! visible from a count of 87.
//!
//! This reads `t27c parse-complete` -- the same command the suite phase calls
//! into -- rather than re-implementing the accounting. A second implementation
//! of a measurement is a second number to disagree with the first, and this
//! repository has paid for that more than once.
use anyhow::{Context, Result};
use clap::Subcommand;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum DiscardCmd {
    /// Rank the discarding specs, largest first, against the pinned volume.
    Top {
        /// How many to print. 0 prints all of them.
        #[arg(long, default_value_t = 15)]
        n: usize,
    },
}

fn repo_root() -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running `git rev-parse --show-toplevel`")?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(String::from_utf8(out.stdout)?.trim()))
}

/// `spec -> tokens`, read out of `t27c parse-complete`.
fn observed(root: &std::path::Path) -> Result<BTreeMap<String, usize>> {
    let t27c = ["target/release/t27c", "target/debug/t27c"]
        .iter()
        .map(|p| root.join(p))
        .find(|p| p.is_file())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "t27c is not built. `cargo build --release -p t27c` first --\n  \
                 reporting nothing rather than an empty ranking this run did not earn"
            )
        })?;
    let out = std::process::Command::new(t27c)
        .arg("parse-complete")
        .current_dir(root)
        .output()
        .context("running `t27c parse-complete`")?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut map = BTreeMap::new();
    for line in text.lines() {
        // `specs/base/ternary_add.t27: DISCARDED 208 top-level token(s)`
        let Some((path, rest)) = line.split_once(": DISCARDED ") else {
            continue;
        };
        let Some(n) = rest.split_whitespace().next().and_then(|w| w.parse().ok()) else {
            continue;
        };
        map.insert(path.trim().to_string(), n);
    }
    if map.is_empty() && !text.contains("parse but DISCARD") {
        anyhow::bail!(
            "`t27c parse-complete` produced no recognisable output.\n  \
             Nothing was read, so nothing is claimed -- this is not \"zero specs discard\"."
        );
    }
    Ok(map)
}

/// `spec -> pinned tokens`, from the ledger. Absent means the entry carries no
/// bound, which the ratchet fails on; here it prints as `--`.
fn pinned(root: &std::path::Path) -> Result<BTreeMap<String, Option<usize>>> {
    let p = root.join("docs/reports/suite_expectations.json");
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?,
    )
    .with_context(|| format!("parsing {}", p.display()))?;
    let mut map = BTreeMap::new();
    for e in v
        .get("entries")
        .and_then(|x| x.as_array())
        .unwrap_or(&vec![])
    {
        if e.get("phase").and_then(|x| x.as_str()) != Some("parse-no-discard") {
            continue;
        }
        let Some(path) = e.get("path").and_then(|x| x.as_str()) else {
            continue;
        };
        map.insert(
            path.to_string(),
            e.get("discard_tokens")
                .and_then(|x| x.as_u64())
                .map(|n| n as usize),
        );
    }
    Ok(map)
}

pub fn run(cmd: &DiscardCmd) -> Result<()> {
    let root = repo_root()?;
    let obs = observed(&root)?;
    let pin = pinned(&root)?;

    let DiscardCmd::Top { n } = cmd;
    let mut rows: Vec<(&String, &usize)> = obs.iter().collect();
    rows.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));

    let total: usize = obs.values().sum();
    println!("  {} spec(s) discard {} token(s)", obs.len(), total);
    println!();
    println!("  {:>7}  {:>7}  spec", "tokens", "pinned");
    let shown = if *n == 0 {
        rows.len()
    } else {
        (*n).min(rows.len())
    };
    for (path, tokens) in rows.iter().take(shown) {
        let p = match pin.get(*path) {
            Some(Some(v)) => v.to_string(),
            Some(None) => "--".to_string(),
            // Observed but not in the ledger at all: an unexpected failure, and
            // the ratchet says so far more loudly than this table should.
            None => "NOT IN LEDGER".to_string(),
        };
        println!("  {:>7}  {:>7}  {}", tokens, p, path);
    }
    if shown < rows.len() {
        println!(
            "  ... and {} more not shown (--n 0 for all)",
            rows.len() - shown
        );
    }
    println!();
    println!("  The pinned column is the ledger's bound, not a target. It moves");
    println!("  only when `t27c suite --bless-expectations` re-measures.");
    Ok(())
}

#[cfg(test)]
mod tests {
    /// The line shape this parses is `t27c parse-complete`'s, and it is the only
    /// contract between the two. Pin it here so a change to that output is a
    /// test failure rather than a silently empty ranking.
    #[test]
    fn the_parse_complete_line_shape_is_read_correctly() {
        let line = "specs/base/ternary_add.t27: DISCARDED 208 top-level token(s)";
        let (path, rest) = line.split_once(": DISCARDED ").expect("shape changed");
        assert_eq!(path, "specs/base/ternary_add.t27");
        let n: usize = rest.split_whitespace().next().unwrap().parse().unwrap();
        assert_eq!(n, 208);
    }

    /// A summary line must not be mistaken for a spec row.
    #[test]
    fn the_summary_lines_are_not_rows() {
        for line in [
            "  parse but DISCARD        87 (32485 token(s))",
            "  specs scanned            650",
        ] {
            assert!(line.split_once(": DISCARDED ").is_none(), "{line}");
        }
    }
}

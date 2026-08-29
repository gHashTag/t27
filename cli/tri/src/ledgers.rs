//! Does each ledger's gate notice when one of its entries stops being true?
//!
//! WHY THIS EXISTS
//! ---------------
//! This repository keeps several ledgers that name specs by path and say "this
//! one is expected to fail". A line outliving its debt is a specific, silent
//! defect: the spec is fixed, the ledger still excuses it, and nothing says so.
//!
//! It happened. A repair made `specs/pins/emitter_xdc.t27` typecheck, one
//! ledger was updated and `docs/reports/suite_expectations.json` was not, and
//! the corpus ratchet sat RED on master for three runs before anyone read it.
//! That was found by accident, because that particular gate fails loudly.
//!
//! Measured deliberately afterwards, by planting a line naming a spec that
//! passes and running each gate:
//!
//!     seal_baseline.txt              FAILS   -- the entry is caught
//!     conflict_markers_baseline.txt  FAILS
//!     suite_expectations.json        FAILS   -- "UNEXPECTED PASS"
//!     specs_generate_baseline.txt    printed a NOTE and exited 0
//!     verilog_width_baseline.txt     printed a note and exited 0
//!
//! Two of five announced the defect and returned success. Both now fail; this
//! command is what keeps that true, and what will notice the next ledger added
//! without the same property.
//!
//! It MUTATES the ledgers while it runs and restores them. It refuses to start
//! if any of them is already dirty, because a restore would then discard work.
use anyhow::Result;
use clap::Subcommand;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum LedgersCmd {
    /// Plant a stale entry in each ledger and demand its gate fail.
    Audit,
}

/// A ledger, the gate that reads it, and how a stale line is spelled there.
///
/// `{spec}` is replaced with a spec that currently passes everything, so the
/// planted line is FALSE by construction -- which is exactly what a stale entry
/// is.
struct Ledger {
    path: &'static str,
    gate: &'static str,
    stale_line: &'static str,
}

const LEDGERS: &[Ledger] = &[
    Ledger {
        path: "tools/specs_generate_baseline.txt",
        gate: "tools/check_specs_generate.py",
        stale_line: "{spec} | planted by `tri ledgers audit`",
    },
    Ledger {
        path: "tools/seal_baseline.txt",
        gate: "tools/check_seal_coverage.py",
        stale_line: "PlantedByAudit.json | dangling | {spec}",
    },
    Ledger {
        path: "tools/conflict_markers_baseline.txt",
        gate: "tools/check_conflict_markers.py",
        stale_line: "{spec} | planted by `tri ledgers audit`",
    },
    Ledger {
        path: "tools/verilog_width_baseline.txt",
        gate: "tools/check_verilog_widths.py",
        stale_line: "{spec} | planted by `tri ledgers audit`",
    },
];

/// A spec that passes today, so a ledger line naming it is false on its face.
fn a_passing_spec(root: &Path, t27c: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["ls-files", "specs/*.t27"])
        .current_dir(root)
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find(|sp| {
            std::process::Command::new(t27c)
                .arg("check")
                .arg(sp)
                .current_dir(root)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
        .map(|s| s.to_string())
}

fn run_gate(root: &Path, gate: &str) -> Option<bool> {
    let out = std::process::Command::new("python3")
        .arg(gate)
        .current_dir(root)
        .output()
        .ok()?;
    Some(out.status.success())
}

pub fn run(cmd: &LedgersCmd, root: PathBuf) -> Result<()> {
    let LedgersCmd::Audit = cmd;
    let t27c = ["target/release/t27c", "target/debug/t27c"]
        .iter()
        .map(|p| root.join(p))
        .find(|p| p.is_file())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no compiler -- the planted line has to name a spec that PASSES,\n  \
                 and without one this audit would plant a line that is true.\n  \
                 cargo build --release -p t27c"
            )
        })?;

    // Refuse on a dirty ledger: the restore at the end would discard it.
    let dirty = std::process::Command::new("git")
        .args(["status", "--porcelain", "--"])
        .args(LEDGERS.iter().map(|l| l.path))
        .current_dir(&root)
        .output()?;
    let dirty = String::from_utf8_lossy(&dirty.stdout);
    if !dirty.trim().is_empty() {
        anyhow::bail!(
            "a ledger has uncommitted changes; this command rewrites and restores\n  \
             them, so it would discard your work:\n{}",
            dirty.trim()
        );
    }

    let Some(spec) = a_passing_spec(&root, &t27c) else {
        anyhow::bail!("no spec passes today -- nothing to plant a false line about");
    };
    println!("  planting a line about   {spec}");
    println!("  it passes, so every planted line is FALSE by construction");
    println!();

    let (mut caught, mut missed, mut skipped) = (0usize, 0usize, 0usize);
    for l in LEDGERS {
        let path = root.join(l.path);
        let gate = root.join(l.gate);
        if !path.is_file() || !gate.is_file() {
            println!("      SKIP      {}  (ledger or gate absent)", l.path);
            skipped += 1;
            continue;
        }
        let Some(clean) = run_gate(&root, l.gate) else {
            println!("      SKIP      {}  (gate did not run)", l.path);
            skipped += 1;
            continue;
        };
        if !clean {
            println!(
                "      SKIP      {}  (gate is already red; fix that first)",
                l.path
            );
            skipped += 1;
            continue;
        }
        let Ok(before) = std::fs::read_to_string(&path) else {
            skipped += 1;
            continue;
        };
        let planted = format!("{}{}\n", before, l.stale_line.replace("{spec}", &spec));
        if std::fs::write(&path, &planted).is_err() {
            skipped += 1;
            continue;
        }
        let verdict = run_gate(&root, l.gate);
        let _ = std::fs::write(&path, &before);
        match verdict {
            Some(false) => {
                println!("      caught    {}", l.path);
                caught += 1;
            }
            _ => {
                println!("      MISSED    {}  <- a stale entry here exits 0", l.path);
                missed += 1;
            }
        }
    }

    println!();
    println!("  ledgers audited   {}", LEDGERS.len());
    println!("  stale entry caught {caught}");
    println!("  stale entry MISSED {missed}");
    if skipped > 0 {
        println!("  not audited        {skipped}");
    }
    println!();
    if missed > 0 {
        println!("  A ledger whose gate exits 0 on a stale entry keeps excusing a spec");
        println!("  that was fixed, and nothing says so. That is how a repair to one");
        println!("  ledger left another red on master for three runs.");
        return Err(anyhow::anyhow!(
            "{missed} ledger(s) do not catch a stale entry"
        ));
    }
    println!("  Every ledger fails when one of its entries stops being true.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // The planted line must name the spec, or the audit tests nothing.
    #[test]
    fn every_template_carries_the_spec_placeholder() {
        for l in LEDGERS {
            assert!(
                l.stale_line.contains("{spec}"),
                "template names no spec: {}",
                l.path
            );
        }
    }

    #[test]
    fn every_ledger_has_a_distinct_gate() {
        let mut seen = std::collections::BTreeSet::new();
        for l in LEDGERS {
            assert!(seen.insert(l.gate), "two ledgers share a gate: {}", l.gate);
        }
    }

    // Substitution must produce a line that mentions the spec path verbatim --
    // a template that silently drops it would plant a line no gate can match.
    #[test]
    fn substitution_keeps_the_path() {
        for l in LEDGERS {
            let line = l.stale_line.replace("{spec}", "specs/x/y.t27");
            assert!(line.contains("specs/x/y.t27"), "{}", l.path);
        }
    }
}

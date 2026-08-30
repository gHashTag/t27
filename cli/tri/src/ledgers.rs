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
    gate: Gate,
    plant: Plant,
}

/// What runs the ledger's claims.
enum Gate {
    /// A python checker, by path.
    Script(&'static str),
    /// A `tri` subcommand. `docs/reports/orphan_modules.json` is gated by
    /// `tri mods orphan --gate`, not by a script, and a meta-gate that only
    /// knows about scripts is a meta-gate with a shape for a population.
    Tri(&'static [&'static str]),
}

/// How to make one entry FALSE without making the file unreadable.
enum Plant {
    /// Append a line. `{spec}` becomes a spec that passes today.
    Line(&'static str),
    /// Add a ceiling for a crate the workspace does not declare.
    ///
    /// Appending a line to a JSON ledger would make the gate fail because the
    /// file no longer PARSES -- a catch for the wrong reason, which is a
    /// control that reports success without measuring anything. The planted
    /// entry has to stay valid and be false.
    GhostCeiling,
}

const LEDGERS: &[Ledger] = &[
    Ledger {
        path: "tools/specs_generate_baseline.txt",
        gate: Gate::Script("tools/check_specs_generate.py"),
        plant: Plant::Line("{spec} | planted by `tri ledgers audit`"),
    },
    Ledger {
        path: "tools/seal_baseline.txt",
        gate: Gate::Script("tools/check_seal_coverage.py"),
        plant: Plant::Line("PlantedByAudit.json | dangling | {spec}"),
    },
    Ledger {
        path: "tools/conflict_markers_baseline.txt",
        gate: Gate::Script("tools/check_conflict_markers.py"),
        plant: Plant::Line("{spec} | planted by `tri ledgers audit`"),
    },
    Ledger {
        path: "tools/verilog_width_baseline.txt",
        gate: Gate::Script("tools/check_verilog_widths.py"),
        plant: Plant::Line("{spec} | planted by `tri ledgers audit`"),
    },
    Ledger {
        path: "docs/reports/orphan_modules.json",
        gate: Gate::Tri(&["mods", "orphan", "--gate"]),
        plant: Plant::GhostCeiling,
    },
];

/// Every file in this repository shaped like a ledger, found by walking rather
/// than written down.
///
/// The hand-written list said four and I extended it to five. Counting from
/// DISK says fifteen: nine `tools/*baseline*.txt` and six
/// `docs/reports/*.json`. The four-of-seven I started from was a sample taken
/// while chasing something else -- this repository's own lesson about counts,
/// applied to me.
///
/// Enumerating from disk is the only version that cannot go stale by addition,
/// which is the defect this meta-gate exists to catch, in the meta-gate.
fn ledger_shaped(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for (dir, want_baseline) in [("tools", true), ("docs/reports", false)] {
        let Ok(rd) = std::fs::read_dir(root.join(dir)) else {
            continue;
        };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let keep = if want_baseline {
                name.contains("baseline") && name.ends_with(".txt")
            } else {
                name.ends_with(".json")
            };
            if keep {
                out.push(format!("{dir}/{name}"));
            }
        }
    }
    out.sort();
    out
}

/// A ledger this audit does NOT plant into, and the measurement behind it.
struct Unaudited {
    path: &'static str,
    why: &'static str,
}

const UNAUDITED: [Unaudited; 2] = [
    Unaudited {
        path: "docs/reports/suite_expectations.json",
        why: "its gate is the corpus ratchet, which compiles every spec in the corpus. \
              Planting into it costs minutes per run and would make this audit too slow \
              to run before a commit -- which is when a meta-gate has to be cheap.",
    },
    Unaudited {
        path: "tools/withdrawn_live_baseline.txt",
        why: "entries are keyed by the sha1 of the line they excuse, so a planted line \
              would need a hash that matches text elsewhere in the tree. A planted entry \
              that cannot be false by construction proves nothing.",
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

fn run_gate(root: &Path, gate: &Gate) -> Option<bool> {
    let out = match gate {
        Gate::Script(path) => std::process::Command::new("python3")
            .arg(path)
            .current_dir(root)
            .output()
            .ok()?,
        Gate::Tri(args) => std::process::Command::new(std::env::current_exe().ok()?)
            .args(*args)
            .current_dir(root)
            .output()
            .ok()?,
    };
    Some(out.status.success())
}

/// A name for the gate, for messages and for asking whether two are the same.
fn gate_name(gate: &Gate) -> String {
    match gate {
        Gate::Script(p) => (*p).to_string(),
        Gate::Tri(args) => format!("tri {}", args.join(" ")),
    }
}

/// Does the gate's file exist? A `tri` subcommand is this binary, always here.
fn gate_present(root: &Path, gate: &Gate) -> bool {
    match gate {
        Gate::Script(p) => root.join(p).is_file(),
        Gate::Tri(_) => true,
    }
}

/// The planted text: false by construction, and still readable by the gate.
fn plant_text(before: &str, plant: &Plant, spec: &str) -> Option<String> {
    match plant {
        Plant::Line(t) => Some(format!("{before}{}\n", t.replace("{spec}", spec))),
        Plant::GhostCeiling => {
            // Textual, so the file's formatting survives: insert one key into
            // the `ceilings` object rather than re-serialising the document.
            let at = before.find("\"ceilings\"")?;
            let brace = before[at..].find('{')? + at + 1;
            Some(format!(
                "{}\n    \"cli/planted-by-ledgers-audit\": 0,{}",
                &before[..brace],
                &before[brace..]
            ))
        }
    }
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
        if !path.is_file() || !gate_present(&root, &l.gate) {
            println!("      SKIP      {}  (ledger or gate absent)", l.path);
            skipped += 1;
            continue;
        }
        let Some(clean) = run_gate(&root, &l.gate) else {
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
        let Some(planted) = plant_text(&before, &l.plant, &spec) else {
            println!("      SKIP      {}  (nothing to plant into)", l.path);
            skipped += 1;
            continue;
        };
        if std::fs::write(&path, &planted).is_err() {
            skipped += 1;
            continue;
        }
        let verdict = run_gate(&root, &l.gate);
        let _ = std::fs::write(&path, &before);
        match verdict {
            Some(false) => {
                // Name the gate too: "caught" without it says a stale entry
                // fails SOMETHING, and which one is the next reader's question.
                println!("      caught    {:<38} by {}", l.path, gate_name(&l.gate));
                caught += 1;
            }
            _ => {
                println!(
                    "      MISSED    {:<38} by {}  <- a stale entry here exits 0",
                    l.path,
                    gate_name(&l.gate)
                );
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
    println!("  Every ledger this audit plants into fails when an entry stops being true.");
    println!();

    let shaped = ledger_shaped(&root);
    let known: Vec<&str> = LEDGERS
        .iter()
        .map(|l| l.path)
        .chain(UNAUDITED.iter().map(|u| u.path))
        .collect();
    let loose: Vec<&String> = shaped
        .iter()
        .filter(|f| !known.contains(&f.as_str()))
        .collect();

    println!(
        "  ledger-shaped files on disk   {}   planted into {}, excused {}, unclassified {}",
        shaped.len(),
        LEDGERS.len(),
        UNAUDITED.len(),
        loose.len()
    );
    println!();
    for u in &UNAUDITED {
        println!("      excused   {}", u.path);
        for chunk in u.why.split_whitespace().collect::<Vec<_>>().chunks(11) {
            println!("                {}", chunk.join(" "));
        }
    }
    if !loose.is_empty() {
        println!();
        println!("      NOT YET CLASSIFIED -- neither planted into nor measured and");
        println!("      excused. A work list, not a verdict: a meta-gate that prints");
        println!("      only what it covers reads as coverage.");
        for f in &loose {
            println!("          {f}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ledger is planted into or measured and excused, never both.
    ///
    /// The same rule the census audit keeps: an exclusion is a measurement, so
    /// it carries the reading that produced it. "too hard" is not one.
    #[test]
    fn no_ledger_is_both_planted_into_and_excused() {
        for u in &UNAUDITED {
            assert!(
                !LEDGERS.iter().any(|l| l.path == u.path),
                "{} is excused and also planted into",
                u.path
            );
            assert!(
                u.why.len() > 60,
                "{}: an exclusion is a measurement, not a shrug -- {:?}",
                u.path,
                u.why
            );
        }
    }

    /// The enumeration reads the tree, so it cannot go stale by addition.
    ///
    /// The hand-written list said four. Disk says fifteen. If this ever finds
    /// fewer than the files this repository is known to carry, the walk is
    /// broken and every "unclassified 0" it prints would be a silence.
    #[test]
    fn the_enumeration_reads_the_tree() {
        let root = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim().to_string()));
        let Some(root) = root else {
            return; // not in a checkout; nothing to read
        };
        let found = ledger_shaped(&root);
        assert!(
            found.len() >= 10,
            "the walk found {} ledger-shaped files, which is fewer than this \
             repository carries -- a broken walk prints `unclassified 0`",
            found.len()
        );
        for l in LEDGERS {
            assert!(
                found.iter().any(|f| f == l.path),
                "{} is planted into but the walk does not see it",
                l.path
            );
        }
    }

    // The planted line must name the spec, or the audit tests nothing.
    #[test]
    fn every_template_carries_the_spec_placeholder() {
        for l in LEDGERS {
            if let Plant::Line(t) = &l.plant {
                assert!(t.contains("{spec}"), "template names no spec: {}", l.path);
            }
        }
    }

    #[test]
    fn every_ledger_has_a_distinct_gate() {
        let mut seen = std::collections::BTreeSet::new();
        for l in LEDGERS {
            let name = gate_name(&l.gate);
            assert!(
                seen.insert(name.clone()),
                "two ledgers share a gate: {name}"
            );
        }
    }

    // Substitution must produce a line that mentions the spec path verbatim --
    // a template that silently drops it would plant a line no gate can match.
    #[test]
    fn substitution_keeps_the_path() {
        for l in LEDGERS {
            match &l.plant {
                Plant::Line(_) => {
                    let planted = plant_text("", &l.plant, "specs/x/y.t27").expect("planted");
                    assert!(planted.contains("specs/x/y.t27"), "{}", l.path);
                }
                // A JSON ledger is falsified by an entry, not by a spec name.
                // The planted text must still PARSE -- appending junk makes the
                // gate fail because the file is unreadable, which is a catch
                // for the wrong reason.
                Plant::GhostCeiling => {
                    let before = "{\n  \"ceilings\": {\n    \"a\": 1\n  }\n}\n";
                    let planted = plant_text(before, &l.plant, "unused").expect("planted");
                    assert!(planted.contains("planted-by-ledgers-audit"), "{}", l.path);
                    assert!(
                        serde_json::from_str::<serde_json::Value>(&planted).is_ok(),
                        "{}: the planted ledger must still parse",
                        l.path
                    );
                }
            }
        }
    }
}

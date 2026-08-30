//! `tri vsim funnel` -- how far each spec gets when its Verilog is actually RUN.
//!
//! WHY THIS EXISTS
//! ---------------
//! Four backends, and only one of them can catch a defect whose whole nature is
//! that it compiles: the one that simulates. `t27c suite --icarus-simulate` is
//! that arm, and its target selector walks `repo/specs/scratch`, a directory
//! untracked in #2283 and absent from the tree. The phase hits the W643 honest
//! floor and hard-errors; nothing in `.github/workflows/` or the Makefile
//! invokes it; and 260 tracked baselines under `.trinity/icarus-baselines/`
//! describe subjects that no longer exist. See #2987.
//!
//! While that decision waits, the measurement should not. Pointed at the corpus
//! by hand the first time, it produced four defects in one sitting -- a float
//! multiply routed through an integer ladder (#2990, fixed), `break` lowered to
//! `disable fork;` against zero `fork` keywords (#2988), an early `return` that
//! does not leave its loop so `binary_search` never terminates (#2989), and
//! every `invariant` emitted as a comment (#2869).
//!
//! WHAT IT REPORTS, AND WHY THE LAST TWO ROWS ARE SEPARATE
//! ------------------------------------------------------
//! `run_icarus_simulate` bails on a line saying FAILED and on nothing else, so
//! "the testbench checked everything and passed" and "the testbench checked
//! nothing" are the same exit code. This command keeps them apart:
//!
//!   gen        `gen-verilog-for-simulation` refused
//!   elab       `iverilog` rejected the testbench
//!   vvp        the simulation itself exited non-zero
//!   testfail   ran, and reported a FAILED verdict
//!   verdict    ran, exit 0, and printed at least one PASSED/FAILED line
//!   silent     ran, exit 0, and printed NO verdict line at all
//!
//! `silent` is the honest floor. A spec there is not passing; nothing asked it
//! anything.
//!
//! WHAT IT REFUSES
//! ---------------
//! Absence of `iverilog` is not a result. With no simulator on `PATH` every spec
//! would land in `elab` and the funnel would read like a catastrophic
//! regression, so the command says so and exits non-zero rather than printing a
//! table it did not earn. Same for a missing `t27c`.
//!
//! It reports and never gates. Which of the specs that report failures today are
//! compiler defects and which are spec defects is not a question a walker may
//! answer, and #2987 says so.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Subcommand)]
pub enum VsimCmd {
    /// Walk the corpus through `t27c icarus-simulate` and print the funnel.
    Funnel {
        /// Stop after this many specs. The population line still names the
        /// whole corpus, so a truncated run cannot read as a complete one.
        #[arg(long)]
        limit: Option<usize>,
        /// Seconds before a spec is recorded as a timeout. Timeouts are counted
        /// and printed apart from failures; a slow spec is not a rejected one.
        #[arg(long, default_value_t = 120)]
        timeout: u64,
    },
}

/// Where a spec stopped. The order is the funnel's order.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Stage {
    Gen,
    Elab,
    Vvp,
    TestFail,
    Verdict,
    Silent,
    Timeout,
}

impl Stage {
    fn label(self) -> &'static str {
        match self {
            Stage::Gen => "gen",
            Stage::Elab => "elab",
            Stage::Vvp => "vvp",
            Stage::TestFail => "testfail",
            Stage::Verdict => "verdict",
            Stage::Silent => "silent",
            Stage::Timeout => "timeout",
        }
    }
}

pub fn run(cmd: &VsimCmd) -> Result<()> {
    match cmd {
        VsimCmd::Funnel { limit, timeout } => funnel(*limit, *timeout),
    }
}

fn funnel(limit: Option<usize>, timeout: u64) -> Result<()> {
    let root = repo_root()?;
    let t27c = root.join("target/release/t27c");
    if !t27c.exists() {
        bail!(
            "no compiler at {}\n  \
             Build it first: cargo build --release -p t27c\n  \
             A funnel measured with no compiler is not an empty funnel.",
            t27c.display()
        );
    }
    // Ask the OS, not an error message: a phrase is the tool's to change and
    // PATH is not. Without this every spec lands in `elab` and the table reads
    // as a corpus-wide regression.
    if Command::new("iverilog").arg("-V").output().is_err() {
        bail!(
            "iverilog is not on PATH.\n  \
             Every spec would be recorded as an elaboration failure, which is a\n  \
             statement about this machine and not about the compiler.\n  \
             Nothing is claimed."
        );
    }

    // Snapshot the population before walking it. A file list regenerated
    // mid-run by something else is how a numerator and a denominator end up
    // describing different corpora.
    let mut specs = Vec::new();
    collect_specs(&root.join("specs"), &root, &mut specs);
    specs.sort();
    let population = specs.len();
    let walked: Vec<&String> = match limit {
        Some(n) => specs.iter().take(n).collect(),
        None => specs.iter().collect(),
    };

    let scratch = std::env::temp_dir().join(format!("tri-vsim-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).context("creating the scratch root")?;

    let mut counts: BTreeMap<Stage, usize> = BTreeMap::new();
    let mut failing: Vec<String> = Vec::new();
    let mut silent: Vec<String> = Vec::new();

    for (i, spec) in walked.iter().enumerate() {
        // One directory per spec. `run_icarus_simulate` writes
        // `<temp_dir>/t27c_icarus_<file_stem>.v`, keyed on the BASENAME alone,
        // and this corpus has stems shared by more than one spec -- so two of
        // them would overwrite each other's Verilog.
        let dir = scratch.join(format!("s{i}"));
        std::fs::create_dir_all(&dir).context("creating a per-spec directory")?;

        let stage = simulate_one(&t27c, &root, spec, &dir, timeout);
        *counts.entry(stage).or_insert(0) += 1;
        match stage {
            Stage::TestFail => failing.push((*spec).clone()),
            Stage::Silent => silent.push((*spec).clone()),
            _ => {}
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
    let _ = std::fs::remove_dir_all(&scratch);

    let n = walked.len();
    println!();
    println!("  specs under specs/                      {population}");
    if n != population {
        println!("  WALKED THIS RUN                         {n}   (--limit)");
    }
    println!();
    let get = |s: Stage| *counts.get(&s).unwrap_or(&0);
    println!(
        "  refused by gen-verilog-for-simulation    {}",
        get(Stage::Gen)
    );
    println!(
        "  iverilog rejected the testbench          {}",
        get(Stage::Elab)
    );
    println!(
        "  simulation exited non-zero               {}",
        get(Stage::Vvp)
    );
    println!(
        "  ran and REPORTED FAILURES                {}",
        get(Stage::TestFail)
    );
    println!(
        "  ran and produced a verdict               {}",
        get(Stage::Verdict)
    );
    println!(
        "  ran, exit 0, and said NOTHING            {}",
        get(Stage::Silent)
    );
    println!(
        "  timed out at {timeout}s                          {}",
        get(Stage::Timeout)
    );

    let sum: usize = counts.values().sum();
    println!();
    println!("  {sum} of {n} accounted for.");
    if sum != n {
        // A census whose parts do not sum to its total has a bucket nobody
        // named. Say so rather than printing a table that looks complete.
        println!("  MISMATCH: {} spec(s) fell through every arm.", n - sum);
    }

    if !failing.is_empty() {
        println!();
        println!("  Reporting failures RIGHT NOW ({}):", failing.len());
        for s in &failing {
            println!("    {s}");
        }
        println!(
            "  Whether these are compiler defects or spec defects is not\n  \
             decided here -- see #2987. What is decided is that a gate\n  \
             pointed at this corpus would be red today."
        );
    }
    if !silent.is_empty() {
        println!();
        println!(
            "  Exit 0 with no verdict line ({}). Not passing -- unasked.",
            silent.len()
        );
        for s in silent.iter().take(10) {
            println!("    {s}");
        }
        if silent.len() > 10 {
            println!("    ... and {} more", silent.len() - 10);
        }
    }
    println!();
    Ok(())
}

/// Run one spec and say where it stopped.
fn simulate_one(t27c: &Path, root: &Path, spec: &str, dir: &Path, timeout: u64) -> Stage {
    // macOS has no `timeout(1)`. `perl -e 'alarm N; exec @ARGV'` exits 142 when
    // the alarm fires, which is how a slow spec is kept out of the failure
    // column.
    let out = Command::new("perl")
        .arg("-e")
        .arg("alarm shift; exec @ARGV")
        .arg(timeout.to_string())
        .arg(t27c)
        .arg("icarus-simulate")
        .arg(root.join(spec))
        .env("TMPDIR", dir)
        .current_dir(root)
        .output();
    let Ok(out) = out else {
        return Stage::Gen;
    };
    if out.status.code() == Some(142) {
        return Stage::Timeout;
    }
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if !out.status.success() {
        // Attributed from the compiler's own words, not from the exit code,
        // which is the same number for all four.
        if log.contains("Verilog generation error") {
            return Stage::Gen;
        }
        if log.contains("iverilog rejected") {
            return Stage::Elab;
        }
        if log.contains("reported test/bench failures") {
            return Stage::TestFail;
        }
        if log.contains("vvp") {
            return Stage::Vvp;
        }
        return Stage::Gen;
    }
    if has_verdict(&log) {
        Stage::Verdict
    } else {
        Stage::Silent
    }
}

/// Does the log carry a runtime verdict line?
///
/// The emitter writes `[TEST] name : PASSED` / `: FAILED` and `[BENCH] ...`.
/// It also writes `[TEST] name : starting` and `: NOT CHECKED`, and neither is
/// a verdict -- a block that announces itself and checks nothing must not be
/// counted as one, which is the whole distinction this command exists to draw.
fn has_verdict(log: &str) -> bool {
    log.lines().any(|l| {
        let l = l.trim();
        (l.starts_with("[TEST]") || l.starts_with("[BENCH]"))
            && (l.ends_with(": PASSED") || l.contains(": FAILED"))
    })
}

fn collect_specs(dir: &Path, root: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_specs(&p, root, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("t27") {
            if let Ok(rel) = p.strip_prefix(root) {
                out.push(rel.to_string_lossy().into_owned());
            }
        }
    }
}

fn repo_root() -> Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running `git rev-parse --show-toplevel`")?;
    if !out.status.success() {
        bail!("not inside a git repository");
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_starting_line_is_not_a_verdict() {
        // The emitter announces every block before it runs. Counting that as a
        // verdict would make `silent` -- the row this command exists for --
        // permanently zero.
        assert!(!has_verdict("[TEST] t : starting\n"));
    }

    #[test]
    fn not_checked_is_not_a_verdict() {
        assert!(!has_verdict(
            "[TEST] t : starting\n[TEST] t : NOT CHECKED (empty body)\n"
        ));
    }

    #[test]
    fn passed_and_failed_both_count() {
        assert!(has_verdict("[TEST] t : PASSED"));
        assert!(has_verdict("[TEST] t : FAILED\n  assert failed:"));
        assert!(has_verdict("[BENCH] b : FAILED"));
    }

    #[test]
    fn a_verdict_for_some_other_tag_does_not_count() {
        // `[INVARIANT]` has no emitter anywhere in compiler.rs today. If one is
        // ever added this test is the reminder that this reader must learn it
        // rather than silently keep reporting those specs as silent.
        assert!(!has_verdict("[INVARIANT] i : PASSED"));
    }

    #[test]
    fn the_stage_labels_are_all_distinct() {
        let all = [
            Stage::Gen,
            Stage::Elab,
            Stage::Vvp,
            Stage::TestFail,
            Stage::Verdict,
            Stage::Silent,
            Stage::Timeout,
        ];
        let mut labels: Vec<&str> = all.iter().map(|s| s.label()).collect();
        labels.sort_unstable();
        let before = labels.len();
        labels.dedup();
        assert_eq!(before, labels.len(), "two stages share a label");
    }
}

//! `tri emit determinism` -- does the same binary produce the same output twice?
//!
//! WHY THIS EXISTS
//! ---------------
//! Every byte-comparison this repository makes over generated code assumes the
//! generator is deterministic: `.trinity/seals/*.json` store `gen_hash_c`,
//! `gen_hash_rust` and `gen_hash_zig`; `emit-bitexact-gate.yml` exists to assert
//! that emission is reproducible; and `t27c corpus --per-spec` is diffed against
//! another binary to name the specs a change moved.
//!
//! Three of the four emitters do not have it. Measured over 650 specs, same
//! binary, two consecutive runs:
//!
//!   gen-c        1 file differs
//!   gen-rust     3
//!   gen  (Zig)   2
//!   gen-verilog  0
//!
//! The one backend the seal gate is historically pointed at is the deterministic
//! one, which is why the wobble has not been noticed. See #3006.
//!
//! WHAT IT PRINTS, AND WHY IT NAMES FILES
//! --------------------------------------
//! A count is what the accident produced; a NAME is what a repair needs. #3006
//! says the cheapest next step is naming the differing specs rather than
//! counting them, so that is what this does. A wobble of one file in 581 reads
//! as a real change on the day it fires, and this repository's log already
//! records the cost of that mistake -- "a gate going red on a change that cannot
//! touch it".
//!
//! WHAT IT REFUSES
//! ---------------
//! With no compiler it says so and exits non-zero. A determinism report of
//! "0 differences" taken with no binary is a statement about this machine, not
//! about the emitters, and reads exactly like health.
//!
//! It reports and never gates. Whether a wobbling file is a defect worth fixing
//! or an ordering that nothing downstream reads is not a walker's question, and
//! a gate that reddens on it before anyone has decided would be muted.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The emitters, in the order the corpus table prints them.
const BACKENDS: &[(&str, &str)] = &[
    ("gen-c", "C"),
    ("gen-rust", "Rust"),
    ("gen", "Zig"),
    ("gen-verilog", "Verilog"),
];

#[derive(Subcommand)]
pub enum EmitCmd {
    /// Generate the corpus twice with the same binary and name what differs.
    Determinism {
        /// Only this backend (`gen-c`, `gen-rust`, `gen`, `gen-verilog`).
        #[arg(long)]
        backend: Option<String>,
        /// Stop after this many specs. The population line still names the whole
        /// corpus, so a truncated run cannot read as a complete one.
        #[arg(long)]
        limit: Option<usize>,
        /// Run each backend this many times instead of twice. A file that
        /// differs on run 3 but not run 2 is the same defect and a rarer draw.
        #[arg(long, default_value_t = 2)]
        runs: usize,
    },
}

pub fn run(cmd: &EmitCmd) -> Result<()> {
    match cmd {
        EmitCmd::Determinism {
            backend,
            limit,
            runs,
        } => determinism(backend.as_deref(), *limit, *runs),
    }
}

fn determinism(only: Option<&str>, limit: Option<usize>, runs: usize) -> Result<()> {
    if runs < 2 {
        bail!("--runs must be at least 2: one run cannot disagree with itself");
    }
    let root = repo_root()?;
    let t27c = root.join("target/release/t27c");
    if !t27c.exists() {
        bail!(
            "no compiler at {}\n  \
             Build it first: cargo build --release -p t27c\n  \
             A determinism report taken with no binary is a statement about this\n  \
             machine, and it reads exactly like health.",
            t27c.display()
        );
    }
    if let Some(b) = only {
        if !BACKENDS.iter().any(|(cmd, _)| *cmd == b) {
            bail!(
                "unknown backend `{b}` -- expected one of: {}",
                BACKENDS
                    .iter()
                    .map(|(c, _)| *c)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    // Snapshot the population before walking it. A file list regenerated
    // mid-run is how a numerator and a denominator end up describing different
    // corpora.
    let mut specs = Vec::new();
    collect_specs(&root.join("specs"), &root, &mut specs);
    specs.sort();
    let population = specs.len();
    let walked: Vec<&String> = match limit {
        Some(n) => specs.iter().take(n).collect(),
        None => specs.iter().collect(),
    };

    println!();
    println!("  specs under specs/     {population}");
    if walked.len() != population {
        println!("  WALKED THIS RUN        {}   (--limit)", walked.len());
    }
    println!("  runs per backend       {runs}");
    println!();

    let mut any = false;
    for (cmd, label) in BACKENDS {
        if let Some(b) = only {
            if b != *cmd {
                continue;
            }
        }
        // spec -> the outputs seen across runs, in order.
        let mut seen: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        let mut generated = 0usize;
        for r in 0..runs {
            for spec in &walked {
                let out = Command::new(&t27c)
                    .arg(cmd)
                    .arg(root.join(spec))
                    .current_dir(&root)
                    .output();
                let text = match out {
                    Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
                    // A spec that does not generate is not a wobble. Record the
                    // refusal itself, so a spec that generates on one run and
                    // refuses on the next still shows up as a difference.
                    _ => String::from("\u{0}DID-NOT-GENERATE"),
                };
                if r == 0 && !text.starts_with('\u{0}') {
                    generated += 1;
                }
                seen.entry(spec.as_str()).or_default().push(text);
            }
        }
        let differing: Vec<&str> = seen
            .iter()
            .filter(|(_, outs)| outs.iter().any(|o| o != &outs[0]))
            .map(|(s, _)| *s)
            .collect();
        if !differing.is_empty() {
            any = true;
        }
        println!(
            "  {label:<8} {cmd:<12} generated {generated:>3}   differing across {runs} runs: {}",
            differing.len()
        );
        for s in &differing {
            println!("      {s}");
        }
    }

    println!();
    if any {
        println!(
            "  A file listed above is emitted differently by the SAME binary from the\n  \
             SAME source. Every byte-comparison over generated output -- seals,\n  \
             emit-bitexact, `corpus --per-spec` diffing -- reads that as a change.\n  \
             See #3006. This command reports; it does not gate."
        );
    } else {
        println!(
            "  No spec differed. That is a result and not a silence: {} spec(s) were\n  \
             generated {runs} times by each backend and compared in full.",
            walked.len()
        );
    }
    println!();
    Ok(())
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
    fn one_run_cannot_disagree_with_itself() {
        let e = determinism(None, Some(1), 1).unwrap_err();
        assert!(
            e.to_string().contains("at least 2"),
            "a single run must be refused, not reported as clean: {e}"
        );
    }

    #[test]
    fn an_unknown_backend_is_refused_by_name() {
        // Only reachable when a compiler exists; when it does not, the earlier
        // refusal fires and is equally correct. Either way the command must not
        // silently walk zero backends and print a clean table.
        let e = determinism(Some("gen-cobol"), Some(1), 2).unwrap_err();
        let m = e.to_string();
        assert!(
            m.contains("gen-cobol") || m.contains("no compiler at"),
            "the refusal must name what it refused: {m}"
        );
    }

    #[test]
    fn every_backend_has_a_distinct_subcommand_and_label() {
        let mut cmds: Vec<&str> = BACKENDS.iter().map(|(c, _)| *c).collect();
        let mut labels: Vec<&str> = BACKENDS.iter().map(|(_, l)| *l).collect();
        let (nc, nl) = (cmds.len(), labels.len());
        cmds.sort_unstable();
        cmds.dedup();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(nc, cmds.len(), "two entries share a subcommand");
        assert_eq!(nl, labels.len(), "two entries share a label");
    }

    #[test]
    fn the_four_emitters_are_all_listed() {
        // A backend missing from this list is a backend this command silently
        // does not measure -- the shape #2988's guard had, and the shape
        // `tri mods orphan` had when it watched two crates of five.
        for cmd in ["gen-c", "gen-rust", "gen", "gen-verilog"] {
            assert!(
                BACKENDS.iter().any(|(c, _)| *c == cmd),
                "`{cmd}` is not measured by this command"
            );
        }
    }
}

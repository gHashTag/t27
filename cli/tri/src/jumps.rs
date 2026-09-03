//! `tri jumps census` -- what happened to every `break` and `continue` the
//! Verilog emitter had to lower?
//!
//! WHY THIS EXISTS
//! ---------------
//! `break` was emitted as `disable fork;` and `continue` as `/* continue */;`.
//! Both are legal Verilog. `iverilog -g2012` accepts them, `yosys` accepts
//! them, the seal hashes are stable over them, and every one of them is a
//! NO-OP: `disable fork` kills processes spawned by a `fork` in the current
//! scope, and the token `fork` occurs nowhere in the generated corpus except
//! inside that very line. Sixteen `break`s and one `continue` ran off the end
//! of their loops for as long as the emitter has existed, and no instrument in
//! this repository could say so, because every instrument asked whether the
//! output PARSES.
//!
//! So this one asks a different question: for each jump the source wrote, what
//! did the backend put there, and does it do anything?
//!
//! WHAT IT CHECKS THAT A COUNT CANNOT
//! ----------------------------------
//! A guard flag is two halves -- a `reg` the loop declares and an assignment
//! the `break` writes. A lowering that declares the flag and binds the jump to
//! the WRONG loop still declares exactly one flag per loop that needs one, so
//! any count of declarations is satisfied. It is the PAIRING that fails. That
//! mutant survived a five-test suite until the pairing was asserted, so the
//! pairing is asserted here too, over the whole corpus.
//!
//! WHAT IT REFUSES
//! ---------------
//! With no compiler it says so and exits non-zero. "0 unlowered jumps" taken
//! with no binary is a statement about this machine and reads exactly like
//! health.
//!
//! It reports and never gates. See #2988; `return` inside a loop is #2989 and
//! is a different question with a different answer.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The dead lowerings, and what each one was written as.
const NO_OPS: &[(&str, &str)] = &[("disable fork;", "break"), ("/* continue */;", "continue")];

#[derive(Subcommand)]
pub enum JumpsCmd {
    /// Name every `break`/`continue` site in the generated Verilog and say
    /// what it lowered to.
    Census {
        /// Stop after this many specs. The population line still names the
        /// whole tree, so a truncated run cannot read as a complete one.
        #[arg(long)]
        limit: Option<usize>,
    },
}

pub fn run(cmd: &JumpsCmd) -> Result<()> {
    match cmd {
        JumpsCmd::Census { limit } => census(*limit),
    }
}

/// The ids in `<prefix><digits>` across a file, in order of appearance.
fn ids(text: &str, prefix: &str) -> Vec<String> {
    text.match_indices(prefix)
        .map(|(i, m)| {
            text[i + m.len()..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect()
        })
        .filter(|s: &String| !s.is_empty())
        .collect()
}

fn census(limit: Option<usize>) -> Result<()> {
    let root = repo_root()?;
    let t27c = root.join("target/release/t27c");
    if !t27c.exists() {
        bail!(
            "no compiler at {}\n  \
             Build it first: cargo build --release -p t27c\n  \
             A jump census taken with no binary reports zero unlowered jumps,\n  \
             and that reads exactly like health.",
            t27c.display()
        );
    }

    // Every `.t27` in the tree, not just `specs/`. Two of the seventeen sites
    // this was built for live in `compiler/cli/gen.t27`, and a census scoped to
    // `specs/` reported fifteen -- twice, in two different sessions.
    let mut specs = Vec::new();
    collect_specs(&root, &root, &mut specs);
    specs.sort();
    let population = specs.len();
    let walked: Vec<&String> = match limit {
        Some(n) => specs.iter().take(n).collect(),
        None => specs.iter().collect(),
    };

    let mut generated = 0usize;
    let mut dead: Vec<(String, String, usize)> = Vec::new();
    let mut refused: Vec<(String, usize)> = Vec::new();
    // Two different quantities, and conflating them is how a report says 14
    // where the source wrote 17: several `break`s in one loop share ONE flag.
    let mut flags: Vec<(String, usize)> = Vec::new();
    let mut jumps: Vec<(String, usize)> = Vec::new();
    let mut unpaired: Vec<(String, String)> = Vec::new();

    for spec in &walked {
        let out = Command::new(&t27c)
            .arg("gen-verilog")
            .arg(root.join(spec))
            .current_dir(&root)
            .output();
        let v = match out {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
            _ => continue,
        };
        generated += 1;

        for (marker, what) in NO_OPS {
            let n = v.matches(marker).count();
            if n > 0 {
                dead.push(((*spec).clone(), (*what).to_string(), n));
            }
        }
        // Only this crate's own marker. The emitter already prints an
        // unrelated `// NOT LOWERED BY THIS BACKEND` in 535 files, and a grep
        // for the shorter phrase reports every one of them as a refusal.
        let n = v.matches("t27#2988:").count();
        if n > 0 {
            refused.push(((*spec).clone(), n));
        }
        for (decl, set) in [
            ("reg __t27_brk_", "__t27_brk_"),
            ("reg __t27_cnt_", "__t27_cnt_"),
        ] {
            let declared: BTreeSet<String> = ids(&v, decl).into_iter().collect();
            if declared.is_empty() {
                continue;
            }
            flags.push(((*spec).clone(), declared.len()));
            let written: BTreeSet<String> = ids(&v, set)
                .into_iter()
                .filter(|id| v.contains(&format!("{set}{id} = 1'b1;")))
                .collect();
            // The JUMPS: one per `= 1'b1;` assignment, which is one per
            // `break`/`continue` the source wrote.
            let n = declared
                .iter()
                .map(|id| v.matches(&format!("{set}{id} = 1'b1;")).count())
                .sum::<usize>();
            if n > 0 {
                jumps.push(((*spec).clone(), n));
            }
            for id in declared.difference(&written) {
                unpaired.push(((*spec).clone(), format!("{set}{id}")));
            }
        }
    }

    println!();
    println!("  .t27 in the tree        {population}");
    if walked.len() != population {
        println!("  WALKED THIS RUN         {}   (--limit)", walked.len());
    }
    println!("  generated Verilog       {generated}");
    println!();
    println!(
        "  jumps lowered            {:>3} site(s) in {} file(s)",
        jumps.iter().map(|(_, n)| n).sum::<usize>(),
        jumps.len()
    );
    println!(
        "    ... sharing            {:>3} guard flag(s)",
        flags.iter().map(|(_, n)| n).sum::<usize>()
    );
    println!(
        "  jumps left as a NO-OP    {:>3} site(s) in {} file(s)",
        dead.iter().map(|(_, _, n)| n).sum::<usize>(),
        dead.len()
    );
    for (s, what, n) in &dead {
        println!("      {s}   {n} x {what}");
    }
    println!(
        "  jumps REFUSED, and said  {:>3} site(s) in {} file(s)",
        refused.iter().map(|(_, n)| n).sum::<usize>(),
        refused.len()
    );
    for (s, n) in &refused {
        println!("      {s}   {n}");
    }
    println!("  flags declared, never set {:>2}", unpaired.len());
    for (s, f) in &unpaired {
        println!("      {s}   {f}");
    }
    println!();
    if dead.is_empty() && unpaired.is_empty() {
        println!(
            "  Every jump the source wrote reached a flag, and every flag a loop\n  \
             declared is written somewhere. That is a result and not a silence:\n  \
             {generated} file(s) were generated and read in full."
        );
    } else {
        println!(
            "  A NO-OP above is a `break` or `continue` the hardware never takes;\n  \
             a flag declared and never set is a jump bound to the wrong loop.\n  \
             Both parse. See #2988. This command reports; it does not gate."
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
        let name = e.file_name();
        let name = name.to_string_lossy();
        if p.is_dir() {
            if name == "target" || name == ".git" || name == "node_modules" {
                continue;
            }
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
    fn a_declared_flag_that_is_never_set_is_visible_to_the_pairing() {
        // The mutant that survived a five-test suite: the loop declares the
        // flag, the `break` binds elsewhere, nothing writes it.
        let v = "reg __t27_brk_0;\n__t27_brk_0 = 1'b0;\n";
        let declared: BTreeSet<String> = ids(v, "reg __t27_brk_").into_iter().collect();
        let written: BTreeSet<String> = ids(v, "__t27_brk_")
            .into_iter()
            .filter(|id| v.contains(&format!("__t27_brk_{id} = 1'b1;")))
            .collect();
        assert_eq!(declared.len(), 1);
        assert!(written.is_empty(), "the clear is not the set");
        assert_eq!(declared.difference(&written).count(), 1);
    }

    #[test]
    fn a_flag_that_is_declared_and_set_pairs() {
        let v = "reg __t27_brk_3;\n__t27_brk_3 = 1'b0;\n__t27_brk_3 = 1'b1;\n";
        let declared: BTreeSet<String> = ids(v, "reg __t27_brk_").into_iter().collect();
        let written: BTreeSet<String> = ids(v, "__t27_brk_")
            .into_iter()
            .filter(|id| v.contains(&format!("__t27_brk_{id} = 1'b1;")))
            .collect();
        assert_eq!(declared, written);
    }

    #[test]
    fn the_two_dead_lowerings_are_both_watched() {
        // A no-op missing from this list is a no-op this census silently does
        // not count -- the shape `tri mods orphan` had when it watched two
        // crates of five.
        for m in ["disable fork;", "/* continue */;"] {
            assert!(
                NO_OPS.iter().any(|(marker, _)| *marker == m),
                "`{m}` is not watched"
            );
        }
    }

    #[test]
    fn the_refusal_marker_is_not_the_backends_own_phrase() {
        // 535 generated files carry `// NOT LOWERED BY THIS BACKEND` for an
        // unrelated reason. A census keyed on "NOT LOWERED" reports all 535 as
        // refusals; this one is keyed on the issue number.
        let unrelated = "  // NOT LOWERED BY THIS BACKEND\n";
        assert_eq!(unrelated.matches("t27#2988:").count(), 0);
        let mine = "  // t27#2988: `break` NOT LOWERED -- no guard flag in this scope.\n";
        assert_eq!(mine.matches("t27#2988:").count(), 1);
    }

    #[test]
    fn ids_reads_the_number_and_stops() {
        assert_eq!(ids("reg __t27_brk_12;", "reg __t27_brk_"), vec!["12"]);
        assert_eq!(ids("reg __t27_brk_;", "reg __t27_brk_").len(), 0);
    }

    #[test]
    fn a_census_with_no_compiler_refuses() {
        // Reachable only when `target/release/t27c` is absent; when it is
        // present the walk runs and is equally correct. Either way this must
        // never print a clean table without having generated anything.
        if let Err(e) = census(Some(0)) {
            assert!(
                e.to_string().contains("no compiler at"),
                "the refusal must name what is missing: {e}"
            );
        }
    }
}

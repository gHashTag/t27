//! `tri mutate` — find the constants in a checker that nothing actually checks.
//!
//! This exists because of one hour. A workflow step was added to catch a
//! toolchain pin that was being silently ignored; the step read an 8-digit date
//! out of `yosys -V`, there is no date in that string, so the variable was
//! empty, the guard skipped, and the step reported success without comparing
//! anything. A vacuous assertion, written to catch a vacuous pin.
//!
//! An hour later the same class appeared in a verifier written to be careful
//! about exactly this: flipping one entry of its lookup table left every claim
//! green, because the table sat on both sides of the identity being tested and
//! cancelled with itself.
//!
//! Neither was caught by reading. Both were caught by changing a constant and
//! noticing nothing went red. That is what this command automates: perturb one
//! literal at a time, re-run the checker, and report every literal the checker
//! did not notice.
//!
//! A surviving mutant is not always a bug — some constants genuinely do not
//! affect the outcome. It is always a question worth answering, because a check
//! that cannot fail is indistinguishable from one that passed.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Subcommand)]
pub enum MutateCmd {
    /// Perturb each numeric literal in a file and report which ones the
    /// checker does not notice.
    Run {
        /// File whose constants are under test.
        #[arg(long)]
        file: String,
        /// Command that must exit 0 when the file is intact.
        #[arg(long)]
        cmd: String,
        /// Stop after this many mutants.
        #[arg(long, default_value_t = 40)]
        max: usize,
    },
}

pub fn run(cmd: &MutateCmd) -> Result<()> {
    match cmd {
        MutateCmd::Run { file, cmd, max } => mutate(Path::new(file), cmd, *max),
    }
}

/// Make the file recoverable before touching it, and say where the copy is.
///
/// The earlier version of this refused to run unless git said the file was
/// clean. That was safe and it was also the wrong trade: every mutation run on
/// work-in-progress needed a throwaway commit first, and a throwaway commit is
/// how a `wip-for-mutation` subject reached a repository whose format gate
/// rejects it -- twice.
///
/// A sibling backup gives the same recovery guarantee without asking the caller
/// to commit anything. Git cleanliness is still reported, because `git checkout`
/// is the nicer recovery path when it is available.
fn make_recoverable(file: &Path, original: &str) -> Result<PathBuf> {
    let backup = file.with_extension(format!(
        "{}.tri-mutate-backup",
        file.extension().and_then(|e| e.to_str()).unwrap_or("bak")
    ));
    std::fs::write(&backup, original)
        .with_context(|| format!("cannot write a backup at {}", backup.display()))?;

    let clean = Command::new("git")
        .args(["status", "--porcelain", "--"])
        .arg(file)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false);

    if clean {
        println!(
            "Recovery: `git checkout -- {}` (also copied to {}).",
            file.display(),
            backup.display()
        );
    } else {
        println!(
            "Recovery: {} (the file has uncommitted changes, so git cannot restore it).",
            backup.display()
        );
    }
    Ok(backup)
}

/// Delete bytecode caches derived from this file.
///
/// Verifying that the source came back byte-for-byte turned out not to be
/// enough. Most mutations here preserve the file's LENGTH -- `5` becomes `6`,
/// `16` becomes `17` -- and Python decides a `.pyc` is current by comparing the
/// source's (mtime, size). Restore the file inside the same filesystem second
/// and both match, so the interpreter serves bytecode compiled from the mutant.
///
/// Measured, not theorised: a benchmark's format table read back as `e5m11` and
/// then `e6m10` on consecutive runs while the file on disk said `e5m10` both
/// times. Clearing the cache made every assertion pass.
///
/// So the restore has to reach the derived artefacts too, or the next
/// measurement in that session is against a mutant nobody can see.
pub(crate) fn clear_derived_caches(file: &Path) {
    let stem = match file.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return,
    };
    let dir = match file.parent() {
        Some(d) => d.join("__pycache__"),
        None => return,
    };
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(&format!("{stem}.")) && name.ends_with(".pyc") {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Mutant {
    line: usize,
    /// 1-based column. Without it, two identical literals on one line produce
    /// two identical report rows: `111  5 -> 6` twice, one caught and one a
    /// survivor. I read such a report, checked the wrong `5` by hand, and
    /// concluded the tool was lying.
    col: usize,
    from: String,
    to: String,
    byte: usize,
    len: usize,
}

/// Byte offsets that are inside a comment or a string literal.
///
/// The first version of this command skipped this step and reported 26
/// survivors on a 200-line verifier. Every one of them was a number in a
/// docstring or a human-readable message — mutating prose cannot fail a check,
/// so each was a guaranteed false survivor, and the one real result was buried
/// under them. A tool whose output is mostly noise gets ignored, which is the
/// failure this whole command exists to prevent.
///
/// Handles `#`, `//`, block comments, quotes and Python triple-quotes. That
/// covers Python, Verilog, Rust, YAML and shell, which is what these checkers
/// are written in.
fn masked(text: &str) -> Vec<bool> {
    const TRIPLE_D: &str = "\"\"\"";
    const TRIPLE_S: &str = "'''";
    let b = text.as_bytes();
    let mut mask = vec![false; b.len()];
    let mut i = 0usize;
    let mark = |mask: &mut Vec<bool>, from: usize, to: usize| {
        for m in mask.iter_mut().take(to.min(b.len())).skip(from) {
            *m = true;
        }
    };
    while i < b.len() {
        let rest = &text[i..];
        // Triple quotes first: a docstring opener would be misread as an
        // ordinary quote and closed three bytes later.
        if rest.starts_with(TRIPLE_D) || rest.starts_with(TRIPLE_S) {
            let q = if rest.starts_with(TRIPLE_D) {
                TRIPLE_D
            } else {
                TRIPLE_S
            };
            let end = rest[3..].find(q).map(|p| i + 3 + p + 3).unwrap_or(b.len());
            mark(&mut mask, i, end);
            i = end;
            continue;
        }
        // Raw strings: `r"..."`, `r#"..."#`, `r##"..."##`, and the byte forms
        // `br#"..."#`. This is not decoration for a language-agnostic masker.
        // `#` opens a comment two rules down, so `r#"` was read as `r` followed
        // by a comment to end of line -- and the string's CONTENTS came back
        // marked as code. Measured with the tool itself: a file holding
        // `pub const FIXTURE: &str = r#"\n threshold = 12345\n other = 6789\n"#;`
        // was reported as "4 literal(s)", offering 12345 and 6789 as constants
        // to perturb. Mutating fixture text and reading the resulting red as
        // "the checker noticed" is the same tautology `drop_test_module_sites`
        // exists to prevent, reached through a different door.
        //
        // Safe in the other languages this runs on: none of them spells a raw
        // string this way, and Python's `r"..."` is a string, so masking it is
        // right there too.
        //
        // A `br#"..."#` branch stood here too, skipping the byte-string prefix.
        // Mutation removed it and nothing failed: `br#"` is `b` followed by
        // `r#"`, so the rule fires one byte later and masks the same content.
        // Only the `b` itself stays unmasked, and `b` is not a digit. Measured
        // on `br#"bytes 4242 here"#`: the same two mutants either way.
        {
            if b[i] == b'r' {
                let mut k = i + 1;
                let mut hashes = 0usize;
                while k < b.len() && b[k] == b'#' {
                    hashes += 1;
                    k += 1;
                }
                // A `!prev_is_word` guard stood here, to stop `str"` or `xr"`
                // from opening a raw string. Mutation removed it and every test
                // stayed green, so I checked why rather than writing a test to
                // cover it: the ordinary-string rule below reaches those bytes
                // FIRST and masks the same span. Run on
                // `let _ = xr"junk 55 junk";` the tool reports the identical two
                // mutants either way -- the guard cannot change what this
                // function returns. Two rules with one testable consequence is
                // one rule and a decoration.
                if k < b.len() && b[k] == b'"' {
                    let mut close = String::from("\"");
                    close.push_str(&"#".repeat(hashes));
                    let end = text[k + 1..]
                        .find(&close)
                        .map(|p| k + 1 + p + close.len())
                        .unwrap_or(b.len());
                    mark(&mut mask, i, end);
                    i = end;
                    continue;
                }
            }
        }
        if b[i] == b'#' || rest.starts_with("//") {
            let end = rest.find('\n').map(|p| i + p).unwrap_or(b.len());
            mark(&mut mask, i, end);
            i = end;
            continue;
        }
        if rest.starts_with("/*") {
            let end = rest.find("*/").map(|p| i + p + 2).unwrap_or(b.len());
            mark(&mut mask, i, end);
            i = end;
            continue;
        }
        if b[i] == b'"' || b[i] == b'\'' {
            let q = b[i];
            let mut j = i + 1;
            while j < b.len() && b[j] != q {
                // A newline ends an unterminated quote rather than swallowing
                // the rest of the file, which an apostrophe in prose would do.
                if b[j] == b'\n' {
                    break;
                }
                if b[j] == b'\\' {
                    j += 1;
                }
                j += 1;
            }
            let end = (j + 1).min(b.len());
            mark(&mut mask, i, end);
            i = end;
            continue;
        }
        i += 1;
    }
    mask
}

/// Drop the sites that sit inside a Rust `#[cfg(test)]` module.
///
/// Returns the survivors and how many were dropped, so the count can be
/// PRINTED rather than silently applied -- a population that shrinks without
/// saying so is the defect one level up from the one this fixes.
fn drop_test_module_sites(file: &Path, text: &str, all: Vec<Mutant>) -> (Vec<Mutant>, usize) {
    if file.extension().and_then(|e| e.to_str()) != Some("rs") {
        return (all, 0);
    }
    let mask = crate::gates::test_module_lines(text);
    let before = all.len();
    let kept: Vec<Mutant> = all
        .into_iter()
        .filter(|m| !mask.get(m.line.saturating_sub(1)).copied().unwrap_or(false))
        .collect();
    let dropped = before - kept.len();
    (kept, dropped)
}

/// Every integer literal in the file, with a perturbed value.
///
/// Deliberately numeric-only and deliberately dumb. A parser per language would
/// be a better mutation engine and a worse tool: this one runs on a Python
/// oracle, a Verilog header and a YAML workflow without knowing which is which.
fn find_mutants(text: &str, max: usize) -> Vec<Mutant> {
    let bytes = text.as_bytes();
    let mask = masked(text);
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    while i < bytes.len() && out.len() < max {
        if bytes[i] == b'\n' {
            line += 1;
            i += 1;
            continue;
        }
        if !bytes[i].is_ascii_digit() || mask[i] {
            i += 1;
            continue;
        }
        // `line` is maintained by the top of this loop, which walks every byte
        // including those inside comments, so masked regions still advance it.
        // Guarded by tests rather than left to be re-derived by the next
        // reader: I misread this once and accused the counter of a bug it did
        // not have.
        // Don't split an identifier like `sha1` or `gf16` — a digit is only a
        // literal if what precedes it cannot be part of a name.
        let prev_is_word = i > 0
            && (bytes[i - 1].is_ascii_alphanumeric()
                || bytes[i - 1] == b'_'
                || bytes[i - 1] == b'.');
        let start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            i += 1;
        }
        debug_assert!(
            !text[start..i].contains('\n'),
            "a token must not span a newline, or the line counter is wrong"
        );
        if prev_is_word {
            continue;
        }
        let tok = &text[start..i];
        // Hex, binary and anything with a letter in it is left alone: mutating
        // `0x3FCF1BBD` by +1 is meaningful, but `1e12` and `0b10` are not
        // reliably parsed here, and a wrong mutant wastes a whole run.
        let (from, to) = if let Some(h) = tok.strip_prefix("0x").or_else(|| tok.strip_prefix("0X"))
        {
            match u64::from_str_radix(h, 16) {
                Ok(v) => (tok.to_string(), format!("0x{:X}", v.wrapping_add(1))),
                Err(_) => continue,
            }
        } else {
            match tok.parse::<i64>() {
                Ok(v) => (tok.to_string(), (v + 1).to_string()),
                Err(_) => continue,
            }
        };
        let line_start = text[..start].rfind('\n').map(|p| p + 1).unwrap_or(0);
        out.push(Mutant {
            line,
            col: start - line_start + 1,
            from,
            to,
            byte: start,
            len: tok.len(),
        });
    }
    out
}

fn passes(cmd: &str) -> Result<bool> {
    let out = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .context("failed to run the checker command")?;
    Ok(out.status.success())
}

/// The mutants to run, and whether `max` cut the walk short.
///
/// Truncation is detected by asking `find_mutants` for one MORE than the cap
/// and seeing whether it comes back. `probe.len() > max` and not `>=`: a file
/// holding exactly `max` literals was walked to the end and is not truncated.
/// That distinction is the whole point -- it decides whether the report may
/// speak about the file or only about a prefix of it -- and it lived inline in
/// `mutate`, where no test could reach it: flipping `>` to `>=` left all 760
/// tests green.
fn mutants_and_truncation(text: &str, max: usize) -> (Vec<Mutant>, bool) {
    let probe = find_mutants(text, max.saturating_add(1));
    let truncated = probe.len() > max;
    (probe.into_iter().take(max).collect(), truncated)
}

fn mutate(file: &Path, cmd: &str, max: usize) -> Result<()> {
    let original =
        std::fs::read_to_string(file).with_context(|| format!("cannot read {}", file.display()))?;
    let backup = make_recoverable(file, &original)?;

    // A checker that is already failing cannot tell us anything about a
    // mutant: every mutant would "survive" by looking exactly like the
    // baseline. Establish the baseline before changing a byte.
    if !passes(cmd)? {
        bail!(
            "the checker does not pass on the unmodified file, so no mutant \
             would mean anything. Fix it first, then run this."
        );
    }

    // Ask for one more than the cap allows. If we get it, the walk was
    // TRUNCATED and every count below describes a prefix of the file rather
    // than the file. Without this the tool prints "N literal(s) in <file>"
    // where N is the cap, and -- when nothing survives -- "Nothing in this
    // file is decorative", which is an assertion of ABSENCE over a region the
    // reader is never told was bounded.
    //
    // Measured on cli/tri/src/fpga.rs: the default cap of 40 stops at line
    // 1764 of 10819 and reports 40, where the file holds 376 production
    // literals (plus 637 inside `#[cfg(test)]`, skipped separately). 10.6% of
    // the population, presented as all of it.
    let (all, truncated) = mutants_and_truncation(&original, max);
    // A literal inside `#[cfg(test)]` is not a constant the checker fails to
    // check -- it is the checker's own arithmetic. Perturbing it breaks the test
    // that holds it, and that red is reported as the checker NOTICING, which is
    // a tautology: something went red and nothing was learned about production.
    //
    // Measured 2026-09-05 on this crate: 45 of the 59 sites this tool finds in
    // `red.rs` are inside its test module -- 76% -- and 1545 of 3198 across the
    // whole crate. Reproduced end to end: perturbing `render_headline(50, ...)`
    // in a test call fails the suite, and `50` is a number that appears only in
    // that test.
    //
    // Rust only, by the same rule `gates` uses. The tool is deliberately
    // language-agnostic and runs on Python, Verilog and YAML, none of which have
    // `#[cfg(test)]`; there the population is unchanged.
    let (mutants, skipped) = drop_test_module_sites(file, &original, all);
    if skipped > 0 {
        println!(
            "  {skipped} literal(s) skipped: they sit inside a `#[cfg(test)]` module.\n  \
             Perturbing a test's own arithmetic fails that test, and reporting it as\n  \
             `the checker noticed` says nothing about the code under test.\n"
        );
    }
    if mutants.is_empty() {
        println!("No numeric literals found in {}.", file.display());
        return Ok(());
    }

    if truncated {
        println!(
            "{} literal(s) in {}, one mutation each.\n\
             THIS IS A PREFIX, NOT THE FILE: the walk stopped at the --max of \
             {} and there are more beyond it. Every count below, and any claim \
             that nothing survived, describes only what was reached. Raise \
             --max to cover the file.\n",
            mutants.len(),
            file.display(),
            max
        );
    } else {
        println!(
            "{} literal(s) in {}, one mutation each.\n",
            mutants.len(),
            file.display()
        );
    }

    let mut survivors = Vec::new();
    for (n, m) in mutants.iter().enumerate() {
        let mut text = String::with_capacity(original.len());
        text.push_str(&original[..m.byte]);
        text.push_str(&m.to);
        text.push_str(&original[m.byte + m.len..]);
        std::fs::write(file, &text)?;
        let survived = passes(cmd).unwrap_or(false);
        std::fs::write(file, &original)?;
        clear_derived_caches(file);

        // Verify the restore instead of assuming it. A measurement taken
        // against a file this command left perturbed is not a measurement, and
        // that is not hypothetical: a perturbed constant survived a hand-run
        // mutation on this machine, was read back as if it were the real value,
        // and produced a written-up finding that did not exist. Failing loudly
        // here costs one read per mutant and makes that silent.
        let back = std::fs::read_to_string(file)
            .with_context(|| format!("cannot re-read {} after restoring it", file.display()))?;
        if back != original {
            bail!(
                "{} was NOT restored after mutating line {}. Recover it from {} \
                 before trusting any measurement taken against it.",
                file.display(),
                m.line,
                backup.display()
            );
        }

        print!("\r  {}/{}   ", n + 1, mutants.len());
        use std::io::Write;
        let _ = std::io::stdout().flush();

        if survived {
            survivors.push(m);
        }
    }
    println!("\r                    ");

    clear_derived_caches(file);
    let _ = std::fs::remove_file(&backup);
    if survivors.is_empty() {
        if truncated {
            println!(
                "Every one of the {} literals REACHED changed the outcome. That \
                 is a statement about the first {} literals, not about this \
                 file: the walk stopped at --max. Nothing here says the rest \
                 are not decorative, because the rest were never mutated.",
                mutants.len(),
                max
            );
        } else {
            println!(
                "Every one of the {} literals changed the outcome. Nothing in \
                 this file is decorative.",
                mutants.len()
            );
        }
        return Ok(());
    }

    println!(
        "{} of {} mutations SURVIVED — the checker did not notice:\n",
        survivors.len(),
        mutants.len()
    );
    for m in &survivors {
        println!(
            "  {}:{}:{}  {} -> {}",
            file.display(),
            m.line,
            m.col,
            m.from,
            m.to
        );
    }
    println!();
    println!("A survivor is a question, not a verdict: some constants genuinely");
    println!("do not affect the outcome. But a check that cannot fail is");
    println!("indistinguishable from one that passed, so answer each of them.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A literal inside `#[cfg(test)]` is the checker's own arithmetic, not a
    /// constant the checker fails to check. Perturbing it fails the test that
    /// holds it, and that red was reported as the checker NOTICING.
    ///
    /// Measured 2026-09-05: 45 of the 59 sites this tool finds in `red.rs` are
    /// inside its test module, and 1545 of 3198 across the crate. Reproduced end
    /// to end: perturbing `render_headline(50, ...)` in a test call fails the
    /// suite, and that `50` appears only in that test.
    #[test]
    fn a_literal_inside_a_test_module_is_not_a_site() {
        let src = "const CAP: usize = 30;\n#[cfg(test)]\nmod t {\n    #[test]\n    fn a() {\n        assert_eq!(f(7), 8);\n    }\n}\n";
        let all = find_mutants(src, 40);
        assert_eq!(
            all.len(),
            3,
            "the raw finder sees all three: 30, 7 and 8 -- {all:?}"
        );
        let (kept, dropped) = drop_test_module_sites(Path::new("x.rs"), src, all);
        assert_eq!(dropped, 2, "the two inside the test module are dropped");
        assert_eq!(kept.len(), 1, "the production constant remains");
        assert_eq!(kept[0].from, "30", "and it is the right one: {kept:?}");
    }

    /// The filter can be right while `mutate` never calls it.
    ///
    /// Replacing the call with `(all, 0)` leaves the three value-level tests
    /// here green and restores the defect exactly. This is the FIFTH change in
    /// five passes whose surviving mutant was the wiring rather than the
    /// function, and the first one I went looking for before running it.
    ///
    /// The needle is split across two literals so this test's own body does not
    /// contain the string it searches for -- a structural test that finds itself
    /// passes against its own mutant, which happened once already.
    #[test]
    fn mutate_actually_calls_the_filter() {
        let src = include_str!("mutate.rs");
        let boundary = src
            .lines()
            .position(|l| l == "#[cfg(test)]")
            .expect("the test module is a line of its own, not a mention in prose");
        let code: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");
        let call = concat!("drop_test_module_", "sites(file, &original, all)");
        assert!(
            code.contains(call),
            "the filter has to be reached from `mutate`, or a test-module literal \
             is perturbed and its red is reported as the checker noticing"
        );
    }

    /// The tool runs on a Python oracle, a Verilog header and a YAML workflow.
    /// None of them has `#[cfg(test)]`, and none may lose a site to a rule
    /// written for Rust.
    #[test]
    fn a_non_rust_file_keeps_every_site() {
        let src = "CAP = 30\n# cfg(test) is not a thing here\nassert f(7) == 8\n";
        let all = find_mutants(src, 40);
        let n = all.len();
        assert!(n >= 3, "the finder sees the literals: {all:?}");
        for name in ["x.py", "x.v", "x.yml", "x"] {
            let (kept, dropped) = drop_test_module_sites(Path::new(name), src, all.clone());
            assert_eq!(dropped, 0, "{name}: no Rust rule may apply");
            assert_eq!(kept.len(), n, "{name}: every site survives");
        }
    }

    /// A population that shrinks without saying so is the defect one level up
    /// from the one this fixes, so the count comes back to be printed.
    #[test]
    fn mutate_asks_the_helper_rather_than_assuming_a_full_walk() {
        // Extracting `mutants_and_truncation` made the PREDICATE testable and
        // left the WIRING open: replacing the call with
        // `(find_mutants(&original, max), false)` keeps every count correct,
        // silently drops the prefix warning, and left all 760 tests green.
        // Twelfth time this pass shape has recurred, so the call site gets its
        // own reader.
        let src = include_str!("mutate.rs");
        // NOT `split("#[cfg(test)]")`: that literal appears FIVE times in doc
        // comments and string literals in this file before the real attribute
        // (lines 216, 355, 358, 371, 375), the earliest at 216, so the split
        // cuts well above `mutate` and the slice never reaches the subject.
        // The attribute is a line of its own; match it as one.
        //
        // "SIX" stood here until an audit recounted: twelve occurrences in the
        // file, the attribute at 487, five above it. The count was never six.
        let boundary = src
            .lines()
            .position(|l| l == concat!("#[cfg(te", "st)]"))
            .expect("the test module attribute is a line of its own");
        let prod: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");
        assert!(
            prod.contains("fn mutate(file: &Path"),
            "the production slice no longer reaches `mutate` -- the assertion \
             below would pass vacuously"
        );
        let call = concat!("mutants_and_truncation(&ori", "ginal, max)");
        assert!(
            prod.contains(call),
            "`mutate` must ask the helper. Building the pair inline is how the \
             truncation flag gets hardcoded to false without any test noticing."
        );
    }

    #[test]
    fn a_raw_string_is_not_code() {
        // `#` opens a comment in this masker, so `r#"` was read as `r` plus a
        // comment to end of line and the string's CONTENTS came back as code.
        // Measured with the tool: the fixture below was reported as
        // "4 literal(s)" and offered 12345 and 6789 to perturb.
        let src = "pub fn a() -> u32 { 7 }\n\
                   pub const F: &str = r#\"\n  threshold = 12345\n\"#;\n\
                   pub fn b() -> u32 { 42 }\n";
        let got: Vec<String> = find_mutants(src, 40)
            .iter()
            .map(|m| m.from.clone())
            .collect();
        assert_eq!(
            got,
            vec!["7", "42"],
            "12345 is string content, not a constant"
        );

        // Hash counting: the close is `"` plus the SAME number of `#`, so a
        // `"#` inside an r##"..."## does not end it.
        let two = "let a = r##\"has \"# inside 999\"##; let n = 5;";
        let got: Vec<String> = find_mutants(two, 40)
            .iter()
            .map(|m| m.from.clone())
            .collect();
        assert_eq!(got, vec!["5"], "999 sits inside the r## string: {got:?}");

        // A normal string ENDING in `r` must not open one -- `"abcr"` puts an
        // `r"` in the text, and reading that as a raw-string opener would mask
        // the rest of the file.
        let ends_r = "let a = \"abcr\"; let n = 5; let m = 6;";
        let got: Vec<String> = find_mutants(ends_r, 40)
            .iter()
            .map(|m| m.from.clone())
            .collect();
        assert_eq!(
            got,
            vec!["5", "6"],
            "the code after `\"abcr\"` is still code"
        );

        // Byte raw strings too.
        let byte = "let a = br#\"77\"#; let n = 5;";
        let got: Vec<String> = find_mutants(byte, 40)
            .iter()
            .map(|m| m.from.clone())
            .collect();
        assert_eq!(got, vec!["5"], "br#\"..\"# is a string: {got:?}");

        // An UNTERMINATED raw string masks to end of file rather than stopping
        // at the quote. Perturbing a constant that is really fixture text is
        // the failure this whole rule exists to prevent, and a truncated file
        // is exactly when the closer is missing. Every other rule here ends the
        // same way; the ordinary-string rule is the one deliberate exception,
        // because an apostrophe in prose would otherwise swallow the file.
        let cut = "let n = 5;\nlet a = r#\"unterminated 4242";
        let got: Vec<String> = find_mutants(cut, 40)
            .iter()
            .map(|m| m.from.clone())
            .collect();
        assert_eq!(
            got,
            vec!["5"],
            "4242 is inside the unterminated string: {got:?}"
        );
    }

    #[test]
    fn a_capped_walk_says_so() {
        // The tool printed "N literal(s) in <file>" where N was the --max, and
        // on an all-killed run "Nothing in this file is decorative" -- a claim
        // about the FILE from a prefix of it. Measured on cli/tri/src/fpga.rs:
        // the default cap of 40 stops at line 1764 of 10819 and the file holds
        // 376 production literals.
        let src = "fn a() { let x = 1; let y = 2; let z = 3; }";
        assert_eq!(find_mutants(src, 40).len(), 3, "uncapped sees all three");
        assert_eq!(find_mutants(src, 2).len(), 2, "capped sees two");
        // The detection is `ask for one more than the cap and see if you get
        // it`, so it must be exact at the boundary: a file with exactly `max`
        // literals is NOT truncated.
        assert_eq!(find_mutants(src, 3).len(), 3);
        assert_eq!(
            find_mutants(src, 4).len(),
            3,
            "asking for more than exist returns what exists -- this is what \
             distinguishes a full walk from a truncated one"
        );

        // The flag itself, at the boundary. `>=` here would call a complete
        // walk truncated and print the prefix warning on every clean run.
        let (got, trunc) = mutants_and_truncation(src, 2);
        assert_eq!(got.len(), 2, "the cap still limits what is returned");
        assert!(trunc, "two of three literals is a truncated walk");
        assert!(
            !mutants_and_truncation(src, 3).1,
            "exactly max is NOT truncated"
        );
        assert!(
            !mutants_and_truncation(src, 4).1,
            "fewer than max is NOT truncated"
        );
        assert_eq!(
            mutants_and_truncation(src, 3).0.len(),
            3,
            "and it returns all of them"
        );
    }

    #[test]
    fn the_number_dropped_is_returned_to_be_printed() {
        let src = "let a = 1;\n#[cfg(test)]\nmod t {\n    fn z() { let b = 2; let c = 3; }\n}\n";
        let all = find_mutants(src, 40);
        let before = all.len();
        let (kept, dropped) = drop_test_module_sites(Path::new("x.rs"), src, all);
        assert_eq!(
            kept.len() + dropped,
            before,
            "every site is either kept or counted as dropped -- none may vanish"
        );
        assert!(dropped > 0, "this fixture has test-module literals to drop");
    }

    /// `gf16` and `sha1` are names, not constants. An early version of this
    /// scanner mutated the `16` in `gf16` and produced a mutant that failed for
    /// a reason unrelated to any check — noise that reads exactly like signal.
    #[test]
    fn digits_inside_an_identifier_are_not_literals() {
        let m = find_mutants("let gf16 = 9;\nsha1 + 2\n", 20);
        let got: Vec<&str> = m.iter().map(|x| x.from.as_str()).collect();
        assert_eq!(got, vec!["9", "2"], "identifier digits must be skipped");
    }

    /// Two identical literals on one line must be distinguishable, or a
    /// survivor cannot be told from the one beside it that was caught.
    #[test]
    fn identical_literals_on_one_line_get_different_columns() {
        let m = find_mutants("x = 5 * b > 5 * c\n", 20);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].line, m[1].line);
        assert_ne!(m[0].col, m[1].col, "the columns must differ");
        assert_eq!(m[0].col, 5);
    }

    #[test]
    fn hex_is_mutated_as_hex() {
        let m = find_mutants("WANT = 0x3FCF1BBD\n", 20);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].from, "0x3FCF1BBD");
        assert_eq!(m[0].to, "0x3FCF1BBE");
    }

    /// Numbers in prose cannot fail a check, so mutating them manufactures
    /// false survivors. The first run of this command produced 26 of them and
    /// one real result, which is a tool nobody would read twice.
    #[test]
    fn numbers_in_comments_and_strings_are_not_mutated() {
        let src = "# phi^2 = phi + 1\nWANT = 7\nmsg = \"1 < phi < 2\"\n";
        let m = find_mutants(src, 20);
        let got: Vec<&str> = m.iter().map(|x| x.from.as_str()).collect();
        assert_eq!(got, vec!["7"], "only the assignment is a real literal");
    }

    /// The line number is the only part of the report a human navigates by, so
    /// it is counted rather than estimated.
    #[test]
    fn line_numbers_survive_multiline_input() {
        let m = find_mutants("a\nb\n= 7\n", 20);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].line, 3);
    }

    /// The test above uses input with no comments in it, so it cannot tell
    /// whether masked regions advance the line counter. These two can. Added
    /// after I wrongly accused the counter of losing newlines — the counter was
    /// right, and an untested property that happens to hold is still untested.
    #[test]
    fn line_numbers_count_newlines_inside_comments_and_strings() {
        let src = "x = 1\n# one\n# two\n# three\ny = 2\n";
        let m = find_mutants(src, 20);
        let lines: Vec<usize> = m.iter().map(|x| x.line).collect();
        assert_eq!(lines, vec![1, 5], "the three comment lines must be counted");
    }

    #[test]
    fn line_numbers_count_newlines_inside_a_docstring() {
        let src = "a = 1\n\"\"\"\nline\nline\nline\n\"\"\"\nb = 2\n";
        let m = find_mutants(src, 20);
        let lines: Vec<usize> = m.iter().map(|x| x.line).collect();
        assert_eq!(lines, vec![1, 7], "a multi-line docstring must be counted");
    }
}

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
        println!("Recovery: `git checkout -- {}` (also copied to {}).", file.display(), backup.display());
    } else {
        println!("Recovery: {} (the file has uncommitted changes, so git cannot restore it).", backup.display());
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
fn clear_derived_caches(file: &Path) {
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
            let q = if rest.starts_with(TRIPLE_D) { TRIPLE_D } else { TRIPLE_S };
            let end = rest[3..].find(q).map(|p| i + 3 + p + 3).unwrap_or(b.len());
            mark(&mut mask, i, end);
            i = end;
            continue;
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
            && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_' || bytes[i - 1] == b'.');
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
        let (from, to) = if let Some(h) = tok.strip_prefix("0x").or_else(|| tok.strip_prefix("0X")) {
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

fn mutate(file: &Path, cmd: &str, max: usize) -> Result<()> {
    let original = std::fs::read_to_string(file)
        .with_context(|| format!("cannot read {}", file.display()))?;
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

    let mutants = find_mutants(&original, max);
    if mutants.is_empty() {
        println!("No numeric literals found in {}.", file.display());
        return Ok(());
    }

    println!(
        "{} literal(s) in {}, one mutation each.\n",
        mutants.len(),
        file.display()
    );

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
        println!(
            "Every one of the {} literals changed the outcome. Nothing in this \
             file is decorative.",
            mutants.len()
        );
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

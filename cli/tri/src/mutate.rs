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
    /// Drop guards, flip operators and empty whole bodies inside the functions
    /// of a .t27 spec, and report the mutants its own tests do not notice.
    Spec {
        /// The .t27 spec. It is never edited: every mutant is a copy in a
        /// temporary directory, lowered with `t27c gen` and run with `zig test`.
        #[arg(long)]
        file: String,
        /// Mutate only the body of this function.
        #[arg(long = "fn")]
        func: Option<String>,
        /// Stop after this many mutants. The walk is in file order, so a cut
        /// leaves the END of the spec unmutated; with #7022's kinds
        /// specs/queen/actors.t27 alone has 261, hence 1000 and not 200.
        #[arg(long, default_value_t = 1000)]
        max: usize,
        /// Mutants run at once: 4 by default, 8 with --lab (trap T11), where
        /// the lab's free pids may cut it (specs/tri/mutate/lab.t27 lab_jobs).
        #[arg(long)]
        jobs: Option<usize>,
        /// zig's -j for each mutant's compile. Default: the cores shared by
        /// the jobs (specs/tri/mutate/lab.t27 zig_j). On the lab, 8 jobs at
        /// zig's own default held 394 pids and at -j6 58, in the same time.
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..=4096))]
        zig_threads: Option<u32>,
        /// Seconds each step of a mutant (`t27c gen`, the zig compile, the test
        /// run) may take, each on its own clock. Only the test run outliving it
        /// is a HANG; gen or the compile outliving it is the machine's load, and
        /// the mutant is NOT RUN (#7148).
        #[arg(long, default_value_t = 300)]
        timeout: u64,
        /// The t27c binary; default target/release/t27c, then t27c on PATH.
        #[arg(long)]
        t27c: Option<String>,
        /// Exit 2 when a mutant survives or hangs and --accepted does not name
        /// it (specs/tri/mutate/survivors.t27, #7303). Without this flag or
        /// --accepted, survivors exit 0 as before.
        #[arg(long)]
        fail_on_survived: bool,
        /// File of accepted mutants, one per line as this command prints them
        /// under SURVIVED or HUNG: `path:line [kind]`; text after `]` is a
        /// note, `#` starts a comment line. Turns the gate on. A listed mutant
        /// that is now killed fails the gate too, until its line is removed.
        #[arg(long)]
        accepted: Option<String>,
        /// Run on the Railway lab instead (#7050). The spec and the accepted
        /// file go up over `railway ssh`, the run is a nohup'd job in its own
        /// directory, and the same command called again reads it back. Exit
        /// 0..2 are the tool's codes, 3 = no result, 4 = orphaned processes,
        /// 5 = still running (specs/tri/mutate/lab.t27). The lab, its binaries
        /// and the runs' directory are T27C_LAB_* variables (`LabEnv`).
        #[arg(long)]
        lab: bool,
        /// With --lab: seconds to wait for the run before exit 5 (default 3600).
        #[arg(long)]
        lab_wait: Option<u64>,
    },
}

pub fn run(cmd: &MutateCmd) -> Result<()> {
    match cmd {
        MutateCmd::Run { file, cmd, max } => mutate(Path::new(file), cmd, *max),
        MutateCmd::Spec {
            file,
            func,
            max,
            jobs,
            zig_threads,
            timeout,
            t27c,
            fail_on_survived,
            accepted,
            lab,
            lab_wait,
        } if *lab => {
            if t27c.is_some() {
                bail!("--t27c names a binary here; on the lab set T27C_LAB_BIN instead");
            }
            let run = lab_request(
                file,
                func.as_deref(),
                accepted.as_deref(),
                *max,
                jobs.unwrap_or(8),
                *zig_threads,
                *timeout,
                *fail_on_survived,
                lab_wait.unwrap_or(3600),
            )?;
            let code = lab_mutate_spec(&LabEnv::from_env(), &run, &mut std::io::stdout(), &|s| {
                std::thread::sleep(std::time::Duration::from_secs(s))
            })
            .unwrap_or_else(|e| {
                println!("error: {e:#}");
                lab::EXIT_NO_RESULT
            });
            if code != lab::EXIT_OK {
                use std::io::Write;
                let _ = std::io::stdout().flush();
                std::process::exit(code as i32);
            }
            Ok(())
        }
        MutateCmd::Spec {
            file,
            func,
            max,
            jobs,
            zig_threads,
            timeout,
            t27c,
            fail_on_survived,
            accepted,
            lab_wait,
            ..
        } => {
            if lab_wait.is_some() {
                bail!("--lab-wait goes with --lab");
            }
            let jobs = jobs.unwrap_or(4);
            let cores = std::thread::available_parallelism().map(|n| n.get() as u32).unwrap_or(1);
            mutate_spec(
                Path::new(file),
                func.as_deref(),
                *max,
                jobs,
                zig_threads.unwrap_or_else(|| lab::zig_j(cores, jobs as u32)),
                *timeout,
                t27c.as_deref(),
                &GateArgs {
                    fail_on_survived: *fail_on_survived,
                    accepted: accepted.as_deref(),
                },
            )
        }
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

// ---------------------------------------------------------------------------
// `tri mutate spec` -- guard, operator and body mutants of a .t27 spec.
//
// `run` perturbs numeric literals, which finds a constant nothing checks. A
// spec's tests miss in another way: a guard no test reaches, a `>=` that could
// be `>`, an `and` that could be `or`. On specs/queen/actors.t27 (#6963, #6971)
// hand-written lists of such mutants found 6 test gaps and 2 dead lines that
// every test had passed over, and each loop tick rewrote the list in /tmp.
// This writes the list instead (#6993). The first version had no arithmetic
// mutants, so a hash or a jitter reported "1 of 1 killed" while a wrong
// multiplier survived its tests; `swap-arith` and `ret-default` close that
// (#7022). Numeric constants stay with `tri mutate run` and the hand list.

/// One mutant: line `line` (1-based) of the spec reads `after` instead of
/// `before`, and lines `line + 1 ..= through` read as empty lines. A dropped
/// line reads as an empty line, so line numbers in a compiler error still
/// point at the original.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpecMutant {
    pub line: usize,
    /// The last line the mutant replaces: `line` itself, except for a
    /// `ret-default` that empties a body spread over several lines.
    pub through: usize,
    pub kind: &'static str,
    pub before: String,
    pub after: String,
}

/// Which bytes of one spec line are code: not inside a string literal, not
/// after `//`, and nothing at all on a `;` comment line.
fn t27_code_mask(line: &str) -> Vec<bool> {
    let b = line.as_bytes();
    let mut mask = vec![false; b.len()];
    if line.trim_start().starts_with(';') {
        return mask;
    }
    let mut in_str = false;
    let mut i = 0;
    while i < b.len() {
        if in_str {
            if b[i] == b'\\' {
                i += 2;
                continue;
            }
            if b[i] == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if b[i] == b'"' {
            in_str = true;
            i += 1;
            continue;
        }
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
            break;
        }
        mask[i] = true;
        i += 1;
    }
    mask
}

fn count_code(line: &str, mask: &[bool], c: u8) -> i64 {
    line.bytes()
        .zip(mask.iter())
        .filter(|(b, m)| **m && *b == c)
        .count() as i64
}

/// The function a line opens, when it is a `fn` or `pub fn` header.
fn t27_fn_header(line: &str) -> Option<String> {
    let t = line.trim_start();
    let rest = t.strip_prefix("pub fn ").or_else(|| t.strip_prefix("fn "))?;
    let name: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn is_word_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// The mutants of one line inside a function body.
fn t27_line_mutants(lineno: usize, line: &str, mask: &[bool], out: &mut Vec<SpecMutant>) {
    let b = line.as_bytes();
    let code = |i: usize, len: usize| i + len <= b.len() && mask[i..i + len].iter().all(|m| *m);
    let mut push = |kind: &'static str, at: usize, len: usize, to: &str| {
        let mut after = String::with_capacity(line.len());
        after.push_str(&line[..at]);
        after.push_str(to);
        after.push_str(&line[at + len..]);
        out.push(SpecMutant {
            line: lineno,
            through: lineno,
            kind,
            before: line.to_string(),
            after,
        });
    };

    // A one-line guard `if (...) { return ...; }`: drop it. A survivor means
    // no test reaches the guard, or the guard changes nothing.
    let t = line.trim();
    let opens = count_code(line, mask, b'{');
    let closes = count_code(line, mask, b'}');
    if t.starts_with("if") && opens == 1 && closes == 1 && t.ends_with('}') {
        let start = line.len() - line.trim_start().len();
        if code(start, 2) && t.contains("return") {
            push("drop-guard", 0, line.len(), "");
        }
    }

    let prev = |i: usize| if i == 0 { b' ' } else { b[i - 1] };
    let next = |i: usize| if i < b.len() { b[i] } else { b' ' };
    let mut i = 0;
    while i < b.len() {
        if !mask[i] {
            i += 1;
            continue;
        }
        // Bytes, not `&line[..]`: a non-ASCII byte in a comment-free code
        // span would make a two-byte str slice panic off a char boundary.
        let two: &[u8] = if i + 1 < b.len() { &b[i..i + 2] } else { &[] };
        match two {
            // A spaced shift swaps direction (#7022); `>>` is never a comparison.
            b">>" | b"<<" if code(i, 2) && prev(i) == b' ' && next(i + 2) == b' ' => {
                push("swap-arith", i, 2, if two == b">>" { "<<" } else { ">>" });
                i += 2;
                continue;
            }
            b">=" if code(i, 2) && prev(i) != b'>' => {
                push("flip-cmp", i, 2, ">");
                i += 2;
                continue;
            }
            b"<=" if code(i, 2) && prev(i) != b'<' => {
                push("flip-cmp", i, 2, "<");
                i += 2;
                continue;
            }
            b"==" if code(i, 2) && !b"=!<>".contains(&prev(i)) && next(i + 2) != b'=' => {
                push("flip-cmp", i, 2, "!=");
                i += 2;
                continue;
            }
            b"!=" if code(i, 2) && next(i + 2) != b'=' => {
                push("flip-cmp", i, 2, "==");
                i += 2;
                continue;
            }
            b"&&" if code(i, 2) => {
                push("swap-logic", i, 2, "||");
                i += 2;
                continue;
            }
            b"||" if code(i, 2) => {
                push("swap-logic", i, 2, "&&");
                i += 2;
                continue;
            }
            _ => {}
        }
        match b[i] {
            // `->`, `=>` and `>>` are not comparisons.
            b'>' if !b"-=>".contains(&prev(i)) && !b"=>".contains(&next(i + 1)) => {
                push("flip-cmp", i, 1, ">=");
            }
            b'<' if prev(i) != b'<' && !b"=<".contains(&next(i + 1)) => {
                push("flip-cmp", i, 1, "<=");
            }
            b'a' if line[i..].starts_with("and")
                && code(i, 3)
                && !is_word_byte(prev(i))
                && !is_word_byte(next(i + 3)) =>
            {
                push("swap-logic", i, 3, "or");
                i += 3;
                continue;
            }
            b'o' if line[i..].starts_with("or")
                && code(i, 2)
                && !is_word_byte(prev(i))
                && !is_word_byte(next(i + 2)) =>
            {
                push("swap-logic", i, 2, "and");
                i += 2;
                continue;
            }
            // `x + 1` -> `x`, `x - 1` -> `x`: an off-by-one no test pins.
            b'+' | b'-'
                if i > 0
                    && prev(i) == b' '
                    && line[i + 1..].starts_with(" 1")
                    && code(i - 1, 4)
                    && !is_word_byte(next(i + 3))
                    && next(i + 3) != b'.' =>
            {
                push("drop-step", i - 1, 4, "");
            }
            _ => {}
        }
        // `a * b` -> `a / b` and its kin: arithmetic no test pins (#7022). A
        // space on both sides keeps `->`, `&&`, a unary minus and `*T` out.
        if prev(i) == b' ' && next(i + 1) == b' ' && code(i, 1) {
            let to = match b[i] {
                b'*' => Some("/"),
                b'/' => Some("*"),
                b'%' => Some("/"),
                b'+' => Some("-"),
                b'-' => Some("+"),
                b'&' => Some("|"),
                b'|' => Some("&"),
                b'^' => Some("|"),
                _ => None,
            };
            if let Some(to) = to {
                push("swap-arith", i, 1, to);
            }
        }
        i += 1;
    }
}

/// The statement a `ret-default` body holds (#7022): `return 0;` for an
/// integer, `return false;` for a bool, `""` for a function that returns
/// nothing, and `None` for a type with no default the tool can name (a
/// struct, a slice, an error union). `brace` is the header's code `{`.
fn t27_default_return(header: &str, mask: &[bool], brace: usize) -> Option<&'static str> {
    let b = header.as_bytes();
    let arrow = (0..brace.saturating_sub(1))
        .rev()
        .find(|&i| mask[i] && mask[i + 1] && b[i] == b'-' && b[i + 1] == b'>');
    let ty = match arrow {
        None => "void",
        Some(a) => header[a + 2..brace].trim(),
    };
    match ty {
        "void" => Some(""),
        "bool" => Some("return false;"),
        "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize" => {
            Some("return 0;")
        }
        "f32" | "f64" => Some("return 0.0;"),
        _ => None,
    }
}

/// The last code `}` of a line, when nothing but a comment follows it.
fn t27_closing_brace(line: &str, mask: &[bool]) -> Option<usize> {
    let b = line.as_bytes();
    let at = (0..b.len()).rev().find(|&i| mask[i] && !b[i].is_ascii_whitespace())?;
    (b[at] == b'}').then_some(at)
}

/// Replace the body of the function whose header is line `h` (0-based) and
/// whose closing `}` is on line `end` with its default return. Nothing when
/// the type has no default or the body already is that default (an
/// equivalent mutant is noise, not a question).
fn t27_ret_default(lines: &[&str], h: usize, brace: usize, end: usize, out: &mut Vec<SpecMutant>) {
    let header = lines[h];
    let hmask = t27_code_mask(header);
    let Some(stmt) = t27_default_return(header, &hmask, brace) else {
        return;
    };
    let emask = t27_code_mask(lines[end]);
    let Some(close) = t27_closing_brace(lines[end], &emask) else {
        return;
    };
    let mut body = String::new();
    for (i, line) in lines.iter().enumerate().take(end + 1).skip(h) {
        let from = if i == h { brace + 1 } else { 0 };
        let to = if i == end { close } else { line.len() };
        if from < to {
            body.push_str(line[from..to].trim());
            body.push(' ');
        }
    }
    if body.trim() == stmt {
        return;
    }
    let after = if stmt.is_empty() {
        format!("{} }}", &header[..=brace])
    } else {
        format!("{} {stmt} }}", &header[..=brace])
    };
    out.push(SpecMutant {
        line: h + 1,
        through: end + 1,
        kind: "ret-default",
        before: header.to_string(),
        after,
    });
}

/// Every mutant inside the function bodies of a spec (or of the one function
/// `func`). Header lines, `test` and `invariant` blocks, constants, comments
/// and string literals are not sites: a header holds `->`, and a test's own
/// asserts going red would say nothing about the code under test.
///
/// A one-line function (`pub fn f(a: u8) -> bool { return a > 1; }`, 326 of
/// them in 72 specs on 2026-10-06) is a site too: its header is masked and the
/// rest of the line is mutated like a body line (#7022).
pub(crate) fn find_spec_mutants(text: &str, func: Option<&str>) -> Vec<SpecMutant> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out = Vec::new();
    // The open function: its name, header line and the header's code `{`.
    let mut current: Option<(String, usize, usize)> = None;
    let mut depth: i64 = 0;
    for (idx, line) in lines.iter().enumerate() {
        let mut mask = t27_code_mask(line);
        let delta = count_code(line, &mask, b'{') - count_code(line, &mask, b'}');
        let (name, h, brace) = match &current {
            None => {
                let Some(name) = t27_fn_header(line) else {
                    continue;
                };
                let Some(brace) = (0..line.len()).find(|&i| mask[i] && line.as_bytes()[i] == b'{')
                else {
                    continue;
                };
                if delta > 0 {
                    depth = delta;
                    current = Some((name, idx, brace));
                } else if delta == 0 && func.map_or(true, |f| f == name) {
                    for m in mask.iter_mut().take(brace + 1) {
                        *m = false;
                    }
                    t27_line_mutants(idx + 1, line, &mask, &mut out);
                    t27_ret_default(&lines, idx, brace, idx, &mut out);
                }
                continue;
            }
            Some(open) => open.clone(),
        };
        depth += delta;
        let wanted = func.map_or(true, |f| f == name);
        if wanted {
            t27_line_mutants(idx + 1, line, &mask, &mut out);
        }
        if depth <= 0 {
            current = None;
            if wanted {
                t27_ret_default(&lines, h, brace, idx, &mut out);
            }
        }
    }
    out
}

/// The spec with one mutant applied; every other byte is unchanged.
pub(crate) fn apply_spec_mutant(text: &str, m: &SpecMutant) -> String {
    let mut out = String::with_capacity(text.len());
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let n = i + 1;
        if n == m.line {
            out.push_str(&m.after);
        } else if n < m.line || n > m.through {
            out.push_str(line);
        }
    }
    out
}

/// Which check noticed a killed mutant.
#[derive(Debug, Clone, Copy, PartialEq)]
enum KilledBy {
    /// A test failed when the test binary ran.
    Test,
    /// t27c lowers an `invariant` to a `comptime` block, so an invariant the
    /// mutant breaks fails zig's compile. That is a check noticing too.
    Invariant,
}

/// What one mutant did. Only `Killed` is a check noticing it: a hang and a
/// compile error used to count as killed and were listed nowhere (#7148).
#[derive(Debug, Clone, PartialEq)]
enum Fate {
    Killed(KilledBy),
    Survived,
    /// It never finished: its tests outlived `--timeout`, or zig's comptime
    /// evaluation of an invariant stopped at its branch quota.
    Hang(&'static str),
    /// `t27c gen` or zig rejected it before any check ran (the first error
    /// line). cargo-mutants calls this unviable and counts it nowhere.
    Unviable(String),
}

/// Run a command; `None` when it outlived `secs` and was killed.
fn run_with_timeout(
    cmd: &mut Command,
    stdout: std::process::Stdio,
    stderr: std::process::Stdio,
    secs: u64,
) -> Result<Option<bool>> {
    let mut child = spawn_retrying(cmd.stdout(stdout).stderr(stderr))?;
    let start = std::time::Instant::now();
    loop {
        if let Some(st) = child.try_wait()? {
            return Ok(Some(st.success()));
        }
        if start.elapsed().as_secs() >= secs {
            kill_with_children(&mut child);
            return Ok(None);
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

/// A full pids cgroup answers EAGAIN to a fork. Each zig compile runs a thread
/// per core, so 16 at once on the 48-core lab (pids.max 1000) hit it, and the
/// one mutant that could not start used to end the whole run (#6993).
fn spawn_retrying(cmd: &mut Command) -> Result<std::process::Child> {
    let mut wait_ms = 100;
    loop {
        match cmd.spawn() {
            Ok(child) => return Ok(child),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && wait_ms <= 6400 => {
                std::thread::sleep(std::time::Duration::from_millis(wait_ms));
                wait_ms *= 2;
            }
            Err(e) => {
                return Err(anyhow::Error::from(e)
                    .context(format!("cannot start {}", cmd.get_program().to_string_lossy())))
            }
        }
    }
}

/// `zig test` runs the test binary as its child. Killing only `zig` left a
/// looping test binary at 100% CPU under PID 1 (two of them, measured on the
/// lab, #6993). Freeze the parent so it starts nothing new, kill what it
/// started, then the parent. Its own process group would do it too, but then
/// Ctrl-C would no longer reach the test binaries.
fn kill_with_children(child: &mut std::process::Child) {
    let pid = child.id().to_string();
    let quiet = |argv: &[&str]| {
        let _ = Command::new(argv[0])
            .args(&argv[1..])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    };
    quiet(&["kill", "-STOP", &pid]);
    quiet(&["pkill", "-KILL", "-P", &pid]);
    let _ = child.kill();
    let _ = child.wait();
}

/// The first `error: ...` line a tool printed, without the path before it.
fn first_error(stderr: &str) -> String {
    stderr
        .lines()
        .find_map(|l| l.find("error: ").map(|i| l[i..].trim().to_string()))
        .unwrap_or_else(|| "exited non-zero with no `error:` line".to_string())
}

/// Read a failed zig compile. A mutant that breaks an invariant fails it, and
/// zig marks that evaluation "called at comptime here": a check noticed.
/// Running out of branch quota there is comptime's own timeout, a loop that
/// never ends. Any other compile error means no check ran; measured on zig
/// 0.16.0, a type error carries no comptime note.
fn compile_fate(stderr: &str) -> Fate {
    if stderr.contains("error: evaluation exceeded") {
        Fate::Hang("zig's comptime branch quota, evaluating an invariant")
    } else if stderr.contains("note: called at comptime here") {
        Fate::Killed(KilledBy::Invariant)
    } else {
        Fate::Unviable(format!("zig: {}", first_error(stderr)))
    }
}

/// Compile a lowered spec's tests, then run them, each on its own clock. One
/// clock over both made a cold compile on a loaded machine read as a hang, and
/// `zig test` exits 1 on a compile error as on a failing test (#7148; the same
/// defect is unclebob/mutator issue 1).
fn zig_fate(zig: &Path, zig_j: u32, compile_secs: u64, run_secs: u64) -> Result<Fate> {
    let dir = zig.parent().unwrap_or(Path::new("."));
    let bin = zig.with_extension("bin");
    let err = zig.with_extension("err");
    let compiled = run_with_timeout(
        &mut zig_compile(zig, &bin, zig_j),
        std::process::Stdio::null(),
        std::fs::File::create(&err)?.into(),
        compile_secs,
    )?;
    match compiled {
        None => bail!(
            "the zig compile outlived {compile_secs} s: the machine's load, not the mutant \
             (comptime stops itself at its branch quota); lower --jobs or raise --timeout"
        ),
        Some(false) => return Ok(compile_fate(&std::fs::read_to_string(&err).unwrap_or_default())),
        Some(true) => {}
    }
    match run_with_timeout(
        Command::new(&bin).current_dir(dir),
        std::process::Stdio::null(),
        std::process::Stdio::null(),
        run_secs,
    )? {
        None => Ok(Fate::Hang("its tests outlived --timeout")),
        Some(false) => Ok(Fate::Killed(KilledBy::Test)),
        Some(true) => Ok(Fate::Survived),
    }
}

/// The compile half of `zig_fate`: zig's own thread count is a pid each, so -j
/// bounds what one job holds (specs/tri/mutate/lab.t27 job_pids).
fn zig_compile(zig: &Path, bin: &Path, zig_j: u32) -> Command {
    let mut c = Command::new("zig");
    c.arg("test")
        .arg(zig)
        .arg("--test-no-exec")
        .arg(format!("-femit-bin={}", bin.display()))
        .arg(format!("-j{zig_j}"))
        .current_dir(zig.parent().unwrap_or(Path::new(".")));
    c
}

/// Lower one spec copy with `t27c gen`, then compile and run it (`zig_fate`).
fn spec_fate(t27c: &str, spec: &Path, zig: &Path, zig_j: u32, secs: u64) -> Result<Fate> {
    let err = zig.with_extension("err");
    let out = std::fs::File::create(zig)?;
    match run_with_timeout(
        Command::new(t27c).arg("gen").arg(spec),
        out.into(),
        std::fs::File::create(&err)?.into(),
        secs,
    )? {
        None => bail!("`t27c gen` outlived {secs} s: the machine's load, not the mutant"),
        Some(false) => Ok(Fate::Unviable(format!(
            "t27c gen: {}",
            first_error(&std::fs::read_to_string(&err).unwrap_or_default())
        ))),
        Some(true) => zig_fate(zig, zig_j, secs, secs),
    }
}

/// One listed mutant: where, which kind, why, and the text it changed.
fn push_site(out: &mut String, file: &Path, m: &SpecMutant, why: &str) {
    out.push_str(&format!("  {}:{} [{}]{why}\n", file.display(), m.line, m.kind));
    out.push_str(&format!("    - {}\n", m.before.trim()));
    if m.after.trim().is_empty() {
        out.push_str("    + (line dropped)\n");
    } else {
        out.push_str(&format!("    + {}\n", m.after.trim()));
    }
    if m.through > m.line {
        out.push_str(&format!("    + (lines {}-{} emptied)\n", m.line + 1, m.through));
    }
}

/// The end of a run. Every mutant that ran is in exactly one of the four
/// counts, and every one that was not killed is listed by line and kind.
fn spec_report(file: &Path, ran: &[(&SpecMutant, Fate)]) -> String {
    let by = |k: KilledBy| ran.iter().filter(|(_, f)| *f == Fate::Killed(k)).count();
    let (by_test, by_invariant) = (by(KilledBy::Test), by(KilledBy::Invariant));
    let survived: Vec<_> = ran.iter().filter(|(_, f)| *f == Fate::Survived).collect();
    let hung: Vec<_> = ran.iter().filter(|(_, f)| matches!(f, Fate::Hang(_))).collect();
    let unviable: Vec<_> = ran.iter().filter(|(_, f)| matches!(f, Fate::Unviable(_))).collect();
    let mut out = format!(
        "{} of {} killed ({by_test} by a failing test, {by_invariant} by an invariant at compile \
         time); {} survived, {} hung, {} unviable.\n",
        by_test + by_invariant,
        ran.len(),
        survived.len(),
        hung.len(),
        unviable.len()
    );
    if !survived.is_empty() {
        out.push_str(&format!("{} SURVIVED -- the spec's tests did not notice:\n", survived.len()));
        for (m, _) in &survived {
            push_site(&mut out, file, m, "");
        }
        out.push_str(
            "Each survivor is a dead line (remove it), a test gap (add an assert), or an \
             equivalent mutant (say why where the work is recorded).\n",
        );
    }
    if !hung.is_empty() {
        out.push_str(&format!(
            "{} HUNG -- not killed: no check failed, the mutant never finished:\n",
            hung.len()
        ));
        for (m, f) in &hung {
            if let Fate::Hang(why) = f {
                push_site(&mut out, file, m, &format!(" ({why})"));
            }
        }
        out.push_str(
            "A hang in a loop the mutant made endless (a dropped step, a flipped bound) is \
             the mutant's; any other is worth a re-run at a lower --jobs.\n",
        );
    }
    if !unviable.is_empty() {
        out.push_str(&format!(
            "{} UNVIABLE -- not killed: t27c gen or zig rejected the mutant before any check ran:\n",
            unviable.len()
        ));
        for (m, f) in &unviable {
            if let Fate::Unviable(why) = f {
                push_site(&mut out, file, m, &format!(": {why}"));
            }
        }
    }
    out
}

// The survivor gate (#7303). The rule is specs/tri/mutate/survivors.t27; the five
// functions below are its copy, and `the_gate_agrees_with_every_assert_row_of_its_spec`
// evaluates every `assert` row of that spec against them, so a row the spec changes
// fails the test until this copy follows. The codes follow cargo-mutants: 2 is the
// gate's verdict, 1 stays the error exit for a run that is not whole.
const EXIT_OK: u8 = 0;
const EXIT_FAILED: u8 = 1;
const EXIT_SURVIVED: u8 = 2;

fn gate_on(fail_on_survived: bool, accepted_given: bool) -> bool {
    fail_on_survived || accepted_given
}

fn not_killed(survived: u32, hung: u32) -> u32 {
    survived + hung
}

fn unaccepted(missed: u32, accepted_hits: u32) -> u32 {
    if accepted_hits > missed {
        return missed;
    }
    missed - accepted_hits
}

fn survivor_exit(on: bool, survived: u32, hung: u32, accepted_hits: u32, accepted_killed: u32) -> u8 {
    if !on {
        return EXIT_OK;
    }
    if unaccepted(not_killed(survived, hung), accepted_hits) > 0 {
        return EXIT_SURVIVED;
    }
    if accepted_killed > 0 {
        return EXIT_SURVIVED;
    }
    EXIT_OK
}

fn mutate_exit(baseline_green: bool, not_run: u32, survivors: u8) -> u8 {
    if !baseline_green {
        return EXIT_FAILED;
    }
    if not_run > 0 {
        return EXIT_FAILED;
    }
    survivors
}

// What `tri mutate spec --lab` decides about a run on the Railway lab (#7050). The
// rule is specs/tri/mutate/lab.t27; this module is its copy, and
// `the_lab_rules_agree_with_every_assert_row_of_their_spec` evaluates every `assert`
// row of that spec against it. `lab_mutate_spec` below is the `railway ssh`
// plumbing that calls it (slice 3 of #7050).
mod lab {
    /// A run's state, read from its directory on the lab.
    pub const RUN_NONE: u8 = 0;
    pub const RUN_RUNNING: u8 = 1;
    pub const RUN_DONE: u8 = 2;
    pub const RUN_LOST: u8 = 3;

    /// `--lab`'s exit codes: 0..2 are the remote tool's, passed through; 3..5 the lab's own.
    pub const EXIT_OK: u8 = 0;
    pub const EXIT_TOOL_FAILED: u8 = 1;
    pub const EXIT_SURVIVED: u8 = 2;
    pub const EXIT_NO_RESULT: u8 = 3;
    pub const EXIT_ORPHANS: u8 = 4;
    pub const EXIT_STILL_RUNNING: u8 = 5;

    /// The remote tool's code for a failed survivor gate: the gate's own constant.
    pub const TOOL_RC_SURVIVED: u32 = super::EXIT_SURVIVED as u32;

    pub const SSH_MAX_ATTEMPTS: u32 = 3;
    pub const POLL_BASE_SECONDS: u32 = 5;
    pub const POLL_CAP_SECONDS: u32 = 60;

    /// Pids left for the lab's other lanes: one batch of 8 jobs at zig's own thread
    /// count on the lab's 48 cores (the measurements are in the spec).
    pub const PID_RESERVE: u32 = 408;
    /// Pids one job holds besides its zig compile's -j threads.
    pub const JOB_OVERHEAD_PIDS: u32 = 3;

    /// The run's state from what its directory shows; an exit file wins over a runner still closing.
    pub fn run_state(dir_exists: bool, exit_written: bool, runner_alive: bool) -> u8 {
        if !dir_exists {
            return RUN_NONE;
        }
        if exit_written {
            return RUN_DONE;
        }
        if runner_alive {
            return RUN_RUNNING;
        }
        RUN_LOST
    }

    /// A run starts only where none is.
    pub fn launch_allowed(state: u8) -> bool {
        state == RUN_NONE
    }

    /// The directory goes only when nothing will write to it again and nothing runs in it.
    pub fn may_remove(state: u8, orphans: u32) -> bool {
        if orphans > 0 {
            return false;
        }
        state == RUN_DONE || state == RUN_LOST
    }

    /// The exit code of `--lab`: orphans outrank the tool's verdict, and a code the
    /// tool never defined is a failed tool, never a verdict.
    pub fn lab_exit(state: u8, tool_rc: u32, orphans: u32) -> u8 {
        if state == RUN_RUNNING {
            return EXIT_STILL_RUNNING;
        }
        if state != RUN_DONE {
            return EXIT_NO_RESULT;
        }
        if orphans > 0 {
            return EXIT_ORPHANS;
        }
        if tool_rc == 0 {
            return EXIT_OK;
        }
        if tool_rc == TOOL_RC_SURVIVED {
            return EXIT_SURVIVED;
        }
        EXIT_TOOL_FAILED
    }

    /// After `attempts` failed tries of one ssh call: only a read, only a transient
    /// failure, at most SSH_MAX_ATTEMPTS tries in all. A write is never resent.
    pub fn ssh_should_retry(attempts: u32, transient: bool, is_write: bool) -> bool {
        if is_write {
            return false;
        }
        transient && attempts < SSH_MAX_ATTEMPTS
    }

    /// Seconds to wait before poll number `polls` (from 0): 5, 10, 20, 40, then 60.
    pub fn poll_wait_seconds(polls: u32) -> u32 {
        let (mut wait, mut n) = (POLL_BASE_SECONDS, 0);
        while n < polls && wait < POLL_CAP_SECONDS {
            wait *= 2;
            n += 1;
        }
        if wait > POLL_CAP_SECONDS {
            return POLL_CAP_SECONDS;
        }
        wait
    }

    /// zig's -j for each mutant's `zig test`: the cores shared by the jobs, at least 1.
    pub fn zig_j(nproc: u32, jobs: u32) -> u32 {
        if jobs == 0 {
            return 1;
        }
        let j = nproc / jobs;
        if j == 0 {
            return 1;
        }
        j
    }

    /// The pids one job holds: its compile's -j threads and the overhead. The flag
    /// caps --zig-threads at 4096 and zig_j is at most the lab's core count, so the
    /// sum cannot overflow.
    pub fn job_pids(zig_threads: u32) -> u32 {
        zig_threads + JOB_OVERHEAD_PIDS
    }

    /// How many mutants may run at once: the request, cut to the pids left after the
    /// reserve. `per_job` 0 is no measurement and refuses; 0 = do not start.
    pub fn lab_jobs(requested: u32, pids_max: u32, pids_used: u32, per_job: u32) -> u32 {
        if per_job == 0 {
            return 0;
        }
        if pids_used + PID_RESERVE >= pids_max {
            return 0;
        }
        let fit = (pids_max - pids_used - PID_RESERVE) / per_job;
        if fit < requested {
            return fit;
        }
        requested
    }
}

// `tri mutate spec --lab` (#7050, slice 3): the same run, on the Railway lab.
// The decisions are `mod lab`'s, so specs/tri/mutate/lab.t27's; what follows
// only carries them over `railway ssh`. A run is a nohup'd job in a directory
// named from a hash of the request, so the same command called again reads the
// run it started, whatever happened to the ssh in between.

/// Where `--lab` runs and how it gets there. The first six names are the ones
/// scripts/tri_loop/t27b.py reads. `T27C_LAB_LOCAL=1` runs the same scripts
/// with `sh -c` on this machine instead: on the lab itself, and in the tests.
struct LabEnv {
    local: bool,
    railway: String,
    project: Option<String>,
    environment: String,
    service: String,
    dir: Option<String>,
    tri: String,
    t27c: String,
    src: String,
    zig: String,
    runs: String,
    /// Where `pids.max` and `pids.current` are read: cgroup v2's root by default.
    cgroup: String,
}

impl LabEnv {
    fn from_env() -> LabEnv {
        let get = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let or = |k: &str, d: &str| get(k).unwrap_or_else(|| d.to_string());
        LabEnv {
            local: get("T27C_LAB_LOCAL").as_deref() == Some("1"),
            railway: or("T27C_LAB_RAILWAY", "railway"),
            project: get("T27C_LAB_PROJECT"),
            environment: or("T27C_LAB_ENV", "production"),
            service: or("T27C_LAB_SERVICE", "t27c-lab"),
            dir: get("T27C_LAB_DIR"),
            tri: or("T27C_LAB_TRI", "/data/target/release/tri"),
            t27c: or("T27C_LAB_BIN", "/data/target/release/t27c"),
            src: or("T27C_LAB_SRC", "/data/src"),
            zig: or("T27C_LAB_ZIG", "/opt/zig"),
            // Not /data: it was down to 160M free on 2026-10-07 (#7062).
            runs: or("T27C_LAB_RUNS", "/tmp"),
            cgroup: or("T27C_LAB_CGROUP", "/sys/fs/cgroup"),
        }
    }

    fn describe(&self) -> String {
        if self.local {
            "`sh -c` here (T27C_LAB_LOCAL=1)".to_string()
        } else {
            format!("`{} ssh -s {} -e {}`", self.railway, self.service, self.environment)
        }
    }
}

/// One `--lab` request, its files already read.
struct LabRun {
    rel: String,
    spec: Vec<u8>,
    func: Option<String>,
    accepted: Option<Vec<u8>>,
    max: usize,
    jobs: usize,
    /// --zig-threads as given; `None` = zig_j over the lab's cores.
    zig_threads: Option<u32>,
    secs: u64,
    fail_on_survived: bool,
    wait: u64,
}

const LAB_BEGIN: &str = "TRI-MUTATE-LAB-BEGIN";
const LAB_END: &str = "TRI-MUTATE-LAB-END";
/// Seconds one ssh call may take, as scripts/tri_loop/t27b.py allows it.
const LAB_CALL_SECS: u64 = 180;
/// Base64 characters per upload call, as t27b.py sends them.
const LAB_CHUNK: usize = 64000;
/// A cgroup's `max` sets no cap below the kernel's: Linux caps pid_max at 2^22.
const LAB_PIDS_UNCAPPED: u32 = 1 << 22;

/// One shell word: single-quoted, each `'` closed, escaped and reopened.
fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// RFC 4648 base64 with padding: the spec reaches the lab as text in a command.
fn base64_encode(b: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(b.len().div_ceil(3) * 4);
    for c in b.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            s.push(if i <= c.len() { A[(n >> (18 - 6 * i) & 63) as usize] as char } else { '=' });
        }
    }
    s
}

/// The lines a script printed between its markers, `\r` trimmed; `None` when
/// the answer did not come back whole (a dropped ssh, a cut stream, a banner
/// only).
fn lab_marked(stdout: &str) -> Option<Vec<String>> {
    let lines: Vec<&str> = stdout.lines().map(|l| l.trim_end_matches('\r')).collect();
    let begin = lines.iter().position(|l| *l == LAB_BEGIN)?;
    let end = lines.iter().rposition(|l| *l == LAB_END)?;
    (begin < end).then(|| lines[begin + 1..end].iter().map(|l| l.to_string()).collect())
}

/// What one call brought back: the marked lines, or why there were none.
type LabAnswer = std::result::Result<Vec<String>, String>;

/// `script` once, in a subshell, so an `exit` in it cannot eat the end
/// marker. An `Err` is a caller that could not start `railway` (or `sh`) at
/// all, which no retry mends.
fn lab_call_once(env: &LabEnv, script: &str) -> Result<LabAnswer> {
    static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let wrapped = format!("echo {LAB_BEGIN}; ( {script} ); echo {LAB_END}");
    let mut cmd = if env.local {
        let mut c = Command::new("sh");
        c.arg("-c").arg(&wrapped);
        c
    } else {
        let mut c = Command::new(&env.railway);
        c.arg("ssh");
        if let Some(p) = &env.project {
            c.args(["-p", p]);
        }
        c.args(["-e", &env.environment, "-s", &env.service]).arg(&wrapped);
        if let Some(d) = &env.dir {
            c.current_dir(d);
        }
        c
    };
    cmd.stdin(std::process::Stdio::null());
    let n = CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let base = std::env::temp_dir().join(format!("tri-mutate-lab-{}-{n}", std::process::id()));
    let (out_path, err_path) = (base.with_extension("out"), base.with_extension("err"));
    let done = run_with_timeout(
        &mut cmd,
        std::fs::File::create(&out_path)?.into(),
        std::fs::File::create(&err_path)?.into(),
        LAB_CALL_SECS,
    );
    let read = |p: &Path| std::fs::read(p).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default();
    let (stdout, stderr) = (read(&out_path), read(&err_path));
    let _ = std::fs::remove_file(&out_path);
    let _ = std::fs::remove_file(&err_path);
    if done.with_context(|| format!("cannot reach the lab with {}", env.describe()))?.is_none() {
        return Ok(Err(format!("no answer within {LAB_CALL_SECS} s")));
    }
    Ok(lab_marked(&stdout).ok_or_else(|| {
        let last = stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
        format!("the answer came back cut{}", if last.is_empty() { String::new() } else { format!(": {last}") })
    }))
}

/// `script` on the lab, tried again only as `lab::ssh_should_retry` allows: a
/// read after a cut answer, never a write.
fn lab_call(env: &LabEnv, script: &str, is_write: bool) -> Result<LabAnswer> {
    let mut attempts = 0;
    loop {
        let got = lab_call_once(env, script)?;
        attempts += 1;
        if !lab::ssh_should_retry(attempts, got.is_err(), is_write) {
            return Ok(got);
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

/// What a probe reads from a run's directory and from the lab around it.
#[derive(Debug, Default, PartialEq)]
struct LabProbe {
    dir: bool,
    /// The tool's exit code, once the exit file is there.
    exit: Option<u32>,
    alive: bool,
    /// `PID CMDLINE` of each process whose working directory is in the run's
    /// directory, the runner itself left out.
    orphans: Vec<String>,
    pids_max: u32,
    pids_used: u32,
    /// The lab's cores (`nproc`); 0 when it did not say, which zig_j reads as -j1.
    nproc: u32,
}

impl LabProbe {
    fn state(&self) -> u8 {
        lab::run_state(self.dir, self.exit.is_some(), self.alive)
    }
}

/// The probe, as one read: the run's three files, every process working in
/// its directory, the pids cgroup's cap and use, and the lab's cores.
fn lab_probe_script(env: &LabEnv, d: &str) -> String {
    format!(
        "D={}; G={}; if [ -d \"$D\" ]; then echo dir 1; else echo dir 0; fi; \
         if [ -f \"$D/exit\" ]; then echo \"exit $(cat \"$D/exit\")\"; fi; \
         P=$(cat \"$D/pid\" 2>/dev/null); \
         if [ -n \"$P\" ] && kill -0 \"$P\" 2>/dev/null; then echo alive 1; else echo alive 0; fi; \
         if [ -d \"$D\" ]; then for p in /proc/[0-9]*; do c=$(readlink \"$p/cwd\" 2>/dev/null) || continue; \
         case \"$c\" in \"$D\"|\"$D\"/*) n=${{p#/proc/}}; if [ \"$n\" != \"$P\" ]; then \
         echo \"orphan $n $(tr '\\000' ' ' < \"$p/cmdline\" 2>/dev/null | cut -c1-200)\"; fi;; esac; done; fi; \
         echo \"pids $(cat \"$G/pids.max\" 2>/dev/null || echo max) $(cat \"$G/pids.current\" 2>/dev/null || echo 0)\"; \
         echo \"nproc $(nproc 2>/dev/null || echo 0)\"",
        sh_quote(d),
        sh_quote(&env.cgroup)
    )
}

/// A probe's lines; `None` when one of its three always-printed lines is
/// missing. An exit file that holds no number is a code the tool does not
/// define, so `lab_exit` reads it as a failed tool.
fn parse_lab_probe(lines: &[String]) -> Option<LabProbe> {
    let mut p = LabProbe::default();
    let (mut dir, mut alive, mut pids) = (false, false, false);
    for l in lines {
        let (k, v) = l.split_once(' ').unwrap_or((l.as_str(), ""));
        match k {
            "dir" => (p.dir, dir) = (v == "1", true),
            "exit" => p.exit = Some(v.trim().parse().unwrap_or(u32::MAX)),
            "alive" => (p.alive, alive) = (v == "1", true),
            "orphan" => p.orphans.push(v.to_string()),
            "pids" => {
                let (m, u) = v.split_once(' ').unwrap_or((v, ""));
                p.pids_max = if m == "max" { LAB_PIDS_UNCAPPED } else { m.parse().unwrap_or(0) };
                p.pids_used = u.trim().parse().unwrap_or(0);
                pids = true;
            }
            "nproc" => p.nproc = v.trim().parse().unwrap_or(0),
            _ => {}
        }
    }
    (dir && alive && pids).then_some(p)
}

fn lab_probe(env: &LabEnv, d: &str) -> Result<std::result::Result<LabProbe, String>> {
    Ok(match lab_call(env, &lab_probe_script(env, d), false)? {
        Ok(lines) => parse_lab_probe(&lines).ok_or_else(|| format!("the probe's answer is incomplete: {lines:?}")),
        Err(why) => Err(why),
    })
}

/// The run's directory name: a hash of everything that makes two requests
/// the same run, so the same command called again finds the run it started.
fn lab_run_name(env: &LabEnv, run: &LabRun) -> String {
    use sha2::{Digest, Sha256};
    let args = format!(
        "{:?} {} {} {:?} {} {} {} {} {} {}",
        run.func,
        run.max,
        run.jobs,
        run.zig_threads,
        run.secs,
        run.fail_on_survived,
        run.accepted.is_some(),
        env.tri,
        env.t27c,
        env.src
    );
    let mut h = Sha256::new();
    for part in [run.rel.as_bytes(), &run.spec, run.accepted.as_deref().unwrap_or(b""), args.as_bytes()] {
        h.update((part.len() as u64).to_le_bytes());
        h.update(part);
    }
    format!("tri-mutate-{}", &hex::encode(h.finalize())[..16])
}

/// The job the lab runs. Every way it can end writes an exit file, the last
/// step through a rename, so a reader never sees half a code. A tri built
/// before a flag answers clap's usage error, exit 2, which would read as the
/// survivor gate's 2: each flag is looked up in its `--help` first.
fn lab_run_sh(env: &LabEnv, run: &LabRun, d: &str, jobs: u32, zig_j: u32) -> String {
    let q = sh_quote;
    let (jobs, zig_j, max, secs) = (jobs.to_string(), zig_j.to_string(), run.max.to_string(), run.secs.to_string());
    let accepted = format!("{d}/accepted");
    let mut args = vec!["mutate", "spec", "--file", &run.rel, "--jobs", &jobs, "--zig-threads", &zig_j, "--max", &max];
    args.extend(["--timeout", &secs, "--t27c", &env.t27c]);
    if let Some(f) = &run.func {
        args.extend(["--fn", f]);
    }
    if run.fail_on_survived {
        args.push("--fail-on-survived");
    }
    if run.accepted.is_some() {
        args.extend(["--accepted", &accepted]);
    }
    let flags: Vec<&str> = args.iter().copied().filter(|a| a.starts_with("--")).collect();
    let (tri, out, exit, tmp) = (q(&env.tri), q(&format!("{d}/out")), q(&format!("{d}/exit")), q(&format!("{d}/exit.tmp")));
    let fail = |why: &str| format!("{{ echo {why} > {out}; echo 1 > {tmp} && mv {tmp} {exit}; exit 1; }}");
    format!(
        "echo $$ > {pid}\n\
         cd {w} || {cd_failed}\n\
         H=$({tri} mutate spec --help 2>&1)\n\
         for f in {flags}; do case \"$H\" in *\"$f\"*) ;; *) {no_flag};; esac; done\n\
         PATH={zig}:\"$PATH\"; export PATH\n\
         {tri} {args} > {out} 2>&1\n\
         echo $? > {tmp} && mv {tmp} {exit}\n",
        pid = q(&format!("{d}/pid")),
        w = q(&format!("{d}/w")),
        cd_failed = fail("\"tri mutate spec --lab: cannot cd to the run's copy of specs/\""),
        flags = flags.join(" "),
        no_flag = fail(&format!(
            "\"tri mutate spec --lab: {} has no $f (built $(date -u -r {tri} +%Y-%m-%dT%H:%M:%SZ 2>/dev/null)); set T27C_LAB_TRI to a newer build\"",
            env.tri.replace('"', "")
        )),
        zig = q(&env.zig),
        args = args.iter().map(|a| q(a)).collect::<Vec<_>>().join(" "),
    )
}

/// Upload the spec (and the accepted file), then start the job: each call a
/// write, sent once. The upload goes to a staging directory; the launch takes
/// the run's directory with one `mkdir`, so two callers cannot both start it,
/// and writes the launching shell's pid at once, so a probe in the gap before
/// the job's own pid sees a live run and not a lost one.
fn lab_launch(env: &LabEnv, run: &LabRun, d: &str, jobs: u32, zig_j: u32) -> Result<LabAnswer> {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|t| t.subsec_nanos()).unwrap_or(0);
    let stage = format!("{d}.up.{}.{nanos}", std::process::id());
    let mut files: Vec<(&str, &[u8])> = vec![("spec.b64", &run.spec)];
    if let Some(a) = &run.accepted {
        files.push(("acc.b64", a));
    }
    for (name, bytes) in &files {
        let b64 = base64_encode(bytes);
        let mut parts: Vec<&str> = b64.as_bytes().chunks(LAB_CHUNK).map(|c| std::str::from_utf8(c).unwrap()).collect();
        if parts.is_empty() {
            parts.push("");
        }
        for part in parts {
            let up = format!("mkdir -p {s} && printf %s {} >> {s}/{name} && echo ok", sh_quote(part), s = sh_quote(&stage));
            if let Err(why) = lab_call(env, &up, true)? {
                return Ok(Err(format!("upload of {name}: {why}; {stage} may be left on the lab")));
            }
        }
    }
    let q = sh_quote;
    let (s, dq, src, tri) = (q(&stage), q(d), q(&env.src), q(&env.tri));
    let parent = Path::new(&run.rel).parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
    let acc = if run.accepted.is_some() { " && base64 -d \"$S/acc.b64\" > \"$D/accepted\"" } else { "" };
    let script = format!(
        "S={s}; D={dq}; trap 'rm -rf \"$S\"' EXIT; \
         if mkdir \"$D\" 2>/dev/null; then echo $$ > \"$D/pid\"; \
         if {{ mkdir \"$D/w\" && cp -r {src}/specs \"$D/w/specs\" && mkdir -p \"$D/w/\"{parent} && \
         base64 -d \"$S/spec.b64\" > \"$D/w/\"{rel}{acc} && printf %s {runsh} | base64 -d > \"$D/run.sh\"; }} 2> \"$D/out\"; then \
         if command -v setsid >/dev/null 2>&1; then nohup setsid sh \"$D/run.sh\" >/dev/null 2>&1 & \
         else nohup sh \"$D/run.sh\" >/dev/null 2>&1 & fi; \
         i=0; while [ \"$(cat \"$D/pid\")\" = \"$$\" ] && [ $i -lt 100 ]; do sleep 0.1; i=$((i+1)); done; \
         echo \"launched pid $(cat \"$D/pid\")\"; \
         echo \"specs from {src_shown} at $(git -C {src} log -1 --format=%h 2>/dev/null)\"; \
         echo \"tool {tri_shown} built $(date -u -r {tri} +%Y-%m-%dT%H:%M:%SZ 2>/dev/null)\"; \
         else echo 1 > \"$D/exit\"; echo 'setup failed on the lab; its errors are the run output'; fi; \
         else echo 'the run directory was already taken'; fi",
        parent = q(&parent),
        rel = q(&run.rel),
        runsh = q(&base64_encode(lab_run_sh(env, run, d, jobs, zig_j).as_bytes())),
        src_shown = env.src.replace('"', ""),
        tri_shown = env.tri.replace('"', ""),
    );
    lab_call(env, &script, true)
}

fn lab_state_name(state: u8) -> &'static str {
    match state {
        lab::RUN_NONE => "none",
        lab::RUN_RUNNING => "running",
        lab::RUN_DONE => "done",
        _ => "lost",
    }
}

/// `tri mutate spec --lab`: start the run on the lab unless the same request
/// already has one, wait for it up to `run.wait` seconds, print its output,
/// and clean up as specs/tri/mutate/lab.t27 allows. Returns `lab_exit`.
fn lab_mutate_spec(env: &LabEnv, run: &LabRun, out: &mut dyn std::io::Write, sleep: &dyn Fn(u64)) -> Result<u8> {
    let d = format!("{}/{}", env.runs.trim_end_matches('/'), lab_run_name(env, run));
    writeln!(out, "Lab run {d}, through {}.", env.describe())?;
    let mut p = match lab_probe(env, &d)? {
        Ok(p) => p,
        Err(why) => {
            writeln!(out, "The lab did not answer ({why}). Nothing was started.")?;
            return Ok(lab::EXIT_NO_RESULT);
        }
    };
    if lab::launch_allowed(p.state()) {
        // zig's -j follows the request, not the cut: a cut batch keeps its per-job cost.
        let zig_j = run.zig_threads.unwrap_or_else(|| lab::zig_j(p.nproc, run.jobs as u32));
        let per_job = lab::job_pids(zig_j);
        let jobs = lab::lab_jobs(run.jobs as u32, p.pids_max, p.pids_used, per_job);
        if jobs == 0 {
            writeln!(
                out,
                "Not started: the lab has {} of {} pids in use, {} are kept for its other lanes, and one job at zig -j{zig_j} \
                 costs {per_job} (specs/tri/mutate/lab.t27 lab_jobs).",
                p.pids_used,
                p.pids_max,
                lab::PID_RESERVE,
            )?;
            return Ok(lab::EXIT_NO_RESULT);
        }
        writeln!(
            out,
            "Starting it: {jobs} job(s) of {} asked, zig -j{zig_j} each, on {} cores; the lab has {} of {} pids in use.",
            run.jobs, p.nproc, p.pids_used, p.pids_max
        )?;
        match lab_launch(env, run, &d, jobs, zig_j)? {
            Ok(lines) => lines.iter().try_for_each(|l| writeln!(out, "  {l}"))?,
            Err(why) => writeln!(out, "The launch did not answer whole ({why}); reading back what landed.")?,
        }
        p = match lab_probe(env, &d)? {
            Ok(p) => p,
            Err(why) => {
                writeln!(out, "The lab did not answer after the launch ({why}). Call the same command again to read the run.")?;
                return Ok(lab::EXIT_NO_RESULT);
            }
        };
        if p.state() == lab::RUN_NONE {
            writeln!(out, "The launch did not land: there is no run directory.")?;
            return Ok(lab::EXIT_NO_RESULT);
        }
    } else {
        writeln!(out, "Reading the run already there ({}).", lab_state_name(p.state()))?;
    }
    let (mut polls, mut waited) = (0u32, 0u64);
    while p.state() == lab::RUN_RUNNING {
        if waited >= run.wait {
            writeln!(out, "Still running after {waited} s of waiting. Call the same command again to read it.")?;
            return Ok(lab::lab_exit(lab::RUN_RUNNING, 0, 0));
        }
        let s = lab::poll_wait_seconds(polls) as u64;
        sleep(s);
        (waited, polls) = (waited + s, polls + 1);
        p = match lab_probe(env, &d)? {
            Ok(p) => p,
            Err(why) => {
                writeln!(out, "Lost the lab while waiting ({why}); the run may go on. Call the same command again to read it.")?;
                return Ok(lab::EXIT_NO_RESULT);
            }
        };
    }
    let state = p.state();
    if state == lab::RUN_DONE {
        let cat = format!("F={}/out; cat \"$F\"; if [ -n \"$(tail -c1 \"$F\")\" ]; then echo; fi", sh_quote(&d));
        match lab_call(env, &cat, false)? {
            Ok(lines) => lines.iter().try_for_each(|l| writeln!(out, "{l}"))?,
            Err(why) => {
                writeln!(out, "The run finished, but its output did not come back ({why}); it stays in {d}.")?;
                return Ok(lab::EXIT_NO_RESULT);
            }
        }
    } else {
        writeln!(out, "The run stopped without an exit code: the lab restarted, or the job was killed.")?;
    }
    let orphans = p.orphans.len() as u32;
    if orphans > 0 {
        writeln!(out, "{orphans} ORPHAN process(es) still run in {d} (trap T9: a killed `zig test` leaves its test binary spinning):")?;
        p.orphans.iter().try_for_each(|o| writeln!(out, "  {o}"))?;
        writeln!(out, "The directory stays until none is left: kill them on the lab, then call the same command again.")?;
    }
    if lab::may_remove(state, orphans) {
        match lab_call(env, &format!("rm -rf {} && echo ok", sh_quote(&d)), true)? {
            Ok(_) => writeln!(out, "Removed {d}.")?,
            Err(why) => writeln!(out, "Could not remove {d} ({why}).")?,
        }
        if state == lab::RUN_LOST {
            writeln!(out, "Call the same command again to start a new run.")?;
        }
    }
    Ok(lab::lab_exit(state, p.exit.unwrap_or(u32::MAX), orphans))
}

/// The `--lab` request from the command line: the spec's path as the lab's
/// copy of the repo will hold it, and every file read here, so a missing one
/// fails before anything reaches the lab.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
fn lab_request(
    file: &str,
    func: Option<&str>,
    accepted: Option<&str>,
    max: usize,
    jobs: usize,
    zig_threads: Option<u32>,
    secs: u64,
    gate: bool,
    wait: u64,
) -> Result<LabRun> {
    let path = Path::new(file);
    let mut rel = PathBuf::new();
    for c in path.components() {
        match c {
            std::path::Component::Normal(n) => rel.push(n),
            std::path::Component::CurDir => {}
            _ => bail!("--lab needs the spec's path relative to the repo root, without `..`: {file}"),
        }
    }
    let rel = rel.to_str().filter(|r| !r.is_empty() && r.is_ascii()).with_context(|| format!("--lab needs an ASCII path: {file}"))?.to_string();
    let spec = std::fs::read(path).with_context(|| format!("cannot read {file}"))?;
    if let Some(f) = func {
        if !String::from_utf8_lossy(&spec).split('\n').any(|l| t27_fn_header(l).as_deref() == Some(f)) {
            bail!("no function named `{f}` in {file}");
        }
    }
    let accepted = match accepted {
        Some(p) => {
            let b = std::fs::read(p).with_context(|| format!("cannot read --accepted {p}"))?;
            parse_accepted(&String::from_utf8_lossy(&b)).with_context(|| format!("in --accepted {p}"))?;
            Some(b)
        }
        None => None,
    };
    Ok(LabRun { rel, spec, func: func.map(str::to_string), accepted, max, jobs, zig_threads, secs, fail_on_survived: gate, wait })
}

/// `--fail-on-survived` and `--accepted`, as given.
pub(crate) struct GateArgs<'a> {
    pub fail_on_survived: bool,
    pub accepted: Option<&'a str>,
}

/// One line of an accepted file: a mutant as `push_site` prints it.
#[derive(Debug, Clone, PartialEq)]
struct Accepted {
    path: String,
    line: usize,
    kind: String,
}

/// Read an accepted file: `path:line [kind]` per line, as the report prints a
/// survivor or a hung mutant, so a line can be copied from it as it stands;
/// text after `]` is a note. Blank lines and `#` lines are skipped. Any other
/// line is an error that names it: a gate that skipped what it cannot read
/// would pass a survivor the file meant to name and missed.
fn parse_accepted(text: &str) -> Result<Vec<Accepted>> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let s = raw.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        match accepted_line(s) {
            Some(a) => out.push(a),
            None => bail!(
                "accepted file line {}: `{s}` is not `path:line [kind]` as `tri mutate spec` \
                 prints a survivor",
                i + 1
            ),
        }
    }
    Ok(out)
}

/// `path:line [kind]` and an optional note, or `None`.
fn accepted_line(s: &str) -> Option<Accepted> {
    let open = s.find(" [")?;
    let close = open + s[open..].find(']')?;
    let kind = &s[open + 2..close];
    let head = &s[..open];
    let colon = head.rfind(':')?;
    let line = head[colon + 1..].parse::<usize>().ok()?;
    if kind.is_empty() || kind.contains(' ') || colon == 0 {
        return None;
    }
    Some(Accepted { path: head[..colon].to_string(), line, kind: kind.to_string() })
}

/// The accepted line names this run's spec: the same file once both resolve,
/// else the same text without a leading `./`.
fn names_file(path: &str, file: &Path) -> bool {
    match (std::fs::canonicalize(path), std::fs::canonicalize(file)) {
        (Ok(a), Ok(b)) => a == b,
        _ => path.trim_start_matches("./") == file.to_string_lossy().trim_start_matches("./"),
    }
}

/// What the gate counts, from the mutants that ran. `missed` lists the
/// not-killed mutants the file does not name; `now_killed` the accepted lines
/// whose mutants all ran and were all killed. A line whose mutants did not run
/// (another `--fn`, past `--max`, a line that moved) or were all unviable is in
/// neither list, as survivors.t27 says.
struct GateCounts<'a> {
    survived: u32,
    hung: u32,
    accepted_hits: u32,
    missed: Vec<&'a SpecMutant>,
    now_killed: Vec<&'a Accepted>,
}

fn gate_counts<'a>(
    file: &Path,
    ran: &[(&'a SpecMutant, Fate)],
    accepted: &'a [Accepted],
) -> GateCounts<'a> {
    let mine: Vec<&Accepted> = accepted.iter().filter(|a| names_file(&a.path, file)).collect();
    let listed = |m: &SpecMutant| mine.iter().any(|a| a.line == m.line && a.kind == m.kind);
    let not_killed: Vec<&SpecMutant> = ran
        .iter()
        .filter(|(_, f)| matches!(f, Fate::Survived | Fate::Hang(_)))
        .map(|(m, _)| *m)
        .collect();
    let mut now_killed = Vec::new();
    for a in &mine {
        let fates: Vec<&Fate> = ran
            .iter()
            .filter(|(m, f)| m.line == a.line && m.kind == a.kind && !matches!(f, Fate::Unviable(_)))
            .map(|(_, f)| f)
            .collect();
        if !fates.is_empty()
            && fates.iter().all(|f| matches!(f, Fate::Killed(_)))
            && !now_killed.contains(a)
        {
            now_killed.push(*a);
        }
    }
    GateCounts {
        survived: ran.iter().filter(|(_, f)| *f == Fate::Survived).count() as u32,
        hung: ran.iter().filter(|(_, f)| matches!(f, Fate::Hang(_))).count() as u32,
        accepted_hits: not_killed.iter().filter(|m| listed(m)).count() as u32,
        missed: not_killed.into_iter().filter(|m| !listed(m)).collect(),
        now_killed,
    }
}

/// The gate's lines under the report, and its exit code.
fn gate_report(file: &Path, from: Option<&str>, c: &GateCounts) -> (String, u8) {
    let code = survivor_exit(
        true,
        c.survived,
        c.hung,
        c.accepted_hits,
        c.now_killed.len() as u32,
    );
    let mut out = format!(
        "Survivor gate (specs/tri/mutate/survivors.t27): {} not killed ({} survived, {} hung), {}; \
         {} accepted line(s) now killed.\n",
        not_killed(c.survived, c.hung),
        c.survived,
        c.hung,
        match from {
            Some(p) => format!("{} accepted by {p}", c.accepted_hits),
            None => "no --accepted file".to_string(),
        },
        c.now_killed.len()
    );
    if !c.missed.is_empty() {
        out.push_str(&format!("{} NOT ACCEPTED -- a test gap until a test kills it or the file names it:\n", c.missed.len()));
        for m in &c.missed {
            out.push_str(&format!("  {}:{} [{}]\n", file.display(), m.line, m.kind));
        }
    }
    if !c.now_killed.is_empty() {
        out.push_str(&format!("{} ACCEPTED BUT KILLED -- remove the line from {}:\n", c.now_killed.len(), from.unwrap_or("the accepted file")));
        for a in &c.now_killed {
            out.push_str(&format!("  {}:{} [{}]\n", a.path, a.line, a.kind));
        }
    }
    out.push_str(if code == EXIT_OK { "Gate passed.\n" } else { "Gate FAILED: exit 2.\n" });
    (out, code)
}

fn resolve_t27c(explicit: Option<&str>) -> String {
    if let Some(p) = explicit {
        return p.to_string();
    }
    ["target/release/t27c", "target/debug/t27c"]
        .iter()
        .find(|p| Path::new(p).exists())
        .map(|p| p.to_string())
        .unwrap_or_else(|| "t27c".to_string())
}

/// Mutants per kind, in a fixed order, so a function with only arithmetic no
/// longer reads as "1 site" (#7022).
fn kind_counts(ms: &[SpecMutant]) -> String {
    let kinds = ["drop-guard", "flip-cmp", "swap-logic", "drop-step", "swap-arith", "ret-default"];
    kinds
        .iter()
        .map(|k| (k, ms.iter().filter(|m| m.kind == *k).count()))
        .filter(|(_, n)| *n > 0)
        .map(|(k, n)| format!("{k} {n}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The `specs/` directory t27c resolves a `use` line against: the first
/// ancestor of the spec that holds a `specs/` or is one, walked the way
/// `find_specs_root` in bootstrap/src/use_resolve.rs walks it.
fn specs_root(file: &Path) -> Option<PathBuf> {
    let abs = std::fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf());
    let mut dir = abs.parent()?.to_path_buf();
    loop {
        if dir.join("specs").is_dir() {
            return Some(dir.join("specs"));
        }
        if dir.file_name().is_some_and(|n| n == "specs") {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Where the copies of `file` are written: `target/` beside the spec's
/// `specs/`, which t27c's walk from a copy reaches again. In the system temp
/// dir t27c finds no `specs/`, drops every `use` and still exits 0 (#7176),
/// so a spec that imports anything failed its own baseline there (#7148:
/// `specs/policy/l2_generation.t27`, zig "use of undeclared identifier
/// 'LIST_END'"). A spec with no `specs/` above it keeps the temp dir.
fn spec_work_dir(file: &Path, pid: u32) -> PathBuf {
    let name = format!("tri-mutate-spec-{pid}");
    match specs_root(file).as_deref().and_then(Path::parent) {
        Some(top) => top.join("target").join(name),
        None => std::env::temp_dir().join(name),
    }
}

fn mutate_spec(
    file: &Path,
    func: Option<&str>,
    max: usize,
    jobs: usize,
    zig_j: u32,
    secs: u64,
    t27c: Option<&str>,
    gate: &GateArgs,
) -> Result<()> {
    let original =
        std::fs::read_to_string(file).with_context(|| format!("cannot read {}", file.display()))?;
    // Read before the baseline: a file the gate cannot read fails in seconds,
    // not after the whole run.
    let accepted = match gate.accepted {
        Some(p) => parse_accepted(
            &std::fs::read_to_string(p).with_context(|| format!("cannot read --accepted {p}"))?,
        )
        .with_context(|| format!("in --accepted {p}"))?,
        None => Vec::new(),
    };
    let on = gate_on(gate.fail_on_survived, gate.accepted.is_some());
    if let Some(f) = func {
        if !original.split('\n').any(|l| t27_fn_header(l).as_deref() == Some(f)) {
            bail!("no function named `{f}` in {}", file.display());
        }
    }
    let t27c = resolve_t27c(t27c);
    let dir = spec_work_dir(file, std::process::id());
    std::fs::create_dir_all(&dir)?;

    // The unmutated spec must pass first: against a red baseline every mutant
    // "is killed" and the count says nothing. The copy lives in the work dir
    // like the mutants, so a `use` it cannot resolve fails here, loudly.
    let base = dir.join("base.t27");
    std::fs::write(&base, &original)?;
    let fate = spec_fate(&t27c, &base, &dir.join("base.zig"), zig_j, secs)?;
    if fate != Fate::Survived {
        let _ = std::fs::remove_dir_all(&dir);
        bail!(
            "the unmutated spec does not pass ({fate:?} via `{t27c} gen`, the zig compile and \
             the test run), so no mutant would mean anything. Fix that first."
        );
    }

    let all = find_spec_mutants(&original, func);
    let truncated = all.len() > max;
    let mutants: Vec<SpecMutant> = all.into_iter().take(max).collect();
    let scope = func.map(|f| format!(" (fn {f})")).unwrap_or_default();
    if mutants.is_empty() {
        let _ = std::fs::remove_dir_all(&dir);
        println!("No guard, operator or body sites in {}{scope}.", file.display());
        return Ok(());
    }
    println!(
        "{} mutant(s) in {}{scope}, {} at a time, zig -j{zig_j} each: {}.",
        mutants.len(),
        file.display(),
        jobs.max(1),
        kind_counts(&mutants)
    );
    if truncated {
        println!(
            "THIS IS A PREFIX, NOT THE SPEC: the walk stopped at --max {max}. Every \
             count below describes only the mutants reached."
        );
    }

    let next = std::sync::atomic::AtomicUsize::new(0);
    let fates: Vec<std::sync::Mutex<Option<Result<Fate, String>>>> =
        mutants.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for _ in 0..jobs.max(1) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if i >= mutants.len() {
                    break;
                }
                let spec = dir.join(format!("m{i}.t27"));
                let zig = dir.join(format!("m{i}.zig"));
                let fate = std::fs::write(&spec, apply_spec_mutant(&original, &mutants[i]))
                    .map_err(anyhow::Error::from)
                    .and_then(|_| spec_fate(&t27c, &spec, &zig, zig_j, secs))
                    .map_err(|e| format!("{e:#}"));
                *fates[i].lock().unwrap() = Some(fate);
                let _ = std::fs::remove_file(&spec);
                let _ = std::fs::remove_file(&zig);
                let _ = std::fs::remove_file(zig.with_extension("err"));
                let _ = std::fs::remove_file(zig.with_extension("bin"));
            });
        }
    });
    let _ = std::fs::remove_dir_all(&dir);

    let mut ran = Vec::new();
    let mut not_run = Vec::new();
    for (m, f) in mutants.iter().zip(fates.iter()) {
        match f.lock().unwrap().take() {
            Some(Ok(fate)) => ran.push((m, fate)),
            Some(Err(e)) => not_run.push((m, e)),
            None => not_run.push((m, "never run".to_string())),
        }
    }
    print!("{}", spec_report(file, &ran));
    if !not_run.is_empty() {
        println!("{} NOT RUN -- counted nowhere above:", not_run.len());
        for (m, e) in &not_run {
            println!("  {}:{} [{}]: {e}", file.display(), m.line, m.kind);
        }
    }
    let mut survivors = EXIT_OK;
    if on && not_run.is_empty() {
        let counts = gate_counts(file, &ran, &accepted);
        let (lines, code) = gate_report(file, gate.accepted, &counts);
        print!("{lines}");
        survivors = code;
    }
    match mutate_exit(true, not_run.len() as u32, survivors) {
        EXIT_OK => Ok(()),
        EXIT_SURVIVED => {
            use std::io::Write;
            let _ = std::io::stdout().flush();
            std::process::exit(EXIT_SURVIVED as i32);
        }
        _ => bail!("{} of {} mutant(s) could not be run", not_run.len(), mutants.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #7148: the baseline copy of `specs/policy/l2_generation.t27` sat in the
    /// system temp dir, where t27c finds no `specs/` to resolve
    /// `use policy::own_language;` against, so the spec "did not pass". A copy
    /// in the work dir must see the spec's own `specs/`, also when that tree
    /// is nested under another directory.
    #[test]
    fn a_copy_in_the_work_dir_resolves_use_against_the_specs_own_tree() {
        let tmp = std::env::temp_dir().join(format!("tri-mutate-root-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        for (spec, top) in [
            ("r/specs/policy/a.t27", "r"),
            ("r/b.t27", "r"),
            ("n/lib/specs/c.t27", "n/lib"),
        ] {
            let file = tmp.join(spec);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(&file, "module A;\n").unwrap();
            let root = std::fs::canonicalize(&tmp).unwrap();
            let work = spec_work_dir(&file, 7);
            assert_eq!(work, root.join(top).join("target").join("tri-mutate-spec-7"), "{spec}");
            let want = specs_root(&file);
            assert_eq!(want, Some(root.join(top).join("specs")), "{spec}");
            assert_eq!(specs_root(&work.join("m0.t27")), want, "{spec}");
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }

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

    // ---- tri mutate spec (#6993) ----

    fn kinds_on(ms: &[SpecMutant], line: usize) -> Vec<(&'static str, String)> {
        ms.iter()
            .filter(|m| m.line == line)
            .map(|m| (m.kind, m.after.trim().to_string()))
            .collect()
    }

    #[test]
    fn a_one_line_guard_is_dropped_and_the_header_arrow_is_not_flipped() {
        let src = "pub fn f(a: u8) -> u8 {\n    if (a >= 3) { return 0; }\n    return a;\n}\n";
        let ms = find_spec_mutants(src, None);
        let one = kinds_on(&ms, 1);
        assert_eq!(one, vec![("ret-default", "pub fn f(a: u8) -> u8 { return 0; }".to_string())]);
        let two = kinds_on(&ms, 2);
        assert!(two.contains(&("drop-guard", String::new())), "{two:?}");
        assert!(two.contains(&("flip-cmp", "if (a > 3) { return 0; }".to_string())), "{two:?}");
        assert_eq!(two.len(), 2, "{two:?}");
    }

    #[test]
    fn comments_strings_constants_and_tests_are_not_sites() {
        let src = "; a >= b and c\n\
                   pub const X : u8 = 1;\n\
                   pub fn f(a: u8) -> bool {\n\
                   \x20   // a == b\n\
                   \x20   return a == 1;\n\
                   }\n\
                   test t {\n\
                   \x20   assert f(1) == true;\n\
                   }\n\
                   invariant i { X < 2 }\n";
        let ms = find_spec_mutants(src, None);
        assert_eq!(ms.len(), 2, "{ms:?}");
        assert_eq!(ms[0].line, 5);
        assert_eq!(ms[0].after.trim(), "return a != 1;");
        assert_eq!((ms[1].kind, ms[1].line, ms[1].through), ("ret-default", 3, 6));
        let s = "pub fn g() -> u8 {\n    return \"a < b\";\n}\n";
        let only = find_spec_mutants(s, None);
        assert_eq!(only.iter().map(|m| m.kind).collect::<Vec<_>>(), vec!["ret-default"]);
    }

    #[test]
    fn shifts_and_arrows_are_not_comparisons() {
        let src = "pub fn f(lane: u64, tag: u64) -> u64 {\n    return ((lane >> tag) & 1) | (lane << 2);\n}\n";
        let ms = find_spec_mutants(src, None);
        assert!(!ms.iter().any(|m| m.kind == "flip-cmp"), "{ms:?}");
        assert_eq!(
            kinds_on(&ms, 2),
            vec![
                ("swap-arith", "return ((lane << tag) & 1) | (lane << 2);".to_string()),
                ("swap-arith", "return ((lane >> tag) | 1) | (lane << 2);".to_string()),
                ("swap-arith", "return ((lane >> tag) & 1) & (lane << 2);".to_string()),
                ("swap-arith", "return ((lane >> tag) & 1) | (lane >> 2);".to_string()),
            ]
        );
    }

    /// The jitter hash of #7002: the first version found no site in it at all.
    #[test]
    fn arithmetic_is_a_site_when_spaced() {
        let src = "pub fn h(seed: u64) -> u64 {\n    \
                   return (((seed % 4294967296) * PHI_HASH) % 4294967296) >> 16;\n}\n";
        let ms = find_spec_mutants(src, None);
        let two: Vec<String> = kinds_on(&ms, 2).into_iter().map(|(k, a)| format!("{k}: {a}")).collect();
        assert_eq!(
            two,
            vec![
                "swap-arith: return (((seed / 4294967296) * PHI_HASH) % 4294967296) >> 16;",
                "swap-arith: return (((seed % 4294967296) / PHI_HASH) % 4294967296) >> 16;",
                "swap-arith: return (((seed % 4294967296) * PHI_HASH) / 4294967296) >> 16;",
                "swap-arith: return (((seed % 4294967296) * PHI_HASH) % 4294967296) << 16;",
            ]
        );
        // Unspaced, unary and compound forms are not binary operators here.
        let not = "pub fn g(a: i32, b: bool) -> i32 {\n    \
                   var x = -a;\n    x += a*2;\n    if (b && true) { return x; }\n    return x;\n}\n";
        let ms = find_spec_mutants(not, None);
        assert!(!ms.iter().any(|m| m.kind == "swap-arith"), "{ms:?}");
    }

    #[test]
    fn a_body_is_replaced_by_its_default_return() {
        let src = "pub fn n(a: u32) -> u32 {\n    var t = a;\n    return t;\n}\n\
                   pub fn b(a: u32) -> bool {\n    return a == 2;\n}\n\
                   pub fn v(a: u32) {\n    var t = a;\n}\n\
                   pub fn r(a: f64) -> f64 {\n    return a;\n}\n\
                   pub fn s(a: u32) -> Pair {\n    return Pair{ .x = a };\n}\n\
                   pub fn z(a: u32) -> u32 {\n    return 0;\n}\n";
        let rd: Vec<(usize, usize, String)> = find_spec_mutants(src, None)
            .into_iter()
            .filter(|m| m.kind == "ret-default")
            .map(|m| (m.line, m.through, m.after))
            .collect();
        assert_eq!(
            rd,
            vec![
                (1, 4, "pub fn n(a: u32) -> u32 { return 0; }".to_string()),
                (5, 7, "pub fn b(a: u32) -> bool { return false; }".to_string()),
                (8, 10, "pub fn v(a: u32) { }".to_string()),
                (11, 13, "pub fn r(a: f64) -> f64 { return 0.0; }".to_string()),
            ],
            "a struct has no default the tool can name; `return 0;` is already the default"
        );
        let m = find_spec_mutants(src, Some("n")).into_iter().find(|m| m.kind == "ret-default").unwrap();
        let out = apply_spec_mutant(src, &m);
        assert_eq!(out.split('\n').count(), src.split('\n').count(), "line numbers kept");
        assert!(out.starts_with("pub fn n(a: u32) -> u32 { return 0; }\n\n\n\npub fn b("), "{out}");
    }

    /// 326 one-line functions in 72 specs had no site at all (2026-10-06).
    #[test]
    fn a_one_line_function_is_a_site_and_its_header_is_not() {
        let src = "pub fn f(a: u8) -> bool { return a > 1; }\n\
                   pub fn g(a: u8) -> Result<u8, E> { return a + 1; }\n";
        let ms = find_spec_mutants(src, None);
        assert_eq!(
            kinds_on(&ms, 1),
            vec![
                ("flip-cmp", "pub fn f(a: u8) -> bool { return a >= 1; }".to_string()),
                ("ret-default", "pub fn f(a: u8) -> bool { return false; }".to_string()),
            ]
        );
        assert_eq!(
            kinds_on(&ms, 2),
            vec![
                ("drop-step", "pub fn g(a: u8) -> Result<u8, E> { return a; }".to_string()),
                ("swap-arith", "pub fn g(a: u8) -> Result<u8, E> { return a - 1; }".to_string()),
            ],
            "the header's `->`, `<` and `>` are not sites; a Result has no default"
        );
        assert_eq!(find_spec_mutants(src, Some("g")).len(), 2);
    }

    #[test]
    fn logic_and_step_mutants() {
        let src = "pub fn f(a: bool, b: bool, n: u8) -> u8 {\n    \
                   var t = n - 1;\n    \
                   while (a and b) {\n        t = t + 1;\n    }\n    \
                   return t + 10;\n}\n";
        let ms = find_spec_mutants(src, None);
        assert_eq!(
            kinds_on(&ms, 2),
            vec![("drop-step", "var t = n;".to_string()), ("swap-arith", "var t = n + 1;".to_string())]
        );
        assert_eq!(kinds_on(&ms, 3), vec![("swap-logic", "while (a or b) {".to_string())]);
        assert_eq!(
            kinds_on(&ms, 4),
            vec![("drop-step", "t = t;".to_string()), ("swap-arith", "t = t - 1;".to_string())]
        );
        assert_eq!(
            kinds_on(&ms, 6),
            vec![("swap-arith", "return t - 10;".to_string())],
            "`+ 10` is not a step of one"
        );
    }

    #[test]
    fn the_fn_filter_keeps_one_body() {
        let src = "pub fn a(x: u8) -> bool {\n    return x == 1;\n}\n\
                   fn b(x: u8) -> bool {\n    return x != 1;\n}\n";
        let ms = find_spec_mutants(src, Some("b"));
        assert_eq!(ms.len(), 2, "{ms:?}");
        assert_eq!(ms[0].after.trim(), "return x == 1;");
        assert_eq!(ms[1].after, "fn b(x: u8) -> bool { return false; }");
        assert_eq!(find_spec_mutants(src, None).len(), 4);
    }

    #[test]
    fn a_dropped_line_keeps_every_line_number() {
        let src = "pub fn f(a: u8) -> u8 {\n    if (a >= 3) { return 0; }\n    return a;\n}\n";
        let m = find_spec_mutants(src, None)
            .into_iter()
            .find(|m| m.kind == "drop-guard")
            .unwrap();
        let out = apply_spec_mutant(src, &m);
        assert_eq!(out.split('\n').count(), src.split('\n').count());
        assert_eq!(out, "pub fn f(a: u8) -> u8 {\n\n    return a;\n}\n");
    }

    #[test]
    fn a_command_that_outlives_its_timeout_is_a_hang() {
        let r = run_with_timeout(
            Command::new("sh").arg("-c").arg("sleep 5"),
            std::process::Stdio::null(),
            std::process::Stdio::null(),
            1,
        )
        .unwrap();
        assert_eq!(r, None);
        let ok = run_with_timeout(
            Command::new("sh").arg("-c").arg("exit 0"),
            std::process::Stdio::null(),
            std::process::Stdio::null(),
            5,
        )
        .unwrap();
        assert_eq!(ok, Some(true));
    }

    /// A timed-out `zig test` used to leave its looping test binary running
    /// under PID 1. The killed command must take what it started with it.
    #[test]
    fn a_hang_takes_what_it_started_with_it() {
        let dir = std::env::temp_dir().join(format!("tri-mutate-orphan-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let pidfile = dir.join("pid");
        let script = format!("sleep 1000 & echo $! > {}; wait", pidfile.display());
        let r = run_with_timeout(
            Command::new("sh").arg("-c").arg(&script),
            std::process::Stdio::null(),
            std::process::Stdio::null(),
            1,
        )
        .unwrap();
        assert_eq!(r, None);
        let pid = std::fs::read_to_string(&pidfile).unwrap().trim().to_string();
        let _ = std::fs::remove_dir_all(&dir);
        // Gone, or a zombie waiting for a PID 1 that does not reap (the lab's).
        let alive = (0..40).all(|_| {
            let out = Command::new("ps").args(["-o", "stat=", "-p", &pid]).output().unwrap();
            let stat = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if stat.is_empty() || stat.starts_with('Z') {
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
            true
        });
        assert!(!alive, "the child of a timed-out command is still running (pid {pid})");
    }

    // ---- #7148: a hang and a compile error are not kills ----

    /// zig 0.16.0's own words for a lowered spec that does not compile, cut
    /// from `zig test --test-no-exec` on t27c output of a probe spec
    /// (2026-10-06): an invariant's assert, a comptime overflow inside an
    /// invariant, an endless loop inside one, and a type error.
    #[test]
    fn zig_says_which_compile_error_was_a_check() {
        let assert = "/tmp/m/inv.zig:8:9: error: assertion failed\n        @compileError(\"assertion failed\");\n\
                      /tmp/m/inv.zig:34:47: note: called at comptime here\n";
        let overflow = "/tmp/m/ovf.zig:21:11: error: overflow of integer type 'usize' with value '-1'\n\
                        /tmp/m/ovf.zig:35:19: note: called at comptime here\n";
        let quota = "/tmp/m/loop.zig:21:5: error: evaluation exceeded 1000 backwards branches\n    while (i < n) {\n\
                     /tmp/m/loop.zig:21:5: note: use @setEvalBranchQuota() to raise the branch limit from 1000\n\
                     /tmp/m/loop.zig:33:19: note: called at comptime here\n";
        let typed = "/tmp/m/ty.zig:24:12: error: expected type 'usize', found 'bool'\n\
                     /tmp/m/ty.zig:18:27: note: function return type declared here\n";
        assert_eq!(compile_fate(assert), Fate::Killed(KilledBy::Invariant));
        assert_eq!(
            compile_fate(overflow),
            Fate::Killed(KilledBy::Invariant),
            "a safety check failing while an invariant is evaluated is the invariant noticing"
        );
        assert_eq!(
            compile_fate(quota),
            Fate::Hang("zig's comptime branch quota, evaluating an invariant"),
            "the branch quota is comptime's timeout, though the note is there too"
        );
        assert_eq!(
            compile_fate(typed),
            Fate::Unviable("zig: error: expected type 'usize', found 'bool'".to_string())
        );
        assert_eq!(
            compile_fate(""),
            Fate::Unviable("zig: exited non-zero with no `error:` line".to_string())
        );
    }

    /// A lowered spec in miniature, in t27c's shape: its assert helper, one
    /// function, one invariant (a `comptime` block) and one test.
    fn lowered(func: &str, invariant: &str, test: &str) -> String {
        format!(
            "const std = @import(\"std\");\n\
             fn fail() noreturn {{\n    if (@inComptime()) {{\n        @compileError(\"assertion failed\");\n    \
             }} else {{\n        @panic(\"assertion failed\");\n    }}\n}}\n\
             {func}\n\
             comptime {{\n    {invariant}\n}}\n\
             test \"t\" {{\n    {test}\n}}\n"
        )
    }

    const COUNT_TO: &str = "fn count_to(n: usize) usize {\n    var i: usize = 0;\n    \
                            while (i < n) {\n        i += 1;\n    }\n    return i;\n}";

    /// The mutant `i += 1` dropped, with t27c's `_ = &i;` so zig still compiles it.
    fn count_to_without_its_step() -> String {
        COUNT_TO.replace("        i += 1;\n", "        _ = &i;\n")
    }

    #[test]
    fn each_compile_gets_its_zig_threads() {
        let c = zig_compile(Path::new("/w/m1.zig"), Path::new("/w/m1.bin"), 6);
        let args: Vec<_> = c.get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, ["test", "/w/m1.zig", "--test-no-exec", "-femit-bin=/w/m1.bin", "-j6"]);
        assert_eq!(c.get_current_dir(), Some(Path::new("/w")));
    }

    fn zig_fate_of(name: &str, src: &str, compile_secs: u64, run_secs: u64) -> Result<Fate> {
        let dir = std::env::temp_dir().join(format!("tri-mutate-fate-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let zig = dir.join("m.zig");
        std::fs::write(&zig, src).unwrap();
        let fate = zig_fate(&zig, 1, compile_secs, run_secs);
        let _ = std::fs::remove_dir_all(&dir);
        fate
    }

    #[test]
    fn a_failing_test_kills_and_a_passing_one_survives() {
        let pass = lowered(COUNT_TO, "if (!(count_to(3) == 3)) fail();", "if (!(count_to(5) == 5)) fail();");
        assert_eq!(zig_fate_of("pass", &pass, 300, 60).unwrap(), Fate::Survived);
        let fail = lowered(COUNT_TO, "if (!(count_to(3) == 3)) fail();", "if (!(count_to(5) == 6)) fail();");
        assert_eq!(zig_fate_of("fail", &fail, 300, 60).unwrap(), Fate::Killed(KilledBy::Test));
    }

    /// #7148's first criterion: a dropped step loops forever when the tests
    /// run. It used to count as killed.
    #[test]
    fn an_endless_loop_in_a_test_is_a_hang_not_a_kill() {
        let src = lowered(&count_to_without_its_step(), "", "if (!(count_to(5) == 5)) fail();");
        assert_eq!(
            zig_fate_of("spin", &src, 300, 2).unwrap(),
            Fate::Hang("its tests outlived --timeout")
        );
    }

    #[test]
    fn an_endless_loop_in_an_invariant_is_a_hang_while_zig_compiles() {
        let src = lowered(&count_to_without_its_step(), "if (!(count_to(3) == 3)) fail();", "");
        assert_eq!(
            zig_fate_of("quota", &src, 300, 60).unwrap(),
            Fate::Hang("zig's comptime branch quota, evaluating an invariant")
        );
    }

    #[test]
    fn an_invariant_the_mutant_breaks_kills_it_while_zig_compiles() {
        let src = lowered(COUNT_TO, "if (!(count_to(3) == 4)) fail();", "if (!(count_to(5) == 5)) fail();");
        assert_eq!(zig_fate_of("inv", &src, 300, 60).unwrap(), Fate::Killed(KilledBy::Invariant));
    }

    /// #7148's second criterion, as measured: a mutant zig cannot type is not
    /// a kill. (A parameter a mutant leaves unused is no such mutant: t27c
    /// emits `_ = p;` for it, and it compiles.)
    #[test]
    fn a_mutant_zig_cannot_type_is_unviable() {
        let src = lowered(
            &COUNT_TO.replace("    return i;", "    return true;"),
            "",
            "if (!(count_to(5) == 5)) fail();",
        );
        assert_eq!(
            zig_fate_of("type", &src, 300, 60).unwrap(),
            Fate::Unviable("zig: error: expected type 'usize', found 'bool'".to_string())
        );
    }

    /// A compile that outlives its clock is the machine's load: NOT RUN, an
    /// error the run reports, never a hang the mutant is charged with.
    #[test]
    fn a_compile_that_outlives_its_clock_is_not_a_hang() {
        let src = lowered(COUNT_TO, "", "if (!(count_to(5) == 5)) fail();");
        let e = zig_fate_of("slow", &src, 0, 60).unwrap_err();
        assert!(format!("{e:#}").contains("the machine's load, not the mutant"), "{e:#}");
    }

    #[test]
    fn a_spec_t27c_cannot_lower_is_unviable() {
        let dir = std::env::temp_dir().join(format!("tri-mutate-gen-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let spec = dir.join("m.t27");
        std::fs::write(&spec, "module M;\n").unwrap();
        let fate = spec_fate("false", &spec, &dir.join("m.zig"), 1, 60);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(
            fate.unwrap(),
            Fate::Unviable("t27c gen: exited non-zero with no `error:` line".to_string())
        );
    }

    /// The summary counts add up to the mutants run, and a hang and an
    /// unviable mutant are listed by line and kind like a survivor.
    #[test]
    fn the_report_lists_what_was_not_killed_by_line_and_kind() {
        let m = |line: usize, kind: &'static str, before: &str, after: &str| SpecMutant {
            line,
            through: line,
            kind,
            before: before.to_string(),
            after: after.to_string(),
        };
        let step = m(7, "drop-step", "        i += 1;", "");
        let guard = m(3, "drop-guard", "    if (a > b) { return a; }", "");
        let cmp = m(9, "flip-cmp", "    return a < b;", "    return a <= b;");
        let arith = m(12, "swap-arith", "    return a + 1;", "    return a - 1;");
        let ret = m(15, "ret-default", "pub fn r() -> u8 {", "pub fn r() -> u8 { return 0; }");
        let ran = vec![
            (&step, Fate::Hang("its tests outlived --timeout")),
            (&guard, Fate::Killed(KilledBy::Test)),
            (&cmp, Fate::Killed(KilledBy::Invariant)),
            (&arith, Fate::Unviable("zig: error: expected type 'usize', found 'bool'".to_string())),
            (&ret, Fate::Survived),
        ];
        let out = spec_report(Path::new("s.t27"), &ran);
        assert_eq!(
            out.lines().next().unwrap(),
            "2 of 5 killed (1 by a failing test, 1 by an invariant at compile time); \
             1 survived, 1 hung, 1 unviable."
        );
        assert!(
            out.contains("1 HUNG -- not killed: no check failed, the mutant never finished:\n  \
                          s.t27:7 [drop-step] (its tests outlived --timeout)\n    - i += 1;\n    + (line dropped)\n"),
            "{out}"
        );
        assert!(
            out.contains("1 UNVIABLE -- not killed: t27c gen or zig rejected the mutant before any check ran:\n  \
                          s.t27:12 [swap-arith]: zig: error: expected type 'usize', found 'bool'\n    \
                          - return a + 1;\n    + return a - 1;\n"),
            "{out}"
        );
        assert!(out.contains("1 SURVIVED -- the spec's tests did not notice:\n  s.t27:15 [ret-default]\n"), "{out}");
        assert!(!out.contains("s.t27:3 ") && !out.contains("s.t27:9 "), "a killed mutant is not listed: {out}");
        let none = spec_report(Path::new("s.t27"), &[(&guard, Fate::Killed(KilledBy::Test))]);
        assert_eq!(
            none,
            "1 of 1 killed (1 by a failing test, 0 by an invariant at compile time); \
             0 survived, 0 hung, 0 unviable.\n",
            "nothing listed when everything was killed"
        );
    }

    /// A value in a spec's asserts: a bool or a count.
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum V {
        B(bool),
        N(u32),
    }

    type Vars = std::collections::HashMap<String, V>;
    type Consts = std::collections::HashMap<String, u32>;

    /// The Rust copy of a spec's functions: name, the whole call (for messages), arguments.
    type Calls<'a> = &'a dyn Fn(&str, &str, &[V]) -> V;

    /// Split a call's arguments at the commas outside parentheses.
    fn top_args(s: &str) -> Vec<&str> {
        let (mut depth, mut start, mut out) = (0, 0, Vec::new());
        for (i, c) in s.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                ',' if depth == 0 => {
                    out.push(&s[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
        out.push(&s[start..]);
        out
    }

    /// Argument `i` of the call `e`, as a bool.
    fn arg_b(a: &[V], i: usize, e: &str) -> bool {
        match a[i] {
            V::B(v) => v,
            other => panic!("argument {i} of `{e}` is {other:?}, not a bool"),
        }
    }

    /// Argument `i` of the call `e`, as a count.
    fn arg_n(a: &[V], i: usize, e: &str) -> u32 {
        match a[i] {
            V::N(v) => v,
            other => panic!("argument {i} of `{e}` is {other:?}, not a count"),
        }
    }

    /// Evaluate one side of a spec's assert with the Rust copy of its functions.
    fn eval(e: &str, vars: &Vars, consts: &Consts, calls: Calls<'_>) -> V {
        let e = e.trim();
        match e {
            "true" => return V::B(true),
            "false" => return V::B(false),
            _ => {}
        }
        if let Ok(n) = e.parse::<u32>() {
            return V::N(n);
        }
        if let (Some(open), true) = (e.find('('), e.ends_with(')')) {
            let a: Vec<V> = top_args(&e[open + 1..e.len() - 1]).iter().map(|x| eval(x, vars, consts, calls)).collect();
            return calls(e[..open].trim(), e, &a);
        }
        if let Some(v) = vars.get(e) {
            return *v;
        }
        match consts.get(e) {
            Some(n) => V::N(*n),
            None => panic!("cannot evaluate `{e}`"),
        }
    }

    /// Every `assert` row in the tests of the spec at `rel` (from the repo root)
    /// holds for `calls`, and the spec's `pub const`s are exactly `rust_consts`.
    /// The rows are read from the spec, not copied here: a row the spec changes
    /// fails the caller's test until the Rust follows it, and so does a constant
    /// the spec adds or changes.
    fn assert_every_row_of(rel: &str, rust_consts: &[(&str, u32)], calls: Calls<'_>) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel);
        let spec = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let mut consts = Consts::new();
        for l in spec.lines() {
            if let Some(rest) = l.strip_prefix("pub const ") {
                let (name, val) = rest.split_once(" = ").expect("pub const NAME : T = N;");
                let name = name.split(':').next().unwrap().trim().to_string();
                consts.insert(name, val.trim_end_matches(';').parse::<u32>().unwrap());
            }
        }
        let mut want: Vec<(String, u32)> = rust_consts.iter().map(|(n, v)| (n.to_string(), *v)).collect();
        let mut got: Vec<(String, u32)> = consts.iter().map(|(n, v)| (n.clone(), *v)).collect();
        want.sort();
        got.sort();
        assert_eq!(got, want, "{rel}: the spec's constants are the Rust copy's");
        let (mut test, mut vars, mut rows) = (None::<String>, Vars::new(), 0);
        for l in spec.lines() {
            let t = l.trim();
            if let Some(name) = l.strip_prefix("test ") {
                test = Some(name.trim_end_matches(" {").to_string());
                vars.clear();
            } else if l == "}" {
                test = None;
            } else if let (Some(_), Some(rest)) = (&test, t.strip_prefix("var ")) {
                let (lhs, rhs) = rest.split_once(" = ").expect("var X : T = E;");
                let var = lhs.split(':').next().unwrap().trim().to_string();
                let v = eval(rhs.trim_end_matches(';'), &vars, &consts, calls);
                vars.insert(var, v);
            } else if let (Some(name), Some(rest)) = (&test, t.strip_prefix("assert ")) {
                let (lhs, rhs) = rest.trim_end_matches(';').split_once(" == ").expect("assert A == B;");
                assert_eq!(eval(lhs, &vars, &consts, calls), eval(rhs, &vars, &consts, calls), "{rel}: test {name}: {t}");
                rows += 1;
            }
        }
        let in_spec = spec.lines().filter(|l| l.trim_start().starts_with("assert ")).count();
        assert!(rows > 0, "no assert rows read from {}", path.display());
        assert_eq!(rows, in_spec, "{rel}: every assert row of the spec is evaluated, none skipped");
    }

    /// #7303: every `assert` row in the tests of specs/tri/mutate/survivors.t27
    /// holds for the Rust gate, and its EXIT_ codes are the spec's.
    #[test]
    fn the_gate_agrees_with_every_assert_row_of_its_spec() {
        let consts = [("EXIT_OK", EXIT_OK as u32), ("EXIT_FAILED", EXIT_FAILED as u32), ("EXIT_SURVIVED", EXIT_SURVIVED as u32)];
        assert_every_row_of("specs/tri/mutate/survivors.t27", &consts, &|f, e, a| match f {
            "gate_on" => V::B(gate_on(arg_b(a, 0, e), arg_b(a, 1, e))),
            "not_killed" => V::N(not_killed(arg_n(a, 0, e), arg_n(a, 1, e))),
            "unaccepted" => V::N(unaccepted(arg_n(a, 0, e), arg_n(a, 1, e))),
            "survivor_exit" => V::N(survivor_exit(arg_b(a, 0, e), arg_n(a, 1, e), arg_n(a, 2, e), arg_n(a, 3, e), arg_n(a, 4, e)) as u32),
            "mutate_exit" => V::N(mutate_exit(arg_b(a, 0, e), arg_n(a, 1, e), arg_n(a, 2, e) as u8) as u32),
            f => panic!("survivors.t27 calls `{f}`, which the Rust gate does not have"),
        });
    }

    /// #7050: every `assert` row in the tests of specs/tri/mutate/lab.t27 holds
    /// for the Rust copy in `lab`, and its constants are the spec's. The lab's
    /// copy of the gate's 2 is the gate's own constant, so a drift between the
    /// two specs fails here or in the test above.
    #[test]
    fn the_lab_rules_agree_with_every_assert_row_of_their_spec() {
        use lab::*;
        let consts = [
            ("RUN_NONE", RUN_NONE as u32),
            ("RUN_RUNNING", RUN_RUNNING as u32),
            ("RUN_DONE", RUN_DONE as u32),
            ("RUN_LOST", RUN_LOST as u32),
            ("EXIT_OK", lab::EXIT_OK as u32),
            ("EXIT_TOOL_FAILED", EXIT_TOOL_FAILED as u32),
            ("EXIT_SURVIVED", lab::EXIT_SURVIVED as u32),
            ("EXIT_NO_RESULT", EXIT_NO_RESULT as u32),
            ("EXIT_ORPHANS", EXIT_ORPHANS as u32),
            ("EXIT_STILL_RUNNING", EXIT_STILL_RUNNING as u32),
            ("TOOL_RC_SURVIVED", TOOL_RC_SURVIVED),
            ("SSH_MAX_ATTEMPTS", SSH_MAX_ATTEMPTS),
            ("POLL_BASE_SECONDS", POLL_BASE_SECONDS),
            ("POLL_CAP_SECONDS", POLL_CAP_SECONDS),
            ("PID_RESERVE", PID_RESERVE),
            ("JOB_OVERHEAD_PIDS", JOB_OVERHEAD_PIDS),
        ];
        assert_every_row_of("specs/tri/mutate/lab.t27", &consts, &|f, e, a| match f {
            "run_state" => V::N(run_state(arg_b(a, 0, e), arg_b(a, 1, e), arg_b(a, 2, e)) as u32),
            "launch_allowed" => V::B(launch_allowed(arg_n(a, 0, e) as u8)),
            "may_remove" => V::B(may_remove(arg_n(a, 0, e) as u8, arg_n(a, 1, e))),
            "lab_exit" => V::N(lab_exit(arg_n(a, 0, e) as u8, arg_n(a, 1, e), arg_n(a, 2, e)) as u32),
            "ssh_should_retry" => V::B(ssh_should_retry(arg_n(a, 0, e), arg_b(a, 1, e), arg_b(a, 2, e))),
            "poll_wait_seconds" => V::N(poll_wait_seconds(arg_n(a, 0, e))),
            "zig_j" => V::N(zig_j(arg_n(a, 0, e), arg_n(a, 1, e))),
            "job_pids" => V::N(job_pids(arg_n(a, 0, e))),
            "lab_jobs" => V::N(lab_jobs(arg_n(a, 0, e), arg_n(a, 1, e), arg_n(a, 2, e), arg_n(a, 3, e))),
            f => panic!("lab.t27 calls `{f}`, which the Rust copy does not have"),
        });
        assert_eq!(TOOL_RC_SURVIVED, super::EXIT_SURVIVED as u32, "the lab passes the gate's own 2 through");
    }

    /// A line copied from the report as it stands is an accepted entry: a
    /// survivor's `path:line [kind]`, and a hung mutant's with its reason.
    /// Anything else fails with its line number, never quietly.
    #[test]
    fn an_accepted_file_reads_as_the_report_prints() {
        let got = parse_accepted(
            "# equivalent: 5 * 2^k never equals 60\n\
             \n  specs/tri/mutate/lab.t27:100 [flip-cmp]\n\
             specs/x.t27:7 [drop-step] (its tests outlived --timeout)\n\
             ./a/b.t27:12 [swap-arith] -- equal at the boundary\n",
        )
        .unwrap();
        let a = |path: &str, line: usize, kind: &str| Accepted { path: path.to_string(), line, kind: kind.to_string() };
        assert_eq!(
            got,
            vec![
                a("specs/tri/mutate/lab.t27", 100, "flip-cmp"),
                a("specs/x.t27", 7, "drop-step"),
                a("./a/b.t27", 12, "swap-arith"),
            ]
        );
        for bad in ["specs/x.t27 [flip-cmp]", "specs/x.t27:7", "specs/x.t27:seven [flip-cmp]", "x.t27:7 []", ":7 [flip-cmp]"] {
            let e = parse_accepted(&format!("# ok\n{bad}\n")).unwrap_err().to_string();
            assert!(e.starts_with("accepted file line 2: "), "{bad}: {e}");
        }
        assert!(names_file("./specs/x.t27", Path::new("specs/x.t27")));
        assert!(!names_file("specs/y.t27", Path::new("specs/x.t27")));
    }

    /// The gate's counts from one run: what the file names is accepted; a
    /// listed line whose mutants were all killed is reported to be removed; an
    /// unviable mutant and another file's line count nowhere.
    #[test]
    fn the_gate_counts_what_ran_against_the_accepted_file() {
        let m = |line: usize, kind: &'static str| SpecMutant {
            line,
            through: line,
            kind,
            before: "x".to_string(),
            after: "y".to_string(),
        };
        let (s1, s2, h1, k1, k2a, k2b, u1) = (
            m(10, "flip-cmp"),
            m(11, "flip-cmp"),
            m(12, "drop-step"),
            m(13, "drop-guard"),
            m(14, "flip-cmp"),
            m(14, "flip-cmp"),
            m(15, "swap-arith"),
        );
        let ran = vec![
            (&s1, Fate::Survived),
            (&s2, Fate::Survived),
            (&h1, Fate::Hang("its tests outlived --timeout")),
            (&k1, Fate::Killed(KilledBy::Test)),
            (&k2a, Fate::Killed(KilledBy::Test)),
            (&k2b, Fate::Survived),
            (&u1, Fate::Unviable("zig: error: x".to_string())),
        ];
        let file = Path::new("s.t27");
        let accepted = parse_accepted(
            "s.t27:10 [flip-cmp]\ns.t27:13 [drop-guard]\ns.t27:14 [flip-cmp]\ns.t27:15 [swap-arith]\nother.t27:11 [flip-cmp]\n",
        )
        .unwrap();
        let c = gate_counts(file, &ran, &accepted);
        assert_eq!((c.survived, c.hung, c.accepted_hits), (3, 1, 2), "s1 and k2b are named; s2 and h1 are not");
        assert_eq!(c.missed.iter().map(|m| m.line).collect::<Vec<_>>(), vec![11, 12]);
        assert_eq!(c.now_killed.iter().map(|a| a.line).collect::<Vec<_>>(), vec![13], "14 still has a survivor; 15 was unviable");
        let (out, code) = gate_report(file, Some("acc.txt"), &c);
        assert_eq!(code, EXIT_SURVIVED);
        assert!(out.starts_with("Survivor gate (specs/tri/mutate/survivors.t27): 4 not killed (3 survived, 1 hung), 2 accepted by acc.txt; 1 accepted line(s) now killed.\n"), "{out}");
        assert!(out.contains("2 NOT ACCEPTED -- a test gap until a test kills it or the file names it:\n  s.t27:11 [flip-cmp]\n  s.t27:12 [drop-step]\n"), "{out}");
        assert!(out.contains("1 ACCEPTED BUT KILLED -- remove the line from acc.txt:\n  s.t27:13 [drop-guard]\n"), "{out}");
        assert!(out.ends_with("Gate FAILED: exit 2.\n"), "{out}");

        let all = parse_accepted("s.t27:10 [flip-cmp]\ns.t27:11 [flip-cmp]\ns.t27:12 [drop-step]\ns.t27:14 [flip-cmp]\n").unwrap();
        let (out, code) = gate_report(file, Some("acc.txt"), &gate_counts(file, &ran, &all));
        assert_eq!(code, EXIT_OK, "every not-killed mutant named, no named line killed: {out}");
        assert!(out.ends_with("Gate passed.\n"), "{out}");

        let none: Vec<Accepted> = Vec::new();
        let (out, code) = gate_report(file, None, &gate_counts(file, &[(&k1, Fate::Killed(KilledBy::Test))], &none));
        assert_eq!(code, EXIT_OK, "the flag alone on a run with no survivor passes");
        assert!(out.starts_with("Survivor gate (specs/tri/mutate/survivors.t27): 0 not killed (0 survived, 0 hung), no --accepted file; 0 accepted line(s) now killed.\n"), "{out}");
    }

    /// RFC 4648 section 10's vectors.
    #[test]
    fn base64_matches_the_rfc_vectors() {
        let vectors = [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"), ("foob", "Zm9vYg=="), ("fooba", "Zm9vYmE="), ("foobar", "Zm9vYmFy")];
        for (plain, coded) in vectors {
            assert_eq!(base64_encode(plain.as_bytes()), coded, "{plain:?}");
        }
    }

    /// A quoted word is one word that `sh` reads back as it was, quotes,
    /// dollars, backslashes and newlines included.
    #[test]
    fn a_quoted_word_reaches_the_shell_unchanged() {
        for w in ["plain", "it's", "a b", "$HOME `id` \\n", "line\nbreak", "''", ""] {
            let out = Command::new("sh").arg("-c").arg(format!("printf %s {}", sh_quote(w))).output().unwrap();
            assert_eq!(String::from_utf8(out.stdout).unwrap(), w);
        }
    }

    /// Only the lines between the markers are the answer; a stream that lost
    /// either marker is no answer at all, so a read can try again.
    #[test]
    fn an_answer_is_what_lies_between_the_markers() {
        let got = lab_marked("banner\r\nTRI-MUTATE-LAB-BEGIN\r\ndir 1\r\nTRI-MUTATE-LAB-END\r\n");
        assert_eq!(got, Some(vec!["dir 1".to_string()]));
        assert_eq!(lab_marked("TRI-MUTATE-LAB-BEGIN\nTRI-MUTATE-LAB-END\n"), Some(vec![]));
        assert_eq!(lab_marked("TRI-MUTATE-LAB-BEGIN\ndir 1\n"), None, "cut before the end");
        assert_eq!(lab_marked("dir 1\nTRI-MUTATE-LAB-END\n"), None, "no begin");
        assert_eq!(lab_marked("TRI-MUTATE-LAB-END\nTRI-MUTATE-LAB-BEGIN\n"), None, "out of order");
    }

    /// The probe's lines read as the state `lab::run_state` decides; a probe
    /// missing a line it always prints is no answer.
    #[test]
    fn a_probe_reads_as_the_spec_decides() {
        let lines = |s: &str| s.lines().map(str::to_string).collect::<Vec<_>>();
        let p = parse_lab_probe(&lines("dir 1\nexit 2\nalive 0\norphan 77 zig test x\npids 1000 745")).unwrap();
        assert_eq!((p.state(), p.exit, p.orphans.len(), p.pids_max, p.pids_used), (lab::RUN_DONE, Some(2), 1, 1000, 745));
        let p = parse_lab_probe(&lines("dir 1\nalive 1\npids max 3")).unwrap();
        assert_eq!((p.state(), p.pids_max), (lab::RUN_RUNNING, LAB_PIDS_UNCAPPED));
        assert_eq!(parse_lab_probe(&lines("dir 1\nalive 0\npids 1000 3")).unwrap().state(), lab::RUN_LOST);
        assert_eq!(parse_lab_probe(&lines("dir 0\nalive 0\npids 1000 3")).unwrap().state(), lab::RUN_NONE);
        let half = parse_lab_probe(&lines("dir 1\nexit \nalive 0\npids 1000 3")).unwrap();
        assert_eq!(lab::lab_exit(half.state(), half.exit.unwrap(), 0), lab::EXIT_TOOL_FAILED, "an exit file with no number");
        assert_eq!(parse_lab_probe(&lines("dir 1\npids 1000 3")), None, "no alive line");
        assert_eq!(parse_lab_probe(&lines("dir 1\nalive 1")), None, "no pids line");
    }

    fn lab_env_at(root: &Path) -> LabEnv {
        let r = root.display();
        LabEnv {
            local: true,
            railway: "railway".into(),
            project: None,
            environment: "production".into(),
            service: "t27c-lab".into(),
            dir: None,
            tri: format!("{r}/tri"),
            t27c: "/bin/true".into(),
            src: format!("{r}/src"),
            zig: "/nonexistent-zig".into(),
            runs: format!("{r}/runs"),
            cgroup: format!("{r}/cg"),
        }
    }

    fn lab_run_of(spec: &str, max: usize, wait: u64) -> LabRun {
        LabRun {
            rel: "specs/x/a.t27".into(),
            spec: spec.as_bytes().to_vec(),
            func: None,
            accepted: None,
            max,
            jobs: 2,
            zig_threads: None,
            secs: 60,
            fail_on_survived: false,
            wait,
        }
    }

    /// The run's name is the request: the same one finds the same directory,
    /// and any change to the spec, the arguments or the binaries is another run.
    #[test]
    fn the_run_name_is_the_request() {
        let env = lab_env_at(Path::new("/r"));
        let base = lab_run_name(&env, &lab_run_of("pub fn a() {}\n", 5, 0));
        assert!(base.starts_with("tri-mutate-") && base.len() == "tri-mutate-".len() + 16, "{base}");
        assert_eq!(base, lab_run_name(&env, &lab_run_of("pub fn a() {}\n", 5, 9)), "--lab-wait is not part of the run");
        assert_ne!(base, lab_run_name(&env, &lab_run_of("pub fn a() {} \n", 5, 0)));
        assert_ne!(base, lab_run_name(&env, &lab_run_of("pub fn a() {}\n", 6, 0)));
        let mut zj = lab_run_of("pub fn a() {}\n", 5, 0);
        zj.zig_threads = Some(6);
        assert_ne!(base, lab_run_name(&env, &zj), "--zig-threads is part of the run");
        let mut acc = lab_run_of("pub fn a() {}\n", 5, 0);
        acc.accepted = Some(Vec::new());
        assert_ne!(base, lab_run_name(&env, &acc), "an empty accepted file still turns the gate on");
        let other = LabEnv { tri: "/other/tri".into(), ..lab_env_at(Path::new("/r")) };
        assert_ne!(base, lab_run_name(&other, &lab_run_of("pub fn a() {}\n", 5, 0)));
    }

    /// The request is checked here, before anything reaches the lab: a path
    /// the lab's copy of the repo cannot hold, a `--fn` the spec does not
    /// have, an accepted file that does not parse.
    #[test]
    fn a_lab_request_is_checked_before_the_lab() {
        let ok = lab_request("./src/mutate.rs", Some("lab_request"), None, 5, 8, None, 60, false, 0).unwrap();
        assert_eq!(ok.rel, "src/mutate.rs");
        for bad in ["/tmp/x.t27", "../x.t27", "src/../x.t27"] {
            let e = lab_request(bad, None, None, 5, 8, None, 60, false, 0).err().expect(bad);
            assert!(format!("{e:#}").contains("relative to the repo root"), "{bad}: {e:#}");
        }
        let e = lab_request("src/mutate.rs", Some("no_such_fn"), None, 5, 8, None, 60, false, 0).err().unwrap();
        assert!(format!("{e:#}").contains("no function named `no_such_fn`"), "{e:#}");
        let acc = std::env::temp_dir().join(format!("tri-lab-acc-{}", std::process::id()));
        std::fs::write(&acc, "a.t27 line 3\n").unwrap();
        let e = lab_request("src/mutate.rs", None, acc.to_str(), 5, 8, None, 60, false, 0).err().unwrap();
        assert!(format!("{e:#}").contains("in --accepted"), "{e:#}");
        let _ = std::fs::remove_file(&acc);
    }

    /// A `railway` that cannot be started is an error, which `run` turns into
    /// exit 3, never a verdict.
    #[test]
    fn a_lab_that_cannot_be_reached_is_an_error() {
        let env = LabEnv { local: false, railway: "/nonexistent/railway".into(), ..lab_env_at(Path::new("/r")) };
        let mut out = Vec::new();
        let e = lab_mutate_spec(&env, &lab_run_of("x", 1, 0), &mut out, &|_| {}).err().unwrap();
        assert!(format!("{e:#}").contains("cannot reach the lab"), "{e:#}");
    }

    #[cfg(target_os = "linux")]
    const LAB_HELP: &str = "--file --fn --max --jobs --zig-threads --timeout --t27c --fail-on-survived --accepted";

    /// zig's -j a lab run on this machine passes: zig_j over its own cores, as the probe reads them.
    #[cfg(target_os = "linux")]
    fn lab_zig_j(jobs: u32) -> u32 {
        let n = Command::new("nproc").output().unwrap();
        lab::zig_j(String::from_utf8_lossy(&n.stdout).trim().parse().unwrap(), jobs)
    }

    /// A lab on this machine: `T27C_LAB_LOCAL=1` with a fake `tri` that
    /// prints the spec it was given, logs its arguments, holds while a `hold`
    /// file exists, leaves a `sleep` behind while an `orphan` file exists, and
    /// exits with the code in `rc`.
    #[cfg(target_os = "linux")]
    fn lab_fixture(tag: &str, help: &str) -> (PathBuf, LabEnv) {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("tri-lab-test-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for d in ["src/specs/x", "runs", "cg"] {
            std::fs::create_dir_all(root.join(d)).unwrap();
        }
        std::fs::write(root.join("src/specs/x/dep.t27"), "dep\n").unwrap();
        std::fs::write(root.join("cg/pids.max"), "1000\n").unwrap();
        std::fs::write(root.join("cg/pids.current"), "100\n").unwrap();
        std::fs::write(root.join("rc"), "0\n").unwrap();
        let r = root.display();
        let tri = format!(
            "#!/bin/sh\n\
             case \"$*\" in *--help*) echo '{help}'; exit 0;; esac\n\
             echo \"$*\" >> {r}/calls\n\
             test -f specs/x/dep.t27 || {{ echo 'no copy of specs/'; exit 9; }}\n\
             cat specs/x/a.t27\n\
             while [ -f {r}/hold ]; do sleep 0.05; done\n\
             if [ -f {r}/orphan ]; then sleep 30 & echo $! > {r}/orphan.pid; fi\n\
             echo '4 of 4 killed'\n\
             exit $(cat {r}/rc)\n"
        );
        std::fs::write(root.join("tri"), tri).unwrap();
        std::fs::set_permissions(root.join("tri"), std::fs::Permissions::from_mode(0o755)).unwrap();
        let env = lab_env_at(&root);
        (root, env)
    }

    #[cfg(target_os = "linux")]
    fn lab_go(env: &LabEnv, run: &LabRun) -> (u8, String) {
        let mut out = Vec::new();
        let code = lab_mutate_spec(env, run, &mut out, &|_| std::thread::sleep(std::time::Duration::from_millis(100))).unwrap();
        (code, String::from_utf8(out).unwrap())
    }

    #[cfg(target_os = "linux")]
    fn lab_calls(root: &Path) -> Vec<String> {
        std::fs::read_to_string(root.join("calls")).unwrap_or_default().lines().map(str::to_string).collect()
    }

    /// Base64 as written here is what `base64 -d` on the lab reads, for every
    /// byte value.
    #[cfg(target_os = "linux")]
    #[test]
    fn every_byte_survives_base64_on_the_lab() {
        let all: Vec<u8> = (0..=255u8).chain((0..=255u8).rev()).collect();
        let out = Command::new("sh").arg("-c").arg(format!("printf %s {} | base64 -d", base64_encode(&all))).output().unwrap();
        assert_eq!(out.stdout, all);
    }

    /// A run starts, its output comes back, the tool's code passes through
    /// as `lab_exit` maps it, and the directory goes.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_lab_run_reads_back_and_removes_its_directory() {
        let (root, env) = lab_fixture("pass", LAB_HELP);
        let run = lab_run_of("pub fn a() {}\n", 5, 3600);
        let (code, out) = lab_go(&env, &run);
        assert_eq!(code, lab::EXIT_OK, "{out}");
        assert!(out.contains("pub fn a() {}\n4 of 4 killed\n"), "the spec went up and the output came back: {out}");
        assert!(out.contains(&format!("Starting it: 2 job(s) of 2 asked, zig -j{} each", lab_zig_j(2))), "{out}");
        let d = format!("{}/{}", env.runs, lab_run_name(&env, &run));
        assert!(out.contains(&format!("Removed {d}.")), "{out}");
        assert!(!Path::new(&d).exists());
        assert_eq!(
            lab_calls(&root),
            vec![format!("mutate spec --file specs/x/a.t27 --jobs 2 --zig-threads {} --max 5 --timeout 60 --t27c /bin/true", lab_zig_j(2))]
        );
        for (rc, want) in [("2", lab::EXIT_SURVIVED), ("7", lab::EXIT_TOOL_FAILED), ("1", lab::EXIT_TOOL_FAILED)] {
            std::fs::write(root.join("rc"), rc).unwrap();
            let (code, out) = lab_go(&env, &lab_run_of(&format!("rc {rc}\n"), 5, 3600));
            assert_eq!(code, want, "tool rc {rc}: {out}");
        }
        assert_eq!(std::fs::read_dir(&env.runs).unwrap().count(), 0, "every run removed, no staging left");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The same run through `railway ssh` instead of `sh -c`: a stand-in
    /// `railway` records the target it was given and runs the script it got.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_lab_run_goes_through_railway_ssh() {
        use std::os::unix::fs::PermissionsExt;
        let (root, mut env) = lab_fixture("ssh", LAB_HELP);
        let r = root.display();
        let fake = format!(
            "#!/bin/sh\n\
             echo \"$1 $2 $3 $4 $5 $6 $7\" >> {r}/railway.args\n\
             for a; do last=$a; done\n\
             exec sh -c \"$last\"\n"
        );
        std::fs::write(root.join("railway"), fake).unwrap();
        std::fs::set_permissions(root.join("railway"), std::fs::Permissions::from_mode(0o755)).unwrap();
        (env.local, env.railway, env.project) = (false, format!("{r}/railway"), Some("p1".into()));
        let (code, out) = lab_go(&env, &lab_run_of("pub fn a() {}\n", 5, 3600));
        assert_eq!(code, lab::EXIT_OK, "{out}");
        assert!(out.contains("pub fn a() {}\n4 of 4 killed\n"), "{out}");
        let args = std::fs::read_to_string(root.join("railway.args")).unwrap();
        assert!(args.lines().count() >= 3, "probe, launch and read-back each went over ssh: {args}");
        assert!(args.lines().all(|l| l == "ssh -p p1 -e production -s t27c-lab"), "{args}");
        assert_eq!(lab_calls(&root).len(), 1, "one run started");
        assert_eq!(std::fs::read_dir(&env.runs).unwrap().count(), 0, "the run was removed");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The pids the lab has left cut the jobs; none left, nothing starts.
    #[cfg(target_os = "linux")]
    #[test]
    fn the_labs_free_pids_cut_the_jobs() {
        let (root, env) = lab_fixture("pids", LAB_HELP);
        std::fs::write(root.join("cg/pids.current"), format!("{}", 1000 - lab::PID_RESERVE - lab::job_pids(lab_zig_j(8)))).unwrap();
        let mut run = lab_run_of("pids\n", 5, 3600);
        run.jobs = 8;
        let (code, out) = lab_go(&env, &run);
        assert_eq!(code, lab::EXIT_OK, "{out}");
        let want = format!("--jobs 1 --zig-threads {} ", lab_zig_j(8));
        assert!(lab_calls(&root)[0].contains(&want), "a cut batch keeps the request's -j: {:?}", lab_calls(&root));
        // An explicit --zig-threads sets the cost: 2 jobs at -j1 cost 2 * job_pids(1).
        std::fs::write(root.join("cg/pids.current"), format!("{}", 1000 - lab::PID_RESERVE - 2 * lab::job_pids(1))).unwrap();
        let mut run = lab_run_of("pids at -j1\n", 5, 3600);
        (run.jobs, run.zig_threads) = (8, Some(1));
        let (code, out) = lab_go(&env, &run);
        assert_eq!(code, lab::EXIT_OK, "{out}");
        assert!(lab_calls(&root)[1].contains("--jobs 2 --zig-threads 1 "), "{:?}", lab_calls(&root));
        std::fs::write(root.join("cg/pids.current"), "900").unwrap();
        let (code, out) = lab_go(&env, &lab_run_of("pids again\n", 5, 3600));
        assert_eq!(code, lab::EXIT_NO_RESULT, "{out}");
        assert!(out.contains("Not started: the lab has 900 of 1000 pids in use"), "{out}");
        assert_eq!(lab_calls(&root).len(), 2, "nothing started");
        assert_eq!(std::fs::read_dir(&env.runs).unwrap().count(), 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Exit 5 leaves the run going; the same command called again reads that
    /// run and starts no second one.
    #[cfg(target_os = "linux")]
    #[test]
    fn the_same_command_again_reads_the_run_it_started() {
        let (root, env) = lab_fixture("again", LAB_HELP);
        std::fs::write(root.join("hold"), "").unwrap();
        let (code, out) = lab_go(&env, &lab_run_of("again\n", 5, 0));
        assert_eq!(code, lab::EXIT_STILL_RUNNING, "{out}");
        assert!(out.contains("Still running after 0 s of waiting"), "{out}");
        std::fs::remove_file(root.join("hold")).unwrap();
        let (code, out) = lab_go(&env, &lab_run_of("again\n", 5, 3600));
        assert_eq!(code, lab::EXIT_OK, "{out}");
        assert!(out.contains("Reading the run already there ("), "{out}");
        assert!(out.contains("4 of 4 killed"), "{out}");
        assert_eq!(lab_calls(&root).len(), 1, "one run, read twice");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A directory whose runner died with no exit file is LOST: exit 3, the
    /// directory goes, nothing is started over it; the next call starts afresh.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_lost_run_is_cleared_not_restarted() {
        let (root, env) = lab_fixture("lost", LAB_HELP);
        let run = lab_run_of("lost\n", 5, 3600);
        let d = PathBuf::from(format!("{}/{}", env.runs, lab_run_name(&env, &run)));
        std::fs::create_dir_all(&d).unwrap();
        let mut dead = Command::new("true").spawn().unwrap();
        dead.wait().unwrap();
        std::fs::write(d.join("pid"), dead.id().to_string()).unwrap();
        let (code, out) = lab_go(&env, &run);
        assert_eq!(code, lab::EXIT_NO_RESULT, "{out}");
        assert!(out.contains("Reading the run already there (lost)."), "{out}");
        assert!(out.contains("stopped without an exit code"), "{out}");
        assert!(!d.exists(), "{out}");
        assert!(lab_calls(&root).is_empty(), "nothing started over a lost run");
        let (code, out) = lab_go(&env, &run);
        assert_eq!(code, lab::EXIT_OK, "{out}");
        assert_eq!(lab_calls(&root).len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A process left working in the run's directory outranks the verdict
    /// (exit 4) and keeps the directory; once it is gone, the same command
    /// reads the verdict and removes the directory (trap T9).
    #[cfg(target_os = "linux")]
    #[test]
    fn an_orphan_keeps_the_directory_until_it_is_gone() {
        let (root, env) = lab_fixture("orphan", LAB_HELP);
        std::fs::write(root.join("orphan"), "").unwrap();
        let run = lab_run_of("orphan\n", 5, 3600);
        let d = PathBuf::from(format!("{}/{}", env.runs, lab_run_name(&env, &run)));
        let (code, out) = lab_go(&env, &run);
        assert_eq!(code, lab::EXIT_ORPHANS, "{out}");
        let pid = std::fs::read_to_string(root.join("orphan.pid")).unwrap().trim().to_string();
        assert!(out.contains(&format!("  {pid} sleep 30")), "{out}");
        assert!(d.exists(), "{out}");
        std::fs::remove_file(root.join("orphan")).unwrap();
        Command::new("kill").args(["-KILL", &pid]).status().unwrap();
        for _ in 0..100 {
            if std::fs::read_link(format!("/proc/{pid}/cwd")).is_err() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let (code, out) = lab_go(&env, &run);
        assert_eq!(code, lab::EXIT_OK, "{out}");
        assert!(!d.exists(), "{out}");
        assert_eq!(lab_calls(&root).len(), 1, "read again, not run again");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A tri built before a flag would answer clap's usage error, exit 2, which
    /// reads as the survivor gate's 2. run.sh looks each flag up in `--help`
    /// first, so the run fails as the tool (1) and names the flag.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_tri_without_a_flag_fails_as_the_tool_not_as_the_gate() {
        let (root, env) = lab_fixture("noflag", "--file --fn --max --jobs --zig-threads --timeout --t27c");
        let mut run = lab_run_of("noflag\n", 5, 3600);
        run.accepted = Some(b"specs/x/a.t27:1 [flip-cmp]\n".to_vec());
        let (code, out) = lab_go(&env, &run);
        assert_eq!(code, lab::EXIT_TOOL_FAILED, "{out}");
        assert!(out.contains("has no --accepted"), "{out}");
        assert!(lab_calls(&root).is_empty(), "the tool never ran");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A launch whose setup fails (here: no specs/ to copy) still writes an
    /// exit file, so it reads as a failed tool with the setup's errors, not as
    /// a lost run.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_failed_setup_reads_as_a_failed_tool() {
        let (root, mut env) = lab_fixture("setup", LAB_HELP);
        env.src = root.join("no-src").display().to_string();
        let (code, out) = lab_go(&env, &lab_run_of("setup\n", 5, 3600));
        assert_eq!(code, lab::EXIT_TOOL_FAILED, "{out}");
        assert!(out.contains("setup failed on the lab"), "{out}");
        assert!(out.contains("no-src/specs"), "cp's own error is the output: {out}");
        assert!(lab_calls(&root).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
}

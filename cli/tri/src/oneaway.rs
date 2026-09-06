//! Which failing specs are ONE defect from compiling, and what that defect is.
//!
//! WHY THIS EXISTS. Three correct emitter repairs in a row moved the Rust column by
//! **zero**, each while demonstrably fixing the emission. The reason was never measured
//! until it was: a fix flips a spec to OK only when it closes that spec's LAST error, and
//! the largest first-error classes are spread across specs that carry three or four more
//! defects each. Targeting the biggest class is a statement about FREQUENCY; moving the
//! column is a question about TERMINATION, and they are different questions asked of the
//! same data.
//!
//! Measured over 243 failing specs at the time this was written: 55 carry exactly one real
//! error, 38 carry two, 150 carry three or more, median four. The 55 are the only specs a
//! single repair can move, and this command names them and their sole class.
//!
//! THE COUNT HAS TO EXCLUDE ITS OWN SUMMARY LINE. `rustc` ends with
//! `error: aborting due to N previous errors`, which matches `^error` like any other.
//! Counting it inflates every bucket by one and turns "55 specs with one error" into
//! "zero specs with one error" -- the opposite conclusion, and the one written down first.

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
/// Failing specs whose generated Rust has exactly ONE rustc error, by class.
///
/// Every spec is read; the population is printed. A spec that does not GENERATE is not a
/// spec with zero errors, and is counted separately rather than folded into either side.
pub struct OneAway {
    /// Also list every spec, not only the class tally.
    #[arg(long)]
    pub names: bool,
    /// Report specs with exactly this many errors. The default, 1, is the only bucket a
    /// single repair can move; `--errors 2` is the next wave.
    #[arg(long, default_value_t = 1)]
    pub errors: usize,
    /// Stop after this many specs. Prints what was dropped -- a silent cap reads as
    /// coverage.
    #[arg(long)]
    pub limit: Option<usize>,
    /// Run the controls and report, changing nothing.
    #[arg(long)]
    pub self_check: bool,
}

/// One spec's outcome.
enum Outcome {
    /// `t27c gen-rust` refused. Not zero errors -- a different fact.
    NoGen,
    /// rustc accepted it.
    Ok,
    /// rustc refused it, with this many real errors, this first class, and whether that
    /// first error was one rustc stops at.
    Errors(usize, String, bool),
}

/// The class of one rustc error line, with the identifiers folded out.
///
/// `cannot find type \`Trit\`` and `cannot find type \`Graph\`` are one class; the names
/// belong to `--names`, not to the tally.
pub fn class_of(line: &str) -> String {
    let s = line
        .trim_start()
        .trim_start_matches("error")
        .trim_start_matches(|c: char| c == '[' || c.is_ascii_alphanumeric() || c == ']')
        .trim_start_matches(':')
        .trim();
    let mut out = String::new();
    let mut in_tick = false;
    for c in s.chars() {
        if c == '`' {
            if !in_tick {
                out.push_str("`X`");
            }
            in_tick = !in_tick;
        } else if !in_tick {
            out.push(c);
        }
    }
    out.trim().to_string()
}

/// Did rustc stop early on this one?
///
/// A diagnostic with an `[E####]` code came from a pass that ran to completion; one
/// without a code is a parse or lex error, and rustc abandons the file at the first of
/// those. So for a spec whose sole error has no code, "one error" is a LOWER BOUND, not a
/// count -- repair it and the next one appears.
///
/// Measured the day this was added: of 56 specs reported as carrying exactly one error,
/// **27** were of this kind. `server/http.t27` was one of them: its sole error was
/// `expected expression, found keyword `fn``, and repairing that revealed
/// `expected expression, found `@`` underneath. The bucket is honest about frequency and
/// dishonest about termination unless it says which half is which.
pub fn is_parse_error(line: &str) -> bool {
    !line.starts_with("error[")
}

/// Real errors: every `error` line rustc printed EXCEPT its own summary.
///
/// The summary is not a diagnostic about the program. Including it is the single mistake
/// that hid this whole measurement the first time it was taken.
pub fn real_error_lines(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter(|l| l.starts_with("error"))
        .filter(|l| !l.contains("aborting due to"))
        .collect()
}

fn repo_root() -> Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git rev-parse")?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

fn judge(root: &Path, spec: &Path, tmp: &Path) -> Result<Outcome> {
    let gen = Command::new(root.join("target/release/t27c"))
        .arg("gen-rust")
        .arg(spec)
        .current_dir(root)
        .output()
        .context("running t27c gen-rust")?;
    if !gen.status.success() {
        return Ok(Outcome::NoGen);
    }
    let src = tmp.join("a.rs");
    std::fs::write(&src, &gen.stdout).context("writing the generated Rust")?;
    // `-o` at a real path, never /dev/null: rustc writes its metadata through a temp file
    // NEXT TO the output, and /dev/null makes that fail on every input, valid or not.
    let rc = Command::new("rustc")
        .args(["--edition", "2021", "--crate-type", "lib", "--emit=metadata", "-A", "warnings", "-o"])
        .arg(tmp.join("o.rmeta"))
        .arg(&src)
        .output()
        .context("running rustc")?;
    if rc.status.success() {
        return Ok(Outcome::Ok);
    }
    let err = String::from_utf8_lossy(&rc.stderr);
    let lines = real_error_lines(&err);
    let first = lines.first().map(|l| class_of(l)).unwrap_or_default();
    let stopped = lines.first().map(|l| is_parse_error(l)).unwrap_or(false);
    Ok(Outcome::Errors(lines.len(), first, stopped))
}

fn specs(root: &Path) -> Result<Vec<PathBuf>> {
    let out = Command::new("git")
        .args(["ls-files", "-z", "--", "specs/"])
        .current_dir(root)
        .output()
        .context("running git ls-files")?;
    if !out.status.success() {
        anyhow::bail!("git ls-files failed; the population is unknown, which is not an empty one");
    }
    let mut v: Vec<PathBuf> = String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|p| p.ends_with(".t27") && !p.starts_with("specs/scratch/"))
        .map(|p| root.join(p))
        .collect();
    v.sort();
    Ok(v)
}

pub fn run(a: &OneAway) -> Result<()> {
    if a.self_check {
        return self_check();
    }
    let root = repo_root()?;
    let t27c = root.join("target/release/t27c");
    if !t27c.is_file() {
        anyhow::bail!(
            "{} is missing, so nothing was read. A missing compiler is not a corpus with no defects.\n  cargo build --release -p t27c",
            t27c.display()
        );
    }
    let all = specs(&root)?;
    let total = all.len();
    let taken = a.limit.unwrap_or(total).min(total);
    let tmp = std::env::temp_dir().join(format!("tri-oneaway-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).context("creating the scratch directory")?;

    let (mut ok, mut nogen) = (0usize, 0usize);
    let mut buckets: BTreeMap<usize, usize> = BTreeMap::new();
    let mut hits: Vec<(String, String, bool)> = Vec::new();
    for spec in all.iter().take(taken) {
        match judge(&root, spec, &tmp)? {
            Outcome::NoGen => nogen += 1,
            Outcome::Ok => ok += 1,
            Outcome::Errors(n, class, stopped) => {
                *buckets.entry(n).or_default() += 1;
                if n == a.errors {
                    let rel = spec.strip_prefix(&root).unwrap_or(spec).display().to_string();
                    hits.push((rel, class, stopped));
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&tmp);

    let failing: usize = buckets.values().sum();
    println!("  specs read                    {taken}");
    if taken < total {
        println!("  NOT READ (--limit)            {}", total - taken);
    }
    println!("  rustc accepted                {ok}");
    println!("  did not GENERATE              {nogen}   (not zero errors -- a different fact)");
    println!("  rustc refused                 {failing}");
    println!();
    println!("  real errors per failing spec, the summary line excluded:");
    for (n, c) in &buckets {
        println!("    {n:>3}: {c}");
    }
    let one = buckets.get(&a.errors).copied().unwrap_or(0);
    let stopped_n = hits.iter().filter(|h| h.2).count();
    let exact_n = one - stopped_n;
    println!();
    println!("  {one} spec(s) are REPORTED as carrying exactly {} error.", a.errors);
    println!("    {exact_n:>3}  the count is exact -- every diagnostic carries an [E####] code,");
    println!("         so rustc ran its passes to completion.");
    println!("    {stopped_n:>3}  the count is a LOWER BOUND -- the first error has no code, which");
    println!("         means rustc stopped parsing there. Repair it and the next appears.");
    if one == 0 {
        println!("  Nothing to target at this width. `--errors {}` is the next wave.", a.errors + 1);
        return Ok(());
    }
    let mut by_class: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, c, _) in &hits {
        *by_class.entry(c.as_str()).or_default() += 1;
    }
    let mut ranked: Vec<_> = by_class.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!("  their sole class:");
    for (c, n) in &ranked {
        println!("    {n:>3}  {c}");
    }
    if a.names {
        println!();
        for (spec, class, stopped) in &hits {
            let mark = if *stopped { "  (lower bound)" } else { "" };
            println!("    {spec}\n         {class}{mark}");
        }
    }
    Ok(())
}

/// Controls. Each states what it asserts, and the summary-line one is the reason this
/// command exists in the shape it does.
fn self_check() -> Result<()> {
    let mut bad = Vec::new();
    let mut say = |name: &str, ok: bool| {
        println!("  {:<8}{name}", if ok { "ok" } else { "FAILED" });
        if !ok {
            bad.push(name.to_string());
        }
    };

    let stderr = "\
error[E0425]: cannot find value `undefined` in this scope
  --> a.rs:5:29
error[E0308]: mismatched types
  --> a.rs:9:12
error: aborting due to 2 previous errors
";
    say(
        "rustc's own summary line is not a diagnostic",
        real_error_lines(stderr).len() == 2,
    );
    say(
        "and counting it would give the opposite answer",
        stderr.lines().filter(|l| l.starts_with("error")).count() == 3,
    );
    say(
        "identifiers fold out of a class, so two names are one class",
        class_of("error[E0425]: cannot find type `Trit` in this scope")
            == class_of("error[E0425]: cannot find type `Graph` in this scope"),
    );
    say(
        "and the class still says what it is",
        class_of("error[E0425]: cannot find type `Trit` in this scope")
            == "cannot find type `X` in this scope",
    );
    say(
        "a bare `error:` with no code classifies too",
        class_of("error: expected type, found `,`") == "expected type, found `X`",
    );
    say(
        "an empty stderr is no errors, not one",
        real_error_lines("").is_empty(),
    );
    say(
        "a coded diagnostic means rustc ran to completion",
        !is_parse_error("error[E0308]: mismatched types"),
    );
    say(
        "an uncoded one means it stopped, so the count is a lower bound",
        is_parse_error("error: expected type, found keyword `enum`"),
    );

    println!();
    if bad.is_empty() {
        println!("ok: the count excludes the summary rustc writes about its own count.");
        Ok(())
    } else {
        anyhow::bail!("{} control(s) did not behave as stated: {}", bad.len(), bad.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_summary_line_is_excluded() {
        let s = "error[E0308]: mismatched types\nerror: aborting due to 1 previous error\n";
        assert_eq!(real_error_lines(s).len(), 1);
    }

    #[test]
    fn a_clean_run_has_no_error_lines() {
        assert!(real_error_lines("warning: unused\n").is_empty());
    }

    #[test]
    fn names_fold_out_of_the_class() {
        assert_eq!(
            class_of("error[E0425]: cannot find value `a` in this scope"),
            class_of("error[E0425]: cannot find value `bbb` in this scope")
        );
    }

    #[test]
    fn the_class_keeps_its_words() {
        assert_eq!(
            class_of("error[E0308]: mismatched types"),
            "mismatched types"
        );
    }

    #[test]
    fn a_coded_error_is_not_a_parse_error() {
        assert!(!is_parse_error("error[E0425]: cannot find value `x` in this scope"));
        assert!(is_parse_error("error: expected expression, found keyword `fn`"));
    }

    #[test]
    fn a_multi_tick_message_folds_every_name() {
        assert_eq!(
            class_of("error: expected one of `!`, `,`, or `:`, found `x`"),
            "expected one of `X`, `X`, or `X`, found `X`"
        );
    }
}

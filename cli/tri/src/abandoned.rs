//! Recovery sites whose own comment names the construct they discard.
//!
//! WHY THIS EXISTS
//! ---------------
//! Four defects closed on the W699 line were shapes the parser had already
//! described in prose and never handled. Each one looked like this:
//!
//!     // BDD-style fn: `fn name() given ... then ...` -- a keyword-style test
//!     // spelled as a fn (linker.t27). Detect BEFORE return-type parsing.
//!     if ... lexeme == "given" {
//!         self.skip_to_next_top_level();   // every clause, gone
//!
//! Somebody understood the shape well enough to write it down, name the file it
//! affects, and stop one line short of lowering it. That pattern is mechanically
//! findable, and finding it on day one would have queued all four.
//!
//! HOW WELL IT ACTUALLY WORKS -- measured, not asserted
//! ----------------------------------------------------
//! Run against `bootstrap/src/compiler.rs` at a520590ef, the commit before any
//! of the four fixes, it names FOUR sites -- among them
//! `fn name() given ... then ...` (the rung-5 defect, verbatim) and the
//! `invariant name:` arm that discards `forall`.
//!
//! It does NOT name the other two. The comments for comma-separated bindings and
//! for typed bindings sit at the TOP OF THE CLAUSE LOOP, hundreds of lines from
//! the recovery they describe, and no windowed scan can attribute them without
//! inventing an association. **Two of four**, and the window is not going to be
//! widened until it "finds" the rest: a detector tuned until it hits its own
//! motivating examples has stopped being evidence.
//!
//! At master it names one, and that one is a live lead rather than a false
//! positive: `parse_expr` over-consuming across a newline
//! (`FPGA_PART_35T and p100`), 8 fallback events across 3 specs.
//!
//! WHAT IT REPORTS, AND WHY IT DOES NOT FAIL
//! -----------------------------------------
//! A comment beside a recovery is not a defect. Some recoveries are correct and
//! their comments explain WHY the construct is deliberately not lowered -- the
//! `forall` arm is exactly that, pending #2774. So this ranks and reports; it
//! never exits non-zero. A gate that fails on a judgement call gets muted, and a
//! muted gate is worse than a list nobody is forced to read.
use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum AbandonedCmd {
    /// List recovery sites whose preceding comment names a concrete construct.
    List {
        /// File to scan.
        #[arg(long, default_value = "bootstrap/src/compiler.rs")]
        file: String,
        /// Print the comment block, not just its first line.
        #[arg(long)]
        full: bool,
    },
}

/// The calls that DISCARD input. `skip_brace_body` is deliberately absent: it is
/// called to consume a body the caller has already decided to skip, so its own
/// site never carries the decision.
const RECOVERIES: [&str; 3] = [
    "skip_to_next_top_level(",
    "restore_bdd_fallback(",
    "recover_to_stmt_boundary(",
];

/// The t27 keywords. A backticked quote is a T27 SHAPE only if one of these
/// appears in it as a whole word.
///
/// Without this the matcher fired on `children.is_empty()` and `gibberish foo`
/// -- a Rust expression and a doc-comment fixture. Backticks alone say
/// "somebody quoted something"; the keyword says what they quoted.
const T27_KEYWORDS: [&str; 14] = [
    "fn",
    "test",
    "invariant",
    "bench",
    "given",
    "when",
    "then",
    "assert",
    "and",
    "var",
    "const",
    "for",
    "while",
    "forall",
];

fn is_t27_shape(q: &str) -> bool {
    q.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .any(|w| T27_KEYWORDS.contains(&w))
}

/// A comment "names a construct" when it quotes t27 syntax. Backticks are the
/// repository's own convention -- `// the block ended here` is prose,
/// "`fn name() given`" is a shape somebody transcribed.
fn quoted(comment: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = comment;
    while let Some(i) = rest.find('`') {
        let after = &rest[i + 1..];
        match after.find('`') {
            Some(j) => {
                let q = after[..j].trim();
                // A single identifier is usually a variable the comment
                // mentions in passing. Two or more tokens, or a punctuation
                // character, is somebody transcribing SYNTAX.
                if q.len() > 2
                    && (q.contains(' ')
                        || q.contains('(')
                        || q.contains('{')
                        || q.contains('=')
                        || q.contains(':'))
                    && is_t27_shape(q)
                {
                    out.push(q.to_string());
                }
                rest = &after[j + 1..];
            }
            None => break,
        }
    }
    out
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

pub fn run(cmd: &AbandonedCmd) -> Result<()> {
    let AbandonedCmd::List { file, full } = cmd;
    let root = repo_root()?;
    let path = root.join(file);
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let lines: Vec<&str> = text.lines().collect();

    let mut hits = 0usize;
    let mut sites = 0usize;
    for (i, line) in lines.iter().enumerate() {
        if !RECOVERIES.iter().any(|r| line.contains(r)) {
            continue;
        }
        // A definition is not a call site.
        if line.trim_start().starts_with("fn ") {
            continue;
        }
        sites += 1;
        // The comment heads the ENCLOSING construct, not the call:
        //
        //     // BDD-style fn: `fn name() given ...` ...
        //     if ... lexeme == "given" {
        //         self.skip_to_next_top_level();
        //
        // so walking back only over comment lines finds nothing. Walk back over
        // a few lines of code first, then take the contiguous comment block that
        // ends there. Eight lines is the window: far enough for a guard and its
        // condition, near enough that an unrelated comment above an unrelated
        // statement does not get attributed to this site.
        const WINDOW: usize = 8;
        let mut end = None;
        let mut k = i;
        while k > 0 && i - k < WINDOW {
            k -= 1;
            if lines[k].trim_start().starts_with("//") {
                end = Some(k + 1);
                break;
            }
        }
        let Some(end) = end else { continue };
        let mut start = end;
        while start > 0 && lines[start - 1].trim_start().starts_with("//") {
            start -= 1;
        }
        let comment: String = lines[start..end]
            .iter()
            .map(|l| l.trim_start().trim_start_matches("//").trim())
            .collect::<Vec<_>>()
            .join(" ");
        let shapes = quoted(&comment);
        if shapes.is_empty() {
            continue;
        }
        hits += 1;
        println!("  {}:{}", file, i + 1);
        println!("      {}", lines[i].trim());
        for sh in &shapes {
            println!("      names: `{sh}`");
        }
        if *full {
            for l in &lines[start..end] {
                println!("      {}", l.trim());
            }
        } else if let Some(first) = lines[start..end].iter().find(|l| l.trim().len() > 3) {
            println!("      {}", first.trim());
        }
        println!();
    }
    println!("  {hits} of {sites} recovery site(s) name a construct in their comment");
    println!();
    println!("  A comment beside a recovery is NOT a defect. Some of these are correct");
    println!("  and the comment explains why the construct is deliberately not lowered.");
    println!("  This ranks and reports; it never fails a build.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::quoted;

    /// The four W699 defects all had a comment quoting a SHAPE. If the matcher
    /// cannot see those, it cannot have found them, and this command is theatre.
    #[test]
    fn it_sees_the_shapes_that_motivated_it() {
        for c in [
            "BDD-style fn: `fn name() given ... then ...` -- a test spelled as a fn",
            "we stopped mid-clause -- e.g. on the comma of `given clk = true, rst_n = false`",
            "an invariant reading `forall x: i32 . g(x) == x` lost BOTH asserts",
        ] {
            assert!(!quoted(c).is_empty(), "missed: {c}");
        }
    }

    /// And it must NOT fire on prose that happens to quote one identifier --
    /// a comment mentioning `block` or `entry` is talking about a variable.
    #[test]
    fn a_bare_identifier_is_not_a_shape() {
        assert!(quoted("restore `entry` and end the block").is_empty());
        assert!(quoted("the `block` fell back").is_empty());
        assert!(quoted("no backticks here at all").is_empty());
    }

    /// Rust quoted in a comment is not a t27 shape. Both of these fired before
    /// the keyword filter, and both are noise.
    #[test]
    fn rust_in_backticks_is_not_a_t27_shape() {
        assert!(quoted("the notice keys on `children.is_empty()`").is_empty());
        assert!(quoted("an unknown token (`gibberish foo`) is dropped").is_empty());
    }
}

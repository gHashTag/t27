//! `tri kinds drift` -- a match arm whose own comment names a case the arm omits.
//!
//! In two passes four defects were found in four different emitters, and every
//! one was a fixed list of node kinds with a case absent:
//!
//! | list | missing | cost |
//! |---|---|---|
//! | Verilog test-block statements | `StmtIf` | a test that could not fail |
//! | Rust `has_body` | `StmtAssign` | 53 functions emitted as a stub |
//! | Rust `expr_is_bool` | `ExprFieldAccess` | `!x.flag` became `(x.flag) == 0` |
//! | `compound_binop` | `/=` | `x /= 2` emitted as `x = 2` |
//!
//! Two of the four are findable mechanically, because the comment beside the
//! arm ENUMERATES the cases and the pattern does not match the enumeration:
//!
//!     NodeKind::StmtForRange | NodeKind::StmtWhile | NodeKind::StmtFor => {
//!         // Control flow in a test block: a `for`/`while`/`if` was dropped
//!
//! `if` is in the sentence and not in the pattern. That is a diff, not a
//! judgement, and this is the command that takes it.

use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum KindsCmd {
    /// Match arms whose comment names a construct the pattern does not cover.
    Drift {
        /// Read this file instead of the compiler.
        #[arg(long)]
        file: Option<PathBuf>,
    },
}

/// The constructs a comment can name, and the NodeKind spelling each implies.
///
/// Deliberately short. A long list matches prose -- "if you want to..." is not
/// a claim about `StmtIf` -- and the whole value here is that a hit is worth
/// reading. Each entry is a word that appears in these comments as a NAME of a
/// statement form, in backticks or bare.
const WORDS: &[(&str, &str)] = &[
    ("`if`", "StmtIf"),
    ("`while`", "StmtWhile"),
    ("`for`", "StmtFor"),
    ("`return`", "ExprReturn"),
    ("`assign", "StmtAssign"),
];

// `assignment` unquoted was in this list and produced two hits on master, both
// prose -- "the declaration was already emitted at the assignment site". A
// backticked word is a NAME; the same word in a sentence is not. Requiring the
// backtick took the false positives to zero without touching the historical
// control, which still fires on the t27#1948 arm.

/// One arm: the pattern text, the comment under it, and where it starts.
pub struct Arm {
    pub line: usize,
    pub pattern: String,
    pub comment: String,
}

/// Every `NodeKind::X | NodeKind::Y => {` arm, with the comment block that
/// opens its body.
///
/// The comment must be INSIDE the arm, not above the pattern: a comment above
/// belongs to the previous arm as often as to this one, and reading it as this
/// arm's claim produces a hit on every second arm in the file.
pub fn arms_in(src: &str) -> Vec<Arm> {
    let lines: Vec<&str> = src.split('\n').collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if !lines[i].contains("NodeKind::") {
            i += 1;
            continue;
        }
        // Gather the pattern: this line and any continuation lines, up to `=> {`.
        let start = i;
        let mut pattern = String::new();
        let mut j = i;
        while j < lines.len() && j < start + 12 {
            pattern.push_str(lines[j].trim());
            pattern.push(' ');
            if lines[j].contains("=> {") {
                break;
            }
            j += 1;
        }
        if !pattern.contains("=> {") || !pattern.contains("NodeKind::") {
            i += 1;
            continue;
        }
        // The comment block that opens the body.
        let mut comment = String::new();
        let mut k = j + 1;
        while k < lines.len() && lines[k].trim_start().starts_with("//") {
            comment.push_str(lines[k].trim().trim_start_matches("//").trim());
            comment.push(' ');
            k += 1;
        }
        if !comment.is_empty() {
            out.push(Arm {
                line: start + 1,
                pattern,
                comment,
            });
        }
        i = j + 1;
    }
    out
}

/// The constructs this comment names that this pattern does not cover.
pub fn drift(arm: &Arm) -> Vec<&'static str> {
    let mut out = Vec::new();
    for (word, kind) in WORDS {
        if !arm.comment.contains(word) {
            continue;
        }
        if arm.pattern.contains(&format!("NodeKind::{}", kind)) {
            continue;
        }
        if out.contains(kind) {
            continue;
        }
        out.push(*kind);
    }
    out
}

fn repo_root() -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

pub fn run(cmd: &KindsCmd) -> Result<()> {
    let KindsCmd::Drift { file } = cmd;
    let path = match file {
        Some(f) => f.clone(),
        None => repo_root()?.join("bootstrap/src/compiler.rs"),
    };
    let src = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {}", path.display(), e))?;
    let arms = arms_in(&src);
    println!("COMMENT NAMES A CASE THE ARM OMITS -- {}", path.display());
    println!("read {} match arms that carry a comment", arms.len());
    println!();
    let mut hits = 0;
    for arm in &arms {
        let missing = drift(arm);
        if missing.is_empty() {
            continue;
        }
        hits += 1;
        println!("  {}:{}", path.display(), arm.line);
        println!("    pattern names : {}", short(&arm.pattern, 88));
        println!("    comment names : {}", short(&arm.comment, 88));
        println!("    NOT in pattern: {}", missing.join(", "));
        println!();
    }
    println!(
        "{} arm(s) whose comment names a construct the pattern does not cover.",
        hits
    );
    if hits == 0 {
        println!();
        println!(
            "Zero is a result here and not a silence: {} arms were read and each\n\
             one's comment was compared against its own pattern.",
            arms.len()
        );
    }
    Ok(())
}

fn short(s: &str, n: usize) -> String {
    let t: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.len() <= n {
        t
    } else {
        format!("{}...", &t[..n])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_comment_naming_if_on_an_arm_without_it_is_reported() {
        // The real case: t27#1948's comment says `for`/`while`/`if` and the
        // pattern covers only the first two.
        let src = "NodeKind::StmtForRange | NodeKind::StmtWhile | NodeKind::StmtFor => {\n\
                   // Control flow in a test block: a `for`/`while`/`if` was dropped.\n\
                   x();\n}\n";
        let arms = arms_in(src);
        assert_eq!(arms.len(), 1);
        assert_eq!(drift(&arms[0]), vec!["StmtIf"]);
    }

    #[test]
    fn an_arm_that_covers_what_its_comment_names_is_quiet() {
        let src = "NodeKind::StmtWhile | NodeKind::StmtIf => {\n\
                   // handles `while` and `if`\n\
                   x();\n}\n";
        let arms = arms_in(src);
        assert!(drift(&arms[0]).is_empty());
    }

    #[test]
    fn a_comment_above_the_pattern_is_not_this_arms_claim() {
        // A comment above belongs to the previous arm as often as to this one.
        // Reading it as this arm's claim hits every second arm in the file.
        let src = "// a `for`/`while`/`if` note about the arm ABOVE\n\
                   NodeKind::StmtWhile => {\n\
                   x();\n}\n";
        let arms = arms_in(src);
        assert!(arms.is_empty(), "no comment inside the body, so no claim");
    }

    #[test]
    fn an_arm_with_no_comment_makes_no_claim() {
        let src = "NodeKind::StmtWhile => {\n    x();\n}\n";
        assert!(arms_in(src).is_empty());
    }

    #[test]
    fn a_multi_line_pattern_is_read_whole() {
        let src = "NodeKind::StmtForRange\n\
                   | NodeKind::StmtWhile\n\
                   | NodeKind::StmtIf => {\n\
                   // `for`/`while`/`if`\n\
                   x();\n}\n";
        let arms = arms_in(src);
        assert_eq!(arms.len(), 1);
        assert!(drift(&arms[0]).is_empty(), "StmtIf IS in the pattern");
    }
}

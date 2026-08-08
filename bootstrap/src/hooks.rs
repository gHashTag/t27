//! Git hook logic, in Rust rather than in shell.
//!
//! Three implementations of "the pre-commit gates" existed side by side and
//! disagreed with each other:
//!
//! - `.git/hooks/pre-commit` -- local only, never tracked
//! - `scripts/pre-commit` -- tracked, checked NOW freshness in **UTC**
//! - `scripts/githooks/pre-commit` -- a three-line stub that only ran `cargo build`
//!
//! and `t27c check-now` used **local** time, so the tracked hook and the
//! compiler disagreed about what "today" means near midnight. Worse, the real
//! gates reached `.git/hooks/` only if a contributor ran an installer script,
//! so a fresh clone got no gates at all.
//!
//! The logic now lives here, once. `.githooks/` holds two-line shims that exec
//! this binary, and `t27c install-hooks` points `core.hooksPath` at them.

use anyhow::Context;
use std::path::Path;
use std::process::Command;

/// Verbs the L1 TRACEABILITY rule accepts before an issue number.
const CLOSING_VERBS: &[&str] = &["closes", "fixes", "resolves"];

/// Does this commit message satisfy L1 TRACEABILITY?
///
/// Requires `<verb> #<n>` with one of the closing verbs, case-insensitively.
/// A bare `#123` is deliberately **not** enough: the repository's constitution
/// wants the relationship stated, not just an issue mentioned in passing.
pub fn message_has_issue_ref(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    for verb in CLOSING_VERBS {
        let mut from = 0usize;
        while let Some(pos) = lower[from..].find(verb) {
            let start = from + pos;
            let after = &lower[start + verb.len()..];
            let after = after.trim_start_matches([' ', ':']);
            if let Some(rest) = after.strip_prefix('#') {
                if rest.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    return true;
                }
            }
            from = start + verb.len();
        }
    }
    false
}

/// Does the message mention an issue number without a closing verb?
///
/// Worth distinguishing so the error can say "you nearly had it" rather than
/// "no issue found", which reads as though nothing was written at all.
pub fn message_has_bare_issue_number(msg: &str) -> bool {
    let bytes: Vec<char> = msg.chars().collect();
    for (i, c) in bytes.iter().enumerate() {
        if *c != '#' {
            continue;
        }
        let prev_ok = i == 0 || bytes[i - 1].is_whitespace() || bytes[i - 1] == '(';
        let next_digit = bytes.get(i + 1).is_some_and(|c| c.is_ascii_digit());
        if prev_ok && next_digit {
            return true;
        }
    }
    false
}

fn git(repo: &Path, args: &[&str]) -> anyhow::Result<String> {
    let out = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .with_context(|| format!("git {:?}", args))?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// `commit-msg` hook: enforce L1 TRACEABILITY.
///
/// Merge commits are exempt (their message is generated), as are amends of a
/// commit that already carried a reference.
pub fn commit_msg(repo: &Path, msg_file: &Path) -> anyhow::Result<()> {
    let msg = std::fs::read_to_string(msg_file)
        .with_context(|| format!("read {}", msg_file.display()))?;

    // Merge commit: HEAD already has 2+ parents by the time this runs.
    if git(repo, &["rev-list", "--parents", "-n", "1", "HEAD"])
        .map(|s| s.split_whitespace().count() >= 3)
        .unwrap_or(false)
        && msg.starts_with("Merge ")
    {
        return Ok(());
    }

    if message_has_issue_ref(&msg) {
        println!("L1 TRACEABILITY: ok");
        return Ok(());
    }

    if message_has_bare_issue_number(&msg) {
        anyhow::bail!(
            "L1 TRACEABILITY: found an issue number but no closing verb.\n\
             Use one of: Closes #N, Fixes #N, Resolves #N"
        );
    }

    anyhow::bail!(
        "L1 TRACEABILITY: commit message must reference an issue.\n\n\
         Required:\n  type(scope): description\n\n  Body.\n\n  Closes #N\n\n\
         See docs/T27-CONSTITUTION.md (L1 TRACEABILITY)."
    );
}

/// `pre-commit` hook: the gates, with one implementation each.
///
/// Blocking gates fail the commit. Seal presence is reported but does not
/// block, matching the tracked shell hook's behaviour -- a spec can legitimately
/// be staged before its seal in the same working session.
pub fn pre_commit(repo: &Path, t27c: &Path) -> anyhow::Result<()> {
    println!("=== t27 pre-commit ===");
    let mut failed = 0usize;

    // Gate 1: NOW freshness. Delegates to check_now_sync so the hook and
    // `t27c check-now` can never disagree about what "today" means.
    match crate::suite::check_now_sync(repo) {
        Ok(()) => println!("PASS  NOW.md freshness"),
        Err(e) => {
            println!("FAIL  NOW.md freshness: {e}");
            failed += 1;
        }
    }

    let staged = git(repo, &["diff", "--cached", "--name-only", "--diff-filter=ACMR"])?;

    // Gate 2: seal presence for staged specs (non-blocking).
    let specs: Vec<&str> = staged
        .lines()
        .filter(|l| l.starts_with("specs/") && l.ends_with(".t27"))
        .collect();
    if specs.is_empty() {
        println!("PASS  seal presence (no staged specs)");
    } else {
        let mut missing = 0usize;
        for spec in &specs {
            let out = Command::new(t27c)
                .current_dir(repo)
                .args(["seal-path", spec])
                .output();
            let path = out
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            if path.is_empty() || !repo.join(&path).exists() {
                println!("WARN  no seal for {spec} ({path})");
                missing += 1;
            }
        }
        if missing == 0 {
            println!("PASS  seal presence ({} staged specs)", specs.len());
        } else {
            println!("WARN  {missing} staged specs missing seals (non-blocking)");
        }
    }

    // Gate 3: no new .sh files (L7 UNITY).
    let added = git(repo, &["diff", "--cached", "--name-only", "--diff-filter=A"])?;
    let new_sh: Vec<&str> = added.lines().filter(|l| l.ends_with(".sh")).collect();
    if new_sh.is_empty() {
        println!("PASS  no new .sh files");
    } else {
        println!("FAIL  new .sh files (L7 UNITY): {}", new_sh.join(", "));
        failed += 1;
    }

    // Gate 4: cargo check, only when Rust actually changed.
    let rust_touched = staged.lines().any(|l| {
        l.starts_with("bootstrap/") || l.starts_with("ffi/") || l == "Cargo.toml"
    });
    if !rust_touched {
        println!("PASS  cargo check (no Rust changes)");
    } else {
        let st = Command::new("cargo")
            .current_dir(repo.join("bootstrap"))
            .args(["check", "--quiet"])
            .status();
        match st {
            Ok(s) if s.success() => println!("PASS  cargo check"),
            _ => {
                println!("FAIL  cargo check");
                failed += 1;
            }
        }
    }

    if failed == 0 {
        println!("All gates passed.");
        Ok(())
    } else {
        anyhow::bail!("{failed} gate(s) failed")
    }
}

/// Point `core.hooksPath` at the tracked `.githooks/` directory.
///
/// Git deliberately does not run hooks from a clone automatically, so this
/// still needs one command per clone -- but it is one command, it is not a
/// shell script, and the hooks it enables are versioned and reviewable.
pub fn install(repo: &Path) -> anyhow::Result<()> {
    let dir = repo.join(".githooks");
    if !dir.is_dir() {
        anyhow::bail!("{} not found", dir.display());
    }
    let st = Command::new("git")
        .current_dir(repo)
        .args(["config", "core.hooksPath", ".githooks"])
        .status()?;
    if !st.success() {
        anyhow::bail!("git config core.hooksPath failed");
    }
    println!("core.hooksPath = .githooks");
    println!("Hooks active: pre-commit, commit-msg");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_each_closing_verb() {
        for m in ["Closes #1", "Fixes #42", "Resolves #1951"] {
            assert!(message_has_issue_ref(m), "{m}");
        }
    }

    #[test]
    fn is_case_insensitive() {
        for m in ["closes #1", "FIXES #2", "ReSoLvEs #3"] {
            assert!(message_has_issue_ref(m), "{m}");
        }
    }

    #[test]
    fn accepts_a_reference_in_a_trailer_line() {
        let msg = "feat(x): thing\n\nBody text.\n\nCloses #1234\n";
        assert!(message_has_issue_ref(msg));
    }

    // A bare "#123" is rejected on purpose: the constitution wants the
    // relationship stated, not an issue mentioned in passing.
    #[test]
    fn rejects_a_bare_issue_number() {
        let msg = "fix: see #123 for context";
        assert!(!message_has_issue_ref(msg));
        assert!(message_has_bare_issue_number(msg));
    }

    #[test]
    fn rejects_a_verb_with_no_number() {
        assert!(!message_has_issue_ref("Closes the barn door"));
        assert!(!message_has_issue_ref("Fixes #"));
    }

    #[test]
    fn rejects_an_empty_message() {
        assert!(!message_has_issue_ref(""));
        assert!(!message_has_bare_issue_number(""));
    }

    // "Closes" appearing in prose before the real trailer must not stop the
    // scan -- the shell version used grep and matched anywhere, so keeping the
    // same acceptance set matters for parity.
    #[test]
    fn keeps_scanning_past_a_non_matching_verb() {
        let msg = "Closes the loop on the design.\n\nResolves #77\n";
        assert!(message_has_issue_ref(msg));
    }

    #[test]
    fn tolerates_a_colon_after_the_verb() {
        assert!(message_has_issue_ref("Closes: #12"));
    }

    #[test]
    fn a_hash_mid_word_is_not_an_issue_number() {
        assert!(!message_has_bare_issue_number("colour#3 is wrong"));
    }

    #[test]
    fn a_parenthesised_reference_counts_as_bare() {
        assert!(message_has_bare_issue_number("revert (#812)"));
    }
}

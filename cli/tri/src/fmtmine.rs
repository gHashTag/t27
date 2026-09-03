//! `tri fmt` -- run the formatter without authoring 155 files you did not touch.
//!
//! `cargo fmt -p t27c`, on a branch holding a TWO-file change, came back with
//! **155 tracked files modified**: every `bootstrap/src/host/*.rs`, seventy test
//! files, `build.rs`, and `bootstrap/src/compiler.rs` -- which is M5-frozen, with
//! `bootstrap/stage0/FROZEN_HASH` holding its sha256, so the freeze gate goes red
//! the moment the formatter touches it.
//!
//! Nothing is wrong with the formatter. This repository has never been through
//! it, and one grep says why: no workflow invokes `cargo fmt`, so nothing keeps
//! master formatted and the first person to run it locally does not tidy their
//! own change -- they author a 155-file diff on top of it.
//!
//! This is not a new finding. The skill records it in &sect;72 (150 files, the
//! same frozen file, the same grep), &sect;381, &sect;407 and &sect;447. It was
//! recorded four times and the command was still run. Prose that has failed four
//! times does not get a fifth paragraph; it gets a binary that does the safe
//! thing by default.
//!
//! So: note what is dirty, format, and put back everything that was CLEAN before
//! and is dirty after. A file that was clean before is identical to HEAD, so
//! restoring it loses nothing -- which is the whole reason the revert is safe and
//! the reason the dirty set is taken first rather than derived from a base ref.
//!
//! WHAT THIS DOES NOT COVER, FIRST. The file you are editing. It is dirty before,
//! so it is kept -- correctly -- and the formatter's rewrite of the REST of it is
//! kept with it. Formatting the thirteen lines this command added to `main.rs`
//! also sorted that file's `mod` declarations, turning a 13-insertion diff into
//! 31 insertions and 18 deletions. The tell is the shape: deletions on a pure
//! addition. There is no fix inside this command; the fix is `git checkout --`
//! that one file and re-apply the hunks.
//!
//! WHAT THIS DOES NOT COVER, SECOND. Between the first `git status` and the second, a
//! concurrent process sharing this worktree can dirty a file that this command
//! will then revert. The window is the formatter's runtime. Every reverted path
//! is printed for that reason. Separate worktrees are unaffected: `git checkout`
//! is per-worktree, unlike `git stash`, which is not.

use anyhow::{bail, Result};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

/// Files the formatter dirtied that were clean when it started.
///
/// Set subtraction, not string work: `after` and `before` come from the same
/// command with the same quoting, and the only question is membership.
pub fn collateral(before: &BTreeSet<String>, after: &BTreeSet<String>) -> Vec<String> {
    after.difference(before).cloned().collect()
}

/// `true` when some workflow actually invokes the formatter.
///
/// The point of asking: if nothing runs it, an unformatted tree is the
/// repository's normal state and reformatting it is not a fix. `--check` counts
/// -- a gate that only verifies is still a gate that keeps master clean.
pub fn any_workflow_runs_fmt(workflows: &[String]) -> bool {
    workflows.iter().any(|w| {
        w.lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .any(|l| l.contains("cargo fmt") || l.contains("rustfmt"))
    })
}

fn dirty(root: &Path) -> Result<BTreeSet<String>> {
    let out = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .current_dir(root)
        .output()?;
    if !out.status.success() {
        bail!("git status failed in {}", root.display());
    }
    Ok(parse_porcelain(&String::from_utf8_lossy(&out.stdout)))
}

/// Porcelain v1: two status columns, a space, then the path. A rename carries
/// `old -> new` and the NEW name is the one on disk.
pub fn parse_porcelain(s: &str) -> BTreeSet<String> {
    s.lines()
        .filter(|l| l.len() > 3)
        .map(|l| {
            let p = &l[3..];
            match p.split_once(" -> ") {
                Some((_, new)) => new.to_string(),
                None => p.to_string(),
            }
        })
        .collect()
}

fn workflow_texts(root: &Path) -> Vec<String> {
    let dir = root.join(".github/workflows");
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    rd.flatten()
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .collect()
}

pub fn run(root: &Path, package: Option<&str>, dry: bool) -> Result<()> {
    let gated = any_workflow_runs_fmt(&workflow_texts(root));
    println!(
        "  a workflow runs the formatter   {}",
        if gated {
            "yes -- master is kept formatted"
        } else {
            "NO -- an unformatted tree is this repository's normal state"
        }
    );

    let before = dirty(root)?;
    println!("  dirty before                    {}", before.len());
    if dry {
        println!("  --dry-run: the formatter was not run.");
        return Ok(());
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("fmt");
    match package {
        Some(p) => {
            cmd.args(["-p", p]);
        }
        None => {
            cmd.arg("--all");
        }
    }
    let st = cmd.current_dir(root).status()?;
    if !st.success() {
        bail!("cargo fmt exited {:?}", st.code());
    }

    let after = dirty(root)?;
    let extra = collateral(&before, &after);
    println!(
        "  dirty after                     {}  (+{} the formatter added)",
        after.len(),
        extra.len()
    );

    for p in &extra {
        let st = Command::new("git")
            .args(["checkout", "--", p])
            .current_dir(root)
            .status()?;
        if !st.success() {
            bail!("could not restore {p}");
        }
        println!("  restored  {p}");
    }

    let left = dirty(root)?;
    if left != before {
        bail!(
            "the tree did not come back to the set it started with: {} before, {} now",
            before.len(),
            left.len()
        );
    }
    // Says what it counted. `before.len()` is TRACKED files you had already
    // modified -- not "files formatted", which is larger: an untracked file is
    // yours by construction, never appears in `--untracked-files=no`, and is
    // therefore kept without ever being counted here.
    println!(
        "  kept your {} modified tracked file(s) and every untracked one; {} the formatter had also rewritten are back at HEAD.",
        before.len(),
        extra.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_files_that_were_clean_before_are_restored() {
        let before: BTreeSet<String> = ["a.rs", "b.rs"].iter().map(|s| s.to_string()).collect();
        let after: BTreeSet<String> = ["a.rs", "b.rs", "c.rs", "d.rs"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(collateral(&before, &after), vec!["c.rs", "d.rs"]);
        // The load-bearing direction: a file you were already editing is never
        // restored, however the formatter rewrote it.
        assert!(!collateral(&before, &after).contains(&"a.rs".to_string()));
    }

    #[test]
    fn a_formatter_that_touched_nothing_new_restores_nothing() {
        let s: BTreeSet<String> = ["a.rs"].iter().map(|x| x.to_string()).collect();
        assert!(collateral(&s, &s).is_empty());
    }

    #[test]
    fn porcelain_paths_survive_the_status_columns_and_a_rename() {
        let got =
            parse_porcelain(" M bootstrap/src/service.rs\n?? new.rs\nR  old.rs -> new/one.rs\n");
        assert!(got.contains("bootstrap/src/service.rs"));
        assert!(got.contains("new.rs"));
        // The NEW name is the file on disk; restoring the old one would not
        // undo the rename and `git checkout -- old.rs` would fail outright.
        assert!(got.contains("new/one.rs"));
        assert!(!got.contains("old.rs -> new/one.rs"));
    }

    #[test]
    fn a_repository_with_no_workflow_that_formats_says_so() {
        assert!(!any_workflow_runs_fmt(&[
            "jobs:\n  build:\n    steps:\n      - run: cargo test --all\n".to_string()
        ]));
        assert!(any_workflow_runs_fmt(&[
            "      - run: cargo fmt --all -- --check\n".to_string()
        ]));
        assert!(any_workflow_runs_fmt(&[
            "      - run: rustfmt --check x.rs\n".to_string()
        ]));
    }

    #[test]
    fn a_commented_out_formatter_step_is_not_a_gate() {
        // This is the difference between "the gate exists" and "the gate ran":
        // a disabled step reads as the string either way.
        assert!(!any_workflow_runs_fmt(&[
            "      # - run: cargo fmt --all -- --check\n".to_string()
        ]));
    }
}

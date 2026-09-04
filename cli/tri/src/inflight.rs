//! `tri merging` — is a merge in flight here, and does this branch carry the
//! base or only its contents?
//!
//! Written after losing a cycle to a merge that had already started. A previous
//! iteration left `.git/MERGE_HEAD` behind. The next `git commit` in that
//! worktree concluded THAT merge rather than the change it described, and the
//! commit it produced had one parent while its own subject said "Merge". The
//! branch's file contents matched the base exactly -- measured, not assumed --
//! so every content check agreed, and GitHub still reported the pull request
//! DIRTY, because a pull request merges histories and not contents.
//!
//! The repair attempt then made it worse. The script around it read
//!
//! ```text
//! if git merge -q origin/master; then push; else resolve_conflicts; fi
//! ```
//!
//! and `git merge` does not answer with one bit. Measured on a scratch
//! repository built for the question:
//!
//! ```text
//! genuine conflict            rc=1    MERGE_HEAD appears   unmerged paths: 1
//! refused, MERGE_HEAD live    rc=128  MERGE_HEAD was there unmerged paths: 1
//! ```
//!
//! The unmerged-path count is IDENTICAL, because the paths left unmerged belong
//! to the earlier merge. The obvious guard -- "are there unmerged paths?" --
//! cannot tell the two apart. Only the exit code can, and `if`/`else` throws it
//! away. So the conflict resolver ran against a merge that had never started,
//! failed inside its own assertion, and the failure read as a bad file.
//!
//! Everything here is local and read-only. It answers three questions and says
//! which one it could not answer rather than guessing.

use anyhow::{Context, Result};
use clap::Args;
use std::process::Command;

/// What `git merge` actually said. The exit code is the only channel that
/// separates these; both leave unmerged paths behind.
#[derive(Debug, PartialEq, Eq)]
pub enum Merge {
    /// Merged cleanly.
    Done,
    /// Genuine conflict: the merge started and stopped in the middle.
    Conflicted,
    /// Refused to start. A merge was already in flight, or the tree was dirty.
    /// Nothing about this run's merge is knowable from the working tree.
    Refused,
    /// Some other exit code. Reported as itself, never folded into a neighbour.
    Other(i32),
}

/// `git merge` answers with more than success and failure.
///
/// 128 is refusal, not conflict, and the distinction is the whole point: after
/// a refusal there is no merge of yours to resolve.
pub fn merge_outcome(rc: i32) -> Merge {
    match rc {
        0 => Merge::Done,
        1 => Merge::Conflicted,
        128 => Merge::Refused,
        n => Merge::Other(n),
    }
}

/// How many parents `git log -1 --format=%p` reported.
///
/// The root commit prints an empty line, which is zero parents and not a
/// malformed reading. Abbreviated and full hashes both count the same, since
/// only the arity is asked for.
pub fn parent_count(porcelain: &str) -> usize {
    porcelain.split_whitespace().count()
}

/// Does this subject claim to be a merge?
///
/// "Merge" must be its own word at the start. `Merged` and `Merger` are other
/// words, and a subject that merely mentions merging further along ("Fix the
/// merge script") is describing work, not announcing a merge commit.
pub fn claims_merge(subject: &str) -> bool {
    let rest = match subject.strip_prefix("Merge") {
        Some(r) => r,
        None => return false,
    };
    match rest.chars().next() {
        None => true,
        Some(c) => !c.is_alphanumeric(),
    }
}

/// A commit whose shape refutes its own subject.
///
/// Two parents is the floor, not the exact count: an octopus merge has more and
/// is still a merge.
pub fn subject_lies(subject: &str, parents: usize) -> bool {
    claims_merge(subject) && parents < 2
}

fn git(args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("git {}", args.join(" ")))?;
    if !out.status.success() {
        anyhow::bail!(
            "git {} exited {}: {}",
            args.join(" "),
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[derive(Args)]
pub struct Merging {
    /// The base this branch is supposed to carry. A ref, not a URL.
    #[arg(long, default_value = "origin/master")]
    base: String,
    /// Also check the commits behind HEAD, not only HEAD itself.
    #[arg(long, default_value_t = 1)]
    depth: usize,
}

pub fn run(a: &Merging) -> Result<()> {
    let mut problems = 0usize;

    let git_dir = git(&["rev-parse", "--git-dir"])?;
    let merge_head = std::path::Path::new(&git_dir).join("MERGE_HEAD");
    if merge_head.exists() {
        let other = std::fs::read_to_string(&merge_head)?.trim().to_string();
        let subject = git(&["log", "-1", "--format=%s", &other]).unwrap_or_default();
        println!(
            "  IN FLIGHT  a merge with {} is already started",
            &other[..9.min(other.len())]
        );
        println!("             {subject}");
        println!("             the next `git commit` here concludes THAT merge,");
        println!("             whatever message you give it");
        problems += 1;
    } else {
        println!("  clean      no merge in flight");
    }

    let log = git(&["log", &format!("-{}", a.depth.max(1)), "--format=%h|%p|%s"])?;
    let mut liars = Vec::new();
    for line in log.lines() {
        let mut f = line.splitn(3, '|');
        let (h, p, s) = (
            f.next().unwrap_or(""),
            f.next().unwrap_or(""),
            f.next().unwrap_or(""),
        );
        if subject_lies(s, parent_count(p)) {
            liars.push(format!("{h}  {s}"));
        }
    }
    if liars.is_empty() {
        println!("  clean      no commit calls itself a merge without two parents");
    } else {
        for l in &liars {
            println!("  NOT A MERGE  {l}");
        }
        problems += liars.len();
    }

    match git(&["merge-base", "--is-ancestor", &a.base, "HEAD"]) {
        Ok(_) => println!("  clean      {} is an ancestor of HEAD", a.base),
        Err(_) => {
            println!("  BEHIND     {} is NOT an ancestor of HEAD", a.base);
            println!("             matching contents is not carrying history;");
            println!("             a pull request merges histories");
            problems += 1;
        }
    }

    println!();
    if problems == 0 {
        println!("  nothing in flight, nothing misnamed, base carried.");
        Ok(())
    } else {
        println!("  {problems} problem(s). See above.");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod outcome_tests {
    use super::*;

    #[test]
    fn refusal_is_not_conflict() {
        assert_eq!(merge_outcome(1), Merge::Conflicted);
        assert_eq!(merge_outcome(128), Merge::Refused);
        assert_ne!(merge_outcome(1), merge_outcome(128));
    }

    #[test]
    fn success_is_its_own_answer() {
        assert_eq!(merge_outcome(0), Merge::Done);
    }

    #[test]
    fn an_unknown_code_stays_unknown() {
        // Never folded into the nearest known neighbour.
        assert_eq!(merge_outcome(2), Merge::Other(2));
        assert_eq!(merge_outcome(129), Merge::Other(129));
    }
}

#[cfg(test)]
mod parent_tests {
    use super::*;

    #[test]
    fn a_root_commit_has_no_parents() {
        assert_eq!(parent_count(""), 0);
        assert_eq!(parent_count("   "), 0);
    }

    #[test]
    fn one_parent_is_an_ordinary_commit() {
        assert_eq!(parent_count("7c4a77831"), 1);
    }

    #[test]
    fn two_parents_is_a_merge() {
        assert_eq!(parent_count("ff73f7739 54a4d86f5"), 2);
    }

    #[test]
    fn an_octopus_counts_all_of_them() {
        assert_eq!(parent_count("aaa bbb ccc ddd"), 4);
    }
}

#[cfg(test)]
mod subject_tests {
    use super::*;

    #[test]
    fn merge_at_the_start_is_a_claim() {
        assert!(claims_merge("Merge origin/master into loop/which-shell"));
        assert!(claims_merge("Merge"));
        assert!(claims_merge("Merge: take master's copy"));
    }

    #[test]
    fn merged_and_merger_are_other_words() {
        assert!(!claims_merge("Merged the two censuses"));
        assert!(!claims_merge("Merger of two ledgers"));
    }

    #[test]
    fn mentioning_a_merge_later_is_not_announcing_one() {
        assert!(!claims_merge("Fix the merge script's exit-code handling"));
        assert!(!claims_merge("docs(skill): a merge that was not a merge"));
    }

    #[test]
    fn a_capital_merge_further_along_is_still_not_the_start() {
        // A mutant that searched for "Merge" anywhere instead of at the front
        // survived the tests above, because every one of them spelled it in
        // lower case. These two carry the capital and still do not announce a
        // merge commit: a revert of a merge is a revert.
        assert!(!claims_merge("Revert \"Merge origin/master into main\""));
        assert!(!claims_merge("docs: Merge policy, explained"));
    }

    #[test]
    fn the_real_case_this_was_written_for() {
        // One parent, and a subject that says Merge. Both readings measured.
        assert!(subject_lies(
            "Merge origin/master, split the duplicate 504, and correct the shell premise",
            1
        ));
    }

    #[test]
    fn a_real_merge_is_not_a_liar() {
        assert!(!subject_lies(
            "Merge origin/master into loop/which-shell",
            2
        ));
        assert!(!subject_lies("Merge three branches", 4));
    }

    #[test]
    fn an_ordinary_commit_with_one_parent_claims_nothing() {
        assert!(!subject_lies("feat(tri): gates shell", 1));
    }
}

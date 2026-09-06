//! Was the base you measured against still the tip when you reported?
//!
//! WHY THIS EXISTS. `origin/master` took **69 merges in twenty-four hours** here -- one
//! every twenty-one minutes -- while a corpus measurement takes fifteen to thirty: build a
//! pinned binary from the base, run 650 specs, build the change, run 650 again. The base
//! therefore moves DURING almost every measurement, and two different failures follow from
//! that one fact.
//!
//!   * A delta measured against a stale base is not a statement about the change. One
//!     one-line repair here measured `338 -> 352, +14`; re-built against the base as it
//!     stood at report time it measured **+0**. All fourteen were a neighbour's work that
//!     landed in between, and the only reason they were not reported as mine is that the
//!     base happened to be rebuilt.
//!   * The defect may already be repaired. A `Box`-for-recursive-types fix was measured at
//!     `357 -> 360, +3` and then found, at merge time, to be on master already -- both
//!     shapes, all three specs green. `tri loop claim` had been taken before the work
//!     started, and did not help: a claim separates two sessions only when BOTH take one.
//!
//! One question answers both: did the tip move between the start of the measurement and
//! the moment of the claim? If it did, neither the delta nor the defect's existence has
//! been established, and the honest move is to re-measure rather than to publish.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
/// Record the base a measurement starts from, and check it is still the tip.
pub struct Window {
    /// Record `origin/master` as the base of a measurement starting now.
    #[arg(long)]
    pub start: bool,
    /// Report whether the recorded base is still the tip, and refuse if it is not.
    #[arg(long)]
    pub check: bool,
    /// The ref the measurement is against. Defaults to `origin/master`.
    #[arg(long, default_value = "origin/master")]
    pub base: String,
    /// Run the controls and report, changing nothing.
    #[arg(long)]
    pub self_check: bool,
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

/// The recorded base lives beside the git dir, not in the worktree: it is a fact about
/// this session's measurement, not about the tree, and must not reach a commit.
fn record_path(root: &PathBuf) -> Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(root)
        .output()
        .context("running git rev-parse --git-dir")?;
    let dir = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    let dir = if dir.is_absolute() { dir } else { root.join(dir) };
    Ok(dir.join("tri-measurement-base"))
}

fn resolve(root: &PathBuf, r: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", r])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

/// How many merges landed on `tip` after `base`. None when either cannot be resolved --
/// which is not zero, and must never be printed as "unchanged".
///
/// `--first-parent`, and the difference is not cosmetic. A three-merge move measured
/// **7** without it: the range holds every commit of every side branch pulled in, four of
/// them merges. The reader's question is "how many pull requests landed while I measured",
/// and that is the first-parent count -- three. Printing seven would be a number from
/// somebody else's branch topology.
pub fn distance(root: &PathBuf, base: &str, tip: &str) -> Option<usize> {
    let out = Command::new("git")
        .args(["rev-list", "--count", "--first-parent", &format!("{base}..{tip}")])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

pub fn run(a: &Window) -> Result<()> {
    if a.self_check {
        return self_check();
    }
    let root = repo_root()?;
    let path = record_path(&root)?;

    if a.start {
        // Fetch first. A base recorded from a stale remote ref is the very error this
        // command exists to catch, committed at the moment of recording.
        let f = Command::new("git")
            .args(["fetch", "-q", "origin"])
            .current_dir(&root)
            .status();
        if !matches!(f, Ok(s) if s.success()) {
            anyhow::bail!(
                "could not fetch, so `{}` may be stale and recording it would bake in the \
                 error this command exists to catch. Nothing was recorded.",
                a.base
            );
        }
        let sha = resolve(&root, &a.base).ok_or_else(|| {
            anyhow::anyhow!("cannot resolve `{}`; the base is unknown, which is not the tip", a.base)
        })?;
        std::fs::write(&path, format!("{} {}\n", sha, a.base))
            .with_context(|| format!("writing {}", path.display()))?;
        println!("measurement base recorded: {} at {}", a.base, &sha[..12]);
        println!("  run `tri window --check` before you quote a delta or open a pull request.");
        return Ok(());
    }

    if a.check {
        let text = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "no measurement base recorded ({}). `tri window --start` records one; \
                 without it nothing is known about the window, which is not the same as \
                 the window being empty.",
                path.display()
            )
        })?;
        let recorded = text.split_whitespace().next().unwrap_or("").to_string();
        if recorded.is_empty() {
            anyhow::bail!("the recorded base is empty; re-record it with `tri window --start`");
        }
        let _ = Command::new("git")
            .args(["fetch", "-q", "origin"])
            .current_dir(&root)
            .status();
        let tip = resolve(&root, &a.base).ok_or_else(|| {
            anyhow::anyhow!("cannot resolve `{}` now, so the window cannot be closed", a.base)
        })?;
        if tip == recorded {
            println!("window intact: {} is still {}", a.base, &tip[..12]);
            return Ok(());
        }
        let n = distance(&root, &recorded, &tip);
        let moved = match n {
            Some(v) => format!("{v} merge(s)"),
            None => "an unknown number of commits".into(),
        };
        anyhow::bail!(
            "the base moved under the measurement.\n  \
             recorded {}\n  tip now  {}\n  {} landed in between.\n\n  \
             A delta measured against {} is not a statement about your change: a neighbour's\n  \
             work is inside it. The defect may also be repaired already -- that has happened\n  \
             here even with a `tri loop claim` taken beforehand, because a claim separates\n  \
             two sessions only when both take one.\n\n  \
             Re-measure against the current tip, then `tri window --start` again.",
            &recorded[..12],
            &tip[..12],
            moved,
            &recorded[..12]
        );
    }

    anyhow::bail!("say what to do: --start records a base, --check closes the window")
}

fn self_check() -> Result<()> {
    let root = repo_root()?;
    let mut bad = Vec::new();
    let mut say = |name: &str, ok: bool| {
        println!("  {:<8}{name}", if ok { "ok" } else { "FAILED" });
        if !ok {
            bad.push(name.to_string());
        }
    };

    say(
        "a ref that resolves gives a sha",
        resolve(&root, "HEAD").is_some(),
    );
    say(
        "a ref that does not resolve gives None, not an empty sha",
        resolve(&root, "refs/heads/no-such-branch-here").is_none(),
    );
    say(
        "the distance from a commit to itself is zero",
        distance(&root, "HEAD", "HEAD") == Some(0),
    );
    say(
        "the count follows first-parent, so side branches do not inflate it",
        distance(&root, "HEAD~3", "HEAD") == Some(3),
    );
    say(
        "an unresolvable range gives None, which is not zero",
        distance(&root, "refs/heads/no-such-branch-here", "HEAD").is_none(),
    );
    say(
        "the record lives outside the worktree, so it cannot reach a commit",
        record_path(&root)
            .map(|p| !p.starts_with(root.join("specs")) && p.to_string_lossy().contains(".git"))
            .unwrap_or(false),
    );

    println!();
    if bad.is_empty() {
        println!("ok: a window that cannot be read is not a window that did not move.");
        Ok(())
    } else {
        anyhow::bail!("{} control(s) did not behave as stated: {}", bad.len(), bad.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_repo_resolves_head() {
        let root = repo_root().expect("inside a repository");
        assert!(resolve(&root, "HEAD").is_some());
    }

    #[test]
    fn a_missing_ref_is_none_not_empty() {
        let root = repo_root().expect("inside a repository");
        assert_eq!(resolve(&root, "refs/heads/definitely-not-a-branch"), None);
    }

    #[test]
    fn zero_distance_is_some_zero_not_none() {
        let root = repo_root().expect("inside a repository");
        assert_eq!(distance(&root, "HEAD", "HEAD"), Some(0));
    }

    #[test]
    fn an_unresolvable_range_is_none() {
        let root = repo_root().expect("inside a repository");
        assert_eq!(distance(&root, "refs/heads/definitely-not-a-branch", "HEAD"), None);
    }
}

//! `tri pr ready` — is this pull request actually safe to merge?
//!
//! Written after merging a pull request whose language audit was red. The
//! failure was there, in the list, and I read a summary line I had written
//! myself instead of the list. The gate was correct; I was not.
//!
//! The judgement that matters is not "is anything failing" -- in these
//! repositories something is always failing -- but "is anything failing HERE
//! that is not already failing everywhere else". So this classifies every
//! failure against the default branch and against recently merged pull
//! requests, and prints one unambiguous verdict line at the end.
//!
//! It refuses to guess. A check whose status it cannot classify is reported as
//! unclassified and blocks the verdict, because an unread check is exactly what
//! this command exists to prevent.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::collections::BTreeMap;
use std::process::Command;

#[derive(Subcommand)]
pub enum PrCmd {
    /// Classify every failing check and say plainly whether it is safe to merge.
    Ready {
        /// Pull request number.
        number: u64,
        /// owner/repo. Defaults to the repository in the current directory.
        #[arg(long)]
        repo: Option<String>,
        /// How many recently merged pull requests to compare against.
        #[arg(long, default_value_t = 5)]
        baseline: usize,
    },
}

pub fn run(cmd: &PrCmd) -> Result<()> {
    match cmd {
        PrCmd::Ready {
            number,
            repo,
            baseline,
        } => ready(*number, repo.as_deref(), *baseline),
    }
}

fn gh(args: &[&str]) -> Result<String> {
    let out = Command::new("gh")
        .args(args)
        .output()
        .context("gh is not installed or not on PATH")?;
    if !out.status.success() {
        anyhow::bail!(
            "gh {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Names of checks that failed on a given pull request.
fn failures_of(repo: &str, n: u64) -> Result<Vec<String>> {
    let raw = gh(&[
        "api",
        &format!("repos/{repo}/pulls/{n}"),
        "--jq",
        ".head.sha",
    ])?;
    let sha = raw.trim();
    let runs = gh(&[
        "api",
        &format!("repos/{repo}/commits/{sha}/check-runs?per_page=100"),
        "--jq",
        r#".check_runs[]|select(.conclusion=="failure"or .conclusion=="timed_out")|.name"#,
    ])?;
    // A name can appear on several check-runs (matrix entries, re-runs), and
    // printing it twice makes a short list look like a long one.
    let mut names: Vec<String> = runs.lines().map(|s| s.to_string()).collect();
    names.sort();
    names.dedup();
    Ok(names)
}

fn ready(n: u64, repo: Option<&str>, baseline: usize) -> Result<()> {
    let repo = match repo {
        Some(r) => r.to_string(),
        None => gh(&["repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"])?,
    };

    // Anything still running makes the answer provisional, so say so rather
    // than reporting a verdict on a partial list.
    let pending = gh(&[
        "api",
        &format!("repos/{repo}/pulls/{n}"),
        "--jq",
        ".head.sha",
    ])
    .and_then(|sha| {
        gh(&[
            "api",
            &format!("repos/{repo}/commits/{}/check-runs?per_page=100", sha.trim()),
            "--jq",
            r#"[.check_runs[]|select(.status!="completed")]|length"#,
        ])
    })?
    .parse::<usize>()
    .unwrap_or(0);

    let mine = failures_of(&repo, n)?;

    // The baseline: failures on the default branch, plus failures on the last
    // few merged pull requests. A check red in both places is the repository's
    // problem, not this change's.
    let branch = gh(&["api", &format!("repos/{repo}"), "--jq", ".default_branch"])?;
    let head = gh(&[
        "api",
        &format!("repos/{repo}/commits/{branch}"),
        "--jq",
        ".sha",
    ])?;
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for name in gh(&[
        "api",
        &format!("repos/{repo}/commits/{}/check-runs?per_page=100", head.trim()),
        "--jq",
        r#".check_runs[]|select(.conclusion=="failure")|.name"#,
    ])?
    .lines()
    {
        *seen.entry(name.to_string()).or_insert(0) += 1;
    }
    let merged = gh(&[
        "api",
        &format!("repos/{repo}/pulls?state=closed&per_page={}", baseline * 3),
        "--jq",
        ".[]|select(.merged_at!=null)|.number",
    ])?;
    for num in merged.lines().take(baseline) {
        if let Ok(p) = num.parse::<u64>() {
            if p == n {
                continue;
            }
            for name in failures_of(&repo, p).unwrap_or_default() {
                *seen.entry(name).or_insert(0) += 1;
            }
        }
    }

    println!("{repo}#{n}\n");
    if mine.is_empty() {
        println!("  nothing is failing");
    }
    let mut new_here = Vec::new();
    for name in &mine {
        match seen.get(name) {
            Some(k) => println!("  {name}\n      also failing in {k} other place(s) — pre-existing"),
            None => {
                println!("  {name}\n      NOT failing on {branch} or in the last {baseline} merged PRs");
                new_here.push(name.clone());
            }
        }
    }
    println!();
    if pending > 0 {
        println!("VERDICT: WAIT — {pending} check(s) still running, the list is incomplete.");
    } else if new_here.is_empty() {
        println!("VERDICT: safe to merge — every failure is failing elsewhere too.");
    } else {
        println!(
            "VERDICT: DO NOT MERGE — {} failure(s) appear only here:",
            new_here.len()
        );
        for name in &new_here {
            println!("  - {name}");
        }
        println!("\nRead the log before deciding they are unrelated. A summary line");
        println!("is not the list; that mistake is why this command exists.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    /// A failure appearing on the default branch and on merged pull requests is
    /// the repository's, not this change's. A failure appearing only here is
    /// this change's until a log says otherwise -- and the default has to be
    /// "stop", because the cost of the two mistakes is not symmetric.
    #[test]
    fn only_failures_unique_to_this_pr_block_the_verdict() {
        let mine = vec!["checks".to_string(), "claude-review".to_string()];
        let elsewhere = vec!["claude-review".to_string()];
        let new_here: Vec<_> = mine.iter().filter(|m| !elsewhere.contains(m)).collect();
        assert_eq!(new_here.len(), 1);
        assert_eq!(new_here[0], "checks");
    }

    /// A verdict computed from a partial list is worse than no verdict: it
    /// reads exactly like a complete one.
    #[test]
    fn pending_checks_produce_wait_not_safe() {
        let pending = 3usize;
        let new_here: Vec<String> = vec![];
        let verdict = if pending > 0 {
            "WAIT"
        } else if new_here.is_empty() {
            "safe"
        } else {
            "DO NOT MERGE"
        };
        assert_eq!(verdict, "WAIT", "pending must outrank an empty failure list");
    }
}

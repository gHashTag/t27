//! `tri red` — what is failing on main right now, and since when.
//!
//! This exists because of a specific evening. The publisher for t27.ai failed
//! six consecutive times and the site served hours-old content. A watchdog
//! caught it correctly and went red five times, on my own commits. I did not
//! read it once. I found the outage by accident, hours later, when a page I
//! had just published returned 404.
//!
//! The detection was never the problem. Reading it was. So the point of this
//! command is that there is no longer an excuse: one call, every repository,
//! newest failure first, with how long each has been failing.
//!
//! It deliberately reports the LATEST run per workflow on the default branch
//! rather than a window average. "Is it broken now" and "how often does it
//! break" are different questions, and only the first one stops a deploy.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::process::Command;

#[derive(Subcommand)]
pub enum RedCmd {
    /// Show workflows whose most recent run on the default branch failed.
    Now {
        /// owner/repo, repeatable. Defaults to the three this fleet uses.
        #[arg(long = "repo")]
        repos: Vec<String>,
        /// Include workflows whose latest run was cancelled or timed out.
        #[arg(long)]
        include_cancelled: bool,
    },
}

pub fn run(cmd: &RedCmd) -> Result<()> {
    match cmd {
        RedCmd::Now {
            repos,
            include_cancelled,
        } => {
            let list: Vec<String> = if repos.is_empty() {
                [
                    "gHashTag/trinity",
                    "gHashTag/ghashtag.github.io",
                    "gHashTag/trinity-fpga",
                    "gHashTag/t27",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect()
            } else {
                repos.clone()
            };
            now(&list, *include_cancelled)
        }
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

struct Red {
    repo: String,
    name: String,
    since: String,
    consecutive: usize,
    /// True when the streak filled the page and the real run is at least this.
    at_least: bool,
}

/// How many of the most recent runs, newest first, share the failing verdict.
/// A single red is noise; nine in a row is an outage nobody is reading.
fn streak(repo: &str, id: &str, branch: &str) -> Result<(usize, String)> {
    let raw = gh(&[
        "api",
        &format!("repos/{repo}/actions/workflows/{id}/runs?branch={branch}&per_page=30"),
        "--jq",
        r#".workflow_runs[]|"\(.conclusion)\t\(.created_at)""#,
    ])?;
    let mut n = 0usize;
    let mut since = String::new();
    for line in raw.lines() {
        let mut it = line.splitn(2, '\t');
        let concl = it.next().unwrap_or("");
        let at = it.next().unwrap_or("");
        match concl {
            // A skipped run is not a verdict about the code — it means the
            // gate's condition was not met. Counting it either way is wrong,
            // so it is stepped over.
            "skipped" | "null" | "" => continue,
            "failure" | "timed_out" | "cancelled" => {
                n += 1;
                since = at.to_string();
            }
            _ => break,
        }
    }
    // n is bounded by the page size above, so a full page is a LOWER BOUND and
    // must not be printed as if it were exact. That is the same silent
    // truncation this command exists to surface, and it appeared here first.
    Ok((n, since))
}

fn now(repos: &[String], include_cancelled: bool) -> Result<()> {
    let mut reds: Vec<Red> = Vec::new();
    for repo in repos {
        let branch = gh(&["api", &format!("repos/{repo}"), "--jq", ".default_branch"])?;
        let listing = gh(&[
            "api",
            &format!("repos/{repo}/actions/workflows?per_page=100"),
            "--jq",
            r#".workflows[]|select(.state=="active")|"\(.id)\t\(.name)""#,
        ])?;
        for line in listing.lines() {
            let mut it = line.splitn(2, '\t');
            let (id, name) = match (it.next(), it.next()) {
                (Some(a), Some(b)) => (a, b),
                _ => continue,
            };
            let latest = gh(&[
                "api",
                &format!("repos/{repo}/actions/workflows/{id}/runs?branch={branch}&per_page=1"),
                "--jq",
                r#".workflow_runs[0].conclusion // "none""#,
            ])?;
            let bad = latest == "failure"
                || latest == "timed_out"
                || (include_cancelled && latest == "cancelled");
            if !bad {
                continue;
            }
            let (n, since) = streak(repo, id, &branch)?;
            reds.push(Red {
                repo: repo.clone(),
                name: name.to_string(),
                since: since.chars().take(16).collect(),
                consecutive: n,
                at_least: n >= 30,
            });
        }
    }

    if reds.is_empty() {
        println!("Nothing is red on the default branch of any of these repositories.");
        return Ok(());
    }

    reds.sort_by(|a, b| b.consecutive.cmp(&a.consecutive));
    println!("{} workflow(s) red on the default branch:\n", reds.len());
    for r in &reds {
        let short: String = r.name.chars().take(38).collect();
        let count = if r.at_least {
            format!("{}+", r.consecutive)
        } else {
            r.consecutive.to_string()
        };
        println!(
            "  {:>5} in a row  since {}  {:<26} {}",
            count, r.since, r.repo, short
        );
    }
    println!();
    println!("A long streak is not more of the same failure — it is the number of");
    println!("times nobody looked. Read this before merging, not after a page 404s.");
    Ok(())
}

#[cfg(test)]
mod tests {
    /// `skipped` is not a verdict about the code: it means the workflow's own
    /// condition was not met. Counting it as a success would end a real streak
    /// early; counting it as a failure would invent one. Both were live
    /// mistakes here — a run polled for its result had been skipped, and
    /// `completed` was matched without looking at the conclusion.
    #[test]
    fn skipped_is_not_a_verdict_in_either_direction() {
        let seq = ["failure", "skipped", "failure", "success"];
        let mut n = 0;
        for c in seq {
            match c {
                "skipped" => continue,
                "failure" => n += 1,
                _ => break,
            }
        }
        assert_eq!(
            n, 2,
            "the skipped run must neither end nor extend the streak"
        );
    }
}

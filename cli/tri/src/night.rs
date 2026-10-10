//! `tri night` — the whole overnight operation in one command.
//!
//! Every loop tick used to begin with fifteen exploratory calls: list the PRs,
//! poll each one's checks, ask for mergeStateStatus, look at failed runs,
//! re-read the notes about what is parked and why. The state was in the
//! operator's head, so a fresh tick (or a fresh agent after compaction)
//! re-derived it from scratch and sometimes missed an item.
//!
//! This command prints that state from the sources that own it:
//!
//! * the train order lives in `.trinity/night-state.json` (written by the
//!   loop, read here verbatim -- this tool never edits it);
//! * each train PR's verdict comes from `gh`, classified against the same
//!   mergeStateStatus vocabulary `tri pr ready` uses;
//! * recent failed workflow runs come from `gh run list`;
//! * anything not in the train is counted, not ignored.
//!
//! The verdict line at the end names exactly one next action. When the state
//! does not support a confident action it says `inspect <n>` rather than
//! guessing, for the same reason `tri pr ready` refuses to guess: an unread
//! state is what this command exists to prevent.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize)]
struct NightState {
    #[serde(default)]
    train: Vec<u64>,
    #[serde(default)]
    parked: std::collections::BTreeMap<u64, String>,
}

/// What mergeStateStatus means for the train, in one word an operator can act on.
fn action_for(state: &str) -> &'static str {
    match state {
        "CLEAN" => "merge",
        "BEHIND" => "update-branch",
        "UNSTABLE" => "compare-vs-master",
        "BLOCKED" => "required-red",
        "DIRTY" => "conflicts-rebase",
        "DRAFT" => "draft",
        _ => "inspect",
    }
}

fn gh_json(args: &[&str]) -> Result<serde_json::Value> {
    let out = Command::new("gh")
        .args(args)
        .output()
        .context("run gh")?;
    if !out.status.success() {
        anyhow::bail!(
            "gh {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(serde_json::from_slice(&out.stdout)?)
}

fn pr_line(repo: &str, number: u64, parked: Option<&str>) -> Result<String> {
    let pr = gh_json(&[
        "pr",
        "view",
        &number.to_string(),
        "--repo",
        repo,
        "--json",
        "title,mergeStateStatus",
    ])?;
    let title = pr["title"].as_str().unwrap_or("").chars().take(48).collect::<String>();
    let status = pr["mergeStateStatus"].as_str().unwrap_or("UNKNOWN");
    let parked_note = parked.map(|p| format!("  PARKED: {}", p)).unwrap_or_default();
    Ok(format!(
        "  #{:<6} {:<50} {:<18} {}{}",
        number,
        title,
        status,
        action_for(status),
        parked_note
    ))
}

pub fn run(repo: Option<&str>, limit: usize) -> Result<()> {
    let repo = repo.unwrap_or("gHashTag/t27");
    let root = std::path::Path::new(".trinity/night-state.json");
    let state: NightState = match std::fs::read_to_string(root) {
        Ok(text) => serde_json::from_str(&text).with_context(|| format!("parse {}", root.display()))?,
        Err(_) => NightState {
            train: Vec::new(),
            parked: Default::default(),
        },
    };

    println!("== tri night == repo {}", repo);

    if state.train.is_empty() && state.parked.is_empty() {
        println!("train: no {} found -- the loop has not written one yet", root.display());
    } else {
        println!("train ({}):", root.display());
        for n in &state.train {
            println!("{}", pr_line(repo, *n, state.parked.get(n).map(|s| s.as_str()))?);
        }
    }

    let open: Vec<serde_json::Value> = gh_json(&[
        "pr",
        "list",
        "--repo",
        repo,
        "--state",
        "open",
        "--limit",
        "100",
        "--json",
        "number",
    ])?
    .as_array()
    .cloned()
    .unwrap_or_default();
    let in_train: std::collections::BTreeSet<u64> = state.train.iter().copied().collect();
    let others: Vec<u64> = open
        .iter()
        .filter_map(|p| p["number"].as_u64())
        .filter(|n| !in_train.contains(n))
        .collect();
    println!("open PRs not in train: {}", others.len());

    let runs: Vec<serde_json::Value> = gh_json(&[
        "run",
        "list",
        "--repo",
        repo,
        "--status",
        "failure",
        "--limit",
        "10",
        "--json",
        "workflowName,createdAt",
    ])?
    .as_array()
    .cloned()
    .unwrap_or_default();
    if runs.is_empty() {
        println!("failed runs (latest 10): none");
    } else {
        println!("failed runs (latest 10):");
        for r in runs.iter().take(limit) {
            println!(
                "  {} {}",
                r["createdAt"].as_str().unwrap_or("?"),
                r["workflowName"].as_str().unwrap_or("?")
            );
        }
    }

    // Exactly one next action, or an honest refusal to pick one.
    let mut next = String::from("none");
    for n in &state.train {
        if state.parked.contains_key(n) {
            continue;
        }
        let pr = gh_json(&[
            "pr",
            "view",
            &n.to_string(),
            "--repo",
            repo,
            "--json",
            "mergeStateStatus",
        ])?;
        let status = pr["mergeStateStatus"].as_str().unwrap_or("UNKNOWN");
        let act = action_for(status);
        if act == "merge" || act == "update-branch" || act == "required-red" || act == "conflicts-rebase" {
            next = format!("#{} {}", n, act);
            break;
        }
    }
    println!("verdict: next={}", next);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::action_for;

    #[test]
    fn clean_merges_and_behind_updates() {
        assert_eq!(action_for("CLEAN"), "merge");
        assert_eq!(action_for("BEHIND"), "update-branch");
    }

    #[test]
    fn unknown_states_refuse_to_guess() {
        assert_eq!(action_for("UNKNOWN"), "inspect");
        assert_eq!(action_for("anything-else"), "inspect");
    }
}

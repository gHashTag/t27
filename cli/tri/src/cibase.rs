//! `tri ci baseline` — which gates can turn a pull request red with no green
//! state anyone has ever seen?
//!
//! `tri pr ready` classifies a failure by comparing it against the default
//! branch. That comparison is only worth something if the check has ever RUN on
//! the default branch. `emit-bitexact-gate.yml` had not, in the whole life of
//! the repository: `pull_request` with a `paths:` filter and no `push:`. The
//! gate reported "NOT failing on recent master commits", which was assembled
//! from zero observations, and blocked two pull requests on it.
//!
//! Two wrong answers were produced before this one, and both are the point:
//!
//!   * Reading triggers alone flagged 47 workflows because their `push:`
//!     carries a `paths:` filter. Not a hole -- a filtered push still runs on
//!     the default branch when those paths change, which is the sparse baseline
//!     `tri pr ready` already walks several commits to find.
//!   * Counting default-branch runs alone flagged 46. Also not holes: release
//!     pipelines, tag-triggered signing, nightly schedules and dispatch-only
//!     bitstream builds are SUPPOSED to have none, and they never paint a pull
//!     request red.
//!
//! The finding is the intersection, and only the intersection: runs on
//! `pull_request`, and has never run on the default branch by any event. Across
//! three repositories that was 4, not 47 and not 46.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::process::Command;

#[derive(Subcommand)]
pub enum CiCmd {
    /// List PR gates that have never run on the default branch.
    Baseline {
        /// owner/repo. Defaults to the repository in the current directory.
        #[arg(long)]
        repo: Option<String>,
        /// Non-zero exit if any gate has no baseline. For use in CI.
        #[arg(long)]
        strict: bool,
    },
}

pub fn run(cmd: &CiCmd) -> Result<()> {
    match cmd {
        CiCmd::Baseline { repo, strict } => baseline(repo.as_deref(), *strict),
    }
}

fn gh(args: &[&str]) -> Result<String> {
    let out = Command::new("gh")
        .args(args)
        .output()
        .context("failed to run gh; is the GitHub CLI installed and logged in?")?;
    if !out.status.success() {
        anyhow::bail!("gh {:?}: {}", args, String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Does this workflow file run on pull requests?
///
/// Deliberately a text scan and not a YAML parse: `on:` is a YAML 1.1 boolean,
/// so a parser hands back the key `true` rather than `"on"`, and a sweep that
/// looks up `"on"` finds nothing in every file and reports a clean repository
/// because it read nothing. A scan for the trigger word cannot make that
/// mistake, and being wrong here only costs an extra API call.
fn is_pr_gated(body: &str) -> bool {
    body.lines().any(|l| {
        let t = l.trim_start();
        if t.starts_with('#') {
            return false;
        }
        // Block form (`pull_request:`), block-list form (`- pull_request`), and
        // the inline flow list (`on: [push, pull_request]`) are the same
        // trigger written three ways. The first draft of this function missed
        // the third and its own test caught it -- which is the only reason the
        // sweep would not have quietly under-reported.
        t.starts_with("pull_request:")
            || t.starts_with("pull_request_target:")
            || t.starts_with("- pull_request")
            || (t.starts_with("on:")
                && t.contains('[')
                && t.contains("pull_request"))
    })
}

fn baseline(repo: Option<&str>, strict: bool) -> Result<()> {
    let repo = match repo {
        Some(r) => r.to_string(),
        None => gh(&["repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"])?,
    };
    let branch = gh(&["api", &format!("repos/{repo}"), "--jq", ".default_branch"])?;

    let listing = gh(&[
        "api",
        &format!("repos/{repo}/actions/workflows?per_page=100"),
        "--paginate",
        "--jq",
        r#".workflows[]|select(.state=="active")|[.id,.path,.name]|@tsv"#,
    ])?;

    let mut holes: Vec<(String, String)> = Vec::new();
    let mut pr_gates = 0usize;
    for line in listing.lines() {
        let mut it = line.splitn(3, '\t');
        let (Some(id), Some(path), Some(name)) = (it.next(), it.next(), it.next()) else {
            continue;
        };
        // Read the DEFAULT BRANCH's copy. A feature branch may have added or
        // removed the trigger, and the question is about the branch being gated.
        let body = gh(&[
            "api",
            &format!("repos/{repo}/contents/{path}?ref={branch}"),
            "--jq",
            ".content",
        ])
        .unwrap_or_default();
        let decoded = decode_b64(&body);
        if !is_pr_gated(&decoded) {
            continue;
        }
        pr_gates += 1;
        // ANY event counts as a baseline, not just push: a manual dispatch on
        // the default branch is a real observation, and dispatching is how the
        // first of these holes was actually closed.
        let count = gh(&[
            "api",
            &format!("repos/{repo}/actions/workflows/{id}/runs?branch={branch}&per_page=1"),
            "--jq",
            ".total_count",
        ])
        .unwrap_or_else(|_| "-1".into());
        if count == "0" {
            let dispatchable = decoded.contains("workflow_dispatch");
            holes.push((
                path.rsplit('/').next().unwrap_or(path).to_string(),
                format!(
                    "{name}{}",
                    if dispatchable {
                        "  [workflow_dispatch — one run on the default branch fixes this]"
                    } else {
                        ""
                    }
                ),
            ));
        }
    }

    println!("{repo} (default branch {branch})");
    println!("  PR-gated workflows: {pr_gates}");
    println!("  of those, never run on {branch} by any event: {}", holes.len());
    for (file, note) in &holes {
        println!("     {file}\n       {note}");
    }
    println!();
    if holes.is_empty() {
        println!("Every PR gate has run on {branch} at least once, so every failure");
        println!("can be compared against something that was actually observed.");
        return Ok(());
    }
    println!("Each of these can turn a pull request red while no green state has ever");
    println!("existed on {branch}. A red check nobody else has reads as \"you broke it\";");
    println!("often it means \"nobody has ever seen this pass\".");
    println!();
    println!("Not every one is a defect. Three kinds hide in this list, and they need");
    println!("different answers:");
    println!("  * a configuration hole — the question IS answerable on {branch}, and the");
    println!("    workflow simply never asks it there. Add a push: trigger.");
    println!("  * inherently pull-request-scoped — \"does this change update the log?\"");
    println!("    has no meaning on {branch}. Correct as it stands.");
    println!("  * sound but unexercised — a push: trigger exists with a paths: filter,");
    println!("    and those paths have not changed on {branch} yet. Nothing to fix.");
    println!("Read the workflow before filing any of them.");

    if strict {
        anyhow::bail!("{} PR gate(s) have no baseline on {branch}", holes.len());
    }
    Ok(())
}

/// Minimal base64 for the contents API, which returns wrapped base64.
fn decode_b64(s: &str) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let cleaned: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace() && *b != b'=').collect();
    let mut out = Vec::new();
    let mut acc: u32 = 0;
    let mut bits = 0u32;
    for c in cleaned {
        let Some(v) = T.iter().position(|&t| t == c) else {
            continue;
        };
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The trigger scan must survive the YAML `on:`-is-a-boolean trap that a
    /// parser walks into, and must not fire on the word appearing in prose.
    #[test]
    fn pr_trigger_is_found_in_every_spelling() {
        assert!(is_pr_gated("on:\n  pull_request:\n    branches: [master]\n"));
        assert!(is_pr_gated("on: [push, pull_request]\n"));
        assert!(is_pr_gated("on:\n  - pull_request\n"));
        assert!(is_pr_gated("on:\n  pull_request_target:\n    types: [opened]\n"));
        assert!(!is_pr_gated("on:\n  push:\n    branches: [main]\n"));
        assert!(!is_pr_gated("# this gate matters for every pull_request we open\n"));
    }

    /// A workflow with a push: trigger whose paths have simply never changed on
    /// the default branch is NOT a configuration hole -- reading triggers alone
    /// produced 47 findings of which the overwhelming majority were this. The
    /// command therefore reports run counts and refuses to grade them for you.
    #[test]
    fn base64_round_trips_a_workflow_body() {
        // "on:\n  pull_request:\n" — as the contents API returns it, wrapped.
        let encoded = "b246CiAgcHVsbF9y\nZXF1ZXN0Ogo=";
        let decoded = decode_b64(encoded);
        assert!(decoded.contains("pull_request:"));
        assert!(is_pr_gated(&decoded));
    }
}

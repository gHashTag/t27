//! `tri gates` — find workflows that have never once succeeded.
//!
//! A gate that has never been green carries no information: it is red before
//! your change and red after it, so nobody reads it — and after a while nobody
//! reads the others either. Eighteen such workflows were found across three of
//! these repositories, between them consuming 8182 runs and producing zero
//! green results.
//!
//! That is not an aesthetic complaint. It is the measured cause of nine
//! defects living undetected in a request path that had executed once in its
//! lifetime: when red is the normal colour, a real red says nothing.
//!
//! This was a hand-run loop of `gh api` calls three times before it became a
//! command. It reports, it does not disable anything — deciding between fix,
//! dispatch-only and delete belongs to whoever owns the workflow.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::process::Command;

#[derive(Subcommand)]
pub enum GatesCmd {
    /// List active workflows whose lifetime success count is zero.
    Dead {
        /// owner/repo, repeatable. Defaults to the three this fleet uses.
        #[arg(long = "repo")]
        repos: Vec<String>,
        /// Ignore workflows with fewer lifetime runs than this, so a new or
        /// rarely-triggered workflow is not reported as dead.
        #[arg(long, default_value_t = 50)]
        min_runs: u64,
    },
}

pub fn run(cmd: &GatesCmd) -> Result<()> {
    match cmd {
        GatesCmd::Dead { repos, min_runs } => {
            let list: Vec<String> = if repos.is_empty() {
                ["gHashTag/trinity", "gHashTag/trinity-fpga", "gHashTag/t27"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            } else {
                repos.clone()
            };
            dead(&list, *min_runs)
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

fn count(repo: &str, id: &str, success_only: bool) -> Result<u64> {
    let path = if success_only {
        format!("repos/{repo}/actions/workflows/{id}/runs?status=success&per_page=1")
    } else {
        format!("repos/{repo}/actions/workflows/{id}/runs?per_page=1")
    };
    let s = gh(&["api", &path, "--jq", ".total_count"])?;
    Ok(s.parse().unwrap_or(0))
}

/// Is a zero success count over `total` lifetime runs too thin to mean
/// anything? A workflow may simply be new, or triggered by a path nobody has
/// touched. Lifted out of `dead` so the floor can be exercised without
/// reaching the network — inline, it was reachable only through `gh`.
fn too_few_runs_to_judge(total: u64, min_runs: u64) -> bool {
    total < min_runs
}

fn dead(repos: &[String], min_runs: u64) -> Result<()> {
    let mut rows: Vec<(String, String, u64)> = Vec::new();
    for repo in repos {
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
            let total = count(repo, id, false)?;
            if too_few_runs_to_judge(total, min_runs) {
                continue;
            }
            if count(repo, id, true)? == 0 {
                rows.push((repo.clone(), name.to_string(), total));
            }
        }
    }

    rows.sort_by(|a, b| b.2.cmp(&a.2));
    if rows.is_empty() {
        println!("No active workflow with >= {min_runs} runs has a zero success count.");
        return Ok(());
    }

    let total: u64 = rows.iter().map(|r| r.2).sum();
    println!(
        "{} workflow(s) have never succeeded, across {} run(s).\n",
        rows.len(),
        total
    );
    for (repo, name, runs) in &rows {
        let short: String = name.chars().take(44).collect();
        println!("  {runs:>6}  {repo:<22} {short}");
    }
    println!();
    println!("A gate that has never been green carries no information: red before");
    println!("your change and red after it. Decide per workflow — fix it, make it");
    println!("workflow_dispatch only, or delete it. Leaving it red is the one");
    println!("option that costs every other gate in the repository.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// `GatesCmd` is a `Subcommand`, so asking clap what `--min-runs` defaults
    /// to needs a root parser. This one exists for no other purpose.
    #[derive(Parser)]
    struct Root {
        #[command(subcommand)]
        action: GatesCmd,
    }

    /// The floor `tri gates dead` actually ships with, read back out of clap
    /// rather than repeated as a literal here.
    fn shipped_floor() -> u64 {
        match Root::parse_from(["tri-gates", "dead"]).action {
            GatesCmd::Dead { min_runs, .. } => min_runs,
        }
    }

    /// The `--min-runs` floor exists because "0 successes" over 2 runs is not
    /// evidence of a dead gate, and reporting it as one would make this
    /// command the thing it is written to find: an alarm nobody reads.
    ///
    /// The guard this replaces declared `let below = 2u64; let at = 50u64;`
    /// and asserted `2 < 50` and `50 >= 50`. It named neither the shipped
    /// default nor the skip inside `dead`: lifted into a file containing no
    /// production code — not even a `use` of this crate — it still compiled
    /// and still passed, and setting `default_value_t` to 0 left all 173
    /// tests green while a two-run workflow became reportable as a dead gate.
    /// The first two assertions below read the shipped floor and put it
    /// through `too_few_runs_to_judge`, the predicate `dead` actually skips
    /// on. Between them they pin the comparison's strictness but barely
    /// constrain the number — assertion 1 fails only for a floor of 0, 1 or
    /// 2 — so the third bounds the value itself (#2374).
    #[test]
    fn the_floor_is_what_makes_a_zero_meaningful() {
        let floor = shipped_floor();

        // A day-old workflow with two runs must be skipped, not reported.
        assert!(
            too_few_runs_to_judge(2, floor),
            "--min-runs defaults to {floor}, so a workflow with 2 lifetime \
             runs and no success would be reported as a dead gate"
        );

        // A workflow standing exactly at the floor must be judged, not skipped.
        assert!(
            !too_few_runs_to_judge(floor, floor),
            "a workflow with exactly {floor} runs and no success must be reported"
        );

        // A floor low enough to judge a handful of runs is no floor at all.
        // At 3, both assertions above still pass while a three-run workflow
        // becomes reportable — the judgement `--min-runs` exists to prevent.
        // A bound rather than an exact pin: retuning stays possible, dropping
        // to a handful does not.
        assert!(
            floor >= 10,
            "--min-runs defaults to {floor}; below 10 a handful of runs is \
             treated as evidence, which is what this flag exists to prevent"
        );
    }
}

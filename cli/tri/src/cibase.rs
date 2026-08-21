//! `tri ci baseline` — which gates can turn a pull request red with no green
//! state anyone has ever seen?
//!
//! `tri pr ready` classifies a failure by comparing it against the default
//! branch. That comparison is only worth something if the check has ever RUN on
//! the default branch. `emit-bitexact-gate.yml` had not, for most of the life
//! of the repository: `pull_request` with a `paths:` filter and no `push:`. The
//! gate reported "NOT failing on recent master commits", which was assembled
//! from zero observations, and blocked two pull requests on it. That is the
//! case that started this line of work, and it is now CLOSED: a manual dispatch
//! on `master` at 2026-08-20T01:05:13Z gave it the baseline it had never had.
//!
//! Two wrong answers were produced before this command. Both are the point:
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
//!
//! Then a third pass corrected the first version of THIS command. Its premise
//! -- "a gate with no baseline can turn a pull request red" -- turned out to be
//! false for one of the four #2309 reported. `seal-staleness-warn` is advisory
//! by construction: every path through its script ends in `exit 0`. Measured
//! over its whole history: 169 runs, 0 failures. It cannot paint anything red,
//! so a missing baseline costs nobody anything.
//!
//! The signal that separates an alarm from a footnote is not the trigger table
//! at all -- it is whether the workflow has ever concluded `failure`. #2309's
//! four were `catalog-count-gate`, `check-now-freshness`, `loop-tools-gate` and
//! `seal-staleness-warn`, and re-measured at 2026-08-21T19:57:20Z they run
//! 37/0, 1446/2, 12/0 and 169/0. Exactly one of them has ever gone red, and it
//! went red twice. So this reports the failure history next to each hole and
//! ranks by it, rather than presenting four findings that are not the same size.
//!
//! `emit-bitexact` is NOT one of those four and never was. #2360 put it at the
//! head of that table as "144 runs, 84 failures, no baseline" -- the run and
//! failure counts are real (147/88 at 2026-08-21T19:57:20Z) but the "no
//! baseline" half was false 41 hours before that pull request opened
//! (dispatch 2026-08-20T01:05:13Z, #2360 opened 2026-08-21T18:05:14Z), and
//! #2309 says so in its own body.
//! #2360's own pasted output listed two holes and did not include it. The
//! measurement that survives is the smaller one above.
//!
//! ## Every answer here can also be MISSING, and missing is not zero
//!
//! Three separate `gh` calls per workflow can fail on a rate limit, a 502 or an
//! expired token. The first version of this command turned each of those into a
//! benign default -- an unreadable failure count became `-1`, which lost every
//! `> 0` test it met, so one transient 502 silently downgraded every hole in
//! the sweep and printed "has never run anywhere at all" about a workflow whose
//! history could not be read. That is the same error #2309 made: an answer
//! assembled from zero observations, presented as an observation.
//!
//! So counts are `Option<i64>` and `None` propagates all the way to the exit
//! code. The one exception is deliberate and measured: a 404 from the contents
//! API IS an answer. 12 of this repository's 54 registered active workflows
//! have no file on `master` at all, and a workflow that is not on the default
//! branch does not gate the default branch. `Fetch` keeps that apart from a
//! failure to reach any answer.

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
        /// Non-zero exit unless every hole has RUN and never once failed.
        /// A hole that has gone red, one that has never run anywhere, and one
        /// whose history could not be read all fail. For use in CI.
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

/// What the API said, with "it is not there" kept apart from "it did not say".
enum Fetch {
    Body(String),
    /// A 404. For a file path this is a real answer: the file is not on that
    /// ref. Treating it as a failure would flag 12 of this repository's 54
    /// registered workflows, none of which gates anything on `master`.
    Absent,
    /// No answer at all: rate limit, 502, expired token, `gh` not on PATH.
    Failed(String),
}

/// `gh` prints a 404 as `gh: Not Found (HTTP 404)` on stderr and exits 1, the
/// same exit code it uses for a rate limit. The status line is the only thing
/// that separates them, so read it rather than the exit code.
fn is_not_found(stderr: &str) -> bool {
    stderr.contains("HTTP 404")
}

fn fetch(args: &[&str]) -> Fetch {
    let out = match Command::new("gh").args(args).output() {
        Ok(o) => o,
        Err(e) => return Fetch::Failed(format!("could not run gh: {e}")),
    };
    if out.status.success() {
        return Fetch::Body(String::from_utf8_lossy(&out.stdout).trim().to_string());
    }
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if is_not_found(&stderr) {
        Fetch::Absent
    } else {
        Fetch::Failed(stderr)
    }
}

/// Parse a `total_count` reply. `None` means the API did not give a number --
/// it is NOT a zero, and nothing downstream may treat it as one. The predecessor
/// of this function returned `-1` here, which every `> 0` test read as quiet.
fn count(reply: &str) -> Option<i64> {
    let n: i64 = reply.trim().parse().ok()?;
    if n < 0 {
        return None;
    }
    Some(n)
}

/// One PR gate with no default-branch baseline, and the history that says
/// whether that costs anyone anything.
struct Hole {
    file: String,
    name: String,
    dispatchable: bool,
    /// `None` = the API refused to say. Not zero.
    runs: Option<i64>,
    /// `None` = the API refused to say. Not zero.
    fails: Option<i64>,
}

/// A workflow the sweep could not finish measuring. Printed, and counted
/// toward `--strict`: the command must not answer a question it could not ask.
struct Unmeasured {
    file: String,
    why: String,
}

/// How loud is a hole? Declaration order IS the ranking: `derive(Ord)` reads it
/// bottom-loudest, and `rank` sorts descending.
///
/// The three quiet-looking states are not interchangeable. `169 runs / 0
/// failures` is evidence of harmlessness. `0 runs` is the absence of evidence.
/// An unreadable count is a failed measurement. #2360 collapsed all three into
/// one exemption; only the first has earned it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Severity {
    /// Has run, often, and has never once concluded `failure`.
    NeverFailed,
    /// Has never run anywhere at all -- the strongest form of "no green state
    /// has ever been observed", not the weakest.
    NeverRan,
    /// The API would not say. An unmeasured gate is not a passing one.
    Unreadable,
    /// Observed to fail, while no green state exists on the branch it gates.
    Fails,
}

fn severity(h: &Hole) -> Severity {
    match (h.runs, h.fails) {
        (Some(_), Some(f)) if f > 0 => Severity::Fails,
        (Some(0), Some(0)) => Severity::NeverRan,
        (Some(_), Some(_)) => Severity::NeverFailed,
        _ => Severity::Unreadable,
    }
}

/// Loudest first: a gate that fails often and has never been seen to pass on
/// the branch it gates is the alarm; one that has run for years without ever
/// failing is a footnote, and printing them at the same volume is how a sweep
/// loses the reader's trust. File name breaks ties so the output is stable.
fn rank(holes: &mut [Hole]) {
    holes.sort_by(|a, b| {
        severity(b)
            .cmp(&severity(a))
            .then_with(|| b.fails.unwrap_or(0).cmp(&a.fails.unwrap_or(0)))
            .then_with(|| a.file.cmp(&b.file))
    });
}

fn verdict(h: &Hole) -> &'static str {
    match severity(h) {
        Severity::Fails => "CAN and DOES fail — no green state has ever been observed",
        Severity::Unreadable => {
            "history could not be read — UNMEASURED, which is not the same as clean"
        }
        Severity::NeverRan => "has never run anywhere at all — no green state exists anywhere",
        Severity::NeverFailed => {
            "has never failed in its whole history; a missing baseline costs nothing yet"
        }
    }
}

fn show(c: Option<i64>) -> String {
    match c {
        Some(n) => n.to_string(),
        None => "?".to_string(),
    }
}

/// How many findings cannot be shown to be harmless. Everything except
/// `NeverFailed`, plus every workflow the sweep could not measure.
fn alarming(holes: &[Hole], unmeasured: &[Unmeasured]) -> usize {
    holes
        .iter()
        .filter(|h| severity(h) != Severity::NeverFailed)
        .count()
        + unmeasured.len()
}

/// The exit-code half of `--strict`, kept out of the command body so it can be
/// tested without a network. `Some(msg)` means the command must fail.
fn strict_verdict(
    strict: bool,
    holes: &[Hole],
    unmeasured: &[Unmeasured],
    branch: &str,
) -> Option<String> {
    if !strict {
        return None;
    }
    let n = alarming(holes, unmeasured);
    if n == 0 {
        return None;
    }
    Some(format!(
        "{n} PR gate(s) have no baseline on {branch} and cannot be shown to be harmless \
         (failed before, never ran anywhere, or history unreadable)"
    ))
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
            || (t.starts_with("on:") && t.contains('[') && t.contains("pull_request"))
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

    let mut holes: Vec<Hole> = Vec::new();
    let mut unmeasured: Vec<Unmeasured> = Vec::new();
    let mut pr_gates = 0usize;
    for line in listing.lines() {
        let mut it = line.splitn(3, '\t');
        let (Some(id), Some(path), Some(name)) = (it.next(), it.next(), it.next()) else {
            continue;
        };
        let short = path.rsplit('/').next().unwrap_or(path).to_string();
        // Read the DEFAULT BRANCH's copy. A feature branch may have added or
        // removed the trigger, and the question is about the branch being gated.
        let decoded = match fetch(&[
            "api",
            &format!("repos/{repo}/contents/{path}?ref={branch}"),
            "--jq",
            ".content",
        ]) {
            Fetch::Body(b) => decode_b64(&b),
            // Not on the default branch, so it gates nothing there.
            Fetch::Absent => continue,
            Fetch::Failed(why) => {
                unmeasured.push(Unmeasured {
                    file: short,
                    why: format!("could not read the workflow file on {branch}: {why}"),
                });
                continue;
            }
        };
        if !is_pr_gated(&decoded) {
            continue;
        }
        pr_gates += 1;
        // ANY event counts as a baseline, not just push: a manual dispatch on
        // the default branch is a real observation, and dispatching is how the
        // first of these holes was actually closed.
        let on_branch = match fetch(&[
            "api",
            &format!("repos/{repo}/actions/workflows/{id}/runs?branch={branch}&per_page=1"),
            "--jq",
            ".total_count",
        ]) {
            Fetch::Body(b) => count(&b),
            _ => None,
        };
        // An unreadable count here used to mean "has a baseline", which deleted
        // the finding outright. It now means the sweep did not finish.
        let Some(seen_on_branch) = on_branch else {
            unmeasured.push(Unmeasured {
                file: short,
                why: format!("could not count its runs on {branch}"),
            });
            continue;
        };
        if seen_on_branch != 0 {
            continue;
        }
        let dispatchable = decoded.contains("workflow_dispatch");
        // Has this workflow ever gone red ANYWHERE? A gate that has never
        // failed in its whole history cannot be shown to cost anyone anything
        // by lacking a baseline, and one that fails constantly is the alarm.
        // The first version of this command reported both at the same volume
        // and was wrong about one of them.
        let runs = match fetch(&[
            "api",
            &format!("repos/{repo}/actions/workflows/{id}/runs?per_page=1"),
            "--jq",
            ".total_count",
        ]) {
            Fetch::Body(b) => count(&b),
            _ => None,
        };
        let fails = match fetch(&[
            "api",
            &format!("repos/{repo}/actions/workflows/{id}/runs?status=failure&per_page=1"),
            "--jq",
            ".total_count",
        ]) {
            Fetch::Body(b) => count(&b),
            _ => None,
        };
        holes.push(Hole {
            file: short,
            name: name.to_string(),
            dispatchable,
            runs,
            fails,
        });
    }

    rank(&mut holes);

    println!("{repo} (default branch {branch})");
    println!("  PR-gated workflows: {pr_gates}");
    println!("  of those, never run on {branch} by any event: {}", holes.len());
    for h in &holes {
        println!("     {}", h.file);
        println!("       {}", h.name);
        println!(
            "       {} run(s), {} failure(s) — {}",
            show(h.runs),
            show(h.fails),
            verdict(h)
        );
        if h.dispatchable {
            println!("       workflow_dispatch — one run on {branch} closes this");
        }
    }
    if !unmeasured.is_empty() {
        println!("  workflows the sweep could not measure: {}", unmeasured.len());
        for u in &unmeasured {
            println!("     {}", u.file);
            println!("       {}", u.why);
        }
        println!("  These are not findings and they are not clean bills of health.");
        println!("  Re-run when the API is answering again.");
    }
    println!();
    if holes.is_empty() && unmeasured.is_empty() {
        println!("Every PR gate has run on {branch} at least once, so every failure");
        println!("can be compared against something that was actually observed.");
        return Ok(());
    }
    let n = alarming(&holes, &unmeasured);
    if n == 0 {
        println!("Every one of them HAS run, and none has ever concluded failure. A missing");
        println!("baseline is only a problem for a gate that can go red, so this list is a");
        println!("note, not an alarm.");
    } else {
        println!("{n} of them cannot be shown to be harmless: each has gone red before, or has");
        println!("never run anywhere, or its history could not be read — while no green state");
        println!("has ever existed on {branch}. A red check nobody else has reads as \"you broke");
        println!("it\"; sometimes it means \"nobody has ever seen this pass\".");
    }
    println!();
    println!("Not every one is a defect. Four kinds hide in this list, and they need");
    println!("different answers:");
    println!("  * a configuration hole — the question IS answerable on {branch}, and the");
    println!("    workflow simply never asks it there. Add a push: trigger.");
    println!("  * inherently pull-request-scoped — \"does this change update the log?\"");
    println!("    has no meaning on {branch}. Correct as it stands.");
    println!("  * sound but unexercised — a push: trigger exists with a paths: filter,");
    println!("    and those paths have not changed on {branch} yet. Nothing to fix.");
    println!("  * advisory by construction — every path through the script ends in");
    println!("    exit 0, so it appears in the check list and can never block. The");
    println!("    failure count above is the tell: this command's first version");
    println!("    reported one of these as a hole, and it was not one.");
    println!("Read the workflow before filing any of them.");

    if let Some(msg) = strict_verdict(strict, &holes, &unmeasured, &branch) {
        anyhow::bail!("{msg}");
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

    fn hole(file: &str, runs: Option<i64>, fails: Option<i64>) -> Hole {
        Hole {
            file: file.into(),
            name: file.into(),
            dispatchable: false,
            runs,
            fails,
        }
    }

    /// Every assertion below calls the SHIPPED `rank`, `severity`, `verdict`,
    /// `alarming`, `strict_verdict` and `count`. The guard that came with #2360
    /// copied the production sort and the `loud` predicate into its own body
    /// and asserted against its own local vector; lifting the whole test into a
    /// file containing no production code at all left it compiling and passing.
    /// It constrained `std::slice::sort_by`, not `tri ci baseline`.
    ///
    /// #2309's four holes, re-measured 2026-08-21T19:57:20Z against the live API:
    /// `catalog-count-gate` 37/0, `check-now-freshness` 1446/2,
    /// `loop-tools-gate` 12/0, `seal-staleness-warn` 169/0. Exactly one has
    /// ever gone red. `emit-bitexact-gate` was never in that set: it has had a
    /// `master` baseline since 2026-08-20T01:05:13Z, 41 hours before #2360
    /// opened, and #2360's own pasted output does not list it.
    #[test]
    fn a_gate_that_has_never_failed_ranks_below_one_that_has() {
        let mut holes = vec![
            hole("seal-staleness-warn.yml", Some(169), Some(0)),
            hole("check-now-freshness.yml", Some(1446), Some(2)),
            hole("catalog-count-gate.yml", Some(37), Some(0)),
        ];
        rank(&mut holes);
        assert_eq!(holes[0].file, "check-now-freshness.yml");
        assert_eq!(holes[2].file, "seal-staleness-warn.yml");

        // A run count alone says nothing about rank. The gate sorted LAST has
        // run four and a half times as often as the one placed above it
        // (169 vs 37); both have never failed, and the tie broke on file name.
        // Ranking by activity would have inverted this pair.
        assert!(holes[2].runs > holes[1].runs);
        assert_eq!(holes[1].file, "catalog-count-gate.yml");

        assert_eq!(severity(&holes[0]), Severity::Fails);
        assert_eq!(severity(&holes[2]), Severity::NeverFailed);
        assert_eq!(alarming(&holes, &[]), 1);
    }

    /// The three quiet-looking states must rank and count apart. `169 runs / 0
    /// failures` is evidence of harmlessness; `0 runs` is the absence of
    /// evidence; a count the API refused to give is a failed measurement.
    #[test]
    fn unknown_and_never_ran_outrank_a_measured_zero() {
        let mut holes = vec![
            hole("measured-zero.yml", Some(169), Some(0)),
            hole("never-ran.yml", Some(0), Some(0)),
            hole("unreadable.yml", None, None),
            hole("goes-red.yml", Some(146), Some(87)),
        ];
        rank(&mut holes);
        let order: Vec<&str> = holes.iter().map(|h| h.file.as_str()).collect();
        assert_eq!(
            order,
            vec![
                "goes-red.yml",
                "unreadable.yml",
                "never-ran.yml",
                "measured-zero.yml"
            ]
        );

        // Only the measured zero is exempt. #2360 exempted all three.
        assert_eq!(alarming(&holes, &[]), 3);
    }

    /// `gh` bails on any non-zero exit, so a 502 or a rate limit used to become
    /// `""`, and `""` used to parse to `-1`, which lost both `> 0` tests it
    /// met: the hole dropped out of the strict count AND the verdict ladder
    /// printed "has never run anywhere at all" about a workflow whose history
    /// nobody had read.
    #[test]
    fn an_unreadable_count_is_not_a_zero_and_does_not_claim_it_never_ran() {
        assert_eq!(count(""), None);
        assert_eq!(count("gh: api rate limit exceeded"), None);
        assert_eq!(count("-1"), None);
        assert_eq!(count("0"), Some(0));
        assert_eq!(count(" 146\n"), Some(146));

        let unknown = hole("unreadable.yml", None, None);
        assert_eq!(severity(&unknown), Severity::Unreadable);
        assert!(verdict(&unknown).contains("could not be read"));
        assert!(!verdict(&unknown).contains("never run anywhere at all"));
        assert_eq!(alarming(std::slice::from_ref(&unknown), &[]), 1);
    }

    /// `--strict` must fail on a gate that has gone red, on one that has never
    /// run anywhere, and on a sweep that could not finish. It must stay silent
    /// only for a gate that HAS run and never once failed, and it must never
    /// fire when the flag is off.
    #[test]
    fn strict_fires_on_red_on_never_ran_and_on_unmeasured() {
        let red = vec![hole("goes-red.yml", Some(146), Some(87))];
        let never_ran = vec![hole("never-ran.yml", Some(0), Some(0))];
        let unknown = vec![hole("unreadable.yml", None, None)];
        let quiet = vec![hole("advisory.yml", Some(169), Some(0))];
        let stalled = vec![Unmeasured {
            file: "x.yml".into(),
            why: "502".into(),
        }];

        assert!(strict_verdict(true, &red, &[], "master").is_some());
        assert!(strict_verdict(true, &never_ran, &[], "master").is_some());
        assert!(strict_verdict(true, &unknown, &[], "master").is_some());
        assert!(strict_verdict(true, &quiet, &stalled, "master").is_some());

        // The one exemption that was actually earned.
        assert!(strict_verdict(true, &quiet, &[], "master").is_none());
        // Nothing at all to report.
        assert!(strict_verdict(true, &[], &[], "master").is_none());
        // The flag is off.
        assert!(strict_verdict(false, &red, &stalled, "master").is_none());

        let msg = strict_verdict(true, &red, &stalled, "master").unwrap();
        assert!(msg.starts_with("2 PR gate(s)"), "message was: {msg}");
    }

    /// A 404 from the contents API is an ANSWER -- 12 of this repository's 54
    /// registered active workflows have no file on `master`, and a workflow
    /// that is not on the default branch gates nothing there. A rate limit is
    /// not an answer. Both exit 1, so only the status line separates them.
    #[test]
    fn a_404_is_an_answer_and_a_rate_limit_is_not() {
        assert!(is_not_found("gh: Not Found (HTTP 404)"));
        assert!(!is_not_found(
            "gh: API rate limit exceeded for user ID 1 (HTTP 403)"
        ));
        assert!(!is_not_found("gh: Bad gateway (HTTP 502)"));
        assert!(!is_not_found(""));
    }

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

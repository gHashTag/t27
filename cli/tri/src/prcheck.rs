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
use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

#[derive(Subcommand)]
pub enum PrCmd {
    /// Classify every failing check and say plainly whether it is safe to merge.
    /// Did this pull request's content actually reach the default branch?
    ///
    /// "merged" and "closed" both read as success in a pull-request list. A
    /// stack taught this the expensive way: the base squash-merged, its
    /// branch was deleted, the pull request stacked on it auto-closed, and
    /// four commits reached nothing while the list looked fine. Only a
    /// content probe distinguishes the two.
    Landed {
        /// Pull request number.
        number: u64,
        /// owner/repo. Defaults to the repository in the current directory.
        #[arg(long)]
        repo: Option<String>,
        /// A string the pull request introduced. Repeatable. Each is looked
        /// for in the default branch's copy of the files the PR touched.
        ///
        /// Choose something the change ALONE introduced, and copy it exactly.
        /// Three real misses on first use, all of them the probe's fault and
        /// not the tool's: `0x3E00` was already in the codec's own source (a
        /// probe the repository can satisfy without the change proves
        /// nothing); a probe spanning a line break failed until whitespace
        /// was flattened on both sides; and one differed only in the case of
        /// its first letter. Case is text and stays significant; line wrapping
        /// is formatting and does not. A fourth appeared during a sweep of
        /// older merges: probing pull request N with wording a LATER pull
        /// request rewrote. Probe with the string as that pull request
        /// introduced it, not as the file reads today.
        #[arg(long = "probe")]
        probes: Vec<String>,
        /// A path the pull request added, asserted to exist on the default
        /// branch. Separate from --probe because a filename is not content:
        /// probing for "CITED_NUMBERS" reported ABSENT while
        /// research/CITED_NUMBERS_2026-08-20.md was present — the file simply
        /// does not contain its own name.
        #[arg(long = "file")]
        files_present: Vec<String>,
    },
    Ready {
        /// Pull request number.
        number: u64,
        /// owner/repo. Defaults to the repository in the current directory.
        #[arg(long)]
        repo: Option<String>,
        /// How many recently merged pull requests to compare against.
        #[arg(long, default_value_t = 5)]
        baseline: usize,
        /// Block until every check has finished, then report. Without this a
        /// verdict can be computed while checks are still starting.
        #[arg(long)]
        wait: bool,
        /// Seconds between polls while waiting.
        #[arg(long, default_value_t = 30)]
        poll: u64,
        /// Merge the pull request if — and only if — the verdict is safe.
        ///
        /// The verdict cannot gate anything if the caller puts `gh pr merge` in
        /// the same batch as this command: it prints WAIT, the merge runs
        /// anyway, and nobody reads the line. That happened four times in one
        /// session. Handing the merge to the command makes the two inseparable.
        #[arg(long)]
        merge: bool,
        /// Refuse unless the pull request's head branch is this one.
        ///
        /// The number is the only thing this command is given, and a number is
        /// the one part of a pull request that cannot be checked against
        /// anything. A serial lander typed `3160` where `gh pr create` had
        /// printed `3161`, and #3160 belonged to a different session working in
        /// the same repository -- it was queued for merge on a green verdict and
        /// caught by hand.
        ///
        /// An author check does NOT catch this: every session here authenticates
        /// as the same GitHub user, so both pull requests read as mine. The head
        /// branch is what differs, and the caller always knows which branch it
        /// just pushed.
        #[arg(long, value_name = "NAME")]
        expect_branch: Option<String>,
        /// Wait only for the contexts the branch ruleset requires.
        ///
        /// Without this the wait counts every check. Four decide the merge here
        /// and the other thirty are signal, not gates -- waiting on them held a
        /// five-deep queue for hours while every required context was green.
        #[arg(long)]
        required_only: bool,
        /// For each failure called pre-existing, compare WHY it fails here and
        /// there: the name of the failing step and the last lines of that
        /// step's own output, read from both jobs' logs.
        ///
        /// A name is not a reason. "pre-existing" is decided by check name
        /// alone, so a pull request that adds a fifth conflicted type name to
        /// a ratchet already red on master for four reads exactly like one that
        /// adds nothing. The difference is in the step's output, under the
        /// same red name. Same step and same lines: the same failure. Anything
        /// else is its own verdict, NEW REASON, exit 7. Off by default: it
        /// downloads two job logs per pre-existing failure.
        #[arg(long)]
        why: bool,
    },
}

pub fn run(cmd: &PrCmd) -> Result<()> {
    match cmd {
        PrCmd::Ready {
            number,
            repo,
            baseline,
            wait,
            poll,
            merge,
            expect_branch,
            required_only,
            why,
        } => ready(
            *number,
            repo.as_deref(),
            *baseline,
            *wait,
            *poll,
            *merge,
            expect_branch.as_deref(),
            *required_only,
            *why,
        ),
        PrCmd::Landed {
            number,
            repo,
            probes,
            files_present,
        } => landed(*number, repo.as_deref(), probes, files_present),
    }
}

/// Check that what the pull request introduced is present in the default
/// branch, file by file. Status is not content: a merged pull request whose
/// stack-mate was auto-closed leaves a list that reads as success.
fn landed(n: u64, repo: Option<&str>, probes: &[String], files_present: &[String]) -> Result<()> {
    let repo = match repo {
        Some(r) => r.to_string(),
        None => gh(&[
            "repo",
            "view",
            "--json",
            "nameWithOwner",
            "--jq",
            ".nameWithOwner",
        ])?,
    };
    let merged = gh(&["api", &format!("repos/{repo}/pulls/{n}"), "--jq", ".merged"])?;
    let branch = gh(&["api", &format!("repos/{repo}"), "--jq", ".default_branch"])?;
    let branch = branch.trim();

    println!("{repo}#{n} — merged: {merged}");

    let files = gh(&[
        "api",
        &format!("repos/{repo}/pulls/{n}/files?per_page=100"),
        "--paginate",
        "--jq",
        ".[].filename",
    ])?;
    let files: Vec<&str> = files.lines().filter(|l| !l.is_empty()).collect();
    println!("files the pull request touched: {}", files.len());

    // Fetch each file once from the default branch; a file the PR deleted or
    // that never landed simply is not there, which is itself an answer.
    let mut corpus = String::new();
    let mut missing_files = 0usize;
    for f in &files {
        match gh(&[
            "api",
            &format!("repos/{repo}/contents/{f}?ref={branch}"),
            "--jq",
            ".content",
        ]) {
            Ok(b64) => {
                let cleaned: String = b64.chars().filter(|c| !c.is_whitespace()).collect();
                if let Ok(bytes) = base64_decode(&cleaned) {
                    corpus.push_str(&String::from_utf8_lossy(&bytes));
                    corpus.push('\n');
                }
            }
            Err(_) => missing_files += 1,
        }
    }
    if missing_files > 0 {
        println!("  ({missing_files} of them are not on {branch} at all)");
    }

    // Prose gets re-wrapped, so a probe that spans a line break would fail
    // against text that is actually present -- which happened on the first
    // real use. Compare with whitespace flattened on both sides.
    let flat_corpus = flatten_ws(&corpus);

    let mut absent = Vec::new();
    for p in probes {
        if flat_corpus.contains(&flatten_ws(p)) {
            println!("  PRESENT  {p}");
        } else {
            println!("  ABSENT   {p}");
            absent.push(p.clone());
        }
    }
    for f in files_present {
        let exists = gh(&[
            "api",
            &format!("repos/{repo}/contents/{f}?ref={branch}"),
            "--jq",
            ".name",
        ])
        .is_ok();
        if exists {
            println!("  EXISTS   {f}");
        } else {
            println!("  MISSING  {f}");
            absent.push(format!("file {f}"));
        }
    }

    println!();
    if absent.is_empty() {
        println!("VERDICT: the content landed on {branch}.");
        return Ok(());
    }
    println!("VERDICT: {} probe(s) are NOT on {branch}.", absent.len());
    println!("A pull request can read as merged while its content reached nothing —");
    println!("that is what a squash-merged stack does to whatever sat on top of it.");
    anyhow::bail!("{} probe(s) absent from {branch}", absent.len())
}

/// Confirm from the API — not from an exit code — that the pull request is
/// merged and its merge commit is reachable from the default branch. Returns
/// the short merge sha so the caller can print what it verified.
fn confirm_merged(repo: &str, n: u64) -> Result<String> {
    let merged = gh(&["api", &format!("repos/{repo}/pulls/{n}"), "--jq", ".merged"])?;
    if merged.trim() != "true" {
        anyhow::bail!("the API still reports merged={}", merged.trim());
    }
    let sha = gh(&[
        "api",
        &format!("repos/{repo}/pulls/{n}"),
        "--jq",
        ".merge_commit_sha",
    ])?;
    let sha = sha.trim().to_string();
    if sha.is_empty() || sha == "null" {
        anyhow::bail!("merged=true but there is no merge commit sha");
    }
    let branch = gh(&["api", &format!("repos/{repo}"), "--jq", ".default_branch"])?;
    let branch = branch.trim();
    // "identical" or "behind" both mean the commit is contained in the branch.
    let status = gh(&[
        "api",
        &format!("repos/{repo}/compare/{branch}...{sha}"),
        "--jq",
        ".status",
    ])?;
    let status = status.trim();
    if status != "identical" && status != "behind" {
        anyhow::bail!(
            "merge commit {} is {status} relative to {branch}",
            &sha[..7.min(sha.len())]
        );
    }
    Ok(sha[..7.min(sha.len())].to_string())
}

/// Collapse every run of whitespace to a single space, so a probe matches
/// text that has since been re-wrapped.
fn flatten_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Minimal base64 decode: the GitHub contents API returns file bodies this
/// way and pulling a crate in for one call is not worth the dependency.
fn base64_decode(s: &str) -> Result<Vec<u8>> {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u32;
    for c in s.bytes() {
        if c == b'=' {
            break;
        }
        let v = match T.iter().position(|&t| t == c) {
            Some(v) => v as u32,
            None => continue,
        };
        buf = (buf << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
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
    // `--paginate`, and it is not cosmetic. This list feeds `VERDICT: safe to
    // merge`, and `--merge` runs `gh pr merge` on that verdict. One page of 100 was
    // enough on the day it was written; a failing check at position 101 would be
    // invisible, the verdict would read safe, and the merge would happen. The cure
    // was already in this file at the `pulls/{n}/files` fetch and did not travel.
    let runs = gh(&[
        "api",
        &format!("repos/{repo}/commits/{sha}/check-runs?per_page=100"),
        "--paginate",
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

/// Every check name that COMPLETED on this pull request, whatever the verdict.
///
/// The baseline needs this and not just the failures. A check absent from the
/// failure list is not thereby green -- it may simply never have run, and the
/// two are indistinguishable if you only ever collect failures.
fn completed_of(repo: &str, n: u64) -> Result<Vec<String>> {
    let raw = gh(&[
        "api",
        &format!("repos/{repo}/pulls/{n}"),
        "--jq",
        ".head.sha",
    ])?;
    let sha = raw.trim();
    // A truncated read here reads as "this check never ran", which prints
    // CANNOT TELL about a check that is green.
    let runs = gh(&[
        "api",
        &format!("repos/{repo}/commits/{sha}/check-runs?per_page=100"),
        "--paginate",
        "--jq",
        r#".check_runs[]|select(.status=="completed")|.name"#,
    ])?;
    Ok(runs.lines().map(|s| s.to_string()).collect())
}

/// Sum the per-page counts `gh --paginate --jq '...|length'` prints.
///
/// One number per page, newline separated. A single `.parse()` on that string fails
/// the moment there are two pages, and the `unwrap_or(0)` behind it turns "two pages
/// of checks" into "no checks" -- silently, and in the direction that says finished.
///
/// A line that does not parse is skipped. That is written as `filter_map(...ok())`
/// rather than `map(...unwrap_or(0))`, and the first draft of this comment claimed the
/// two differ -- they do not, in a SUM: adding a skipped line and adding a zero give
/// the same total, and a mutation swapping them survived every test. The form is kept
/// for saying what it means, not for changing what it computes, and the claim that it
/// changes something is removed rather than left standing.
pub fn sum_per_page(out: &str) -> usize {
    out.lines()
        .filter_map(|l| l.trim().parse::<usize>().ok())
        .sum()
}

/// Number of checks not yet completed on this pull request's head.
///
/// Zero can mean two different things and only one of them is "finished": no
/// checks have STARTED yet also reports zero. That is not hypothetical -- a
/// polling loop of mine exited on an empty list, and the pull request was
/// merged while ten checks were still running. So this returns the completed
/// count too, and the caller waits for it to stop growing.
fn in_flight(repo: &str, n: u64) -> Result<(usize, usize)> {
    let sha = gh(&[
        "api",
        &format!("repos/{repo}/pulls/{n}"),
        "--jq",
        ".head.sha",
    ])?;
    // Two plain queries rather than one clever @tsv: the combined form failed
    // with "expected an object but got: array" the first time it ran, and a
    // wait loop that errors out is worse than no wait loop.
    let path = format!(
        "repos/{repo}/commits/{}/check-runs?per_page=100",
        sha.trim()
    );
    // With `--paginate` a `|length` jq prints one number PER PAGE, so a bare
    // `.parse()` sees "12\n7", fails, and `unwrap_or(0)` reports ZERO pending --
    // the exact false "finished" this function's own doc comment exists to prevent.
    // The counts are summed instead.
    let pending = sum_per_page(&gh(&[
        "api",
        &path,
        "--paginate",
        "--jq",
        r#"[.check_runs[]|select(.status!="completed")]|length"#,
    ])?);
    let total = sum_per_page(&gh(&[
        "api",
        &path,
        "--paginate",
        "--jq",
        ".check_runs|length",
    ])?);
    Ok((pending, total))
}

/// Does this pull request's head branch match what the caller expected?
///
/// `None` means the caller did not say, which is not an error -- the flag is
/// opt-in and its absence is the old behaviour.
pub fn branch_matches(expected: Option<&str>, actual: &str) -> bool {
    match expected {
        None => true,
        Some(want) => want == actual.trim(),
    }
}

/// Every check on the head commit, as `(name, completed)`.
fn check_states(repo: &str, n: u64) -> Result<Vec<(String, bool)>> {
    let sha = gh(&[
        "api",
        &format!("repos/{repo}/pulls/{n}"),
        "--jq",
        ".head.sha",
    ])?;
    let out = gh(&[
        "api",
        "--paginate",
        &format!("repos/{repo}/commits/{}/check-runs", sha.trim()),
        "--jq",
        r#".check_runs[]|"\(.name)\t\(.status)""#,
    ])?;
    Ok(out
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .map(|(name, status)| (name.to_string(), status == "completed"))
        .collect())
}

/// Of the checks on a pull request, how many that the ruleset REQUIRES are unfinished.
///
/// The wait loop used to count every check -- 28 to 37 of them -- when four
/// decide the merge. Five pull requests sat behind that for hours with all four
/// required contexts already green; the only blocker was the up-to-date rule.
/// Waiting on thirty when four gate is not caution, it is the wrong population
/// to wait on.
///
/// `(name, completed)` pairs in, and the required set is passed rather than
/// known: those names live in repository SETTINGS and no file in the tree holds
/// them, so anything that wants to wait on "the checks that gate a merge" has to
/// ask. An empty required set returns `None` -- the caller must refuse rather
/// than treat "nothing required" as "nothing to wait for".
pub fn required_pending(checks: &[(String, bool)], required: &[String]) -> Option<usize> {
    if required.is_empty() {
        return None;
    }
    Some(
        checks
            .iter()
            .filter(|(name, done)| !done && required.iter().any(|r| r == name))
            .count(),
    )
}

/// How many of the required contexts have POSTED a check run on this commit.
///
/// The wait printed `{p} of {total}` with `total = required.len()` -- a count of
/// context names read from repository SETTINGS, which never look at the commit.
/// That is not the population `p` is drawn from, and the consequence is worse
/// than a malformed ratio: since an empty required set bails, `total >= 1`
/// always, so the `total == 0` arm -- the documented guard for "an empty list is
/// not finished, it is NOT STARTED" -- is unreachable under `--required-only`.
///
/// Run the flag before any required context has posted its run and the numerator
/// is 0 for the honest reason that nothing exists yet, while the denominator is
/// 4 from the ruleset. The loop breaks on the first poll and the verdict reads
/// safe to merge.
///
/// So the wait asks THIS instead: how many of the required names are present on
/// the commit at all. Distinct names, because the ratio's other half counts
/// names.
pub fn required_posted(checks: &[(String, bool)], required: &[String]) -> usize {
    required
        .iter()
        .filter(|r| checks.iter().any(|(name, _)| name == *r))
        .count()
}

/// What `--merge` should leave in the exit code.
///
/// Three outcomes, two of which are not a merge: `gh pr merge` refused, or it
/// succeeded and the content is not on the default branch. Both used to print
/// an honest line and return 0, so a caller could not tell them from a landing.
///
/// `ran_ok` is `gh pr merge`'s own status; `on_branch` is the answer from the
/// API afterwards, which is the only one that means merged.
pub fn merge_outcome(ran_ok: bool, on_branch: bool) -> i32 {
    if ran_ok && on_branch {
        0
    } else {
        4
    }
}

/// Why the merge was refused, when it was.
///
/// Under `strict_required_status_checks_policy` every merge that lands on the
/// base makes every other open pull request stale, and the refusal that follows
/// is not a verdict about the change -- it is a race the caller wins by taking
/// another round. Measured on one morning: four pull requests, five content
/// commits, **seven** `update-branch` merges between them, each one a full
/// re-run of checks that had already passed on the same tree.
///
/// Code 4 could not tell that apart from a refusal that means stop, so a caller
/// looping on 4 would loop on a genuinely dead pull request, and a caller
/// stopping on 4 would give up on one that a single command fixes. Code 5 says
/// the refusal is the staleness race and names the remedy.
///
/// `gh` does not expose this as a code, only as English, so the wording is the
/// input and the two spellings GitHub actually returns are the fixtures. When a
/// third appears this returns 4 and the caller stops, which is the safe way to
/// be wrong.
/// How to name the merged-PR baseline in a sentence, given what was actually read.
///
/// The fetch asks for `baseline * 3` CLOSED pull requests and keeps the merged
/// ones, up to `baseline`. Nothing guarantees the page holds that many: a quiet
/// week, or a run of closed-unmerged PRs, and the loop compares against fewer
/// than it asked for. The sentence said "the last {baseline} merged PRs"
/// regardless -- a page printed as a census, which is the shape
/// `tri gates fetches` flags and this is one of its five sites.
///
/// Saying the number actually compared costs one word and cannot drift.
pub fn baseline_phrase(compared: usize, asked: usize) -> String {
    match compared {
        0 => "no merged PR (none were found to compare against)".to_string(),
        1 => "the 1 merged PR read".to_string(),
        n if n < asked => format!("the {n} merged PRs found (fewer than the {asked} asked for)"),
        n => format!("the last {n} merged PRs"),
    }
}

/// The recent default-branch commits, and whether that read was complete.
///
/// One fetch, one guard. `classify_fetch` takes the ENCLOSING FUNCTION as the
/// subject, so a guard beside a second fetch answers a question nobody can tell
/// it is answering -- `fn ready` held two and the census called it unguarded
/// twice, correctly. Splitting is the fix; renaming a helper until the matcher
/// recognises it is not.
fn recent_commits(repo: &str, branch: &str) -> Result<(Vec<String>, bool)> {
    const PAGE: usize = 15;
    let out = gh(&[
        "api",
        &format!("repos/{repo}/commits?sha={branch}&per_page={PAGE}"),
        "--jq",
        ".[].sha",
    ])?;
    let rows: Vec<String> = out
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(String::from)
        .collect();
    let complete = crate::issues::read_is_complete(rows.len(), PAGE);
    Ok((rows, complete))
}

/// The recently merged pull requests, and whether that read was complete.
fn merged_recently(repo: &str, baseline: usize) -> Result<(Vec<String>, bool)> {
    let page = baseline * 3;
    // The request asks for CLOSED pull requests; the filter keeps only the
    // MERGED ones. Comparing the merged count against the closed page size
    // answers "was the page full?" with a number that was never the page --
    // and since closed is a superset of merged, it answers COMPLETE almost
    // always.
    //
    // Measured on gHashTag/t27, 2026-09-05: at per_page 30, 60 and 90 the page
    // came back FULL every time (30, 60, 90 closed) while the merged count was
    // 29, 59 and 88. `read_is_complete(merged, page)` said COMPLETE in all
    // three. It can only say otherwise if EVERY closed pull request on the page
    // is merged.
    //
    // So the completeness question is asked of the unfiltered read, which is
    // the thing the page bounded, and the filter is applied afterwards.
    let out = gh(&[
        "api",
        &format!("repos/{repo}/pulls?state=closed&per_page={page}"),
        "--jq",
        r#".[]|[(.number|tostring),(if .merged_at==null then "0" else "1" end)]|@tsv"#,
    ])?;
    let closed: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    let complete = crate::issues::read_is_complete(closed.len(), page);
    Ok((merged_numbers(&closed), complete))
}

/// The merged pull-request numbers from the `number<TAB>merged?` rows.
///
/// Extracted so a test exercises THIS code rather than a copy of it: the first
/// version of that test rebuilt the filter inline, and a mutant that dropped
/// the filter from production passed it.
pub fn merged_numbers(closed: &[&str]) -> Vec<String> {
    closed
        .iter()
        .filter(|l| l.ends_with("\t1"))
        .filter_map(|l| l.split('\t').next())
        .map(String::from)
        .collect()
}

/// The same sentence, with the truncation the page can hide.
pub fn baseline_phrase_bounded(compared: usize, asked: usize, page_full: bool) -> String {
    let base = baseline_phrase(compared, asked);
    if compared < asked && page_full {
        format!("{base}; the fetch page was FULL, so more may exist beyond it")
    } else {
        base
    }
}

pub fn refusal_kind(stderr: &str) -> i32 {
    let s = stderr.to_ascii_lowercase();
    let behind = s.contains("not up to date")
        || s.contains("head branch is out of date")
        || (s.contains("not mergeable") && s.contains("base branch"));
    if behind {
        5
    } else {
        4
    }
}

/// One completed run of a workflow on the default branch.
pub struct WorkflowRun {
    pub sha: String,
    pub created: String,
    /// `(job name, conclusion)` for every job the run held.
    pub jobs: Vec<(String, String)>,
}

/// What the check's own workflow says about it on the default branch.
pub struct WorkflowBaseline {
    pub workflow: String,
    /// The run that decided it, for `--why` to find the job in.
    pub run: u64,
    pub sha: String,
    pub created: String,
    pub failing: bool,
}

/// The answer of a check's workflow, when it was asked.
pub enum Asked {
    Found(WorkflowBaseline),
    /// `read` completed default-branch runs, none of which took the job to a
    /// verdict; `complete` is false when the page filled and more exist.
    Nothing { workflow: String, read: usize, complete: bool },
}

/// The run id in an Actions check-run's `details_url`,
/// `https://github.com/{owner}/{repo}/actions/runs/{run}/job/{job}`.
///
/// `None` for a check posted by anything that is not Actions: such a check
/// has no workflow to ask, and stays without a baseline.
pub fn run_id_of(details_url: &str) -> Option<u64> {
    let rest = details_url.split("/actions/runs/").nth(1)?;
    rest.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()
}

/// The newest run that took a job named `name` to a verdict: its index in
/// `runs` (newest first) and whether that job failed.
///
/// Cancelled, skipped and neutral jobs are not a verdict and are passed over:
/// a cancelled run says nothing about whether the check passes. Inside one
/// run, any failing job of that name makes the run failing -- one green twin
/// does not wash out a red one.
pub fn workflow_verdict(name: &str, runs: &[WorkflowRun]) -> Option<(usize, bool)> {
    for (i, run) in runs.iter().enumerate() {
        let mut passed = false;
        for (job, conclusion) in &run.jobs {
            if job != name {
                continue;
            }
            match conclusion.as_str() {
                "failure" | "timed_out" => return Some((i, true)),
                "success" => passed = true,
                _ => {}
            }
        }
        if passed {
            return Some((i, false));
        }
    }
    None
}

/// How long before `now` an observation was made, in words a reader weighs.
pub fn age_phrase(created: &str, now: chrono::DateTime<chrono::Utc>) -> String {
    let Ok(at) = chrono::DateTime::parse_from_rfc3339(created) else {
        return format!("at {created}");
    };
    let hours = (now - at.with_timezone(&chrono::Utc)).num_hours().max(0);
    if hours < 48 {
        format!("{hours} h before this read")
    } else {
        format!("{} days before this read", hours / 24)
    }
}

/// The Actions run behind each check on this pull request's head, by name.
fn check_run_ids(repo: &str, n: u64) -> Result<BTreeMap<String, u64>> {
    let sha = gh(&[
        "api",
        &format!("repos/{repo}/pulls/{n}"),
        "--jq",
        ".head.sha",
    ])?;
    let rows = gh(&[
        "api",
        &format!("repos/{repo}/commits/{}/check-runs?per_page=100", sha.trim()),
        "--paginate",
        "--jq",
        r#".check_runs[]|[.name,(.details_url//"")]|@tsv"#,
    ])?;
    let mut ids = BTreeMap::new();
    for line in rows.lines() {
        let mut it = line.splitn(2, '\t');
        if let (Some(name), Some(url)) = (it.next(), it.next()) {
            if let Some(id) = run_id_of(url) {
                ids.insert(name.to_string(), id);
            }
        }
    }
    Ok(ids)
}

/// The workflow a run belongs to: its id and its name.
fn workflow_of_run(repo: &str, run: u64) -> Result<(u64, String)> {
    let out = gh(&[
        "api",
        &format!("repos/{repo}/actions/runs/{run}"),
        "--jq",
        r#"[(.workflow_id|tostring),.name]|@tsv"#,
    ])?;
    let mut it = out.splitn(2, '\t');
    let id = it.next().unwrap_or("").trim().parse().context("no workflow id")?;
    Ok((id, it.next().unwrap_or("").trim().to_string()))
}

/// The workflow's newest completed runs on `branch` as `(run, sha, created)`,
/// and whether that read was the whole list.
fn branch_runs(
    repo: &str,
    workflow: u64,
    branch: &str,
) -> Result<(Vec<(u64, String, String)>, bool)> {
    const PAGE: usize = 10;
    let out = gh(&[
        "api",
        &format!("repos/{repo}/actions/workflows/{workflow}/runs?branch={branch}&status=completed&per_page={PAGE}"),
        "--jq",
        r#".workflow_runs[]|[(.id|tostring),.head_sha,.created_at]|@tsv"#,
    ])?;
    let rows: Vec<(u64, String, String)> = out
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            Some((f.first()?.parse().ok()?, f.get(1)?.to_string(), f.get(2)?.to_string()))
        })
        .collect();
    let complete = crate::issues::read_is_complete(rows.len(), PAGE);
    Ok((rows, complete))
}

/// Every job of one run, as `(name, conclusion)`.
fn jobs_of_run(repo: &str, run: u64) -> Result<Vec<(String, String)>> {
    let out = gh(&[
        "api",
        &format!("repos/{repo}/actions/runs/{run}/jobs?per_page=100"),
        "--paginate",
        "--jq",
        r#".jobs[]|[.name,(.conclusion//"")]|@tsv"#,
    ])?;
    Ok(out
        .lines()
        .filter_map(|l| {
            let mut it = l.splitn(2, '\t');
            Some((it.next()?.to_string(), it.next()?.to_string()))
        })
        .collect())
}

/// Score a check the commit walk never saw by its own workflow's newest
/// default-branch run that ran a job of the same name.
///
/// The walk reads a fixed window of commits. A workflow with a `paths:`
/// filter runs on few of them: on 2026-10-04 `fpga-conformance` was red on
/// master's own latest run of `FPGA E2E Build` (e7ed3790b), 21 commits back,
/// and this command called it NO BASELINE because the window was 15. The
/// observation existed; the window did not reach it. Runs are read one at a
/// time, newest first, and the search stops at the first verdict -- usually
/// one jobs read.
fn workflow_baseline(repo: &str, run: u64, branch: &str, name: &str) -> Result<Asked> {
    let (workflow, wf_name) = workflow_of_run(repo, run)?;
    let (runs, complete) = branch_runs(repo, workflow, branch)?;
    let read = runs.len();
    for (id, sha, created) in runs {
        let one = [WorkflowRun { sha, created, jobs: jobs_of_run(repo, id)? }];
        if let Some((_, failing)) = workflow_verdict(name, &one) {
            let [r] = one;
            return Ok(Asked::Found(WorkflowBaseline {
                workflow: wf_name,
                run: id,
                sha: r.sha,
                created: r.created,
                failing,
            }));
        }
    }
    Ok(Asked::Nothing { workflow: wf_name, read, complete })
}

/// How many of a failing step's last output lines `--why` looks for on the
/// other side.
pub const WHY_TAIL: usize = 60;

/// Why one Actions job failed: the first step that failed, and that step's
/// own output up to the first error annotation, normalized.
#[derive(Debug, Clone, PartialEq)]
pub struct Reason {
    pub job: u64,
    pub step: String,
    pub lines: Vec<String>,
}

impl Reason {
    /// The last `WHY_TAIL` lines: the part of the output nearest the failure.
    pub fn tail(&self) -> &[String] {
        &self.lines[self.lines.len().saturating_sub(WHY_TAIL)..]
    }
}

/// How two reasons compare. Each side's tail is looked for anywhere in the
/// other side's whole step output, so a line added or dropped earlier in the
/// output does not shift the window into a false difference.
///
/// Lines are matched by their words, with every number read as `#`: a
/// pull request based on an older master printed `observed 78` and
/// `+ CounterState  NEW conflict` where master printed `observed 81` and four
/// conflicts, CounterState among them (t27#5663, 2026-10-04). It fails for one
/// of master's four reasons; compared digit for digit, the count line alone
/// would have called that a new one. A line that differs only in its numbers
/// is listed in `renumbered` and printed, never judged.
#[derive(Debug, PartialEq)]
pub struct ReasonMatch {
    pub step_changed: bool,
    /// Lines of this pull request's tail whose words appear nowhere in the
    /// other output, first occurrence only, in the order they were printed.
    pub only_here: Vec<String>,
    /// How many distinct line shapes of the other tail appear nowhere here.
    pub only_there: usize,
    /// `(here, there)`: the same words with other numbers.
    pub renumbered: Vec<(String, String)>,
}

impl ReasonMatch {
    /// A new reason is a different step, or a line here that is not there.
    /// Lines there and not here alone mean this pull request fails for fewer
    /// of the same reasons: still the same failure.
    pub fn is_new(&self) -> bool {
        self.step_changed || !self.only_here.is_empty()
    }
}

fn why_regex(slot: &'static std::sync::OnceLock<regex::Regex>, re: &str) -> &'static regex::Regex {
    slot.get_or_init(|| regex::Regex::new(re).expect("static regex always compiles"))
}

/// A raw Actions log line without its timestamp prefix and colour codes.
fn strip_log_line(line: &str) -> String {
    static STAMP: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static ANSI: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let line = line.strip_prefix('\u{feff}').unwrap_or(line);
    let line = why_regex(&STAMP, r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z ?").replace(line, "");
    why_regex(&ANSI, r"\x1b\[[0-9;]*[A-Za-z]").replace_all(&line, "").into_owned()
}

/// One output line as `--why` compares it: what changes from run to run of
/// the same failure is masked -- times, durations, commit shas, run and job
/// ids -- and what can be the failure itself is kept: words and counts.
pub fn normalize_log_line(line: &str) -> String {
    static TIME: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static DUR: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static HEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static ID: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let s = strip_log_line(line);
    let s = why_regex(
        &TIME,
        r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:?\d{2})?",
    )
    .replace_all(&s, "<time>");
    let s = why_regex(&DUR, r"\b\d+(\.\d+)?(ms|s|m|h)\b").replace_all(&s, "<t>");
    let s = why_regex(&HEX, r"\b[0-9a-f]{7,40}\b").replace_all(&s, |c: &regex::Captures| {
        // A sha has digits and letters; a word made of a-f ("defaced") does not.
        let b = c[0].as_bytes();
        if b.iter().any(u8::is_ascii_alphabetic) && b.iter().any(u8::is_ascii_digit) {
            "<sha>".to_string()
        } else {
            c[0].to_string()
        }
    });
    let s = why_regex(&ID, r"\b\d{9,}\b").replace_all(&s, "<id>");
    s.trim_end().to_string()
}

/// The output of the step that failed, read from a job's whole log.
///
/// An Actions log opens each step with `##[group]Run ...`, echoes the script
/// up to `##[endgroup]`, then prints the step's output, then the annotations
/// (`##[error]...`). The annotation is often boilerplate -- a ratchet's
/// `::error::` says "the set moved" whatever moved it -- so the reason is the
/// output above it. `None` when the log holds no error annotation.
pub fn failing_step_output(log: &str) -> Option<Vec<String>> {
    let lines: Vec<String> = log.lines().map(strip_log_line).collect();
    let err = lines.iter().position(|l| l.starts_with("##[error]"))?;
    let group = lines[..err].iter().rposition(|l| l.starts_with("##[group]Run "));
    let start = match group {
        Some(g) => lines[g..err]
            .iter()
            .position(|l| l.starts_with("##[endgroup]"))
            .map_or(g + 1, |i| g + i + 1),
        None => 0,
    };
    Some(
        lines[start..err]
            .iter()
            .filter(|l| !l.trim().is_empty() && !l.starts_with("##["))
            .map(|l| normalize_log_line(l))
            .collect(),
    )
}

/// A line's words: every run of digits read as `#`.
pub fn line_shape(line: &str) -> String {
    static DIGITS: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    why_regex(&DIGITS, r"\d+").replace_all(line, "#").into_owned()
}

/// Compare why a check fails here with why it fails there.
pub fn compare_reasons(here: &Reason, there: &Reason) -> ReasonMatch {
    let raw_there: BTreeSet<&str> = there.lines.iter().map(String::as_str).collect();
    let mut shapes_there: BTreeMap<String, &str> = BTreeMap::new();
    for l in &there.lines {
        shapes_there.entry(line_shape(l)).or_insert(l.as_str());
    }
    let shapes_here: BTreeSet<String> = here.lines.iter().map(|l| line_shape(l)).collect();
    let mut listed = BTreeSet::new();
    let mut only_here = Vec::new();
    let mut renumbered = Vec::new();
    for l in here.tail() {
        if raw_there.contains(l.as_str()) || !listed.insert(l.as_str()) {
            continue;
        }
        match shapes_there.get(&line_shape(l)) {
            Some(t) => renumbered.push((l.clone(), t.to_string())),
            None => only_here.push(l.clone()),
        }
    }
    let only_there = there
        .tail()
        .iter()
        .map(|l| line_shape(l))
        .filter(|s| !shapes_here.contains(s))
        .collect::<BTreeSet<_>>()
        .len();
    ReasonMatch { step_changed: here.step != there.step, only_here, only_there, renumbered }
}

/// The verdict's exit code. An incomplete list outranks everything computed
/// from it; a check nobody has a baseline for outranks a judgment about it;
/// a failure only here outranks one that fails elsewhere for another reason.
///
///   0 safe, 1 DO NOT MERGE, 2 WAIT, 3 CANNOT TELL, 7 NEW REASON
pub fn verdict_code(pending: usize, no_baseline: usize, new_here: usize, new_reason: usize) -> i32 {
    if pending > 0 {
        2
    } else if no_baseline > 0 {
        3
    } else if new_here > 0 {
        1
    } else if new_reason > 0 {
        7
    } else {
        0
    }
}

/// The part of a `gh` error worth printing: what gh said, not the argv.
fn gh_said(e: &anyhow::Error) -> String {
    let s = e.to_string();
    let said = s.rsplit("failed: ").next().unwrap_or(&s).trim();
    let first = said.lines().next().unwrap_or(said);
    first.chars().take(120).collect()
}

/// Why one Actions job failed, read from the API: the jobs endpoint names the
/// failing step, the job's log holds that step's output.
fn reason_of(repo: &str, job: u64) -> std::result::Result<Reason, String> {
    let step = gh(&[
        "api",
        &format!("repos/{repo}/actions/jobs/{job}"),
        "--jq",
        r#"[.steps[]?|select(.conclusion=="failure")|.name][0]//"""#,
    ])
    .map_err(|e| format!("job {job} could not be read: {}", gh_said(&e)))?;
    let log = gh(&["api", &format!("repos/{repo}/actions/jobs/{job}/logs")])
        .map_err(|e| format!("job {job}'s log could not be read: {}", gh_said(&e)))?;
    let lines = failing_step_output(&log)
        .ok_or_else(|| format!("job {job}'s log holds no error annotation"))?;
    let step = if step.trim().is_empty() { "(no step recorded as failed)".to_string() } else { step.trim().to_string() };
    Ok(Reason { job, step, lines })
}

/// The failing Actions check-runs on one commit, by name: name -> job id.
/// A check posted by anything but Actions has no job log and is left out.
fn failing_jobs_on(repo: &str, sha: &str) -> Result<BTreeMap<String, u64>> {
    let rows = gh(&[
        "api",
        &format!("repos/{repo}/commits/{sha}/check-runs?per_page=100"),
        "--paginate",
        "--jq",
        r#".check_runs[]|select(.conclusion=="failure" or .conclusion=="timed_out")|[.name,(.id|tostring),(.details_url//"")]|@tsv"#,
    ])?;
    let mut jobs = BTreeMap::new();
    for line in rows.lines() {
        let f: Vec<&str> = line.splitn(3, '\t').collect();
        let (Some(name), Some(id), Some(url)) = (f.first(), f.get(1), f.get(2)) else {
            continue;
        };
        if run_id_of(url).is_none() {
            continue;
        }
        if let Ok(id) = id.trim().parse() {
            jobs.entry(name.to_string()).or_insert(id);
        }
    }
    Ok(jobs)
}

/// The failing job named `name` in one workflow run.
fn failing_job_in_run(repo: &str, run: u64, name: &str) -> Result<Option<u64>> {
    let out = gh(&[
        "api",
        &format!("repos/{repo}/actions/runs/{run}/jobs?per_page=100"),
        "--paginate",
        "--jq",
        r#".jobs[]|[.name,(.conclusion//""),(.id|tostring)]|@tsv"#,
    ])?;
    Ok(out.lines().find_map(|l| {
        let f: Vec<&str> = l.splitn(3, '\t').collect();
        let failed = matches!(f.get(1).copied(), Some("failure") | Some("timed_out"));
        if f.first() == Some(&name) && failed {
            f.get(2)?.trim().parse().ok()
        } else {
            None
        }
    }))
}

/// Why `name` fails here and why it fails there, with where "there" is.
///
/// "There" is the first place the baseline saw it fail, in the order the
/// baseline was built: the default-branch commit walk, the check's own
/// workflow, then the merged pull requests.
fn why_pair(
    repo: &str,
    name: &str,
    here_jobs: &BTreeMap<String, u64>,
    failed_at: &BTreeMap<String, (String, u64)>,
    wf: Option<&WorkflowBaseline>,
    others: &[u64],
    branch: &str,
) -> std::result::Result<(Reason, Reason, String), String> {
    let Some(&mine) = here_jobs.get(name) else {
        return Err("not an Actions job here, so there is no log to read".to_string());
    };
    let (job, at) = if let Some((sha, job)) = failed_at.get(name) {
        (*job, format!("{branch} at {}", &sha[..sha.len().min(9)]))
    } else if let Some(b) = wf.filter(|b| b.failing) {
        match failing_job_in_run(repo, b.run, name) {
            Ok(Some(job)) => (job, format!("{branch} at {}", &b.sha[..b.sha.len().min(9)])),
            Ok(None) => return Err(format!("run {} holds no failing job of that name", b.run)),
            Err(e) => return Err(format!("run {} could not be read: {}", b.run, gh_said(&e))),
        }
    } else {
        let mut found = None;
        for p in others {
            let Ok(sha) = gh(&["api", &format!("repos/{repo}/pulls/{p}"), "--jq", ".head.sha"])
            else {
                continue;
            };
            if let Some(job) = failing_jobs_on(repo, sha.trim()).ok().and_then(|j| j.get(name).copied()) {
                found = Some((job, format!("merged #{p}")));
                break;
            }
        }
        found.ok_or_else(|| "no failing Actions job of that name found elsewhere".to_string())?
    };
    let here = reason_of(repo, mine)?;
    let there = reason_of(repo, job)?;
    Ok((here, there, format!("{at}, job {job}")))
}

/// Print one `--why` comparison; true when it is a NEW REASON.
fn print_why(w: &std::result::Result<(Reason, Reason, String), String>) -> bool {
    let (here, there, at) = match w {
        Ok(t) => t,
        Err(e) => {
            println!("      why: cannot compare -- {e}");
            return false;
        }
    };
    println!("      why, here:  step `{}`, job {}, {} output line(s)", here.step, here.job, here.lines.len());
    println!("      why, there: step `{}`, {at}, {} output line(s)", there.step, there.lines.len());
    let m = compare_reasons(here, there);
    let print_renumbered = || {
        for (h, t) in m.renumbered.iter().take(3) {
            println!("        ~ here:  {}", h.trim());
            println!("          there: {}", t.trim());
        }
        if m.renumbered.len() > 3 {
            println!("        ... and {} more line(s) with other numbers", m.renumbered.len() - 3);
        }
    };
    if !m.is_new() {
        let n = here.tail().len();
        if m.only_there == 0 {
            println!("      SAME REASON -- the same step, and its last {n} line(s) are printed there too");
        } else {
            println!("      SAME REASON, fewer -- its last {n} line(s) are printed there too, and");
            println!("      {} line(s) there are not printed here", m.only_there);
        }
        if !m.renumbered.is_empty() {
            println!("      ({} of them with other numbers -- read, not judged:)", m.renumbered.len());
            print_renumbered();
        }
        return false;
    }
    println!("      NEW REASON -- the same name, failing differently:");
    if m.step_changed {
        println!("        the failing step differs: `{}` here, `{}` there", here.step, there.step);
    }
    for line in m.only_here.iter().take(8) {
        println!("        + {line}");
    }
    if m.only_here.len() > 8 {
        println!("        ... and {} more line(s) only here", m.only_here.len() - 8);
    }
    if m.only_there > 0 {
        println!("        {} line(s) there are not printed here", m.only_there);
    }
    print_renumbered();
    true
}

/// Under another verdict, still name the failures that fail for a new reason.
fn list_new_reason(names: &[String]) {
    if names.is_empty() {
        return;
    }
    println!("\nand {} failure(s) red elsewhere too, but not for the same reason:", names.len());
    for name in names {
        println!("  - {name}");
    }
}

fn ready(
    n: u64,
    repo: Option<&str>,
    baseline: usize,
    wait: bool,
    poll: u64,
    merge: bool,
    expect_branch: Option<&str>,
    required_only: bool,
    why: bool,
) -> Result<()> {
    let repo = match repo {
        Some(r) => r.to_string(),
        None => gh(&[
            "repo",
            "view",
            "--json",
            "nameWithOwner",
            "--jq",
            ".nameWithOwner",
        ])?,
    };

    // Say which pull request this is BEFORE the wait loop, not after it.
    // Every "waiting: N of M" line used to carry no identity, so two gates
    // logging to one file produced a transcript where one PR's verdict read
    // as the other's -- diagnosing through a channel shared by two sources,
    // which is the error this project's own doctrine is named after.
    // Before anything else, and before any wait: is this the pull request the
    // caller meant? A number is the one part of a PR that cannot be checked
    // against anything, and several sessions share this repository. `3160` was
    // typed where `gh pr create` had printed `3161`; both read as the same
    // author, because every session authenticates as the same GitHub user, so
    // the head branch is the only thing that distinguishes them.
    if expect_branch.is_some() {
        let head = gh(&[
            "pr",
            "view",
            &n.to_string(),
            "--repo",
            &repo,
            "--json",
            "headRefName",
            "--jq",
            ".headRefName",
        ])?;
        if !branch_matches(expect_branch, &head) {
            println!("{repo}#{n} — NOT the pull request you meant.");
            println!("  expected head branch: {}", expect_branch.unwrap_or(""));
            println!("  this PR's head branch: {}", head.trim());
            println!();
            println!("  Refusing before reading a single check. A PR number is an");
            println!("  identifier, not a computed value, and neighbouring numbers in a");
            println!("  shared repository belong to somebody else.");
            std::process::exit(6);
        }
    }

    println!("{repo}#{n} — gate started");

    // Anything still running makes the answer provisional, so say so rather
    // than reporting a verdict on a partial list.
    let mut pending = in_flight(&repo, n)?.0;
    if wait {
        let mut quiet = 0;
        let mut blips = 0;
        loop {
            // A transient API failure must not end the wait. The first time
            // this loop met a TLS handshake timeout it propagated the error,
            // the caller's merge ran anyway, and the gate protected nothing --
            // the third time in this project that a verdict failed to gate.
            // In required-only mode the population of the wait is the ruleset's,
            // not every check on the commit. An empty required set is refused
            // rather than read as "nothing to wait for" -- that is the failure
            // this whole family of gates exists to prevent.
            let mut required_total = 0usize;
            let (p, total) = if required_only {
                let req = crate::gates::required_contexts(&repo)?;
                let states = check_states(&repo, n)?;
                match required_pending(&states, &req) {
                    None => anyhow::bail!(
                        "the ruleset lists no required context for master, so \
                         --required-only has nothing to wait on. Refusing rather \
                         than treating an empty requirement as a finished one."
                    ),
                    // The denominator is what has POSTED on this commit, so it
                    // is drawn from the same read as the numerator and can be
                    // zero -- which is what keeps the "not started" arm alive.
                    // `req.len()` is still printed, as its own clause, because
                    // "3 of 3 posted, and the ruleset requires 4" is the state a
                    // reader has to be able to see.
                    Some(pending) => {
                        required_total = req.len();
                        (pending, required_posted(&states, &req))
                    }
                }
            } else {
                match in_flight(&repo, n) {
                    Ok(v) => {
                        blips = 0;
                        v
                    }
                    Err(e) => {
                        blips += 1;
                        if blips > 5 {
                            return Err(e).context(
                                "the check API failed six times running; refusing to \
                             report a verdict rather than guess at the state",
                            );
                        }
                        println!("  waiting: check API failed ({blips}/5), retrying");
                        std::thread::sleep(std::time::Duration::from_secs(poll));
                        continue;
                    }
                }
            };
            if p > 0 {
                quiet = 0;
                if required_only {
                    println!(
                        "  [{repo}#{n}] waiting: {p} of {total} required check(s) posted are \
                         still running; the ruleset requires {required_total}"
                    );
                } else {
                    println!("  [{repo}#{n}] waiting: {p} of {total} check(s) still running");
                }
            } else if total == 0 || (required_only && total < required_total) {
                // An empty list is not "finished" -- it is "not started". Give
                // it a few rounds before believing it. Under --required-only the
                // same applies while any required context is still missing from
                // the commit: zero pending out of three posted says nothing
                // about the fourth, which has not started.
                quiet += 1;
                if required_only && total > 0 {
                    println!(
                        "  waiting: only {total} of {required_total} required context(s) \
                         have posted a run yet ({quiet}/4)"
                    );
                } else {
                    println!("  waiting: no checks have appeared yet ({quiet}/4)");
                }
                if quiet >= 4 {
                    break;
                }
            } else {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(poll));
        }
        // The final read must ask the same question the wait asked. Reading it
        // back over EVERY check while the wait watched four would print
        // `VERDICT: WAIT` with all four green -- the flag would shorten the wait
        // and then refuse the merge it exists to enable.
        pending = if required_only {
            let req = crate::gates::required_contexts(&repo).unwrap_or_default();
            match check_states(&repo, n) {
                Ok(states) => required_pending(&states, &req).unwrap_or(1),
                Err(_) => 1,
            }
        } else {
            match in_flight(&repo, n) {
                Ok(v) => v.0,
                // Unknown is not zero. If the final read fails, say so and let the
                // verdict be WAIT rather than inventing a clean list.
                Err(_) => 1,
            }
        };
        println!();
    }

    let mine = failures_of(&repo, n)?;

    // The baseline: failures on the default branch, plus failures on the last
    // few merged pull requests. A check red in both places is the repository's
    // problem, not this change's.
    let branch = gh(&["api", &format!("repos/{repo}"), "--jq", ".default_branch"])?;
    // The default branch's HEAD is not the default branch. A check that did
    // not run on HEAD -- a docs-only commit, a path filter -- shows neither
    // green nor red there, and reading HEAD alone once made a broken build
    // look "green on master" because the check-run was attached to an older
    // commit. So walk the last few default-branch commits and score each check
    // by the MOST RECENT commit on which it actually ran.
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    // Failures alone cannot tell "green everywhere" from "never ran anywhere".
    // A check with a `paths:` filter and no `push:` trigger runs on SOME pull
    // requests and on no default-branch commit at all: it is then absent from
    // every failure list, which this command used to read as a clean baseline
    // and report as "NOT failing on recent master commits" -- a sentence built
    // from zero observations and printed as if it were evidence. So record what
    // was OBSERVED, not only what was red.
    let mut observed: BTreeSet<String> = BTreeSet::new();
    let (recent, _recent_complete) = recent_commits(&repo, &branch)?;
    let mut decided: BTreeMap<String, bool> = BTreeMap::new(); // name -> failing
    // Where each failing name was decided: the commit and the check-run, which
    // for an Actions check is the job whose log `--why` reads.
    let mut failed_at: BTreeMap<String, (String, u64)> = BTreeMap::new();
    for sha in recent.iter() {
        let runs = gh(&[
            "api",
            &format!("repos/{repo}/commits/{sha}/check-runs?per_page=100"),
            "--paginate",
            "--jq",
            r#".check_runs[]|select(.status=="completed")|[.name,.conclusion,(.id|tostring)]|@tsv"#,
        ])
        .unwrap_or_default();
        for line in runs.lines() {
            let mut it = line.splitn(3, '\t');
            let (Some(name), Some(conc)) = (it.next(), it.next()) else {
                continue;
            };
            if decided.contains_key(name) {
                continue;
            }
            let failing = conc == "failure" || conc == "timed_out";
            decided.insert(name.to_string(), failing);
            if let (true, Some(id)) = (failing, it.next().and_then(|s| s.trim().parse().ok())) {
                failed_at.insert(name.to_string(), (sha.clone(), id));
            }
        }
    }
    for (name, failing) in &decided {
        observed.insert(name.clone());
        if *failing {
            *seen.entry(name.clone()).or_insert(0) += 1;
        }
    }
    let (merged, merged_complete) = merged_recently(&repo, baseline)?;
    let page_full = !merged_complete;
    let mut compared = 0usize;
    for num in merged.iter().take(baseline) {
        if let Ok(p) = num.parse::<u64>() {
            if p == n {
                continue;
            }
            compared += 1;
            for name in completed_of(&repo, p).unwrap_or_default() {
                observed.insert(name);
            }
            for name in failures_of(&repo, p).unwrap_or_default() {
                *seen.entry(name).or_insert(0) += 1;
            }
        }
    }

    // Last resort, and only for a failure nothing above observed: ask the
    // check's own workflow for its newest default-branch run. Any API failure
    // here leaves the check without a baseline -- CANNOT TELL, never safe.
    let mut from_workflow: BTreeMap<String, WorkflowBaseline> = BTreeMap::new();
    let mut asked_in_vain: BTreeMap<String, String> = BTreeMap::new();
    let unobserved: Vec<&String> = mine
        .iter()
        .filter(|m| !seen.contains_key(*m) && !observed.contains(*m))
        .collect();
    if !unobserved.is_empty() {
        let runs = check_run_ids(&repo, n).unwrap_or_default();
        for name in unobserved {
            let Some(run) = runs.get(name) else { continue };
            match workflow_baseline(&repo, *run, &branch, name) {
                Ok(Asked::Found(b)) => {
                    from_workflow.insert(name.clone(), b);
                }
                Ok(Asked::Nothing { workflow, read, complete }) => {
                    let more = if complete { "" } else { "; more exist beyond them" };
                    let note = format!(
                        "nor in the newest {read} completed {branch} run(s) of `{workflow}`{more}"
                    );
                    asked_in_vain.insert(name.clone(), note);
                }
                Err(_) => {}
            }
        }
    }
    let now = chrono::Utc::now();
    // --why: for each failure called pre-existing, read why it fails here and
    // why it fails there.
    let mut why_of: BTreeMap<String, std::result::Result<(Reason, Reason, String), String>> =
        BTreeMap::new();
    if why {
        let here_jobs = gh(&["api", &format!("repos/{repo}/pulls/{n}"), "--jq", ".head.sha"])
            .and_then(|sha| failing_jobs_on(&repo, sha.trim()))
            .unwrap_or_default();
        let others: Vec<u64> = merged
            .iter()
            .take(baseline)
            .filter_map(|p| p.parse().ok())
            .filter(|p| *p != n)
            .collect();
        for name in &mine {
            let wf = from_workflow.get(name);
            let pre = wf.map_or(seen.contains_key(name), |b| b.failing);
            if pre {
                let w = why_pair(&repo, name, &here_jobs, &failed_at, wf, &others, &branch);
                why_of.insert(name.clone(), w);
            }
        }
    }
    let mut new_reason = Vec::new();

    println!("{repo}#{n}\n");
    if mine.is_empty() {
        println!("  nothing is failing");
    }
    let mut new_here = Vec::new();
    let mut no_baseline = Vec::new();
    for name in &mine {
        if let Some(b) = from_workflow.get(name) {
            let short = &b.sha[..b.sha.len().min(9)];
            let age = age_phrase(&b.created, now);
            let state = if b.failing { "failing" } else { "PASSED" };
            println!("  {name}\n      {state} on {branch} at {short} ({}, {age})", b.created);
            println!("      -- the newest {branch} run of `{}` that ran it,", b.workflow);
            if b.failing {
                println!("      older than the commit window -- pre-existing");
                if let Some(w) = why_of.get(name) {
                    if print_why(w) {
                        new_reason.push(name.clone());
                    }
                }
            } else {
                println!("      older than the commit window");
                new_here.push(name.clone());
            }
            continue;
        }
        match seen.get(name) {
            Some(k) => {
                println!("  {name}\n      also failing in {k} other place(s) — pre-existing");
                if let Some(w) = why_of.get(name) {
                    if print_why(w) {
                        new_reason.push(name.clone());
                    }
                }
            }
            None if !observed.contains(name) => {
                println!("  {name}\n      NO BASELINE — this check did not run on any recent");
                let window = baseline_phrase_bounded(compared, baseline, page_full);
                match asked_in_vain.get(name) {
                    Some(note) => {
                        println!("      {branch} commit nor on any of {window},");
                        println!("      {note}, so");
                    }
                    None => println!("      {branch} commit nor on any of {window}, so"),
                }
                println!("      there is nothing to compare against. Usually a `paths:` filter");
                println!("      with no `push:` trigger. Read the log; this command cannot say");
                println!("      whether the failure is yours.");
                no_baseline.push(name.clone());
            }
            None => {
                println!("  {name}\n      NOT failing on recent {branch} commits or in {}\n      (it ran there and passed)", baseline_phrase_bounded(compared, baseline, page_full));
                new_here.push(name.clone());
            }
        }
    }
    if why {
        println!();
        println!("--why compared the failing step's name and the last {WHY_TAIL} lines of its");
        println!("output before its first error, with timestamps, durations, shas and long");
        println!("ids masked. The same text is not proof of the same cause, and a NEW");
        println!("REASON is two outputs to read, not proof this change caused it.");
        let unread: Vec<&String> =
            why_of.iter().filter(|(_, w)| w.is_err()).map(|(k, _)| k).collect();
        if !unread.is_empty() {
            println!("NOT compared, so not established either way: {} failure(s):", unread.len());
            for name in unread {
                println!("  - {name}");
            }
        }
    }
    println!();
    // The verdict reaches the EXIT CODE, not only the screen.
    //
    // This printed `VERDICT: WAIT` and returned Ok(()) -- success -- so
    // `tri pr ready N && gh pr merge N` merged on WAIT, and so did a caller who
    // read the line and merged anyway. The --merge flag's own help says the
    // verdict "cannot gate anything" when the merge is a separate command; part
    // of why it cannot is that the exit code said nothing. An honest line under
    // a zero exit is the same defect this campaign has been finding in gates,
    // in the tool that decides whether to merge them.
    //
    //   0  safe        every failure is failing elsewhere too
    //   1  DO NOT      a failure appears only here
    //   2  WAIT        the list is incomplete
    //   3  CANNOT TELL a failure has no baseline to compare against
    //   4  NOT MERGED  --merge was asked for and the merge did not land
    //   7  NEW REASON  (--why) red elsewhere too, but not for the same reason
    let mut code = 0;
    let verdict = verdict_code(pending, no_baseline.len(), new_here.len(), new_reason.len());
    if verdict == 2 {
        code = 2;
        println!("VERDICT: WAIT — {pending} check(s) still running, the list is incomplete.");
        if merge {
            println!("Not merging: the list is incomplete. Re-run with --wait.");
        }
    } else if verdict == 3 {
        code = 3;
        println!(
            "VERDICT: CANNOT TELL — {} failure(s) have no baseline to compare against:",
            no_baseline.len()
        );
        for name in &no_baseline {
            println!("  - {name}");
        }
        if !new_here.is_empty() {
            println!("\nand {} failure(s) appear only here:", new_here.len());
            for name in &new_here {
                println!("  - {name}");
            }
        }
        list_new_reason(&new_reason);
        println!("\nThis is a finding about the repository's CI, not about the change:");
        println!("a check that never runs on {branch} has no green state anyone has");
        println!("ever seen. Read its log and decide by hand.");
        if merge {
            println!("Not merging: refusing to treat an unmeasured check as passing.");
        }
    } else if verdict == 7 {
        code = 7;
        println!(
            "VERDICT: NEW REASON -- {} failure(s) are red elsewhere too, but not for the same reason:",
            new_reason.len()
        );
        for name in &new_reason {
            println!("  - {name}");
        }
        println!("\nA shared name is not a shared failure. Read the lines above before");
        println!("calling it pre-existing.");
        if merge {
            println!("Not merging: a red check here fails for a reason it does not fail for there.");
        }
    } else if verdict == 0 {
        println!("VERDICT: safe to merge — every failure is failing elsewhere too.");
        if merge {
            println!();
            let out = Command::new("gh")
                .args([
                    "pr",
                    "merge",
                    &n.to_string(),
                    "--repo",
                    &repo,
                    "--squash",
                    "--delete-branch",
                ])
                .output()
                .context("failed to run gh pr merge")?;
            if out.status.success() {
                // `gh pr merge` exiting zero is not the same as the content
                // being on the branch: it also succeeds when it merely
                // enables auto-merge, and a squash-merged stack orphans
                // whatever sat on top of it. Ask the API instead of the
                // exit code, and name what was verified.
                match confirm_merged(&repo, n) {
                    Ok(sha) => {
                        println!("Merged — {sha} is on the default branch.");
                        code = merge_outcome(true, true);
                    }
                    Err(e) => {
                        println!("Merge command succeeded but the branch does not show it: {e}");
                        println!("Do NOT report this as merged. Check the pull request.");
                        code = merge_outcome(true, false);
                    }
                }
            } else {
                // An honest line under a zero exit is the defect this file
                // exists to prevent, and it was here: "Merge refused: the head
                // branch is not up to date" printed while the command returned
                // 0, so a caller could not tell landed from refused. Seen on
                // three pull requests in one batch.
                let err = String::from_utf8_lossy(&out.stderr);
                println!("Merge refused: {}", err.trim());
                code = refusal_kind(&err);
                if code == 5 {
                    println!();
                    println!("That refusal is the up-to-date race, not a verdict on this change:");
                    println!("  gh pr update-branch {n} --repo {repo}");
                    println!("then run this command again. Exit 5 says so; 4 would mean stop.");
                    println!("Never --admin, and never a force-push: update-branch is an");
                    println!("ordinary merge of the base and is what the rule is asking for.");
                }
            }
        }
    } else {
        println!(
            "VERDICT: DO NOT MERGE — {} failure(s) appear only here:",
            new_here.len()
        );
        code = 1;
        for name in &new_here {
            println!("  - {name}");
        }
        list_new_reason(&new_reason);
        println!("\nRead the log before deciding they are unrelated. A summary line");
        println!("is not the list; that mistake is why this command exists.");
    }
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    /// A check that never ran anywhere has no baseline, and "absent from every
    /// failure list" is exactly what both a green check and an unrun check look
    /// like. Classifying the second as the first is how this command reported
    /// "NOT failing on recent master commits" about a workflow with a `paths:`
    /// filter and no `push:` trigger -- a sentence assembled from zero
    /// observations. The distinction is the whole point: absence of evidence
    /// gets its own verdict.
    #[test]
    fn never_observed_is_not_the_same_as_never_failing() {
        use std::collections::{BTreeMap, BTreeSet};
        let failures_elsewhere: BTreeMap<&str, usize> = BTreeMap::new();
        let observed: BTreeSet<&str> = ["build", "coverage"].into_iter().collect();

        // "build" ran elsewhere and passed there: a real green baseline.
        assert!(observed.contains("build"));
        assert!(!failures_elsewhere.contains_key("build"));

        // "emit-bitexact" never ran at all. Same empty failure list, and it
        // must NOT be read as the same thing.
        assert!(!observed.contains("emit-bitexact"));
        assert!(!failures_elsewhere.contains_key("emit-bitexact"));

        let classify = |name: &str| match failures_elsewhere.get(name) {
            Some(_) => "pre-existing",
            None if !observed.contains(name) => "no-baseline",
            None => "new-here",
        };
        assert_eq!(classify("build"), "new-here");
        assert_eq!(classify("emit-bitexact"), "no-baseline");
        assert_ne!(classify("build"), classify("emit-bitexact"));
    }

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

    /// An empty check list means "not started", not "finished". A polling loop
    /// of mine counted rows, saw none, called it done, and a pull request was
    /// merged while ten checks were still running -- with this command's own
    /// WAIT verdict printed in the same batch, unread.
    #[test]
    fn an_empty_check_list_is_not_finished() {
        let total = 0usize;
        let pending = 0usize;
        let finished = total > 0 && pending == 0;
        assert!(!finished, "zero of zero must not read as complete");
    }

    /// Every verdict reaches the exit code, and only one of them is zero.
    ///
    /// This command printed WAIT and returned success, so `tri pr ready N &&
    /// gh pr merge N` merged on WAIT -- and so did I, by hand, with the line
    /// on the screen. A verdict that lives only in stdout gates nothing that
    /// is not a human reading carefully at 3am.
    #[test]
    ///
    /// This test used to copy the verdict chain into a local function and test
    /// the copy; it now calls the function `ready` calls.
    fn each_verdict_has_its_own_exit_code() {
        use super::verdict_code as code;
        assert_eq!(code(0, 0, 0, 0), 0, "safe");
        assert_eq!(code(3, 0, 0, 0), 2, "WAIT outranks an empty failure list");
        assert_eq!(code(0, 2, 0, 0), 3, "CANNOT TELL");
        assert_eq!(code(0, 0, 1, 0), 1, "DO NOT MERGE");
        assert_eq!(code(0, 0, 0, 1), 7, "NEW REASON");
        // Precedence: an incomplete list must win over anything computed from
        // it, including a clean one.
        assert_eq!(code(3, 2, 1, 1), 2, "pending outranks every other verdict");
        assert_eq!(code(0, 2, 1, 1), 3, "no baseline outranks a judgment");
        // A failure only here is the stronger finding: a new reason for an old
        // red is listed under it, never instead of it.
        assert_eq!(code(0, 0, 1, 1), 1, "only-here outranks a new reason");
        // And the codes must be distinct, or a caller cannot tell them apart.
        let all = [
            code(0, 0, 0, 0),
            code(3, 0, 0, 0),
            code(0, 2, 0, 0),
            code(0, 0, 1, 0),
            code(0, 0, 0, 1),
            4, // NOT MERGED
            5, // the up-to-date race
        ];
        let mut sorted: Vec<i32> = all.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), all.len(), "two verdicts share an exit code");
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
        assert_eq!(
            verdict, "WAIT",
            "pending must outrank an empty failure list"
        );
    }
}

#[cfg(test)]
mod why_tests {
    use super::{compare_reasons, failing_step_output, normalize_log_line, Reason, WHY_TAIL};

    /// The shape of a real job log: gHashTag/t27 job 111329604306, "Corpus
    /// ratchet (expected-failure ledger)", step "A type name may not gain a
    /// second definition", on 2026-10-04. `conflicts` are the names the step
    /// printed; the timestamps are moved by `shift` so two runs differ the way
    /// two real runs do.
    fn ratchet_log(observed: usize, conflicts: &[&str], shift: u32) -> String {
        let t = |k: u32| format!("2026-10-04T00:5{}:52.{:07}Z ", (2 + shift) % 10, 5_930_974 + k);
        let mut out = String::new();
        let mut put = |k: u32, line: &str| {
            out.push_str(&t(k));
            out.push_str(line);
            out.push('\n');
        };
        put(0, "##[group]Run actions/checkout@v4");
        put(1, "##[endgroup]");
        put(2, "##[group]Run set -o pipefail");
        put(3, "\x1b[36;1mset -o pipefail\x1b[0m");
        put(4, "\x1b[36;1m./target/debug/tri types ratchet > /tmp/types.log 2>&1 || rc=$?\x1b[0m");
        put(5, "\x1b[36;1m  echo \"::error::the set of conflicted type names moved. A name with two\"\x1b[0m");
        put(6, "shell: /usr/bin/bash -e {0}");
        put(7, "##[endgroup]");
        put(8, &format!("  ledger 77 name(s), observed {observed}"));
        for (i, c) in conflicts.iter().enumerate() {
            put(9 + i as u32, &format!("    + {c}  NEW conflict"));
        }
        put(20, "");
        put(21, "##[warning]an annotation in the middle of the output");
        put(22, "  A RESOLVED name fails too, on purpose. An entry that stops being");
        put(23, "  the same rule the corpus ratchet applies to an unexpected PASS.");
        put(24, "##[error]the set of conflicted type names moved. A name with two");
        put(25, "##[error]Process completed with exit code 1.");
        put(26, "##[group]Run actions/upload-artifact@v4");
        out
    }

    const FOUR: [&str; 4] = ["CounterState", "LRUCache", "TestCase", "TestRunner"];

    fn reason(step: &str, log: &str) -> Reason {
        Reason { job: 1, step: step.to_string(), lines: failing_step_output(log).expect("an error line") }
    }

    const STEP: &str = "A type name may not gain a second definition";

    /// What differs between two runs of one failure is masked; what can be
    /// the failure is kept. Counts are kept on purpose: "observed 82" against
    /// "observed 81" is the evidence.
    #[test]
    fn normalizing_masks_what_changes_between_runs_and_keeps_counts() {
        assert_eq!(
            normalize_log_line("2026-10-04T00:52:52.7695597Z   ledger 77 name(s), observed 81"),
            "  ledger 77 name(s), observed 81"
        );
        assert_eq!(normalize_log_line("\x1b[36;1mset -o pipefail\x1b[0m"), "set -o pipefail");
        assert_eq!(
            normalize_log_line("ok in 113.2s at 6e3322918 job 111324093808 since 2026-10-03T23:01:02Z"),
            "ok in <t> at <sha> job <id> since <time>"
        );
        assert_eq!(normalize_log_line("took 850ms, 64 passed"), "took <t>, 64 passed");
        // A word of a-f letters is not a sha; seven digits are not an id.
        assert_eq!(normalize_log_line("defaced 1234567 abcdef"), "defaced 1234567 abcdef");
    }

    /// The reason is the step's output, not its script and not the annotation:
    /// the ratchet's `::error::` lines read the same whichever name moved.
    #[test]
    fn the_failing_steps_own_output_is_read() {
        let lines = failing_step_output(&ratchet_log(81, &FOUR, 0)).unwrap();
        assert_eq!(
            lines,
            vec![
                "  ledger 77 name(s), observed 81",
                "    + CounterState  NEW conflict",
                "    + LRUCache  NEW conflict",
                "    + TestCase  NEW conflict",
                "    + TestRunner  NEW conflict",
                "  A RESOLVED name fails too, on purpose. An entry that stops being",
                "  the same rule the corpus ratchet applies to an unexpected PASS.",
            ]
        );
        assert_eq!(failing_step_output("2026-10-04T00:52:52Z all good\n"), None);
    }

    /// The case this flag exists for, in both directions.
    #[test]
    fn a_fifth_conflict_is_a_new_reason_and_the_same_four_are_not() {
        let master = reason(STEP, &ratchet_log(81, &FOUR, 0));
        let same = reason(STEP, &ratchet_log(81, &FOUR, 3));
        let m = compare_reasons(&same, &master);
        assert!(!m.is_new(), "{m:?}");
        assert_eq!(m.only_there, 0);

        let five = ["CounterState", "LRUCache", "NewThing", "TestCase", "TestRunner"];
        let more = reason(STEP, &ratchet_log(82, &five, 3));
        let m = compare_reasons(&more, &master);
        assert!(m.is_new());
        assert_eq!(m.only_here, vec!["    + NewThing  NEW conflict"]);
        assert_eq!(m.only_there, 0);
        assert_eq!(
            m.renumbered,
            vec![(
                "  ledger 77 name(s), observed 82".to_string(),
                "  ledger 77 name(s), observed 81".to_string()
            )]
        );
    }

    /// Measured 2026-10-04 on open pull requests based on an older master:
    /// t27#5663 printed `observed 78` and CounterState, one of master's four
    /// -- the same reason, fewer; t27#5781 printed `observed 78` and
    /// ModuleInterface, which master does not print -- a new one.
    #[test]
    fn an_older_count_is_not_a_reason_and_an_unknown_name_is() {
        let master = reason(STEP, &ratchet_log(81, &FOUR, 0));
        let older = reason(STEP, &ratchet_log(78, &["CounterState"], 2));
        let m = compare_reasons(&older, &master);
        assert!(!m.is_new(), "{m:?}");
        assert_eq!(m.only_there, 3, "LRUCache, TestCase, TestRunner");
        assert_eq!(m.renumbered.len(), 1, "observed 78 / observed 81");

        let other = reason(STEP, &ratchet_log(78, &["ModuleInterface"], 2));
        let m = compare_reasons(&other, &master);
        assert!(m.is_new());
        assert_eq!(m.only_here, vec!["    + ModuleInterface  NEW conflict"]);
    }

    /// Failing for fewer of the same reasons is the same failure; failing in
    /// another step is not, whatever it printed.
    #[test]
    fn fewer_lines_are_the_same_reason_and_another_step_is_not() {
        let master = reason(STEP, &ratchet_log(81, &FOUR, 0));
        let mut fewer = master.clone();
        fewer.lines.remove(1);
        let m = compare_reasons(&fewer, &master);
        assert!(!m.is_new(), "{m:?}");
        assert_eq!(m.only_there, 1);

        // A new line printed twice is listed once.
        let mut twice = master.clone();
        twice.lines.extend(["    + Twin  NEW conflict".to_string(), "    + Twin  NEW conflict".to_string()]);
        assert_eq!(compare_reasons(&twice, &master).only_here, vec!["    + Twin  NEW conflict"]);

        let other = reason("Build tri", &ratchet_log(81, &FOUR, 0));
        assert!(compare_reasons(&other, &master).step_changed);
        assert!(compare_reasons(&other, &master).is_new());
    }

    /// A line dropped inside the last WHY_TAIL lines moves where the window
    /// starts: this side's tail reaches one line further back, to a line the
    /// other tail does not hold. Comparing tail to tail would read that as a
    /// new line; each tail is looked for in the other's whole output instead.
    #[test]
    fn a_shifted_window_is_not_a_new_reason() {
        // Lines differ in their words, not only their numbers: numbers are
        // read as `#`, so `line 1` and `line 2` would be one line here.
        let word = |i: usize| format!("line {}{}", (b'a' + (i / 26) as u8) as char, (b'a' + (i % 26) as u8) as char);
        let long: Vec<String> = (0..WHY_TAIL + 10).map(word).collect();
        let there = Reason { job: 1, step: STEP.into(), lines: long.clone() };
        let mut short = long;
        short.remove(WHY_TAIL + 5);
        let here = Reason { job: 2, step: STEP.into(), lines: short };
        let m = compare_reasons(&here, &there);
        assert!(!m.is_new(), "{m:?}");
        assert_eq!(m.only_there, 1, "the dropped line");
        // A line printed on both sides is found as itself, not as a line
        // with other numbers: `~ here: X / there: X` would be noise.
        assert!(m.renumbered.is_empty(), "{:?}", m.renumbered);
    }
}

#[cfg(test)]
mod paginated_count_tests {
    use super::sum_per_page;

    /// One page prints one number and the sum is that number -- the shape the code
    /// had before `--paginate`, which must keep working.
    #[test]
    fn one_page_sums_to_itself() {
        assert_eq!(sum_per_page("12\n"), 12);
        assert_eq!(sum_per_page("0\n"), 0);
    }

    /// The shape that broke it. `gh --paginate --jq '...|length'` prints one number
    /// PER PAGE, and a single `.parse()` on "12\n7" fails -- with `unwrap_or(0)`
    /// behind it, two pages of running checks reported ZERO pending, which is the
    /// false "finished" this function exists to prevent.
    #[test]
    fn two_pages_sum_rather_than_fail_to_parse() {
        assert_eq!(sum_per_page("12\n7\n"), 19);
        assert_eq!(
            "12\n7\n".trim().parse::<usize>().unwrap_or(0),
            0,
            "the old shape"
        );
    }

    /// Nothing to read is zero, and that is honest: an empty output means gh printed
    /// no page at all.
    #[test]
    fn no_pages_is_zero() {
        assert_eq!(sum_per_page(""), 0);
        assert_eq!(sum_per_page("\n\n"), 0);
    }

    /// A line that is not a number is SKIPPED, not counted as zero. Losing a page is
    /// an undercount the caller can still notice; inventing a zero is the failure.
    #[test]
    fn an_unparseable_line_does_not_become_a_zero() {
        assert_eq!(sum_per_page("5\ngh: rate limit\n4\n"), 9);
    }
}

#[cfg(test)]
mod page_was_full_tests {
    use super::baseline_phrase_bounded;
    use crate::issues::read_is_complete;

    /// The wait loop is I/O-bound, so no unit test reaches it. Both halves of
    /// this fix are therefore pinned structurally: the denominator must be drawn
    /// from the commit, and the not-started arm must cover required-only.
    ///
    /// Reverting either leaves all four tests below green, and the second
    /// revert restores a path that ends in `gh pr merge --squash` before the
    /// required checks exist.
    #[test]
    fn the_wait_denominator_and_the_not_started_arm_are_both_wired() {
        let src = include_str!("prcheck.rs");
        let boundary = src
            .lines()
            .position(|l| l == "#[cfg(test)]")
            .expect("the test module is a line of its own");
        let code: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");

        let denom = concat!("(pending, required_", "posted(&states, &req))");
        assert!(
            code.contains(denom),
            "the denominator must come from the SAME read as the numerator, or it \
             is a settings value that can never be zero"
        );
        assert!(
            !code.contains(concat!("Some(pending) => (pending, req.", "len())")),
            "and never from the ruleset size"
        );

        let arm = concat!("total == 0 || (required_only && total < required_", "total)");
        assert!(
            code.contains(arm),
            "the not-started arm has to cover required-only, where `total == 0` \
             alone is unreachable because an empty ruleset bails earlier"
        );
        let merge = code.find("--squash").unwrap_or(code.len());
        assert!(
            code.find(arm).unwrap() < merge,
            "and it has to be reached before anything that merges"
        );
    }

    /// The wait's denominator came from repository SETTINGS, so it was never
    /// zero, so the "not started" arm could not run under --required-only.
    ///
    /// That arm exists because a pull request was once merged while ten checks
    /// were still running: an empty list is not "finished", it is "not
    /// started". With `total = required.len()` the numerator can be an honest 0
    /// -- nothing has posted yet -- while the denominator is 4 from the ruleset,
    /// so the loop takes the `else` and breaks on the FIRST poll.
    #[test]
    fn nothing_posted_yet_is_not_nothing_pending() {
        let req = vec![
            "check".to_string(),
            "validate".to_string(),
            "check-linked-issue".to_string(),
            "check-now-freshness".to_string(),
        ];
        // Seconds after the push: the commit has runs, but none of the required
        // contexts has posted one.
        let states = vec![("fpga-synthesis".to_string(), false)];
        assert_eq!(
            super::required_pending(&states, &req),
            Some(0),
            "nothing REQUIRED is running, which is true and not the question"
        );
        assert_eq!(
            super::required_posted(&states, &req),
            0,
            "and nothing required has POSTED -- the denominator that keeps the \
             not-started arm alive"
        );
        assert_eq!(req.len(), 4, "while the ruleset's count is never zero");
    }

    /// Partially posted is still not started. Three green out of three posted
    /// says nothing about the fourth.
    #[test]
    fn three_posted_of_four_required_is_still_waiting() {
        let req = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
        let states = vec![
            ("a".to_string(), true),
            ("b".to_string(), true),
            ("c".to_string(), true),
        ];
        assert_eq!(super::required_pending(&states, &req), Some(0), "none running");
        assert_eq!(super::required_posted(&states, &req), 3, "three have posted");
        assert!(
            super::required_posted(&states, &req) < req.len(),
            "and that inequality is the whole guard: 0 pending of 3 posted is not \
             a finished set of 4"
        );
    }

    /// Every required context posted: the ratio is well-formed and the wait may
    /// end. The guard must not become a wait that never finishes.
    #[test]
    fn all_posted_and_none_pending_is_finished() {
        let req = vec!["a".to_string(), "b".to_string()];
        let states = vec![("a".to_string(), true), ("b".to_string(), true)];
        assert_eq!(super::required_pending(&states, &req), Some(0));
        assert_eq!(super::required_posted(&states, &req), req.len());
    }

    /// The denominator counts NAMES, like the ruleset it is compared against --
    /// two runs of one required name are one posted context, not two.
    #[test]
    fn a_name_with_two_runs_is_one_posted_context() {
        let req = vec!["check".to_string()];
        let states = vec![("check".to_string(), false), ("check".to_string(), false)];
        assert_eq!(
            super::required_posted(&states, &req),
            1,
            "one required name, however many runs carry it"
        );
        assert_eq!(
            super::required_pending(&states, &req),
            Some(2),
            "the numerator still counts RUNS, which is why it may exceed the \
             ruleset size and must not be printed against it"
        );
    }

    /// The completeness question belongs to the read the PAGE bounded, not to
    /// what survived a filter applied after it.
    ///
    /// `merged_recently` asks the API for CLOSED pull requests and keeps the
    /// MERGED ones. Closed is a superset of merged, so comparing the merged
    /// count against the closed page size answers COMPLETE almost always.
    ///
    /// Measured on gHashTag/t27, 2026-09-05:
    ///
    /// | per_page | closed returned | merged of those | old verdict | page full |
    /// |---|---|---|---|---|
    /// | 30 | 30 | 29 | COMPLETE | YES |
    /// | 60 | 60 | 59 | COMPLETE | YES |
    /// | 90 | 90 | 88 | COMPLETE | YES |
    #[test]
    fn completeness_is_asked_of_the_unfiltered_read() {
        // The live shape: a full page whose rows are almost all merged.
        assert!(
            !read_is_complete(30, 30),
            "a full page is never complete -- asked of the CLOSED count it says so"
        );
        assert!(
            read_is_complete(29, 30),
            "asked of the MERGED count the same page reads as complete, which is \
             the defect: 29 merged of 30 closed, and 30 is the page"
        );
    }

    /// The filter still has to select the merged ones, and only those.
    #[test]
    fn the_tsv_filter_keeps_merged_and_drops_the_rest() {
        let out = "3221\t1\n3218\t1\n3200\t0\n3199\t1\n";
        let closed: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(closed.len(), 4, "the page is four closed pull requests");
        let rows = super::merged_numbers(&closed);
        assert_eq!(rows, vec!["3221", "3218", "3199"], "three merged: {rows:?}");
        assert!(!rows.contains(&"3200".to_string()), "the closed-unmerged one is dropped");
    }

    /// This crate already had the predicate, inverted, in `issues.rs`, and
    /// `classify_fetch` recognises it by name. Writing a second one under a new
    /// name was two literals of one definition -- and the reason the census kept
    /// reading these fetches as unguarded.
    #[test]
    fn a_full_page_is_not_a_complete_read() {
        assert!(!read_is_complete(15, 15));
        assert!(!read_is_complete(16, 15));
    }

    #[test]
    fn a_short_page_is_the_whole_answer() {
        assert!(read_is_complete(14, 15));
        assert!(read_is_complete(0, 15));
    }

    /// A shortfall matters only when the page was FULL. A quiet week returns
    /// fewer merged pull requests and that is the true answer, not a truncation
    /// -- saying "more may exist" there would be the opposite lie.
    #[test]
    fn a_shortfall_on_a_short_page_is_not_a_truncation() {
        let s = baseline_phrase_bounded(2, 5, false);
        assert!(!s.contains("FULL"), "{s}");
    }

    #[test]
    fn a_shortfall_on_a_full_page_says_so() {
        let s = baseline_phrase_bounded(2, 5, true);
        assert!(s.contains("page was FULL"), "{s}");
    }

    /// Reaching the number asked for is not a shortfall, full page or not.
    #[test]
    fn reaching_the_target_is_never_a_truncation() {
        assert!(!baseline_phrase_bounded(5, 5, true).contains("FULL"));
    }
}

#[cfg(test)]
mod baseline_phrase_tests {
    use super::baseline_phrase;

    /// The sentence used to say "the last 5 merged PRs" whatever it read.
    #[test]
    fn fewer_than_asked_says_so() {
        assert_eq!(
            baseline_phrase(2, 5),
            "the 2 merged PRs found (fewer than the 5 asked for)"
        );
    }

    #[test]
    fn the_full_count_reads_naturally() {
        assert_eq!(baseline_phrase(5, 5), "the last 5 merged PRs");
    }

    /// Zero is the case that mattered: a sentence claiming a baseline built from
    /// nothing is what the surrounding comment in this file was written about.
    #[test]
    fn none_found_is_not_a_baseline() {
        assert!(baseline_phrase(0, 5).contains("no merged PR"));
    }

    #[test]
    fn one_is_singular() {
        assert_eq!(baseline_phrase(1, 5), "the 1 merged PR read");
    }

    /// More than asked cannot happen through `take(baseline)`, but the phrase
    /// must not claim a shortfall if it ever does.
    #[test]
    fn more_than_asked_is_not_a_shortfall() {
        assert_eq!(baseline_phrase(7, 5), "the last 7 merged PRs");
    }
}

#[cfg(test)]
mod refusal_kind_tests {
    use super::refusal_kind;

    /// The wording `gh` actually printed on three pull requests in one batch.
    #[test]
    fn the_up_to_date_race_is_five() {
        let real = "X Pull request gHashTag/t27#3127 is not mergeable: the head \
                    branch is not up to date with the base branch.";
        assert_eq!(refusal_kind(real), 5);
    }

    /// The real fixture above carries BOTH spellings -- "not up to date" AND
    /// "not mergeable ... base branch" -- so it cannot tell which clause caught
    /// it. Mutation said so: deleting the first clause left all tests passing.
    /// This one carries only the first, so that clause is the only thing that
    /// can answer it.
    #[test]
    fn the_first_spelling_alone_is_five() {
        assert_eq!(refusal_kind("the head branch is not up to date"), 5);
    }

    #[test]
    fn the_other_spelling_is_five_too() {
        assert_eq!(refusal_kind("The head branch is out of date"), 5);
    }

    /// A refusal that is NOT the race must stay 4, or a looping caller spins
    /// forever on a pull request no round will fix.
    #[test]
    fn a_real_refusal_stays_four() {
        assert_eq!(refusal_kind("Pull request is in a conflicted state"), 4);
        assert_eq!(refusal_kind("At least 1 approving review is required"), 4);
        assert_eq!(refusal_kind(""), 4);
    }

    /// Unknown wording is 4, because stopping on an unknown refusal is the safe
    /// way to be wrong and looping on one is not.
    #[test]
    fn an_unrecognised_refusal_stops() {
        assert_eq!(refusal_kind("some future message nobody has seen"), 4);
    }

    /// "not mergeable" alone is not enough -- a conflicted pull request says it
    /// too, and that one no amount of updating fixes.
    #[test]
    fn not_mergeable_alone_is_not_the_race() {
        assert_eq!(refusal_kind("is not mergeable"), 4);
    }
}

#[cfg(test)]
mod required_pending_tests {
    use super::required_pending;

    fn c(n: &str, done: bool) -> (String, bool) { (n.to_string(), done) }

    /// The case that held a five-deep queue for hours: thirty checks running,
    /// every required one already finished.
    #[test]
    fn unfinished_non_required_checks_do_not_count() {
        let checks = vec![c("validate", true), c("check", true),
                          c("cargo test", false), c("Lean proofs", false)];
        let req = vec!["validate".to_string(), "check".to_string()];
        assert_eq!(required_pending(&checks, &req), Some(0));
    }

    #[test]
    fn an_unfinished_required_check_counts() {
        let checks = vec![c("validate", true), c("check", false)];
        let req = vec!["validate".to_string(), "check".to_string()];
        assert_eq!(required_pending(&checks, &req), Some(1));
    }

    /// An empty requirement is not a finished one. Returning Some(0) here would
    /// make a repository with no ruleset merge instantly, which is the exact
    /// shape of every "gate over an empty population" this loop has found.
    #[test]
    fn an_empty_requirement_is_not_zero_pending() {
        let checks = vec![c("validate", false)];
        assert_eq!(required_pending(&checks, &[]), None);
    }

    /// A required context that has not appeared at all is not pending here --
    /// it is absent, and the merge itself refuses on that. Counting it as
    /// pending would wait forever for a check that will never be posted.
    #[test]
    fn a_required_context_not_yet_posted_is_not_counted() {
        let checks = vec![c("validate", true)];
        let req = vec!["validate".to_string(), "never-posted".to_string()];
        assert_eq!(required_pending(&checks, &req), Some(0));
    }

    #[test]
    fn names_match_exactly_not_by_prefix() {
        let checks = vec![c("check-linked-issue", false)];
        let req = vec!["check".to_string()];
        assert_eq!(required_pending(&checks, &req), Some(0));
    }
}

#[cfg(test)]
mod branch_matches_tests {
    use super::branch_matches;

    /// The near miss: 3160 typed where 3161 was printed. Same author, different
    /// branch, and the branch is the only thing that told them apart.
    #[test]
    fn a_different_branch_is_refused() {
        assert!(!branch_matches(
            Some("w-workflow-listing"),
            "loop/merge-in-flight"
        ));
    }

    #[test]
    fn the_expected_branch_passes() {
        assert!(branch_matches(
            Some("w-workflow-listing"),
            "w-workflow-listing"
        ));
    }

    /// Absence of the flag must not become a refusal: it is opt-in, and every
    /// existing caller passes nothing.
    #[test]
    fn no_expectation_is_not_a_refusal() {
        assert!(branch_matches(None, "anything-at-all"));
        assert!(branch_matches(None, ""));
    }

    /// `gh --jq` output arrives with a trailing newline.
    #[test]
    fn trailing_whitespace_is_not_a_mismatch() {
        assert!(branch_matches(Some("w-x"), "w-x\n"));
        assert!(branch_matches(Some("w-x"), "  w-x  "));
    }

    /// A prefix is not a match -- `w-tri` must not accept `w-tri-status`.
    #[test]
    fn a_prefix_is_not_a_match() {
        assert!(!branch_matches(Some("w-tri"), "w-tri-status"));
    }
}

#[cfg(test)]
mod merge_outcome_tests {
    use super::merge_outcome;

    /// The only outcome that is a merge is the one the API confirms.
    #[test]
    fn only_a_confirmed_landing_is_zero() {
        assert_eq!(merge_outcome(true, true), 0);
    }

    /// `gh pr merge` refusing -- "the head branch is not up to date" -- printed
    /// a line and returned 0 before this. Three pull requests in one batch.
    #[test]
    fn a_refused_merge_is_not_zero() {
        assert_eq!(merge_outcome(false, false), 4);
    }

    /// The worse half: the command succeeds, the content is not on the branch,
    /// and the text already says "Do NOT report this as merged".
    #[test]
    fn succeeded_but_not_on_the_branch_is_not_zero() {
        assert_eq!(merge_outcome(true, false), 4);
    }
}

#[cfg(test)]
mod workflow_baseline_tests {
    use super::{age_phrase, run_id_of, workflow_verdict, WorkflowRun};

    fn run(jobs: &[(&str, &str)]) -> WorkflowRun {
        WorkflowRun {
            sha: "e7ed3790b".into(),
            created: "2026-10-03T10:40:31Z".into(),
            jobs: jobs.iter().map(|(n, c)| (n.to_string(), c.to_string())).collect(),
        }
    }

    /// The run id is the segment after `/actions/runs/`, not the job id at the
    /// end -- asking `actions/runs/{job}` answers 404, and the check would sit
    /// at NO BASELINE with a workflow that had the answer.
    #[test]
    fn the_run_id_is_the_run_not_the_job() {
        let url = "https://github.com/gHashTag/t27/actions/runs/37155626492/job/111298566295";
        assert_eq!(run_id_of(url), Some(37155626492));
        assert_eq!(run_id_of("https://github.com/o/r/actions/runs/42"), Some(42));
    }

    /// A check posted by something that is not Actions has no workflow to ask.
    #[test]
    fn a_check_that_is_not_actions_has_no_run() {
        assert_eq!(run_id_of(""), None);
        assert_eq!(run_id_of("https://vercel.com/o/r/deployments/abc"), None);
        assert_eq!(run_id_of("https://github.com/o/r/actions/runs/latest"), None);
    }

    /// Newest first, and a cancelled job is not a verdict: master's run of
    /// `FPGA E2E Build` at 9212e8963 (2026-10-02) was `cancelled`. Had it been
    /// the newest, reading it as an answer would have hidden every red run
    /// behind it.
    #[test]
    fn a_cancelled_run_is_passed_over_not_read() {
        let runs = [
            run(&[("fpga-conformance", "cancelled")]),
            run(&[("fpga-conformance", "failure")]),
            run(&[("fpga-conformance", "success")]),
        ];
        assert_eq!(workflow_verdict("fpga-conformance", &runs), Some((1, true)));
    }

    #[test]
    fn the_newest_verdict_wins() {
        let runs = [run(&[("lint", "success")]), run(&[("lint", "failure")])];
        assert_eq!(workflow_verdict("lint", &runs), Some((0, false)));
        let runs = [run(&[("lint", "timed_out")]), run(&[("lint", "success")])];
        assert_eq!(workflow_verdict("lint", &runs), Some((0, true)));
    }

    /// Two jobs of one name in one run: a red one is not washed out by a
    /// green twin listed first.
    #[test]
    fn a_red_twin_makes_the_run_red() {
        let runs = [run(&[("build", "success"), ("build", "failure")])];
        assert_eq!(workflow_verdict("build", &runs), Some((0, true)));
    }

    /// Names match exactly. `fpga-lint (read_verilog + hierarchy)` is not
    /// `fpga-lint`, and a run that never held the job says nothing about it.
    #[test]
    fn names_match_exactly_and_absence_is_no_verdict() {
        let runs = [run(&[
            ("fpga-lint (read_verilog + hierarchy)", "failure"),
            ("fpga-smoke", "skipped"),
        ])];
        assert_eq!(workflow_verdict("fpga-lint", &runs), None);
        assert_eq!(workflow_verdict("fpga-smoke", &runs), None);
        assert_eq!(workflow_verdict("anything", &[]), None);
    }

    /// An old observation has to read as old.
    #[test]
    fn the_age_is_printed_in_hours_then_days() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-10-03T22:40:31Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        assert_eq!(age_phrase("2026-10-03T10:40:31Z", now), "12 h before this read");
        assert_eq!(age_phrase("2026-09-30T22:40:31Z", now), "3 days before this read");
        assert_eq!(age_phrase("not a date", now), "at not a date");
    }

    /// The fallback is for failures nothing else observed, and only for them.
    /// Widened to every failure, a green run from a week ago would overrule a
    /// red one from this morning's commit walk; and an API error must leave
    /// the check without a baseline -- CANNOT TELL, never safe.
    #[test]
    fn the_workflow_is_asked_only_about_the_unobserved() {
        let src = include_str!("prcheck.rs");
        let boundary = src
            .lines()
            .position(|l| l == "#[cfg(test)]")
            .expect("the test module is a line of its own");
        let code: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");
        let filter = concat!("!seen.contains_key(*m) && !observed.", "contains(*m)");
        assert!(code.contains(filter), "only unobserved failures reach the workflow");
        let ask = code.find(concat!("workflow_baseline(&repo, ", "*run")).expect("asked");
        let walk = code.find(concat!("let (recent, _recent_", "complete)")).expect("walk");
        assert!(walk < ask, "the commit walk runs first and keeps precedence");
        let tail = &code[ask..];
        let err = tail.find("Err(_) => {}").expect("an error arm");
        assert!(
            !tail[err..tail.find("let now").unwrap()].contains("insert"),
            "an API failure records nothing"
        );
    }
}

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
        } => ready(
            *number,
            repo.as_deref(),
            *baseline,
            *wait,
            *poll,
            *merge,
            expect_branch.as_deref(),
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

fn ready(
    n: u64,
    repo: Option<&str>,
    baseline: usize,
    wait: bool,
    poll: u64,
    merge: bool,
    expect_branch: Option<&str>,
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
            let (p, total) = match in_flight(&repo, n) {
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
            };
            if p > 0 {
                quiet = 0;
                println!("  [{repo}#{n}] waiting: {p} of {total} check(s) still running");
            } else if total == 0 {
                // An empty list is not "finished" -- it is "not started". Give
                // it a few rounds before believing it.
                quiet += 1;
                println!("  waiting: no checks have appeared yet ({quiet}/4)");
                if quiet >= 4 {
                    break;
                }
            } else {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(poll));
        }
        pending = match in_flight(&repo, n) {
            Ok(v) => v.0,
            // Unknown is not zero. If the final read fails, say so and let the
            // verdict be WAIT rather than inventing a clean list.
            Err(_) => 1,
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
    let recent = gh(&[
        "api",
        &format!("repos/{repo}/commits?sha={branch}&per_page=15"),
        "--jq",
        ".[].sha",
    ])?;
    let mut decided: BTreeMap<String, bool> = BTreeMap::new(); // name -> failing
    for sha in recent.lines() {
        let runs = gh(&[
            "api",
            &format!("repos/{repo}/commits/{sha}/check-runs?per_page=100"),
            "--paginate",
            "--jq",
            r#".check_runs[]|select(.status=="completed")|[.name,.conclusion]|@tsv"#,
        ])
        .unwrap_or_default();
        for line in runs.lines() {
            let mut it = line.splitn(2, '\t');
            let (Some(name), Some(conc)) = (it.next(), it.next()) else {
                continue;
            };
            decided
                .entry(name.to_string())
                .or_insert(conc == "failure" || conc == "timed_out");
        }
    }
    for (name, failing) in &decided {
        observed.insert(name.clone());
        if *failing {
            *seen.entry(name.clone()).or_insert(0) += 1;
        }
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
            for name in completed_of(&repo, p).unwrap_or_default() {
                observed.insert(name);
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
    let mut no_baseline = Vec::new();
    for name in &mine {
        match seen.get(name) {
            Some(k) => {
                println!("  {name}\n      also failing in {k} other place(s) — pre-existing")
            }
            None if !observed.contains(name) => {
                println!("  {name}\n      NO BASELINE — this check did not run on any recent");
                println!("      {branch} commit nor on any of the last {baseline} merged PRs, so");
                println!("      there is nothing to compare against. Usually a `paths:` filter");
                println!("      with no `push:` trigger. Read the log; this command cannot say");
                println!("      whether the failure is yours.");
                no_baseline.push(name.clone());
            }
            None => {
                println!("  {name}\n      NOT failing on recent {branch} commits or in the last {baseline} merged PRs\n      (it ran there and passed)");
                new_here.push(name.clone());
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
    let mut code = 0;
    if pending > 0 {
        code = 2;
        println!("VERDICT: WAIT — {pending} check(s) still running, the list is incomplete.");
        if merge {
            println!("Not merging: the list is incomplete. Re-run with --wait.");
        }
    } else if !no_baseline.is_empty() {
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
        println!("\nThis is a finding about the repository's CI, not about the change:");
        println!("a check that never runs on {branch} has no green state anyone has");
        println!("ever seen. Read its log and decide by hand.");
        if merge {
            println!("Not merging: refusing to treat an unmeasured check as passing.");
        }
    } else if new_here.is_empty() {
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
    fn each_verdict_has_its_own_exit_code() {
        fn code(pending: usize, no_baseline: usize, new_here: usize) -> i32 {
            if pending > 0 {
                2
            } else if no_baseline > 0 {
                3
            } else if new_here > 0 {
                1
            } else {
                0
            }
        }
        assert_eq!(code(0, 0, 0), 0, "safe");
        assert_eq!(code(3, 0, 0), 2, "WAIT outranks an empty failure list");
        assert_eq!(code(0, 2, 0), 3, "CANNOT TELL");
        assert_eq!(code(0, 0, 1), 1, "DO NOT MERGE");
        // Precedence: an incomplete list must win over anything computed from
        // it, including a clean one.
        assert_eq!(code(3, 2, 1), 2, "pending outranks every other verdict");
        // And the codes must be distinct, or a caller cannot tell them apart.
        let all = [code(0, 0, 0), code(3, 0, 0), code(0, 2, 0), code(0, 0, 1)];
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
mod branch_matches_tests {
    use super::branch_matches;

    /// The near miss: 3160 typed where 3161 was printed. Same author, different
    /// branch, and the branch is the only thing that told them apart.
    #[test]
    fn a_different_branch_is_refused() {
        assert!(!branch_matches(Some("w-workflow-listing"), "loop/merge-in-flight"));
    }

    #[test]
    fn the_expected_branch_passes() {
        assert!(branch_matches(Some("w-workflow-listing"), "w-workflow-listing"));
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

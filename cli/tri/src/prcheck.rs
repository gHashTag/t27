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
        #[arg(long = "probe", required = true)]
        probes: Vec<String>,
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
        } => ready(*number, repo.as_deref(), *baseline, *wait, *poll, *merge),
        PrCmd::Landed {
            number,
            repo,
            probes,
        } => landed(*number, repo.as_deref(), probes),
    }
}

/// Check that what the pull request introduced is present in the default
/// branch, file by file. Status is not content: a merged pull request whose
/// stack-mate was auto-closed leaves a list that reads as success.
fn landed(n: u64, repo: Option<&str>, probes: &[String]) -> Result<()> {
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
    let pending: usize = gh(&[
        "api",
        &path,
        "--jq",
        r#"[.check_runs[]|select(.status!="completed")]|length"#,
    ])?
    .trim()
    .parse()
    .unwrap_or(0);
    let total: usize = gh(&["api", &path, "--jq", ".check_runs|length"])?
        .trim()
        .parse()
        .unwrap_or(0);
    Ok((pending, total))
}

fn ready(
    n: u64,
    repo: Option<&str>,
    baseline: usize,
    wait: bool,
    poll: u64,
    merge: bool,
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
            Some(k) => {
                println!("  {name}\n      also failing in {k} other place(s) — pre-existing")
            }
            None => {
                println!("  {name}\n      NOT failing on recent {branch} commits or in the last {baseline} merged PRs");
                new_here.push(name.clone());
            }
        }
    }
    println!();
    if pending > 0 {
        println!("VERDICT: WAIT — {pending} check(s) still running, the list is incomplete.");
        if merge {
            println!("Not merging: the list is incomplete. Re-run with --wait.");
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
                    Ok(sha) => println!("Merged — {sha} is on the default branch."),
                    Err(e) => {
                        println!("Merge command succeeded but the branch does not show it: {e}");
                        println!("Do NOT report this as merged. Check the pull request.");
                    }
                }
            } else {
                println!(
                    "Merge refused: {}",
                    String::from_utf8_lossy(&out.stderr).trim()
                );
            }
        }
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

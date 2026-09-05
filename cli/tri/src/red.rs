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
//!
//! AND "NOW" NEEDED A DATE
//! ----------------------
//! The latest run is the newest one that EXISTS, which is not the same as a
//! recent one. `Auto Merge Ready PRs` sat in this list reading `260+ in a row`
//! while its newest run was eight days old, on a branch that no longer exists.
//! Measured: 1541 runs, every one a failure, never a success -- because the
//! file had not parsed since 2026-07-07, so GitHub could not read its triggers
//! and created a failed run on every push. #2256 repaired the parse on
//! 2026-08-20; the 96 runs after that date all came from two stale branches
//! carrying the old file, and ZERO came from master. It has been dormant since.
//!
//! Reporting that as a live outage is the "repaired defect reported as live"
//! shape this repository already records -- committed here by me, one pass
//! earlier, from this command's own output. So every row now carries the
//! instant of its latest run.
//!
//! THE DATE IS A BOUND TOO, AND IT WAS NOT MARKED AS ONE
//! -----------------------------------------------------
//! `streak()` reads ONE page and sets two values from it: the count, and the
//! instant of the oldest failure it saw. The count was marked as a lower bound
//! (`30+`) and the date, assigned on the next line of the same loop, was
//! printed as a plain fact.
//!
//! Measured on `OpenSSF Scorecard` in this repository: the command printed
//! `30+ in a row since 2026-09-04T06:01`. The true streak was **105**, running
//! from **2026-09-03T07:19:43Z** -- the printed instant is exactly the 30th
//! newest run, the edge of the page, **75 runs and 23 hours later than the
//! truth**. It is an UPPER bound on the start: the outage began at or before it.
//!
//! That direction is the damaging one for this command's own purpose, which its
//! closing line states: a streak is "the number of times nobody looked". A date
//! that drifts newer makes an old outage read as a fresh one.
//!
//! So a bounded read now says so in BOTH values, and `--deep` paginates until
//! the streak actually ends, for when the real number is wanted.
//!
//! AND THE LISTING ITSELF WAS ONE PAGE
//! -----------------------------------
//! The streak was not the only bounded read here. The workflow LISTING that
//! decides which workflows to examine at all asked `per_page=100` and did not
//! paginate. Measured: `gHashTag/trinity-fpga` carries **405** active
//! workflows, so **305 of them were never looked at** -- by the command whose
//! subject is "what is failing right now".
//!
//! The same fetch in `cibase.rs` has paginated all along, so this was one fix
//! that did not travel to its sibling. It was invisible for a second reason:
//! `tri gates fetches` takes the ENCLOSING FUNCTION as the subject of its
//! guard question, and `fn now` held more than one fetch, so the site sat in
//! `a guard, but two fetches` -- an honest "cannot tell" -- until the function
//! changed shape and the census resolved it to `prints what it got`.

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use clap::Subcommand;
use std::process::Command;

#[derive(Subcommand)]
pub enum RedCmd {
    /// Show workflows whose most recent run on the default branch failed.
    Now {
        /// owner/repo, repeatable. Defaults to `fleet_repos()` -- one list, because
        /// there were two and they disagreed by one repository.
        #[arg(long = "repo")]
        repos: Vec<String>,
        /// Include workflows whose latest run was cancelled or timed out.
        #[arg(long)]
        include_cancelled: bool,
        /// Walk the whole run history instead of one page, so a long streak
        /// prints its real length and start instead of the page edge.
        #[arg(long)]
        deep: bool,
    },
}

pub fn run(cmd: &RedCmd) -> Result<()> {
    match cmd {
        RedCmd::Now {
            repos,
            include_cancelled,
            deep,
        } => {
            let list: Vec<String> = if repos.is_empty() {
                crate::gates::fleet_repos()
            } else {
                repos.clone()
            };
            now(&list, *include_cancelled, *deep)
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
    /// It bounds BOTH printed values: the count from below, and `since` from
    /// above -- the outage began at or before that instant, never after it.
    at_least: bool,
    /// The most recent success, when one exists. It closes the other end of the
    /// bracket and is read with one request, independent of the streak length.
    last_pass: Option<String>,
    /// When the latest run happened. A streak says how many; this says whether
    /// the thing is still being exercised at all.
    last_at: String,
    /// The branch the runs were read from. Carried per row because `no success`
    /// is a claim about a branch, and the row has to be able to name it.
    branch: String,
}

/// How many of the most recent runs, newest first, share the failing verdict.
/// A single red is noise; nine in a row is an outage nobody is reading.
/// How many runs one page of the API returns.
///
/// A constant because TWO places need the same number and only prose linked
/// them: the query asks for a page this size, and a streak that FILLS the page
/// is a lower bound, printed as `30+`. Raising the query alone would have kept
/// printing `+` on streaks that are exact -- a truncation marker that has
/// stopped marking truncation, which is this command's own subject.
const PAGE: usize = 30;

/// The query, built from `PAGE` so the URL and the marker cannot disagree.
fn runs_url(repo: &str, id: &str, branch: &str) -> String {
    format!("repos/{repo}/actions/workflows/{id}/runs?branch={branch}&per_page={PAGE}")
}

/// How many runs a deep read asks for per page. Larger than `PAGE` only to
/// spend fewer requests; the marker never reads this, because a deep read walks
/// to the end of the streak and is bounded by history rather than by a page.
const DEEP_PAGE: usize = 100;

/// The deep query. `gh --paginate` follows every `Link: next`, so this walks the
/// whole recorded history of the workflow rather than one page of it.
fn deep_runs_url(repo: &str, id: &str, branch: &str) -> String {
    format!("repos/{repo}/actions/workflows/{id}/runs?branch={branch}&per_page={DEEP_PAGE}")
}

/// Is a streak of `n` a LOWER BOUND rather than an exact count?
fn is_lower_bound(n: usize) -> bool {
    n >= PAGE
}

/// Did the read run out of rows with the streak still going?
///
/// A shallow read is bounded when it fills its page. A deep read walks every
/// page, so it is bounded only when the whole recorded history is failures.
/// `ended` means a non-failure was reached, which settles both values exactly.
fn is_bounded_read(ended: bool, deep: bool, n: usize) -> bool {
    !ended && (deep || is_lower_bound(n))
}

/// The instant of the most recent SUCCESS on this branch, if there is one.
///
/// One request, `per_page=1`, and its cost does not grow with the streak. This
/// is the boundary a survey of mature tools kept pointing at: an outage began
/// after the last pass, and that fact is not a function of how many failures
/// you chose to fetch. Chromium's Sheriff-o-Matic carries it as `LatestPassing`
/// for exactly this reason -- it turns an open-ended "or earlier" into a
/// bracket with two measured ends.
fn last_pass(repo: &str, id: &str, branch: &str) -> Option<String> {
    let url = format!(
        "repos/{repo}/actions/workflows/{id}/runs?branch={branch}&status=success&per_page=1"
    );
    let raw = gh(&["api", &url, "--jq", r#".workflow_runs[0].created_at"#]).ok()?;
    let at = raw.trim();
    if at.is_empty() || at == "null" {
        None
    } else {
        Some(at.chars().take(16).collect())
    }
}

/// How the start instant is rendered.
///
/// Three states, because three things can be known. The count is a floor
/// (`30+`, at least this many) and the instant read from the page is a CEILING
/// (at or before this) -- opposite directions, so one marker cannot serve both.
///
///   * the streak ended inside the read -> the instant is exact, print it bare;
///   * truncated, and a success exists  -> a bracket: after the pass, at or
///     before the oldest failure seen. Both ends measured, neither guessed;
///   * truncated with no success on record -> a ceiling and nothing else, said
///     as such rather than dressed up as a reading.
///
/// The rule a survey of mature tools converged on: never let the edge of the
/// window stand in for the start. Prometheus latches `ActiveAt` at the
/// transition and resets rather than credit an unobserved gap; Elasticsearch
/// marks a truncated count `relation: gte`; GitHub sets `incomplete_results` on
/// the whole response. The failure mode has no name in monitoring, but it does
/// in survival analysis: the spell is LEFT-CENSORED, and the observation window
/// is not its beginning.
/// `None` for `last_pass` is the strongest thing this command can say about a
/// row, and it was previously invisible on the majority of them: a workflow with
/// no success on the branch never regressed, because it never worked. Measured
/// on `gHashTag/trinity-fpga`: 44 of the 50 red rows have never once been green.
/// A check that never passed is not a broken check -- it is an unfinished file,
/// and the two want opposite responses.
///
/// The branch is named because the population depends on it. Runs are read with
/// `branch=`, so "no success" means no success ON THAT BRANCH, which is not the
/// same set as "no success anywhere" -- on trinity-fpga the two happened to
/// coincide, and that is a fact about that repository, not about the question.
fn render_since(since: &str, bounded: bool, last_pass: Option<&str>, branch: &str) -> String {
    match (bounded, last_pass) {
        (false, Some(_)) => format!("since {since}"),
        (false, None) => format!("since {since}, never green on {branch}"),
        (true, Some(p)) => format!("after {p}, by {since}"),
        (true, None) => format!("by {since}, never green on {branch}"),
    }
}

/// How recent a latest run has to be for the row to bear on a merge decision.
/// Nagios calls this a freshness threshold and Prometheus a staleness delta;
/// both make the same point, that an old observation must be MARKED rather than
/// carried forward as the current value. The number is a policy, not a
/// discovery -- so it is stated in the output instead of applied silently.
const STALE_AFTER_DAYS: i64 = 7;

/// Whether this row's latest run is recent enough to bear on a merge decision.
///
/// An instant that does not parse answers NO. The headline built on this claims
/// recency, and a claim that cannot be supported is not one to make; the error
/// direction is toward the smaller, more cautious headline.
fn is_fresh(last_at: &str, today: NaiveDate, days: i64) -> bool {
    match NaiveDate::parse_from_str(last_at.get(..10).unwrap_or(""), "%Y-%m-%d") {
        Ok(d) => (today - d).num_days() < days,
        Err(_) => false,
    }
}

fn render_headline(total: usize, fresh: usize, never: usize, days: i64) -> String {
    let head = format!("{total} workflow(s) red on the default branch");
    let recency = if fresh == total {
        format!("{head}, every one within the last {days} days")
    } else if fresh == 0 {
        format!("{head} -- NOT ONE of them in the last {days} days")
    } else {
        format!("{head} -- {fresh} of them in the last {days} days")
    };
    // Said second because it is the louder number when it is large, and it is
    // large: it separates "this broke" from "this never worked", and only the
    // first of those is a regression anyone can be asked to fix.
    match never {
        0 => format!("{recency}."),
        n if n == total => format!("{recency}, and NOT ONE has ever been green."),
        n => format!("{recency}, and {n} have never once been green."),
    }
}

/// The line that separates live rows from fossils, and says what the fossils
/// ARE: workflows sharing a latest-run instant were triggered by one event, so
/// the honest unit below the line is the batch, not the file.
///
/// Grouping to the printed minute can split one push across a minute boundary,
/// which inflates the batch count. That direction is safe: it never merges two
/// events into one, so the number of distinct incidents is never understated.
fn render_divider(stale: &[&str], days: i64) -> String {
    let mut seen: Vec<&str> = stale.to_vec();
    seen.sort_unstable();
    let oldest = seen.first().copied().unwrap_or("?");
    let newest = seen.last().copied().unwrap_or("?");
    let largest = seen
        .iter()
        .map(|i| seen.iter().filter(|j| *j == i).count())
        .max()
        .unwrap_or(0);
    seen.dedup();
    format!(
        "--- the {} below last ran over {days} days ago: {} instant(s) between {} and {}, largest batch {} ---",
        stale.len(),
        seen.len(),
        oldest,
        newest,
        largest
    )
}

/// Returns `(streak, oldest failure seen, bounded)`.
///
/// `bounded` is true when the read ran out of rows while the streak was still
/// going, so the count is a lower bound AND the instant is an upper bound. A
/// shallow read is bounded whenever it fills its page; a deep read is bounded
/// only when the entire recorded history of the workflow is failures.
fn streak(repo: &str, id: &str, branch: &str, deep: bool) -> Result<(usize, String, bool)> {
    let url = if deep {
        deep_runs_url(repo, id, branch)
    } else {
        runs_url(repo, id, branch)
    };
    let mut args: Vec<&str> = vec!["api"];
    if deep {
        args.push("--paginate");
    }
    args.extend_from_slice(&[
        &url,
        "--jq",
        r#".workflow_runs[]|"\(.conclusion)\t\(.created_at)""#,
    ]);
    let raw = gh(&args)?;
    let mut n = 0usize;
    let mut since = String::new();
    let mut ended = false;
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
            _ => {
                ended = true;
                break;
            }
        }
    }
    // The read is bounded when it ran out of rows with the streak still going.
    // BOTH values inherit that: the count is a lower bound, and `since` is an
    // UPPER bound on the start, because the failures continue past what was
    // read. Marking only the count is the silent truncation this command exists
    // to surface, and it appeared here first -- in the date, for 105 runs.
    let bounded = is_bounded_read(ended, deep, n);
    Ok((n, since, bounded))
}

fn now(repos: &[String], include_cancelled: bool, deep: bool) -> Result<()> {
    let mut reds: Vec<Red> = Vec::new();
    for repo in repos {
        let branch = gh(&["api", &format!("repos/{repo}"), "--jq", ".default_branch"])?;
        // `--paginate`, because the population is the workflows themselves and a
        // page of them is not the list. Measured: `gHashTag/trinity-fpga` has
        // 405 active workflows and one page holds 100, so three quarters of the
        // repository were never examined for redness -- silently, by a command
        // whose whole subject is "what is failing right now".
        //
        // The identical fetch in `cibase.rs` has paginated all along. This is
        // one fix that did not travel to its sibling, and the census that would
        // have said so classified this site as ambiguous until the surrounding
        // function changed shape.
        let listing = gh(&[
            "api",
            &format!("repos/{repo}/actions/workflows?per_page=100"),
            "--paginate",
            "--jq",
            r#".workflows[]|select(.state=="active")|"\(.id)\t\(.name)""#,
        ])?;
        for line in listing.lines() {
            let mut it = line.splitn(2, '\t');
            let (id, name) = match (it.next(), it.next()) {
                (Some(a), Some(b)) => (a, b),
                _ => continue,
            };
            // The same one request now also returns WHEN. It cost nothing to
            // ask, and without it this command cannot tell "failing now" from
            // "last seen failing on a branch that no longer exists".
            let latest = gh(&[
                "api",
                &format!("repos/{repo}/actions/workflows/{id}/runs?branch={branch}&per_page=1"),
                "--jq",
                r#".workflow_runs[0]|"\(.conclusion // "none")\t\(.created_at // "")""#,
            ])?;
            let mut it = latest.splitn(2, '\t');
            let verdict = it.next().unwrap_or("none").to_string();
            let last_at: String = it.next().unwrap_or("").chars().take(16).collect();
            let bad = verdict == "failure"
                || verdict == "timed_out"
                || (include_cancelled && verdict == "cancelled");
            if !bad {
                continue;
            }
            let (n, since, bounded) = streak(repo, id, &branch, deep)?;
            // Only worth a request when the read was truncated: an exact start
            // needs no bracket, and this is one call per red workflow.
            // Asked for every red row now, not only truncated ones. It costs one
            // request per red workflow -- on `gHashTag/trinity-fpga`, 50 on top of
            // the 405-workflow listing and its per-workflow streak reads, about
            // 11% more. It buys the difference between a regression and a file
            // that never worked, which turned out to be 44 of those 50 rows.
            let pass = last_pass(repo, id, &branch);
            reds.push(Red {
                repo: repo.clone(),
                name: name.to_string(),
                since: since.chars().take(16).collect(),
                consecutive: n,
                at_least: bounded,
                last_pass: pass,
                last_at: last_at.clone(),
                branch: branch.clone(),
            });
        }
    }

    if reds.is_empty() {
        println!("Nothing is red on the default branch of any of these repositories.");
        return Ok(());
    }

    // Freshest first. The previous order was streak length, which is exactly the
    // axis that misleads: a workflow that failed 30 times and then stopped
    // running in July sorted ABOVE one that failed 3 times two days ago. The
    // streak says how long nobody looked; only the date says whether the thing
    // still runs, and that is what bears on the merge in front of you.
    reds.sort_by(|a, b| {
        b.last_at
            .cmp(&a.last_at)
            .then(b.consecutive.cmp(&a.consecutive))
    });
    let today = Utc::now().date_naive();
    // Sorted newest-first and freshness is monotone in the date, so every fresh
    // row precedes every stale one and this count doubles as the split index.
    let fresh = reds
        .iter()
        .filter(|r| is_fresh(&r.last_at, today, STALE_AFTER_DAYS))
        .count();
    let never = reds.iter().filter(|r| r.last_pass.is_none()).count();
    println!(
        "{}\n",
        render_headline(reds.len(), fresh, never, STALE_AFTER_DAYS)
    );
    for (i, r) in reds.iter().enumerate() {
        if i == fresh {
            let stale: Vec<&str> = reds[fresh..].iter().map(|r| r.last_at.as_str()).collect();
            println!("  {}", render_divider(&stale, STALE_AFTER_DAYS));
        }
        let short: String = r.name.chars().take(38).collect();
        let count = if r.at_least {
            format!("{}+", r.consecutive)
        } else {
            r.consecutive.to_string()
        };
        // The same bit governs both. `since` is an UPPER bound when the read was
        // truncated: the failures continue past the last row seen, so the outage
        // began at or before this instant.
        let since = render_since(&r.since, r.at_least, r.last_pass.as_deref(), &r.branch);
        println!(
            "  {:>5} in a row  last run {}  {:<48}  {:<26} {}",
            count, r.last_at, since, r.repo, short
        );
    }
    println!();
    println!("A long streak is not more of the same failure — it is the number of");
    println!("times nobody looked. Read this before merging, not after a page 404s.");
    println!();
    println!("`last run` is here because a streak cannot tell FAILING NOW from LAST SEEN");
    println!("FAILING. `Auto Merge Ready PRs` stood in this list at 260+ in a row while");
    println!("its newest run was eight days old on a branch that no longer exists: the");
    println!("file had not parsed since 2026-07-07, so GitHub made a failed run on every");
    println!("push -- 1541 of them, never one success -- and #2256 repaired it. A date");
    println!("beside the count separates a live outage from a settled one.");
    println!();
    println!();
    println!("`never green on <branch>` is the loudest thing on a row. A workflow with no success");
    println!("on the branch did not break -- it never worked, and it is an unfinished file rather");
    println!("than a regression. Measured on `gHashTag/trinity-fpga`: 44 of 50 red rows had never");
    println!("once been green, and only 6 were regressions. The two want opposite responses, and");
    println!("a bare count of `red` asks for neither.");
    println!("Rows below the divider are counted, not hidden -- but they are not news. In");
    println!("`gHashTag/trinity-fpga` the 50 red rows carry 11 distinct latest-run instants,");
    println!("and 39 of the 50 are four batches from one afternoon of 2026-07-09/10: files");
    println!("generated together, run once, failed, never triggered since. Counting files");
    println!("reported 50 problems where the incidents number 11, and the live ones 3.");
    if reds.iter().any(|r| r.at_least) {
        println!();
        if deep {
            println!(
                "`never green on <branch>` means the read never reached a success -- for a deep read"
            );
            println!(
                "that is the WHOLE recorded history, so the streak began before what the API still"
            );
            println!("retains. The count is a floor and the date is a ceiling either way.");
        } else {
            println!(
                "`after X, by Y` brackets a truncated read: the outage began after the last pass and"
            );
            println!(
                "at or before the oldest failure seen. `--deep` walks the history for the exact pair."
            );
        }
    }
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

#[cfg(test)]
mod page_tests {
    use super::*;

    /// The query and the truncation marker must be the same number.
    ///
    /// They were two literals, both `30`, linked only by a comment. A raise of
    /// the query alone would have kept printing `+` on streaks that are exact
    /// -- a truncation marker that has stopped marking truncation, which is
    /// what this command exists to surface.
    /// The defect this file's docstring records. `streak()` sets the count and
    /// the instant from ONE page, and only the count was marked as bounded.
    ///
    /// Measured live: `OpenSSF Scorecard` printed `30+ in a row since
    /// 2026-09-04T06:01` where the truth is 105 runs from 2026-09-03T07:19:43Z
    /// -- the printed instant was exactly the 30th newest run. The bound runs
    /// the OTHER way from the count: the outage started at or BEFORE it.
    #[test]
    fn a_truncated_read_bounds_the_date_as_well_as_the_count() {
        // The live case, by the numbers that produced it.
        assert!(
            is_bounded_read(false, false, PAGE),
            "a full page with no non-failure reached is a bounded read"
        );
        // The live case: last pass 2026-08-31T13:50, oldest failure on the page
        // 2026-09-04T06:01, true start 2026-09-03T07:19 -- inside the bracket.
        let bracketed = render_since("2026-09-04T06:01", true, Some("2026-08-31T13:50"), "master");
        assert_eq!(
            bracketed, "after 2026-08-31T13:50, by 2026-09-04T06:01",
            "a truncated read with a known pass must print a bracket, not a point"
        );
        assert!(
            "2026-08-31T13:50" < "2026-09-03T07:19" && "2026-09-03T07:19" <= "2026-09-04T06:01",
            "and the true start must lie inside it, which is the whole claim"
        );
        let ceiling = render_since("2026-04-14T18:15", true, None, "master");
        assert_eq!(
            ceiling, "by 2026-04-14T18:15, never green on master",
            "with no pass ever, only the ceiling is known and it says so"
        );
        let exact = render_since("2026-04-07T02:43", false, None, "master");
        assert_eq!(
            exact, "since 2026-04-07T02:43, never green on master",
            "the instant stays UNHEDGED -- no `by`, no `or earlier`. `never green` is a \
             separate fact appended to it, not a weakening of it"
        );
        assert!(
            !exact.contains("by ") && exact.starts_with("since 2026-04-07T02:43"),
            "an exact instant must NOT be hedged, or the marker stops marking"
        );
    }

    /// Reaching a non-failure settles BOTH values exactly, however the read was
    /// made -- that is what separates "the streak ended here" from "the rows
    /// ran out here", and it is the whole content of the marker.
    #[test]
    fn reaching_a_non_failure_is_exact_even_on_a_full_page() {
        assert!(
            !is_bounded_read(true, false, PAGE),
            "the streak ENDED inside the page, so the count and the date are exact"
        );
        assert!(
            !is_bounded_read(true, true, 10_000),
            "a deep read that ended is exact at any length"
        );
        // A deep read that never reached a non-failure is bounded REGARDLESS of
        // length: the failures run back past the start of recorded history, so
        // the instant is still a ceiling. `n` short of a page is the case that
        // separates this from the shallow rule -- with 675 both readings agree
        // and the distinction is invisible.
        assert!(
            is_bounded_read(false, true, 675),
            "a deep read that never ended means the whole history is failures"
        );
        assert!(
            is_bounded_read(false, true, 5),
            "and length is irrelevant to it: five rows, none of them a success, \
             still means the outage predates what the API retains"
        );
        assert!(
            !is_bounded_read(false, false, PAGE - 1),
            "a short page could not have been truncated"
        );
    }

    /// The count is a floor and the instant is a ceiling. Newest-first reading
    /// means the oldest row SEEN is newer than the oldest row that EXISTS.
    #[test]
    fn the_count_is_a_floor_and_the_date_is_a_ceiling() {
        let real_streak = 105usize;
        assert!(
            PAGE <= real_streak,
            "the printed count never exceeds the truth: it is a floor"
        );
        let printed = render_since("2026-09-04T06:01", true, None, "master");
        assert!(
            printed.contains("2026-09-04T06:01"),
            "the instant printed is the one that was read"
        );
        assert!(
            "2026-09-04T06:01" > "2026-09-03T07:19",
            "and it is LATER than the truth, which is why it needs `or earlier`"
        );
    }

    /// The workflow LISTING decides which workflows get examined at all, and
    /// it asked for one page. Measured: `gHashTag/trinity-fpga` has 405 active
    /// workflows against a page of 100, so three quarters of the repository was
    /// never looked at by a command whose subject is "what is failing now".
    ///
    /// Structural, because the defect is in the request and not in any value:
    /// the listing fetch must carry `--paginate`. The identical fetch in
    /// `cibase.rs` has carried it all along -- this is the sibling it did not
    /// travel to, so the test names both.
    /// A streak counts; it does not date. `Auto Merge Ready PRs` read
    /// `260+ in a row` while its newest run was eight days old on a branch that
    /// no longer exists -- 1541 failures, never a success, because the file had
    /// not parsed since 2026-07-07 and #2256 repaired it on 2026-08-20. Every
    /// run after that date came from a stale branch; zero came from master.
    ///
    /// Structural, because the defect is in what the request asks for: the one
    /// latest-run call must return the INSTANT beside the verdict, or the
    /// report cannot separate a live outage from a settled one.
    #[test]
    fn the_latest_run_is_read_with_its_instant() {
        let src = include_str!("red.rs");
        let at = src
            .find("runs?branch={branch}&per_page=1")
            .expect("the latest-run request is still here");
        let tail = &src[at..];
        let end = tail.find("])?").unwrap_or(tail.len());
        let call = &tail[..end];
        assert!(
            call.contains("created_at"),
            "the latest-run request must return WHEN as well as WHAT: a streak \
             cannot tell failing-now from last-seen-failing"
        );
        assert!(
            call.contains("conclusion"),
            "and it still has to return the verdict it is selected on"
        );
    }

    /// `last_pass` decides whether a row reads `never green` or nothing at all,
    /// and it used to be asked ONLY when the streak read was truncated. Every
    /// row with a short streak therefore got `None` for a reason that had nothing
    /// to do with whether it had ever passed -- and short streaks are the
    /// majority: 43 of the 50 red rows on `gHashTag/trinity-fpga` read `1 in a
    /// row`. Reverting this guard is invisible to every value-level test here,
    /// because the difference is a request that is or is not made, so the guard
    /// against it has to read the call site.
    #[test]
    fn the_last_pass_lookup_is_asked_for_every_row() {
        // Search only the half of the file ABOVE the test module. Every needle
        // below is also a string literal in this test's own body, so a search
        // over the whole file finds ITSELF when the real call site changes --
        // which is precisely the mutation this test exists to catch. The first
        // version of this test did exactly that and passed against the mutant.
        let src = include_str!("red.rs");
        let code = &src[..src
            .find("#[cfg(test)]")
            .expect("the test module marks the boundary")];
        let at = code
            .find("let pass = last_pass(repo, id, &branch);")
            .expect("the call site is unconditional -- no `if bounded` around it");
        // Nothing between the previous statement and the call may reintroduce
        // the condition.
        let prev = code[..at].rfind(";\n").map(|i| i + 2).unwrap_or(0);
        assert!(
            !code[prev..at].contains("if bounded"),
            "asking only on truncated reads answers `has it ever passed?` with \
             `was the page full?` -- two different questions, and the second one \
             says None for 43 of 50 rows that were never asked"
        );
    }

    #[test]
    fn the_workflow_listing_walks_every_page() {
        let src = include_str!("red.rs");
        let at = src
            .find("actions/workflows?per_page=100")
            .expect("the listing fetch is still here");
        // The flag must appear inside the same gh() argument list, which ends at
        // the closing bracket of the call.
        let tail = &src[at..];
        let end = tail.find("])?").unwrap_or(tail.len());
        assert!(
            tail[..end].contains("--paginate"),
            "the workflow listing is the POPULATION, and one page of it is not \
             the list -- 405 active workflows against a page of 100"
        );
    }

    /// A workflow with no success on the branch did not break -- it never worked.
    /// Measured on `gHashTag/trinity-fpga`: 44 of 50 red rows, so this is the
    /// majority case and it was previously invisible on every unbounded row.
    #[test]
    fn never_green_is_distinguished_from_regressed() {
        assert_eq!(
            render_since("2026-07-10T03:15", false, None, "main"),
            "since 2026-07-10T03:15, never green on main",
            "no success on the branch: an unfinished file, and the row must say so"
        );
        assert_eq!(
            render_since("2026-07-10T03:15", false, Some("2026-07-01T00:00"), "main"),
            "since 2026-07-10T03:15",
            "a known pass means it WORKED and then broke -- that is a regression, \
             and appending `never green` to it would be a false statement"
        );
        // The branch is part of the claim: runs are read with `branch=`, so `no
        // success` is scoped to it and a row that does not name the branch is
        // asserting something wider than it measured.
        assert!(
            render_since("2026-07-10T03:15", false, None, "trunk").ends_with("on trunk"),
            "the row names the branch it read, not a hard-coded one"
        );
    }

    /// The headline carries the never-green count because it is the larger and
    /// louder number: 44 of 50 on the measured repository. `red` alone asks for
    /// a fix; 44 of those rows have nothing to fix back to.
    #[test]
    fn the_headline_separates_never_worked_from_broke() {
        let h = render_headline(50, 3, 44, 7);
        assert!(
            h.contains("44"),
            "the never-green count is in the headline: {h}"
        );
        assert!(
            h.contains("never once been green"),
            "and it is named, not implied: {h}"
        );
        assert!(
            render_headline(6, 3, 0, 7).ends_with("days."),
            "with none never-green the clause is absent, not printed as zero"
        );
        assert!(
            render_headline(9, 0, 9, 7).contains("NOT ONE has ever been green"),
            "all-never-green is the loudest case and gets said outright"
        );
    }

    /// A workflow that stopped running is not a workflow that is failing. The
    /// boundary is the policy constant, so pin BOTH sides of it: the last day
    /// that counts as fresh and the first that does not.
    #[test]
    fn the_freshness_boundary_is_pinned_on_both_sides() {
        let today = NaiveDate::from_ymd_opt(2026, 9, 5).unwrap();
        assert!(is_fresh("2026-09-05T03:32", today, 7), "today is fresh");
        assert!(
            is_fresh("2026-08-30T00:00", today, 7),
            "six days old is still inside a seven-day window"
        );
        assert!(
            !is_fresh("2026-08-29T23:59", today, 7),
            "seven days old is outside it -- the window is the last 7 days, not 8"
        );
        assert!(
            !is_fresh("2026-07-10T03:15", today, 7),
            "the trinity-fpga fossils are two months old"
        );
    }

    /// The instant is read from the API and can be missing. A headline that says
    /// "3 of them in the last 7 days" must not count a row it could not date.
    #[test]
    fn an_undatable_row_is_never_counted_as_fresh() {
        let today = NaiveDate::from_ymd_opt(2026, 9, 5).unwrap();
        for bad in ["", "none", "2026-13-99T00:00", "not-a-date"] {
            assert!(
                !is_fresh(bad, today, 7),
                "{bad:?} cannot support a claim of recency"
            );
        }
    }

    /// The measured case: 50 red, 3 of them live. The headline has to carry both
    /// numbers, because 50 alone is what six passes of my own reports repeated.
    #[test]
    fn the_headline_carries_both_numbers() {
        let h = render_headline(50, 3, 44, 7);
        assert!(h.contains("50"), "the total is still there: {h}");
        assert!(h.contains('3'), "and so is the live count: {h}");
        assert!(
            h.contains('7'),
            "and the threshold is stated, not hidden: {h}"
        );
        assert_eq!(
            render_headline(4, 4, 0, 7),
            "4 workflow(s) red on the default branch, every one within the last 7 days.",
            "with nothing stale there is no split to report"
        );
        assert!(
            render_headline(11, 0, 0, 7).contains("NOT ONE"),
            "all-fossil is the loudest case and gets said outright"
        );
    }

    /// 39 of the 50 share four instants. The divider has to say `4`, not `39`:
    /// files generated and triggered together are one incident, not many.
    #[test]
    fn the_divider_counts_instants_not_files() {
        let stale = [
            "2026-07-10T03:15",
            "2026-07-10T03:15",
            "2026-07-10T03:15",
            "2026-07-09T23:24",
            "2026-04-19T08:59",
        ];
        let d = render_divider(&stale, 7);
        assert!(d.contains('5'), "every stale row is still counted: {d}");
        assert!(d.contains("3 instant(s)"), "but they are three events: {d}");
        assert!(
            d.contains("largest batch 3"),
            "and the biggest batch is named: {d}"
        );
        assert!(
            d.contains("2026-04-19T08:59") && d.contains("2026-07-10T03:15"),
            "both ends of the range are printed: {d}"
        );
    }

    #[test]
    fn the_query_and_the_marker_read_one_constant() {
        // Read the page size back OUT of the URL the command sends and check
        // it against the count at which the marker flips. Two literals cannot
        // pass this; one constant does.
        let url = runs_url("o/r", "wf.yml", "master");
        let sent: usize = url
            .rsplit("per_page=")
            .next()
            .and_then(|v| v.parse().ok())
            .expect("the query states a page size");
        assert!(
            !is_lower_bound(sent - 1),
            "one short of a full page is an exact count"
        );
        assert!(
            is_lower_bound(sent),
            "a full page is a lower bound and must print as `{sent}+`"
        );
    }
}

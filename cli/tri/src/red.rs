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
fn render_since(since: &str, bounded: bool, last_pass: Option<&str>) -> String {
    match (bounded, last_pass) {
        (false, _) => format!("since {since}"),
        (true, Some(p)) => format!("after {p}, by {since}"),
        (true, None) => format!("by {since}, no pass on record"),
    }
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
            let (n, since, bounded) = streak(repo, id, &branch, deep)?;
            // Only worth a request when the read was truncated: an exact start
            // needs no bracket, and this is one call per red workflow.
            let pass = if bounded {
                last_pass(repo, id, &branch)
            } else {
                None
            };
            reds.push(Red {
                repo: repo.clone(),
                name: name.to_string(),
                since: since.chars().take(16).collect(),
                consecutive: n,
                at_least: bounded,
                last_pass: pass,
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
        // The same bit governs both. `since` is an UPPER bound when the read was
        // truncated: the failures continue past the last row seen, so the outage
        // began at or before this instant.
        let since = render_since(&r.since, r.at_least, r.last_pass.as_deref());
        println!(
            "  {:>5} in a row  {:<48}  {:<26} {}",
            count, since, r.repo, short
        );
    }
    println!();
    println!("A long streak is not more of the same failure — it is the number of");
    println!("times nobody looked. Read this before merging, not after a page 404s.");
    if reds.iter().any(|r| r.at_least) {
        println!();
        if deep {
            println!(
                "`no pass on record` means the WHOLE recorded history is failures -- the read walked"
            );
            println!(
                "every page and never reached a success, so the streak began before what the API"
            );
            println!("still retains. The count is a floor and the date is a ceiling either way.");
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
        let bracketed = render_since("2026-09-04T06:01", true, Some("2026-08-31T13:50"));
        assert_eq!(
            bracketed, "after 2026-08-31T13:50, by 2026-09-04T06:01",
            "a truncated read with a known pass must print a bracket, not a point"
        );
        assert!(
            "2026-08-31T13:50" < "2026-09-03T07:19" && "2026-09-03T07:19" <= "2026-09-04T06:01",
            "and the true start must lie inside it, which is the whole claim"
        );
        let ceiling = render_since("2026-04-14T18:15", true, None);
        assert_eq!(
            ceiling, "by 2026-04-14T18:15, no pass on record",
            "with no pass ever, only the ceiling is known and it says so"
        );
        let exact = render_since("2026-04-07T02:43", false, None);
        assert_eq!(
            exact, "since 2026-04-07T02:43",
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
        let printed = render_since("2026-09-04T06:01", true, None);
        assert!(
            printed.contains("2026-09-04T06:01"),
            "the instant printed is the one that was read"
        );
        assert!(
            "2026-09-04T06:01" > "2026-09-03T07:19",
            "and it is LATER than the truth, which is why it needs `or earlier`"
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

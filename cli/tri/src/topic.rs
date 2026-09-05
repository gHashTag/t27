//! `tri topic` -- has someone already done this, or are they doing it now?
//!
//! Twice in one session I started work another session had shipped or had open.
//! `tri gates quiet` and `tri gates empty` (#3049) cover two of the four lenses a
//! read-only fan-out of mine was scanning at the same moment, and `tri skill
//! refs` (#3056) is the tool that would have caught a dangling cross-reference I
//! had just written a paragraph about. The second one landed on master while my
//! own branch was open on the same subject.
//!
//! Neither was expensive to discover afterwards and both were free to discover
//! BEFORE: the first was an OPEN pull request the whole time, and the second was
//! a merged commit. What was missing was one command instead of four.
//!
//! So: open pull requests, recent commits on the base branch, open issues, and
//! the section titles of every SKILL.md. A row matches when it carries any of
//! the keywords, and rows are ordered by how many DISTINCT keywords they carry,
//! because a row matching three of your words is the one you have to read.
//!
//! WHAT THIS DOES NOT DO. It reads titles and headlines, not bodies or diffs --
//! someone whose PR title does not name the subject is invisible here, and the
//! ordering is a word count, not an understanding. It is a two-second check
//! against a class of waste measured at two occurrences in one session, not a
//! search engine.
//!
//! And it refuses rather than reporting an empty result when `gh` cannot run:
//! "no one else is working on this" and "I could not ask" are the same empty
//! list, which is the defect this repository has spent the week naming.

use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

pub struct Row {
    pub origin: &'static str,
    pub label: String,
    pub hits: usize,
}

/// How many of `keywords` appear in `text`, case-insensitively.
///
/// Distinct keywords, not occurrences: a title repeating one word five times is
/// not a better match than one carrying two different words, and ranking by
/// occurrences would say otherwise.
/// What `gh` is asked for. Named so the request and the guard cannot drift: the
/// marker below compares against the same constant the URL was built from.
const PR_CAP: usize = 800;
const ISSUE_CAP: usize = 800;

/// Read, and say whether the read filled its cap.
///
/// A response of exactly `cap` rows is a LOWER BOUND, not a count -- there is no
/// way to tell a corpus of exactly `cap` from one of ten times that. Returning
/// the bit beside the rows is what lets the label say which of the two it has.
fn capped_read(args: &[&str], root: &std::path::Path, cap: usize) -> Result<(Vec<String>, bool)> {
    let out = gh(args, root)?;
    let lines: Vec<String> = out.lines().map(|l| l.to_string()).collect();
    let capped = lines.len() >= cap;
    Ok((lines, capped))
}

/// The parenthetical after `rows searched`.
///
/// It always named the commit window and never named the two API caps, so a
/// reader who saw one bound disclosed reasonably concluded the rest were not
/// bounded. Now every bound that BOUND is named, and one that did not bind is
/// not mentioned -- an unbound cap is not information, and printing it would
/// train the reader to skip the line.
fn render_scope(commits: usize, pr_capped: bool, issue_capped: bool) -> String {
    let pr = if pr_capped { "the first {PR_CAP} open PRs" } else { "open PRs" };
    let pr = pr.replace("{PR_CAP}", &PR_CAP.to_string());
    let iss = if issue_capped { "the first {ISSUE_CAP} open issues" } else { "open issues" };
    let iss = iss.replace("{ISSUE_CAP}", &ISSUE_CAP.to_string());
    let mut s = format!("({pr}, {iss}, last {commits} commits, every SKILL.md section)");
    if pr_capped || issue_capped {
        s.push_str("  <- A CAP WAS REACHED: this is a LOWER BOUND, raise it to see the rest");
    }
    s
}

pub fn hits(text: &str, keywords: &[String]) -> usize {
    let hay = text.to_lowercase();
    keywords
        .iter()
        .map(|k| k.to_lowercase())
        .filter(|k| !k.is_empty() && hay.contains(k.as_str()))
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

/// Rows carrying at least one keyword, most keywords first, then by origin and
/// label so the order is stable between runs.
pub fn rank(rows: Vec<Row>) -> Vec<Row> {
    let mut out: Vec<Row> = rows.into_iter().filter(|r| r.hits > 0).collect();
    out.sort_by(|a, b| {
        b.hits
            .cmp(&a.hits)
            .then(a.origin.cmp(b.origin))
            .then(a.label.cmp(&b.label))
    });
    out
}

/// `## 179. Title` -> `Title`, for every SKILL.md under `.claude/skills`.
pub fn skill_titles(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let dir = root.join(".claude/skills");
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return out;
    };
    for e in rd.flatten() {
        let p = e.path().join("SKILL.md");
        let Ok(text) = std::fs::read_to_string(&p) else {
            continue;
        };
        let name = e.file_name().to_string_lossy().to_string();
        for (n, t) in crate::skillnum::sections(&text) {
            out.push(format!("{name} §{n}. {t}"));
        }
    }
    out
}

fn gh(args: &[&str], root: &Path) -> Result<String> {
    let out = Command::new("gh").args(args).current_dir(root).output();
    match out {
        Err(_) => bail!(
            "tri topic could not run: `gh` is not on PATH.\n  \
             An empty answer here would read as \"nobody else is working on this\",\n  \
             which is the one thing this command exists to avoid saying wrongly."
        ),
        Ok(o) if !o.status.success() => bail!(
            "tri topic could not run: `gh {}` exited {:?}.\n  {}",
            args.join(" "),
            o.status.code(),
            String::from_utf8_lossy(&o.stderr).trim()
        ),
        Ok(o) => Ok(String::from_utf8_lossy(&o.stdout).to_string()),
    }
}

/// A keyword carrying whitespace, which is a quoting mistake nine times in ten.
///
/// Found on this command's SECOND use. `for q in "a b c"; do tri topic $q; done` in zsh does
/// not word-split, so the whole phrase arrives as ONE keyword, matches nothing,
/// and the command answers `rows matching 0` -- which reads as "nobody else is
/// working on this". That is the exact sentence this command exists to keep
/// from being said wrongly, produced by its own input handling.
pub fn quoted_phrase(keywords: &[String]) -> Option<&String> {
    keywords.iter().find(|k| k.trim().contains(char::is_whitespace))
}

pub fn run(keywords: &[String], commits: usize) -> Result<()> {
    if keywords.iter().all(|k| k.trim().is_empty()) {
        bail!("tri topic needs at least one keyword");
    }
    if let Some(k) = quoted_phrase(keywords) {
        bail!(
            "tri topic: {k:?} is one keyword carrying spaces, and it will match nothing.\n  \
             Pass the words separately: `tri topic {}`.\n  \
             Refused rather than answered, because \"0 rows matching\" here reads as\n  \
             \"nobody else is working on this\", which is what this command exists to\n  \
             keep from being said wrongly.",
            k.split_whitespace().collect::<Vec<_>>().join(" ")
        );
    }
    let root = crate::find_trinity_root()?;
    let mut rows: Vec<Row> = Vec::new();

    // The label below names the commit window ("last N commits") and used to
    // name no other bound, while the two `gh` reads carried caps of 100 and 200.
    // Measured 2026-09-05 on gHashTag/t27: 12 open PRs, so that cap was slack --
    // and 509 open issues, so the command searched 200 of them and reported
    // "rows searched 759 (open PRs, open issues, ...)" having never looked at
    // 309. Disclosing one of three bounds reads as disclosing all of them.
    let (pr_lines, pr_capped) = capped_read(
        &["pr", "list", "--state", "open", "--limit", &PR_CAP.to_string(), "--json",
          "number,title", "--jq", r##".[]|"#\(.number) \(.title)""##],
        &root,
        PR_CAP,
    )?;
    for line in pr_lines {
        rows.push(Row { origin: "open PR", label: line.clone(), hits: hits(&line, keywords) });
    }
    let (issue_lines, issue_capped) = capped_read(
        &["issue", "list", "--state", "open", "--limit", &ISSUE_CAP.to_string(), "--json",
          "number,title", "--jq", r##".[]|"#\(.number) \(.title)""##],
        &root,
        ISSUE_CAP,
    )?;
    for line in issue_lines {
        rows.push(Row { origin: "open issue", label: line.clone(), hits: hits(&line, keywords) });
    }

    let log = Command::new("git")
        .args(["log", "origin/master", "--oneline", "-n", &commits.to_string()])
        .current_dir(&root)
        .output()?;
    if !log.status.success() {
        bail!("tri topic could not run: `git log origin/master` failed -- fetch first");
    }
    for line in String::from_utf8_lossy(&log.stdout).lines() {
        rows.push(Row { origin: "recent commit", label: line.to_string(), hits: hits(line, keywords) });
    }

    for t in skill_titles(&root) {
        rows.push(Row { origin: "skill section", label: t.clone(), hits: hits(&t, keywords) });
    }

    let searched = rows.len();
    let found = rank(rows);
    println!();
    println!("  WHO ELSE HAS TOUCHED THIS: {}", keywords.join(", "));
    println!();
    println!("  rows searched   {searched}   {}", render_scope(commits, pr_capped, issue_capped));
    println!("  rows matching   {}", found.len());
    // The distribution, because "468 matching" out of 694 is not a result and
    // the ranking is what makes the list readable. A row carrying one of three
    // words is usually noise; a row carrying two is the one to open. Printed
    // rather than thresholded: a cutoff would need a number nobody has measured,
    // and the reader can see the shape in one line.
    let mut by_hits: std::collections::BTreeMap<usize, usize> = std::collections::BTreeMap::new();
    for r in &found {
        *by_hits.entry(r.hits).or_default() += 1;
    }
    let dist: Vec<String> = by_hits
        .iter()
        .rev()
        .map(|(h, n)| format!("{n} with {h}"))
        .collect();
    if !dist.is_empty() {
        println!("  by keyword count  {}", dist.join(", "));
    }
    println!();
    if found.is_empty() {
        println!("      nothing -- and every source above was actually read, which is");
        println!("      why this command refuses instead of printing an empty list");
        println!("      when `gh` cannot run.");
    }
    for r in found.iter().take(25) {
        println!("   {:>2}  {:<14} {}", r.hits, r.origin, r.label);
    }
    if found.len() > 25 {
        println!("\n      ... and {} more, not shown", found.len() - 25);
    }
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kw(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    /// The trap this command fell into on its own second use.
    #[test]
    fn a_keyword_carrying_spaces_is_refused_rather_than_answered() {
        assert!(quoted_phrase(&kw(&["gate absent quiet"])).is_some());
        assert!(quoted_phrase(&kw(&["gate", "absent", "quiet"])).is_none());
        // A single word padded with spaces is not a quoting mistake.
        assert!(quoted_phrase(&kw(&["  gate  "])).is_none());
        // And it must find the phrase even when other keywords are fine.
        assert_eq!(
            quoted_phrase(&kw(&["gate", "two words"])).map(|s| s.as_str()),
            Some("two words")
        );
    }

    #[test]
    fn distinct_keywords_are_counted_not_occurrences() {
        let k = kw(&["gate", "quiet"]);
        assert_eq!(hits("gates quiet -- steps whose pass survives", &k), 2);
        // Five occurrences of one word is still one keyword.
        assert_eq!(hits("gate gate gate gate gate", &k), 1);
        assert_eq!(hits("nothing relevant here", &k), 0);
    }

    #[test]
    fn matching_is_case_insensitive_and_substring() {
        assert_eq!(hits("Gates Quiet", &kw(&["gate", "QUIET"])), 2);
        assert_eq!(hits("cross-reference", &kw(&["reference"])), 1);
    }

    #[test]
    fn an_empty_keyword_matches_nothing_rather_than_everything() {
        // `contains("")` is true for every string; without the filter this
        // command would rank the entire repository as a match.
        assert_eq!(hits("anything at all", &kw(&["", "  "])), 0);
    }

    #[test]
    fn rows_with_no_hit_are_dropped_and_the_rest_are_ordered_by_hits() {
        let rows = vec![
            Row { origin: "open PR", label: "one word".into(), hits: 1 },
            Row { origin: "open PR", label: "no words".into(), hits: 0 },
            Row { origin: "open PR", label: "three words".into(), hits: 3 },
        ];
        let r = rank(rows);
        assert_eq!(r.len(), 2, "the zero-hit row must be dropped");
        assert_eq!(r[0].label, "three words", "most keywords first");
    }

    /// "Nobody else is working on this" and "I could not ask" are the same empty
    /// list, and this command exists to avoid saying the first when the second
    /// is true. Err either way here: a `gh` that is absent and a `gh` that
    /// refuses the subcommand both have to refuse, never return nothing.
    #[test]
    fn a_gh_that_cannot_answer_refuses_instead_of_returning_nothing() {
        let root = std::env::temp_dir();
        let e = super::gh(&["tri-topic-no-such-subcommand"], &root);
        assert!(e.is_err(), "an unanswerable gh must not produce an empty result");
        let msg = format!("{}", e.unwrap_err());
        assert!(
            msg.contains("could not run"),
            "and must say so in the words this repository uses: {msg}"
        );
    }

    /// The two cases this command was written for, replayed against the titles
    /// as they actually read. Both were visible before the work started.
    #[test]
    fn the_two_collisions_that_prompted_this_would_have_been_found() {
        let gates = "feat(tri): gates quiet -- the steps whose pass survives the subject going missing";
        assert!(
            hits(gates, &kw(&["gate", "absent", "pass"])) >= 2,
            "the open PR for the gates class must surface"
        );
        let refs = "feat(tri): skill refs -- every cross-reference, and whether it resolves";
        assert!(
            hits(refs, &kw(&["cross-reference", "section"])) >= 1,
            "the merged commit for the refs tool must surface"
        );
    }

    /// The parenthetical disclosed the commit window and hid two API caps.
    ///
    /// Measured 2026-09-05 on gHashTag/t27: 12 open PRs against a cap of 100, so
    /// that bound was slack -- and 509 open issues against a cap of 200, so the
    /// command searched 200 and printed "rows searched 759 (open PRs, open
    /// issues, ...)" having never looked at 309 of them. Raising both caps to
    /// 800 took the same command from 759 rows to 1068, and matches from 535 to
    /// 569.
    #[test]
    fn a_cap_that_bound_is_named_and_one_that_did_not_is_not() {
        let clean = render_scope(40, false, false);
        assert_eq!(
            clean,
            "(open PRs, open issues, last 40 commits, every SKILL.md section)",
            "with nothing truncated the line reads exactly as it always did"
        );
        assert!(
            !clean.contains("LOWER BOUND"),
            "an unbound cap is not information, and printing it teaches the reader to skip the line"
        );

        let hit = render_scope(40, false, true);
        assert!(
            hit.contains(&format!("the first {ISSUE_CAP} open issues")),
            "a cap that BOUND has to appear where the population is named: {hit}"
        );
        assert!(
            hit.contains("LOWER BOUND"),
            "and the count beside it is a floor, which the reader must be told: {hit}"
        );
        assert!(
            hit.contains("open PRs") && !hit.contains(&format!("the first {PR_CAP} open PRs")),
            "while the half that was NOT truncated is still named plainly: {hit}"
        );

        assert!(
            render_scope(40, true, false).contains(&format!("the first {PR_CAP} open PRs")),
            "and the same holds with the halves the other way round"
        );
    }

    /// `capped_read` compares against the constant the request was built from.
    /// Two literals for one number is how a raised cap and an unraised guard
    /// come to disagree silently.
    #[test]
    fn the_request_and_the_marker_read_one_constant() {
        let src = include_str!("topic.rs");
        let code = &src[..src.find("#[cfg(test)]").expect("the test module bounds the search")];
        assert!(
            code.contains("&PR_CAP.to_string()") && code.contains("&ISSUE_CAP.to_string()"),
            "the URL is built FROM the constant, not from a literal beside it"
        );
        assert!(
            code.contains("lines.len() >= cap"),
            "and the marker compares against the cap that was passed, not a second literal"
        );
    }
}

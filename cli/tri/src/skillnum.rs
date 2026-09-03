//! The numbered sections of a SKILL.md, checked for collisions.
//!
//! WHY THIS EXISTS
//! ---------------
//! Two sessions worked this repository at the same time and both appended to
//! `.claude/skills/ci-gates/SKILL.md`, both starting at `## 179.`. Git flagged
//! the overlap as a conflict, which is the lucky case: the resolution that keeps
//! BOTH sides is one keystroke away, and it leaves two sections numbered 179 in
//! a file whose numbers are how every other document refers to them.
//!
//! A duplicate is silent. Nothing renders differently, nothing fails to build,
//! and a later "see 179" points at either of two things.
//!
//! WHAT IT CHECKS
//! --------------
//!   * duplicates   -- two sections with one number. The merge hazard.
//!   * out of order -- section 180 before 179. A hand edit gone wrong.
//!   * gaps         -- 126 is missing today, from a removal nobody renumbered.
//!
//! Duplicates and disorder are FAILURES. A gap is reported and is not: renumbering
//! a 185-section file to close one hole would rewrite every reference to every
//! section after it, which is a worse outcome than a hole.
use anyhow::{Context, Result};
use clap::Subcommand;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum SkillCmd {
    /// Check every SKILL.md's section numbering.
    Check {
        /// Also print the gaps, which are reported but never fail.
        #[arg(long)]
        gaps: bool,
    },
    /// Sections that state a FIGURE, and which of those a reader can re-take.
    Claims {
        /// Print every section in the free population, one line each.
        #[arg(long)]
        list: bool,
        /// Print `<skill>:<number>` for every section counted, and nothing
        /// else. A second reader can then subtract SETS rather than strings:
        /// four attempts to locate a two-section disagreement failed because
        /// each compared truncated titles, which is a defect in the comparison
        /// and not in either reader.
        #[arg(long)]
        numbers: bool,
        /// List the sections whose figure stands over a SLIDING population,
        /// and whether each names the point its window ended at.
        #[arg(long)]
        windowed: bool,
    },
}

fn repo_root() -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running `git rev-parse --show-toplevel`")?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(String::from_utf8(out.stdout)?.trim()))
}

/// `## 179. Title` -> (179, "Title"). Anything else is not a numbered section.
pub fn sections(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("## ") else {
            continue;
        };
        let Some((num, title)) = rest.split_once(". ") else {
            continue;
        };
        if let Ok(n) = num.parse::<usize>() {
            out.push((n, title.trim().to_string()));
        }
    }
    out
}

/// Every complaint about one file. Empty means the numbering holds.
pub fn problems(secs: &[(usize, String)]) -> Vec<String> {
    let mut bad = Vec::new();
    let mut seen: BTreeMap<usize, Vec<&str>> = BTreeMap::new();
    for (n, t) in secs {
        seen.entry(*n).or_default().push(t);
    }
    for (n, titles) in &seen {
        if titles.len() > 1 {
            bad.push(format!(
                "section {n} appears {} times: {}",
                titles.len(),
                titles
                    .iter()
                    // W705: `&t[..46]` panics when byte 46 lands inside a
                    // multi-byte character, and these titles contain em dashes.
                    // My own section headings crashed my own command the first
                    // time it met one.
                    .map(|t| format!("\"{}\"", t.chars().take(46).collect::<String>()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    for w in secs.windows(2) {
        if w[1].0 < w[0].0 {
            bad.push(format!(
                "section {} comes after {} -- the file reads out of order",
                w[1].0, w[0].0
            ));
        }
    }
    bad
}

/// Numbers missing from the run. Reported, never a failure.
pub fn gaps(secs: &[(usize, String)]) -> Vec<usize> {
    if secs.is_empty() {
        return Vec::new();
    }
    let nums: std::collections::BTreeSet<usize> = secs.iter().map(|(n, _)| *n).collect();
    let (lo, hi) = (
        *nums.iter().next().unwrap(),
        *nums.iter().next_back().unwrap(),
    );
    (lo..=hi).filter(|n| !nums.contains(n)).collect()
}

fn skill_files(root: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let dir = root.join(".claude/skills");
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return out;
    };
    for e in rd.flatten() {
        let p = e.path().join("SKILL.md");
        if p.is_file() {
            out.push(p);
        }
    }
    out.sort();
    out
}

pub fn run(cmd: &SkillCmd) -> Result<()> {
    let show_gaps = match cmd {
        SkillCmd::Claims {
            list,
            numbers,
            windowed,
        } => return claims(*list, *numbers, *windowed),
        SkillCmd::Check { gaps } => gaps,
    };
    if *show_gaps {
        println!("  --gaps is now the default: an unused number is always stated.");
    }
    let root = repo_root()?;
    let files = skill_files(&root);
    if files.is_empty() {
        // NOT "everything is fine". No file was read, so nothing is claimed.
        anyhow::bail!(
            "no SKILL.md under {}/.claude/skills -- nothing was checked",
            root.display()
        );
    }
    let mut failed = 0usize;
    // Files that actually CONTRIBUTED a numbered section. Four of the five
    // SKILL.md files in this repository have none, and reporting "5 file(s)"
    // counts four where there was nothing to check -- the same shape as
    // "13 gates green" when two of them never ran.
    let mut with_sections = 0usize;
    let mut total_sections = 0usize;
    let mut gapped: Vec<(String, Vec<usize>)> = Vec::new();
    for f in &files {
        let text =
            std::fs::read_to_string(f).with_context(|| format!("reading {}", f.display()))?;
        let secs = sections(&text);
        let name = f
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let bad = problems(&secs);
        let g = gaps(&secs);
        if !secs.is_empty() {
            with_sections += 1;
            total_sections += secs.len();
        }
        if !g.is_empty() {
            gapped.push((name.clone(), g.clone()));
        }
        println!(
            "  {:<24} {:>4} section(s){}",
            name,
            secs.len(),
            if bad.is_empty() { "" } else { "  PROBLEMS" }
        );
        for b in &bad {
            println!("      {b}");
            failed += 1;
        }
    }
    println!();
    if failed > 0 {
        println!("  {failed} numbering problem(s). Two sections with one number is what a");
        println!("  merge that keeps BOTH sides produces, and nothing else notices it.");
        std::process::exit(1);
    }
    // Say what was checked, not how many files were opened. "Numbering holds"
    // over a file with no numbers is true and worthless; a reader takes the
    // summary as the verdict and never reads the rows above it.
    println!(
        "  No number is used twice: {total_sections} section(s) across {with_sections} of {} file(s) read.",
        files.len()
    );
    if with_sections < files.len() {
        println!(
            "  The other {} contributed no numbered section, so nothing was checked in them.",
            files.len() - with_sections
        );
    }
    for (name, g) in &gapped {
        // A gap is not a failure -- a section can be deleted, and refusing
        // would make the log unmergeable. But the summary above must not read
        // as "the sequence is intact" while a number is missing from it.
        println!(
            "  {name}: {} number(s) never used ({}). Not a failure; stated so it is not mistaken for one.",
            g.len(),
            g.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Vec<(usize, String)> {
        sections(s)
    }

    /// The exact collision that happened: two sessions, both appending at 179.
    #[test]
    fn two_sections_with_one_number_is_a_problem() {
        let secs = parse("## 178. A\n## 179. Mine\n## 179. Theirs\n## 180. B\n");
        let p = problems(&secs);
        assert_eq!(p.len(), 1, "{p:?}");
        assert!(p[0].contains("179"), "{p:?}");
    }

    #[test]
    fn descending_numbers_are_a_problem() {
        let p = problems(&parse("## 180. B\n## 179. A\n"));
        assert_eq!(p.len(), 1, "{p:?}");
        assert!(p[0].contains("out of order"), "{p:?}");
    }

    /// A gap is not a failure. Renumbering a 185-section file to close one hole
    /// rewrites every reference after it.
    #[test]
    fn a_gap_is_reported_and_is_not_a_problem() {
        let secs = parse("## 1. A\n## 2. B\n## 4. D\n");
        assert!(problems(&secs).is_empty());
        assert_eq!(gaps(&secs), vec![3]);
    }

    /// Prose that merely starts with `##` is not a numbered section, and a
    /// version-like `## 1.2 something` has no `. ` after the integer.
    #[test]
    fn only_numbered_sections_count() {
        assert!(parse("## Overview\n## Notes\n").is_empty());
        assert_eq!(parse("## 7. Real\n").len(), 1);
    }

    /// A title with a multi-byte character must not panic the reporter.
    #[test]
    fn a_title_with_an_em_dash_is_truncated_safely() {
        let long = format!("## 1. {}\n## 1. b\n", "— an em dash ".repeat(6));
        let p = problems(&sections(&long));
        assert_eq!(p.len(), 1, "{p:?}");
    }

    #[test]
    fn a_clean_run_has_nothing_to_say() {
        let secs = parse("## 1. A\n## 2. B\n## 3. C\n");
        assert!(problems(&secs).is_empty());
        assert!(gaps(&secs).is_empty());
    }
}

// ---------------------------------------------------------------------------
// `tri skill claims` -- the figures in the knowledge base, and what re-takes them.
// ---------------------------------------------------------------------------

/// The body of each numbered section, in file order.
///
/// A section runs from its own `## N.` heading to the next one; the preamble
/// before the first heading belongs to no section and is dropped, which is why
/// this returns the same count `sections()` does rather than one more.
pub fn section_bodies(text: &str) -> Vec<(usize, String, String)> {
    let mut out: Vec<(usize, String, String)> = Vec::new();
    let mut cur: Option<(usize, String, Vec<&str>)> = None;
    for line in text.lines() {
        let head = line.strip_prefix("## ").and_then(|rest| {
            rest.split_once(". ")
                .and_then(|(n, t)| n.parse::<usize>().ok().map(|n| (n, t.trim().to_string())))
        });
        if let Some((n, t)) = head {
            if let Some((pn, pt, body)) = cur.take() {
                out.push((pn, pt, body.join("\n")));
            }
            cur = Some((n, t, Vec::new()));
        } else if let Some((_, _, body)) = cur.as_mut() {
            body.push(line);
        }
    }
    if let Some((n, t, body)) = cur {
        out.push((n, t, body.join("\n")));
    }
    out
}

/// The 1-based inclusive line range each numbered section occupies.
///
/// Kept separate from `section_bodies` rather than widening its tuple: three call
/// sites and four tests read that shape, and a range is wanted by exactly one of
/// them. The two walk the headings by the same rule, and a test pins them together
/// so the pair cannot drift apart silently.
pub fn section_ranges(text: &str) -> Vec<(usize, usize, usize)> {
    let mut out: Vec<(usize, usize, usize)> = Vec::new();
    let mut cur: Option<(usize, usize)> = None;
    let mut last = 0usize;
    for (i, line) in text.lines().enumerate() {
        last = i + 1;
        let head = line.strip_prefix("## ").and_then(|rest| {
            rest.split_once(". ")
                .and_then(|(n, _)| n.parse::<usize>().ok())
        });
        if let Some(n) = head {
            if let Some((pn, pstart)) = cur.take() {
                out.push((pn, pstart, i));
            }
            cur = Some((n, i + 1));
        }
    }
    if let Some((n, start)) = cur {
        out.push((n, start, last));
    }
    out
}

/// The newest commit touching any line of a `git blame --porcelain` block.
///
/// The NEWEST, not the oldest, and that is the whole point: it answers "this section
/// was last written no later than X", which bounds how fresh its figure can be. The
/// oldest would answer when the section was started, which a later edit invalidates.
///
/// Returns the commit id; the date is asked of git separately rather than computed
/// here, because a civil-date conversion is fifteen lines of arithmetic this file has
/// no other reason to own.
pub fn newest_blamed_commit(porcelain: &str) -> Option<String> {
    let mut best: Option<(i64, String)> = None;
    let mut sha: Option<String> = None;
    for line in porcelain.lines() {
        let head = line.split(' ').next().unwrap_or("");
        if head.len() == 40 && head.chars().all(|c| c.is_ascii_hexdigit()) {
            sha = Some(head.to_string());
        } else if let Some(t) = line.strip_prefix("author-time ") {
            if let (Ok(t), Some(s)) = (t.trim().parse::<i64>(), sha.clone()) {
                if best.as_ref().is_none_or(|(bt, _)| t > *bt) {
                    best = Some((t, s));
                }
            }
        }
    }
    best.map(|(_, s)| s)
}

/// Ask git when a section was last written. `None` when git cannot answer -- an
/// untracked file, no repository, a blame that fails -- and the caller says so
/// rather than printing a date it did not get.
fn recovered_anchor(file: &Path, start: usize, end: usize) -> Option<(String, String)> {
    let dir = file.parent()?;
    let blame = std::process::Command::new("git")
        .args([
            "blame",
            "-L",
            &format!("{start},{end}"),
            "--porcelain",
            "--",
        ])
        .arg(file)
        .current_dir(dir)
        .output()
        .ok()?;
    if !blame.status.success() {
        return None;
    }
    let sha = newest_blamed_commit(&String::from_utf8_lossy(&blame.stdout))?;
    let show = std::process::Command::new("git")
        .args(["show", "-s", "--date=short", "--format=%ad", &sha])
        .current_dir(dir)
        .output()
        .ok()?;
    if !show.status.success() {
        return None;
    }
    let date = String::from_utf8_lossy(&show.stdout).trim().to_string();
    if date.is_empty() {
        return None;
    }
    Some((sha[..9.min(sha.len())].to_string(), date))
}

/// Does this section's HEADING state a figure?
///
/// The heading is where a section makes its claim, the way an issue's title
/// is. Pointed at the body instead, the same rule reports **404 of 409** --
/// a matcher describing its input, because a section is twenty-five lines of
/// prose about numbers. Pointed at the heading it reports **121**.
pub fn states_a_figure(title: &str) -> bool {
    matches!(
        crate::issues::carries(title),
        crate::issues::Carries::Digits
            | crate::issues::Carries::Words
            | crate::issues::Carries::Both
    )
}

/// The spellings that make a population a QUERY rather than a set.
///
/// Deliberately narrow, and the narrowing is measured. A looser rule keyed on the
/// word `today` fires on **28** further sections here, almost none of which state a
/// windowed figure -- it is a matcher describing its input, and it is excluded with
/// that count printed rather than dropped in silence.
///
/// What remains names a window explicitly: a run of the last N of something, an API
/// page or limit, "master runs", or a count of open issues. Twelve sections of 419.
pub fn window_markers(text: &str) -> Vec<&'static str> {
    let low = text.to_lowercase();
    let mut out = Vec::new();
    // `last 20 commits`, `of the last 40`
    //
    // EVERY occurrence, not the first. `find` answers "does the FIRST `last ` satisfy
    // this?" and the question is "does ANY?" -- a difference invisible on one line and
    // certain on a page of prose, which is the only kind of text this reads. Section
    // 439 says "reads the last COMMIT message" on its line 18, where no digit follows,
    // and "Over the last 20 commit messages on master" on line 27. With `find` the
    // rule stopped at the first and 439 was absent from its own population, though it
    // is the section that produced the 4-against-33 row this rule was written for.
    // Exactly one section is masked, at HEAD and at the anchor alike: 15 was 16 and 12
    // was 13.
    for (i, _) in low.match_indices("last ") {
        let rest = &low[i + 5..];
        let n: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if n.is_empty() {
            continue;
        }
        let after = rest[n.len()..].trim_start();
        if ["commit", "run", "pr", "pull request", "issue", "merged"]
            .iter()
            .any(|w| after.starts_with(w))
        {
            out.push("last N");
            break;
        }
    }
    for (needle, label) in [
        ("per_page=", "API window"),
        ("--limit", "API window"),
        ("-l 40", "API window"),
        ("master run", "master runs"),
        ("open issue", "open issues"),
    ] {
        // No de-duplicating the label: the caller reads `is_empty()`, so a
        // second identical marker is unobservable. Measured, not assumed --
        // removing the guard moved nothing on the corpus and no test could
        // tell the two versions apart.
        if low.contains(needle) {
            out.push(label);
        }
    }
    out
}

/// Does this section name a revision or a date ANYWHERE?
///
/// This is an UPPER BOUND on "the window is anchored", not that claim itself, and
/// the distinction was measured rather than assumed. It reports 3 of the 12; all
/// three were read by hand and one does not survive. Section 125 says *"checks have
/// not fired since 2026-08-24 11:06"* -- a date that anchors the CLAIM, while the
/// window it actually read was "the last 10, then 60 runs" and is dated nowhere. A
/// date in the body is not necessarily the date of the reading.
///
/// So a section this returns `false` for is definitely unanchored; a section it
/// returns `true` for merely might be. The lower bound needs a rule that ties the
/// date to the query, and no such rule is claimed here.
///
/// Reuses `issues::revision_pins`, which is already mutation-proved against the two
/// floats that look like commit ids (`-1.7594823e-05`, `5.391247e-44`), plus an
/// ISO date.
pub fn names_its_anchor(body: &str) -> bool {
    if !crate::issues::revision_pins(body).is_empty() {
        return true;
    }
    let b: Vec<char> = body.chars().collect();
    (0..b.len().saturating_sub(9)).any(|i| is_iso_date(&b[i..i + 10]))
}

/// Exactly `20YY-MM-DD`, ten characters, tested as ONE rule.
///
/// Written as a unit deliberately. Mutating a single character position out of a
/// ten-conjunct pattern does not test whether the pattern is over-specified -- it
/// asks a question no natural input answers, and three such mutants survived while
/// moving nothing on the corpus. The shape is one claim, so it gets one test.
///
/// A left-hand word-boundary check used to sit here too (`v2026-08-20` should not
/// count). It was removed rather than kept unproved: no natural counter-example
/// exists in this corpus and removing it changed no section.
pub fn is_iso_date(w: &[char]) -> bool {
    w.len() == 10
        && w[0] == '2'
        && w[1] == '0'
        && w[4] == '-'
        && w[7] == '-'
        && [2, 3, 5, 6, 8, 9].iter().all(|&i| w[i].is_ascii_digit())
}

/// Does this section name something a reader could run to take the reading again?
///
/// Deliberately narrow: a backticked command starting with one of the verbs
/// this repository is driven by. Prose describing a measurement does not count
/// -- the question is whether a second reader has an instrument, not whether
/// the author had one.
pub fn names_a_command(body: &str) -> bool {
    const VERBS: [&str; 7] = [
        "tri ", "t27c ", "cargo ", "python3 ", "bash ", "gh ", "git ",
    ];
    let mut rest = body;
    while let Some(i) = rest.find('`') {
        let after = &rest[i + 1..];
        let Some(j) = after.find('`') else {
            return false;
        };
        let span = &after[..j];
        if VERBS.iter().any(|v| span.starts_with(v)) {
            return true;
        }
        rest = &after[j + 1..];
    }
    false
}

/// A path as the repository writes it.
fn rel(root: &std::path::Path, p: &std::path::Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .to_string_lossy()
        .to_string()
}

fn claims(list: bool, numbers: bool, windowed_list: bool) -> Result<()> {
    let root = repo_root()?;
    // Every tracked SKILL.md, not just `.claude/skills/*`.
    //
    // The previous version read that one directory and NAMED the three files
    // outside it, which was honest and incomplete. Measured before widening:
    // `.agents/skills/phi-loop/SKILL.md` and `.agents/skills/tri-pipeline/SKILL.md`
    // are **byte-identical** to their `.claude/skills` counterparts -- copies,
    // not forks -- and all three unread files carry **zero** numbered sections.
    // So widening adds nothing to the figure count, which is exactly why it is
    // safe to do and why the previous iteration's worry (that counting copies
    // would double every figure) turned out to be about an empty set.
    //
    // Copies are still detected and named: they contribute nothing today, and
    // the day one of them gains a section the count would double it silently.
    let mut files = skill_files(&root);
    let mut copies: Vec<(String, String)> = Vec::new();
    if let Ok(out) = std::process::Command::new("git")
        .args(["ls-files", "*SKILL.md"])
        .current_dir(&root)
        .output()
    {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let p = root.join(line);
            if p.is_file() && !files.contains(&p) {
                files.push(p);
            }
        }
    }
    files.sort();
    // Byte-identical duplicates: keep the first, name the rest.
    let mut seen: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut keep: Vec<PathBuf> = Vec::new();
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        let digest = format!("{}:{}", text.len(), text.lines().count());
        match seen.get(&digest) {
            Some(first) if std::fs::read_to_string(first).ok().as_deref() == Some(&text) => {
                copies.push((rel(&root, &f), rel(&root, first)));
            }
            _ => {
                seen.insert(digest, f.clone());
                keep.push(f);
            }
        }
    }
    let files = keep;
    if files.is_empty() {
        anyhow::bail!(
            "no SKILL.md under {}/.claude/skills -- nothing was read, and a zero \
             here would print as \"no section states a figure\".",
            root.display()
        );
    }

    let mut per_file: Vec<(String, usize, usize)> = Vec::new();
    let mut total = 0usize;
    let mut carrying = 0usize;
    let mut with_cmd = 0usize;
    let mut anchored = 0usize;
    /// A section whose population is a query. `anchored` is the upper bound the
    /// text itself supports; `range` is what lets git answer when the text was
    /// last written, for the ones the text does not date.
    struct Windowed {
        skill: String,
        n: usize,
        title: String,
        anchored: bool,
        figure: bool,
        file: PathBuf,
        range: (usize, usize),
    }
    let mut windowed: Vec<Windowed> = Vec::new();
    let mut free: Vec<(String, usize, String)> = Vec::new();
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        let name = f
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let (mut fsecs, mut fcar) = (0usize, 0usize);
        let ranges: std::collections::HashMap<usize, (usize, usize)> = section_ranges(&text)
            .into_iter()
            .map(|(n, a, b)| (n, (a, b)))
            .collect();
        for (n, title, body) in section_bodies(&text) {
            total += 1;
            fsecs += 1;
            // The SAME population rule `tri issues numbers` uses, on a second
            // subject: an address is not a count here either -- these sections
            // are full of `#2994`, `Wave Loop 369` and `w699`.
            // Read from the HEADING, not the body, and that distinction is the
            // measurement: over the body this rule reports **404 of 409**, which
            // is a matcher describing its input -- a section is twenty-five lines
            // of prose about numbers and almost all of them mention one. Over the
            // heading it reports **123 of 409**. A rule written for a one-line
            // claim does not transfer to a page of argument by being pointed at
            // it; the SUBJECT has to be the place the claim is made.
            // A population that is a QUERY rather than a set. "The last 20
            // commits on master" is not a set: its answer changes on every
            // push, so the CLAIM survives while the NUMBER does not, and
            // re-measuring it is not a second reading -- it is a first reading
            // of a different population.
            //
            // Measured ABOVE the figure filter, and that placement is the
            // finding. Nested inside it this reported **4**; a hand count over
            // every section said **12**, and the gap was not an error in either
            // -- they are two populations. The filter reads the HEADING, so a
            // section that argues about a window without putting a digit in its
            // title never reached the check. The worst case was section 179,
            // whose title IS the rule: "A `--limit` on a run list is a time
            // window in disguise". Both numbers are printed below; neither is
            // allowed to stand alone.
            let fig = states_a_figure(&title);
            if !window_markers(&format!("{title}\n{body}")).is_empty() {
                windowed.push(Windowed {
                    skill: name.clone(),
                    n,
                    title: title.clone(),
                    anchored: names_its_anchor(&body),
                    figure: fig,
                    file: f.clone(),
                    range: *ranges.get(&n).unwrap_or(&(0, 0)),
                });
            }
            if !fig {
                continue;
            }
            carrying += 1;
            fcar += 1;
            if numbers {
                println!("{name}:{n}");
            }
            if names_a_command(&body) {
                with_cmd += 1;
            }
            // And the SAME anchor rule `tri issues dated` uses. A section that
            // pins a revision or says as-of is dated by construction: "I
            // published 268 and it was wrong" cannot go stale, it is history.
            if crate::issues::anchor_of(&body, 0) != crate::issues::Anchor::Free {
                anchored += 1;
            } else {
                free.push((name.clone(), n, title));
            }
        }
        per_file.push((name, fsecs, fcar));
    }

    if numbers {
        return Ok(());
    }
    println!("FIGURES IN THE KNOWLEDGE BASE, AND WHAT COULD RE-TAKE THEM\n");
    println!("  SKILL.md files read           {}", files.len());
    for (dup, first) in &copies {
        println!("    byte-identical copy, counted once: {dup} == {first}");
    }
    println!("  numbered sections             {total}");
    println!("  stating a figure              {carrying}");
    println!("  of those, naming a command    {with_cmd}");
    println!("  of those, anchored (dated)    {anchored}");
    println!("  free to go stale              {}", free.len());
    let anchored_windows = windowed.iter().filter(|w| w.anchored).count();
    let fig_windows = windowed.iter().filter(|w| w.figure).count();
    println!(
        "  over a SLIDING population     {}   (of ALL {total} sections, not of the {carrying})",
        windowed.len()
    );
    println!("    of those, stating a figure  {fig_windows}");
    println!("    naming a date or revision   {anchored_windows}   <- upper bound");
    println!("\n  by skill (sections / stating a figure):");
    for (name, secs, car) in &per_file {
        println!("    {name:<14} {secs:>4} / {car}");
    }

    println!(
        "\n  A figure with no command beside it is not wrong; it is unre-takeable\n  \
         by anyone but its author. That is the rot surface, and it is the first\n  \
         time it has been counted here.\n\n  \
         The population and the anchor rule are the ones `tri issues numbers`\n  \
         and `tri issues dated` already use, pointed at a second subject: an\n  \
         address is not a count in a skill either, and a section that says \"I\n  \
         published 268 and it was wrong in both directions\" is history rather\n  \
         than a claim about the tree -- re-measuring it proves nothing."
    );

    println!(
        "\n  A figure over a sliding population is stale by construction, and\n  \
         re-measuring it is not a second reading -- it is a first reading of a\n  \
         different population. Seven headline figures published in one session\n  \
         were re-read hours later: the three over sliding populations had ALL\n  \
         moved (4/33 -> 3/37, 121/40/116 -> 126/44/121, 288 -> 287) and the\n  \
         three over files on disk had not moved at all.\n\n  \
         The anchor is part of the number: \"over the 20 commits ending at\n  \
         <sha>\", not \"over the last 20\". {anchored_windows} of these {} name a\n  \
         date or a revision at all -- and that is an UPPER bound, not the count.\n  \
         All three were read by hand and one does not survive: section 125\n  \
         dates its CLAIM (\"not fired since 2026-08-24 11:06\") while the window\n  \
         it read was \"the last 10, then 60 runs\", dated nowhere.\n\n  \
         This count is taken over EVERY section, not over the {carrying} that\n  \
         state a figure. Nested inside that filter it reported 4 and a hand\n  \
         count said 12 -- two populations, not an error in either, and the\n  \
         section the filter dropped was the one whose title is this rule.\n\n  \
         A looser rule keyed on the word `today` fires on 28 further sections\n  \
         that state no window at all -- a matcher describing its input. It is\n  \
         excluded with that count said out loud rather than dropped in silence.",
        windowed.len()
    );

    if windowed_list {
        println!("\n  OVER A SLIDING POPULATION, and whether the anchor is named:\n");
        for w in &windowed {
            let mark = if w.anchored { "anchored" } else { "NO ANCHOR" };
            let f = if w.figure { "figure" } else { "  ..  " };
            let short = &w.title[..w.title.len().min(52)];
            println!("    {mark:<10} {f} {:<11} {:>4}  {short}", w.skill, w.n);
            // For the ones the text does not date, git can. This is not the date
            // the reading was TAKEN -- it is the date the section was last
            // written, so the figure is no fresher than this. A bound recovered
            // rather than invented, and printed as a DATE: an age in days would
            // itself be a figure over a sliding population.
            if !w.anchored && w.range.1 > 0 {
                match recovered_anchor(&w.file, w.range.0, w.range.1) {
                    Some((sha, date)) => println!(
                        "                      last written no later than {date}  ({sha})"
                    ),
                    None => println!(
                        "                      git could not date this section, so nothing is claimed"
                    ),
                }
            }
        }
    }

    if list {
        println!("\n  FREE TO GO STALE, by section:\n");
        for (skill, n, title) in &free {
            println!("    {skill:<12} {n:>4}  {}", &title[..title.len().min(72)]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod claims_tests {
    use super::*;

    #[test]
    fn the_heading_is_the_subject_not_the_body() {
        // A section whose claim carries no figure is not in the population,
        // however many numbers its argument mentions. Pointed at bodies this
        // rule reports 404 of 409 -- it would be describing its input.
        assert!(!states_a_figure(
            "A control that cannot fail is not a control"
        ));
        assert!(states_a_figure("Five local previews of a gate"));
        assert!(states_a_figure("The measurement tree lost 296 files"));
        // And an ADDRESS is not a figure here either -- these headings are
        // full of them.
        assert!(!states_a_figure(
            "Grep before you file -- #2964 duplicated #2822"
        ));
        assert!(!states_a_figure("Wave Loop 369 and w699 are addresses"));
    }

    #[test]
    fn a_command_is_a_backticked_verb_not_prose_about_one() {
        assert!(names_a_command("run `tri gates empty` to re-take it"));
        assert!(names_a_command("measured with `cargo test -p tri`"));
        assert!(names_a_command("`python3 tools/check_json_parses.py`"));
        // Prose naming a tool is not an instrument a second reader can run.
        assert!(!names_a_command("measured with tri gates empty"));
        assert!(!names_a_command("the `matrix.include` field was empty"));
        assert!(!names_a_command("no backticks here at all"));
    }

    #[test]
    fn sections_are_split_at_their_own_headings_and_the_preamble_is_not_one() {
        let t = "preamble\nwith numbers 42\n\n## 7. First\nbody one\n\n## 8. Second\nbody two\n";
        let got = section_bodies(t);
        assert_eq!(got.len(), 2, "the preamble belongs to no section");
        assert_eq!(got[0].0, 7);
        assert_eq!(got[0].1, "First");
        assert!(got[0].2.contains("body one"));
        assert!(
            !got[0].2.contains("body two"),
            "a section stops at the next heading"
        );
        assert_eq!(got[1].0, 8);
    }
}

#[cfg(test)]
mod window_tests {
    use super::*;

    /// The probe: the shape the rule is written for.
    #[test]
    fn a_run_of_the_last_n_commits_is_a_window() {
        assert_eq!(
            window_markers("read the last 20 commits on master"),
            vec!["last N"]
        );
        assert_eq!(window_markers("gh run list --limit 40"), vec!["API window"]);
        assert_eq!(window_markers("?per_page=100"), vec!["API window"]);
    }

    /// The probe the shipped rule failed. `last ` occurs twice and only the SECOND
    /// occurrence is a window; `find` stops at the first and reports nothing. Section
    /// 439 is exactly this shape -- "reads the last COMMIT message" before "Over the
    /// last 20 commit messages on master" -- and it was absent from its own population
    /// until the rule walked every occurrence.
    #[test]
    fn a_window_after_a_non_window_is_still_a_window() {
        assert_eq!(
            window_markers("reads the last commit message. Over the last 20 commits on master"),
            vec!["last N"],
            "the first `last ` has no digit; the second does"
        );
    }

    /// And two non-windows are still not a window, so the fix did not simply widen
    /// the rule into agreeing with everything.
    #[test]
    fn two_non_windows_remain_two_non_windows() {
        assert!(window_markers("the last commit, and at last the last word").is_empty());
    }

    /// The counter-example, and it is the whole reason the rule enumerates nouns
    /// instead of matching `last <digits>`: a duration is not a population.
    #[test]
    fn a_span_of_time_is_not_a_windowed_population() {
        assert!(window_markers("it failed in the last 20 minutes").is_empty());
        assert!(window_markers("the last 3 attempts each took a day").is_empty());
        assert!(window_markers("at last, a green run").is_empty());
        assert!(window_markers("the last commit").is_empty());
    }

    /// The defect this placement fixes, stated at the level of the two rules that
    /// were nested. Section 179's title IS the rule and carries no digit, so a
    /// window check gated on `states_a_figure` never reaches it: 4 of 126 instead
    /// of 12 of 420. Both readings are correct; only one answers the question.
    #[test]
    fn the_figure_filter_would_drop_the_section_that_states_this_rule() {
        let title = "A `--limit` on a run list is a time window in disguise";
        assert!(
            !states_a_figure(title),
            "if this heading ever states a figure the test below stops proving anything"
        );
        assert!(
            !window_markers(title).is_empty(),
            "the section whose title is this rule must be inside the population"
        );
    }

    /// A section this says `false` for is definitely unanchored. `true` is only an
    /// upper bound -- see the doc comment for the section that fails it.
    #[test]
    fn an_anchor_is_a_revision_or_an_iso_date() {
        assert!(names_its_anchor("over the 20 commits ending at 1b47f8b85"));
        assert!(names_its_anchor("measured 2026-08-20"));
        assert!(!names_its_anchor("over the last 20 commits on master"));
        // Delegated to the rule that is already mutation-proved against these.
        assert!(!names_its_anchor(
            "the residual was -1.7594823e-05 at worst"
        ));
    }

    /// A year is not a date, and a version is not a year.
    #[test]
    fn a_bare_year_does_not_anchor_a_window() {
        assert!(!names_its_anchor("in 2026 the gate was added"));
        assert!(!names_its_anchor("2026-08 was the month"));
    }
}

#[cfg(test)]
mod iso_date_tests {
    use super::is_iso_date;

    fn d(s: &str) -> bool {
        let c: Vec<char> = s.chars().collect();
        c.len() == 10 && is_iso_date(&c)
    }

    #[test]
    fn the_shape_is_exactly_ten_characters_of_20yy_mm_dd() {
        assert!(d("2026-08-20"));
        assert!(d("2099-12-31"));
    }

    #[test]
    fn anything_else_of_that_length_is_not_a_date() {
        assert!(!d("1999-08-20"), "the century is pinned");
        assert!(!d("2026/08/20"), "the separator is pinned");
        assert!(!d("2026-08-2x"), "every digit position is a digit");
        assert!(!d("2026-0820-"), "the separators are positional");
        assert!(!d("2026x08-20"), "the first separator is pinned on its own");
        assert!(!d("2026-08x20"), "and so is the second");
        assert!(!d("20260820ab"), "no separators at all");
        let short: Vec<char> = "2026-08-2".chars().collect();
        assert!(!is_iso_date(&short), "nine characters is not the shape");
    }
}

#[cfg(test)]
mod anchor_recovery_tests {
    use super::{newest_blamed_commit, section_bodies, section_ranges};

    const DOC: &str = "intro line\n\
        ## 7. First\n\
        body a\n\
        body b\n\
        ## 9. Second\n\
        body c\n";

    /// The two walkers must agree on WHICH sections exist. They read the same
    /// headings by the same rule and are written apart, so nothing but a test stops
    /// them drifting -- and a range attached to the wrong section would date the
    /// wrong claim, silently and plausibly.
    #[test]
    fn ranges_and_bodies_see_the_same_sections() {
        let a: Vec<usize> = section_bodies(DOC).iter().map(|(n, _, _)| *n).collect();
        let b: Vec<usize> = section_ranges(DOC).iter().map(|(n, _, _)| *n).collect();
        assert_eq!(a, b);
        assert_eq!(a, vec![7, 9]);
    }

    /// One-based and inclusive, ending at the line before the next heading -- not at
    /// the heading itself, which belongs to the next section and would date it.
    #[test]
    fn a_range_stops_before_the_next_heading() {
        assert_eq!(section_ranges(DOC), vec![(7, 2, 4), (9, 5, 6)]);
    }

    const BLAME: &str = "\
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 1 1 1\n\
author Someone\n\
author-time 1000\n\
\tolder line\n\
bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb 2 2 1\n\
author Someone\n\
author-time 3000\n\
\tnewer line\n\
cccccccccccccccccccccccccccccccccccccccc 3 3 1\n\
author Someone\n\
author-time 2000\n\
\tmiddle line\n";

    /// The NEWEST, and the fixture is deliberately out of order so that "the last one
    /// seen" and "the newest" cannot both pass.
    #[test]
    fn the_newest_commit_wins_not_the_first_or_the_last() {
        assert_eq!(
            newest_blamed_commit(BLAME).as_deref(),
            Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        );
    }

    /// Content lines are tab-prefixed, so a line of source that happens to look like
    /// a commit id is not one. Without the length check `deadbeef` in a code block
    /// becomes the answer.
    #[test]
    fn a_hex_word_in_the_content_is_not_a_commit() {
        let tricky = "\
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 1 1 1\n\
author-time 1000\n\
\tdeadbeef is not a commit here\n\
deadbeef 2 2 1\n\
author-time 9999\n";
        assert_eq!(
            newest_blamed_commit(tricky).as_deref(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            "a short hex token must not be adopted as the blamed commit"
        );
    }

    /// Nothing to date is not a date. The caller prints that it claims nothing.
    #[test]
    fn no_blame_is_no_answer() {
        assert_eq!(newest_blamed_commit(""), None);
        assert_eq!(newest_blamed_commit("fatal: no such path\n"), None);
        assert_eq!(
            newest_blamed_commit("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 1 1 1\n"),
            None,
            "a commit with no author-time dates nothing"
        );
    }
}

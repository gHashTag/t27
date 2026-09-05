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
    /// Write a new lesson to the spool, unnumbered.
    ///
    /// The collision this removes: two branches each append `## N.` to SKILL.md
    /// numbered from their OWN base, both merge, and the number appears twice.
    /// It happened twice in two passes (#3236), the repairs raced each other
    /// once more, and no branch-side check can see it -- `tri skill check`
    /// passes on both sides and fails only on the merge result.
    ///
    /// A spool file has a unique path, so two branches writing two lessons
    /// write two paths and there is nothing to conflict on. This is the shape
    /// `docs/now/` already uses, for the same reason and after the same defect.
    Add {
        /// The section title, as it will read in SKILL.md.
        title: String,
        /// Which skill to file it under. Defaults to `ci-gates`.
        #[arg(long, default_value = "ci-gates")]
        skill: String,
    },
    /// Fold every spooled lesson into SKILL.md, numbering them on the way in.
    ///
    /// The number is assigned HERE, against the SKILL.md that exists now, which
    /// is why this is the step that cannot collide: it runs once, on one
    /// branch, with the file in front of it.
    Fold {
        /// Report what would move and write nothing.
        #[arg(long)]
        check: bool,
        /// Which skill's spool to fold. Defaults to `ci-gates`.
        #[arg(long, default_value = "ci-gates")]
        skill: String,
    },
    /// Check every SKILL.md's section numbering.
    Check {
        /// Also print the gaps, which are reported but never fail.
        #[arg(long)]
        gaps: bool,
    },
    /// Sections whose body was truncated at some point in the file's history.
    ///
    /// A section here is a claim with evidence attached, and the evidence is
    /// usually a quoted artefact. `tri skill renumber` destroyed one on
    /// 2026-09-05 -- it cut a tail at a QUOTED heading, dropped the section
    /// holding it, and left the opening fence unclosed so the next section was
    /// swallowed. The `titles_lost` guard added afterwards refuses on titles and
    /// cannot see a body that was merely cut short, so this asks the question
    /// the guard cannot.
    ///
    /// The signature is exact: the body on `--base` is a strict PREFIX of the
    /// body when the section first appeared. An edit in place is not a prefix; a
    /// truncation is. Differences that are only trailing blank lines are not
    /// reported -- measured across 281 commits, 38 of the 40 prefix hits were
    /// exactly that.
    Lost {
        /// Which document. `docs/NOW.md` is the other append-only file here and
        /// has NO numbered headings at all -- 312 of them, every one of the
        /// shape `## fix(...)`. A version of this command that insisted on
        /// `## N. ` would have walked its whole history, found nothing, and
        /// printed a clean bill of health over an empty population.
        #[arg(long, default_value = ".claude/skills/ci-gates/SKILL.md")]
        file: String,
        /// Compare against this ref instead of `origin/master`.
        #[arg(long, default_value = "origin/master")]
        base: String,
        /// Exit 1 if anything is found, for use as a gate.
        #[arg(long)]
        gate: bool,
    },
    /// Whether every section this branch adds came through the spool.
    ///
    /// The guard `tri skill add` needs in order to survive a change of context.
    /// The rule it enforces is written at the top of SKILL.md, and a rule that
    /// lives only in prose is the same class of thing that already failed here:
    /// the next pass reads what is convenient and reaches for `cat >>`.
    Spooled {
        /// The branch this one is measured against.
        #[arg(long, default_value = "origin/master")]
        base: String,
        /// Exit 1 when a section arrived without a spool file. Off by default so
        /// the command can be read before it is enforced.
        #[arg(long)]
        gate: bool,
    },
    /// Every cross-reference in the skills, and whether it resolves.
    Refs {
        /// Print every reference counted, not only the ones that dangle.
        #[arg(long)]
        list: bool,
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
    // A heading inside a fenced block is a QUOTATION, not a section. This file
    // quotes section headings as evidence -- a section about a duplicated
    // heading shows the duplicate -- and counting those cost a real section:
    // `tri skill renumber` cut its tail at a quoted title, dropped the section
    // that contained it, and left the opening fence unclosed so the next
    // section was swallowed too. Measured on master 4d63859: 518 lines match
    // `## N. `, and 3 of them are inside a fenced block.
    //
    // CommonMark's rule, and it is what makes this reliable here: an OPENING
    // fence may carry an info string, a CLOSING fence may not. A naive toggle
    // on every ``` gets 19 pairings wrong in this file, because ``` lines that
    // carry a language tag were treated as closers.
    let mut in_fence = false;
    for line in text.lines() {
        if let Some(info) = line.strip_prefix("```") {
            if !in_fence {
                in_fence = true;
            } else if info.trim().is_empty() {
                in_fence = false;
            }
            continue;
        }
        if in_fence {
            continue;
        }
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

/// The sections a branch adds to SKILL.md that did NOT come through the spool.
///
/// `tri skill add` removes a collision that no branch-side check can see: two
/// branches each append `## N.` numbered from their OWN base, both merge, and
/// the number appears twice. The COLLISION is invisible here -- but the practice
/// that causes it is not. `tri skill fold` deletes one spool file for every
/// section it appends; `cat >> SKILL.md` deletes nothing.
///
/// Compared by TITLE, and not by number and not by diff line:
///   - `tri skill renumber` rewrites every number and keeps every title, so a
///     number-set comparison would report the whole file as new.
///   - a `+## N. ` diff line cannot be told from a heading QUOTED inside a fence.
///     3 of the 518 `## N. ` lines on master are quotations, and miscounting one
///     of those has already cost a real section here.
///
/// Returns the new titles and whether the branch is clean. A fold of K lessons
/// deletes K spool files, so K new titles are allowed; a direct append allows 0.
pub fn unspooled(
    base: &[(usize, String)],
    head: &[(usize, String)],
    folded: usize,
) -> (Vec<String>, bool) {
    let was: std::collections::BTreeSet<&str> = base.iter().map(|(_, t)| t.as_str()).collect();
    let new: Vec<String> = head
        .iter()
        .filter(|(_, t)| !was.contains(t.as_str()))
        .map(|(_, t)| t.clone())
        .collect();
    let ok = new.len() <= folded;
    (new, ok)
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

/// `git show <rev>:<path>`, or empty when the path did not exist there.
fn at(rev: &str, path: &str, root: &std::path::Path) -> String {
    let out = std::process::Command::new("git")
        .args(["show", &format!("{rev}:{path}")])
        .current_dir(root)
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => String::new(),
    }
}

/// Whether `now` is `then` cut short, ignoring a difference that is only
/// trailing blank lines.
///
/// Measured across 281 commits of this file: 40 sections had a strict-prefix
/// body, and 38 of them differed by trailing blanks alone -- an artefact of
/// where a section ends, not a loss. Reporting those would bury the two that
/// were real, and both of THOSE turned out to be blocks that moved elsewhere in
/// the document and are still present.
pub fn truncated(then: &[String], now: &[String]) -> bool {
    if now.len() >= then.len() {
        return false;
    }
    if then[..now.len()] != *now {
        return false;
    }
    then[now.len()..].iter().any(|l| !l.trim().is_empty())
}

/// Identifiers that say which entry a line BELONGS to: a wave number and a
/// `gfNN` format name.
///
/// Issue numbers are deliberately excluded. An entry cites other issues as a
/// matter of course -- with `#NNNN` counted, 49 of 312 NOW.md entries flag and
/// the one real case is buried. The prototype for this check appeared to give
/// the right answer with issue numbers in its pattern, and did so only because
/// `\b#` requires a word character immediately before the `#`, so that
/// alternative never matched anything. **A dead alternative produced the right
/// number for the wrong reason**, and the reason is what had to be written down.
pub fn identifiers(text: &str) -> std::collections::BTreeSet<String> {
    let re = regex::Regex::new(
        r"(?i)\bwave[ -]loop[ -](\d+)\b|\bW(\d{3})\b|\bwave-loop-(\d+)\b|\b(gf\d+)\b",
    )
    .expect("static pattern");
    let mut out = std::collections::BTreeSet::new();
    for c in re.captures_iter(text) {
        for i in 1..=4 {
            if let Some(m) = c.get(i) {
                out.insert(m.as_str().to_lowercase());
            }
        }
    }
    out
}

/// Entries whose body names NONE of its own identifiers and DOES name one that
/// another entry's heading owns.
///
/// The shape found on 2026-09-05: `SW-conformance — gf48` carried 39 lines of
/// `Wave Loop 434` boot evidence, fifty lines below its own heading, and never
/// mentioned gf48. An empty-body check cannot see this -- the entry HAS a body,
/// it is simply not its own -- and it is worse than a loss, because the entry
/// claims another's evidence as its own.
///
/// The naive form of this question is useless here: 58 of the 63 NOW.md entries
/// with a wave number name SOME other wave, because an entry routinely points at
/// the next one. What discriminates is naming none of its OWN.
pub fn misattributed(entries: &[(String, Vec<String>)]) -> Vec<(String, Vec<String>)> {
    let mut owned = std::collections::BTreeSet::new();
    for (h, _) in entries {
        owned.extend(identifiers(h));
    }
    let mut out = Vec::new();
    for (h, body) in entries {
        let mine = identifiers(h);
        if mine.is_empty() {
            continue;
        }
        let text = body.join("\n");
        if text.trim().is_empty() {
            continue;
        }
        let theirs = identifiers(&text);
        if mine.intersection(&theirs).next().is_some() {
            continue;
        }
        let foreign: Vec<String> = theirs.intersection(&owned).cloned().collect();
        if !foreign.is_empty() {
            out.push((h.clone(), foreign));
        }
    }
    out
}

/// Every heading and its body, IN ORDER, with nothing collapsed.
///
/// `bodies()` keys by title, which is right for the history walk -- a title is
/// the identity that survives renumbering -- and wrong for any question about
/// the file AS IT STANDS. A repeated heading text is one key there, and the
/// later `insert` OVERWRITES, so only the last copy's body is ever examined.
///
/// Measured 2026-09-05 on `docs/NOW.md`: **312 headings, 310 distinct titles**.
/// `Honesty limits (BINDING)` appears at lines 1479 and 1708, and a
/// `Wave Loop 777` subject at 4504 and 4601. The hollow check ran over 310 seats
/// while four comments in this same file said 312, and it was right only because
/// all four of those occurrences happen to have bodies. **Two questions, two
/// populations -- and the file had said so two passes before the code did.**
pub fn occurrences(text: &str) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    let mut cur: Option<String> = None;
    let mut body: Vec<String> = Vec::new();
    let mut in_fence = false;
    for line in text.lines() {
        if let Some(info) = line.strip_prefix("```") {
            if !in_fence {
                in_fence = true;
            } else if info.trim().is_empty() {
                in_fence = false;
            }
            if cur.is_some() {
                body.push(line.to_string());
            }
            continue;
        }
        let head = if in_fence {
            None
        } else {
            line.strip_prefix("## ")
        };
        if let Some(h) = head {
            if let Some(c) = cur.take() {
                out.push((c, std::mem::take(&mut body)));
            }
            cur = Some(section_key(h));
            continue;
        }
        if cur.is_some() {
            body.push(line.to_string());
        }
    }
    if let Some(c) = cur {
        out.push((c, body));
    }
    out
}

/// Headings with nothing under them.
///
/// The same damage as a truncation and visible with no history at all, so it is
/// the cheaper question. Measured 2026-09-05: `docs/NOW.md` has 2 of 312, at
/// two CONSECUTIVE lines, and `SKILL.md` has 0 of 523.
pub fn hollow_headings(occurrences: &[(String, Vec<String>)]) -> Vec<&String> {
    occurrences
        .iter()
        .filter(|(_, b)| !b.iter().any(|l| !l.trim().is_empty()))
        .map(|(t, _)| t)
        .collect()
}

/// The key a section is tracked by, from its heading text.
///
/// A leading `N. ` is stripped, so renumbering is invisible to any comparison
/// built on this -- that is the whole point, since half of what happens to
/// `SKILL.md` is renumbering. A heading with no number is its own key, which is
/// what makes `docs/NOW.md` -- 312 headings, not one of them numbered -- a
/// population this can see at all.
pub fn section_key(heading: &str) -> String {
    match heading.split_once(". ") {
        Some((n, rest)) if n.parse::<usize>().is_ok() => rest.trim().to_string(),
        _ => heading.trim().to_string(),
    }
}

/// Section bodies, keyed by title, fence-aware.
pub fn bodies(text: &str) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut out = std::collections::BTreeMap::new();
    let mut cur: Option<String> = None;
    let mut body: Vec<String> = Vec::new();
    let mut in_fence = false;
    for line in text.lines() {
        if let Some(info) = line.strip_prefix("```") {
            if !in_fence {
                in_fence = true;
            } else if info.trim().is_empty() {
                in_fence = false;
            }
            if cur.is_some() {
                body.push(line.to_string());
            }
            continue;
        }
        let head = if in_fence {
            None
        } else {
            line.strip_prefix("## ")
        };
        if let Some(h) = head {
            if let Some(c) = cur.take() {
                out.insert(c, std::mem::take(&mut body));
            }
            cur = Some(section_key(h));
            continue;
        }
        if cur.is_some() {
            body.push(line.to_string());
        }
    }
    if let Some(c) = cur {
        out.insert(c, body);
    }
    out
}

fn lost(path: &str, base: &str, gate: bool) -> Result<()> {
    let root = repo_root()?;
    let log = std::process::Command::new("git")
        .args(["log", base, "--format=%H", "--reverse", "--", path])
        .current_dir(&root)
        .output()?;
    let commits: Vec<String> = String::from_utf8_lossy(&log.stdout)
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    println!();
    println!("  {path}");
    println!("  walking {} commit(s) on {base}", commits.len());

    let mut first: std::collections::BTreeMap<String, (String, Vec<String>)> =
        std::collections::BTreeMap::new();
    for c in &commits {
        let text = at(c, path, &root);
        if text.is_empty() {
            continue;
        }
        for (title, body) in bodies(&text) {
            first.entry(title).or_insert((c.clone(), body));
        }
    }
    let now = bodies(&at(base, path, &root));
    // Two questions, two populations, both printed with their unit. Hollow and
    // misattribution are about the file AS IT STANDS, so they run over
    // occurrences; the history walk is about identity across renumbering, so it
    // runs over titles.
    let here = occurrences(&at(base, path, &root));
    println!("  titles ever written    {}", first.len());
    println!("  headings on {base:<11} {}   (## lines)", here.len());
    println!(
        "  distinct titles        {}   (what the history walk compares)",
        now.len()
    );

    // A heading with nothing under it is the same damage, visible WITHOUT any
    // history: whatever was there is gone and the heading is left standing.
    // It is the cheaper question and it is asked first, because the history
    // walk above costs one `git show` per commit and this costs one read.
    //
    // Measured 2026-09-05: `docs/NOW.md` has 2 of 312, at lines 6359 and 6361 --
    // two CONSECUTIVE bare headings -- and `SKILL.md` has 0 of 523.
    let hollow = hollow_headings(&here);
    let wrong = misattributed(&here);
    if hollow.is_empty() {
        println!("  Every heading has a body.");
    } else {
        println!(
            "  {} heading(s) with an EMPTY body on {base}:",
            hollow.len()
        );
        for t in &hollow {
            println!("    {t}");
        }
        println!("  Nothing was written under these. No history was needed to see it.");
    }
    if wrong.is_empty() {
        println!("  Every entry that names an identifier names its own.");
    } else {
        println!();
        println!(
            "  {} entry(s) whose body names NO identifier of its own, and does",
            wrong.len()
        );
        println!("  name one another entry owns:");
        for (h, foreign) in &wrong {
            println!("    {h}");
            println!("      body names: {}", foreign.join(", "));
        }
        println!("  A body that is not its own is worse than a missing one: the entry");
        println!("  claims another's evidence. Measured on this repository before the");
        println!("  2026-09-05 repair: exactly 1, and 0 after.");
    }
    println!();

    let mut cut = Vec::new();
    let mut gone = Vec::new();
    for (title, (sha, then)) in &first {
        match now.get(title) {
            None => gone.push((title.clone(), sha.clone())),
            Some(n) => {
                if truncated(then, n) {
                    cut.push((title.clone(), sha.clone(), then.len(), n.len()));
                }
            }
        }
    }
    println!();
    if cut.is_empty() {
        println!("  No section's body is a truncation of an earlier version.");
    } else {
        println!("  {} section(s) whose body was CUT SHORT:", cut.len());
        for (t, sha, a, b) in &cut {
            println!(
                "    -{:>3} lines ({a} -> {b})  first in {}  {}",
                a - b,
                &sha[..9],
                t
            );
        }
        println!();
        println!("  A cut tail is not by itself a loss. A section's body runs to the next");
        println!("  heading, so a block that later MOVED elsewhere in the document reads");
        println!("  exactly like a truncation -- both hits on SKILL.md were that, and both");
        println!("  are still present. Grep for a distinctive line before calling it damage.");
    }
    if !gone.is_empty() {
        println!();
        println!("  {} title(s) no longer present:", gone.len());
        for (t, sha) in &gone {
            println!("    first in {}  {t}", &sha[..9]);
        }
        println!();
        println!("  A missing TITLE is not by itself a loss: a section can be rewritten");
        println!("  under a longer heading, or withdrawn on purpose. Both happened here.");
        println!("  Read the commit that dropped it before calling it damage.");
    }
    if gate && !cut.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

/// `incoming/` for one skill: the spool a pass appends to.
fn spool_dir(root: &std::path::Path, skill: &str) -> PathBuf {
    root.join(".claude/skills").join(skill).join("incoming")
}

/// A file name from a title: date, then the title lowercased to hyphens.
///
/// The DATE leads so the fold order is the order they were written, and the
/// slug follows so two lessons on one day still take two paths. Truncated at 60
/// characters of slug because a path is not a place to keep a sentence.
fn spool_name(date: &str, title: &str) -> String {
    let mut slug = String::new();
    let mut dash = false;
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            dash = false;
        } else if !dash && !slug.is_empty() {
            slug.push('-');
            dash = true;
        }
    }
    let slug = slug.trim_end_matches('-');
    let slug: String = slug.chars().take(60).collect();
    format!("{date}-{}.md", slug.trim_end_matches('-'))
}

fn add(title: &str, skill: &str) -> Result<()> {
    let root = repo_root()?;
    let dir = spool_dir(&root, skill);
    if !root.join(".claude/skills").join(skill).is_file()
        && !root
            .join(".claude/skills")
            .join(skill)
            .join("SKILL.md")
            .is_file()
    {
        anyhow::bail!(
            "no .claude/skills/{skill}/SKILL.md -- refusing to spool a lesson for a skill that does not exist"
        );
    }
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let date = today_utc();
    let path = dir.join(spool_name(&date, title));
    if path.exists() {
        anyhow::bail!(
            "{} already exists -- pick a different title rather than overwriting a lesson",
            path.display()
        );
    }
    // No number. That is the point: the number is what collides, and it is
    // assigned by `fold` against the SKILL.md that exists at that moment.
    let body =
        format!("## {title}\n\n<!-- write the lesson here; `tri skill fold` numbers it -->\n");
    std::fs::write(&path, body).with_context(|| format!("writing {}", path.display()))?;
    println!("  spooled: {}", path.display());
    println!();
    println!("  Write the lesson into that file. It carries NO number, so another");
    println!("  branch spooling its own lesson writes a different path and the two");
    println!("  cannot collide. `tri skill fold` appends it to SKILL.md and assigns");
    println!("  the number then, against the file as it stands at that moment.");
    Ok(())
}

/// Today, UTC, as `YYYY-MM-DD`.
fn today_utc() -> String {
    let out = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%d"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    out.unwrap_or_else(|| "0000-00-00".to_string())
}

fn fold(check: bool, skill: &str) -> Result<()> {
    let root = repo_root()?;
    let dir = spool_dir(&root, skill);
    let target = root.join(".claude/skills").join(skill).join("SKILL.md");
    if !target.is_file() {
        anyhow::bail!(
            "{} does not exist -- nothing to fold into",
            target.display()
        );
    }
    let mut spooled: Vec<PathBuf> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
            .collect(),
        Err(_) => Vec::new(),
    };
    spooled.sort();
    if spooled.is_empty() {
        println!("  {} holds no spooled lesson.", dir.display());
        println!("  Nothing to fold. This is not a failure: an empty spool is the");
        println!("  normal state between passes.");
        return Ok(());
    }

    let text = std::fs::read_to_string(&target)?;
    let highest = sections(&text).iter().map(|(n, _)| *n).max().unwrap_or(0);

    println!(
        "  {} spooled lesson(s), folding after section {highest}:",
        spooled.len()
    );
    let mut next = highest;
    let mut appended = String::new();
    for p in &spooled {
        let body = std::fs::read_to_string(p)?;
        let Some(first) = body.lines().next() else {
            anyhow::bail!("{} is empty", p.display());
        };
        let Some(title) = first.strip_prefix("## ") else {
            anyhow::bail!(
                "{} does not open with `## <title>` -- got {first:?}. A spooled lesson \
                 carries its title on the first line and no number.",
                p.display()
            );
        };
        if first.strip_prefix("## ").is_some_and(|r| {
            r.split_once(". ")
                .is_some_and(|(n, _)| n.parse::<usize>().is_ok())
        }) {
            anyhow::bail!(
                "{} is already numbered -- the number is assigned here, not when the \
                 lesson is written, and a pre-assigned one is exactly what collides",
                p.display()
            );
        }
        next += 1;
        println!("    {next}. {title}");
        appended.push('\n');
        appended.push_str(&format!("## {next}. {title}\n"));
        for l in body.lines().skip(1) {
            appended.push_str(l);
            appended.push('\n');
        }
    }

    if check {
        println!();
        println!("  --check: nothing written.");
        return Ok(());
    }

    let mut out = text;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&appended);
    std::fs::write(&target, out)?;
    for p in &spooled {
        std::fs::remove_file(p)?;
    }
    println!();
    println!("  folded into {} and the spool is empty.", target.display());
    Ok(())
}

/// Read a file at a rev, distinguishing "absent there" from "git could not run".
fn at_opt(rev: &str, path: &str, root: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["show", &format!("{rev}:{path}")])
        .current_dir(root)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).to_string())
}

/// Spool files this branch deleted -- which is what folding one looks like.
fn folded_here(base: &str, dir: &str, root: &std::path::Path) -> usize {
    let out = std::process::Command::new("git")
        .args([
            "diff",
            "--name-status",
            "--diff-filter=D",
            base,
            "--",
            &format!("{dir}/incoming/"),
        ])
        .current_dir(root)
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).lines().count(),
        _ => 0,
    }
}

fn spooled(base: &str, gate: bool) -> Result<()> {
    let root = repo_root()?;
    let files = skill_files(&root);
    println!("base: {base}   files: {}", files.len());
    let mut offenders = 0usize;
    let mut checked = 0usize;
    for f in &files {
        let path = rel(&root, f);
        let Some(before) = at_opt(base, &path, &root) else {
            // A SKILL.md this branch CREATES cannot collide with a base that has
            // no such file, so this is not an offence -- but say so, because a
            // population that quietly shrinks is how a clean bill of health gets
            // printed over nothing.
            println!("  NEW      {path} -- absent on {base}, nothing to collide with");
            continue;
        };
        let head = std::fs::read_to_string(f).unwrap_or_default();
        let dir = std::path::Path::new(&path)
            .parent()
            .map(|d| d.display().to_string())
            .unwrap_or_default();
        let folded = folded_here(base, &dir, &root);
        let (new, ok) = unspooled(&sections(&before), &sections(&head), folded);
        checked += 1;
        if new.is_empty() {
            println!("  ok       {path} -- adds no section");
            continue;
        }
        if ok {
            println!(
                "  ok       {path} -- {} new section(s), {folded} spool file(s) folded",
                new.len()
            );
            continue;
        }
        offenders += 1;
        println!(
            "  UNSPOOLED {path} -- {} new section(s) but {folded} spool file(s) folded",
            new.len()
        );
        for t in new.iter().take(10) {
            println!("             {t}");
        }
        if new.len() > 10 {
            println!("             ... and {} more", new.len() - 10);
        }
    }
    println!("checked {checked} file(s) that exist on {base}; {offenders} unspooled");
    if offenders > 0 {
        println!();
        println!("A section was appended to SKILL.md directly. Two branches doing that");
        println!("choose the same number and the duplicate appears only after the merge,");
        println!("where no branch-side check can see it. Use the spool instead:");
        println!();
        println!("    tri skill add \"<title>\"      # writes incoming/<date>-<slug>.md");
        println!("    tri skill fold               # appends and numbers, on one branch");
        println!();
        if gate {
            std::process::exit(1);
        }
    }
    Ok(())
}

pub fn run(cmd: &SkillCmd) -> Result<()> {
    let show_gaps = match cmd {
        SkillCmd::Spooled { base, gate } => return spooled(base, *gate),
        SkillCmd::Refs { list } => return refs(*list),
        SkillCmd::Claims {
            list,
            numbers,
            windowed,
        } => return claims(*list, *numbers, *windowed),
        SkillCmd::Lost { file, base, gate } => return lost(file, base, *gate),
        SkillCmd::Add { title, skill } => return add(title, skill),
        SkillCmd::Fold { check, skill } => return fold(*check, skill),
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

    fn secs(v: &[(usize, &str)]) -> Vec<(usize, String)> {
        v.iter().map(|(n, t)| (*n, t.to_string())).collect()
    }

    #[test]
    fn a_direct_append_is_unspooled() {
        use super::unspooled;
        let base = secs(&[(1, "one"), (2, "two")]);
        let head = secs(&[(1, "one"), (2, "two"), (3, "appended by hand")]);
        let (new, ok) = unspooled(&base, &head, 0);
        assert_eq!(new, vec!["appended by hand".to_string()]);
        assert!(!ok, "a section with no spool file behind it must not pass");
    }

    #[test]
    fn a_fold_of_one_spooled_lesson_passes() {
        use super::unspooled;
        let base = secs(&[(1, "one")]);
        let head = secs(&[(1, "one"), (2, "folded")]);
        let (new, ok) = unspooled(&base, &head, 1);
        assert_eq!(new.len(), 1);
        assert!(ok, "one new section against one deleted spool file is a fold");
    }

    #[test]
    fn a_renumber_moves_every_number_and_adds_no_work() {
        use super::unspooled;
        // Why the comparison is by TITLE. `tri skill renumber` rewrites every
        // heading, so comparing NUMBERS would call the whole file new -- and the
        // guard would fire hardest on the one command whose job is to avoid
        // collisions.
        let base = secs(&[(1, "one"), (2, "two"), (3, "three")]);
        let head = secs(&[(7, "one"), (8, "two"), (9, "three")]);
        let (new, ok) = unspooled(&base, &head, 0);
        assert!(new.is_empty(), "renumber adds no title, got {new:?}");
        assert!(ok);
    }

    #[test]
    fn a_withdrawal_removes_and_never_fires() {
        use super::unspooled;
        let base = secs(&[(1, "one"), (2, "two")]);
        let head = secs(&[(1, "one")]);
        let (new, ok) = unspooled(&base, &head, 0);
        assert!(new.is_empty());
        assert!(ok);
    }

    #[test]
    fn folding_two_lessons_needs_two_spool_files() {
        use super::unspooled;
        let base = secs(&[(1, "one")]);
        let head = secs(&[(1, "one"), (2, "a"), (3, "b")]);
        assert!(!unspooled(&base, &head, 1).1, "two sections, one spool file");
        assert!(unspooled(&base, &head, 2).1);
    }

    #[test]
    fn a_heading_quoted_in_a_fence_is_not_a_new_section() {
        use super::{sections, unspooled};
        // A diff-line matcher counts this as an appended section and demands a
        // spool file for it. 3 of the 518 `## N. ` lines on master have exactly
        // this shape, and miscounting one has already destroyed a real section.
        let base_text = "## 1. one\n\nbody\n";
        let quoted = "## 1. one\n\nbody\n\n```text\n## 2. quoted, not a section\n```\n";
        let (new, ok) = unspooled(&sections(base_text), &sections(quoted), 0);
        assert!(new.is_empty(), "a quotation is not a section, got {new:?}");
        assert!(ok);
        // The control, without which the assertion above passes for the wrong
        // reason: the SAME line OUTSIDE a fence must be counted.
        let real = format!("{base_text}\n## 2. quoted, not a section\n");
        let (new, ok) = unspooled(&sections(base_text), &sections(&real), 0);
        assert_eq!(new.len(), 1, "outside a fence this IS a section");
        assert!(!ok);
    }

    #[test]
    fn a_spool_path_is_unique_per_lesson() {
        use super::spool_name;
        // The whole point: two lessons on one day take two paths, so two
        // branches spooling them cannot conflict. The date leads so the fold
        // order is the order they were written.
        let a = spool_name("2026-09-05", "A spooled lesson carries no number");
        let b = spool_name("2026-09-05", "The merge creates the defect");
        assert_ne!(a, b);
        assert_eq!(a, "2026-09-05-a-spooled-lesson-carries-no-number.md");
        // Punctuation collapses to single hyphens and never trails.
        assert_eq!(
            spool_name("2026-01-02", "`X of Y` -- where X can exceed Y!"),
            "2026-01-02-x-of-y-where-x-can-exceed-y.md"
        );
        // A long title is cut, and the cut must not leave a trailing hyphen.
        let long = spool_name("2026-01-02", &"word ".repeat(40));
        assert!(long.len() < 80, "{long}");
        assert!(
            !long.contains("-.md"),
            "a truncation must not leave a dangling hyphen: {long}"
        );
        // Two DIFFERENT dates keep two paths even for one title.
        assert_ne!(
            spool_name("2026-01-02", "same"),
            spool_name("2026-01-03", "same")
        );
    }

    /// `truncated` can be right while `lost` never asks it, and the command
    /// then reports a clean file forever.
    ///
    /// EIGHTH change in eight passes whose surviving mutant was the wiring --
    /// and the first found by mutating the call site BEFORE writing a single
    /// test for the helper, which is the rule the previous seven produced.
    /// `hollow_headings` can be right while `lost` never calls it.
    #[test]
    fn a_quoted_heading_inside_a_fence_is_not_a_section() {
        // `section_bodies` and `section_ranges` had NO fence rule while three
        // other readers in this file did. Measured on the repository's own
        // SKILL.md: `tri skill check` read 532 numbered sections and
        // `tri skill claims` read 535 -- the same file, three apart, because
        // three section headings are QUOTED inside code fences as evidence.
        let doc = "## 1. First\n\
                   body\n\
                   ```\n\
                   ## 2. Quoted as evidence, not a section\n\
                   ```\n\
                   more body\n\
                   ## 3. Second\n\
                   tail\n";
        let got: Vec<usize> = super::section_bodies(doc)
            .iter()
            .map(|(n, _, _)| *n)
            .collect();
        assert_eq!(got, vec![1, 3], "the fenced `## 2.` is a quotation");
        let ranges: Vec<usize> = super::section_ranges(doc)
            .iter()
            .map(|(n, _, _)| *n)
            .collect();
        assert_eq!(
            ranges, got,
            "the two readers must walk the headings by one rule"
        );

        // CommonMark: an OPENING fence may carry an info string, a CLOSING one
        // may not. A naive toggle mispairs, and it mispairs enough of this
        // repository's own SKILL.md to read 441 sections where there are 532.
        use super::fence_toggle;
        assert_eq!(
            fence_toggle("```rust", false),
            Some(true),
            "info string opens"
        );
        assert_eq!(
            fence_toggle("```", true),
            Some(false),
            "bare backticks close"
        );
        assert_eq!(
            fence_toggle("```text", true),
            Some(true),
            "an info string cannot CLOSE a fence -- this is where a naive toggle goes wrong"
        );
        assert_eq!(fence_toggle("ordinary line", false), None);
    }

    #[test]
    fn lost_actually_reports_hollow_headings() {
        let src = include_str!("skillnum.rs");
        let boundary = src
            .lines()
            .position(|l| l == "#[cfg(test)]")
            .expect("the test module is a line of its own");
        let code: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");
        // Pinned to the POPULATION, not merely to the call. Asking this over
        // the title map answers it for the last copy of a repeated heading, and
        // that is the defect this argument fixes.
        let call = concat!("let hollow = hollow_", "headings(&here);");
        assert!(
            code.contains(call),
            "the hollow question runs over OCCURRENCES, not the collapsed title map"
        );
        let src_of_here = concat!("let here = occur", "rences(&at(base, path, &root));");
        assert!(
            code.contains(src_of_here),
            "and `here` is the uncollapsed list"
        );
    }

    #[test]
    fn lost_actually_consults_truncated() {
        let src = include_str!("skillnum.rs");
        let boundary = src
            .lines()
            .position(|l| l == "#[cfg(test)]")
            .expect("the test module is a line of its own");
        let code: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");
        let call = concat!("if trunc", "ated(then, n) {");
        assert!(
            code.contains(call),
            "without this the walk runs, finds nothing by construction, and \
             prints a clean bill of health for any file"
        );
    }

    /// The shape found on 2026-09-05: an entry whose body is another entry's.
    /// An empty-body check cannot see it -- the entry HAS a body.
    #[test]
    fn a_body_that_names_none_of_its_own_ids_is_flagged() {
        let mut e: Vec<(String, Vec<String>)> = Vec::new();
        // The real body: `XADC_LIVE_W434_...` alone does NOT match, because the
        // `_` before `W434` is a word character and the pattern is anchored on a
        // word boundary. The first fixture used only that form and the test
        // failed -- correctly. What made the live case detectable is the plain
        // `wave-loop-434` on its branch line.
        e.push((
            "SW-conformance — gf48 promoted".to_string(),
            vec![
                "- Branch: `wave-loop-434`".to_string(),
                "- Added XADC_LIVE_W434_OPERATING_POINT".to_string(),
            ],
        ));
        e.push((
            "Wave Loop 434 — boot evidence".to_string(),
            vec!["- Branch: wave-loop-434".to_string()],
        ));
        let out = super::misattributed(&e);
        assert_eq!(
            out.len(),
            1,
            "only the entry carrying another's id: {out:?}"
        );
        assert!(out[0].0.starts_with("SW-conformance — gf48"));
        assert_eq!(out[0].1, vec!["434".to_string()]);
    }

    /// The naive form of this question is useless here: 58 of the 63 NOW.md
    /// entries with a wave number name SOME other wave, because an entry
    /// routinely points at the next one. Naming its OWN is what clears it --
    /// and all 58 of them do, which is why the refined check returns 0 on the
    /// repaired file and 1 on the damaged one.
    #[test]
    fn naming_a_neighbour_is_not_misattribution() {
        let mut e: Vec<(String, Vec<String>)> = Vec::new();
        e.push((
            "Wave Loop 889 close-out".to_string(),
            vec![
                "- Branch: `wave-loop-889`, follows Wave Loop 888, next Wave Loop 890".to_string(),
            ],
        ));
        e.push((
            "Wave Loop 888 close-out".to_string(),
            vec!["- body".to_string()],
        ));
        e.push((
            "Wave Loop 890 close-out".to_string(),
            vec!["- body".to_string()],
        ));
        assert!(
            super::misattributed(&e).is_empty(),
            "889 names its own number, so pointing at 888 and 890 is a cross-reference"
        );
    }

    /// An id no entry OWNS is a reference to something outside the document --
    /// a sibling repository's wave, a format the file never wrote up. Flagging
    /// it would accuse an entry of carrying a body that does not exist here.
    #[test]
    fn an_id_no_heading_owns_is_not_evidence() {
        let mut e: Vec<(String, Vec<String>)> = Vec::new();
        e.push((
            "SW-conformance — gf48 promoted".to_string(),
            vec!["- see wave-loop-999 in trinity-fpga".to_string()],
        ));
        e.push((
            "Wave Loop 434 — boot".to_string(),
            vec!["- wave-loop-434".to_string()],
        ));
        assert!(
            super::misattributed(&e).is_empty(),
            "999 belongs to no entry here, so nothing was taken from anything"
        );
    }

    /// `misattributed` can be right while `lost` never calls it.
    #[test]
    fn lost_actually_reports_misattribution() {
        let src = include_str!("skillnum.rs");
        let boundary = src
            .lines()
            .position(|l| l == "#[cfg(test)]")
            .expect("the test module is a line of its own");
        let code: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");
        let call = concat!("let wrong = misattrib", "uted(&here);");
        assert!(
            code.contains(call),
            "misattribution is a fact about an entry as it stands, so it runs \
             over occurrences too"
        );
    }

    /// Issue numbers are excluded on purpose. With `#NNNN` counted, 49 of 312
    /// NOW.md entries flag and the one real case is buried. The prototype
    /// appeared to give the right answer WITH them in its pattern, and only
    /// because `\b#` requires a word character before the `#` -- a dead
    /// alternative that produced the right number for the wrong reason.
    #[test]
    fn an_issue_number_is_not_an_ownership_identifier() {
        let ids = super::identifiers("Closes #1358 and refers to #1702");
        assert!(ids.is_empty(), "issue numbers are not ownership: {ids:?}");
        let ids = super::identifiers("Wave Loop 434 and gf48 and W431");
        assert_eq!(
            ids.iter().cloned().collect::<Vec<_>>(),
            vec!["431".to_string(), "434".to_string(), "gf48".to_string()],
            "waves and formats are"
        );
    }

    /// The key strips a leading number so RENUMBERING is invisible -- half of
    /// what happens to SKILL.md is renumbering, and a rename it is not. A
    /// heading with no number is its own key, which is what makes docs/NOW.md
    /// visible at all: 312 headings there and not one of them numbered.
    #[test]
    fn the_key_ignores_a_number_and_keeps_everything_else() {
        assert_eq!(super::section_key("553. The audit"), "The audit");
        assert_eq!(super::section_key("1. A"), "A");
        assert_eq!(
            super::section_key("fix(freeze): reseal FROZEN_HASH (Closes #2316)"),
            "fix(freeze): reseal FROZEN_HASH (Closes #2316)",
            "an unnumbered heading is its own key, or NOW.md has no population"
        );
        assert_eq!(
            super::section_key("2026-04-08 — CI stabilization"),
            "2026-04-08 — CI stabilization",
            "a leading token that is not `N. ` is not a number to strip"
        );
        assert_eq!(
            super::section_key("v1. Something"),
            "v1. Something",
            "`v1` is not a number, so nothing is stripped -- otherwise any \
             heading with a dotted prefix silently loses it and two different \
             sections can collide on one key"
        );
        assert_eq!(
            super::section_key("fix(x): a. b"),
            "fix(x): a. b",
            "and the split must be anchored at the START, not at any `. `"
        );
    }

    /// A repeated heading text is ONE key in `bodies()` and the later insert
    /// OVERWRITES, so only the last copy's body is ever examined. Asking the
    /// hollow question over that map answers it for a heading that is not the
    /// one being reported.
    ///
    /// Measured on `docs/NOW.md`: 312 headings, 310 distinct titles. The check
    /// ran over 310 seats while four comments in this file said 312, and it was
    /// right only because all four colliding occurrences happen to have bodies.
    #[test]
    fn a_repeated_heading_is_two_seats_not_one() {
        let src = "## A\n\n## A\nbody\n";
        assert_eq!(super::bodies(src).len(), 1, "the title map collapses them");
        assert_eq!(
            super::occurrences(src).len(),
            2,
            "the file has two headings"
        );

        // The first copy is bare and the second is not. Over TITLES the map
        // keeps only the last body, so the answer is "every heading has a body"
        // -- false of the file.
        let by_title: Vec<(String, Vec<String>)> = super::bodies(src).into_iter().collect();
        assert!(
            super::hollow_headings(&by_title).is_empty(),
            "the collapsed map sees no bare heading, which is the defect"
        );

        let occ = super::occurrences(src);
        let hollow = super::hollow_headings(&occ);
        assert_eq!(
            hollow.len(),
            1,
            "over occurrences the bare copy is found: {hollow:?}"
        );

        // Mirrored: the LAST copy is the bare one. The title map now happens to
        // be right, for a reason that has nothing to do with the file.
        let mirrored = super::occurrences("## A\nbody\n\n## A\n");
        assert_eq!(mirrored.len(), 2);
        assert_eq!(super::hollow_headings(&mirrored).len(), 1);
    }

    /// A heading with nothing under it is the same damage as a truncation, and
    /// visible without any history. Measured: docs/NOW.md has 2 of 312, at two
    /// CONSECUTIVE lines; SKILL.md has 0 of 523.
    #[test]
    fn a_heading_with_no_body_is_found_without_history() {
        let src = "## 1. Has one\nbody\n\n## 2. Has none\n\n## 3. Also has one\ntext\n";
        let b = super::bodies(src);
        let b: Vec<(String, Vec<String>)> = b.into_iter().collect();
        assert_eq!(
            super::hollow_headings(&b),
            vec!["Has none"],
            "blank lines are not a body: {b:?}"
        );
    }

    /// The signature of the loss that actually happened: a body cut short.
    /// An edit in place is not a prefix; a truncation is.
    #[test]
    fn a_truncation_is_a_prefix_and_an_edit_is_not() {
        let then: Vec<String> = ["a", "b", "c", "d"].iter().map(|s| s.to_string()).collect();
        let cut: Vec<String> = ["a", "b"].iter().map(|s| s.to_string()).collect();
        assert!(super::truncated(&then, &cut), "the tail was dropped");
        let edited: Vec<String> = ["a", "X", "c", "d"].iter().map(|s| s.to_string()).collect();
        assert!(
            !super::truncated(&then, &edited),
            "same length, changed in place"
        );
        let shorter_but_different: Vec<String> = ["a", "X"].iter().map(|s| s.to_string()).collect();
        assert!(
            !super::truncated(&then, &shorter_but_different),
            "shorter AND different is a rewrite, not a cut"
        );
        assert!(!super::truncated(&then, &then), "unchanged is not a cut");
    }

    /// 38 of the 40 prefix hits across 281 commits differed by trailing blank
    /// lines alone -- an artefact of where a section ends. Reporting those would
    /// bury the two that were real.
    #[test]
    fn trailing_blanks_alone_are_not_a_truncation() {
        let then: Vec<String> = ["a", "b", "", ""].iter().map(|s| s.to_string()).collect();
        let now: Vec<String> = ["a", "b"].iter().map(|s| s.to_string()).collect();
        assert!(!super::truncated(&then, &now), "only blanks went");
        let then2: Vec<String> = ["a", "b", "", "real"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            super::truncated(&then2, &now),
            "a blank AND a real line went, so it is a cut"
        );
    }

    /// A body runs to the next NUMBERED heading, and a heading inside a fenced
    /// block is a quotation -- the same rule `sections` uses, because a body
    /// that stops at a quoted heading is how the loss happened in the first
    /// place.
    #[test]
    fn a_body_runs_to_the_next_numbered_heading_and_ignores_quotations() {
        let src = "## 1. One\nalpha\n\n```\n## 9. Quoted\n```\nbeta\n\n## 2. Two\ngamma\n";
        let b = super::bodies(src);
        assert_eq!(b.len(), 2, "the quoted heading starts nothing: {b:?}");
        let one = &b["One"];
        assert!(
            one.iter().any(|l| l == "beta"),
            "the body continues PAST the quoted heading: {one:?}"
        );
        assert!(
            one.iter().any(|l| l.contains("## 9. Quoted")),
            "and the quotation stays inside it as evidence"
        );
        assert_eq!(b["Two"].iter().filter(|l| *l == "gamma").count(), 1);
    }

    /// A heading inside a fenced block is a QUOTATION, and counting it cost a
    /// real section: `tri skill renumber` cut its tail at a quoted title,
    /// dropped the section containing it, and left the opening fence unclosed
    /// so the next section was swallowed too. Measured on master 4d63859: 518
    /// lines match `## N. ` and 3 of them are quoted.
    #[test]
    fn a_heading_inside_a_fence_is_a_quotation() {
        let src = "## 1. Real\n\n```\n## 2. Quoted\n## 3. Quoted\n```\n\n## 4. Also real\n";
        let secs = super::sections(src);
        assert_eq!(
            secs.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
            vec![1, 4],
            "the two inside the block are evidence, not sections: {secs:?}"
        );
    }

    /// CommonMark: an OPENING fence may carry an info string, a CLOSING fence
    /// may not. A naive toggle on every ``` mispairs 19 fences in the real
    /// file, because ``` lines carrying a language tag were read as closers.
    #[test]
    fn an_info_string_marks_an_opener_not_a_closer() {
        // The shape that actually occurs: a fenced block QUOTING output that
        // itself contains a ``` line with an info string. This file has 19 of
        // them. Treating that inner line as a closer flips the parity for
        // everything after it.
        let src = "## 1. Real\n\n```\nsome output\n``` numbers\nmore output\n```\n\n## 2. Real\n";
        let secs = super::sections(src);
        assert_eq!(
            secs.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
            vec![1, 2],
            "the inner ``` carries an info string, so it is not a closer; if it \
             is taken for one the block re-opens and swallows the next heading: {secs:?}"
        );
    }

    /// An unclosed fence must swallow what follows rather than silently
    /// re-admitting it: that is the state the real file was left in, and the
    /// count is what made it visible.
    #[test]
    fn an_unclosed_fence_swallows_the_rest() {
        let src = "## 1. Real\n\n```\n## 2. Swallowed\n\n## 3. Swallowed too\n";
        let secs = super::sections(src);
        assert_eq!(
            secs.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
            vec![1],
            "nothing after an unclosed fence is a section, and a shrinking count \
             is exactly how the damage was found: {secs:?}"
        );
    }
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
/// CommonMark's fence rule, in one place.
///
/// An OPENING fence may carry an info string (` ```rust `); a CLOSING fence may
/// not. A naive toggle on every ``` line therefore mispairs, and it mispairs 19
/// of them in this repository's own SKILL.md.
///
/// Returns the new `in_fence` when this line IS a fence marker, `None` when it
/// is an ordinary line.
///
/// It lived open-coded in three readers and was MISSING from two -- and those
/// two, `section_bodies` and `section_ranges`, counted 535 numbered sections in
/// SKILL.md where the fence-aware readers see 441. Ninety-four of them are
/// section headings QUOTED inside code fences, five of them counted twice. One
/// definition, five callers, so the next repair cannot reach only some of them.
pub fn fence_toggle(line: &str, in_fence: bool) -> Option<bool> {
    let info = line.strip_prefix("```")?;
    Some(if !in_fence {
        true
    } else if info.trim().is_empty() {
        false
    } else {
        in_fence
    })
}

pub fn section_bodies(text: &str) -> Vec<(usize, String, String)> {
    let mut out: Vec<(usize, String, String)> = Vec::new();
    let mut cur: Option<(usize, String, Vec<&str>)> = None;
    let mut in_fence = false;
    for line in text.lines() {
        if let Some(next) = fence_toggle(line, in_fence) {
            in_fence = next;
            if let Some((_, _, body)) = cur.as_mut() {
                body.push(line);
            }
            continue;
        }
        let head = if in_fence {
            None
        } else {
            line.strip_prefix("## ")
        }
        .and_then(|rest| {
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
    let mut in_fence = false;
    for (i, line) in text.lines().enumerate() {
        last = i + 1;
        if let Some(next) = fence_toggle(line, in_fence) {
            in_fence = next;
            continue;
        }
        let head = if in_fence {
            None
        } else {
            line.strip_prefix("## ")
        }
        .and_then(|rest| {
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

/// How a section is pointed at from prose.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RefKind {
    /// `&sect;179` or `§179` -- unambiguous, this file's own convention.
    Symbol,
    /// `section 179` / `Section 179` -- the same pointer written out. Counted apart
    /// because the words can also appear about something that is not this file, and
    /// a reader deserves to see which population a dangling count came from.
    Word,
}

/// Every cross-reference in a chunk of skill prose, as `(number, kind)`.
///
/// Deliberately does NOT read `## 179.` headings as references: a heading is the
/// target, not a pointer at one, and counting it would make every section resolve to
/// itself.
pub fn references(text: &str) -> Vec<(usize, RefKind)> {
    let mut out = Vec::new();
    for line in text.lines() {
        if line.starts_with("## ") {
            continue;
        }
        let low = line.to_lowercase();
        for (needle, kind) in [
            ("&sect;", RefKind::Symbol),
            ("\u{a7}", RefKind::Symbol),
            ("section ", RefKind::Word),
        ] {
            for (i, _) in low.match_indices(needle) {
                let rest = low[i + needle.len()..].trim_start();
                let n: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(v) = n.parse::<usize>() {
                    out.push((v, kind));
                }
            }
        }
    }
    out
}

/// A reference written with no number at all: `(&sect;—the same rule …)`.
///
/// One of these is real and in the file. It resolves to nothing and never will, and a
/// count of dangling NUMBERS cannot see it, because there is no number to fail to
/// resolve.
pub fn empty_references(text: &str) -> usize {
    let mut n = 0;
    for line in text.lines() {
        if line.starts_with("## ") {
            continue;
        }
        for needle in ["&sect;", "\u{a7}"] {
            for (i, _) in line.match_indices(needle) {
                let rest = &line[i + needle.len()..];
                if !rest.trim_start().starts_with(|c: char| c.is_ascii_digit()) {
                    n += 1;
                }
            }
        }
    }
    n
}

fn refs(list: bool) -> Result<()> {
    let root = repo_root()?;
    let files = skill_files(&root);
    println!("CROSS-REFERENCES IN THE SKILLS, AND WHETHER THEY RESOLVE\n");
    let mut any_dead = false;
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        let have: std::collections::BTreeSet<usize> =
            section_bodies(&text).iter().map(|(n, _, _)| *n).collect();
        if have.is_empty() {
            continue;
        }
        let name = f
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let all = references(&text);
        let dead: Vec<&(usize, RefKind)> = all.iter().filter(|(n, _)| !have.contains(n)).collect();
        let empty = empty_references(&text);
        let sym = all.iter().filter(|r| r.1 == RefKind::Symbol).count();
        println!("  {name}");
        println!("    sections                    {}", have.len());
        println!(
            "    references                  {}   ({sym} by symbol, {} written out)",
            all.len(),
            all.len() - sym
        );
        println!("    with no number at all       {empty}   a count of dangling NUMBERS cannot see these");
        let mut nums: Vec<usize> = dead.iter().map(|(n, _)| *n).collect();
        nums.sort_unstable();
        nums.dedup();
        println!(
            "    POINTING AT NOTHING         {}   across {} distinct numbers",
            dead.len(),
            nums.len()
        );
        if !nums.is_empty() {
            any_dead = true;
            println!("      never existed: {nums:?}");
            for (i, line) in text.lines().enumerate() {
                if line.starts_with("## ") {
                    continue;
                }
                for (n, _) in references(line) {
                    if !have.contains(&n) {
                        let t = line.trim();
                        println!("      {}:{}  {}", name, i + 1, &t[..t.len().min(92)]);
                        break;
                    }
                }
            }
        }
        if list {
            let mut seen: Vec<usize> = all.iter().map(|(n, _)| *n).collect();
            seen.sort_unstable();
            seen.dedup();
            println!("      all targets: {seen:?}");
        }
    }

    println!(
        "\n  A pointer at a section that does not exist is not a broken link; it is a\n  \
         claim about what this file says, and the claim is false. The numbers here are\n  \
         the fingerprint of a renumbering: a block of consecutive missing targets means\n  \
         the sections moved and the pointers did not.\n\n  \
         References written OUT (`section 179`) are counted apart from the symbol form\n  \
         because the words can also be about a document that is not this one -- so a\n  \
         dangling count drawn only from the symbols is the conservative reading, and\n  \
         both are printed rather than merged."
    );
    if any_dead {
        println!(
            "\n  Reported, not failed: fixing a pointer means deciding what it MEANT,\n  \
                  and that is a reading, not a rename."
        );
    }
    Ok(())
}

#[cfg(test)]
mod reference_tests {
    use super::{empty_references, references, RefKind};

    #[test]
    fn both_spellings_of_a_pointer_are_found_and_kept_apart() {
        let t = "Related: &sect;234 for the ruler, and section 179 says otherwise.\n";
        assert_eq!(
            references(t),
            vec![(234, RefKind::Symbol), (179, RefKind::Word)]
        );
    }

    /// A HEADING is the target, not a pointer at one. Counting it would make every
    /// section resolve to itself and the dangling count would always be zero.
    /// The literal section sign is the same pointer as the HTML entity, and both are
    /// the SYMBOL form. A mutation filing the sign under the written-out form survived
    /// until a fixture used the character itself.
    #[test]
    fn the_section_sign_and_the_entity_are_one_kind() {
        assert_eq!(
            references("see \u{a7}234 now\n"),
            vec![(234, RefKind::Symbol)]
        );
        assert_eq!(
            references("see &sect;234 now\n"),
            vec![(234, RefKind::Symbol)]
        );
    }

    #[test]
    fn a_heading_is_not_a_reference() {
        assert!(references("## 179. A `--limit` on a run list\n").is_empty());
        assert_eq!(
            references("## 179. See &sect;234\n"),
            vec![],
            "the whole heading line is skipped, targets included"
        );
    }

    /// Several pointers on one line all count -- the first version searched with
    /// `find` and would have seen only one, which is the defect section 465 records.
    #[test]
    fn every_pointer_on_a_line_counts_not_the_first() {
        let t =
            "Related: &sect;234 for the ruler, &sect;235 for the check, &sect;241 for the guard.\n";
        assert_eq!(references(t).len(), 3);
    }

    /// `(&sect;—the same rule …)` resolves to nothing and never will. A count of
    /// dangling NUMBERS cannot see it, because there is no number to fail.
    #[test]
    fn a_pointer_with_no_number_is_counted_separately() {
        let t = "(&sect;\u{2014}the same rule the widths ledger states)\n";
        assert!(references(t).is_empty(), "no number, so no target");
        assert_eq!(empty_references(t), 1);
    }

    #[test]
    fn a_numbered_pointer_is_not_an_empty_one() {
        assert_eq!(empty_references("see &sect;234 and \u{a7}235\n"), 0);
        assert_eq!(
            empty_references("## 234. heading &sect;x\n"),
            0,
            "headings skipped"
        );
    }

    /// The word form needs the space, or `sections` and `sectional` would each
    /// contribute a phantom pointer to whatever digits followed.
    #[test]
    fn the_word_form_requires_the_separator() {
        assert_eq!(references("section 12 says\n"), vec![(12, RefKind::Word)]);
        assert!(references("sections12 says\n").is_empty());
        // The discriminating input, and the only one the trailing space earns its
        // place on: digits glued straight to the word. Without the space this reads
        // as a pointer at section 12; a mutation dropping it survived until this
        // line existed.
        assert!(
            references("section12 says\n").is_empty(),
            "`section12` is a word, not a pointer at 12"
        );
        assert!(references("section twelve says\n").is_empty());
    }
}

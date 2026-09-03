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
use std::path::PathBuf;

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
        SkillCmd::Claims { list } => return claims(*list),
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

fn claims(list: bool) -> Result<()> {
    let root = repo_root()?;
    let files = skill_files(&root);
    // Name what is outside the population rather than leaving it silent: this
    // reads `.claude/skills/*/SKILL.md`, and the repository tracks SKILL.md
    // files under other roots that no command here has ever opened.
    let mut unread: Vec<String> = Vec::new();
    if let Ok(out) = std::process::Command::new("git")
        .args(["ls-files", "*SKILL.md"])
        .current_dir(&root)
        .output()
    {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if !line.starts_with(".claude/skills/") {
                unread.push(line.to_string());
            }
        }
    }
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
            if !states_a_figure(&title) {
                continue;
            }
            carrying += 1;
            fcar += 1;
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

    println!("FIGURES IN THE KNOWLEDGE BASE, AND WHAT COULD RE-TAKE THEM\n");
    println!("  SKILL.md files read           {}", files.len());
    for f in &unread {
        println!("    NOT read (outside .claude/skills): {f}");
    }
    println!("  numbered sections             {total}");
    println!("  stating a figure              {carrying}");
    println!("  of those, naming a command    {with_cmd}");
    println!("  of those, anchored (dated)    {anchored}");
    println!("  free to go stale              {}", free.len());
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

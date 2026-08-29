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
                    .map(|t| format!("\"{}\"", &t[..t.len().min(46)]))
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
    let SkillCmd::Check { gaps: show_gaps } = cmd;
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
        if *show_gaps && !g.is_empty() {
            println!(
                "      gaps (reported, not a failure): {}",
                g.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
    println!();
    if failed > 0 {
        println!("  {failed} numbering problem(s). Two sections with one number is what a");
        println!("  merge that keeps BOTH sides produces, and nothing else notices it.");
        std::process::exit(1);
    }
    println!("  Numbering holds in {} file(s).", files.len());
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

    #[test]
    fn a_clean_run_has_nothing_to_say() {
        let secs = parse("## 1. A\n## 2. B\n## 3. C\n");
        assert!(problems(&secs).is_empty());
        assert!(gaps(&secs).is_empty());
    }
}

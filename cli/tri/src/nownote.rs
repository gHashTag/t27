//! `tri now` -- write a docs/now/ entry without hand-writing the frame.
//!
//! Every pull request in this repository must add an entry (the
//! check-now-freshness gate), and the entry format is rigid enough that
//! writing it by hand invites drift: a forgotten date, a heading that does
//! not match the section, a missing issue reference. One forgotten entry
//! cost a full gate round trip. This stamps the frame; the caller supplies
//! only the content.
//!
//! Entries are one file per unit of work, `docs/now/<YYYY-MM-DD>-<slug>.md`.
//! This used to prepend to the single file docs/NOW.md, which meant every PR
//! rewrote the same first line and GitHub marked every concurrent PR
//! CONFLICTING. Writing a distinct path removes the shared line entirely, and
//! the writer gets simpler: a create, not a read-modify-write.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum NowCmd {
    /// Write a docs/now/ entry: title, bullets, optional issue ref.
    Add {
        /// Entry title, used for both the page heading and the section.
        title: String,
        /// Bullet lines, repeatable. At least one is required — an entry
        /// with no content is exactly the vacuous touch the gate invites.
        ///
        /// `allow_hyphen_values` because entry text legitimately starts with a
        /// flag name: writing this command's own NOW entry failed on the
        /// bullet describing `--expect`, which clap read as an argument.
        #[arg(long = "bullet", required = true, allow_hyphen_values = true)]
        bullets: Vec<String>,
        /// Issue number for the section's "(Closes #N)" suffix.
        #[arg(long)]
        closes: Option<u64>,
    },
}

pub fn run(cmd: &NowCmd) -> Result<()> {
    match cmd {
        NowCmd::Add {
            title,
            bullets,
            closes,
        } => add(title, bullets, *closes),
    }
}

fn repo_root() -> Result<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("git is not installed or not on PATH")?;
    if !out.status.success() {
        anyhow::bail!("not inside a git repository");
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

/// Today's date from the local clock, YYYY-MM-DD, with no chrono dependency:
/// `git` is already a hard requirement of this command and its author date
/// formatting is stable.
fn today() -> Result<String> {
    let out = std::process::Command::new("date")
        .args(["+%Y-%m-%d"])
        .output()
        .context("date is not available")?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Filename-safe slug: lowercase, runs of non-alphanumerics collapsed to a
/// single `-`, trimmed, capped so paths stay readable. The gate's filename
/// pattern is `[A-Za-z0-9._-]+`, and this stays well inside it.
fn slugify(title: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            // Non-ASCII and punctuation alike become a separator. Dropping the
            // character rather than transliterating keeps the filename ASCII,
            // which the repo's L3 PURITY gate requires of added lines anyway.
            pending_dash = true;
        }
    }
    // Cap at 60 chars, then trim a trailing dash the cut may have exposed.
    const MAX: usize = 60;
    if out.len() > MAX {
        out.truncate(MAX);
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn add(title: &str, bullets: &[String], closes: Option<u64>) -> Result<()> {
    let date = today()?;
    let slug = slugify(title);
    if slug.is_empty() {
        anyhow::bail!(
            "title {title:?} has no ASCII alphanumerics, so it yields an empty filename slug; \
             give the entry a title that can name a file"
        );
    }

    let dir = repo_root()?.join("docs").join("now");
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let path = dir.join(format!("{date}-{slug}.md"));

    // Refuse to clobber. Two entries on one day are fine -- they just need
    // distinct titles. Silently overwriting, or silently appending a numeric
    // suffix, would both hide a duplicate that is nearly always a mistake.
    if path.exists() {
        anyhow::bail!(
            "{} already exists; give this entry a distinct title",
            path.display()
        );
    }

    let suffix = match closes {
        Some(n) => format!(" (Closes #{n})"),
        None => String::new(),
    };
    // No `Last updated:` line: the filename carries the date, and a second copy
    // inside the file is exactly the duplicated line the old layout fought over.
    let mut entry = format!("# NOW -- {title} ({date})\n\n## {title}{suffix}\n\n");
    for b in bullets {
        entry.push_str(&format!("- {b}\n"));
    }
    std::fs::write(&path, entry).with_context(|| format!("write {}", path.display()))?;
    println!("wrote NOW entry: {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::slugify;

    #[test]
    fn slug_lowercases_and_joins_words() {
        assert_eq!(
            slugify("Retire the NOW.md bottleneck"),
            "retire-the-now-md-bottleneck"
        );
    }

    #[test]
    fn slug_collapses_runs_of_punctuation() {
        assert_eq!(
            slugify("fix(ci):  now --  sync gate!!"),
            "fix-ci-now-sync-gate"
        );
    }

    #[test]
    fn slug_has_no_leading_or_trailing_dash() {
        let s = slugify("  ...leading and trailing...  ");
        assert!(!s.starts_with('-'), "{s:?}");
        assert!(!s.ends_with('-'), "{s:?}");
        assert_eq!(s, "leading-and-trailing");
    }

    #[test]
    fn slug_is_capped_and_still_clean() {
        let s = slugify(&"word ".repeat(40));
        assert!(s.len() <= 60, "len {} for {s:?}", s.len());
        assert!(!s.ends_with('-'), "{s:?}");
    }

    #[test]
    fn slug_drops_non_ascii() {
        assert_eq!(slugify("ternary \u{2014} node"), "ternary-node");
    }

    #[test]
    fn slug_empty_when_no_alphanumerics() {
        assert_eq!(slugify("--- ... ---"), "");
    }

    /// The filename this produces must satisfy the CI gate's own pattern.
    #[test]
    fn slug_matches_gate_filename_pattern() {
        let re = regex::Regex::new(r"^docs/now/[0-9]{4}-[0-9]{2}-[0-9]{2}-[A-Za-z0-9._-]+\.md$")
            .unwrap();
        for title in [
            "Retire the NOW.md bottleneck",
            "fix(ci): now -- sync gate!!",
            "wave 913: GF16 conformance",
        ] {
            let name = format!("docs/now/2026-08-20-{}.md", slugify(title));
            assert!(re.is_match(&name), "gate would reject {name:?}");
        }
    }
}

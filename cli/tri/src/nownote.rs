//! `tri now` — prepend a docs/NOW.md entry without hand-writing the frame.
//!
//! Every pull request in this repository must touch docs/NOW.md (the
//! check-now-freshness gate), and the entry format is rigid enough that
//! writing it by hand invites drift: a forgotten date, a heading that does
//! not match the section, a missing issue reference. One forgotten entry
//! cost a full gate round trip. This stamps the frame; the caller supplies
//! only the content.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum NowCmd {
    /// Prepend an entry to docs/NOW.md: title, bullets, optional issue ref.
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

fn add(title: &str, bullets: &[String], closes: Option<u64>) -> Result<()> {
    let path = repo_root()?.join("docs").join("NOW.md");
    let old = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let date = today()?;
    let suffix = match closes {
        Some(n) => format!(" (Closes #{n})"),
        None => String::new(),
    };
    let mut entry =
        format!("# NOW -- {title} ({date})\n\nLast updated: {date}\n\n## {title}{suffix}\n\n");
    for b in bullets {
        entry.push_str(&format!("- {b}\n"));
    }
    entry.push('\n');
    std::fs::write(&path, entry + &old).with_context(|| format!("write {}", path.display()))?;
    println!("prepended NOW entry: {title} ({date})");
    Ok(())
}

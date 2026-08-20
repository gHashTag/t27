//! `tri hooks ...` — pure-Rust ports of repository commit / push gates.
//!
//! Replaces the Bash gates that previously lived in `.claude/hooks/`. The
//! original `.sh` files now forward to these subcommands so any existing
//! harness wiring keeps working without re-introducing logic in shell.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use clap::Subcommand;
use chrono::Utc;
use regex::Regex;

#[derive(Subcommand, Debug)]
pub enum HooksCmd {
    /// Run every migrated commit-time gate in sequence (l1-check + now-gate).
    PreCommit,
    /// L1 TRACEABILITY: last commit message must reference an issue
    /// (`Closes #N` / `Fixes #N` / `Resolves #N` / `Reference #N`).
    L1Check,
    /// Verify `docs/NOW.md` "Last updated" line matches today's UTC date.
    NowGate {
        /// Path to NOW.md. Defaults to `docs/NOW.md` under repo root.
        #[arg(long)]
        path: Option<PathBuf>,
        /// Override the expected "today" (YYYY-MM-DD) for tests / CI.
        #[arg(long)]
        today: Option<String>,
    },
    /// Session-start guard for the Claude Code harness. Emits a one-line
    /// status string to stdout; never blocks (the Bash gate is a soft
    /// telemetry hook).
    SessionGate,
}

pub fn run(cmd: &HooksCmd) -> Result<()> {
    match cmd {
        HooksCmd::PreCommit => pre_commit(),
        HooksCmd::L1Check => l1_check(),
        HooksCmd::NowGate { path, today } => now_gate(path.as_deref(), today.as_deref()),
        HooksCmd::SessionGate => session_gate(),
    }
}

fn pre_commit() -> Result<()> {
    now_gate(None, None)?;
    l1_check()?;
    println!("tri hooks pre-commit: PASSED");
    Ok(())
}

pub fn l1_check() -> Result<()> {
    let out = Command::new("git")
        .args(["log", "-1", "--pretty=%B", "HEAD"])
        .output()
        .context("failed to invoke `git log -1`")?;
    if !out.status.success() {
        bail!("git log -1 exited with {:?}", out.status);
    }
    let msg = String::from_utf8(out.stdout).context("commit message is not UTF-8")?;
    check_commit_message(&msg)?;
    Ok(())
}

fn check_commit_message(msg: &str) -> Result<()> {
    let re = Regex::new(r"(?i)(Closes|Fixes|Resolves|Reference)\s+#(\d+)")
        .expect("static regex always compiles");
    match re.captures(msg) {
        Some(caps) => {
            let issue = caps.get(2).map(|m| m.as_str()).unwrap_or("?");
            println!("L1 PASSED: Issue #{} referenced", issue);
            Ok(())
        }
        None => {
            eprintln!("L1 VIOLATION: Commit missing issue reference");
            eprintln!("Commit message: {}", msg.trim());
            eprintln!("Required pattern: Closes #N | Fixes #N | Resolves #N | Reference #N");
            Err(anyhow!("L1 traceability violation"))
        }
    }
}

pub fn now_gate(path: Option<&Path>, today_override: Option<&str>) -> Result<()> {
    let resolved: PathBuf = match path {
        Some(p) => p.to_path_buf(),
        None => repo_root()?.join("docs/NOW.md"),
    };

    let body = std::fs::read_to_string(&resolved)
        .with_context(|| format!("read {}", resolved.display()))?;

    let expected = match today_override {
        Some(s) => s.to_string(),
        None => Utc::now().format("%Y-%m-%d").to_string(),
    };

    // Match the format the producer actually writes. `nownote.rs` emits a
    // PLAIN `Last updated: <date>` line, and all 136 stamps in `docs/NOW.md`
    // use that form -- zero use the bold one this pattern required before, so
    // the gate could never take the `Some` branch on the real document. The
    // `**` markers stay optional because archived snapshots (and the entry
    // still sitting in root `NOW.md`) predate the switch to plain.
    let re = Regex::new(r"(?m)^(?:\*\*)?Last updated:(?:\*\*)?\s*(\d{4}-\d{2}-\d{2})")
        .expect("static regex always compiles");
    match re.captures(&body) {
        Some(caps) => {
            let got = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            if got != expected {
                bail!(
                    "NOW gate violation: docs/NOW.md `Last updated: {}` != expected `{}`",
                    got,
                    expected
                );
            }
            println!("NOW gate PASSED: Last updated = {}", got);
            Ok(())
        }
        None => bail!(
            "NOW gate violation: no `Last updated: YYYY-MM-DD` line found in {}",
            resolved.display()
        ),
    }
}

fn session_gate() -> Result<()> {
    let root = repo_root().unwrap_or_else(|_| PathBuf::from("."));
    let id_file = root.join(".trinity/current_task/.notebook_id");
    if id_file.is_file() {
        let id = std::fs::read_to_string(&id_file)
            .with_context(|| format!("read {}", id_file.display()))?;
        let id = id.trim();
        if id.is_empty() {
            println!("session: no notebook id");
        } else {
            println!("session: notebook={}", id);
        }
    } else {
        println!("session: gate disabled (no .notebook_id file)");
    }
    Ok(())
}

fn repo_root() -> Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("invoke git rev-parse")?;
    if !out.status.success() {
        bail!("git rev-parse exited with {:?}", out.status);
    }
    let s = String::from_utf8(out.stdout).context("repo root not UTF-8")?;
    Ok(PathBuf::from(s.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l1_accepts_closes() {
        assert!(check_commit_message("feat: foo\n\nCloses #592\n").is_ok());
    }

    #[test]
    fn l1_accepts_fixes_case_insensitive() {
        assert!(check_commit_message("fix: bar\n\nfixes #1\n").is_ok());
    }

    #[test]
    fn l1_rejects_refs() {
        assert!(check_commit_message("feat: foo\n\nRefs #1\n").is_err());
    }

    #[test]
    fn l1_rejects_bare_hash() {
        assert!(check_commit_message("feat: foo\n\n#1\n").is_err());
    }

    #[test]
    fn now_gate_accepts_today_override() {
        let tmp = std::env::temp_dir().join(format!("now_gate_ok_{}.md", std::process::id()));
        std::fs::write(&tmp, "# x\n\n**Last updated:** 2026-05-12\n").unwrap();
        let r = now_gate(Some(&tmp), Some("2026-05-12"));
        std::fs::remove_file(&tmp).ok();
        assert!(r.is_ok(), "{:?}", r);
    }

    #[test]
    fn now_gate_rejects_stale_date() {
        let tmp = std::env::temp_dir().join(format!("now_gate_stale_{}.md", std::process::id()));
        std::fs::write(&tmp, "# x\n\n**Last updated:** 2025-01-01\n").unwrap();
        let r = now_gate(Some(&tmp), Some("2026-05-12"));
        std::fs::remove_file(&tmp).ok();
        assert!(r.is_err());
    }

    /// The two tests above write their own fixture in the bold form, so they
    /// only ever proved the regex is self-consistent. This one pins the shape
    /// `nownote.rs` actually writes (see its `add()`): a PLAIN `Last updated:`
    /// line. It fails against the pre-fix bold-only pattern.
    #[test]
    fn now_gate_accepts_the_plain_form_nownote_writes() {
        let date = "2026-05-12";
        let body = "# NOW -- some entry (2026-05-12)\n\
                    \n\
                    Last updated: 2026-05-12\n\
                    \n\
                    ## some entry (Closes #1)\n\
                    \n\
                    - x\n\n";
        let tmp = std::env::temp_dir().join(format!("now_gate_plain_{}.md", std::process::id()));
        std::fs::write(&tmp, body).unwrap();
        let r = now_gate(Some(&tmp), Some(date));
        std::fs::remove_file(&tmp).ok();
        assert!(r.is_ok(), "{:?}", r);
    }

    /// Liveness: run the gate against the real `docs/NOW.md` rather than a
    /// fixture. The expected date is re-derived with the *live* gate's own
    /// rule (`bootstrap/src/suite.rs` takes the first line containing
    /// "Last updated:"), so this asserts the two implementations agree on the
    /// actual document. It does not assert freshness, so it cannot go red
    /// merely because the file is a day old.
    #[test]
    fn now_gate_agrees_with_the_live_gate_on_the_real_document() {
        let root = match repo_root() {
            Ok(r) => r,
            Err(_) => return, // not in a git checkout; nothing to check
        };
        let path = root.join("docs/NOW.md");
        let body = match std::fs::read_to_string(&path) {
            Ok(b) => b,
            Err(_) => return, // file absent (sparse checkout); nothing to check
        };
        let stamped = body
            .lines()
            .find(|l| l.contains("Last updated:"))
            .and_then(|l| l.split("Last updated:").nth(1))
            .map(|s| s.trim().trim_start_matches("**").trim().to_string())
            .expect("docs/NOW.md must carry a `Last updated:` line");
        let date = stamped
            .get(..10)
            .expect("`Last updated:` value must start with YYYY-MM-DD");
        let r = now_gate(Some(&path), Some(date));
        assert!(r.is_ok(), "gate rejected the real docs/NOW.md: {:?}", r);
    }
}

//! `tri hooks ...` — pure-Rust ports of repository commit / push gates.
//!
//! Replaces the Bash gates that previously lived in `.claude/hooks/`. The
//! original `.sh` files now forward to these subcommands so any existing
//! harness wiring keeps working without re-introducing logic in shell.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use clap::Subcommand;
use regex::Regex;

#[derive(Subcommand, Debug)]
pub enum HooksCmd {
    /// Run every migrated commit-time gate in sequence (l1-check + now-gate).
    PreCommit,
    /// L1 TRACEABILITY: last commit message must reference an issue
    /// (`Closes #N` / `Fixes #N` / `Resolves #N` / `Reference #N`).
    L1Check,
    /// Verify a fresh `docs/now/<YYYY-MM-DD>-<slug>.md` entry exists.
    NowGate {
        /// Entries directory. Defaults to `docs/now` under repo root.
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

/// Require a fresh entry under `docs/now/`.
///
/// This previously parsed `^\*\*Last updated:\*\*` out of docs/NOW.md. That
/// regex demanded a BOLD label; `tri now` has only ever written the plain
/// `Last updated:` form, and `docs/NOW.md` contains zero bold occurrences --
/// every stamp in it is plain. The gate could therefore never pass on a real
/// checkout -- it was dead code that looked like enforcement. Entries now carry
/// their date in the filename, so the check is a directory listing with nothing
/// to misparse.
///
/// The accepted window is `expected -1 .. expected +1` day, matching
/// scripts/ci/now-sync-gate-diff.sh exactly. A local gate that is stricter than
/// CI rejects work CI would take, which is how contributors learn to skip it.
pub fn now_gate(path: Option<&Path>, today_override: Option<&str>) -> Result<()> {
    let dir: PathBuf = match path {
        Some(p) => p.to_path_buf(),
        None => repo_root()?.join("docs/now"),
    };

    let expected = match today_override {
        Some(s) => s.to_string(),
        None => Utc::now().format("%Y-%m-%d").to_string(),
    };
    let center = chrono::NaiveDate::parse_from_str(&expected, "%Y-%m-%d")
        .with_context(|| format!("expected date {expected:?} is not YYYY-MM-DD"))?;
    let lo = (center - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    let hi = (center + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let entries = std::fs::read_dir(&dir)
        .with_context(|| format!("read entries directory {}", dir.display()))?;

    let re = Regex::new(r"^(\d{4}-\d{2}-\d{2})-[A-Za-z0-9._-]+\.md$")
        .expect("static regex always compiles");

    let mut newest: Option<String> = None;
    for ent in entries {
        let ent = ent.context("read directory entry")?;
        let name = ent.file_name().to_string_lossy().to_string();
        let Some(caps) = re.captures(&name) else {
            continue;
        };
        let date = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        // ISO-8601 zero-padded dates compare correctly as strings.
        if date.as_str() >= lo.as_str() && date.as_str() <= hi.as_str() {
            println!("NOW gate PASSED: {} ({})", name, date);
            return Ok(());
        }
        let is_newer = match newest.as_deref() {
            None => true,
            Some(n) => date.as_str() > n,
        };
        if is_newer {
            newest = Some(date);
        }
    }

    bail!(
        "NOW gate violation: no entry in {} dated within {} .. {} \
         (newest found: {}). Write one with: tri now add \"<title>\" --bullet \"<what changed>\"",
        dir.display(),
        lo,
        hi,
        newest.as_deref().unwrap_or("<none>")
    )
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

    /// Build a throwaway `docs/now`-shaped directory holding `names`.
    fn entries_dir(tag: &str, names: &[&str]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("now_gate_{}_{}", tag, std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        for n in names {
            std::fs::write(dir.join(n), "# entry\n\n- did a thing\n").unwrap();
        }
        dir
    }

    #[test]
    fn now_gate_accepts_entry_dated_today() {
        let dir = entries_dir("today", &["2026-05-12-some-title.md"]);
        let r = now_gate(Some(&dir), Some("2026-05-12"));
        std::fs::remove_dir_all(&dir).ok();
        assert!(r.is_ok(), "{:?}", r);
    }

    /// The window matches CI: yesterday and tomorrow both pass, so a
    /// contributor east of UTC is not rejected while UTC lags a day.
    #[test]
    fn now_gate_accepts_adjacent_days() {
        for name in ["2026-05-11-yesterday.md", "2026-05-13-tomorrow.md"] {
            let dir = entries_dir("adjacent", &[name]);
            let r = now_gate(Some(&dir), Some("2026-05-12"));
            std::fs::remove_dir_all(&dir).ok();
            assert!(r.is_ok(), "{name} should pass: {r:?}");
        }
    }

    #[test]
    fn now_gate_rejects_stale_entry() {
        let dir = entries_dir("stale", &["2025-01-01-ancient.md"]);
        let r = now_gate(Some(&dir), Some("2026-05-12"));
        std::fs::remove_dir_all(&dir).ok();
        assert!(r.is_err());
    }

    #[test]
    fn now_gate_rejects_empty_directory() {
        let dir = entries_dir("empty", &[]);
        let r = now_gate(Some(&dir), Some("2026-05-12"));
        std::fs::remove_dir_all(&dir).ok();
        assert!(r.is_err());
    }

    /// A README or any other non-entry file must not satisfy the gate.
    #[test]
    fn now_gate_ignores_undated_files() {
        let dir = entries_dir("readme", &["README.md", "notes.md"]);
        let r = now_gate(Some(&dir), Some("2026-05-12"));
        std::fs::remove_dir_all(&dir).ok();
        assert!(r.is_err(), "undated files must not pass: {r:?}");
    }

    /// Liveness. Every other test in this module builds its own throwaway
    /// fixture, so between them they only prove the gate is self-consistent.
    /// This one runs the gate against the REAL `docs/now/` directory in the
    /// checkout and cross-checks it against the OTHER implementation of the
    /// same rule, `scripts/ci/now-sync-gate-diff.sh`, whose entry pattern is
    /// duplicated below on purpose so the two are compared rather than shared.
    ///
    /// It replaces `now_gate_agrees_with_the_live_gate_on_the_real_document`,
    /// which read `docs/NOW.md` as a FILE and cannot survive this change --
    /// `now_gate` now takes a directory, and `read_dir` on a file is ENOTDIR.
    /// That test was the only one here touching the real repository, so it is
    /// re-established in the directory form rather than dropped.
    ///
    /// HONEST LIMITATION: `docs/now/` does not exist on master -- this very PR
    /// creates it. So on the merge-base this test would have no tracked state
    /// to read, and what it asserts against today is the directory this PR
    /// itself adds. From the merge commit onward it is a true liveness test of
    /// tracked repository state; on this branch it is a test of the branch's
    /// own new content. It is written to fail, not skip, on a missing or
    /// non-conforming directory, because `now_gate(None, ..)` in `pre_commit`
    /// hard-requires that directory in production -- a test that shrugged
    /// where production bails would be weaker than the thing it guards.
    ///
    /// It deliberately does NOT assert freshness: the expected date is taken
    /// from the newest entry present, not from `Utc::now()`, so it cannot go
    /// red tomorrow merely because nobody has written an entry today.
    #[test]
    fn now_gate_agrees_with_the_ci_gate_on_the_real_entries_directory() {
        let root = match repo_root() {
            Ok(r) => r,
            Err(_) => return, // not a git checkout (e.g. vendored build); nothing to check
        };
        let dir = root.join("docs/now");
        assert!(
            dir.is_dir(),
            "docs/now/ must exist and be a directory: {}",
            dir.display()
        );

        // The pattern from scripts/ci/now-sync-gate-diff.sh (ENTRY_RE), minus
        // its `docs/now/` prefix, restated independently of `now_gate`'s own
        // regex. If the two ever drift, the gate a contributor runs locally
        // and the gate CI runs stop agreeing, and this fails.
        let ci_re = Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}-[A-Za-z0-9._-]+\.md$")
            .expect("static regex always compiles");

        let mut newest: Option<(String, PathBuf)> = None;
        for ent in std::fs::read_dir(&dir).expect("docs/now/ must be readable") {
            let ent = ent.expect("read docs/now/ entry");
            let name = ent.file_name().to_string_lossy().to_string();
            if !ci_re.is_match(&name) {
                continue; // README.md and friends are not entries
            }
            let date = name[..10].to_string();
            let is_newer = match newest.as_ref() {
                None => true,
                Some((n, _)) => date.as_str() > n.as_str(),
            };
            if is_newer {
                newest = Some((date, ent.path()));
            }
        }

        let (date, path) = newest.expect(
            "docs/now/ must contain at least one entry named <YYYY-MM-DD>-<slug>.md; \
             the CI gate (scripts/ci/now-sync-gate-diff.sh) accepts nothing else",
        );

        // The assertion that matters: the real, tracked directory satisfies the
        // real gate. A regex change that stops matching the names actually on
        // disk turns this red even though every fixture test still passes.
        let r = now_gate(Some(dir.as_path()), Some(date.as_str()));
        assert!(
            r.is_ok(),
            "gate rejected the real docs/now/ (newest entry {date}): {r:?}"
        );

        // CI additionally requires a heading and a bullet in the qualifying
        // entry. `now_gate` does not look inside the file, so an entry can pass
        // locally and still be rejected by CI. Pin that the shipped entry
        // satisfies both, otherwise the local gate is quietly the weaker one.
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let heading = Regex::new(r"(?m)^#{1,6} +\S").expect("static regex always compiles");
        let bullet = Regex::new(r"(?m)^[-*] +\S").expect("static regex always compiles");
        assert!(
            heading.is_match(&body),
            "{} has no Markdown heading; CI would reject it",
            path.display()
        );
        assert!(
            bullet.is_match(&body),
            "{} has no bullet; CI would reject it as a vacuous touch",
            path.display()
        );
    }
}

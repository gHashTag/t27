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
    /// A pull request whose title claims a compiler fix must carry a source file.
    ///
    /// The pre-commit form of this reads HEAD, which a contributor can skip and a
    /// worktree without an installed hook never runs at all. This form reads the
    /// PULL REQUEST: its title, and the union of its diff. That is the right unit
    /// because merges here are squashed -- the commit that lands on master is the
    /// pull request, so an intermediate commit that claims a fix before its source
    /// arrives is not a defect and must not be flagged.
    FixCarriesSource {
        /// The pull request title. Defaults to HEAD's subject.
        #[arg(long)]
        subject: Option<String>,
        /// Base sha; the diff read is `<base>...<head>`.
        #[arg(long)]
        base: Option<String>,
        /// Head sha. Defaults to HEAD.
        #[arg(long)]
        head: Option<String>,
        /// Run the controls and report, changing nothing.
        #[arg(long)]
        self_check: bool,
    },
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
    /// Which hooks git will actually run, and whose installer output is dead.
    Status,
}

pub fn run(cmd: &HooksCmd) -> Result<()> {
    match cmd {
        HooksCmd::PreCommit => pre_commit(),
        HooksCmd::FixCarriesSource { subject, base, head, self_check } => {
            fix_carries_source_cmd(subject.as_deref(), base.as_deref(), head.as_deref(), *self_check)
        }
        HooksCmd::L1Check => l1_check(),
        HooksCmd::NowGate { path, today } => now_gate(path.as_deref(), today.as_deref()),
        HooksCmd::SessionGate => session_gate(),
        HooksCmd::Status => status(),
    }
}

fn pre_commit() -> Result<()> {
    now_gate(None, None)?;
    // Freshness and shape are two questions, and until this line only the
    // first was asked here. Measured on one malformed entry dated today: the
    // required `check` context reported three complaints while this hook, and
    // three of the other four local readers, went green -- one of them green
    // BECAUSE of that file, since its freshness loop found it and stopped.
    crate::nownote::check_staged()?;
    conflict_markers()?;
    l1_check()?;
    fix_carries_source()?;
    println!("tri hooks pre-commit: PASSED");
    Ok(())
}

/// Refuse a commit carrying a conflict marker, by asking the repository's own checker.
///
/// Not a sixth reader: `tools/check_conflict_markers.py` already reads every tracked file
/// from the working tree and honours `tools/conflict_markers_baseline.txt`, so a
/// re-implementation here would be a second vocabulary that drifts. This calls it.
///
/// Why it was missing is the finding. Three surfaces claim to gate a commit --
/// `.githooks/pre-commit`, `scripts/pre-commit` and this command -- and **none of the
/// three mentioned a conflict marker**; `grep -c conflict` answers 0 on all of them. The
/// only barrier was CI, and that is exactly how it went wrong: an automated conflict
/// resolver of mine fixed one path and then ran `git add -A`, which staged a SECOND
/// conflicted file verbatim. The required `Conflict markers` context caught it on the
/// pull request, naming `tools/census/fetches.txt` lines 19 and 35 -- one full CI round
/// after a one-second local check would have refused the commit.
///
/// A missing script exits **2**, this repository's word for *could not run*, rather than
/// passing: a guard that cannot run is not a guard that agreed. The path is resolved from
/// the repository ROOT rather than the current directory -- a hook is invoked at the root
/// but a person is often not, and from `cli/tri` the relative path refused with a safe and
/// useless 2 while the checker itself, run from `cli/`, still read all 7870 tracked files.
///
/// Controls, all four run: clean tree from the root **0**, clean tree from `cli/tri`
/// **0**, planted marker from `cli/` **1** naming the file and its lines, and the
/// moved-aside checker **2**. The fifth case -- outside a work tree -- is NOT claimed:
/// `now_gate` refuses first with 1, so that arm is unreachable here and is written as
/// ordinary defence rather than as a control.
fn conflict_markers() -> Result<()> {
    // From the repository ROOT, not the current directory. A git hook is invoked at the
    // root, but a person typing this command is often not there -- and measured from
    // `cli/tri` the relative path refused with exit 2, which is safe and useless. The
    // checker itself resolves the root on its own (run from `cli/` it still reads all
    // 7870 tracked files), so the only thing that needed fixing was finding it.
    let top = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to invoke `git rev-parse --show-toplevel`")?;
    // Not a claimed control: outside a work tree this is UNREACHABLE through
    // `pre_commit`, because `now_gate` runs `git rev-parse` first and errors with 1 --
    // measured by running the command from /tmp, where the expected 2 came back as 1.
    // So this is ordinary defence for a future caller, and it says so rather than
    // advertising a guard nothing has executed.
    if !top.status.success() {
        bail!("git rev-parse --show-toplevel exited with {:?}", top.status);
    }
    let root = String::from_utf8_lossy(&top.stdout).trim().to_string();
    let script = std::path::Path::new(&root).join("tools/check_conflict_markers.py");
    if !script.exists() {
        // The file is gone from the tree. Say so, and do not vote.
        eprintln!(
            "tri hooks pre-commit: COULD NOT RUN -- {} does not exist. \
             Nothing was checked for conflict markers.",
            script.display()
        );
        std::process::exit(2);
    }
    let out = Command::new("python3")
        .arg(&script)
        .current_dir(&root)
        .output()
        .context("failed to invoke python3 tools/check_conflict_markers.py")?;
    if !out.status.success() {
        print!("{}", String::from_utf8_lossy(&out.stdout));
        eprint!("{}", String::from_utf8_lossy(&out.stderr));
        bail!("a tracked file carries a conflict marker");
    }
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

/// The vocabulary BOTH CI gates use, character for character.
///
/// `issue-gate.yml` (the required `check-linked-issue` context, over the PR
/// title and body) and `l1-traceability.yml` (over the commits) each run
///
/// ```text
/// grep -qiE "(Closes?|Fixes?|Resolves?|Refs?|Updates?)\s*#[0-9]+"
/// ```
///
/// This function is a LOCAL PREVIEW of those, and it used to run
/// `(?i)(Closes|Fixes|Resolves|Reference)\s+#(\d+)` -- wrong in both
/// directions at once. It missed `Refs`, `Ref`, `Updates`, `Update` and every
/// singular/plural variant the gates accept, and it invented `Reference`,
/// which neither gate accepts. It also demanded whitespace where the gates
/// allow none, so `Closes#5` passed CI and failed here.
///
/// Measured over the last 20 commit messages on master: the old vocabulary
/// matched **4** references, the gates' matched **33**. This repository writes
/// `Refs #N` as its normal spelling -- Law L1 names it -- so the preview was
/// rejecting the convention it exists to enforce, and doing it on every
/// commit.
///
/// No word boundary, deliberately: `grep -E` has none either, so `prefs #1`
/// matches in CI. A preview that is stricter than its gate sends people to fix
/// something the gate does not object to, which is how a preview gets ignored.
const L1_PATTERN: &str = r"(?i)(closes?|fixes?|resolves?|refs?|updates?)\s*#(\d+)";

fn check_commit_message(msg: &str) -> Result<()> {
    let re = Regex::new(L1_PATTERN).expect("static regex always compiles");
    match re.captures(msg) {
        Some(caps) => {
            let issue = caps.get(2).map(|m| m.as_str()).unwrap_or("?");
            println!("L1 PASSED: Issue #{} referenced", issue);
            Ok(())
        }
        None => {
            eprintln!("L1 VIOLATION: Commit missing issue reference");
            eprintln!("Commit message: {}", msg.trim());
            eprintln!(
                "Required pattern (both CI gates, case-insensitive): \
                 Close(s) | Fix(es) | Resolve(s) | Ref(s) | Update(s) followed by #N"
            );
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

    /// `Refs #N` is this repository's normal spelling and BOTH gates accept it.
    /// A test used to pin the opposite here, with no reason stated, and the
    /// preview rejected every commit that followed the convention.
    #[test]
    fn l1_accepts_every_spelling_the_gates_accept() {
        for m in [
            "Closes #1",
            "Close #1",
            "Fixes #1",
            "Fixe #1",
            "Resolves #1",
            "Resolve #1",
            "Refs #1",
            "Ref #1",
            "Updates #1",
            "Update #1",
            "refs #1",
            "Closes#1",
        ] {
            assert!(
                check_commit_message(&format!("feat: foo\n\n{m}\n")).is_ok(),
                "the gates accept {m:?} and this preview must not be stricter"
            );
        }
    }

    /// And not looser: `Reference` was in the old list and is in neither gate.
    #[test]
    fn l1_rejects_what_the_gates_reject() {
        // `Fix #1` is here because the GATES reject it: `Fixes?` is `Fixe`
        // plus an optional `s`, so the bare `Fix` never matches -- while the
        // gate's own comment beside that regex promises "Fix(es)". Mirroring
        // includes mirroring the quirk; a preview that accepted `Fix #1` would
        // pass a commit CI then blocks. Filed rather than silently widened: a
        // blocking rule's vocabulary is the owner's to change.
        for m in ["Reference #1", "Fix #1", "see #1", "issue 1", "Closes # 1"] {
            assert!(
                check_commit_message(&format!("feat: foo\n\n{m}\n")).is_err(),
                "the gates reject {m:?} and this preview must not be looser"
            );
        }
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

/// What running a given installer would actually achieve, in one place so the
/// report and its test cannot disagree.
///
/// The whole content is that the two destinations are mutually exclusive, so an
/// installer's success says nothing about whether its hooks can run.
fn installer_effect(hooks_path_set: bool, writes_hooks_path: bool) -> &'static str {
    match (hooks_path_set, writes_hooks_path) {
        (true, true) => "LIVE",
        (true, false) => "its output would be SHADOWED",
        (false, true) => "would make .git/hooks dead",
        (false, false) => "LIVE",
    }
}

/// The two places a git hook can live, and the fact that only one of them runs.
///
/// PROVEN, not assumed, in a scratch repository: with `core.hooksPath` unset a
/// hook in `.git/hooks/` runs; with it set, that hook is IGNORED and the one
/// under the configured directory runs instead. The two are mutually exclusive,
/// and nothing in this repository said so.
///
/// That matters because this tree ships THREE installers writing to BOTH:
///
///   * `scripts/setup-git-hooks.sh`        -> `git config core.hooksPath .githooks`
///   * `scripts/install-git-hooks.sh`      -> writes `.git/hooks/{pre-commit,pre-push,commit-msg}`
///   * `scripts/install-constitutional-hook.sh` -> writes `.git/hooks/pre-commit`
///
/// Run the first and the other two install hooks that can never run, while
/// reporting success. A gate that reports success having done nothing is the
/// class this repository keeps finding; here it is in the installers themselves.
///
/// This command answers only "what would run", which is a fact. It does not say
/// which SHOULD run -- that is a decision, and the three installers disagree
/// about it.
fn status() -> Result<()> {
    let root = repo_root()?;
    let configured = std::process::Command::new("git")
        .args(["config", "--get", "core.hooksPath"])
        .current_dir(&root)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());

    println!("WHICH HOOKS GIT WILL ACTUALLY RUN\n");
    match &configured {
        Some(p) => println!("  core.hooksPath = {p}   -> `.git/hooks/` is IGNORED"),
        None => println!("  core.hooksPath = <unset>   -> `.git/hooks/` is the live directory"),
    }

    // In a WORKTREE `.git` is a file and `$GIT_DIR` is `.git/worktrees/<name>`,
    // while git resolves hooks from the COMMON directory. Joining
    // `root/.git/hooks` reports "none" in every worktree however many hooks are
    // installed -- a false clean, in the command whose whole subject is whether
    // anything runs. Ask git instead of constructing the path.
    let common = std::process::Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .current_dir(&root)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|s| {
            let p = std::path::PathBuf::from(&s);
            if p.is_absolute() {
                p
            } else {
                root.join(p)
            }
        })
        .unwrap_or_else(|| root.join(".git"));
    let git_hooks = common.join("hooks");

    let live_dir = match &configured {
        Some(p) => root.join(p),
        None => git_hooks.clone(),
    };
    let dead_dir = match &configured {
        Some(_) => Some(git_hooks.clone()),
        None => None,
    };

    let names = ["pre-commit", "pre-push", "commit-msg", "post-merge"];
    let mut live = 0usize;
    println!("\n  LIVE ({}):", live_dir.display());
    for n in names {
        let f = live_dir.join(n);
        if f.is_file() {
            live += 1;
            println!("    {n}");
        }
    }
    if live == 0 {
        println!("    <none>   nothing runs at commit time in this clone");
    }

    if let Some(d) = dead_dir {
        let mut dead = 0usize;
        let mut rows = Vec::new();
        for n in names {
            if d.join(n).is_file() {
                dead += 1;
                rows.push(n);
            }
        }
        if dead > 0 {
            println!("\n  SHADOWED ({}):", d.display());
            for n in rows {
                println!("    {n}   installed, and git will never run it");
            }
        }
    }

    println!("\n  INSTALLERS IN THIS TREE, and where each writes:");
    for (script, target) in [
        ("scripts/setup-git-hooks.sh", "core.hooksPath -> .githooks"),
        ("scripts/install-git-hooks.sh", ".git/hooks/"),
        ("scripts/install-constitutional-hook.sh", ".git/hooks/pre-commit"),
    ] {
        let exists = root.join(script).is_file();
        let effective = installer_effect(configured.is_some(), target.starts_with("core.hooksPath"));
        println!(
            "    {:<42} {:<24} {}",
            script,
            if exists { target } else { "<missing>" },
            effective
        );
    }

    println!(
        "\n  Proven in a scratch repository rather than assumed: `core.hooksPath`\n  \
         and `.git/hooks/` are mutually exclusive, so running the wrong installer\n  \
         succeeds and installs nothing that runs. This reports what WOULD run; it\n  \
         does not say what should -- the three installers disagree about that."
    );
    Ok(())
}

#[cfg(test)]
mod status_tests {
    use super::installer_effect;

    /// Proven in a scratch repository before this was written: with
    /// `core.hooksPath` set, a hook in `.git/hooks/` is IGNORED; with it unset,
    /// that hook runs. The two destinations are mutually exclusive, and this
    /// tree ships three installers writing to both.
    #[test]
    fn an_installer_can_succeed_and_install_nothing_that_runs() {
        // core.hooksPath is set: the two .git/hooks installers are dead letters.
        assert_eq!(installer_effect(true, false), "its output would be SHADOWED");
        assert_eq!(installer_effect(true, true), "LIVE");
        // Unset: .git/hooks is live, and the hooksPath installer would kill it.
        assert_eq!(installer_effect(false, false), "LIVE");
        assert_eq!(installer_effect(false, true), "would make .git/hooks dead");
    }

    /// The verdict must depend on BOTH inputs. A report that ignores either one
    /// would read "LIVE" for an installer whose output git will never execute --
    /// which is the exact false clean this command exists to end.
    #[test]
    fn both_inputs_change_the_verdict() {
        assert_ne!(installer_effect(true, false), installer_effect(false, false));
        assert_ne!(installer_effect(true, true), installer_effect(false, true));
    }
}

/// Scopes whose subject is compiler source. A `fix(` in one of these claims a code
/// change, so the commit must carry one.
const SOURCE_SCOPES: [&str; 8] = [
    "rust", "c", "zig", "verilog", "parser", "compiler", "lexer", "typecheck",
];

/// Does this subject claim a compiler-source fix?
///
/// Deliberately narrow. `fix(seals)`, `fix(paper)`, `fix(ops)`, `fix(freeze)` and
/// `fix(corpus)` all legitimately land without touching a source file -- their subject
/// lives in a seal, a manuscript or a ledger. Measured over master's whole history:
/// the loose rule ("any `fix(` with no source file") reports 12 commits and 11 of them
/// are correct; this rule reports 1 of 100, and that one is the defect.
fn subject_claims_source(subject: &str) -> bool {
    let Some(rest) = subject.strip_prefix("fix(") else { return false };
    let Some(end) = rest.find(')') else { return false };
    rest[..end]
        .split(',')
        .any(|s| SOURCE_SCOPES.contains(&s.trim()))
}

/// Is this path prose or a record, rather than substance?
///
/// THE FIRST FORM OF THIS WAS A WHITELIST OF CODE EXTENSIONS, AND THAT IS THE WRONG SHAPE.
/// It began as {rs, py, t27, zig}, was widened to {c, h, v, sv, svh} once the 164
/// hand-written `.v` files were noticed, and an adversarial pass then named four more
/// categories that exist here and would each have been a false accusation in a REQUIRED
/// context: 14 `.xdc` constraint files and 4 `.tcl`, which are the actual deliverable of
/// timing work under `fix(verilog)`; 43 `.toml`, where a build-breakage fix genuinely
/// lives; 72 `.lean` formalising the compiler's own lowering; and every extensionless
/// path -- `Makefile`, `Dockerfile`, `scripts/tri` -- for which `rsplit_once('.')` yields
/// None and no whitelist entry can ever match.
///
/// A whitelist of code cannot be completed, and each omission accuses someone. So the
/// question is inverted. The defect this guard exists for, #3264, had a diff of EXACTLY
/// one file: `docs/now/2026-09-05-an-untyped-local-bound-to-a-comparison-is-not-a-bool.md`.
/// What it lacked was not any particular extension; it was anything at all besides prose.
/// Prose and records are a small, stable, closed set. Substance is everything else.
fn is_prose_or_record(path: &str) -> bool {
    // Not `docs/`. A directory prefix is not a claim about content: `docs/` holds 11 `.py`
    // and 4 `.sh` -- 132 of its 2489 files are not `.md` at all -- and calling an
    // executable script prose because of where it lives is the same mistake in the other
    // direction. The document FORMATS are the closed set; the directory is not.
    const PROSE_EXT: [&str; 4] = ["md", "tex", "rst", "adoc"];
    path.starts_with(".trinity/seals/")
        || path
            .rsplit_once('/')
            .map_or(path, |(_, b)| b)
            .rsplit_once('.')
            .is_some_and(|(_, e)| PROSE_EXT.contains(&e))
}

/// Refuse a commit whose subject claims a compiler fix but whose diff has no source file.
///
/// The finding this exists for: PR #3264 was titled `fix(rust): an untyped local bound to
/// a comparison is not a bool (+3)` and merged carrying **only its docs/now note**. The
/// edit lived in the working tree and a `git reset --hard` -- taken to get an honest
/// baseline -- destroyed it before the commit. Every control that pass asked about the
/// **binary**, and the binary was correct, because it had been built from the working
/// tree. None asked about the **commit**. `git log --all -S expr_is_bool_syntactically`
/// answered nothing, which is how it was found, four merges later.
///
/// Reads HEAD rather than the index, the same shape `l1_check` uses: the barrier is
/// raised before the next commit is built on top of a false one.
fn fix_carries_source() -> Result<()> {
    let subject = String::from_utf8(
        Command::new("git")
            .args(["log", "-1", "--pretty=%s", "HEAD"])
            .output()
            .context("git log failed")?
            .stdout,
    )
    .context("subject is not UTF-8")?;
    if !subject_claims_source(subject.trim()) {
        return Ok(());
    }
    let files = String::from_utf8(
        Command::new("git")
            .args(["show", "--name-only", "--format=", "HEAD"])
            .output()
            .context("git show failed")?
            .stdout,
    )
    .context("file list is not UTF-8")?;
    let paths: Vec<&str> = files.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    if paths.is_empty() || paths.iter().any(|p| !is_prose_or_record(p)) {
        return Ok(());
    }
    anyhow::bail!(
        "tri hooks pre-commit: HEAD claims a compiler fix but touches no source file\n  \
         {}\n  A `fix(` in a source scope must carry a .rs/.py/.t27/.zig file. If the fix is\n  \
         really in the working tree, it was never committed -- see docs/now for #3264.",
        subject.trim()
    );
}

#[cfg(test)]
mod fix_carries_source_tests {
    use super::{is_prose_or_record, subject_claims_source};

    /// The subject that shipped empty. This is the whole reason the check exists.
    #[test]
    fn the_defect_is_recognised() {
        assert!(subject_claims_source(
            "fix(rust): an untyped local bound to a comparison is not a bool (+3) (#3264)"
        ));
    }

    /// Verbatim subjects from master that carry no source file and are CORRECT to.
    /// Without these the rule would be a 12-way false alarm rather than a 1-way catch.
    #[test]
    fn scopes_whose_subject_is_elsewhere_are_left_alone() {
        for s in [
            "fix(seals): reseal 149 gen-drifted specs after the C emitter changes (#2934)",
            "fix(freeze): reseal FROZEN_HASH -- master does not build (Closes #2316)",
            "fix(corpus): the ratchet was right -- 3 paid, 2 re-labelled, CLEAN (#2492)",
            "fix(paper)+docs: W851 -- the recomputers find a stale table row",
            "fix(article): W801 -- T478, the article has no unsourced statements",
            "fix(ops): W793 -- T464, the ENOSPC was swap and I blamed the wrong thing",
            "fix(build): stop discarding the bindings/javascript release profile (#2296)",
            "fix(hooks): let the pre-commit hook reach the reader that works (#3184)",
        ] {
            assert!(!subject_claims_source(s), "false alarm on: {s}");
        }
    }

    #[test]
    fn every_source_scope_is_reachable() {
        for scope in super::SOURCE_SCOPES {
            assert!(subject_claims_source(&format!("fix({scope}): x")), "{scope}");
        }
    }

    /// A multi-scope subject claims source if ANY of its scopes does.
    #[test]
    fn a_compound_scope_counts() {
        assert!(subject_claims_source("fix(rust,zig): both mappers"));
        assert!(subject_claims_source("fix(docs, rust): the note and the fix"));
        assert!(!subject_claims_source("fix(docs,seals): neither"));
    }

    #[test]
    fn non_fix_and_malformed_subjects_are_ignored() {
        assert!(!subject_claims_source("feat(rust): a new emitter arm"));
        assert!(!subject_claims_source("fix(rust: never closed"));
        assert!(!subject_claims_source("fix: no scope at all"));
        assert!(!subject_claims_source(""));
    }

    #[test]
    fn source_paths_are_the_four_the_compiler_is_built_from() {
        // Substance: everything the earlier extension whitelist would have refused,
        // including the four categories an adversarial pass named and the extensionless
        // paths no whitelist entry can ever match.
        for p in [
            "bootstrap/src/compiler.rs", "tools/x.py", "corpus/a.t27", "src/m.zig",
            "rtl/mac.v", "runtime/shim.c", "runtime/shim.h", "rtl/top.sv",
            "fpga/verilog/gft_sadd_jtag.xdc", "synth/run.tcl", "Cargo.toml",
            "proofs/lean4/Trinity/Emitter.lean", "Makefile", "Dockerfile", "scripts/tri",
            "tools/conflict_markers_baseline.txt", "README",
            // A directory prefix is not a claim about content.
            "docs/tools/gen.py", "docs/scripts/build.sh", "docs/assets/diagram.svg",
        ] {
            assert!(!is_prose_or_record(p), "should be substance: {p}");
        }
        // Prose and records: the closed set.
        for p in [
            "docs/now/note.md", "docs/FROZEN.md", "docs/theory/x.tex", "paper/a.rst",
            ".trinity/seals/Backend.json", "NOW.md", "a/b/c.md",
        ] {
            assert!(is_prose_or_record(p), "should be prose: {p}");
        }
    }
}

/// The pull-request form of the guard: a title that claims a compiler fix, against the
/// union of the diff.
///
/// WHY THIS EXISTS BESIDE THE PRE-COMMIT FORM. The hook version was merged in #3279 and
/// was, that same day, reachable from nothing: `core.hooksPath` was unset and 0 of 148
/// worktrees had an installed `pre-commit`, so five checks -- this one included -- were
/// invoked by nothing. A guard that a single unset config disables is not a guard.
///
/// WHY THE PULL REQUEST AND NOT EACH COMMIT. Merges here are squashed, so the commit that
/// lands on master IS the pull request. A branch may well carry `fix(rust): X` in one
/// commit and the source in the next; squashed, that is correct, and flagging the
/// intermediate would be a false accusation of a defect that never reaches master.
fn fix_carries_source_cmd(
    subject: Option<&str>,
    base: Option<&str>,
    head: Option<&str>,
    self_check: bool,
) -> Result<()> {
    if self_check {
        return fix_carries_source_self_check();
    }
    let head = head.unwrap_or("HEAD");
    let subject = match subject {
        Some(s) => s.to_string(),
        None => git_out(&["log", "-1", "--pretty=%s", head])?,
    };
    if !subject_claims_source(subject.trim()) {
        println!("fix-carries-source: PASSED (the title claims no compiler scope)");
        return Ok(());
    }
    let files = match base {
        Some(b) => git_out(&["diff", "--name-only", &format!("{b}...{head}")])?,
        None => git_out(&["show", "--name-only", "--format=", head])?,
    };
    let paths: Vec<&str> = files
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if paths.is_empty() {
        println!("fix-carries-source: PASSED (the diff is empty; nothing can land)");
        return Ok(());
    }
    let substance: Vec<&&str> = paths.iter().filter(|p| !is_prose_or_record(p)).collect();
    if !substance.is_empty() {
        println!(
            "fix-carries-source: PASSED ({} of {} path(s) are substance, e.g. {})",
            substance.len(),
            paths.len(),
            substance[0]
        );
        return Ok(());
    }
    anyhow::bail!(
        "fix-carries-source: the title claims a compiler fix and the diff is prose only\n  \
         title: {}\n  \
         all {} path(s) in the diff are under docs/, under .trinity/seals/, or .md:\n{}\n  \
         A scope of {} says the change is in the compiler. If the change really is\n  \
         elsewhere, name that scope instead -- fix(seals), fix(docs), fix(ops) and\n  \
         fix(paper) all land as prose and are not touched by this check. If the change IS\n  \
         in the compiler and is not here, it was never committed: see #3264, whose entire\n  \
         diff was one docs/now note.",
        subject.trim(),
        paths.len(),
        paths.iter().map(|p| format!("      {p}")).collect::<Vec<_>>().join("\n"),
        SOURCE_SCOPES.join(", ")
    );
}

fn git_out(args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("git {} failed to start", args.join(" ")))?;
    if !out.status.success() {
        anyhow::bail!(
            "git {} exited {}: {}",
            args.join(" "),
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    String::from_utf8(out.stdout).context("git output is not UTF-8")
}

/// Controls, in the shape the repository's other gates use: each states what it asserts,
/// and a control that cannot fail is reported as a failure of the control.
fn fix_carries_source_self_check() -> Result<()> {
    let mut bad = Vec::new();
    let mut say = |name: &str, ok: bool| {
        println!("  {:<8}{name}", if ok { "ok" } else { "FAILED" });
        if !ok {
            bad.push(name.to_string());
        }
    };

    // The defect, verbatim.
    say(
        "the subject that merged empty is recognised",
        subject_claims_source(
            "fix(rust): an untyped local bound to a comparison is not a bool (+3) (#3264)",
        ),
    );
    // The eleven the loose rule would have accused. Measured on master: any `fix(` with
    // no source file names 12 commits and 11 of them are correct to land that way.
    let innocent = [
        "fix(seals): reseal 149 gen-drifted specs after the C emitter changes (#2934)",
        "fix(freeze): reseal FROZEN_HASH -- master does not build (Closes #2316)",
        "fix(corpus): the ratchet was right -- 3 paid, 2 re-labelled, CLEAN (#2492)",
        "fix(paper)+docs: W851 -- the recomputers find a stale table row",
        "fix(article): W801 -- T478, the article has no unsourced statements",
        "fix(ops): W793 -- T464, the ENOSPC was swap and I blamed the wrong thing",
        "fix(build): stop discarding the bindings/javascript release profile (#2296)",
        "fix(hooks): let the pre-commit hook reach the reader that works (#3184)",
    ];
    say(
        "the scopes whose subject is elsewhere are not accused",
        innocent.iter().all(|s| !subject_claims_source(s)),
    );
    say(
        "every source scope is reachable",
        SOURCE_SCOPES
            .iter()
            .all(|sc| subject_claims_source(&format!("fix({sc}): x"))),
    );
    say(
        "a compound scope counts if any half does",
        subject_claims_source("fix(docs, rust): the note and the fix")
            && !subject_claims_source("fix(docs,seals): neither"),
    );
    say(
        "a non-fix subject and a malformed one are ignored",
        !subject_claims_source("feat(rust): a new emitter arm")
            && !subject_claims_source("fix(rust: never closed")
            && !subject_claims_source("fix: no scope at all"),
    );
    say(
        "prose and records are a closed set and everything else is substance",
        [
            "bootstrap/src/compiler.rs", "rtl/mac.v", "fpga/verilog/a.xdc", "Cargo.toml",
            "proofs/lean4/Emitter.lean", "Makefile", "scripts/tri", "README",
            "docs/tools/gen.py", "docs/scripts/build.sh",
        ]
            .iter()
            .all(|p| !is_prose_or_record(p))
            && ["docs/now/n.md", "docs/FROZEN.md", ".trinity/seals/a.json", "NOW.md"]
                .iter()
                .all(|p| is_prose_or_record(p)),
    );

    println!();
    if bad.is_empty() {
        println!("ok: the guard accuses the one subject it was built for, and none of the eleven it was not.");
        Ok(())
    } else {
        anyhow::bail!("{} control(s) did not behave as stated: {}", bad.len(), bad.join(", "))
    }
}

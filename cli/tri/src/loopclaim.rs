//! A mutex for the loop, built from the one atomic operation git already gives
//! us over a shared remote: creating a ref that does not exist.
//!
//! Two sessions of this loop ran concurrently and took the SAME three tasks
//! from the same list of recommendations, opening PRs for all of them. Nothing
//! in the flow says who is working on what.
//!
//! THE OBVIOUS VERSION DOES NOT LOCK. Pushing `origin/master` to a claim tag
//! succeeds for the second claimant too: git treats re-pushing the SAME value
//! to an existing tag as a no-op and exits 0, so both sessions believe they
//! hold it. Measured before this was written -- first push exit 0, second push
//! exit 0. Only a DIFFERENT value is rejected.
//!
//! So the claim is a commit that no other claimant can produce: an empty tree
//! with a message naming this host, process and instant. Then the second
//! claimant's push carries a different sha and git refuses it, and the tag's
//! message says who holds it.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::process::Command;

#[derive(Subcommand)]
pub enum LoopCmd {
    /// Take the named claim, or say who already holds it.
    ///
    /// Exit 0 -- it is yours. Exit 1 -- someone else has it, and the line
    /// printed names them. Exit 2 -- the claim could not be attempted at all,
    /// which is not the same as being refused.
    Claim {
        /// What is being claimed, e.g. `pass-118` or `skill-renumber`.
        name: String,
        /// Give it back.
        #[arg(long)]
        release: bool,
        /// Say who holds it without trying to take it.
        #[arg(long)]
        who: bool,
    },
    /// Read and validate the loop's `state.md` -- the file every firing reads
    /// FIRST, before touching anything else.
    ///
    /// `tri loop claim` guards the REMOTE collision (two sessions, one task
    /// list). This guards the LOCAL one, measured on the first overnight loop
    /// (2026-09-29): a session resumed in the main checkout while the loop's
    /// worktree sat elsewhere, and state.md's `branch:` line was the only
    /// thing that said so -- unread. A state file updated by hand after every
    /// iteration is also a state file a crashed write can leave half-empty,
    /// and that is exactly the moment a firing reads it.
    ///
    /// Exit 0 -- state parsed, tree matches it (dirty files are WARNED, not
    /// failed: mid-iteration is the normal dirty state). Exit 1 -- the state
    /// and the tree disagree (wrong branch; the resume must not proceed on
    /// this checkout). Exit 2 -- the state could not be read at all, which is
    /// not the same as disagreeing.
    State {
        /// Path to the loop's `state.md` (or its directory). Default: the
        /// newest `docs/loop/*/state.md`. Two loop dirs is exit 2 -- a guess
        /// between them is how the wrong loop gets resumed.
        path: Option<String>,
    },
}

fn git(args: &[&str]) -> Result<(i32, String)> {
    let out = Command::new("git")
        .args(args)
        .output()
        .context("git is not on PATH")?;
    Ok((
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    ))
}

fn tag_ref(name: &str) -> String {
    format!("refs/tags/loop-claim/{name}")
}

fn holder(name: &str) -> Option<String> {
    let r = tag_ref(name);
    // Ask the REMOTE, not a local tag: a stale local copy would report a claim
    // that has since been released, and a missing local copy would report a
    // free claim that is held.
    let (_, out) = git(&["ls-remote", "origin", &r]).ok()?;
    if out.trim().is_empty() {
        return None;
    }
    let sha = out.split_whitespace().next()?.to_string();
    let _ = git(&["fetch", "-q", "origin", &format!("{r}:{r}")]);
    let (code, msg) = git(&["log", "-1", "--format=%s (%cr)", &sha]).ok()?;
    Some(if code == 0 && !msg.trim().is_empty() {
        msg.trim().to_string()
    } else {
        format!(
            "a claim this checkout cannot read ({})",
            &sha[..8.min(sha.len())]
        )
    })
}

pub fn run(cmd: &LoopCmd) -> Result<()> {
    match cmd {
        LoopCmd::Claim { name, release, who } => claim(name, *release, *who),
        LoopCmd::State { path } => state(path.as_deref()),
    }
}

// --- tri loop state ---------------------------------------------------------

/// The keys a state.md must carry for a firing to resume safely. Missing any
/// of them means the file was written by something that did not finish.
const REQUIRED_STATE_KEYS: [&str; 4] = ["loop", "branch", "iteration", "current-task"];

/// Parse the fenced ``` block at the top of a state.md into key/value pairs.
///
/// The format the loop actually writes (docs/loop/auto-2026-09-29/state.md):
/// keys at column 0 ending in `:`, values on the same line, continuation
/// lines indented and belonging to the key above them -- `current-task:` is
/// routinely two lines. A parser that reads one line per key would report a
/// TRUNCATED task and the firing would resume from half an instruction.
pub fn parse_state_block(src: &str) -> Vec<(String, String)> {
    let mut in_block = false;
    let mut pairs: Vec<(String, String)> = Vec::new();
    for line in src.lines() {
        let trimmed_end = line.trim_end();
        if trimmed_end.starts_with("```") {
            if in_block {
                break; // first fence closes the block we read
            }
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if let Some(rest) = line.strip_prefix("    ") {
            // Continuation of the previous key, indented to align under the
            // value column. The ALIGNMENT padding is presentation, not
            // content -- trim it from both ends or every resumed task string
            // carries thirteen spaces of `current-task:` column alignment.
            // Only append when there IS a previous key; a continuation with
            // no key is a malformed block and dropping it silently is how
            // state lies.
            if let Some(last) = pairs.last_mut() {
                last.1.push('\n');
                last.1.push_str(rest.trim());
            }
            continue;
        }
        // A key line: `name:` or `name:   value`. The key itself is ASCII
        // lowercase words and dashes; anything else is prose outside the
        // block's key space and belongs to no key.
        let Some((key, value)) = trimmed_end.split_once(':') else {
            continue;
        };
        if key.is_empty()
            || !key
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
        {
            continue;
        }
        pairs.push((key.to_string(), value.trim().to_string()));
    }
    pairs
}

fn state(path: Option<&str>) -> Result<()> {
    use std::path::PathBuf;

    // Resolve the state file. The default must not guess between loops: two
    // docs/loop/*/state.md files is exactly the moment resuming the WRONG one
    // duplicates or clobbers a live iteration.
    let file: PathBuf = match path {
        Some(p) => {
            let pb = PathBuf::from(p);
            if pb.is_dir() {
                pb.join("state.md")
            } else {
                pb
            }
        }
        None => {
            let mut candidates: Vec<PathBuf> = Vec::new();
            if let Ok(rd) = std::fs::read_dir("docs/loop") {
                for entry in rd.flatten() {
                    let sm = entry.path().join("state.md");
                    if sm.is_file() {
                        candidates.push(sm);
                    }
                }
            }
            match candidates.len() {
                0 => {
                    eprintln!("  no docs/loop/*/state.md exists here -- nothing to resume");
                    std::process::exit(2);
                }
                1 => candidates.remove(0),
                n => {
                    eprintln!("  {n} loops have a state.md; resuming a guess between them is the bug this command exists for:");
                    for c in &candidates {
                        eprintln!("    {}", c.display());
                    }
                    eprintln!("  name one: `tri loop state <path>`");
                    std::process::exit(2);
                }
            }
        }
    };

    let src = match std::fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  could not read {} -- {e}", file.display());
            std::process::exit(2);
        }
    };
    let pairs = parse_state_block(&src);
    let get = |k: &str| pairs.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());

    let missing: Vec<&str> = REQUIRED_STATE_KEYS
        .iter()
        .copied()
        .filter(|k| get(k).is_none())
        .collect();
    if !missing.is_empty() {
        eprintln!("  {} is missing keys:", file.display());
        for m in &missing {
            eprintln!("    {m}:");
        }
        eprintln!("  a half-written state file is the moment this check matters most");
        std::process::exit(2);
    }

    println!("  state: {}", file.display());
    println!("  loop {} | iteration {} | PR {}", get("loop").unwrap_or_default(), get("iteration").unwrap_or_default(), get("pr").unwrap_or_else(|| "(none named)".into()));
    println!("  current-task: {}", get("current-task").unwrap_or_default().replace('\n', "\n                "));
    if let Some(next) = get("next-up") {
        println!("  next-up:      {}", next.replace('\n', "\n                "));
    }

    // The checks. Each one is a way tonight's loop could have resumed wrong.
    let here = std::env::current_dir().unwrap_or_default();
    let (_code, out) = git(&["rev-parse", "--abbrev-ref", "HEAD"])?;
    let current_branch = out.trim();
    let branch = current_branch.to_string();
    let want = get("branch").unwrap_or_default();
    // state.md writes the branch with a parenthetical ("branch: x (pushed;
    // PR #N)"); the check is on the branch NAME, before the first space.
    let want_name = want.split_whitespace().next().unwrap_or(&want);
    if current_branch != want_name {
        println!("  BRANCH DIVERGED: state says `{want_name}`, this checkout ({}) is on `{current_branch}`", here.display());
        println!("  resuming here would commit loop work onto a branch the state file does not name");
        std::process::exit(1);
    }
    println!("  branch ok: {current_branch}");

    let (code, dirty) = git(&["status", "--porcelain"])?;
    if code == 0 && !dirty.trim().is_empty() {
        let n = dirty.trim().lines().count();
        println!(
            "  WARN {n} dirty file(s) (mid-iteration is normal; a crashed iteration is not -- inspect before continuing):"
        );
        for l in dirty.trim().lines().take(5) {
            println!("    {l}");
        }
        if n > 5 {
            println!("    ... and {} more", n - 5);
        }
    }

    // `@{u}` maps the branch through remote.<name>.fetch; this repo's worktrees
    // deliberately carry a narrow master-only refspec, so @{u} fails even when
    // the remote-tracking ref exists. Fall back to the explicit ref before
    // claiming nothing was checkpointed -- a wrong WARN is the exact lie the
    // ci-gates skill documents.
    let upstream_ref = format!("refs/remotes/origin/{branch}");
    let (code, ahead) = git(&["rev-list", "--count", "--left-only", "HEAD...@{u}"])?;
    let (code, ahead) = if code == 0 {
        (code, ahead)
    } else {
        let (c2, a2) = git(&["rev-list", "--count", "--left-only", &format!("HEAD...{upstream_ref}")])?;
        if c2 != 0 {
            println!(
                "  WARN branch has no upstream AND no {upstream_ref}: nothing has been \
                 checkpointed remotely (if upstream IS set, the narrow master-only \
                 remote.origin.fetch refspec hides it -- add a scoped refspec)"
            );
            return Ok(());
        }
        (c2, a2)
    };
    match ahead.trim().parse::<usize>() {
        Ok(0) => println!("  pushed: up to date with upstream"),
        Ok(n) => println!("  WARN {n} unpushed commit(s) -- the last checkpoint is local only"),
        Err(_) => println!("  WARN could not read the ahead-count: {ahead}"),
    }

    Ok(())
}

fn claim(name: &str, release: bool, who: bool) -> Result<()> {
    let r = tag_ref(name);

    if who {
        match holder(name) {
            Some(h) => {
                println!("  {name}: HELD -- {h}");
                std::process::exit(1);
            }
            None => {
                println!("  {name}: free");
                return Ok(());
            }
        }
    }

    if release {
        let (code, out) = git(&["push", "--delete", "origin", &r])?;
        if code == 0 {
            println!("  {name}: released");
            return Ok(());
        }
        eprintln!("  {name}: could not release -- {}", out.trim());
        std::process::exit(2);
    }

    // A value no other claimant can produce. The empty tree is shared; the
    // message is not.
    let (c1, empty) = git(&["hash-object", "-t", "tree", "/dev/null"])?;
    if c1 != 0 {
        eprintln!("  could not build the empty tree -- nothing was attempted");
        std::process::exit(2);
    }
    let host = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown-host".into());
    let stamp = git(&["log", "-1", "--format=%H", "HEAD"])
        .map(|(_, s)| s.trim().chars().take(8).collect::<String>())
        .unwrap_or_default();
    let msg = format!(
        "loop claim `{name}` by {host} pid {} at {stamp}",
        std::process::id()
    );
    let (c2, commit) = git(&["commit-tree", empty.trim(), "-m", &msg])?;
    if c2 != 0 {
        eprintln!("  could not build the claim commit -- nothing was attempted");
        std::process::exit(2);
    }

    let refspec = format!("{}:{}", commit.trim(), r);
    let (code, out) = git(&["push", "origin", &refspec])?;
    if code == 0 {
        println!("  {name}: CLAIMED by this session");
        println!("  release it with `tri loop claim {name} --release` when the pass ends.");
        return Ok(());
    }

    match holder(name) {
        Some(h) => {
            println!("  {name}: already held -- {h}");
            println!("  Another session is on this. Pick something else.");
            std::process::exit(1);
        }
        None => {
            // Refused, but nobody holds it: that is not a lost race, it is a
            // broken push, and saying "held" would be a lie.
            eprintln!("  {name}: the push was refused and no holder exists.");
            eprintln!("  {}", out.trim());
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_claim_value_is_built_not_borrowed() {
        // The defect this file exists to avoid, and the one I measured before
        // writing it: pushing an EXISTING ref (origin/master) to the claim tag
        // succeeds for the second claimant too, because git treats re-pushing
        // the same value as a no-op and exits 0. Both sessions then believe
        // they hold the claim. Only a value unique to the claimant is refused.
        let src = include_str!("loopclaim.rs");
        let boundary = src
            .lines()
            .position(|l| l == concat!("#[cfg(te", "st)]"))
            .expect("the test module attribute is a line of its own");
        let prod: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");
        assert!(
            prod.contains("fn claim("),
            "the production slice no longer reaches `claim` -- this would pass vacuously"
        );
        assert!(
            prod.contains(concat!("commit-", "tree")),
            "the claim must be a commit this session BUILDS; pushing an existing \
             ref is a no-op for the second claimant and locks nothing"
        );
        assert!(
            !prod.contains(concat!("origin/mas", "ter\":")),
            "and it must not be built from a ref both sessions already share"
        );
    }

    #[test]
    fn a_refused_push_with_no_holder_is_could_not_run() {
        // Exit 1 means someone else has it. Exit 2 means the attempt failed.
        // Reporting a broken push as "held" would send the next session away
        // from work nobody is doing -- the same shape as a gate that reports
        // could-not-run as clean.
        let src = include_str!("loopclaim.rs");
        let boundary = src
            .lines()
            .position(|l| l == concat!("#[cfg(te", "st)]"))
            .expect("the test module attribute is a line of its own");
        let prod: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");
        // Pin the ARM, not the file. `contains("exit(2)")` over everything after
        // the first `match holder` is satisfied by any of the four other exit-2
        // sites, and a mutant that changed THIS one to exit(1) survived it.
        let needle = concat!(
            "no holder exists.\");\n",
            "            eprintln!(\"  {}\", out.trim());\n",
            "            std::process::exit(2);"
        );
        assert!(
            prod.contains(needle),
            "a refused push with no holder has to exit 2, not 1 -- reporting a broken \
             push as HELD sends the next session away from work nobody is doing"
        );
    }

    #[test]
    fn state_keys_parse_multiline_values_the_loop_actually_writes() {
        // The real format, lifted from docs/loop/auto-2026-09-29/state.md:
        // `current-task:` carries a four-space continuation line. A parser
        // that reads one line per key would hand a firing HALF an instruction.
        let sample = concat!(
            "# state.md -- live loop state\n\n",
            "```\n",
            "loop:            auto-2026-09-29\n",
            "branch:          loop/auto-2026-09-29 (pushed; PR #5084)\n",
            "iteration:       3 (in flight)\n",
            "current-task:    L2 #5079 verilog_bench_harness.t27 deliberate rewrite\n",
            "                 then tri CLI additions\n",
            "pr:              #5084\n",
            "```\n",
        );
        let pairs = parse_state_block(sample);
        let task = pairs
            .iter()
            .find(|(k, _)| k == "current-task")
            .map(|(_, v)| v.as_str())
            .expect("current-task must parse");
        assert_eq!(
            task,
            "L2 #5079 verilog_bench_harness.t27 deliberate rewrite\nthen tri CLI additions",
            "the continuation line must be APPENDED to the key, not dropped"
        );
        let branch = pairs
            .iter()
            .find(|(k, _)| k == "branch")
            .map(|(_, v)| v.as_str())
            .expect("branch must parse");
        assert_eq!(branch, "loop/auto-2026-09-29 (pushed; PR #5084)");
    }

    #[test]
    fn missing_required_keys_are_named_not_counted() {
        // Exit 2 with the MISSING KEY NAMES, not a count: "1 key missing" sends
        // the reader hunting; "branch: is missing" ends the hunt. The state
        // writer is a human under time pressure at 2am -- name the wound.
        let sample = concat!(
            "```\nloop: x\niteration: 1\ncurrent-task: y\n```\n",
        );
        let pairs = parse_state_block(sample);
        let missing: Vec<&str> = REQUIRED_STATE_KEYS
            .iter()
            .copied()
            .filter(|k| !pairs.iter().any(|(key, _)| key == k))
            .collect();
        assert_eq!(missing, vec!["branch"], "exactly the missing key, by name");
    }

    #[test]
    fn branch_divergence_exits_one_not_two() {
        // Exit 1 = state and tree disagree (actionable: go to the right
        // checkout). Exit 2 = could not run. A divergence reported as
        // could-not-run hides a checkout that would happily commit loop work
        // onto the wrong branch -- the exact incident of 2026-09-29, a
        // session resumed in the main repo while the loop's worktree sat
        // elsewhere.
        let src = include_str!("loopclaim.rs");
        let boundary = src
            .lines()
            .position(|l| l == concat!("#[cfg(te", "st)]"))
            .expect("the test module attribute is a line of its own");
        let prod: String = src.lines().take(boundary).collect::<Vec<_>>().join("\n");
        let needle = concat!(
            "would commit loop work onto a branch the state file does not name\");\n",
            "        std::process::exit(1);"
        );
        assert!(
            prod.contains(needle),
            "branch divergence must exit 1 -- it is a disagreement the reader can act on,\n\
             not a failure to run"
        );
    }
}

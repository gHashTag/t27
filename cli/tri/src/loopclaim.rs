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
    }
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
}

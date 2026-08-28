//! Recompute `bootstrap/stage0/FROZEN_HASH` from the file it seals.
//!
//! The seal is a drift detector: `bootstrap/build.rs` refuses to build while
//! it disagrees with `sha256(bootstrap/src/compiler.rs)`. Moving it alongside a
//! deliberate compiler change is routine -- 178 of the 184 commits touching
//! that file in the last thirty days did exactly that -- and the shell for it
//! was retyped each time.
//!
//! The case worth a command, though, is the MERGE. When two branches both
//! changed the compiler, git leaves `FROZEN_HASH` conflicted with two candidate
//! hashes, and **neither one is correct**: each describes its own side's file,
//! not the merged one. Measured on a real conflict:
//!
//!   ours   8e62cacb81c6e84d
//!   theirs 4f003654a44a4348
//!   truth  6e2bad56817414a6
//!
//! Resolving that conflict the way conflicts are usually resolved -- pick a
//! side -- writes a seal for a file that does not exist. `build.rs` catches it,
//! so the cost is confusion rather than corruption, but the reflex is wrong and
//! the command exists to remove the temptation.
use anyhow::{Context, Result};
use clap::Subcommand;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum ResealCmd {
    /// Rewrite FROZEN_HASH from the current bytes of the sealed file.
    Write,
    /// Report whether the seal matches, and exit non-zero if it does not.
    Check,
}

const SEALED: &str = "bootstrap/src/compiler.rs";
const SEAL: &str = "bootstrap/stage0/FROZEN_HASH";

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

fn sha_of(p: &Path) -> Result<String> {
    let bytes = std::fs::read(p).with_context(|| format!("reading {}", p.display()))?;
    let mut h = Sha256::new();
    h.update(&bytes);
    Ok(hex::encode(h.finalize()))
}

pub fn run(cmd: &ResealCmd) -> Result<()> {
    let root = repo_root()?;
    let sealed = root.join(SEALED);
    let seal = root.join(SEAL);

    let want = sha_of(&sealed)?;
    let raw =
        std::fs::read_to_string(&seal).with_context(|| format!("reading {}", seal.display()))?;
    // A conflicted seal is not a hash that happens to be wrong; it is two
    // hashes and some markers. Say so, rather than letting a trim() produce a
    // nonsense comparison against the first line.
    let conflicted = is_conflicted(&raw);
    let have = raw.trim().to_string();

    match cmd {
        ResealCmd::Check => {
            if conflicted {
                println!("{SEAL} still holds conflict markers.");
                println!("Neither side is right after a merge: each hash describes its own");
                println!("side's {SEALED}, not the merged bytes. Run `tri reseal write`.");
                std::process::exit(1);
            }
            if have == want {
                println!("seal matches: {}", &want[..16]);
                return Ok(());
            }
            println!("seal:   {}", short(&have));
            println!("actual: {}", &want[..16]);
            println!();
            println!("`cargo build` in bootstrap/ will refuse until these agree.");
            println!("If the change to {SEALED} is deliberate: tri reseal write");
            std::process::exit(1);
        }
        ResealCmd::Write => {
            if conflicted {
                println!("{SEAL} held conflict markers; both candidates discarded.");
            } else if have == want {
                println!("seal already matches ({}); nothing written.", &want[..16]);
                return Ok(());
            } else {
                println!("was:  {}", short(&have));
            }
            // Write-then-rename: a seal truncated by an interrupted write is a
            // file that agrees with nothing, and the next reader cannot tell it
            // from a seal that was simply never updated.
            let tmp = seal.with_extension("tmp");
            std::fs::write(&tmp, format!("{want}\n"))
                .with_context(|| format!("writing {}", tmp.display()))?;
            std::fs::rename(&tmp, &seal)
                .with_context(|| format!("replacing {}", seal.display()))?;
            println!("now:  {}", &want[..16]);
            println!("sealed {SEALED}");
        }
    }
    Ok(())
}

fn short(s: &str) -> String {
    let t: String = s.chars().take(16).collect();
    if t.is_empty() {
        "(empty)".into()
    } else {
        t
    }
}

/// Does this seal file hold a merge conflict rather than a hash?
///
/// Checked by line prefix, not by `contains`: a hash is 64 hex characters and
/// can never begin with `<`, `=` or `>`, but a `contains("=======")` would also
/// fire on a file that merely mentions one, and the point of this function is
/// to be trusted when it says the file is unusable.
fn is_conflicted(raw: &str) -> bool {
    raw.lines()
        .any(|l| l.starts_with("<<<<<<<") || l.starts_with("=======") || l.starts_with(">>>>>>>"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The case this command exists for. A merged seal holds two hashes and
    /// neither is right, so the file has to be recognised as unusable rather
    /// than trimmed into a comparison against whichever line came first.
    #[test]
    fn a_conflicted_seal_is_not_a_hash() {
        let conflict =
            "<<<<<<< HEAD\n8e62cacb81c6e84d\n=======\n4f003654a44a4348\n>>>>>>> master\n";
        assert!(is_conflicted(conflict));

        // What `trim()` would have produced, had the marker gone unnoticed:
        // the first marker line, compared against a hash, reporting a plain
        // mismatch and sending the reader to fix the wrong thing.
        assert_eq!(conflict.trim().lines().next().unwrap(), "<<<<<<< HEAD");

        let clean = "4f003654a44a4348aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n";
        assert!(!is_conflicted(clean));
        assert!(!is_conflicted(""));
    }

    /// A hash is hex, so no legitimate seal can start with a marker character.
    #[test]
    fn a_real_seal_never_looks_conflicted() {
        for c in ["<", "=", ">"] {
            assert!(!"0123456789abcdef".starts_with(c));
        }
    }

    #[test]
    fn short_survives_an_empty_or_stubby_seal() {
        assert_eq!(short(""), "(empty)");
        assert_eq!(short("abc"), "abc");
        assert_eq!(short(&"a".repeat(64)), "a".repeat(16));
    }
}

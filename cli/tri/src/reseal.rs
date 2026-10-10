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
//!
//! Every rule is specs/tri/reseal/seal.t27 (#8716); this file is glue around its `t27c gen-rust`.
use anyhow::{Context, Result};
use clap::Subcommand;
use sha2::{Digest, Sha256};
#[path = "../../../gen/rust/tri/reseal/seal.rs"] #[allow(dead_code, unused_parens)]
mod rs; // t27c gen-rust of specs/tri/reseal/seal.t27

#[derive(Subcommand)]
pub enum ResealCmd {
    /// Rewrite FROZEN_HASH from the current bytes of the sealed file.
    Write,
    /// Report whether the seal matches, and exit non-zero if it does not.
    Check,
}

pub fn run(cmd: &ResealCmd) -> Result<()> {
    let out = std::process::Command::new("git").args(["rev-parse", "--show-toplevel"]).output().context("running `git rev-parse --show-toplevel`")?;
    anyhow::ensure!(out.status.success(), "not inside a git repository");
    let root = std::path::PathBuf::from(String::from_utf8(out.stdout)?.trim());
    let (sealed, seal) = (root.join(rs::SEALED), root.join(rs::SEAL));
    let want = hex::encode(Sha256::digest(std::fs::read(&sealed).with_context(|| format!("reading {}", sealed.display()))?));
    let raw: &'static str = std::fs::read_to_string(&seal).with_context(|| format!("reading {}", seal.display()))?.leak();
    let have = &raw[rs::digest_from(raw)..rs::digest_to(raw)];
    let v = rs::verdict(rs::is_conflicted(raw), have == want, matches!(cmd, ResealCmd::Write));
    let shown = if rs::short_to(have) == 0 { rs::EMPTY } else { &have[..rs::short_to(have)] };
    for k in 0..rs::lines(v) {
        if k == rs::write_at(v) {
            let (tmp, rel) = (seal.with_extension("tmp"), if rs::keeps_path(raw) { &raw[rs::path_from(raw)..rs::path_to(raw)] } else { rs::SEALED });
            std::fs::write(&tmp, format!("{want} {rel}\n")).with_context(|| format!("writing {}", tmp.display()))?;
            std::fs::rename(&tmp, &seal).with_context(|| format!("replacing {}", seal.display()))?;
        }
        println!("{}", rs::line(v, k).replace(rs::HAVE, shown).replace(rs::WANT, &want[..rs::SHORT]));
    }
    if rs::exit_code(v) != 0 {
        std::process::exit(rs::exit_code(v) as i32);
    }
    Ok(())
}

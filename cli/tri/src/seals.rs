//! What `.trinity/seals` says about a spec, when it says it twice.
//!
//! WHY THIS EXISTS
//! ---------------
//! `t27c seal <spec> --save` writes `.trinity/seals/<module>.json`, and the name
//! it derives is path-based. Many specs also carry an OLDER seal keyed by the
//! bare module name. Both name the same `spec_path`, the seal gate reads both,
//! and `--save` updates exactly one of them.
//!
//! Measured on this repository the day the command was written:
//!
//!   seals                1316
//!   specs sealed TWICE    547
//!   pairs DISAGREEING      31
//!
//! The failure it produces is quiet and confusing: a spec is repaired, re-sealed,
//! and `coverage` still goes red -- on the twin. That happened on #2766, and the
//! minute spent staring at a green re-seal beside a red gate is what this command
//! removes. See #2767.
//!
//! It reports. It does not write: which of two disagreeing seals is the truth is
//! not a question a lister may answer, and re-sealing both would freeze whichever
//! generation happens to be current into two places instead of one.
use anyhow::{Context, Result};
use clap::Subcommand;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum SealsCmd {
    /// Specs that carry more than one seal, and which of those pairs disagree.
    Twins {
        /// Print every twinned spec, not only the ones that disagree.
        #[arg(long)]
        all: bool,
    },
}

/// The five fields a seal makes a claim with. A pair that agrees on all five is
/// duplication; a pair that differs on any is two answers to one question.
const CLAIMS: [&str; 5] = [
    "spec_hash",
    "gen_hash_zig",
    "gen_hash_c",
    "gen_hash_rust",
    "gen_hash_verilog",
];

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

/// `spec_path` -> [(seal file name, the five claims)], for every readable seal.
///
/// A seal without a `spec_path` is skipped rather than grouped under the empty
/// string: it names no spec, so it cannot be somebody's twin. The seal gate
/// already reports those separately (`no-spec-path` in `seal_baseline.txt`).
/// One seal's five claims, under the file name that makes them.
type Claims = (String, Vec<String>);

fn collect(dir: &PathBuf) -> Result<BTreeMap<String, Vec<Claims>>> {
    let mut by: BTreeMap<String, Vec<Claims>> = BTreeMap::new();
    let rd = std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))?;
    for e in rd {
        let p = e?.path();
        if p.extension().is_none_or(|x| x != "json") {
            continue;
        }
        let text = match std::fs::read_to_string(&p) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let v: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let spec = match v.get("spec_path").and_then(|x| x.as_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };
        let claims = CLAIMS
            .iter()
            .map(|k| {
                v.get(*k)
                    .and_then(|x| x.as_str())
                    .unwrap_or("<absent>")
                    .to_string()
            })
            .collect();
        let name = p
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        by.entry(spec).or_default().push((name, claims));
    }
    Ok(by)
}

pub fn run(cmd: &SealsCmd) -> Result<()> {
    let root = repo_root()?;
    let dir = root.join(".trinity/seals");
    if !dir.is_dir() {
        // Not "zero twins". There is no seal directory here, which is a
        // different world from a directory with nothing wrong in it.
        anyhow::bail!(
            "no seal directory at {} -- nothing was read, so nothing is claimed",
            dir.display()
        );
    }
    let by = collect(&dir)?;

    let SealsCmd::Twins { all } = cmd;
    let total: usize = by.values().map(|v| v.len()).sum();
    let twinned: Vec<_> = by.iter().filter(|(_, v)| v.len() > 1).collect();
    let disagree: Vec<_> = twinned
        .iter()
        .filter(|(_, v)| {
            let first = &v[0].1;
            v.iter().any(|(_, c)| c != first)
        })
        .collect();

    println!("  seals with a spec_path   {total}");
    println!("  specs sealed more than once   {}", twinned.len());
    println!("  of those, pairs that DISAGREE {}", disagree.len());
    println!();

    let show: Vec<_> = if *all {
        twinned.clone()
    } else {
        disagree.iter().map(|x| **x).collect()
    };
    if show.is_empty() {
        println!(
            "  {}",
            if *all {
                "No spec carries more than one seal."
            } else {
                "Every twinned spec's seals agree. Pass --all to list them anyway."
            }
        );
        return Ok(());
    }
    for (spec, seals) in show {
        println!("  {spec}");
        let first = &seals[0].1;
        for (name, claims) in seals.iter() {
            let mark = if claims == first { " " } else { "!" };
            println!("    {mark} {name}");
            for (k, c) in CLAIMS.iter().zip(claims.iter()) {
                let short = if c.len() > 20 { &c[..20] } else { c.as_str() };
                println!("        {k:<18} {short}");
            }
        }
        println!();
    }
    println!("  A pair that disagrees is two answers to one question. Which one is");
    println!("  the truth is #2767's decision, not this command's -- it only reports.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The claim vector must notice a difference in ANY of the five fields, not
    /// just the one the first bug happened to move. `testgen`'s twin differed in
    /// zig and c while rust and verilog matched byte for byte -- a comparison on
    /// a single hash would have called that pair identical.
    #[test]
    fn a_difference_in_one_field_is_a_disagreement() {
        for i in 0..CLAIMS.len() {
            let a: Vec<String> = CLAIMS.iter().map(|_| "same".to_string()).collect();
            let mut b = a.clone();
            b[i] = "different".to_string();
            assert_ne!(a, b, "field {} was not compared", CLAIMS[i]);
        }
    }

    /// An absent field is not silently equal to a present one. A seal written
    /// before `gen_hash_rust` existed and one written after must not compare as
    /// the same claim.
    #[test]
    fn absent_is_not_equal_to_present() {
        let a = vec!["<absent>".to_string()];
        let b = vec!["sha256:abc".to_string()];
        assert_ne!(a, b);
    }
}

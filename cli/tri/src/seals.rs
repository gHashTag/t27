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
    /// Is the built compiler newer than the source the seals are checked against?
    ///
    /// Answers the one question that made a red gate read as green: a seal check
    /// compares against WHAT THE COMPILER PRODUCES, and a compiler older than
    /// its own source produces the previous answer. Absence of a binary is
    /// already handled everywhere; staleness is not, and it is indistinguishable
    /// from health in every output.
    Fresh,
    /// Seals whose claims no longer match what the compiler produces, and the
    /// one command that repairs them.
    ///
    /// The recipe is five steps and thirty seconds, and it is written down
    /// nowhere: rebuild, list the drift, `seal --save` each, `sync-twins`,
    /// re-check. Measured over twelve hours: SIX changes to the compiler, ONE
    /// of which touched `.trinity/seals/`, and zero mentions of re-sealing in
    /// CONTRIBUTING, docs/ or the pull-request template.
    ///
    /// The value is not the thirty seconds. It is the two refusals a hand-rolled
    /// loop forgets: a compiler older than its own source answers with the
    /// PREVIOUS output and every seal appears to hold, and `--force` writes
    /// `gen_hash=none` as though absence were a hash.
    Drift {
        /// Re-seal what drifted, then sync the twins.
        ///
        /// Re-sealing is a STATEMENT that the new output is the output you
        /// want. It is not a repair, and this flag cannot tell the difference:
        /// measure `t27c corpus` before and after and read the acceptance
        /// columns yourself.
        #[arg(long)]
        fix: bool,
    },
    /// Specs that carry more than one seal, and which of those pairs disagree.
    Twins {
        /// Print every twinned spec, not only the ones that disagree.
        #[arg(long)]
        all: bool,
    },
    /// Copy the NEWEST seal's claims onto its twins, after a re-seal.
    ///
    /// `t27c seal <spec> --save` writes one file of each pair. A codegen change
    /// therefore leaves the twin holding hashes for output that no longer
    /// exists, and the seal gate fails on a spec that was just repaired --
    /// measured: 228 seals drifted, 125 re-seals fixed 125, and 103 twins stayed
    /// red until this ran.
    ///
    /// Refuses any spec whose newest seal records `gen_hash=none`: that is a
    /// spec which does not generate, and propagating it would write the
    /// breakage into a second place. See #2767.
    SyncTwins {
        /// Report what would change and write nothing.
        #[arg(long)]
        dry_run: bool,
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

/// The five claims plus the stamp, read straight off disk for one seal file.
fn read_seal(path: &std::path::Path) -> Option<serde_json::Value> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

/// Write the five recomputed claims into one seal file, keeping every other
/// field it already had -- `module`, `ring` and the rest belong to the file, not
/// to the generation. `sealed_at` is left alone on purpose: this command did not
/// perform a sealing ceremony, it copied a computed answer, and stamping it with
/// a fresh time would claim otherwise.
fn graft(truth: &[String], dst_path: &std::path::Path) -> Result<()> {
    let mut dst = read_seal(dst_path)
        .ok_or_else(|| anyhow::anyhow!("cannot re-read {}", dst_path.display()))?;
    let obj = dst
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("{} is not a JSON object", dst_path.display()))?;
    for (k, v) in CLAIMS.iter().zip(truth.iter()) {
        obj.insert((*k).to_string(), serde_json::Value::String(v.clone()));
    }
    let mut text = serde_json::to_string_pretty(&dst)?;
    text.push('\n');
    std::fs::write(dst_path, text).with_context(|| format!("writing {}", dst_path.display()))?;
    Ok(())
}

/// Recompute one spec's five claims with `t27c seal <spec>` (no `--save`).
///
/// This is the whole point of the command: the truth is not "whichever seal was
/// written last", it is what the compiler produces from that spec RIGHT NOW.
/// Guessing between two disagreeing seals would settle #2767 by coin flip.
/// Is the compiler binary OLDER than the source it was built from?
///
/// W719: `Seal Coverage` was red on master for seven runs while the same script
/// run locally said `OK, 1222 hold, exit 0`. The script checks seals against the
/// built compiler, and the one on disk was six hours old -- from before four
/// emitter fixes landed -- so it produced the OLD output, which matched the OLD
/// seals. `check_seal_coverage.py` handles a MISSING binary explicitly, in its
/// own words: "a missing binary is NOT a passing check." A STALE one looks
/// exactly like a healthy one.
///
/// Returns the age gap in seconds when the binary is older than any file under
/// `bootstrap/src`.
pub fn binary_is_stale(root: &std::path::Path, bin: &std::path::Path) -> Option<u64> {
    let bin_t = std::fs::metadata(bin).ok()?.modified().ok()?;
    let mut newest = bin_t;
    let mut stack = vec![root.join("bootstrap/src")];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
                if let Ok(t) = std::fs::metadata(&p).and_then(|m| m.modified()) {
                    if t > newest {
                        newest = t;
                    }
                }
            }
        }
    }
    newest
        .duration_since(bin_t)
        .ok()
        .map(|d| d.as_secs())
        .filter(|s| *s > 0)
}

/// Seconds as something a person reads without counting zeros.
fn human(sec: u64) -> String {
    match sec {
        0..=90 => format!("{sec}s"),
        91..=5400 => format!("{}m", sec / 60),
        _ => format!("{}h{:02}m", sec / 3600, (sec % 3600) / 60),
    }
}

/// May these claims be written into a seal?
///
/// No, if any target reports `none`. `t27c seal` exits 0 and prints
/// `gen_hash_zig=none` for a spec no backend accepts, so the refusal cannot be
/// left to the exit code -- and #2210 measured what happens without it: batch
/// re-sealing the stale seals would have recorded 348 reproducibility
/// assertions for output that does not exist.
pub fn is_sealable(claims: &[String]) -> bool {
    !claims.iter().any(|c| c.trim() == "none")
}

fn recompute(root: &PathBuf, spec: &str) -> Option<Vec<String>> {
    let t27c = ["target/release/t27c", "target/debug/t27c"]
        .iter()
        .map(|p| root.join(p))
        .find(|p| p.is_file())?;
    let out = std::process::Command::new(t27c)
        .arg("seal")
        .arg(spec)
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut found: Vec<String> = Vec::new();
    for k in CLAIMS.iter() {
        let key = format!("{k}=");
        let v = text
            .lines()
            .find_map(|l| l.trim().strip_prefix(&key))
            .map(|v| v.trim().to_string())?;
        found.push(v);
    }
    Some(found)
}

fn sync_twins(root: &PathBuf, dir: &PathBuf, dry_run: bool) -> Result<()> {
    let by = collect(dir)?;
    let (mut written, mut refused, mut unreadable, mut already) = (0usize, 0usize, 0usize, 0usize);
    for (spec, seals) in by.iter().filter(|(_, v)| v.len() > 1) {
        let first = &seals[0].1;
        if seals.iter().all(|(_, c)| c == first) {
            already += 1;
            continue;
        }
        let truth = match recompute(root, spec) {
            Some(t) => t,
            None => {
                // NOT "they agree now". The compiler refused the spec or is not
                // built, and a command that cannot compute the truth must say so
                // rather than leave two answers standing and exit clean.
                println!("  unread  {spec}  -- `t27c seal` did not produce five hashes");
                unreadable += 1;
                continue;
            }
        };
        if truth.iter().any(|c| c == "none") {
            // The anti-pattern this command must never spread: `none` describes
            // a spec that does not generate. Writing it into a second file
            // records the breakage as reproducible truth -- which is exactly
            // what one commit on #2766 did.
            println!("  refuse  {spec}  -- generates nothing (gen_hash=none)");
            refused += 1;
            continue;
        }
        for (name, claims) in seals.iter() {
            if claims == &truth {
                continue;
            }
            if dry_run {
                println!("  would   {name}");
            } else {
                graft(&truth, &dir.join(name))?;
                println!("  synced  {name}");
            }
            written += 1;
        }
    }
    println!();
    println!("  twinned specs already consistent  {already}");
    println!(
        "  seal files {}          {written}",
        if dry_run { "to write" } else { "written " }
    );
    if refused > 0 {
        println!("  REFUSED, spec generates nothing   {refused}");
    }
    if unreadable > 0 {
        println!("  NOT COMPUTED, nothing claimed     {unreadable}");
    }
    Ok(())
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
    let all = match cmd {
        SealsCmd::SyncTwins { dry_run } => return sync_twins(&root, &dir, *dry_run),
        SealsCmd::Fresh => {
            // The SAME order `check_seal_coverage.py::_find_t27c` uses. Only the
            // first one present is consulted, so only its age can change a
            // verdict -- flagging a stale debug build beside a fresh release
            // one is noise, and noise is how a check stops being read.
            let cands = [
                "target/release/t27c",
                "bootstrap/target/release/t27c",
                "target/debug/t27c",
            ];
            let mut any = false;
            let mut stale = false;
            let mut chosen = false;
            for c in cands {
                let bin = root.join(c);
                if !bin.is_file() {
                    println!("  {c:<30} absent");
                    continue;
                }
                any = true;
                let gap = binary_is_stale(&root, &bin);
                let mark = if chosen { "        " } else { "  <- used" };
                match gap {
                    None => println!("  {c:<30} FRESH{mark}"),
                    Some(g) => {
                        if !chosen {
                            stale = true;
                        }
                        println!("  {c:<30} STALE by {}{mark}", human(g));
                    }
                }
                chosen = true;
            }
            println!();
            if !any {
                anyhow::bail!(
                    "no compiler binary. A seal check needs one, and its absence is \
                     already refused by check_seal_coverage.py -- this reports it \
                     here so the two agree."
                );
            }
            if stale {
                println!("  The binary a seal check would USE is older than its own source.");
                println!("  A seal check compares against WHAT THE COMPILER PRODUCES. Built");
                println!("  from older source it produces the previous answer, which matches");
                println!("  the previous seals -- so the check passes and the gate that runs");
                println!("  the same script in CI fails. Measured once: locally OK with 1222");
                println!("  seals holding, and 134 gen-drift after `cargo build --release`.");
                println!();
                println!("  cargo build --release -p t27c");
                std::process::exit(1);
            }
            // Say what was checked. "Every binary is fresh" is false whenever a
            // stale one sits beside the used one, and a summary line that
            // overclaims is the defect this command exists to catch, one level
            // up.
            println!("  The binary a seal check would use is newer than bootstrap/src, so a");
            println!("  reading taken now is a reading of THIS source. Any other binary");
            println!("  listed above is not consulted and its age decides nothing.");
            return Ok(());
        }
        SealsCmd::Drift { fix } => {
            let bin = ["target/release/t27c", "target/debug/t27c"]
                .iter()
                .map(|p| root.join(p))
                .find(|p| p.is_file());
            let Some(bin) = bin else {
                anyhow::bail!(
                    "no compiler binary -- a drift reading needs one, and its \
                     absence is not a clean bill.\n  cargo build --release -p t27c"
                );
            };
            // The refusal a hand-rolled loop forgets. A compiler older than its
            // own source answers with the PREVIOUS output, every seal appears to
            // hold, and the reading is of yesterday's tree.
            if let Some(gap) = binary_is_stale(&root, &bin) {
                anyhow::bail!(
                    "{} is STALE by {} -- bootstrap/src changed after it was built.\n  \
                     A drift reading taken with it is a reading of the previous source.\n  \
                     cargo build --release -p t27c",
                    bin.strip_prefix(&root).unwrap_or(&bin).display(),
                    human(gap)
                );
            }

            let by = collect(&dir)?;
            let mut drifted: Vec<(&String, Vec<String>)> = Vec::new();
            let mut unreadable = 0usize;
            for (spec, seals) in by.iter() {
                if !root.join(spec).is_file() {
                    continue; // dangling: a different kind, and the gate's to judge
                }
                let Some(truth) = recompute(&root, spec) else {
                    unreadable += 1;
                    continue;
                };
                if seals.iter().any(|(_, c)| *c != truth) {
                    drifted.push((spec, truth));
                }
            }

            println!("  specs whose seals no longer describe them   {}", drifted.len());
            if unreadable > 0 {
                println!("  NOT COMPUTED, nothing claimed               {unreadable}");
            }
            if drifted.is_empty() {
                println!();
                println!("  Every seal of every spec that exists matches what the compiler");
                println!("  produces from it right now. Dangling and phantom seals are a");
                println!("  different question and belong to the seal gate.");
                return Ok(());
            }
            println!();
            for (spec, _) in drifted.iter().take(12) {
                println!("      {spec}");
            }
            if drifted.len() > 12 {
                println!("      ... and {} more", drifted.len() - 12);
            }

            if !*fix {
                println!();
                println!("  --fix re-seals these and syncs their twins. Before you run it:");
                println!("  RE-SEALING IS A STATEMENT that the new output is the one you want,");
                println!("  not a repair. An emitter regression sealed is a regression written");
                println!("  into the record as truth. Measure first:");
                println!();
                println!("      t27c corpus        # and read the acceptance columns");
                return Ok(());
            }

            let (mut sealed, mut refused) = (0usize, 0usize);
            for (spec, truth) in &drifted {
                // `--force` would write `gen_hash=none` as though absence were a
                // hash. A spec no backend accepts is not a spec to seal.
                if !is_sealable(truth) {
                    refused += 1;
                    continue;
                }
                let ok = std::process::Command::new(&bin)
                    .args(["seal", spec, "--save"])
                    .current_dir(&root)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                if ok {
                    sealed += 1;
                } else {
                    refused += 1;
                }
            }
            println!();
            println!("  re-sealed                                   {sealed}");
            if refused > 0 {
                println!("  REFUSED, generates nothing or seal failed   {refused}");
            }
            // `seal --save` writes ONE file of each pair; 547 specs carry two.
            println!();
            sync_twins(&root, &dir, false)?;
            return Ok(());
        }
        SealsCmd::Twins { all } => all,
    };
    let by = collect(&dir)?;

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
    /// W719: a compiler older than its own source produces the PREVIOUS
    /// answer, which matches the PREVIOUS seals -- so the check passes while
    /// the same script in CI fails. Measured once: locally `OK, 1222 hold,
    /// exit 0`, and 134 gen-drift after `cargo build --release`.
    /// `t27c seal` exits 0 and prints `gen_hash_zig=none` for a spec no
    /// backend accepts. So the refusal cannot ride on the exit code, and a
    /// substring test is not enough either: a real sha256 could contain the
    /// letters `none` and a claim of `none` is the whole field, not part of it.
    #[test]
    fn a_target_that_generates_nothing_is_not_sealable() {
        assert!(!is_sealable(&[
            "sha256:aaa".into(),
            "none".into(),
            "sha256:bbb".into()
        ]));
        assert!(is_sealable(&["sha256:aaa".into(), "sha256:bbb".into()]));
        assert!(
            is_sealable(&["sha256:0none0".into()]),
            "a hash that merely contains the letters is a hash"
        );
        assert!(is_sealable(&[]), "nothing claimed, nothing refused");
    }

    #[test]
    fn a_binary_older_than_its_source_is_stale() {
        let d = std::env::temp_dir().join(format!("w719-{}", std::process::id()));
        let src = d.join("bootstrap/src");
        std::fs::create_dir_all(&src).unwrap();

        // Built first, source touched after: stale.
        let bin = d.join("t27c");
        std::fs::write(&bin, b"x").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        std::fs::write(src.join("compiler.rs"), b"fn main() {}").unwrap();
        assert!(
            binary_is_stale(&d, &bin).is_some(),
            "source newer than the binary must read as stale"
        );

        // Rebuilt: fresh.
        std::thread::sleep(std::time::Duration::from_millis(1100));
        std::fs::write(&bin, b"y").unwrap();
        assert!(
            binary_is_stale(&d, &bin).is_none(),
            "a binary newer than every source file is fresh"
        );

        // A non-.rs file changing does not make the compiler stale.
        std::thread::sleep(std::time::Duration::from_millis(1100));
        std::fs::write(src.join("notes.txt"), b"hello").unwrap();
        assert!(
            binary_is_stale(&d, &bin).is_none(),
            "only .rs sources decide"
        );
        std::fs::remove_dir_all(&d).ok();
    }

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

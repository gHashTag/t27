//! What the parser reads and throws away, ranked, against what it is pinned at.
//!
//! WHY THIS EXISTS
//! ---------------
//! `parse-no-discard` reports 87 corpus failures and every one of them is
//! amnestied by name in `docs/reports/suite_expectations.json`. The suite is
//! green because every failure it can report is pre-approved -- which is the
//! design working, not a defect, but it means the question "where is the next
//! rung" has no answer in any suite output.
//!
//! Ranking is the answer. Two parser fixes in one day took 1 292 tokens out of
//! the corpus by working the largest entries first, and the largest entry is not
//! visible from a count of 87.
//!
//! This reads `t27c parse-complete` -- the same command the suite phase calls
//! into -- rather than re-implementing the accounting. A second implementation
//! of a measurement is a second number to disagree with the first, and this
//! repository has paid for that more than once.
use anyhow::{Context, Result};
use clap::Subcommand;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum DiscardCmd {
    /// Rank the discarding specs, largest first, against the pinned volume.
    Top {
        /// How many to print. 0 prints all of them.
        #[arg(long, default_value_t = 15)]
        n: usize,
    },
    /// Group the discard by what the parser stopped on.
    ///
    /// A ranked list says where the tokens are; it does not say whether the top
    /// six are six problems or one. They were one: 38 specs and 20 991 of the
    /// 30 451 tokens are quantified invariants (`forall x : T ... ==>`), a
    /// construct the grammar does not contain (#2774). That is a language
    /// decision, not a parser rung, and the ranking alone could not tell.
    Classify,
}

/// The buckets, in the order they are TESTED -- first match wins, so the more
/// specific pattern must come first. `forall` before `var`, because a quantified
/// invariant's body often declares one too.
///
/// This is a coarse keyword match over `parse-complete --show` traces, and it
/// says so: `other` is where anything mis-binned lands, and a large `other` is
/// the signal that these buckets have stopped describing the corpus.
const CLASSES: [(&str, &[&str]); 3] = [
    ("forall/==> (quantified)", &["forall", "==>", "== >"]),
    ("var/const statement", &["dropped: var ", "dropped: const "]),
    ("assert", &["assert"]),
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

/// `spec -> tokens`, read out of `t27c parse-complete`.
fn observed(root: &std::path::Path) -> Result<BTreeMap<String, usize>> {
    let t27c = ["target/release/t27c", "target/debug/t27c"]
        .iter()
        .map(|p| root.join(p))
        .find(|p| p.is_file())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "t27c is not built. `cargo build --release -p t27c` first --\n  \
                 reporting nothing rather than an empty ranking this run did not earn"
            )
        })?;
    let out = std::process::Command::new(t27c)
        .arg("parse-complete")
        .current_dir(root)
        .output()
        .context("running `t27c parse-complete`")?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut map = BTreeMap::new();
    for line in text.lines() {
        // `specs/base/ternary_add.t27: DISCARDED 208 top-level token(s)`
        let Some((path, rest)) = line.split_once(": DISCARDED ") else {
            continue;
        };
        let Some(n) = rest.split_whitespace().next().and_then(|w| w.parse().ok()) else {
            continue;
        };
        map.insert(path.trim().to_string(), n);
    }
    if map.is_empty() && !text.contains("parse but DISCARD") {
        anyhow::bail!(
            "`t27c parse-complete` produced no recognisable output.\n  \
             Nothing was read, so nothing is claimed -- this is not \"zero specs discard\"."
        );
    }
    Ok(map)
}

/// `spec -> pinned tokens`, from the ledger. Absent means the entry carries no
/// bound, which the ratchet fails on; here it prints as `--`.
fn pinned(root: &std::path::Path) -> Result<BTreeMap<String, Option<usize>>> {
    let p = root.join("docs/reports/suite_expectations.json");
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?,
    )
    .with_context(|| format!("parsing {}", p.display()))?;
    let mut map = BTreeMap::new();
    for e in v
        .get("entries")
        .and_then(|x| x.as_array())
        .unwrap_or(&vec![])
    {
        if e.get("phase").and_then(|x| x.as_str()) != Some("parse-no-discard") {
            continue;
        }
        let Some(path) = e.get("path").and_then(|x| x.as_str()) else {
            continue;
        };
        map.insert(
            path.to_string(),
            e.get("discard_tokens")
                .and_then(|x| x.as_u64())
                .map(|n| n as usize),
        );
    }
    Ok(map)
}

/// The drop trace for one spec, as `t27c parse-complete --show` prints it.
fn drop_trace(root: &std::path::Path, spec: &str) -> Option<String> {
    let t27c = ["target/release/t27c", "target/debug/t27c"]
        .iter()
        .map(|p| root.join(p))
        .find(|p| p.is_file())?;
    let out = std::process::Command::new(t27c)
        .args(["parse-complete", "--show", spec])
        .current_dir(root)
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

fn classify(root: &std::path::Path, obs: &BTreeMap<String, usize>) -> Result<()> {
    let mut specs: BTreeMap<&str, usize> = BTreeMap::new();
    let mut toks: BTreeMap<&str, usize> = BTreeMap::new();
    let mut unread = 0usize;
    for (spec, n) in obs {
        let Some(trace) = drop_trace(root, spec) else {
            // Not "other". No trace was read, so no cause is claimed.
            unread += 1;
            continue;
        };
        let dropped: String = trace
            .lines()
            .filter(|l| l.trim_start().starts_with("dropped:"))
            .collect::<Vec<_>>()
            .join("\n");
        let name = CLASSES
            .iter()
            .find(|(_, pats)| pats.iter().any(|p| dropped.contains(p)))
            .map(|(n, _)| *n)
            .unwrap_or("other");
        *specs.entry(name).or_default() += 1;
        *toks.entry(name).or_default() += n;
    }
    let mut rows: Vec<_> = toks.iter().collect();
    rows.sort_by(|a, b| b.1.cmp(a.1));
    println!("  {:<26} {:>6} {:>9}", "class", "specs", "tokens");
    for (name, t) in rows {
        println!("  {:<26} {:>6} {:>9}", name, specs[*name], t);
    }
    println!(
        "  {:<26} {:>6} {:>9}",
        "TOTAL",
        specs.values().sum::<usize>(),
        toks.values().sum::<usize>()
    );
    if unread > 0 {
        println!();
        println!("  {unread} spec(s) yielded no trace -- NOT counted as `other`.");
    }
    println!();
    println!("  Coarse keyword match over `parse-complete --show`. A large `other`");
    println!("  means these buckets have stopped describing the corpus, not that");
    println!("  the corpus has stopped having causes.");
    Ok(())
}

pub fn run(cmd: &DiscardCmd) -> Result<()> {
    let root = repo_root()?;
    let obs = observed(&root)?;
    if matches!(cmd, DiscardCmd::Classify) {
        return classify(&root, &obs);
    }
    let pin = pinned(&root)?;

    let DiscardCmd::Top { n } = cmd else {
        unreachable!("Classify returned above")
    };
    let mut rows: Vec<(&String, &usize)> = obs.iter().collect();
    rows.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));

    let total: usize = obs.values().sum();
    println!("  {} spec(s) discard {} token(s)", obs.len(), total);
    println!();
    println!("  {:>7}  {:>7}  spec", "tokens", "pinned");
    let shown = if *n == 0 {
        rows.len()
    } else {
        (*n).min(rows.len())
    };
    for (path, tokens) in rows.iter().take(shown) {
        let p = match pin.get(*path) {
            Some(Some(v)) => v.to_string(),
            Some(None) => "--".to_string(),
            // Observed but not in the ledger at all: an unexpected failure, and
            // the ratchet says so far more loudly than this table should.
            None => "NOT IN LEDGER".to_string(),
        };
        println!("  {:>7}  {:>7}  {}", tokens, p, path);
    }
    if shown < rows.len() {
        println!(
            "  ... and {} more not shown (--n 0 for all)",
            rows.len() - shown
        );
    }
    println!();
    println!("  The pinned column is the ledger's bound, not a target. It moves");
    println!("  only when `t27c suite --bless-expectations` re-measures.");
    Ok(())
}

#[cfg(test)]
mod tests {
    /// The line shape this parses is `t27c parse-complete`'s, and it is the only
    /// contract between the two. Pin it here so a change to that output is a
    /// test failure rather than a silently empty ranking.
    #[test]
    fn the_parse_complete_line_shape_is_read_correctly() {
        let line = "specs/base/ternary_add.t27: DISCARDED 208 top-level token(s)";
        let (path, rest) = line.split_once(": DISCARDED ").expect("shape changed");
        assert_eq!(path, "specs/base/ternary_add.t27");
        let n: usize = rest.split_whitespace().next().unwrap().parse().unwrap();
        assert_eq!(n, 208);
    }

    /// First match wins, so a quantified invariant whose body also declares a
    /// `var` must land in the quantified bucket. Reordering CLASSES silently
    /// re-attributes thousands of tokens, which is why the order is tested.
    #[test]
    fn the_more_specific_class_is_tested_first() {
        let trace = "dropped: forall x : T\ndropped: var y = 1 ;";
        let name = super::CLASSES
            .iter()
            .find(|(_, pats)| pats.iter().any(|p| trace.contains(p)))
            .map(|(n, _)| *n)
            .unwrap_or("other");
        assert_eq!(name, "forall/==> (quantified)");
    }

    /// `==>` reaches the trace as `== >` because the lexer splits it. A matcher
    /// that only looked for `==>` would report zero of the 38 specs.
    #[test]
    fn the_split_implication_arrow_is_matched() {
        let trace = "dropped: input . activations . len == 4 == >";
        let hit = super::CLASSES[0].1.iter().any(|p| trace.contains(p));
        assert!(
            hit,
            "the lexer splits `==>`; match what the trace actually says"
        );
    }

    /// A summary line must not be mistaken for a spec row.
    #[test]
    fn the_summary_lines_are_not_rows() {
        for line in [
            "  parse but DISCARD        87 (32485 token(s))",
            "  specs scanned            650",
        ] {
            assert!(line.split_once(": DISCARDED ").is_none(), "{line}");
        }
    }
}

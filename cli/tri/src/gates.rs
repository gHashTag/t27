//! `tri gates` — find workflows that have never once succeeded.
//!
//! A gate that has never been green carries no information: it is red before
//! your change and red after it, so nobody reads it — and after a while nobody
//! reads the others either. Eighteen such workflows were found across three of
//! these repositories, between them consuming 8182 runs and producing zero
//! green results.
//!
//! That is not an aesthetic complaint. It is the measured cause of nine
//! defects living undetected in a request path that had executed once in its
//! lifetime: when red is the normal colour, a real red says nothing.
//!
//! This was a hand-run loop of `gh api` calls three times before it became a
//! command. It reports, it does not disable anything — deciding between fix,
//! dispatch-only and delete belongs to whoever owns the workflow.

use anyhow::{Context, Result};
use clap::Subcommand;
use std::process::Command;

#[derive(Subcommand, Debug)]
pub enum GatesCmd {
    /// Run every gate script and its negative control; name the ones with none.
    Sweep {
        /// Skip the gates themselves and only report which have no control.
        #[arg(long)]
        controls_only: bool,
    },
    /// Break each gate's failure path and demand its control notices.
    ///
    /// `sweep` reports whether a control EXISTS. That is a label, not a
    /// property: a control can be present, pass, and be incapable of failing.
    /// Measured on check_catalog_integrity.py -- with main()'s `return 1`
    /// rewritten to `return 0` the gate printed OK on a broken catalog, and
    /// its control still reported every branch red.
    ///
    /// One mutant per site: each `return 1..4` outside the control's own
    /// functions is flipped to `return 0` in turn, the control is run, and the
    /// mutant is killed if the control goes non-zero. The file is restored
    /// after every run. Refuses to start on a dirty tools/ tree, so an
    /// interrupted run is always recoverable with `git checkout tools/`.
    Mutate {
        /// Only this gate, by file name (e.g. check_vector_data.py).
        #[arg(long)]
        only: Option<String>,
    },
    /// List active workflows whose lifetime success count is zero.
    Dead {
        /// owner/repo, repeatable. Defaults to the three this fleet uses.
        #[arg(long = "repo")]
        repos: Vec<String>,
        /// Ignore workflows with fewer lifetime runs than this, so a new or
        /// rarely-triggered workflow is not reported as dead.
        #[arg(long, default_value_t = 50)]
        min_runs: u64,
    },
}

/// Gate scripts whose control lives in a SEPARATE file, and the file that
/// holds it. Counting these as control-less would be wrong; counting the
/// control files themselves as gates would be wrong twice.
const EXTERNAL_CONTROL: &[(&str, &str)] = &[("wp18_conformance_gate.py", "wp18_selftest_gate.py")];

/// Files under tools/ that ARE controls rather than gates.
const IS_A_CONTROL: &[&str] = &[
    "wp18_selftest_gate.py",
    "wp18_gate_selfconsistent_selftest.py",
];

fn sweep(controls_only: bool) -> Result<()> {
    let root = repo_root()?;
    let tools = root.join("tools");
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&tools)
        .with_context(|| format!("read {}", tools.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let n = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            n.ends_with(".py") && (n.starts_with("check_") || n.contains("gate"))
        })
        .collect();
    files.sort();

    let mut uncontrolled: Vec<String> = Vec::new();
    let mut rows: Vec<(String, String, String)> = Vec::new();

    for f in &files {
        let name = f
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if IS_A_CONTROL.contains(&name.as_str()) {
            continue;
        }
        let src = std::fs::read_to_string(f).unwrap_or_default();
        // The flag the script itself dispatches on -- read from its source, so a
        // control that is only mentioned in a comment is not counted as one.
        let flag = ["--self-check-drop", "--self-check", "--selftest"]
            .iter()
            .find(|fl| src.contains(&format!("\"{}\"", fl)))
            .map(|s| s.to_string());
        let external = EXTERNAL_CONTROL
            .iter()
            .find(|(g, _)| *g == name)
            .map(|(_, c)| c.to_string());

        let gate = if controls_only {
            "-".to_string()
        } else {
            code(&root, &name, &[])
        };
        let ctrl = match (&flag, &external) {
            (Some(fl), _) if !controls_only => code(&root, &name, &[fl]),
            (Some(_), _) => "-".to_string(),
            (None, Some(c)) if !controls_only => code(&root, c, &[]),
            (None, Some(_)) => "-".to_string(),
            (None, None) => {
                uncontrolled.push(name.clone());
                "NONE".to_string()
            }
        };
        rows.push((name, gate, ctrl));
    }

    println!("{:<38} {:>6}  {:>6}", "gate", "run", "control");
    for (n, g, c) in &rows {
        println!("{:<38} {:>6}  {:>6}", n, g, c);
    }
    println!();
    println!(
        "{} gate(s); {} with no negative control at all:",
        rows.len(),
        uncontrolled.len()
    );
    for n in &uncontrolled {
        println!("  {}", n);
    }
    println!();
    println!("A gate is proven working only by a RED on a deliberately broken input.");
    println!("`args` means the script needs arguments this sweep does not supply --");
    println!("it was not run, which is not the same as passing or failing.");
    println!("`run` is the gate on the tree as it stands; `control` is its own");
    println!("negative check. NONE means nothing in the repository demonstrates that");
    println!("this gate can fail -- which is the same evidence as a gate that cannot.");
    Ok(())
}

/// Top-level `def`s that belong to the CONTROL, not to the gate. Mutating a
/// `return` inside one of these breaks the instrument instead of the thing
/// being measured -- done by accident once, and it made two sound controls
/// look like they passed vacuously. Reading the printed output rather than the
/// exit code is what separated the two.
fn is_control_fn(name: &str) -> bool {
    name.contains("self_check") || name.contains("selftest") || name.contains("self_test")
}

/// Byte offsets of every `return 1..4` that belongs to the gate's own logic.
/// Line-based on purpose: these files are flat, and a real Python parse would
/// buy nothing a `def` at column zero does not already give.
fn mutable_sites(src: &str) -> Vec<(usize, usize, String)> {
    let mut sites = Vec::new();
    let mut in_control = false;
    let mut off = 0usize;
    for line in src.split_inclusive('\n') {
        if let Some(rest) = line.strip_prefix("def ") {
            let fname: String = rest.chars().take_while(|c| *c != '(').collect();
            in_control = is_control_fn(&fname);
        }
        if !in_control {
            let t = line.trim_start();
            if let Some(n) = t.strip_prefix("return ") {
                let d = n.trim_end();
                if matches!(d, "1" | "2" | "3" | "4") {
                    let col = line.len() - t.len();
                    sites.push((off + col, t.len() - 1, format!("return {}", d)));
                }
            }
        }
        off += line.len();
    }
    sites
}

fn line_of(src: &str, byte: usize) -> usize {
    src[..byte].matches('\n').count() + 1
}

fn mutate(only: Option<&str>) -> Result<()> {
    let root = repo_root()?;
    let dirty = Command::new("git")
        .args(["status", "--porcelain", "--", "tools/"])
        .current_dir(&root)
        .output()
        .context("git status failed")?;
    if !String::from_utf8_lossy(&dirty.stdout).trim().is_empty() {
        anyhow::bail!(
            "tools/ has uncommitted changes. This command rewrites those files in \
             place and restores them; starting from a dirty tree means an \
             interrupted run cannot be told from your own edits. Commit or stash first."
        );
    }

    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(root.join("tools"))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let n = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            n.ends_with(".py") && (n.starts_with("check_") || n.contains("gate"))
        })
        .collect();
    files.sort();

    println!("{:<38} {:>9}  {}", "gate", "mutants", "verdict");
    let mut total_survived: Vec<String> = Vec::new();

    for f in &files {
        let name = f
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if IS_A_CONTROL.contains(&name.as_str()) {
            continue;
        }
        if let Some(o) = only {
            if o != name {
                continue;
            }
        }
        let pristine = std::fs::read_to_string(f)?;
        let flag = ["--self-check-drop", "--self-check", "--selftest"]
            .iter()
            .find(|fl| pristine.contains(&format!("\"{}\"", fl)))
            .map(|s| s.to_string());
        let external = EXTERNAL_CONTROL
            .iter()
            .find(|(g, _)| *g == name)
            .map(|(_, c)| c.to_string());
        if flag.is_none() && external.is_none() {
            println!("{:<38} {:>9}  {}", name, "-", "no control to run");
            continue;
        }

        let sites = mutable_sites(&pristine);
        if sites.is_empty() {
            println!("{:<38} {:>9}  {}", name, 0, "no failure path to break");
            continue;
        }

        let mut killed = 0usize;
        let mut survivors: Vec<usize> = Vec::new();
        for (at, len, _text) in &sites {
            let mut m = String::with_capacity(pristine.len());
            m.push_str(&pristine[..*at]);
            m.push_str("return 0");
            m.push_str(&pristine[at + len..]);
            std::fs::write(f, &m)?;
            let ctrl = match (&flag, &external) {
                (Some(fl), _) => code(&root, &name, &[fl]),
                (_, Some(c)) => code(&root, c, &[]),
                _ => unreachable!(),
            };
            // Restore before judging, so an early return can never leave the
            // tree mutated.
            std::fs::write(f, &pristine)?;
            if ctrl == "0" {
                survivors.push(line_of(&pristine, *at));
            } else {
                killed += 1;
            }
        }
        debug_assert_eq!(std::fs::read_to_string(f).unwrap_or_default(), pristine);

        let verdict = if survivors.is_empty() {
            "all killed".to_string()
        } else {
            total_survived.push(name.clone());
            format!(
                "SURVIVED at line{} {}",
                if survivors.len() == 1 { "" } else { "s" },
                survivors
                    .iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        println!(
            "{:<38} {:>9}  {}",
            name,
            format!("{}/{}", killed, sites.len()),
            verdict
        );
    }

    println!();
    if total_survived.is_empty() {
        println!("Every gate's control noticed every break in its failure path.");
    } else {
        println!(
            "{} gate(s) whose control did not notice:",
            total_survived.len()
        );
        for n in &total_survived {
            println!("  {}", n);
        }
        println!();
        println!("A survivor means the gate stopped being able to fail and its own");
        println!("control still passed. Usually the control exercises the checking");
        println!("FUNCTION but not the wiring from that function to the exit code.");
    }
    Ok(())
}

fn code(root: &std::path::Path, script: &str, args: &[&String]) -> String {
    let mut c = Command::new("python3");
    c.arg(format!("tools/{}", script));
    for a in args {
        c.arg(a.as_str());
    }
    c.current_dir(root);
    match c.output() {
        Ok(o) => {
            // A gate that needs --ssot/--vectors exits 2 from argparse without
            // running anything. Printing "2" in the run column would report a
            // refusal to parse arguments as a gate verdict -- the same
            // exit-status-mistaken-for-the-property mistake this command
            // exists to surface. Say what actually happened.
            let err = String::from_utf8_lossy(&o.stderr);
            if o.status.code() == Some(2) && err.contains("arguments are required") {
                return "args".into();
            }
            o.status
                .code()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "sig".into())
        }
        Err(_) => "ERR".into(),
    }
}

fn repo_root() -> Result<std::path::PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("git rev-parse failed")?;
    Ok(std::path::PathBuf::from(
        String::from_utf8_lossy(&out.stdout).trim(),
    ))
}

pub fn run(cmd: &GatesCmd) -> Result<()> {
    match cmd {
        GatesCmd::Sweep { controls_only } => sweep(*controls_only),
        GatesCmd::Mutate { only } => mutate(only.as_deref()),
        GatesCmd::Dead { repos, min_runs } => {
            let list: Vec<String> = if repos.is_empty() {
                ["gHashTag/trinity", "gHashTag/trinity-fpga", "gHashTag/t27"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            } else {
                repos.clone()
            };
            dead(&list, *min_runs)
        }
    }
}

fn gh(args: &[&str]) -> Result<String> {
    let out = Command::new("gh")
        .args(args)
        .output()
        .context("gh is not installed or not on PATH")?;
    if !out.status.success() {
        anyhow::bail!(
            "gh {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn count(repo: &str, id: &str, success_only: bool) -> Result<u64> {
    let path = if success_only {
        format!("repos/{repo}/actions/workflows/{id}/runs?status=success&per_page=1")
    } else {
        format!("repos/{repo}/actions/workflows/{id}/runs?per_page=1")
    };
    let s = gh(&["api", &path, "--jq", ".total_count"])?;
    Ok(s.parse().unwrap_or(0))
}

/// Is a zero success count over `total` lifetime runs too thin to mean
/// anything? A workflow may simply be new, or triggered by a path nobody has
/// touched. Lifted out of `dead` so the floor can be exercised without
/// reaching the network — inline, it was reachable only through `gh`.
fn too_few_runs_to_judge(total: u64, min_runs: u64) -> bool {
    total < min_runs
}

fn dead(repos: &[String], min_runs: u64) -> Result<()> {
    let mut rows: Vec<(String, String, u64)> = Vec::new();
    for repo in repos {
        let listing = gh(&[
            "api",
            &format!("repos/{repo}/actions/workflows?per_page=100"),
            "--jq",
            r#".workflows[]|select(.state=="active")|"\(.id)\t\(.name)""#,
        ])?;
        for line in listing.lines() {
            let mut it = line.splitn(2, '\t');
            let (id, name) = match (it.next(), it.next()) {
                (Some(a), Some(b)) => (a, b),
                _ => continue,
            };
            let total = count(repo, id, false)?;
            if too_few_runs_to_judge(total, min_runs) {
                continue;
            }
            if count(repo, id, true)? == 0 {
                rows.push((repo.clone(), name.to_string(), total));
            }
        }
    }

    rows.sort_by(|a, b| b.2.cmp(&a.2));
    if rows.is_empty() {
        println!("No active workflow with >= {min_runs} runs has a zero success count.");
        return Ok(());
    }

    let total: u64 = rows.iter().map(|r| r.2).sum();
    println!(
        "{} workflow(s) have never succeeded, across {} run(s).\n",
        rows.len(),
        total
    );
    for (repo, name, runs) in &rows {
        let short: String = name.chars().take(44).collect();
        println!("  {runs:>6}  {repo:<22} {short}");
    }
    println!();
    println!("A gate that has never been green carries no information: red before");
    println!("your change and red after it. Decide per workflow — fix it, make it");
    println!("workflow_dispatch only, or delete it. Leaving it red is the one");
    println!("option that costs every other gate in the repository.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// `GatesCmd` is a `Subcommand`, so asking clap what `--min-runs` defaults
    /// to needs a root parser. This one exists for no other purpose.
    #[derive(Parser)]
    struct Root {
        #[command(subcommand)]
        action: GatesCmd,
    }

    /// The floor `tri gates dead` actually ships with, read back out of clap
    /// rather than repeated as a literal here.
    fn shipped_floor() -> u64 {
        match Root::parse_from(["tri-gates", "dead"]).action {
            GatesCmd::Dead { min_runs, .. } => min_runs,
            other => panic!("this test is about Dead's default, got {other:?}"),
        }
    }

    /// The `--min-runs` floor exists because "0 successes" over 2 runs is not
    /// evidence of a dead gate, and reporting it as one would make this
    /// command the thing it is written to find: an alarm nobody reads.
    ///
    /// The guard this replaces declared `let below = 2u64; let at = 50u64;`
    /// and asserted `2 < 50` and `50 >= 50`. It named neither the shipped
    /// default nor the skip inside `dead`: lifted into a file containing no
    /// production code — not even a `use` of this crate — it still compiled
    /// and still passed, and setting `default_value_t` to 0 left all 173
    /// tests green while a two-run workflow became reportable as a dead gate.
    /// The first two assertions below read the shipped floor and put it
    /// through `too_few_runs_to_judge`, the predicate `dead` actually skips
    /// on. Between them they pin the comparison's strictness but barely
    /// constrain the number — assertion 1 fails only for a floor of 0, 1 or
    /// 2 — so the third bounds the value itself (#2374).
    #[test]
    fn the_floor_is_what_makes_a_zero_meaningful() {
        let floor = shipped_floor();

        // A day-old workflow with two runs must be skipped, not reported.
        assert!(
            too_few_runs_to_judge(2, floor),
            "--min-runs defaults to {floor}, so a workflow with 2 lifetime \
             runs and no success would be reported as a dead gate"
        );

        // A workflow standing exactly at the floor must be judged, not skipped.
        assert!(
            !too_few_runs_to_judge(floor, floor),
            "a workflow with exactly {floor} runs and no success must be reported"
        );

        // A floor low enough to judge a handful of runs is no floor at all.
        // At 3, both assertions above still pass while a three-run workflow
        // becomes reportable — the judgement `--min-runs` exists to prevent.
        // A bound rather than an exact pin: retuning stays possible, dropping
        // to a handful does not.
        assert!(
            floor >= 10,
            "--min-runs defaults to {floor}; below 10 a handful of runs is \
             treated as evidence, which is what this flag exists to prevent"
        );
    }
}

#[cfg(test)]
mod sweep_tests {
    use super::*;

    #[test]
    fn mutation_never_touches_the_control_itself() {
        // The mistake this guards against was made by hand first: one regex
        // over every `return 1..4` also rewrote the returns inside self_check,
        // so two sound controls reported a vacuous pass when what had actually
        // broken was their ability to report at all.
        let src = "\
def check(root):\n    if bad:\n        return 1\n    return 0\n\
def self_check():\n    if not ok:\n        return 1\n    return 0\n\
def main():\n    if problems:\n        return 2\n    return 0\n";
        let sites = mutable_sites(src);
        let lines: Vec<usize> = sites.iter().map(|(at, _, _)| line_of(src, *at)).collect();
        assert_eq!(lines, vec![3, 11], "got {:?}", sites);
        // The `return 1` at line 7 is self_check's own. Mutating it would
        // break the instrument rather than the subject.
        assert!(!lines.contains(&7));
    }

    #[test]
    fn a_returns_zero_is_not_a_failure_path() {
        // Only 1..4 are verdicts. Flipping `return 0` to `return 0` is a
        // no-op mutant, and a no-op mutant that "survives" would be scored as
        // a gap that is not there -- a finding invented by the tool.
        assert!(mutable_sites("def main():\n    return 0\n").is_empty());
        assert!(mutable_sites("def main():\n    return 5\n").is_empty());
    }

    #[test]
    fn nested_defs_do_not_end_the_control_region() {
        // `case()` and friends are defined INSIDE self_check, indented. Only a
        // `def` at column zero changes which region we are in.
        let src = "def self_check():\n    def case(x):\n        return 1\n    return 0\n";
        assert!(mutable_sites(src).is_empty());
    }

    #[test]
    fn control_files_are_not_counted_as_gates() {
        // T85: wp18_selftest_gate.py IS the control for wp18_conformance_gate.py.
        // Counting it as a gate in its own right would report it as having no
        // control -- inventing a finding out of the naming convention.
        assert!(IS_A_CONTROL.contains(&"wp18_selftest_gate.py"));
        assert!(EXTERNAL_CONTROL
            .iter()
            .any(|(g, c)| *g == "wp18_conformance_gate.py" && *c == "wp18_selftest_gate.py"));
    }

    #[test]
    fn a_gate_is_never_both_a_gate_and_its_own_control() {
        for (g, c) in EXTERNAL_CONTROL {
            assert_ne!(g, c, "a script cannot be its own negative control");
            assert!(
                IS_A_CONTROL.contains(c),
                "{c} must be excluded from the gate list"
            );
        }
    }
}

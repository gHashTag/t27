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

        /// Sweep this directory instead of `<repo>/tools`. Same refusals as
        /// `mutate --dir`: a missing path, a file, or anywhere outside a git
        /// work tree is rejected by name rather than run.
        #[arg(long)]
        dir: Option<String>,
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
    ///
    /// A control that is red BEFORE any mutation scores nothing and is named:
    /// red-no-matter-what would otherwise read as a perfect score.
    Mutate {
        /// Only this gate, by file name (e.g. check_vector_data.py).
        #[arg(long)]
        only: Option<String>,

        /// Push gates the OTHER way: `return 0` -> `return 1`, and ask whether
        /// anything notices a gate that fails on a clean tree.
        ///
        /// The default operator turns failures into passes, so a control made
        /// entirely of cases demanding RED satisfies every one of them. This
        /// asks the question those cases cannot: is there an assertion that
        /// requires SILENCE?
        #[arg(long)]
        loud: bool,

        /// Invert the CONDITIONS that reach a verdict, rather than the returns.
        ///
        /// The two return operators ask whether a gate can still reach its
        /// verdicts. This asks whether it reaches the RIGHT one: a gate that
        /// fires its FAIL branch on a healthy tree and its OK branch on a
        /// broken one passes both of them.
        #[arg(long)]
        invert: bool,

        /// Audit gates in this directory instead of `<repo>/tools`.
        ///
        /// The gate SCRIPTS refuse a path override on purpose -- a flag that
        /// aims a live gate somewhere harmless is a way to make it pass. This
        /// is the auditor, not a gate: pointing it at another repository is
        /// the whole job, and refusing to would mean every lesson this command
        /// has learned stays inside one tree.
        ///
        /// The dirty-tree refusal follows the directory, so an interrupted run
        /// is still recoverable with `git checkout` in the right repository.
        #[arg(long)]
        dir: Option<String>,

        /// Move the comparison in a boundary: `>` <-> `>=`, `<` <-> `<=`.
        ///
        /// The only operator that had no flag of its own, reachable solely
        /// through `--all`, so it is the one you most want to iterate on alone.
        /// Four operators selectable and this one not was exactly backwards.
        ///
        /// This text used to advertise a score -- "the worst kill rate of the
        /// five (26 of 77 across the tree)". Both halves went stale in one
        /// commit: fixing the scanner that stopped at `def self_check()`
        /// DOUBLED the site count, so the denominator was wrong and the rate
        /// was arithmetic on it. A number frozen in help text has no way to
        /// learn that, and a reader has no way to tell.
        ///
        /// The denominator here also mixes two populations -- every comparison,
        /// including loop bounds where surviving is correct -- so a rate was
        /// never the right summary anyway. Run the command; it prints what it
        /// found, today.
        #[arg(long)]
        boundary: bool,

        /// Neuter assertions: `assert C, "msg"` -> `assert True, "msg"`.
        ///
        /// The operator for gates whose verdicts are asserts rather than exit
        /// codes. Without it such a gate scores 0/0 in every column, which
        /// prints exactly like a gate with nothing to break.
        #[arg(long = "assert")]
        assert_op: bool,

        /// Re-measure everything, ignoring the cache.
        #[arg(long)]
        fresh: bool,

        /// Run all operators in one pass and print them as columns.
        ///
        /// Three commands answering one question is the same shape as two
        /// naming conventions or two parse commands: the operators are only
        /// comparable when read together, and read separately nobody noticed
        /// that `--invert` was printing the silent operator's numbers.
        #[arg(long)]
        all: bool,
    },
    /// Open pull requests whose path-filtered CI never ran.
    ///
    /// T133: a pull request that is CONFLICTING when an event fires does not
    /// get its path-filtered workflows for that event -- GitHub cannot compute
    /// the merge diff, so `paths:` cannot be evaluated, and only the path-less
    /// workflows run.
    ///
    /// The first version of this comment said "a CONFLICTING pull request loses
    /// most of its checks", on a correlation measured once: four conflicting
    /// pull requests had 3, 3, 9 and 7 checks while the rest had 21 to 35. An
    /// hour later two of those four reported 21 and 26 -- they had been
    /// mergeable when their events fired, kept those results, and only
    /// conflicted afterwards. **A conflict does not retract past runs.**
    ///
    /// So the detectable shape is not "conflicting". It is conflicting AND a
    /// check list far shorter than its siblings', which is why this command
    /// computes a reference from the non-conflicting pull requests rather than
    /// asserting from the state alone.
    ///
    /// The danger is not that the checks are red. It is that they are ABSENT,
    /// and a short list of green checks reads like a passing pull request. Two
    /// of the affected ones change `bootstrap/src/compiler.rs` -- the exact
    /// file whose gate carries the comment "a PR that rewrites the C emitter
    /// merges with the cross-target proof never running".
    Prs {
        /// owner/repo. Defaults to the repository of the working directory.
        #[arg(long)]
        repo: Option<String>,
    },
    /// What the tree says is required, against what the ruleset requires.
    ///
    /// The only drift class in this repository with no detector. A required
    /// status check is named in repository SETTINGS; no file in the tree can
    /// read it, so a comment claiming a gate blocks cannot go stale against
    /// anything. `seal-coverage.yml` records learning "the hard way in #2191"
    /// that renaming its job made a PR go BLOCKED -- true evidence that its
    /// context WAS required, and no evidence that it still is. It is not:
    /// `coverage` failed on 32 of the last 40 merged pull requests, and all 40
    /// merged.
    Required {
        /// owner/repo. Defaults to the repository of the working directory.
        #[arg(long)]
        repo: Option<String>,
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
    /// Workflows with no recent run on the default branch: their green is
    /// about frequency, not health.
    ///
    /// `dead` asks "ran a lot and never passed". This asks the opposite and
    /// harder question: "never ran, so nobody knows". Three gates in this
    /// repository were in that state at once -- rings-rust, secret-scan and
    /// cli-tri, all `paths:`-filtered on the root Cargo.toml, which nothing
    /// had edited in months. Editing it woke all three: seventeen ring crates
    /// had never compiled, 233 files carried a developer's home directory, and
    /// `tri rtl check` had been dying on a submodule that was declared but not
    /// registered. Every one of them had been reading as passing.
    Unmeasured {
        /// owner/repo, repeatable. Defaults to the repository you are in.
        #[arg(long = "repo")]
        repos: Vec<String>,
        /// Call a workflow unmeasured when its last default-branch run is
        /// older than this many days, or when it has none at all.
        #[arg(long, default_value_t = 30)]
        stale_days: u64,
    },
}

/// Gate scripts whose control lives in a SEPARATE file, and the file that
/// holds it. Counting these as control-less would be wrong; counting the
/// control files themselves as gates would be wrong twice.
/// Not 1:1. `check_gate_preconditions.py` is ONE control covering the
/// precondition branch of six gates at once -- the class this command found,
/// where a gate's own self-check plants faults inside a well-formed world and
/// never breaks the world's existence. Without these rows those branches keep
/// reporting as survivors while a control that does cover them sits in the
/// tree, which is the same "it exists but nothing connects it" mistake in the
/// tool rather than in the repository. It is also a gate in its own right, with
/// its own self-check, so it appears in the table below as well.
const EXTERNAL_CONTROL: &[(&str, &str)] = &[
    ("wp18_conformance_gate.py", "wp18_selftest_gate.py"),
    (
        "check_duplicate_agreement.py",
        "check_gate_preconditions.py",
    ),
    ("check_elab_ratchet.py", "check_gate_preconditions.py"),
    ("check_seal_coverage.py", "check_gate_preconditions.py"),
    ("check_specs_generate.py", "check_gate_preconditions.py"),
    ("check_specs_parse.py", "check_gate_preconditions.py"),
    ("check_vector_data.py", "check_gate_preconditions.py"),
];

/// Scripts that serve as somebody else's negative control AND are gates in
/// their own right. They stay in the gate list -- they have their own
/// self-check and are measured like anything else -- but naming them here says
/// the double role is intended rather than a wiring mistake.
const CONTROL_IS_ALSO_A_GATE: &[&str] = &["check_gate_preconditions.py"];

/// Files under tools/ that ARE controls rather than gates.
const IS_A_CONTROL: &[&str] = &[
    "wp18_selftest_gate.py",
    "wp18_gate_selfconsistent_selftest.py",
];

fn sweep(controls_only: bool, dir: Option<&str>) -> Result<()> {
    let (root, tools) = resolve_target(dir)?;
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&tools)
        .with_context(|| format!("read {}", tools.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let n = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            // Both separators. The hyphen was not an oversight worth ignoring:
            // aimed at another repository with --dir, the underscore-only
            // filter silently found nothing and reported an empty table, which
            // reads exactly like a clean suite. Two naming conventions for one
            // thing is a shape this campaign has met before.
            n.ends_with(".py")
                && (n.starts_with("check_")
                    || n.starts_with("check-")
                    || n.contains("gate")
                    || is_gate_by_property(
                        &root,
                        n,
                        &std::fs::read_to_string(p).unwrap_or_default(),
                    ))
        })
        .collect();
    files.sort();

    let mut uncontrolled: Vec<(String, Vec<String>)> = Vec::new();
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
        // Every declared flag, not the first. A gate may carry more than one
        // control; reporting a single one understates its coverage and made
        // `mutate` invent a survivor. Both commands read the set the same way.
        let flags: Vec<String> = ["--self-check-drop", "--self-check", "--selftest"]
            .iter()
            .filter(|fl| src.contains(&format!("\"{}\"", fl)))
            .map(|s| s.to_string())
            .collect();
        let external = EXTERNAL_CONTROL
            .iter()
            .find(|(g, _)| *g == name)
            .map(|(_, c)| c.to_string());

        let gate = if controls_only {
            "-".to_string()
        } else {
            code(&root, &tools, &name, &[])
        };
        // Every form, not only the two this command can RUN. A gate whose
        // control is a workflow fixture or a test file is controlled; calling
        // it uncontrolled because this tool cannot execute that form is a
        // statement about the tool.
        let forms = control_forms(&root, &src, &name);
        let ctrl = if flags.is_empty() && external.is_none() {
            uncontrolled.push((name.clone(), forms.clone()));
            if forms.is_empty() {
                "NONE".to_string()
            } else {
                // Found, but not runnable from here. Distinct from NONE on
                // purpose: "I cannot run it" and "it does not exist" are
                // different findings and were conflated once already.
                "OTHER".to_string()
            }
        } else if controls_only {
            "-".to_string()
        } else if !flags.is_empty() {
            // The worst verdict across the gate's controls: a green one does
            // not excuse a red sibling.
            flags
                .iter()
                .map(|fl| code(&root, &tools, &name, &[fl]))
                .find(|c| c != "0")
                .unwrap_or_else(|| "0".to_string())
        } else {
            code(&root, &tools, external.as_ref().unwrap(), &[])
        };
        rows.push((name, gate, ctrl));
    }

    // T112: the same refusal `mutate` grew one iteration ago, in the sibling
    // that did not get it. An empty set printed "0 gate(s); 0 with no control"
    // and exited 0 -- a sentence in which every number is zero and which reads
    // as a clean sweep. Fixing a vacuous pass in one command and not in the one
    // beside it is how the class survives its own repair.
    if files.is_empty() {
        println!(
            "FAIL: no gate scripts under {}\n\n  \
             Looked for *.py named check_* / check-* or containing \"gate\".\n  \
             Nothing was swept, which is not the same as nothing being wrong.",
            tools.display()
        );
        anyhow::bail!("no gate scripts under {}", tools.display());
    }

    println!("{:<38} {:>6}  {:>6}", "gate", "run", "control");
    for (n, g, c) in &rows {
        println!("{:<38} {:>6}  {:>6}", n, g, c);
    }
    println!();
    let none: Vec<&(String, Vec<String>)> =
        uncontrolled.iter().filter(|(_, f)| f.is_empty()).collect();
    let other: Vec<&(String, Vec<String>)> =
        uncontrolled.iter().filter(|(_, f)| !f.is_empty()).collect();
    println!(
        "{} gate(s); {} with no control in any form; {} with one this command cannot run:",
        rows.len(),
        none.len(),
        other.len()
    );
    for (n, _) in &none {
        println!("  NONE   {}", n);
    }
    for (n, forms) in &other {
        println!("  OTHER  {}  ->  {}", n, forms.join(", "));
    }
    println!();
    // T112: the forms searched, printed whether or not anything was found.
    // Reporting "no control" while having looked for exactly one kind is how
    // six gates in another repository were called control-less, three of them
    // wrongly. A reader cannot weigh a NONE without seeing the search behind it.
    // T112: the FILE FILTER is an assumption too, and a narrower one than the
    // control search. `conformance_check.py` and `signal_health.py` are gates in
    // the second repository and match neither `check_*` nor "gate"; this command
    // never saw them, and a table that does not list them reads as a repository
    // that does not have them. Printed with the row count so the denominator is
    // never read as "every gate here".
    // T113: `rows.len()` is the count AFTER the control files are excluded, so
    // the old wording ("13 of 28 matched") understated the match by exactly the
    // number of controls and made me report 15 invisible files where there are
    // 13. The disclosure line added for honesty was itself off by two.
    println!(
        "Files considered: {} gate(s) from {} *.py under {} — by name (check_*, \
         check-*, *gate*) or by property (a workflow invokes it AND it can exit \
         non-zero); control files excluded.",
        rows.len(),
        std::fs::read_dir(&tools)
            .map(|d| d
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|x| x == "py"))
                .count())
            .unwrap_or(rows.len()),
        tools.display()
    );
    println!("Selection by name alone failed twice: `check-` against `check_`, and");
    println!("then verify_*/run_*, which hid three verdict-carrying CI scripts here.");
    println!();
    println!("Forms searched: a --self-check/--selftest flag in the script; the");
    println!("EXTERNAL_CONTROL table in this command; tests/test_<name>.py; and a");
    println!("workflow naming the script beside fixture/expect/planted/broken/must.");
    println!("The last is reported as a CANDIDATE -- it is a keyword match, not proof");
    println!("that anything is asserted. Nothing here upgrades an absence to a pass.");
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
/// Does this line end whatever function preceded it?
///
/// Any statement at column 0 that is not a `def`, a decorator, or a continuation
/// of the line before. Comments and blank lines decide nothing.
fn leaves_function(line: &str) -> bool {
    let t = line.trim_end();
    if t.is_empty() {
        return false;
    }
    let first = t.as_bytes()[0];
    if first == b' ' || first == b'\t' || first == b'#' || first == b'@' || first == b')' {
        return false;
    }
    true
}

fn is_control_fn(name: &str) -> bool {
    name.contains("self_check") || name.contains("selftest") || name.contains("self_test")
}

/// Whether a line's text is prose inside a triple-quoted string.
///
/// The three LINE-oriented site finders had no string state at all, and
/// `leaves_function` decides on the first byte of a line. So a flush-left line
/// inside a control function's docstring -- ordinary prose, wrapped -- read as a
/// top-level statement and cleared `in_control`, handing the operators the
/// control's own `assert` and `return 1` as mutable sites. Neutering a control
/// makes it pass, so those sites are recorded as survivors nobody can ever kill,
/// and the doc comment on `leaves_function` says exactly why that is the worst
/// outcome: it breaks the instrument instead of the thing being measured.
///
/// Measured on two files differing by four spaces of indentation on ONE
/// docstring line: 3 assert sites versus 0, and every extra site was inside the
/// control function.
///
/// The same blindness scored `assert anything.` -- word-wrapped prose in a
/// module docstring -- as an assertion to neuter.
///
/// `boundary_sites` already tracked this, because it is byte-oriented and had to.
struct Docstring {
    open: Option<[u8; 3]>,
}

impl Docstring {
    fn new() -> Self {
        Self { open: None }
    }

    /// True when this line BEGINS inside a triple-quoted string, i.e. its text
    /// is prose. Advances the state past whatever the line opens or closes.
    ///
    /// A line that opens a docstring is still code up to the quote, so it
    /// returns false and lets the caller read it; only the lines after it are
    /// prose. That is the same rule a reader applies.
    fn is_prose(&mut self, line: &str) -> bool {
        let started_inside = self.open.is_some();
        let b = line.as_bytes();
        let mut i = 0usize;
        while i + 3 <= b.len() {
            let t = [b[i], b[i + 1], b[i + 2]];
            if t == [b'"', b'"', b'"'] || t == [b'\'', b'\'', b'\''] {
                match self.open {
                    Some(o) if o == t => {
                        self.open = None;
                        i += 3;
                        continue;
                    }
                    // A `'''` inside a `"""` block is text, not a delimiter.
                    Some(_) => {}
                    None => {
                        self.open = Some(t);
                        i += 3;
                        continue;
                    }
                }
            }
            i += 1;
        }
        started_inside
    }
}

/// Is this return expression a VERDICT LITERAL, or a ternary of them?
///
/// T103: both predicates used to scan for a standalone digit anywhere in the
/// expression, which is not the same question. `return v == 0` is a boolean
/// comparison; `return out.splitlines()[0][:88] if out else "(nothing)"` is a
/// string with an index in it. Both were taken as verdicts, and the loud
/// operator reported two helper functions as gates that nothing keeps silent.
///
/// A return is a verdict when the whole expression is a literal, or a ternary
/// whose two arms are literals. Anything else is a value the caller decides
/// about, and mutating it perturbs a helper rather than a failure path.
fn verdict_literals(expr: &str) -> Option<Vec<i32>> {
    fn lit(s: &str) -> Option<i32> {
        let s = s.trim();
        if s.len() == 1 && s.chars().all(|c| c.is_ascii_digit()) {
            s.parse().ok()
        } else {
            None
        }
    }
    let e = expr.trim();
    if let Some(v) = lit(e) {
        return Some(vec![v]);
    }
    // `A if C else B` -- arms must both be literals; the condition may be
    // anything, since neither mutation preserves it.
    let (a, rest) = e.split_once(" if ")?;
    let (_cond, b) = rest.rsplit_once(" else ")?;
    Some(vec![lit(a)?, lit(b)?])
}

/// Every failure path that belongs to the gate's own logic: byte offset, the
/// length to replace, and what to put there.
///
/// T90: the first version matched only a BARE `return 1..4`. Measured across
/// the twelve gate scripts: 34 sites seen, 8 missed -- seven ternary returns
/// (`return 1 if bad else 0`) and one `raise SystemExit(3)`. Its denominator
/// was short by a fifth of what it claimed to measure, and it reported
/// pack_index_consistency_gate.py as having "no failure path to break" when
/// every one of that gate's verdicts is a ternary. A scanner that cannot see a
/// failure path scores it as covered, which is the same substitution this
/// command exists to catch.
///
/// Line-based on purpose: these files are flat, and a real Python parse would
/// buy nothing a `def` at column zero does not already give.
/// Which way a mutant pushes the gate.
///
/// T99: `Silent` is the operator this command was missing. Every mutation it
/// made turned a failure into a pass, so a control made entirely of cases that
/// demand RED satisfied all of them -- and a gate rewritten to fail on a clean
/// tree passed unnoticed. Measured on check_duplicate_agreement.py, whose two
/// controls both assert exit 1: a gate reporting a split where every copy
/// agrees was caught only by the sibling control, which exists for a different
/// branch. Coverage by accident is not coverage, and a green report does not
/// tell the two apart.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Direction {
    /// `return 1` -> `return 0`: can the gate still fail?
    Silent,
    /// `return 0` -> `return 1`: does anything notice the gate getting LOUDER?
    Loud,
    /// `if <cond>:` -> `if not (<cond>):`, on conditions whose body carries a
    /// verdict. The two return operators ask whether the gate can still reach
    /// its verdicts; this asks whether it reaches the RIGHT one.
    ///
    /// T104: scoped to conditions whose body holds a `return 1..4`, a
    /// `raise SystemExit(N)` or a FAIL print. Inverting a loop guard or a
    /// plumbing check makes the gate CRASH, and a control that reds on a
    /// traceback would score that as a kill -- the wrong reason, which is the
    /// mistake this command exists to catch.
    Invert,
    /// `>` <-> `>=`, `<` <-> `<=`: an off-by-one at a comparison that decides a
    /// verdict. The three operators above ask whether a gate reaches a verdict
    /// and whether it reaches the right one. This asks whether it reaches it at
    /// the right PLACE -- ratchets and thresholds live on a boundary, and a
    /// control that tests "clearly worse" and "clearly better" never tests
    /// equal.
    Boundary,
    /// `assert C, "msg"` -> `assert True, "msg"`: can an assertion still fail?
    ///
    /// T125. The three return operators and the boundary operator all read
    /// `return` / `sys.exit` / comparisons. A gate whose verdicts are `assert`s
    /// has NONE of those, and scored 0/0 in every column -- eighteen assertions
    /// in one file, invisible to every question the tool could ask. An assert
    /// is a verdict delivered through a traceback; neutering it is exactly the
    /// silent operator, spelled the way a test-shaped gate spells it.
    Assert,
}

/// Comparison swaps, longest first so `>=` is matched before `>`.
const BOUNDARY_SWAPS: &[(&str, &str)] = &[(">=", ">"), ("<=", "<"), (">", ">="), ("<", "<=")];

/// Sites for the boundary operator: every comparison outside a control
/// function, outside a comment, and outside a string literal.
///
/// Deliberately NOT scoped to verdict-bearing lines the way `invert_sites` is.
/// That scope was written for conditions, where the body says whether the line
/// decides anything; a comparison decides through a variable that may be read
/// three statements later, and a scope guessed in advance would be exactly the
/// kind of limitation this campaign has twice found to be invented. Measure
/// first, narrow only on evidence.
/// `assert <cond>[, msg]` outside a control function, neutered to `assert True`.
///
/// The message is KEPT: a mutant that also drops the text would be caught by a
/// control asserting that text, and the kill would be for the wrong reason --
/// the message changing rather than the check stopping. Only the condition
/// moves.
fn assert_sites(src: &str) -> Vec<(usize, usize, String)> {
    let mut sites = Vec::new();
    let mut in_control = false;
    let mut off = 0usize;
    let mut doc = Docstring::new();
    for line in src.split_inclusive('\n') {
        if doc.is_prose(line) {
            off += line.len();
            continue;
        }
        if let Some(rest) = line.strip_prefix("def ") {
            let fname: String = rest.chars().take_while(|c| *c != '(').collect();
            in_control = is_control_fn(&fname);
        } else if leaves_function(line) {
            // T125: a function ends at the next TOP-LEVEL statement, not only at
            // the next `def`. Without this, everything after the last function
            // inherits that function's control status -- and when the last
            // function is a self_check, the whole `if __name__ == "__main__":`
            // block below it is scored as control code.
            //
            // Sixteen assertions in gft_backprop_microcode.py live in exactly
            // that block. The assert operator found ONE site -- the only assert
            // above the self_check -- and printed 0/1, which reads as a gate
            // with almost nothing to break.
            in_control = false;
        }
        if !in_control {
            let t = line.trim_start();
            let col = line.len() - t.len();
            let body = t.trim_end();
            if let Some(rest) = body.strip_prefix("assert ") {
                // `assert True` is already vacuous; mutating it changes nothing
                // and would score a site that cannot be killed by anyone.
                if !rest.trim_start().starts_with("True") {
                    // Split on the LAST top-level comma is wrong: a message may
                    // contain one, and so may the condition. The message is
                    // whatever follows the first comma that is not inside
                    // brackets or quotes -- and if there is none, the whole rest
                    // is the condition.
                    let b = rest.as_bytes();
                    let (mut depth, mut quote, mut cut) = (0i32, None::<u8>, None);
                    // A backslash escapes the NEXT byte, so it must skip two.
                    // `continue` alone advanced one, and the escaped quote then
                    // closed the string. Measured on two real shapes:
                    //   assert s == "a\",b", "the header row must be quoted"
                    //     -> mutant `assert True,b", "the header ...` -- which
                    //        does not parse, so python exits 1 and the site is
                    //        scored KILLED. A false green in the one column this
                    //        command exists to make trustworthy.
                    //   assert t == 'it\'s', "the label must carry an apostrophe"
                    //     -> `quote` stays Some for the rest of the line, no
                    //        top-level comma is found, and the MESSAGE IS
                    //        DROPPED -- violating the invariant stated three
                    //        lines above this loop.
                    // `boundary_sites`, byte-oriented, had `i += 2` all along.
                    let mut skip_next = false;
                    for (i, &c) in b.iter().enumerate() {
                        if skip_next {
                            skip_next = false;
                            continue;
                        }
                        match quote {
                            Some(q) => {
                                if c == b'\\' {
                                    skip_next = true;
                                    continue;
                                }
                                if c == q {
                                    quote = None;
                                }
                            }
                            None => match c {
                                b'"' | b'\'' => quote = Some(c),
                                b'(' | b'[' | b'{' => depth += 1,
                                b')' | b']' | b'}' => depth -= 1,
                                b',' if depth == 0 => {
                                    cut = Some(i);
                                    break;
                                }
                                _ => {}
                            },
                        }
                    }
                    let replacement = match cut {
                        Some(i) => format!("assert True{}", &rest[i..]),
                        None => "assert True".to_string(),
                    };
                    sites.push((off + col, body.len(), replacement));
                }
            }
        }
        off += line.len();
    }
    sites
}

fn boundary_sites(src: &str) -> Vec<(usize, usize, String)> {
    let b = src.as_bytes();
    let mut sites = Vec::new();
    let mut i = 0usize;
    let mut at_line_start = true;
    let mut in_control = false;
    let mut triple: Option<[u8; 3]> = None;
    let mut quote: Option<u8> = None;
    let mut comment = false;

    while i < b.len() {
        if at_line_start {
            at_line_start = false;
            // Only when not inside a docstring: `def` at column 0 inside one is
            // prose, and treating it as a definition is how a scanner walks out
            // of a string it is still in.
            if triple.is_none() && src[i..].starts_with("def ") {
                let fname: String = src[i + 4..].chars().take_while(|c| *c != '(').collect();
                in_control = is_control_fn(&fname);
            } else if triple.is_none() {
                // T125 again, in the one scanner that never got it. A function
                // ends at the next TOP-LEVEL statement, not only at the next
                // `def` -- and the other three site finders say so in these
                // exact words, because they are line-oriented and this one is
                // byte-oriented, so the fix did not transfer when `--boundary`
                // was added.
                //
                // Cost, measured on tools/gft_backprop_microcode.py: it holds
                // `def self_check()` at line 380 and drops EVERYTHING after it,
                // to the end of the file. The boundary column read 31 sites
                // where the file has more, and the missing region is the
                // `if __name__ == "__main__":` block -- which is where its
                // accuracy thresholds live.
                //
                // The count looked right by coincidence: 31 also happens to be
                // the number of comparisons before `__main__`, so the totals
                // matched a plausible story and the scan had actually stopped
                // nine lines earlier.
                let line: &str = src[i..].split('\n').next().unwrap_or("");
                if leaves_function(line) {
                    in_control = false;
                }
            }
        }
        let c = b[i];
        if c == b'\n' {
            at_line_start = true;
            comment = false;
            quote = None; // an unterminated single quote cannot cross a line
            i += 1;
            continue;
        }
        // T107b: triple quotes FIRST. Tracking quotes per line took every `>`
        // in every module docstring -- lines 10, 43, 136, 230 of four different
        // gates, prose about ratchets and usage, reported as surviving mutants.
        // The count was real and its meaning was not (number-audit 8.5); this
        // is the check that stops it, and the reason the operator was measured
        // before it was believed.
        if quote.is_none() && !comment && i + 2 < b.len() {
            let t3 = [b[i], b[i + 1], b[i + 2]];
            if (t3 == [b'"', b'"', b'"']) || (t3 == [b'\'', b'\'', b'\'']) {
                match triple {
                    Some(open) if open == t3 => triple = None,
                    Some(_) => {}
                    None => triple = Some(t3),
                }
                i += 3;
                continue;
            }
        }
        if triple.is_some() || comment {
            i += 1;
            continue;
        }
        match quote {
            Some(q) => {
                if c == b'\\' {
                    i += 2;
                    continue;
                }
                if c == q {
                    quote = None;
                }
            }
            None => {
                if c == b'"' || c == b'\'' {
                    quote = Some(c);
                } else if c == b'#' {
                    comment = true;
                } else if (c == b'<' || c == b'>') && !in_control {
                    let prev = if i > 0 { b[i - 1] } else { b' ' };
                    let next = if i + 1 < b.len() { b[i + 1] } else { b' ' };
                    let shift = next == c || prev == c;
                    let annot = c == b'>' && prev == b'-';
                    if !shift && !annot {
                        let two = next == b'=';
                        let found = match (c, two) {
                            (b'>', true) => ">=",
                            (b'<', true) => "<=",
                            (b'>', false) => ">",
                            _ => "<",
                        };
                        if let Some((_, to)) = BOUNDARY_SWAPS.iter().find(|(f, _)| *f == found) {
                            sites.push((i, found.len(), to.to_string()));
                        }
                        if two {
                            i += 1;
                        }
                    }
                }
            }
        }
        i += 1;
    }
    sites
}
fn mutable_sites(src: &str) -> Vec<(usize, usize, String)> {
    sites_in_direction(src, Direction::Silent)
}

fn sites_in_direction(src: &str, dir: Direction) -> Vec<(usize, usize, String)> {
    // T105: this delegation is the whole of `--invert`, and for one merged
    // commit it did not exist. `Direction::Invert` was declared and documented,
    // `invert_sites` was written and unit-tested, and NOTHING joined them:
    // mutate() chose `if loud { Loud } else { Silent }`, so the flag printed an
    // invert banner over a silent run. The result published from it -- "one
    // invert survivor, the same declared branch" -- was the silent survivor
    // relabelled, and it looked right precisely BECAUSE it was a real
    // measurement of the wrong thing.
    if matches!(dir, Direction::Invert) {
        return invert_sites(src);
    }
    if matches!(dir, Direction::Boundary) {
        return boundary_sites(src);
    }
    if matches!(dir, Direction::Assert) {
        return assert_sites(src);
    }
    let mut sites = Vec::new();
    let mut in_control = false;
    let mut off = 0usize;
    let mut doc = Docstring::new();
    for line in src.split_inclusive('\n') {
        if doc.is_prose(line) {
            off += line.len();
            continue;
        }
        if let Some(rest) = line.strip_prefix("def ") {
            let fname: String = rest.chars().take_while(|c| *c != '(').collect();
            in_control = is_control_fn(&fname);
        } else if leaves_function(line) {
            // T125: a function ends at the next TOP-LEVEL statement, not only at
            // the next `def`. Without this, everything after the last function
            // inherits that function's control status -- and when the last
            // function is a self_check, the whole `if __name__ == "__main__":`
            // block below it is scored as control code.
            //
            // Sixteen assertions in gft_backprop_microcode.py live in exactly
            // that block. The assert operator found ONE site -- the only assert
            // above the self_check -- and printed 0/1, which reads as a gate
            // with almost nothing to break.
            in_control = false;
        }
        if !in_control {
            let t = line.trim_start();
            let col = line.len() - t.len();
            let body = t.trim_end();
            // Whole-line replacement in every case, so a ternary is neutered
            // entirely rather than having one of its arms edited.
            if let Some(expr) = body.strip_prefix("return ") {
                match dir {
                    Direction::Silent
                        if verdict_literals(expr)
                            .is_some_and(|v| v.iter().any(|&x| (1..=4).contains(&x))) =>
                    {
                        sites.push((off + col, body.len(), "return 0".to_string()));
                    }
                    // A bare `return 0`, or a ternary that can yield 0. Both
                    // become an unconditional failure, which is what LOUD means.
                    Direction::Loud if verdict_literals(expr).is_some_and(|v| v.contains(&0)) => {
                        sites.push((off + col, body.len(), "return 1".to_string()));
                    }
                    _ => {}
                }
            } else if let Some(rest) = body.strip_prefix("sys.exit(") {
                // T114: the same verdict, spelled the way half this repository
                // spells it. `verify_multitarget.py` carries every one of its
                // exits through sys.exit() and scored 0/0 on both return
                // operators -- two empty columns that read as "nothing here to
                // break" for a gate that is nothing but verdicts.
                //
                // `sys.exit(main())` is a dispatch, not a verdict, and
                // verdict_literals rejects it for the same reason it rejects
                // `raise SystemExit(main())`.
                if let Some(inner) = rest.strip_suffix(')') {
                    match dir {
                        Direction::Silent
                            if verdict_literals(inner)
                                .is_some_and(|v| v.iter().any(|&x| (1..=4).contains(&x))) =>
                        {
                            sites.push((off + col, body.len(), "sys.exit(0)".to_string()));
                        }
                        Direction::Loud
                            if verdict_literals(inner).is_some_and(|v| v.contains(&0)) =>
                        {
                            sites.push((off + col, body.len(), "sys.exit(1)".to_string()));
                        }
                        _ => {}
                    }
                }
            } else if let Some(rest) = body.strip_prefix("raise SystemExit(") {
                // `raise SystemExit(main())` is a dispatch, not a verdict.
                if let Some(inner) = rest.strip_suffix(')') {
                    match dir {
                        Direction::Silent
                            if verdict_literals(inner)
                                .is_some_and(|v| v.iter().any(|&x| (1..=4).contains(&x))) =>
                        {
                            sites.push((off + col, body.len(), "raise SystemExit(0)".to_string()));
                        }
                        Direction::Loud
                            if verdict_literals(inner).is_some_and(|v| v.contains(&0)) =>
                        {
                            sites.push((off + col, body.len(), "raise SystemExit(1)".to_string()));
                        }
                        _ => {}
                    }
                }
            }
        }
        off += line.len();
    }
    sites
}

/// Conditions whose body carries a verdict: `(offset, len, replacement)`.
///
/// Line-based like everything else here, and the body is read to a dedent
/// rather than parsed. A condition whose body only prints or only accumulates
/// is not a decision this command has anything to say about.
fn invert_sites(src: &str) -> Vec<(usize, usize, String)> {
    let lines: Vec<&str> = src.split_inclusive('\n').collect();
    let mut sites = Vec::new();
    let mut in_control = false;
    let mut off = 0usize;
    let offsets: Vec<usize> = {
        let mut v = Vec::with_capacity(lines.len());
        let mut o = 0usize;
        for l in &lines {
            v.push(o);
            o += l.len();
        }
        v
    };
    let mut doc = Docstring::new();
    for (i, line) in lines.iter().enumerate() {
        off = offsets[i];
        if doc.is_prose(line) {
            continue;
        }
        if let Some(rest) = line.strip_prefix("def ") {
            let fname: String = rest.chars().take_while(|c| *c != '(').collect();
            in_control = is_control_fn(&fname);
        } else if leaves_function(line) {
            // T125: a function ends at the next TOP-LEVEL statement, not only at
            // the next `def`. Without this, everything after the last function
            // inherits that function's control status -- and when the last
            // function is a self_check, the whole `if __name__ == "__main__":`
            // block below it is scored as control code.
            //
            // Sixteen assertions in gft_backprop_microcode.py live in exactly
            // that block. The assert operator found ONE site -- the only assert
            // above the self_check -- and printed 0/1, which reads as a gate
            // with almost nothing to break.
            in_control = false;
        }
        if in_control {
            continue;
        }
        let t = line.trim_start();
        let indent = line.len() - t.len();
        if indent == 0 {
            continue;
        }
        let body_line = t.trim_end();
        let cond = match body_line
            .strip_prefix("if ")
            .and_then(|c| c.strip_suffix(':'))
        {
            Some(c) if !c.is_empty() => c,
            _ => continue,
        };
        // The body, to the first line at or left of this indent.
        let mut carries_verdict = false;
        for l in lines.iter().skip(i + 1) {
            let s = l.trim_end();
            if !s.trim().is_empty() && (l.len() - l.trim_start().len()) <= indent {
                break;
            }
            let b = s.trim();
            if b.starts_with("return ")
                && verdict_literals(&b[7..])
                    .is_some_and(|v| v.iter().any(|&x| (1..=4).contains(&x)))
                || b.starts_with("raise SystemExit(")
                || b.contains("FAIL")
            {
                carries_verdict = true;
                break;
            }
        }
        if carries_verdict {
            sites.push((off + indent, body_line.len(), format!("if not ({cond}):")));
        }
    }
    let _ = off;
    sites
}

/// Lines carrying a `# mutant-equivalent: <why>` claim, mapped to the reason.
///
/// The marker may sit on the code line itself or anywhere in the contiguous
/// comment block above it; it names the first following line that is neither a
/// comment nor blank. A fixed offset would break the moment the proof needed
/// more than one line -- which is the first thing it needed.
/// Equivalence claims this run refutes.
///
/// A claim says the mutant at that line cannot die. If it died, the claim is
/// false -- either it was wrong when written, or the code moved out from under
/// it. Both read as settled analysis to whoever comes next.
///
/// A line can hold more than one mutable site (`if a < 1 or b < 1:` holds two),
/// so this compares COUNTS: a claimed line with two sites of which one survived
/// has been contradicted once. Reporting only when the line vanishes from the
/// survivor list entirely would miss exactly that case, and a partially-true
/// claim is the harder one to spot by eye.
///
/// Separate from the run loop so it can be tested without a repository, a
/// mutant, or a subprocess -- the check exists because nothing had ever
/// falsified one of these claims, and a checker nobody can test is the same
/// failure one level up.
fn contradicted_claims(
    gate: &str,
    dir: &str,
    site_lines: &[usize],
    survivors: &[usize],
    claims: &std::collections::HashMap<usize, String>,
) -> Vec<String> {
    let mut out = Vec::new();
    // Sorted so the report is stable: a HashMap's order would reshuffle the
    // block between runs and make a diff of two reports unreadable.
    let mut lines: Vec<&usize> = claims.keys().collect();
    lines.sort();
    for line in lines {
        let n_sites = site_lines.iter().filter(|l| *l == line).count();
        if n_sites == 0 {
            continue;
        }
        let n_surv = survivors.iter().filter(|l| *l == line).count();
        if n_surv < n_sites {
            out.push(format!(
                "{}:{} -- {} of {} {} mutant(s) DIED, but the line claims: {}",
                gate,
                line,
                n_sites - n_surv,
                n_sites,
                dir,
                claims[line]
            ));
        }
    }
    out
}

fn equivalence_claims(src: &str) -> std::collections::HashMap<usize, String> {
    const MARK: &str = "mutant-equivalent:";
    let lines: Vec<&str> = src.lines().collect();
    let mut out = std::collections::HashMap::new();
    for (i, l) in lines.iter().enumerate() {
        // The marker must OPEN the comment, not merely appear inside it.
        //
        // This used to be `l.find(MARK)` anywhere in any comment, and prose
        // that MENTIONS the marker -- "that reasoning now sits on the line as a
        // `# mutant-equivalent:` claim" -- registered as a claim of its own,
        // bound to whatever code line happened to follow. A claim nobody made,
        // attached to a line it says nothing about, which the refutation check
        // would then report as contradicted the moment that line's mutant died.
        //
        // Found by writing exactly that sentence and watching the count go to
        // two. Every real claim in the tree already opens its comment this way.
        let t = l.trim_start();
        let Some(rest) = t.strip_prefix('#') else {
            continue;
        };
        let Some(why) = rest.trim_start().strip_prefix(MARK) else {
            continue;
        };
        let why = why.trim().to_string();
        // Walk to the first line that is code.
        let mut j = i + 1;
        while j < lines.len() {
            let t = lines[j].trim();
            if !t.is_empty() && !t.starts_with('#') {
                break;
            }
            j += 1;
        }
        if j < lines.len() {
            out.insert(j + 1, why);
        }
    }
    out
}

/// Where to audit, and a refusal for every way the answer can be nothing.
///
/// T111: each branch below is a degenerate input this flag accepted on the day
/// it shipped, found by feeding it the inputs its own author had no reason to
/// try -- which is the lesson the command exists to teach, arriving in the
/// command.
fn resolve_target(dir: Option<&str>) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    let Some(d) = dir else {
        let r = repo_root()?;
        let t = r.join("tools");
        return Ok((r, t));
    };
    let tools = std::fs::canonicalize(d).with_context(|| format!("--dir {d}"))?;
    if !tools.is_dir() {
        // Previously this reached the dirty-tree check and died with
        // "git status failed", which names the instrument rather than the
        // mistake.
        anyhow::bail!("--dir {d} is not a directory");
    }
    // The gate runs with cwd at ITS repository root, not at the directory
    // holding it: a gate that resolves paths relative to the working directory
    // would otherwise be run somewhere it has never been run, and the
    // difference would look like a finding.
    //
    // And a directory outside any repository is REFUSED, not defaulted. This
    // command rewrites gate files in place and restores them afterwards; the
    // restore is only a promise because `git checkout` can undo an interrupted
    // run. Outside a work tree there is no undo, and the dirty-tree guard that
    // is supposed to protect against exactly that passed silently there --
    // `git status` fails, stdout is empty, and empty reads as clean.
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&tools)
        .output();
    match out {
        Ok(o) if o.status.success() => Ok((
            std::path::PathBuf::from(String::from_utf8_lossy(&o.stdout).trim().to_string()),
            tools,
        )),
        _ => anyhow::bail!(
            "--dir {d} is not inside a git work tree.\n\
             This command rewrites each gate in place and restores it. That restore \
             is only recoverable because `git checkout` exists; with no repository \
             an interrupted run leaves a mutated file and no way back."
        ),
    }
}

/// Every FORM a negative control takes, and the evidence found for each.
///
/// T112: the tool used to look for one thing -- a `--self-check`-shaped flag --
/// plus a hand-kept table of siblings. Pointed at another repository it reported
/// "no control" for six gates, three of which are well controlled: two by
/// workflow jobs with clean/broken fixtures asserting exact counts, one by a
/// test file whose docstring names this campaign's own lesson.
///
/// So the search is widened, and -- more importantly -- the FORMS SEARCHED are
/// printed. An absence proved by one mechanism is a statement about the
/// mechanism, and the only way a reader can weigh a "none" is to see what was
/// looked for.
///
/// Workflow evidence is reported as a CANDIDATE, never as proof. A heuristic
/// that upgrades "no control" to "controlled" is the one error direction that
/// hurts here: an uncontrolled gate reading as controlled is exactly the
/// false green this command exists to find.
/// Is this script a gate, by what it DOES rather than by what it is called?
///
/// T113: naming was the selector for the whole campaign and it failed twice --
/// `check-` against `check_`, and then `verify_*` / `run_*`, which is how three
/// verdict-carrying CI scripts in this very repository stayed invisible to a
/// command whose output looks exhaustive.
///
/// The property that actually matters is measurable: a workflow invokes it, and
/// it can exit non-zero. Anything that can turn a pipeline red is a gate whether
/// or not its name says so. Name-matching stays as a second rule, because a gate
/// may be invoked by a script rather than by a workflow directly.
fn is_gate_by_property(root: &std::path::Path, name: &str, src: &str) -> bool {
    // T119: a TERNARY exit counts. `sys.exit(0 if ok else 1)` is the ordinary
    // way this repository ends a verifier, and a substring test for
    // "sys.exit(1" cannot see it -- so a gate that fails correctly was not
    // classified as a gate at all.
    //
    // Found by a wrong claim of my own: a one-off grep with this exact blind
    // spot said verify_trainer_c.py "could not fail", and its last line is
    // `sys.exit(0 if ok else 1)`. The campaign wrote verdict_literals() to
    // handle ternaries in the mutation scanner months ago, and the shortcut
    // here reintroduced the same blindness in the selector.
    let exits_nonzero = src.lines().any(|l| {
        let t = l.trim();
        let inner = t
            .strip_prefix("sys.exit(")
            .and_then(|r| r.strip_suffix(')'))
            .or_else(|| {
                t.strip_prefix("raise SystemExit(")
                    .and_then(|r| r.strip_suffix(')'))
            })
            .or_else(|| t.strip_prefix("return "));
        match inner {
            Some(e) => verdict_literals(e).is_some_and(|v| v.iter().any(|&x| (1..=4).contains(&x))),
            None => false,
        }
    });
    if !exits_nonzero {
        return false;
    }
    let Ok(rd) = std::fs::read_dir(root.join(".github/workflows")) else {
        return false;
    };
    rd.filter_map(|e| e.ok()).any(|e| {
        let p = e.path();
        p.extension().is_some_and(|x| x == "yml" || x == "yaml")
            && std::fs::read_to_string(&p)
                .unwrap_or_default()
                .contains(name)
    })
}

fn control_forms(root: &std::path::Path, src: &str, name: &str) -> Vec<String> {
    let mut found = Vec::new();

    for fl in ["--self-check-drop", "--self-check", "--selftest"] {
        if src.contains(&format!("\"{}\"", fl)) {
            found.push(format!("flag {}", fl));
        }
    }
    if let Some((_, c)) = EXTERNAL_CONTROL.iter().find(|(g, _)| *g == name) {
        found.push(format!("sibling {}", c));
    }

    let stem = name.trim_end_matches(".py");
    for cand in [
        format!("tests/test_{}.py", stem),
        format!("tests/{}_test.py", stem),
        format!("tests/test_{}.py", stem.trim_start_matches("check_")),
    ] {
        if root.join(&cand).exists() {
            found.push(format!("test file {}", cand));
        }
    }

    // A workflow that both NAMES the script and carries the vocabulary of a
    // planted fault. Named as a candidate: this is a keyword match, not a proof
    // that anything is asserted.
    if let Ok(rd) = std::fs::read_dir(root.join(".github/workflows")) {
        let mut hits: Vec<String> = rd
            .filter_map(|e| e.ok())
            .filter(|e| {
                let p = e.path();
                let is_yml = p.extension().is_some_and(|x| x == "yml" || x == "yaml");
                if !is_yml {
                    return false;
                }
                let body = std::fs::read_to_string(&p).unwrap_or_default();
                // T113: NEAR the invocation, and only the strong words.
                //
                // The first version asked whether the file contained the script
                // name anywhere and a planted-fault word anywhere. On its first
                // real use it labelled three uncontrolled gates as having a
                // control, on the strength of the word "must" sitting in a prose
                // comment 760 lines away from the call. That is the false green
                // this command exists to find, produced by the command.
                //
                // `broken` and `must` are dropped: both are ordinary English in
                // a workflow comment. `fixture`, `expect_` and `planted` are
                // vocabulary somebody chose on purpose.
                let lines: Vec<&str> = body.lines().collect();
                let calls: Vec<usize> = lines
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| l.contains(name))
                    .map(|(i, _)| i)
                    .collect();
                if calls.is_empty() {
                    return false;
                }
                lines.iter().enumerate().any(|(i, l)| {
                    ["fixture", "expect_", "planted"]
                        .iter()
                        .any(|w| l.contains(w))
                        && calls.iter().any(|c| i.abs_diff(*c) <= 30)
                })
            })
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        hits.sort();
        for h in hits {
            found.push(format!("workflow candidate {}", h));
        }
    }
    // The three test-file patterns collide when the stem does not begin with
    // `check_`, and a name printed twice reads as two independent controls.
    found.dedup();
    found
}

/// A measurement, keyed by what it depends on.
///
/// T127: the five-operator run passed twenty minutes and kept growing --
/// `gft_backprop_microcode.py` alone has 47 sites, each a ten-second subprocess.
/// A run nobody can finish is not a measurement, and the whole point of `--all`
/// was the full picture.
///
/// The result of mutating a gate depends on the gate's bytes and on the bytes of
/// whatever control judges it. Both are hashed; a row is reused only when both
/// match, and reused rows are MARKED. A cached green that read like a fresh one
/// would be the same lie this command exists to find.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct CachedRun {
    gate_sha: String,
    ctrl_sha: String,
    /// The bytes of the binary that produced this row.
    ///
    /// The key covered the SUBJECT and not the INSTRUMENT. Changing how sites
    /// are selected -- the whole point of iterating on an operator -- left
    /// every row valid, so a rebuilt `tri` served 24 rows measured by the
    /// version before the change and the table did not move. Two runs, one
    /// number, and the edit looked like it did nothing.
    ///
    /// R2: your own instrument is the first suspect. A cache that cannot see
    /// its own instrument change is the instrument lying about itself.
    #[serde(default)]
    tool_sha: String,
    killed: usize,
    total: usize,
    survivors: Vec<usize>,
}

/// A cache key over the bytes of `paths`.
///
/// Two hazards live in the obvious four-line version of this, and both make
/// DIFFERENT inputs share a key -- which is the direction that hurts, because
/// a shared key means a row measured against one input is served for another.
///
///   * An unreadable file hashed as `unwrap_or_default()` is the empty string,
///     so "the file is gone" and "the file is empty" are one key, and any two
///     unreadable files are one key.
///   * Concatenating contents with no separator lets the boundary move:
///     ["ab", "c"] and ["a", "bc"] hash identically. `judges` is a LIST, so
///     this is reachable whenever a gate carries more than one control.
///
/// Both are closed by hashing a length-prefixed record per file: the path, the
/// read status, and the bytes, each preceded by its length.
fn sha_of(paths: &[std::path::PathBuf]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    let mut field = |h: &mut Sha256, b: &[u8]| {
        h.update((b.len() as u64).to_le_bytes());
        h.update(b);
    };
    for p in paths {
        field(&mut h, p.to_string_lossy().as_bytes());
        match std::fs::read(p) {
            Ok(bytes) => {
                h.update([1u8]);
                field(&mut h, &bytes);
            }
            // Not "no bytes" -- an outcome of its own, and one that must not
            // collide with an empty file or with another unreadable path.
            Err(e) => {
                h.update([0u8]);
                field(&mut h, e.kind().to_string().as_bytes());
            }
        }
    }
    hex::encode(&h.finalize()[..8])
}

fn cache_path(root: &std::path::Path) -> std::path::PathBuf {
    root.join("target/.tri-mutate-cache.json")
}

/// T135: three silent failure paths lived in this function's six lines --
/// unreadable file, unparseable file, and a discarded write result. Each
/// degraded to "no data", which is indistinguishable from "nothing measured
/// yet", so a corrupt cache looked exactly like a first run.
///
/// Measured: a full run re-measured gates whose hashes matched entries already
/// in the file, and the entry count climbed 30 -> 40 -> 80 DURING that run --
/// it was rebuilding a cache it should have loaded. The cause is below.
fn load_cache(root: &std::path::Path) -> std::collections::HashMap<String, CachedRun> {
    let p = cache_path(root);
    if !p.exists() {
        return std::collections::HashMap::new();
    }
    match std::fs::read_to_string(&p) {
        Err(e) => {
            eprintln!(
                "warning: the cache at {} exists and could not be read ({e}).",
                p.display()
            );
            eprintln!("         Every row will be measured fresh.");
            std::collections::HashMap::new()
        }
        Ok(s) => match serde_json::from_str(&s) {
            Ok(m) => m,
            Err(e) => {
                // The likely cause, and it is worth naming rather than
                // shrugging at: the old writer truncated the file in place, so
                // a run killed mid-write left half a JSON document behind.
                eprintln!(
                    "warning: the cache at {} is unreadable JSON ({e}).",
                    p.display()
                );
                eprintln!("         A run killed mid-write can truncate it. Every row will be");
                eprintln!("         measured fresh, and this run rewrites the file atomically.");
                std::collections::HashMap::new()
            }
        },
    }
}

fn save_cache(root: &std::path::Path, c: &std::collections::HashMap<String, CachedRun>) {
    let p = cache_path(root);
    if let Some(d) = p.parent() {
        if let Err(e) = std::fs::create_dir_all(d) {
            eprintln!("warning: cannot create {} for the cache ({e})", d.display());
            return;
        }
    }
    let s = match serde_json::to_string_pretty(c) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("warning: the cache could not be serialised ({e})");
            return;
        }
    };
    // Write-then-rename. `fs::write` truncates in place, so a kill between the
    // truncate and the write leaves a partial document -- which the old reader
    // then swallowed as "no cache". A rename is atomic on the same filesystem:
    // the file is either the old complete one or the new complete one.
    let tmp = p.with_extension("json.tmp");
    if let Err(e) = std::fs::write(&tmp, s) {
        eprintln!("warning: cannot write the cache ({e}); this run will not be reusable");
        return;
    }
    if let Err(e) = std::fs::rename(&tmp, &p) {
        eprintln!("warning: cannot replace the cache ({e}); this run will not be reusable");
        let _ = std::fs::remove_file(&tmp);
    }
}

/// Open pull requests, with how much CI each one actually got.
fn prs(repo: Option<&str>) -> Result<()> {
    let mut base = vec![
        "pr",
        "list",
        "--state",
        "open",
        "--limit",
        "50",
        "--json",
        "number,title,mergeable",
    ];
    if let Some(r) = repo {
        base.push("--repo");
        base.push(r);
    }
    let out = Command::new("gh")
        .args(&base)
        .output()
        .context("gh pr list failed -- is the GitHub CLI installed and authenticated?")?;
    if !out.status.success() {
        anyhow::bail!(
            "gh pr list exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let list: serde_json::Value = serde_json::from_slice(&out.stdout).context("parse gh json")?;
    let items = list.as_array().cloned().unwrap_or_default();
    if items.is_empty() {
        println!("No open pull requests.");
        return Ok(());
    }

    println!(
        "{:<7} {:<13} {:>7}  {}",
        "pr", "mergeable", "checks", "title"
    );
    let mut blind: Vec<(i64, usize, String)> = Vec::new();
    let mut rows: Vec<(i64, String, usize)> = Vec::new();
    for it in &items {
        let n = it["number"].as_i64().unwrap_or(0);
        let m = it["mergeable"].as_str().unwrap_or("?").to_string();
        let title = it["title"].as_str().unwrap_or("");
        let mut cargs = vec![
            "pr".into(),
            "checks".into(),
            n.to_string(),
            "--json".into(),
            "name".into(),
        ];
        if let Some(r) = repo {
            cargs.push("--repo".into());
            cargs.push(r.to_string());
        }
        // `gh pr checks` exits non-zero when a check has failed, so the count is
        // read from stdout regardless of status -- an exit code here is about
        // the checks' colours, not about whether the listing worked.
        let c = Command::new("gh").args(&cargs).output();
        let count = c
            .ok()
            .and_then(|o| serde_json::from_slice::<serde_json::Value>(&o.stdout).ok())
            .and_then(|v| v.as_array().map(|a| a.len()))
            .unwrap_or(0);
        println!(
            "#{:<6} {:<13} {:>7}  {}",
            n,
            m,
            count,
            // By CHARS, not bytes. Slicing a String by byte index panics in the
            // middle of a multi-byte character, and the first title this
            // command ever printed contained an em dash -- from a pull request
            // this campaign opened.
            title.chars().take(46).collect::<String>()
        );
        rows.push((n, m, count));
    }

    // The reference is what a pull request in this repository normally gets.
    // Asserting from the CONFLICTING state alone was wrong: a conflict does not
    // retract runs that already happened, so a pull request can be conflicting
    // now and still carry a full list from when it was not.
    // T134: the median of EVERY pull request, not of the non-conflicting ones.
    // Filtering by state made the reference move with the state -- it read 21 on
    // one run and 35 on the next, with no pull request changed, because two rows
    // crossed between UNKNOWN and CONFLICTING in between. A median over all rows
    // is unmoved by a few short lists and does not depend on a value GitHub
    // recomputes while you are looking at it.
    let mut all: Vec<usize> = rows.iter().map(|(_, _, c)| *c).collect();
    all.sort_unstable();
    let reference = all.get(all.len() / 2).copied().unwrap_or(0);

    for (n, m, c) in &rows {
        // T134: flag by the COUNT, not by the state. `mergeable` is computed on
        // demand and reports UNKNOWN while GitHub is still working it out, so a
        // detector keyed on "CONFLICTING" finds a pull request one hour and
        // loses it the next -- measured: two with three checks each went from
        // CONFLICTING to UNKNOWN between two runs, and the alarm went silent
        // while nothing about them had changed.
        //
        // The short check list is the observable. The mergeable state is the
        // explanation for it, and belongs in the row rather than in the test.
        if reference > 0 && *c * 2 < reference {
            blind.push((*n, *c, m.clone()));
        }
    }

    println!();
    if reference == 0 {
        println!("No pull request has any checks, so there is no reference to compare against.");
        return Ok(());
    }
    println!(
        "Reference: the median open pull request here gets {} checks.",
        reference
    );
    if blind.is_empty() {
        println!("No pull request has a check list far below it.");
        return Ok(());
    }
    println!();
    println!(
        "{} pull request(s) with a check list far below the reference:",
        blind.len()
    );
    for (n, c, m) in &blind {
        println!(
            "  #{}  {} check(s) against a reference of {}   (mergeable: {})",
            n, c, reference, m
        );
    }
    println!();
    println!("A pull request that is conflicting when an event fires cannot have its merge");
    println!("diff computed, so every workflow with a `paths:` filter is skipped for that");
    println!("event. The checks that remain are the ones that never look at the diff -- and");
    println!("they are green, which reads exactly like a passing pull request.");
    println!();
    println!("A conflict does NOT retract earlier runs: a pull request that was mergeable");
    println!("when it was last pushed keeps that list. Rebase to get a real one.");
    println!();
    println!("`mergeable: UNKNOWN` means GitHub has not finished computing it -- not that");
    println!("the pull request is fine. A short list with UNKNOWN beside it is the same");
    println!("finding as one with CONFLICTING beside it, seen a moment earlier.");
    Ok(())
}

fn label(d: Direction) -> &'static str {
    match d {
        Direction::Silent => "silent",
        Direction::Loud => "loud",
        Direction::Invert => "invert",
        Direction::Boundary => "boundary",
        Direction::Assert => "assert",
    }
}

fn line_of(src: &str, byte: usize) -> usize {
    src[..byte].matches('\n').count() + 1
}

fn mutate(
    only: Option<&str>,
    loud: bool,
    invert: bool,
    boundary: bool,
    assert_op: bool,
    fresh: bool,
    all: bool,
    dir: Option<&str>,
) -> Result<()> {
    let (root, tools) = resolve_target(dir)?;
    // T126: a marker for an INTERRUPTED run. The loop writes a mutant, runs the
    // control, and restores; a kill lands between the first and the third and
    // leaves the tree mutated. The docstring says that is recoverable with
    // `git checkout tools/` -- true, and useless unless you know it happened.
    //
    // Measured, on myself: a ten-minute timeout killed an --all run, a boundary
    // mutant stayed in gft_backprop_microcode.py, and `git add -A` committed and
    // pushed it. The dirty-tree guard could not help: it refuses to START on a
    // dirty tree, and by then the damage was already staged.
    //
    // Under target/, which every Rust checkout already ignores, so the marker
    // itself can never be the dirt it warns about.
    let marker = root.join("target/.tri-mutating");
    if marker.exists() {
        let who = std::fs::read_to_string(&marker).unwrap_or_default();
        anyhow::bail!(
            "a previous `tri gates mutate` did not finish{}.\n\
             It may have left a mutant in the tree. Recover with:\n\
             \n    git -C {} checkout -- tools/\n    rm {}\n\
             \nThis marker exists because an interrupted run is silent otherwise: \
             the loop restores each file after its control, and a kill between \
             those two steps leaves the mutation in place.",
            if who.trim().is_empty() {
                String::new()
            } else {
                format!(" (it was on {})", who.trim())
            },
            root.display(),
            marker.display()
        );
    }
    if let Some(d) = marker.parent() {
        let _ = std::fs::create_dir_all(d);
    }

    // T128: BEFORE the dirty-tree check, not after. The marker exists for the
    // interrupted case, and in that case the tree IS dirty -- so the older,
    // less informative guard spoke first and the message naming the gate and
    // the recovery commands was never seen. Found by hitting a real interrupt
    // and watching the wrong error come out.
    let dirty = Command::new("git")
        .args(["status", "--porcelain", "--", "."])
        .current_dir(&tools)
        .output()
        .context("git status failed")?;
    if !String::from_utf8_lossy(&dirty.stdout).trim().is_empty() {
        anyhow::bail!(
            "tools/ has uncommitted changes. This command rewrites those files in \
             place and restores them; starting from a dirty tree means an \
             interrupted run cannot be told from your own edits. Commit or stash first."
        );
    }

    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&tools)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let n = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            // Both separators. The hyphen was not an oversight worth ignoring:
            // aimed at another repository with --dir, the underscore-only
            // filter silently found nothing and reported an empty table, which
            // reads exactly like a clean suite. Two naming conventions for one
            // thing is a shape this campaign has met before.
            n.ends_with(".py")
                && (n.starts_with("check_")
                    || n.starts_with("check-")
                    || n.contains("gate")
                    || is_gate_by_property(
                        &root,
                        n,
                        &std::fs::read_to_string(p).unwrap_or_default(),
                    ))
        })
        .collect();
    files.sort();

    // T111: nothing found is a FINDING, not a pass. Aimed at a directory with
    // no gate scripts, this printed its header, printed no rows, and exited 0 --
    // which reads exactly like a clean suite. The command whose whole subject is
    // a check that cannot fail had one.
    if files.is_empty() {
        println!(
            "FAIL: no gate scripts under {}\n\n  \
             Looked for *.py named check_* / check-* or containing \"gate\".\n  \
             Nothing was measured, which is not the same as nothing being wrong.",
            tools.display()
        );
        // A message is not a verdict. The first version of this block printed
        // FAIL and returned Ok, so the command announced that nothing had been
        // measured and exited 0 -- the vacuous pass, written into the fix for
        // the vacuous pass, in the command whose subject is vacuous passes.
        anyhow::bail!("no gate scripts under {}", tools.display());
    }

    let directions: &[Direction] = if all {
        &[
            Direction::Silent,
            Direction::Loud,
            Direction::Invert,
            Direction::Boundary,
            Direction::Assert,
        ]
    } else if assert_op {
        &[Direction::Assert]
    } else if boundary {
        &[Direction::Boundary]
    } else if invert {
        &[Direction::Invert]
    } else if loud {
        &[Direction::Loud]
    } else {
        &[Direction::Silent]
    };

    if all {
        println!(
            "{:<30}{:>8}{:>8}{:>8}{:>9}{:>8}  {}",
            "gate", "silent", "loud", "invert", "boundary", "assert", "verdict"
        );
        println!("(silent: `return 1..4` -> `return 0`  -- can the gate still FAIL?)");
        println!(
            "(loud:   `return 0`    -> `return 1`  -- does anything require it to be SILENT?)"
        );
        println!("(invert: `if C:` -> `if not (C):`     -- does it reach the RIGHT verdict?)");
        println!("(bound:  `>` <-> `>=`, `<` <-> `<=`    -- at the right PLACE?)");
        println!("(assert: `assert C` -> `assert True`   -- can the assertion still fail?)");
    } else {
        println!("{:<38} {:>9}  {}", "gate", "mutants", "verdict");
        if invert {
            println!("(invert: `if C:` -> `if not (C):` where the body carries a verdict.");
            println!(" A survivor means the gate can reach the WRONG verdict unnoticed.)");
        } else if loud {
            println!("(loud: `return 0` -> `return 1`. A survivor means NOTHING requires this gate to be silent.)");
        }
    }
    let mut total_survived: Vec<String> = Vec::new();
    // Equivalence claims that this run CONTRADICTS.
    //
    // `# mutant-equivalent: <why>` annotates a survivor as unkillable by
    // construction, and until now nothing ever checked one. Six sit in tools/.
    // A claim is a statement about the code, it ages with the code, and the
    // run best placed to notice it has gone stale is this one: it already
    // built the mutant and already knows the verdict. An unfalsifiable claim
    // is prose wearing the costume of an analysis.
    let mut claims_broken: Vec<String> = Vec::new();
    let mut claims_seen = 0usize;
    let mut cache = if fresh {
        std::collections::HashMap::new()
    } else {
        load_cache(&root)
    };
    let (mut n_cached, mut n_measured) = (0usize, 0usize);

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
        // Refuse to mutate rather than mutate unmarked. The marker is what
        // makes a killed run recoverable: without it on disk, an interrupted
        // run leaves a mutated gate behind and the next run has no way to know.
        // Swallowing this failure disarms the guard exactly when it is needed.
        std::fs::write(&marker, &name).map_err(|e| {
            anyhow::anyhow!(
                "cannot write the interrupt marker {}: {e}\n\
                 Refusing to mutate {name}: an interrupted run would leave the \
                 file mutated with nothing on disk to say so.",
                marker.display()
            )
        })?;
        let pristine = std::fs::read_to_string(f)?;
        // EVERY flag the gate declares, not the first one found. A gate can
        // carry several controls aimed at different branches, and running one
        // of them scores the branches the others cover as uncovered.
        // check_duplicate_agreement.py has two: --self-check-drop misses the
        // drift verdict at line 298 entirely, --self-check kills it. The first
        // version of this command took `.find()` and reported that line as a
        // survivor -- a finding invented by the tool, published before it was
        // checked. A mutant is killed if ANY declared control notices.
        let flags: Vec<String> = ["--self-check-drop", "--self-check", "--selftest"]
            .iter()
            .filter(|fl| pristine.contains(&format!("\"{}\"", fl)))
            .map(|s| s.to_string())
            .collect();
        let external = EXTERNAL_CONTROL
            .iter()
            .find(|(g, _)| *g == name)
            .map(|(_, c)| c.to_string());
        if flags.is_empty() && external.is_none() {
            println!("{:<38} {:>9}  {}", name, "-", "no control to run");
            continue;
        }

        // T91: the baseline this command did not take, found by an adversarial
        // reviewer of its own output rather than of the code it audits. A
        // mutant is scored killed when the control exits non-zero, so a
        // control that is red BEFORE any mutation is red after every one of
        // them, and this printed a perfect score. Reproduced, one variable,
        // with a `return 1` planted at the top of check_json_parses.py's
        // self_check:
        //
        //   old:  check_json_parses.py    1/1  all killed
        //   new:  check_json_parses.py      -  CONTROL ALREADY RED
        //
        // The exact inverse of the defect this command was written to find.
        // There a control that could not FAIL scored everything as covered;
        // here a control that cannot PASS does the same. Both replace a
        // measurement with a constant.
        //
        // The gate is NAMED in the survivor list rather than silently
        // credited: "nothing was measured here" is a finding, and a row that
        // quietly vanished would repeat the mistake one level up.
        crate::mutate::clear_derived_caches(f);
        let mut already_red: Vec<String> = Vec::new();
        for fl in &flags {
            if code(&root, &tools, &name, &[fl]) != "0" {
                already_red.push(fl.clone());
            }
        }
        if let Some(c) = &external {
            if code(&root, &tools, c, &[]) != "0" {
                already_red.push(c.clone());
            }
        }
        if !already_red.is_empty() {
            total_survived.push(name.clone());
            println!(
                "{:<38} {:>9}  CONTROL ALREADY RED: {} -- scored nothing",
                name,
                "-",
                already_red.join(", ")
            );
            continue;
        }

        // What this row depends on: the gate's bytes and its judges' bytes.
        let mut judges: Vec<std::path::PathBuf> = flags.iter().map(|_| f.clone()).take(1).collect();
        if let Some(c) = &external {
            judges.push(tools.join(c));
        }
        let gate_sha = sha_of(&[f.clone()]);
        let ctrl_sha = sha_of(&judges);
        let tool_sha = std::env::current_exe()
            .map(|p| sha_of(&[p]))
            .unwrap_or_default();

        let mut scores: Vec<(Direction, usize, usize, Vec<usize>)> = Vec::new();
        let (mut n_row_cached, mut n_row_fresh) = (0usize, 0usize);
        for dir in directions {
            let key = format!("{}|{}", name, label(*dir));
            if let Some(c) = cache.get(&key) {
                if c.gate_sha == gate_sha && c.ctrl_sha == ctrl_sha && c.tool_sha == tool_sha {
                    scores.push((*dir, c.killed, c.total, c.survivors.clone()));
                    n_row_cached += 1;
                    n_cached += 1;
                    continue;
                }
            }
            n_row_fresh += 1;
            n_measured += 1;
            let sites = sites_in_direction(&pristine, *dir);
            let mut killed = 0usize;
            let mut survivors: Vec<usize> = Vec::new();
            for (at, len, replacement) in &sites {
                let mut m = String::with_capacity(pristine.len());
                m.push_str(&pristine[..*at]);
                m.push_str(replacement);
                m.push_str(&pristine[at + len..]);
                std::fs::write(f, &m)?;
                // T92: a fifth defect, and the one that makes the numbers above
                // non-deterministic rather than merely incomplete. Python keys a
                // cached .pyc on (source mtime in whole seconds, source size).
                // `return 1` -> `return 0` preserves the size, and this loop writes
                // mutant, restore, next mutant well inside one second -- so an
                // IMPORTED gate can be served bytecode compiled from the previous
                // state. tools/wp18_selftest_gate.py does
                // `import wp18_conformance_gate as G`, and that .pyc is on disk.
                //
                // `tri mutate` already solved this and this command did not call
                // it. Found by an adversarial reviewer who went looking in the
                // sibling module rather than in the file under review.
                crate::mutate::clear_derived_caches(f);
                let mut noticed = false;
                for fl in &flags {
                    if code(&root, &tools, &name, &[fl]) != "0" {
                        noticed = true;
                        break;
                    }
                }
                if !noticed {
                    if let Some(c) = &external {
                        noticed = code(&root, &tools, c, &[]) != "0";
                    }
                }
                // Restore before judging, so an early return can never leave the
                // tree mutated. Clear again: the restore is the same
                // same-size-same-second write in the other direction, and a stale
                // mutant .pyc would poison the NEXT site's measurement.
                std::fs::write(f, &pristine)?;
                crate::mutate::clear_derived_caches(f);
                if !noticed {
                    survivors.push(line_of(&pristine, *at));
                } else {
                    killed += 1;
                }
            }
            debug_assert_eq!(std::fs::read_to_string(f).unwrap_or_default(), pristine);
            cache.insert(
                key,
                CachedRun {
                    gate_sha: gate_sha.clone(),
                    ctrl_sha: ctrl_sha.clone(),
                    tool_sha: tool_sha.clone(),
                    killed,
                    total: sites.len(),
                    survivors: survivors.clone(),
                },
            );
            save_cache(&root, &cache);
            scores.push((*dir, killed, sites.len(), survivors));
        }

        // A gate with no sites in a direction has nothing to say about it. In
        // single-operator mode that is the whole row; in --all it is one column,
        // and it must not read as a clean score -- "0/0" is printed, never a
        // blank or a dash that a reader could take for a pass.
        if scores.iter().all(|(_, _, t, _)| *t == 0) {
            let what = match directions {
                [Direction::Loud] => "no success path to break",
                [Direction::Invert] => "no verdict-bearing condition to invert",
                [Direction::Boundary] => "no comparison to move",
                [Direction::Assert] => "no assertion to neuter",
                [Direction::Silent] => "no failure path to break",
                _ => "no mutable site in any direction",
            };
            // T127: the THIRD print path. A zero-site row is still a row, and a
            // cached one printed with no marker -- so the property went into two
            // of three branches, which is how it got into one of two the first
            // time.
            println!(
                "{:<38} {:>9}  {}{}",
                name,
                0,
                what,
                if n_row_fresh == 0 && n_row_cached > 0 {
                    " [cached]"
                } else {
                    ""
                }
            );
            continue;
        }

        // T107: a survivor whose line CLAIMS to be a functional equivalence.
        // The claim is PRINTED, never acted on: the row still reads SURVIVED
        // and still counts, because suppressing a row on the strength of a
        // comment is how a declared UNCOVERED stood for a week while being
        // false. The marker's job is to stop the next reader re-deriving a
        // proof that is already written beside the code.
        //
        // The marker applies to the first CODE line after it, not to a fixed
        // offset. The first version of this took marker+2 -- the proof it was
        // written for is a fifteen-line comment block, so it named a line in
        // the middle of its own explanation. A ruler measured against one
        // example, in the file whose subject is rulers measured against one
        // example.
        let equiv_lines = equivalence_claims(&pristine);

        // Contradict the claims. A claimed line whose mutant DIED is a false
        // statement sitting in the source, and it reads as settled analysis to
        // everyone after it.
        //
        // Claims are NOT operator-scoped -- the marker names no direction, and
        // every one in the tree today argues about a comparison ("so >= is >").
        // So a line legitimately equivalent under `boundary` may well die under
        // `invert`, and this names the direction rather than pretending to
        // judge. That ambiguity is in the marker's design, not in the check;
        // reporting it is what makes it visible enough to fix.
        claims_seen += equiv_lines.len();
        for (dir, _, _, survivors) in &scores {
            let site_lines: Vec<usize> = sites_in_direction(&pristine, *dir)
                .into_iter()
                .map(|(at, _, _)| line_of(&pristine, at))
                .collect();
            claims_broken.extend(contradicted_claims(
                &name,
                label(*dir),
                &site_lines,
                survivors,
                &equiv_lines,
            ));
        }

        let mut survived_here: Vec<String> = Vec::new();
        for (dir, _, _, survivors) in &scores {
            if !survivors.is_empty() {
                // For boundary, the line number alone is unreadable: a reader
                // cannot tell `while len(v) < N` -- where a survivor is the
                // correct answer -- from a comparison that decides an exit
                // code. Printing the source makes the two populations visible
                // without opening the file, which is the only honest way to
                // present a column whose denominator mixes them (the filter
                // that tried to separate them removed proven kills).
                let src_lines: Vec<&str> = pristine.lines().collect();
                let with_src = matches!(dir, Direction::Boundary);
                let shown: Vec<String> = survivors
                    .iter()
                    .map(|l| {
                        let base = match equiv_lines.get(l) {
                            Some(why) => format!("{} (claims equivalent: {})", l, why),
                            None => l.to_string(),
                        };
                        if !with_src {
                            return base;
                        }
                        match src_lines.get(l.saturating_sub(1)) {
                            Some(text) => {
                                let t = text.trim();
                                let t: String = t.chars().take(46).collect();
                                format!("{base}  `{t}`")
                            }
                            None => base,
                        }
                    })
                    .collect();
                survived_here.push(format!(
                    "{} line{} {}",
                    label(*dir),
                    if survivors.len() == 1 { "" } else { "s" },
                    shown.join(", ")
                ));
            }
        }
        if !survived_here.is_empty() {
            total_survived.push(name.clone());
        }

        if directions.len() == 1 {
            let (_, killed, total, _) = &scores[0];
            let verdict = {
                let v = if survived_here.is_empty() {
                    "all killed".to_string()
                } else {
                    format!("SURVIVED at {}", survived_here[0])
                };
                // T127: the marker belongs in BOTH shapes. It was added to the
                // multi-column branch only, so a cached single-operator row
                // printed exactly like a fresh one -- the failure this marker
                // exists to prevent, in the half of the code that prints it.
                match (n_row_cached, n_row_fresh) {
                    (0, _) => v,
                    (_, 0) => format!("{} [cached]", v),
                    (c, f) => format!("{} [{} cached, {} fresh]", v, c, f),
                }
            };
            println!(
                "{:<38} {:>9}  {}",
                name,
                format!("{}/{}", killed, total),
                verdict
            );
        } else {
            let cols: String = scores
                .iter()
                .map(|(_, k, t, _)| format!("{:>8}", format!("{}/{}", k, t)))
                .collect();
            println!("{:<30}{}  {}", name, cols, {
                let v = if survived_here.is_empty() {
                    "all killed".to_string()
                } else {
                    format!("SURVIVED: {}", survived_here.join("; "))
                };
                // T127: a reused row says so. A cached green that read like
                // a fresh one would be the same lie this command exists to
                // find.
                // T130: per-ROW precision. A row with two columns measured
                // and three reused was labelled `[cached]` wholesale --
                // under-claiming rather than over-claiming, so the safe
                // direction, and still wrong. The point of the marker is
                // that a reader can tell which it is.
                match (n_row_cached, n_row_fresh) {
                    (0, _) => v,
                    (_, 0) => format!("{} [cached]", v),
                    (c, f) => format!("{} [{} cached, {} fresh]", v, c, f),
                }
            });
        }
    }

    let _ = std::fs::remove_file(&marker);
    save_cache(&root, &cache);
    if n_cached > 0 {
        println!();
        println!(
            "{} row(s) MEASURED, {} reused from cache (gate and control bytes unchanged).",
            n_measured, n_cached
        );
        println!("A cached row is a measurement from an earlier run, not from this one.");
        println!("`--fresh` re-measures everything. The cache lives in target/ and is");
        println!("keyed on the gate's bytes and its control's -- a fixture changing");
        println!("underneath both is a way for a reused row to be stale, and is why");
        println!("the marker exists rather than being silently omitted.");
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

    println!();
    if claims_seen == 0 {
        println!("No `# mutant-equivalent:` claim was in scope for this run.");
    } else if claims_broken.is_empty() {
        println!(
            "{} equivalence claim(s) in scope, none contradicted.",
            claims_seen
        );
        println!("Each says its mutant cannot die, and each mutant survived. That is");
        println!("the whole check -- a claim about the FUTURE of the code is worth");
        println!("only the run that could have refuted it and did not.");
    } else {
        println!(
            "{} of {} equivalence claim(s) CONTRADICTED:",
            claims_broken.len(),
            claims_seen
        );
        for c in &claims_broken {
            println!("  {}", c);
        }
        println!();
        println!("A `# mutant-equivalent:` comment says the mutant at that line cannot");
        println!("die. These died. Either the reasoning was wrong when it was written,");
        println!("or the code moved out from under it -- and both read as settled");
        println!("analysis to everyone who comes after.");
        println!();
        println!("Claims name no operator, and every one in the tree argues about a");
        println!("comparison. A line equivalent under `boundary` can be killable under");
        println!("`invert`, so read the direction named above before believing this.");
    }

    // The boundary column's denominator holds two populations, and only one of
    // them is about gates.
    //
    // The other four operators mutate something that reaches a verdict by
    // construction: a return of a verdict literal, a SystemExit, an assert, or
    // a condition whose body carries one. `boundary` moves EVERY comparison,
    // including `while len(v) < N` and `if len(out) > 6` -- loop bounds and
    // display cutoffs, where a survivor is the correct answer and says nothing
    // about the gate.
    //
    // Filtering them out was tried and is wrong: the filter removed sites whose
    // mutants were being KILLED (6/6 in check_vector_data, 3/3 in
    // check_seal_coverage, 1/1 in check_catalog_integrity), and a kill is proof
    // that the comparison reaches a verdict. Verdict-reachability is a dataflow
    // property -- `if x > t: problems.append(...)` decides an exit code several
    // statements later -- and no line-local pattern decides it.
    //
    // So the killed count is a LOWER BOUND on the verdict-bearing population,
    // established after the fact, which is the only way it can be established.
    // A ratio over a mixed denominator is not a rate, and printing one invites
    // exactly the conclusion this file exists to prevent.
    if directions.contains(&Direction::Boundary) {
        println!();
        println!("On the boundary column: its denominator counts EVERY comparison,");
        println!("including loop bounds and display cutoffs, where a survivor is the");
        println!("right answer. The killed count is a lower bound on the comparisons");
        println!("that reach a verdict -- proven by the kill itself. Do not read");
        println!("killed/total there as a rate; the denominator is two populations.");
    }
    Ok(())
}

fn code(root: &std::path::Path, tools: &std::path::Path, script: &str, args: &[&String]) -> String {
    let mut c = Command::new("python3");
    // Absolute, so the script found is the one in `tools` rather than whatever
    // sits under a directory of that name below cwd. With no --dir this is the
    // same file the old relative form resolved to.
    c.arg(tools.join(script));
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
        GatesCmd::Sweep { controls_only, dir } => sweep(*controls_only, dir.as_deref()),
        GatesCmd::Mutate {
            only,
            loud,
            invert,
            boundary,
            assert_op,
            fresh,
            all,
            dir,
        } => mutate(
            only.as_deref(),
            *loud,
            *invert,
            *boundary,
            *assert_op,
            *fresh,
            *all,
            dir.as_deref(),
        ),
        GatesCmd::Prs { repo } => prs(repo.as_deref()),
        GatesCmd::Unmeasured { repos, stale_days } => {
            let list: Vec<String> = if repos.is_empty() {
                vec![current_repo()?]
            } else {
                repos.clone()
            };
            unmeasured(&list, *stale_days)
        }
        GatesCmd::Required { repo } => required(repo.as_deref()),
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

/// The repository this working tree belongs to, as `owner/name`.
fn current_repo() -> Result<String> {
    let s = gh(&[
        "repo",
        "view",
        "--json",
        "nameWithOwner",
        "--jq",
        ".nameWithOwner",
    ])?;
    let s = s.trim().to_string();
    if s.is_empty() {
        anyhow::bail!(
            "`gh repo view` named no repository -- run this inside a checkout, or pass --repo"
        );
    }
    Ok(s)
}

/// Does this workflow restrict itself with `paths:`? A path filter is what
/// turns "has not failed" into "has not run", so it is reported beside the
/// staleness rather than left for the reader to go and look up.
fn has_path_filter(root: &std::path::Path, rel: &str) -> bool {
    let p = root.join(rel);
    match std::fs::read_to_string(&p) {
        Ok(t) => t.lines().any(|l| {
            let t = l.trim_start();
            t.starts_with("paths:") || t.starts_with("paths-ignore:")
        }),
        Err(_) => false,
    }
}

/// Can this workflow ever produce a default-branch run WITHOUT a human?
///
/// `unmeasured` above reports staleness: how long since the last default-branch
/// run. That is the right question and it has a blind spot, which cost a
/// measurement on 2026-08-30. `emit-bitexact-gate.yml` has no `push:` trigger at
/// all, so it produces no automatic default-branch history ever -- but it had
/// been DISPATCHED by hand two days earlier, so it was not stale, so it never
/// appeared in that table, and its absence read as health.
///
/// Staleness and reachability are different facts. A workflow can be fresh and
/// still be structurally incapable of producing the next reading on its own.
///
/// Parsed by line rather than by a YAML crate, matching `has_path_filter` and
/// `has_dispatch` above -- and with the two traps that reading takes:
///   * a `push:` inside a COMMENT is not a trigger. This repository has a
///     workflow whose header comment recommends `push: branches: [master]` in
///     prose, and a naive scan reads its own advice as compliance.
///   * `push:` nested under something else (a job `if:`, a `paths:` list) is not
///     the trigger either, so only the first indent level inside `on:` counts.
fn has_auto_default_run(text: &str, default_branch: &str) -> bool {
    let mut in_on = false;
    let mut on_indent = usize::MAX;
    let mut push_indent = usize::MAX;
    let mut in_push = false;
    let mut saw_push = false;
    let mut branches: Option<Vec<String>> = None;

    for raw in text.lines() {
        let trimmed = raw.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = raw.len() - trimmed.len();

        if !in_on {
            if indent == 0 && (trimmed.starts_with("on:") || trimmed.starts_with("\"on\":")) {
                // Inline form: `on: [push, pull_request]` or `on: push`.
                let rest = trimmed.splitn(2, ':').nth(1).unwrap_or("").trim();
                if !rest.is_empty() {
                    return rest.contains("push");
                }
                in_on = true;
            }
            continue;
        }

        // A column-0 key ends the `on:` block.
        if indent == 0 {
            break;
        }
        if on_indent == usize::MAX {
            on_indent = indent;
        }

        if indent == on_indent {
            in_push = trimmed.starts_with("push:");
            if in_push {
                saw_push = true;
                push_indent = indent;
                let rest = trimmed.splitn(2, ':').nth(1).unwrap_or("").trim();
                if !rest.is_empty() {
                    branches = Some(parse_branch_list(rest));
                }
            }
            continue;
        }

        if in_push && indent > push_indent && trimmed.starts_with("branches:") {
            let rest = trimmed.splitn(2, ':').nth(1).unwrap_or("").trim();
            branches = Some(parse_branch_list(rest));
        }
        // `branches:` written as a block list under `push:`.
        if in_push && branches.is_some() && trimmed.starts_with("- ") {
            if let Some(b) = branches.as_mut() {
                b.push(trimmed[2..].trim().trim_matches('"').trim_matches('\'').to_string());
            }
        }
    }

    if !saw_push {
        return false;
    }
    match branches {
        // `push:` with nothing under it fires on every branch.
        None => true,
        Some(list) if list.is_empty() => true,
        Some(list) => list.iter().any(|b| branch_pattern_matches(b, default_branch)),
    }
}

/// GitHub branch filters are PATTERNS, not names.
///
/// An equality test reads `branches: ['ma*']` as "does not cover master" and
/// reports a workflow structurally unmeasurable when it runs on every push. The
/// list in this repository happens to contain no such pattern today -- an
/// independent yaml+fnmatch reader agrees with this function on all 49 files --
/// so the equality version would have been correct here and wrong in principle,
/// which is the worst kind of correct.
///
/// The subset implemented is GitHub's: `*` matches within one path segment,
/// `**` matches across segments, and everything else is literal. `?`, `+` and
/// character classes are NOT implemented and are treated as literals, which
/// errs toward reporting a workflow as uncovered -- a false entry in a work
/// list rather than a silence.
fn branch_pattern_matches(pattern: &str, branch: &str) -> bool {
    fn walk(p: &[u8], b: &[u8]) -> bool {
        if p.is_empty() {
            return b.is_empty();
        }
        if p[0] == b'*' {
            let double = p.len() > 1 && p[1] == b'*';
            let rest = if double { &p[2..] } else { &p[1..] };
            // `*` stops at a `/`; `**` crosses them.
            let mut i = 0;
            loop {
                if walk(rest, &b[i..]) {
                    return true;
                }
                if i >= b.len() {
                    return false;
                }
                if !double && b[i] == b'/' {
                    return false;
                }
                i += 1;
            }
        }
        !b.is_empty() && p[0] == b[0] && walk(&p[1..], &b[1..])
    }
    walk(pattern.as_bytes(), branch.as_bytes())
}

/// `[master, main]`, `[ "master" ]`, or an empty string for a block list below.
fn parse_branch_list(rest: &str) -> Vec<String> {
    let inner = rest.trim().trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Can a human get the missing reading at all? Without `workflow_dispatch:`
/// there is no way to fire it against the default branch on purpose, so the
/// gap cannot be closed even by someone who wants to.
fn has_dispatch(root: &std::path::Path, rel: &str) -> bool {
    match std::fs::read_to_string(root.join(rel)) {
        Ok(t) => t
            .lines()
            .any(|l| l.trim_start().starts_with("workflow_dispatch:")),
        Err(_) => false,
    }
}

fn unmeasured(repos: &[String], stale_days: u64) -> Result<()> {
    let root = repo_root()?;
    let mut no_auto: Vec<(String, String, bool, String)> = Vec::new();
    // Named once, so the printed sentence quotes the branch actually queried
    // rather than the word "master" hardcoded into a message.
    let mut default_branch_seen = String::new();
    let mut rows: Vec<(String, String, String, bool, bool)> = Vec::new();
    let mut checked = 0usize;
    let mut unreadable = 0usize;
    let mut ghosts: Vec<(String, String, String)> = Vec::new();

    for repo in repos {
        let default_branch = gh(&["api", &format!("repos/{repo}"), "--jq", ".default_branch"])?
            .trim()
            .to_string();

        default_branch_seen = default_branch.clone();
        let listing = gh(&[
            "api",
            &format!("repos/{repo}/actions/workflows?per_page=100"),
            "--jq",
            r#".workflows[]|select(.state=="active")|"\(.id)\t\(.name)\t\(.path)""#,
        ])?;

        for line in listing.lines() {
            let mut it = line.splitn(3, '\t');
            let (id, name, path) = match (it.next(), it.next(), it.next()) {
                (Some(a), Some(b), Some(c)) => (a, b, c),
                _ => continue,
            };
            checked += 1;

            // A workflow GitHub calls active whose file is not in the tree is a
            // THIRD thing, and calling it unmeasured is wrong twice over: it can
            // never run, so it will never be measured, and it inflates the count of
            // gates someone might go and fix. Thirteen of the fifty-nine here are
            // that -- deleted from the repository, still registered with Actions,
            // still listed as active. Found by dispatching all 27 unmeasured
            // workflows and having 12 refuse.
            if !root.join(path).is_file() {
                ghosts.push((repo.clone(), name.to_string(), path.to_string()));
                continue;
            }

            // The most recent run ON THE DEFAULT BRANCH. A run on a pull
            // request says nothing about the branch everything merges into.
            // The jq here was once written as a raw string with the closing
            // `""` trimmed off at runtime, which produced `... // ` -- invalid
            // jq. Every query then failed, `unwrap_or_default` turned each
            // failure into an empty string, and the command reported that all
            // 58 workflows had never run. A confident wrong answer, produced by
            // the exact mechanism this command exists to find.
            //
            // So: no default on error. A query that did not run is not a
            // workflow that did not run, and the two are now said differently.
            let query = format!(
                "repos/{repo}/actions/workflows/{id}/runs?branch={default_branch}&per_page=1"
            );
            let last = match gh(&[
                "api",
                &query,
                "--jq",
                ".workflow_runs[0].created_at // \"\"",
            ]) {
                Ok(v) => v.trim().to_string(),
                Err(e) => {
                    eprintln!("  ?  could not ask about {name}: {e}");
                    unreadable += 1;
                    continue;
                }
            };

            let stale = if last.is_empty() {
                true
            } else {
                match days_since(&last) {
                    Some(d) => d > stale_days,
                    // An unparseable date is not a fresh one. Saying "fine"
                    // here is the same defect this command exists to find.
                    None => true,
                }
            };
            // Reachability, not staleness: asked of EVERY workflow with a file,
            // including the ones that are perfectly fresh. A workflow that was
            // dispatched by hand yesterday is not stale and still cannot take
            // tomorrow's reading by itself.
            if let Ok(text) = std::fs::read_to_string(root.join(path)) {
                if !has_auto_default_run(&text, &default_branch) {
                    no_auto.push((
                        repo.clone(),
                        name.to_string(),
                        has_dispatch(&root, path),
                        if last.is_empty() {
                            "never".to_string()
                        } else {
                            last[..10].to_string()
                        },
                    ));
                }
            }

            if stale {
                rows.push((
                    repo.clone(),
                    name.to_string(),
                    if last.is_empty() {
                        "never".into()
                    } else {
                        last[..10].to_string()
                    },
                    has_path_filter(&root, path),
                    has_dispatch(&root, path),
                ));
            }
        }
    }

    if !ghosts.is_empty() {
        println!(
            "{} workflow(s) are registered and active but have NO FILE in the tree.\n\
             They cannot run, so they can never be measured -- delete the workflow in\n\
             Actions, or restore the file:\n",
            ghosts.len()
        );
        for (repo, name, path) in &ghosts {
            println!("  {path}  ({name}, {repo})");
        }
        println!();
    }
    if unreadable > 0 {
        println!(
            "  {unreadable} workflow(s) could not be asked about; they are NOT counted as \
             either fresh or stale."
        );
    }
    if no_auto.is_empty() {
        println!(
            "  Workflows with no automatic default-branch trigger: 0 \
             (every one can produce its own baseline)."
        );
    } else {
        no_auto.sort_by(|a, b| a.1.cmp(&b.1));
        println!(
            "{} workflow(s) can NEVER produce a default-branch run on their own -- no\n\
             `push:` covering `{}`. Their default-branch history exists only where a\n\
             human dispatched them, so \"is it red on the default branch too?\" has no\n\
             standing answer when one of them goes red on a pull request.\n",
            no_auto.len(),
            if default_branch_seen.is_empty() { "the default branch" } else { default_branch_seen.as_str() }
        );
        println!("  {:<10}  {:<9}  {}", "LAST", "dispatch", "WORKFLOW");
        for (repo, name, dispatch, last) in &no_auto {
            println!(
                "  {:<10}  {:<9}  {}  ({})",
                last,
                if *dispatch { "yes" } else { "NO" },
                name,
                repo
            );
        }
        println!(
            "\n  `dispatch: yes` means the missing reading can be TAKEN -- fire it at the\n\
               default branch rather than inferring the baseline from sibling branches.\n\
               `LAST` here is a lifetime per-workflow query, not a window over recent\n\
               runs: reading a window and reporting a lifetime is how this section came\n\
               to exist.\n"
        );
    }

    if rows.is_empty() {
        println!(
            "Every active workflow has run on the default branch within {stale_days} days \
             ({checked} checked)."
        );
        return Ok(());
    }

    rows.sort_by(|a, b| a.2.cmp(&b.2).then(a.1.cmp(&b.1)));
    println!(
        "{} of {} active workflow(s) with a file have no default-branch run within {} days.\n",
        rows.len(),
        checked - ghosts.len(),
        stale_days
    );
    println!(
        "  {:<10}  {:<7}  {:<9}  {}",
        "LAST", "paths:", "dispatch", "WORKFLOW"
    );
    for (repo, name, last, filtered, dispatch) in &rows {
        println!(
            "  {:<10}  {:<7}  {:<9}  {}  ({})",
            last,
            if *filtered { "yes" } else { "-" },
            if *dispatch { "yes" } else { "NO" },
            name,
            repo
        );
    }
    println!(
        "\n  A gate that has not run on the default branch is not passing there; it is\n\
           unmeasured. `paths: yes` is usually the reason. `dispatch: NO` means the\n\
           reading cannot be taken on purpose -- add `workflow_dispatch:` first."
    );
    Ok(())
}

/// Whole days between an ISO-8601 timestamp and now, or None if it will not
/// parse. Kept separate so the staleness rule can be tested without a network.
fn days_since(iso: &str) -> Option<u64> {
    let ts = chrono::DateTime::parse_from_rfc3339(iso).ok()?;
    let now = chrono::Utc::now();
    let secs = now
        .signed_duration_since(ts.with_timezone(&chrono::Utc))
        .num_seconds();
    if secs < 0 {
        return Some(0);
    }
    Some(secs as u64 / 86_400)
}

/// The job contexts a workflow file emits: each job's `name:` if set, else its id.
///
/// GitHub matches a required check by CONTEXT, and a job's context is its display
/// name when one is given. Matching by file name instead would report every
/// workflow as unrequired.
pub fn contexts_of(yaml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Some(pos) = yaml.find("\njobs:") else {
        return out;
    };
    let body = &yaml[pos + 6..];
    let mut cur: Option<String> = None;
    let mut named = false;
    for line in body.lines() {
        if line.starts_with("  ") && !line.starts_with("   ") && line.trim_end().ends_with(':') {
            if let Some(id) = cur.take() {
                if !named {
                    out.push(id);
                }
            }
            let id = line.trim().trim_end_matches(':').to_string();
            named = false;
            cur = Some(id);
            continue;
        }
        if cur.is_some() && !named {
            if let Some(rest) = line.strip_prefix("    name:") {
                out.push(rest.trim().trim_matches('"').trim_matches('\'').to_string());
                named = true;
            }
        }
        if !line.starts_with(' ') && !line.trim().is_empty() {
            break;
        }
    }
    if let Some(id) = cur {
        if !named {
            out.push(id);
        }
    }
    out
}

/// Workflow files the tree claims are required, with where the claim is written.
fn claims(root: &std::path::Path) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let doc = root.join("docs/BRANCH-PROTECTION.md");
    if let Ok(t) = std::fs::read_to_string(&doc) {
        for line in t.lines() {
            if let Some(i) = line.find(".github/workflows/") {
                let rest = &line[i + 18..];
                let f: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
                    .collect();
                if f.ends_with(".yml") {
                    out.push((f, "docs/BRANCH-PROTECTION.md".into()));
                }
            }
        }
    }
    let script = root.join("scripts/ci/check_pr_branch_filters.py");
    if let Ok(t) = std::fs::read_to_string(&script) {
        // The DEFINITION, not the first mention. `MERGE_CRITICAL` appears in this
        // script's own docstring five lines above the assignment, and anchoring on
        // the bare name matched the prose: the segment that followed held no
        // quoted filenames at all, so fifteen claims read as zero.
        if let Some(i) = t.find("MERGE_CRITICAL = (") {
            let seg = &t[i..];
            let end = seg.find(")\n").unwrap_or(seg.len());
            for m in seg[..end].split('"').skip(1).step_by(2) {
                if m.ends_with(".yml") {
                    out.push((m.to_string(), "check_pr_branch_filters.py MERGE_CRITICAL".into()));
                }
            }
        }
    }
    out.sort();
    out.dedup();
    // One row per workflow, listing every place the claim is written. Two rows for
    // the same file read as two workflows, and the count of hollow gates is the
    // number that matters.
    let mut grouped: Vec<(String, String)> = Vec::new();
    for (f, w) in out {
        match grouped.last_mut() {
            Some((lf, lw)) if *lf == f => {
                lw.push_str(" and ");
                lw.push_str(&w);
            }
            _ => grouped.push((f, w)),
        }
    }
    grouped
}

fn required(repo: Option<&str>) -> Result<()> {
    let root = {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()?;
        if !out.status.success() {
            anyhow::bail!("not inside a git repository");
        }
        std::path::PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    let slug = match repo {
        Some(r) => r.to_string(),
        None => {
            let out = std::process::Command::new("git")
                .args(["remote", "get-url", "origin"])
                .output()?;
            let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let s = url.trim_end_matches(".git");
            let tail = s.rsplit_once(':').map(|(_, t)| t).unwrap_or(s);
            let parts: Vec<&str> = tail.trim_start_matches('/').rsplit('/').take(2).collect();
            if parts.len() != 2 {
                anyhow::bail!("cannot read owner/name from origin url `{url}`");
            }
            format!("{}/{}", parts[1], parts[0])
        }
    };

    let listing = gh(&[
        "api",
        &format!("repos/{slug}/rules/branches/master"),
        "--jq",
        r#".[]|select(.type=="required_status_checks")|.parameters.required_status_checks[].context"#,
    ])?;
    let req: Vec<String> = listing.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
    if req.is_empty() {
        anyhow::bail!(
            "no required contexts came back for {slug}. That is either a branch with \
             none, or a token that cannot read rules -- and those are different facts, \
             so this refuses rather than calling every claim in the tree false."
        );
    }

    let claimed = claims(&root);
    if claimed.is_empty() {
        anyhow::bail!(
            "no workflow file is claimed as required anywhere in the tree. Either \
             docs/BRANCH-PROTECTION.md and MERGE_CRITICAL changed shape, or the claims \
             are gone -- a clean report here would be this parser."
        );
    }

    println!("CLAIMED REQUIRED vs ACTUALLY REQUIRED   ({slug}, branch master)");
    println!();
    println!("  ruleset requires {} context(s):", req.len());
    for c in &req {
        println!("    {c}");
    }
    println!();

    let mut satisfied: Vec<String> = Vec::new();
    let mut hollow: Vec<(String, String, Vec<String>)> = Vec::new();
    for (file, where_) in &claimed {
        let path = root.join(".github/workflows").join(file);
        let Ok(y) = std::fs::read_to_string(&path) else {
            hollow.push((file.clone(), format!("{where_} (no such file)"), vec![]));
            continue;
        };
        let ctx = contexts_of(&y);
        if ctx.iter().any(|c| req.contains(c)) {
            satisfied.extend(ctx.iter().filter(|c| req.contains(c)).cloned());
        } else {
            hollow.push((file.clone(), where_.clone(), ctx));
        }
    }
    satisfied.sort();
    satisfied.dedup();

    if hollow.is_empty() {
        println!("  Every workflow the tree calls required emits a required context.");
    } else {
        println!("  Claimed required, emits no required context -- cannot block a merge:");
        for (file, where_, ctx) in &hollow {
            println!(
                "    {:<34} {}",
                file,
                if ctx.is_empty() {
                    "(no jobs read)".to_string()
                } else {
                    format!("emits: {}", ctx.join(", "))
                }
            );
            println!("      claimed in {where_}");
        }
    }

    let unclaimed: Vec<&String> = req.iter().filter(|c| !satisfied.contains(c)).collect();
    if !unclaimed.is_empty() {
        println!();
        println!("  Required by the ruleset and claimed by nothing in the tree:");
        for c in &unclaimed {
            println!("    {c}");
        }
        println!("  A check that blocks every merge and that no document mentions is as");
        println!("  hard to reason about as one that claims to block and does not.");
    }
    println!();
    println!(
        "  {} claim(s), {} of them hollow; {} required context(s), {} unclaimed.",
        claimed.len(),
        hollow.len(),
        req.len(),
        unclaimed.len()
    );
    println!();
    println!(
        "A required check is named in repository SETTINGS. No file in the tree can read\n\
         it, so a comment claiming a gate blocks cannot go stale against anything --\n\
         this is the only drift here with no detector, which is why it is a command."
    );
    Ok(())
}

/// GitHub keeps a workflow registered as `active` after its file is deleted, so
/// `state=="active"` is the API's word and not the repository's: 61 registrations
/// here against 48 files, and 13 of the registrations have nothing left to fix.
/// A deleted workflow with a zero success count is history, not a dead gate, and
/// listing the two together makes the fixable ones harder to see.
fn workflow_file_present(path: &str) -> bool {
    std::path::Path::new(path).is_file()
}

/// Where a never-succeeded workflow belongs.
#[derive(Debug, PartialEq, Eq)]
pub enum Bucket {
    /// Has a file and enough runs to judge: a dead gate, reported.
    Reported,
    /// Has a file but fewer runs than the floor: named separately, never dropped.
    Suppressed,
    /// No file in the tree: history for a workflow that no longer exists.
    Deleted,
}

/// The three-way split, without the network.
///
/// Deleted is decided FIRST. A registration with no file cannot be under- or
/// over-run: there is nothing to fix either way, and letting the run count decide
/// its bucket would file a phantom under whichever threshold it happened to meet.
pub fn classify(on_disk: bool, total: u64, min_runs: u64) -> Bucket {
    if !on_disk {
        Bucket::Deleted
    } else if too_few_runs_to_judge(total, min_runs) {
        Bucket::Suppressed
    } else {
        Bucket::Reported
    }
}

fn dead(repos: &[String], min_runs: u64) -> Result<()> {
    let mut rows: Vec<(String, String, u64)> = Vec::new();
    let mut deleted: Vec<(String, String, u64)> = Vec::new();
    // Every workflow the threshold hid, so a bounded report never reads as a
    // complete one. `brain-seal-refresh.yml` fails structurally -- its last step
    // is a `git push` this repository's own ruleset rejects -- and has 8 lifetime
    // runs, so the shipped floor of 50 suppresses it entirely.
    let mut suppressed: Vec<(String, String, u64)> = Vec::new();
    let single_repo = repos.len() == 1;
    for repo in repos {
        let listing = gh(&[
            "api",
            &format!("repos/{repo}/actions/workflows?per_page=100"),
            "--jq",
            r#".workflows[]|select(.state=="active")|"\(.id)\t\(.path)\t\(.name)""#,
        ])?;
        for line in listing.lines() {
            let mut it = line.splitn(3, '\t');
            let (id, path, name) = match (it.next(), it.next(), it.next()) {
                (Some(a), Some(b), Some(c)) => (a, b, c),
                _ => continue,
            };
            let total = count(repo, id, false)?;
            let never = count(repo, id, true)? == 0;
            if !never {
                continue;
            }
            // The path check only means anything for the repository we are standing
            // in; for any other repo in the list, take the API at its word.
            let on_disk = !single_repo || workflow_file_present(path);
            let row = (repo.clone(), name.to_string(), total);
            match classify(on_disk, total, min_runs) {
                Bucket::Deleted => deleted.push(row),
                Bucket::Suppressed => suppressed.push(row),
                Bucket::Reported => rows.push(row),
            }
        }
    }
    deleted.sort_by(|a, b| b.2.cmp(&a.2));
    suppressed.sort_by(|a, b| b.2.cmp(&a.2));

    rows.sort_by(|a, b| b.2.cmp(&a.2));
    if rows.is_empty() {
        println!("No workflow with a file and >= {min_runs} runs has a zero success count.");
        report_suppressed_and_deleted(&suppressed, &deleted, min_runs);
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
    report_suppressed_and_deleted(&suppressed, &deleted, min_runs);
    Ok(())
}

/// Say what was left out. A bounded report that does not name its bound reads as
/// a complete one, and the four workflows below the shipped floor include the two
/// whose failure is structural rather than situational.
fn report_suppressed_and_deleted(
    suppressed: &[(String, String, u64)],
    deleted: &[(String, String, u64)],
    min_runs: u64,
) {
    if !suppressed.is_empty() {
        println!();
        println!(
            "{} more have never succeeded but fall under --min-runs {min_runs}:",
            suppressed.len()
        );
        for (repo, name, runs) in suppressed {
            let short: String = name.chars().take(44).collect();
            println!("  {runs:>6}  {repo:<22} {short}");
        }
        println!("Few runs is not few enough to be safe: a workflow whose last step is");
        println!("forbidden by this repository's own ruleset fails every time it runs,");
        println!("and runs rarely.");
    }
    if !deleted.is_empty() {
        println!();
        println!(
            "{} registration(s) are `active` to the API with no file in the tree --",
            deleted.len()
        );
        println!("history, not a gate, and nothing to fix:");
        for (repo, name, runs) in deleted {
            let short: String = name.chars().take(44).collect();
            println!("  {runs:>6}  {repo:<22} {short}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// The tuple is found by its assignment, not by its name.
    ///
    /// `MERGE_CRITICAL` occurs in the checker's own docstring thirty lines before
    /// the assignment, and the text between the two holds an ODD number of
    /// quotes -- the docstring's own closing `"""` plus one quoted phrase. Pairing
    /// quotes from the wrong start inverts the parity, so every filename lands on
    /// an even index and `.skip(1).step_by(2)` sees none of them. Fifteen claims
    /// read as zero, and the report looked cleaner than the tree.
    #[test]
    fn the_tuple_is_found_by_its_assignment_not_its_name() {
        let q = '"';
        let src = format!(
            "{q}{q}{q}Every workflow in MERGE_CRITICAL must exist.\n\
             A rule inferred as {q}anything named gate{q} would stop covering it.\n\
             {q}{q}{q}\n\
             MERGE_CRITICAL = (\n    {q}a.yml{q},\n    {q}b.yml{q},\n)\n"
        );
        let mention = src.find("MERGE_CRITICAL").expect("docstring mention");
        let assignment = src.find("MERGE_CRITICAL = (").expect("assignment");
        assert!(mention < assignment, "the mention comes first -- that is the trap");
        assert_eq!(
            src[mention..assignment].matches('"').count() % 2,
            1,
            "the gap must hold an odd number of quotes, or the trap is not reproduced"
        );

        let pick = |from: usize| -> Vec<String> {
            let seg = &src[from..];
            let end = seg.find(")\n").unwrap_or(seg.len());
            seg[..end]
                .split('"')
                .skip(1)
                .step_by(2)
                .filter(|m| m.ends_with(".yml"))
                .map(|m| m.to_string())
                .collect()
        };
        assert_eq!(pick(assignment), vec!["a.yml", "b.yml"], "anchored on the assignment");
        assert!(
            pick(mention).is_empty(),
            "anchored on the name it must find nothing, or this test proves nothing"
        );
    }

    /// A job's context is its `name:` when it has one, else its id.
    ///
    /// GitHub matches a required check by context. Matching by file name would
    /// report every workflow as unrequired; matching by job id alone would miss
    /// every job that sets a display name.
    #[test]
    fn a_jobs_context_is_its_name_when_it_has_one() {
        let y = "on:\n  push:\n\njobs:\n  bare:\n    runs-on: x\n  named:\n    name: Pretty Name\n    runs-on: x\n";
        assert_eq!(contexts_of(y), vec!["bare", "Pretty Name"]);
    }

    /// A registration with no file is history, whatever its run count.
    ///
    /// GitHub keeps a workflow `active` after its file is deleted: 61 registrations
    /// against 48 files here. Nine of them have never succeeded, and one has 31
    /// failures -- more than four of the six real ones. Bucketing by run count
    /// first would put that phantom above the workflow whose last step this
    /// repository's own ruleset rejects.
    #[test]
    fn a_deleted_workflow_is_history_at_any_run_count() {
        assert_eq!(classify(false, 1000, 50), Bucket::Deleted, "many runs");
        assert_eq!(classify(false, 1, 50), Bucket::Deleted, "one run");
        assert_eq!(classify(false, 0, 50), Bucket::Deleted, "none");
    }

    /// Below the floor is named, not dropped.
    ///
    /// `brain-seal-refresh.yml` has 8 lifetime runs across five months and fails
    /// every one: its last step is a `git push` to master, which the ruleset
    /// answers with GH013. The shipped floor of 50 hid it completely, and a
    /// bounded report that does not name its bound reads as a complete one.
    #[test]
    fn under_the_floor_is_a_bucket_and_not_a_silence() {
        assert_eq!(classify(true, 8, 50), Bucket::Suppressed);
        assert_eq!(classify(true, 62, 50), Bucket::Reported);
        assert_eq!(
            classify(true, 50, 50),
            Bucket::Reported,
            "the floor is a minimum to judge, so meeting it is enough"
        );
    }

    /// Each operator has a flag, and each flag selects exactly one.
    ///
    /// `--boundary` had none: it was reachable only through `--all`, and it is
    /// the operator with the worst kill rate of the five, so it is the one you
    /// most want to run alone. Four selectable and the weakest not is exactly
    /// backwards, and nothing here would have said so -- the gap was in the
    /// argument parser, where no measurement looks.
    #[test]
    fn every_operator_has_its_own_flag() {
        for f in ["--loud", "--invert", "--boundary", "--assert"] {
            assert!(
                gates_mutate_accepts(f),
                "{f} is not accepted by `tri gates mutate`"
            );
        }
        // And a spelling that does not exist must be refused, or the loop above
        // would pass against a parser that accepts anything.
        assert!(!gates_mutate_accepts("--bounadry"));
    }

    fn gates_mutate_accepts(flag: &str) -> bool {
        // `Root` wraps GatesCmd directly, so the argv is `<prog> mutate <flag>`
        // and not `tri gates mutate <flag>`. Getting this wrong made every flag
        // look unaccepted, including the four that already worked -- which is
        // what a negative control is for: `--loud` failing was the tell that the
        // harness was wrong, not the parser.
        Root::try_parse_from(["tri-gates", "mutate", flag]).is_ok()
    }

    /// Three ways two DIFFERENT control sets used to share one cache key. That
    /// direction is the harmful one: a shared key serves a row measured against
    /// one input when asked about another.
    ///
    /// Each case here fails against the `unwrap_or_default()` + bare-concat
    /// version -- verified by reverting the function and watching them go red,
    /// which is the only evidence that a test tests anything.
    #[test]
    fn sha_of_separates_what_used_to_collide() {
        let d = std::env::temp_dir().join(format!("tri-sha-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let w = |n: &str, b: &str| {
            let p = d.join(n);
            std::fs::write(&p, b).unwrap();
            p
        };

        // The split moves, the concatenation does not: "ab"+"c" == "a"+"bc".
        // Reachable whenever a gate declares more than one control.
        let (ab, c) = (w("ab", "ab"), w("c", "c"));
        let (a, bc) = (w("a", "a"), w("bc", "bc"));
        assert_ne!(
            sha_of(&[ab, c]),
            sha_of(&[a, bc]),
            "two control sets differing only in where the boundary falls must not share a key"
        );

        // "the file is gone" is not "the file is empty".
        let empty = w("empty", "");
        let gone = d.join("gone");
        assert_ne!(
            sha_of(&[empty]),
            sha_of(&[gone.clone()]),
            "a missing control must not hash as an empty one"
        );

        // Two missing controls are two different absences.
        assert_ne!(
            sha_of(&[gone]),
            sha_of(&[d.join("also-gone")]),
            "two distinct missing paths must not share a key"
        );

        // And the point of a cache: same bytes, same key.
        let stable = w("stable", "same");
        assert_eq!(sha_of(&[stable.clone()]), sha_of(&[stable]));
        let _ = std::fs::remove_dir_all(&d);
    }

    /// `GatesCmd` is a `Subcommand`, so asking clap what `--min-runs` defaults
    /// to needs a root parser. This one exists for no other purpose.
    #[derive(Parser)]
    struct Root {
        #[command(subcommand)]
        action: GatesCmd,
    }

    /// The floor `tri gates dead` actually ships with, read back out of clap
    /// rather than repeated as a literal here.
    /// `days_since` is the whole staleness rule, so it is exercised without a
    /// network. The case that matters is the LAST one: an unparseable date must
    /// not read as fresh, because "I could not tell" and "it is fine" are the
    /// two answers this command exists to keep apart.
    #[test]
    fn days_since_counts_whole_days_and_refuses_to_guess() {
        let now = chrono::Utc::now();
        let mk = |d: i64| (now - chrono::Duration::days(d)).to_rfc3339();

        assert_eq!(super::days_since(&mk(0)), Some(0));
        assert_eq!(super::days_since(&mk(1)), Some(1));
        assert_eq!(super::days_since(&mk(45)), Some(45));

        // A clock skewed into the future is 0 days old, not a negative number
        // that would underflow the comparison.
        let future = (now + chrono::Duration::days(3)).to_rfc3339();
        assert_eq!(super::days_since(&future), Some(0));

        // Not a date. None, so the caller treats it as stale.
        assert_eq!(super::days_since("never"), None);
        assert_eq!(super::days_since(""), None);
        assert_eq!(super::days_since("2026-08-28"), None);
    }

    /// The staleness decision itself, spelled out: None must mean stale.
    #[test]
    fn an_unreadable_date_is_stale_not_fresh() {
        let stale_days = 30u64;
        let decide = |iso: &str| match super::days_since(iso) {
            Some(d) => d > stale_days,
            None => true,
        };
        let fresh = (chrono::Utc::now() - chrono::Duration::days(2)).to_rfc3339();
        let old = (chrono::Utc::now() - chrono::Duration::days(200)).to_rfc3339();
        assert!(!decide(&fresh), "a two-day-old run is not stale");
        assert!(decide(&old), "a 200-day-old run is stale");
        assert!(
            decide("garbage"),
            "an unreadable date must not read as fresh"
        );
    }

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
        // A bare name is a value the caller decides about, not a verdict.
        // Forcing `return fails` to 0 mutates a helper and scores the result
        // against the gate.
        assert!(mutable_sites("def collect():\n    return fails\n").is_empty());
        assert!(mutable_sites("def go():\n    raise SystemExit(main())\n").is_empty());
    }

    #[test]
    fn the_loud_operator_takes_only_bare_success_returns() {
        // T99: `return 0` is the success return; forcing it to 1 asks whether
        // anything requires the gate to be SILENT. A ternary is excluded on
        // purpose -- it can yield 0 on one arm and a verdict on the other, so
        // forcing the whole line to 1 is the Silent operator seen backwards and
        // would be scored against the wrong control.
        let s = sites_in_direction("def main():\n    return 0\n", Direction::Loud);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].2, "return 1");

        // A ternary that can yield 0 IS a success path: forcing the whole line
        // to 1 makes the gate fail unconditionally, which is what loud means.
        // Excluding it printed "no success path to break" for a gate whose
        // every verdict is a ternary -- a clean-looking row that was an absence
        // of measurement.
        for src in [
            "def main():\n    return 0 if ok else 1\n",
            "def main():\n    return 1 if bad else 0\n",
        ] {
            let s = sites_in_direction(src, Direction::Loud);
            assert_eq!(s.len(), 1, "loud missed a ternary success path in {src:?}");
            assert_eq!(s[0].2, "return 1");
        }
        for src in [
            "def main():\n    return 1\n",
            "def main():\n    return 2\n",
            "def main():\n    return fails\n",
            "def main():\n    return code0\n",
        ] {
            assert!(
                sites_in_direction(src, Direction::Loud).is_empty(),
                "loud took a non-success return in {src:?}"
            );
        }
        // And the two directions never claim the same line.
        let both = "def main():\n    if bad:\n        return 1\n    return 0\n";
        let quiet = sites_in_direction(both, Direction::Silent);
        let loud = sites_in_direction(both, Direction::Loud);
        assert_eq!(quiet.len(), 1);
        assert_eq!(loud.len(), 1);
        assert_ne!(quiet[0].0, loud[0].0);
    }

    #[test]
    fn a_ternary_return_is_a_failure_path() {
        // T90: the first scanner matched only a bare `return 1..4` and missed
        // eight sites across seven files -- seven ternaries and one
        // SystemExit. It then reported pack_index_consistency_gate.py as
        // having "no failure path to break" when every verdict in that gate is
        // a ternary. A path the scanner cannot see is scored as covered.
        for src in [
            "def main():\n    return 1 if bad else 0\n",
            "def main():\n    return 0 if killed else 1\n",
            "def main():\n    return 0 if not fails else 1\n",
        ] {
            let s = mutable_sites(src);
            assert_eq!(s.len(), 1, "missed a ternary verdict in {src:?}");
            assert_eq!(
                s[0].2, "return 0",
                "a ternary is neutered whole, not per arm"
            );
        }
        let s = mutable_sites("def main():\n    raise SystemExit(3)\n");
        assert_eq!(s.len(), 1, "missed a SystemExit verdict");
        assert_eq!(s[0].2, "raise SystemExit(0)");
    }

    #[test]
    fn a_digit_inside_a_name_is_not_a_verdict() {
        // `return t27c_failures` and `return code2` contain a 1..4 character.
        // Reading them as verdicts would invent sites, and an invented site
        // that survives is an invented finding.
        // A verdict is a LITERAL, or a ternary of two literals. Anything else
        // is a value the caller decides about.
        assert!(verdict_literals("t27c_failures").is_none());
        assert!(verdict_literals("code2").is_none());
        assert!(verdict_literals("x.f1").is_none());
        assert_eq!(verdict_literals("1"), Some(vec![1]));
        assert_eq!(verdict_literals("1 if bad else 0"), Some(vec![1, 0]));

        // T103: these two were taken as verdicts and were not. `v == 0` is a
        // comparison; the second is a string with an index in it. The loud
        // operator reported both helpers as gates nothing keeps silent.
        assert!(verdict_literals("v == 0").is_none());
        assert!(verdict_literals(r#"out.splitlines()[0][:88] if out else "(nothing)""#).is_none());
    }

    #[test]
    fn nested_defs_do_not_end_the_control_region() {
        // `case()` and friends are defined INSIDE self_check, indented. Only a
        // `def` at column zero changes which region we are in.
        let src = "def self_check():\n    def case(x):\n        return 1\n    return 0\n";
        assert!(mutable_sites(src).is_empty());
    }

    #[test]
    fn an_equivalence_claim_names_the_line_it_describes() {
        // T107. The first version used marker+2, and the proof it was written
        // for is a fifteen-line comment block -- so it named a line in the
        // middle of its own explanation. A one-line proof would have passed.
        let one = "# mutant-equivalent: guards force it\nif a > b:\n";
        assert_eq!(
            equivalence_claims(one).get(&2).map(String::as_str),
            Some("guards force it")
        );

        let many = "# mutant-equivalent: proven below\n# line two\n# line three\n\
                    # line four\nif a > b:\n";
        let c = equivalence_claims(many);
        assert_eq!(
            c.get(&5).map(String::as_str),
            Some("proven below"),
            "a multi-line proof lost its target: {c:?}"
        );
        assert!(
            c.get(&3).is_none(),
            "named a line inside its own comment block"
        );

        // A blank line between proof and code is still the same claim.
        assert!(equivalence_claims("# mutant-equivalent: x\n\nif a > b:\n").contains_key(&3));
        // The words in running prose are not a claim; only a comment is.
        assert!(equivalence_claims("s = \"mutant-equivalent: no\"\nif a > b:\n").is_empty());
        // A marker with nothing after it names nothing rather than panicking.
        assert!(equivalence_claims("# mutant-equivalent: x\n").is_empty());
    }

    #[test]
    fn a_ternary_exit_is_a_failure_path() {
        // T119: `sys.exit(0 if ok else 1)` is how this repository ends a
        // verifier, and a substring test for "sys.exit(1" cannot see it. Three
        // CI gates were classified as not-gates for that reason, and the count
        // of uncovered gates was reported too low for a week.
        let base = std::env::temp_dir().join(format!("tri_t119_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(".github/workflows")).unwrap();
        std::fs::write(
            base.join(".github/workflows/ci.yml"),
            "run: python3 tools/verifier.py\nrun: python3 tools/reporter.py\n",
        )
        .unwrap();

        assert!(
            is_gate_by_property(&base, "verifier.py", "    sys.exit(0 if ok else 1)\n"),
            "a ternary exit is a failure path"
        );
        assert!(
            is_gate_by_property(&base, "verifier.py", "    return 1 if bad else 0\n"),
            "a ternary return is a failure path"
        );
        assert!(
            is_gate_by_property(&base, "verifier.py", "    sys.exit(1)\n"),
            "the plain form must keep working"
        );

        // And the other direction, or everything becomes a gate: a script that
        // only ever succeeds is not one, and neither is a failing script nobody
        // invokes.
        assert!(
            !is_gate_by_property(&base, "reporter.py", "    sys.exit(0)\n"),
            "a script that cannot fail is not a gate"
        );
        assert!(
            !is_gate_by_property(&base, "unmentioned.py", "    sys.exit(1)\n"),
            "a script no workflow invokes is not a gate"
        );
        // `sys.exit(main())` is a dispatch, not a verdict.
        assert!(
            !is_gate_by_property(&base, "verifier.py", "    sys.exit(main())\n"),
            "a dispatch is not a verdict"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_control_is_found_in_every_form_it_takes() {
        // T112: the search that reported "no control" for three well-controlled
        // gates in another repository, because it looked for a flag and a
        // hand-kept table and nothing else.
        let base = std::env::temp_dir().join(format!("tri_t112_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("tests")).unwrap();
        std::fs::create_dir_all(base.join(".github/workflows")).unwrap();

        // Nothing planted: the alarm must still be an alarm.
        assert!(
            control_forms(&base, "print('hi')\n", "check_bare.py").is_empty(),
            "an uncontrolled gate must stay uncontrolled"
        );

        // The flag, read from the source rather than from a comment.
        let f = control_forms(&base, "if \"--self-check\" in sys.argv:\n", "check_x.py");
        assert!(
            f.iter().any(|s| s.starts_with("flag --self-check")),
            "{f:?}"
        );
        assert!(
            control_forms(&base, "# mentions --self-check in prose\n", "check_y.py").is_empty(),
            "a flag named only in a comment is not a control"
        );

        // A test file beside the gate.
        std::fs::write(base.join("tests/test_check_paths.py"), "x").unwrap();
        let f = control_forms(&base, "", "check_paths.py");
        assert!(f.iter().any(|s| s.contains("test file")), "{f:?}");

        // A workflow that names the gate AND carries planted-fault vocabulary.
        std::fs::write(
            base.join(".github/workflows/selftest.yml"),
            "run: python3 tools/check_paths.py --root tests/fixture/broken\n",
        )
        .unwrap();
        let f = control_forms(&base, "", "check_paths.py");
        assert!(f.iter().any(|s| s.contains("workflow candidate")), "{f:?}");

        // T113: the false positive this heuristic produced on its first real
        // use. The word `must` sat in a prose comment far from the call, and
        // three uncontrolled gates were labelled as having a control -- the
        // false green this command exists to find, produced by the command.
        std::fs::write(
            base.join(".github/workflows/faraway.yml"),
            format!(
                "# a comment that says must and broken\n{}run: python3 tools/check_far.py\n",
                "# filler\n".repeat(60)
            ),
        )
        .unwrap();
        let f = control_forms(&base, "", "check_far.py");
        assert!(
            f.is_empty(),
            "prose 60 lines from the call is not a control, got {f:?}"
        );

        // And the strong words still count when they are beside the call.
        std::fs::write(
            base.join(".github/workflows/near.yml"),
            "run: python3 tools/check_near.py --root tests/fixture/broken\n",
        )
        .unwrap();
        assert!(
            control_forms(&base, "", "check_near.py")
                .iter()
                .any(|s| s.contains("workflow candidate")),
            "a fixture path beside the call is the evidence this looks for"
        );

        // A workflow that merely RUNS the gate is not evidence of a control.
        std::fs::write(
            base.join(".github/workflows/plain.yml"),
            "run: python3 tools/check_other.py\n",
        )
        .unwrap();
        let f = control_forms(&base, "", "check_other.py");
        assert!(
            f.is_empty(),
            "running a gate is not controlling it, got {f:?}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn every_degenerate_dir_is_refused_by_name() {
        // T111: the four inputs `--dir` accepted on the day it shipped. Two of
        // them exited 0 -- an empty directory printed a header and no rows,
        // which reads exactly like a clean suite, and a directory outside any
        // repository passed the dirty-tree guard because `git status` fails
        // there and its empty stdout reads as clean.
        //
        // The second is the one with teeth: this command rewrites each gate in
        // place and restores it, and that restore is only recoverable because
        // `git checkout` exists. Outside a work tree an interrupted run leaves
        // a mutated file and no way back.
        let base = std::env::temp_dir().join(format!("tri_t111_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("plain")).unwrap();
        std::fs::write(base.join("afile"), "x").unwrap();

        let msg = |d: &std::path::Path| match resolve_target(Some(d.to_str().unwrap())) {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("{e:#}"),
        };

        assert!(
            msg(&base.join("nope")).contains("--dir"),
            "a missing dir must name the flag"
        );
        assert!(
            msg(&base.join("afile")).contains("not a directory"),
            "a file must be refused as a file, not as a git failure"
        );
        assert!(
            msg(&base.join("plain")).contains("git work tree"),
            "a directory outside a repository must be refused: got {:?}",
            msg(&base.join("plain"))
        );

        // And the accepting direction, or the three refusals above would pass
        // for a function that refuses everything.
        let repo = base.join("repo");
        std::fs::create_dir_all(repo.join("tools")).unwrap();
        let ran = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&repo)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ran {
            let (root, tools) = resolve_target(Some(repo.join("tools").to_str().unwrap()))
                .expect("a tools dir inside a repository is the normal case");
            assert!(tools.ends_with("tools"));
            assert_eq!(
                std::fs::canonicalize(&root).unwrap(),
                std::fs::canonicalize(&repo).unwrap(),
                "root must be the work tree, not the parent of the directory"
            );
        }
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn the_boundary_operator_takes_comparisons_and_nothing_else() {
        // T107. The operator is an off-by-one, so what it must NOT take matters
        // as much as what it takes: a shift, a return annotation, a comment or
        // a string mutated here would make the gate crash or change a message,
        // and a control that reds on a traceback scores that as a kill for the
        // wrong reason -- the mistake the whole command exists to catch.
        let s = |src| sites_in_direction(src, Direction::Boundary);

        assert_eq!(s("if a > b:\n").len(), 1);
        assert_eq!(s("if a >= b:\n")[0].2, ">");
        assert_eq!(s("if a > b:\n")[0].2, ">=");
        assert_eq!(s("if a <= b:\n")[0].2, "<");
        assert_eq!(s("if a < b:\n")[0].2, "<=");
        // Two comparisons on one line are two independent sites.
        assert_eq!(s("if a > b and c < d:\n").len(), 2);

        assert!(s("x = a << 2\n").is_empty(), "took a left shift");
        assert!(s("x = a >> 2\n").is_empty(), "took a right shift");
        assert!(
            s("def f(a) -> int:\n").is_empty(),
            "took a return annotation"
        );
        assert!(s("# a > b\n").is_empty(), "took a comment");
        assert!(s("x = 1  # a > b\n").is_empty(), "took a trailing comment");
        assert!(s("print(\"a > b\")\n").is_empty(), "took a string");
        assert!(
            s("print('a > b')\n").is_empty(),
            "took a single-quoted string"
        );
        // The control's own functions are off limits, same as every operator.
        assert!(s("def self_check():\n    if a > b:\n").is_empty());

        // T107b: docstrings. The first scanner tracked quotes per LINE, so
        // every `>` in every module docstring became a site -- prose about
        // ratchets and usage on lines 10, 43, 136 and 230 of four gates,
        // reported as surviving mutants. Real count, wrong meaning.
        assert!(
            s("\"\"\"usage: x --a <b>\n  and total > 0 means data\n\"\"\"\nx = 1\n").is_empty(),
            "took a comparison out of a module docstring"
        );
        assert!(
            s("def f():\n    \"\"\"doc: a > b\n    more: c < d\n    \"\"\"\n    return 0\n")
                .is_empty(),
            "took a comparison out of a function docstring"
        );
        assert!(
            s("x = \'\'\'a > b\'\'\'\n").is_empty(),
            "took a single-quoted triple"
        );
        // A `def` at column 0 INSIDE a docstring is prose, not a definition:
        // acting on it is how a scanner walks out of a string it is still in.
        assert!(
            s("\"\"\"\ndef self_check():\n\"\"\"\nif a > b:\n    x = 1\n").len() == 1,
            "a def inside a docstring changed the control scope"
        );
        // And code AFTER a docstring is still code.
        assert_eq!(
            s("\"\"\"doc a > b\"\"\"\nif c > d:\n    return 1\n").len(),
            1,
            "lost the code after a docstring"
        );

        // And it is its own operator, not a relabelling of another. This is the
        // assertion #2500 was missing.
        let src = "def main():\n    if a > b:\n        return 1\n    return 0\n";
        let bnd = sites_in_direction(src, Direction::Boundary);
        assert_ne!(bnd, sites_in_direction(src, Direction::Silent));
        assert_ne!(bnd, sites_in_direction(src, Direction::Loud));
        assert_ne!(bnd, sites_in_direction(src, Direction::Invert));
    }

    #[test]
    fn each_direction_reaches_its_own_operator() {
        // T105: the wiring test the ten tests above did not contain. Every one
        // of them exercised invert_sites() or sites_in_direction() alone, and
        // all ten passed while the two were not joined: Direction::Invert was
        // declared, documented and never constructed, invert_sites() had zero
        // callers, and `--invert` printed its banner over a silent run.
        //
        // This is the same shape as the defect the whole command exists to
        // find -- the checking FUNCTION is covered and the path from it to the
        // answer is not -- arriving one level up, in the auditor.
        let src = "def main():\n    if bad:\n        return 1\n    return 0\n";
        let s = sites_in_direction(src, Direction::Silent);
        let l = sites_in_direction(src, Direction::Loud);
        let i = sites_in_direction(src, Direction::Invert);

        assert_eq!(s.len(), 1, "silent lost its site");
        assert_eq!(l.len(), 1, "loud lost its site");
        assert_eq!(i.len(), 1, "invert lost its site");
        assert_eq!(s[0].2, "return 0");
        assert_eq!(l[0].2, "return 1");
        assert!(i[0].2.starts_with("if not ("), "invert wrote {:?}", i[0].2);

        // Three DISTINCT sites on one fixture. Equality here is the bug: it is
        // what a direction silently falling through to another produces, and
        // it is what shipped.
        assert_ne!(s, l, "silent and loud returned one operator's sites");
        assert_ne!(s, i, "invert fell through to the silent operator");
        assert_ne!(l, i, "invert fell through to the loud operator");
        assert_eq!(i[0].0, invert_sites(src)[0].0, "Invert is not invert_sites");
    }

    #[test]
    fn invert_takes_only_verdict_bearing_conditions() {
        // T104: inverting a loop guard or a plumbing check makes the gate
        // CRASH, and a control that reds on a traceback scores that as a kill
        // for the wrong reason. Measured over 19 mutations on four gates with
        // this scoping: 19 killed by MESSAGE, zero tracebacks.
        let s = invert_sites("def main():\n    if bad:\n        return 1\n");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].2, "if not (bad):");

        // A body that only prints decides nothing this command can speak to.
        assert!(invert_sites("def main():\n    if x:\n        print(x)\n").is_empty());
        // A body that returns success is not a verdict branch either.
        assert!(invert_sites("def main():\n    if x:\n        return 0\n").is_empty());
        // A FAIL print counts even without a return: the branch still decides.
        assert_eq!(
            invert_sites("def main():\n    if x:\n        print(\"FAIL: no\")\n").len(),
            1
        );
        // Inside the control, never.
        assert!(invert_sites("def self_check():\n    if bad:\n        return 1\n").is_empty());
        // Top-level `if` is module plumbing, not a gate decision.
        assert!(invert_sites("if bad:\n    return 1\n").is_empty());
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
    fn an_external_control_is_never_reported_as_uncontrolled() {
        // The invariant that matters is not "a control file is not a gate" --
        // that was the wp18 shape, where the control does nothing else.
        // check_gate_preconditions.py breaks it deliberately: it covers the
        // precondition branch of six gates AND is a gate in its own right,
        // with its own self-check. What must hold is that a script named as
        // somebody's control never ends up in the uncontrolled list, which
        // would be a finding invented out of the wiring.
        for (g, c) in EXTERNAL_CONTROL {
            assert_ne!(g, c, "a script cannot be its own negative control");
            assert!(
                IS_A_CONTROL.contains(c) || CONTROL_IS_ALSO_A_GATE.contains(c),
                "{c} is named as a control but is neither excluded from the gate \
                 list nor declared as a gate that carries its own control"
            );
        }
        // And the declaration is not free: a script listed there must really
        // have a control of its own, or the exemption hides the thing the
        // sweep exists to report.
        for c in CONTROL_IS_ALSO_A_GATE {
            let src = std::fs::read_to_string(repo_root().expect("git repo").join("tools").join(c))
                .unwrap_or_default();
            assert!(
                ["--self-check-drop", "--self-check", "--selftest"]
                    .iter()
                    .any(|fl| src.contains(&format!("\"{}\"", fl))),
                "{c} claims to be a gate with its own control and declares no control flag"
            );
        }
    }

    fn claim(line: usize, why: &str) -> std::collections::HashMap<usize, String> {
        let mut m = std::collections::HashMap::new();
        m.insert(line, why.to_string());
        m
    }

    #[test]
    fn boundary_leaves_the_control_function_at_the_next_top_level_statement() {
        // Three regions, three answers. The middle one is a control and must
        // stay excluded; the third is the `__main__` block, which the byte
        // scanner used to swallow along with it because it never reset the
        // flag -- T125, fixed in the other three site finders and not in this
        // one until now.
        let src = "def helper():\n    if a > 1: pass\n\n\ndef self_check():\n    if b > 2: pass\n\n\nif __name__ == \"__main__\":\n    if c > 3: pass\n";
        let sites = super::sites_in_direction(src, super::Direction::Boundary);
        let lines: Vec<usize> = sites
            .iter()
            .map(|(at, _, _)| super::line_of(src, *at))
            .collect();
        assert!(
            lines.contains(&2),
            "helper's comparison is a site: {lines:?}"
        );
        assert!(
            !lines.contains(&6),
            "self_check's comparison is CONTROL and must not be a site: {lines:?}"
        );
        assert!(
            lines.contains(&10),
            "the __main__ block is not part of self_check: {lines:?}"
        );
        assert_eq!(lines.len(), 2, "exactly two sites: {lines:?}");
    }

    #[test]
    fn an_escaped_quote_in_an_assert_message_keeps_the_message() {
        // The mutant must stay parseable AND keep the message. With the
        // backslash advancing one byte instead of two, the escaped quote closed
        // the string: the first line produced `assert True,b", "..."` (does not
        // parse -- python exits 1 and the site scores as KILLED), and the second
        // stranded the quote state so no top-level comma was found and the
        // message was silently dropped.
        let src = "assert s == \"a\\\",b\", \"the header row must be quoted\"\n";
        let sites = super::sites_in_direction(src, super::Direction::Assert);
        assert_eq!(sites.len(), 1, "one assertion: {sites:?}");
        let rep = &sites[0].2;
        assert!(
            rep.contains("the header row must be quoted"),
            "the message must survive: {rep}"
        );
        assert!(
            !rep.contains("b\","),
            "the mutant must not carry half of the old condition: {rep}"
        );

        let src2 = "assert t == 'it\\'s', \"the label must carry an apostrophe\"\n";
        let sites2 = super::sites_in_direction(src2, super::Direction::Assert);
        assert_eq!(sites2.len(), 1);
        assert!(
            sites2[0].2.contains("the label must carry an apostrophe"),
            "an apostrophe escape must not drop the message: {}",
            sites2[0].2
        );
    }

    #[test]
    fn prose_in_a_docstring_is_not_an_assertion() {
        // Word-wrapped prose that happens to begin a line with the word
        // `assert` was scored as an assertion to neuter. It is not code.
        let src = "\"\"\"A gate that documents itself.\n\nassert anything.\n\"\"\"\nx = 1\n";
        let sites = super::sites_in_direction(src, super::Direction::Assert);
        assert!(sites.is_empty(), "prose is not an assertion: {sites:?}");
    }

    #[test]
    fn a_flush_left_docstring_line_does_not_end_the_control_function() {
        // Two sources differing by the indentation of ONE docstring line. The
        // flush-left one used to clear `in_control`, handing the operators the
        // control's own assert -- a site nobody can kill, because neutering a
        // control makes it pass.
        let flush = "def self_check():\n    \"\"\"Doc.\n\nWrapped prose at column zero.\n\"\"\"\n    assert 1 == 1, \"a control assert must never be a mutation site\"\n";
        let indented = "def self_check():\n    \"\"\"Doc.\n\n    Wrapped prose, indented.\n    \"\"\"\n    assert 1 == 1, \"a control assert must never be a mutation site\"\n";
        let a = super::sites_in_direction(flush, super::Direction::Assert);
        let b = super::sites_in_direction(indented, super::Direction::Assert);
        assert!(b.is_empty(), "indented control body yields no sites: {b:?}");
        assert_eq!(
            a.len(),
            b.len(),
            "indentation of a docstring line must not change the site count: {a:?} vs {b:?}"
        );
    }

    #[test]
    fn prose_that_mentions_the_marker_is_not_a_claim() {
        // The sentence that found this: a comment ABOUT the mechanism, which
        // registered as a claim bound to the next code line.
        let src = "# that reasoning sits on the line as a `# mutant-equivalent:` claim\nx = 1\n";
        assert!(
            super::equivalence_claims(src).is_empty(),
            "a mention is not a claim"
        );
    }

    #[test]
    fn a_marker_opening_the_comment_is_a_claim() {
        let src = "# mutant-equivalent: the guard forces it\nif a >= 1:\n";
        let c = super::equivalence_claims(src);
        assert_eq!(c.len(), 1);
        assert_eq!(c[&2], "the guard forces it");
    }

    #[test]
    fn a_claim_whose_mutant_survived_is_not_contradicted() {
        // The ordinary case, and the one that must stay silent: one site on
        // the claimed line, and it survived.
        let out = super::contradicted_claims("g.py", "boundary", &[7], &[7], &claim(7, "forced"));
        assert!(
            out.is_empty(),
            "a surviving mutant refutes nothing: {out:?}"
        );
    }

    #[test]
    fn a_claim_whose_mutant_died_is_contradicted() {
        let out = super::contradicted_claims("g.py", "boundary", &[7], &[], &claim(7, "forced"));
        assert_eq!(out.len(), 1);
        assert!(out[0].contains("g.py:7"), "{}", out[0]);
        assert!(out[0].contains("DIED"), "{}", out[0]);
        assert!(
            out[0].contains("forced"),
            "the WHY must survive into the report: {}",
            out[0]
        );
    }

    #[test]
    fn a_partially_true_claim_is_still_contradicted() {
        // Two sites on one line -- `if a < 1 or b < 1:` -- one dead, one alive.
        // Keying on "did the line vanish from the survivor list" would call
        // this claim intact, and a half-true claim is the hardest to see by eye.
        let out =
            super::contradicted_claims("g.py", "boundary", &[7, 7], &[7], &claim(7, "forced"));
        assert_eq!(out.len(), 1, "one of two mutants died: {out:?}");
        assert!(out[0].contains("1 of 2"), "{}", out[0]);
    }

    #[test]
    fn a_claim_on_a_line_with_no_site_in_this_direction_is_silent() {
        // The claim is real but this operator does not mutate that line. Saying
        // anything here would report the OPERATOR's scope as a false claim.
        let out = super::contradicted_claims("g.py", "invert", &[9], &[9], &claim(7, "forced"));
        assert!(out.is_empty(), "no site, no verdict: {out:?}");
    }
}

#[cfg(test)]
mod auto_default_run_tests {
    use super::has_auto_default_run;

    /// The shape that cost the measurement: `pull_request` and a `paths:`
    /// filter, no `push:` at all. Two default-branch runs exist for this file,
    /// both dispatched by hand, so a STALENESS check reports it healthy.
    #[test]
    fn pull_request_only_cannot_produce_a_baseline() {
        let y = "name: x\non:\n  pull_request:\n    paths:\n      - 'bootstrap/**'\n  workflow_dispatch:\njobs: {}\n";
        assert!(!has_auto_default_run(y, "master"));
    }

    #[test]
    fn push_to_the_default_branch_can() {
        let y = "name: x\non:\n  pull_request:\n  push:\n    branches: [master]\n  workflow_dispatch:\njobs: {}\n";
        assert!(has_auto_default_run(y, "master"));
    }

    /// `push:` with nothing under it fires on every branch, the default
    /// included. Reading "no branches listed" as "no branches" would call a
    /// workflow unreachable that runs on every push in the repository.
    #[test]
    fn a_bare_push_covers_every_branch() {
        let y = "name: x\non:\n  push:\n  pull_request:\njobs: {}\n";
        assert!(has_auto_default_run(y, "master"));
    }

    #[test]
    fn push_to_some_other_branch_does_not() {
        let y = "name: x\non:\n  push:\n    branches: [release, staging]\njobs: {}\n";
        assert!(!has_auto_default_run(y, "master"));
    }

    #[test]
    fn a_block_list_is_read_as_well_as_an_inline_one() {
        let y = "name: x\non:\n  push:\n    branches:\n      - main\n      - master\njobs: {}\n";
        assert!(has_auto_default_run(y, "master"));
    }

    #[test]
    fn the_inline_on_form_is_read() {
        assert!(has_auto_default_run("on: [push, pull_request]\njobs: {}\n", "master"));
        assert!(!has_auto_default_run("on: [pull_request]\njobs: {}\n", "master"));
    }

    /// COUNTEREXAMPLE. `harness-scratch.yml` recommends `push: branches:
    /// [master]` in prose in its own header. A scan that trims whitespace and
    /// looks for `push:` reads that advice as compliance and reports the file
    /// covered when it is not.
    #[test]
    fn a_push_inside_a_comment_is_not_a_trigger() {
        let y = "# `push: branches: [master]` costs one line and is worth it.\n\
                 name: x\n\
                 on:\n\
                 \x20 pull_request:\n\
                 jobs: {}\n";
        assert!(!has_auto_default_run(y, "master"));
    }

    /// COUNTEREXAMPLE. A `push` deeper than the first level inside `on:` -- or
    /// anywhere in the jobs below -- is not the trigger either.
    #[test]
    fn a_nested_push_is_not_the_trigger() {
        let y = "name: x\non:\n  pull_request:\n    paths:\n      - 'push:'\njobs:\n  a:\n    steps:\n      - run: git push\n";
        assert!(!has_auto_default_run(y, "master"));
    }

    /// A wildcard branch list covers the default branch.
    #[test]
    fn a_wildcard_branch_list_covers_it() {
        let y = "name: x\non:\n  push:\n    branches: ['**']\njobs: {}\n";
        assert!(has_auto_default_run(y, "master"));
    }

    /// The default branch is a parameter, not the literal `master`.
    #[test]
    fn the_default_branch_is_not_hardcoded() {
        let y = "name: x\non:\n  push:\n    branches: [main]\njobs: {}\n";
        assert!(has_auto_default_run(y, "main"));
        assert!(!has_auto_default_run(y, "master"));
    }

    /// Every workflow file in this repository is read, and the answer is
    /// compared against a second reader written differently: the file has a
    /// `push:` line at the first level inside `on:` at all. The two may
    /// disagree only where a branch filter excludes the default branch, and
    /// that disagreement is the whole point of the function.
    #[test]
    fn every_workflow_in_the_tree_parses_without_panicking() {
        let dir = std::path::Path::new(".github/workflows");
        if !dir.is_dir() {
            return; // run from a subdirectory; nothing to say
        }
        let mut read = 0usize;
        for e in std::fs::read_dir(dir).expect("read .github/workflows") {
            let p = e.expect("entry").path();
            if p.extension().and_then(|s| s.to_str()) != Some("yml") {
                continue;
            }
            let t = std::fs::read_to_string(&p).expect("read workflow");
            let _ = has_auto_default_run(&t, "master");
            read += 1;
        }
        assert!(
            read > 10,
            "expected to read the workflow directory, read {read} file(s) -- a test \
             that reads nothing passes vacuously"
        );
    }
}

#[cfg(test)]
mod branch_pattern_tests {
    use super::{branch_pattern_matches, has_auto_default_run};

    #[test]
    fn a_literal_matches_itself_and_nothing_else() {
        assert!(branch_pattern_matches("master", "master"));
        assert!(!branch_pattern_matches("master", "main"));
        assert!(!branch_pattern_matches("mast", "master"));
        assert!(!branch_pattern_matches("master", "mast"));
    }

    /// The case an equality test gets wrong: a pattern that covers the default
    /// branch without naming it.
    #[test]
    fn a_star_covers_the_default_branch() {
        assert!(branch_pattern_matches("ma*", "master"));
        assert!(branch_pattern_matches("*", "master"));
        assert!(branch_pattern_matches("**", "master"));
        let y = "on:\n  push:\n    branches: ['ma*']\njobs: {}\n";
        assert!(has_auto_default_run(y, "master"));
    }

    /// `*` stops at a slash and `**` does not -- GitHub's rule, and the reason
    /// `feature/**` does not cover `master` while `**` does.
    #[test]
    fn one_star_stops_at_a_slash_and_two_do_not() {
        assert!(!branch_pattern_matches("feature/*", "feature/a/b"));
        assert!(branch_pattern_matches("feature/**", "feature/a/b"));
        assert!(branch_pattern_matches("feature/*", "feature/a"));
        assert!(!branch_pattern_matches("*", "a/b"));
        assert!(branch_pattern_matches("**", "a/b"));
    }

    /// The live shape that separates "has a push key" from "has a push covering
    /// master": notebook-sync.yml pushes on four patterns, none of them master.
    /// Counting the KEY gives 16 files with no push; counting COVERAGE gives 17,
    /// and 17 is the number that answers "can this produce a baseline".
    #[test]
    fn a_push_whose_patterns_exclude_the_default_is_not_coverage() {
        let y = "on:\n  push:\n    branches: ['feature/**', 'fix/**', 'ring-*/**', 'issue-*/**']\njobs: {}\n";
        assert!(!has_auto_default_run(y, "master"));
        assert!(has_auto_default_run(y, "feature/x"));
    }

    /// Unimplemented syntax is treated as a literal, so it fails to match rather
    /// than matching everything. A wrong answer in a work list is visible; a
    /// wrong silence is not.
    #[test]
    fn unimplemented_syntax_errs_toward_reporting() {
        assert!(!branch_pattern_matches("mast?r", "master"));
        assert!(!branch_pattern_matches("+([a-z])", "master"));
    }
}

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
    for line in src.split_inclusive('\n') {
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
                    for (i, &c) in b.iter().enumerate() {
                        match quote {
                            Some(q) => {
                                if c == b'\\' {
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
    for line in src.split_inclusive('\n') {
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
    for (i, line) in lines.iter().enumerate() {
        off = offsets[i];
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
fn equivalence_claims(src: &str) -> std::collections::HashMap<usize, String> {
    const MARK: &str = "mutant-equivalent:";
    let lines: Vec<&str> = src.lines().collect();
    let mut out = std::collections::HashMap::new();
    for (i, l) in lines.iter().enumerate() {
        let Some(pos) = l.find(MARK) else { continue };
        if !l.trim_start().starts_with('#') {
            continue;
        }
        let why = l[pos + MARK.len()..].trim().to_string();
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
            .or_else(|| t.strip_prefix("raise SystemExit(").and_then(|r| r.strip_suffix(')')))
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
                    ["fixture", "expect_", "planted"].iter().any(|w| l.contains(w))
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
    killed: usize,
    total: usize,
    survivors: Vec<usize>,
}

fn sha_of(paths: &[std::path::PathBuf]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    for p in paths {
        h.update(std::fs::read(p).unwrap_or_default());
    }
    hex::encode(&h.finalize()[..8])
}

fn cache_path(root: &std::path::Path) -> std::path::PathBuf {
    root.join("target/.tri-mutate-cache.json")
}

fn load_cache(root: &std::path::Path) -> std::collections::HashMap<String, CachedRun> {
    std::fs::read_to_string(cache_path(root))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_cache(root: &std::path::Path, c: &std::collections::HashMap<String, CachedRun>) {
    if let Some(d) = cache_path(root).parent() {
        let _ = std::fs::create_dir_all(d);
    }
    if let Ok(s) = serde_json::to_string_pretty(c) {
        let _ = std::fs::write(cache_path(root), s);
    }
}

/// Open pull requests, with how much CI each one actually got.
fn prs(repo: Option<&str>) -> Result<()> {
    let mut base = vec!["pr", "list", "--state", "open", "--limit", "50", "--json",
                        "number,title,mergeable"];
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

    println!("{:<7} {:<13} {:>7}  {}", "pr", "mergeable", "checks", "title");
    let mut blind = Vec::new();
    let mut rows: Vec<(i64, String, usize)> = Vec::new();
    for it in &items {
        let n = it["number"].as_i64().unwrap_or(0);
        let m = it["mergeable"].as_str().unwrap_or("?").to_string();
        let title = it["title"].as_str().unwrap_or("");
        let mut cargs = vec!["pr".into(), "checks".into(), n.to_string(),
                             "--json".into(), "name".into()];
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
    let mut healthy: Vec<usize> = rows
        .iter()
        .filter(|(_, m, _)| m != "CONFLICTING")
        .map(|(_, _, c)| *c)
        .collect();
    healthy.sort_unstable();
    let reference = healthy.get(healthy.len() / 2).copied().unwrap_or(0);

    for (n, m, c) in &rows {
        if m == "CONFLICTING" && reference > 0 && *c * 2 < reference {
            blind.push((*n, *c));
        }
    }

    println!();
    if reference == 0 {
        println!("No non-conflicting pull request to compare against, so no reference.");
        return Ok(());
    }
    println!("Reference: a non-conflicting pull request here gets {} checks (median).", reference);
    if blind.is_empty() {
        println!("No pull request has a check list far below it.");
        return Ok(());
    }
    println!();
    println!(
        "{} pull request(s) CONFLICTING with a check list far below the reference:",
        blind.len()
    );
    for (n, c) in &blind {
        println!("  #{}  {} check(s) against a reference of {}", n, c, reference);
    }
    println!();
    println!("A pull request that is conflicting when an event fires cannot have its merge");
    println!("diff computed, so every workflow with a `paths:` filter is skipped for that");
    println!("event. The checks that remain are the ones that never look at the diff -- and");
    println!("they are green, which reads exactly like a passing pull request.");
    println!();
    println!("A conflict does NOT retract earlier runs: a pull request that was mergeable");
    println!("when it was last pushed keeps that list. Rebase to get a real one.");
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
        println!("(loud:   `return 0`    -> `return 1`  -- does anything require it to be SILENT?)");
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
        let _ = std::fs::write(&marker, &name);
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
        let mut judges: Vec<std::path::PathBuf> =
            flags.iter().map(|_| f.clone()).take(1).collect();
        if let Some(c) = &external {
            judges.push(tools.join(c));
        }
        let gate_sha = sha_of(&[f.clone()]);
        let ctrl_sha = sha_of(&judges);

        let mut scores: Vec<(Direction, usize, usize, Vec<usize>)> = Vec::new();
        let (mut n_row_cached, mut n_row_fresh) = (0usize, 0usize);
        for dir in directions {
        let key = format!("{}|{}", name, label(*dir));
        if let Some(c) = cache.get(&key) {
            if c.gate_sha == gate_sha && c.ctrl_sha == ctrl_sha {
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

        let mut survived_here: Vec<String> = Vec::new();
        for (dir, _, _, survivors) in &scores {
            if !survivors.is_empty() {
                let shown: Vec<String> = survivors
                    .iter()
                    .map(|l| match equiv_lines.get(l) {
                        Some(why) => format!("{} (claims equivalent: {})", l, why),
                        None => l.to_string(),
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
            println!(
                "{:<30}{}  {}",
                name,
                cols,
                {
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
                }
            );
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
            assert_op,
            fresh,
            all,
            dir,
        } => mutate(
            only.as_deref(),
            *loud,
            *invert,
            *assert_op,
            *fresh,
            *all,
            dir.as_deref(),
        ),
        GatesCmd::Prs { repo } => prs(repo.as_deref()),
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
        assert_eq!(equivalence_claims(one).get(&2).map(String::as_str),
                   Some("guards force it"));

        let many = "# mutant-equivalent: proven below\n# line two\n# line three\n\
                    # line four\nif a > b:\n";
        let c = equivalence_claims(many);
        assert_eq!(c.get(&5).map(String::as_str), Some("proven below"),
                   "a multi-line proof lost its target: {c:?}");
        assert!(c.get(&3).is_none(), "named a line inside its own comment block");

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
        assert!(f.iter().any(|s| s.starts_with("flag --self-check")), "{f:?}");
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

        assert!(msg(&base.join("nope")).contains("--dir"), "a missing dir must name the flag");
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
        assert!(s("def f(a) -> int:\n").is_empty(), "took a return annotation");
        assert!(s("# a > b\n").is_empty(), "took a comment");
        assert!(s("x = 1  # a > b\n").is_empty(), "took a trailing comment");
        assert!(s("print(\"a > b\")\n").is_empty(), "took a string");
        assert!(s("print('a > b')\n").is_empty(), "took a single-quoted string");
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
            s("def f():\n    \"\"\"doc: a > b\n    more: c < d\n    \"\"\"\n    return 0\n").is_empty(),
            "took a comparison out of a function docstring"
        );
        assert!(s("x = \'\'\'a > b\'\'\'\n").is_empty(), "took a single-quoted triple");
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
}

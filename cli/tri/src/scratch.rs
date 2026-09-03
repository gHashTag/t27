//! `tri tests scratch` -- test binaries whose tests share one scratch directory.
//!
//! The shape, found in `bootstrap/tests/scaffold_c.rs`:
//!
//! ```ignore
//! let dir = temp_dir().join(format!("t27-scaffold-{}-{}", process::id(), src.len()));
//! std::fs::create_dir_all(&dir)?;
//! ...
//! let _ = std::fs::remove_dir_all(&dir);      // deletes the WHOLE directory
//! ```
//!
//! Every test in a binary shares the pid, so the key is really `src.len()`. Two
//! tests whose sources happen to be the same length compute the same path, and
//! each deletes it on the way out. Under the default parallel runner one erases
//! the spec another is mid-read of, the compiler prints nothing, and the
//! assertion reports an empty result.
//!
//! Measured on `scaffold_c.rs` by printing the paths of one run: six tests
//! produced THREE directories, four of them sharing one. The collision happens
//! every run; only the timing of the delete decides whether a test dies. It
//! failed roughly one run in three, and passed the first time it was written.
//!
//! TWO COLLISIONS, AND THIS DETECTOR SEES ONE. Everything below is about tests
//! inside ONE process. A second collision is between concurrent RUNS of the same
//! binary sharing `$TMPDIR`, and a key that separates threads does not separate
//! those: `verilog_real_arithmetic.rs` with a counter but no pid still failed
//! 29 of 64 runs with 16 copies going at once, and 0 of 64 with both. Nothing
//! here detects that; the printed advice now says so rather than recommending
//! against the pid.
//!
//! What this looks for is the CONJUNCTION -- more than one `#[test]`, a scratch
//! path under `temp_dir()`, a `remove_dir_all` of that path, and a key with no
//! per-call component. Any one of those alone is fine.
//!
//! Two things this deliberately does NOT flag, both checked by hand first:
//!
//!   * a key ending in a caller-supplied label (`format!("x_{}_{}", pid, label)`)
//!     -- distinct per test by construction, so `verilog_decl_hoist.rs` and
//!     `on_clock_plain_assign.rs` are clean;
//!   * `src.len()` used for something that is not a path, e.g.
//!     `String::with_capacity(src.len())` in `verilog_r_si_1.rs`. Grepping the
//!     symptom rather than the construct convicts that file wrongly.

use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct Finding {
    pub file: PathBuf,
    pub tests: usize,
    pub key: String,
}

/// The per-call components that make a scratch path unique per invocation.
const PER_CALL: [&str; 3] = ["AtomicUsize", "SystemTime", "thread::current"];

pub fn scan(root: &Path) -> Result<Vec<Finding>> {
    let mut out = Vec::new();
    let mut dirs = vec![root.join("bootstrap/tests"), root.join("cli/tri/tests")];
    dirs.retain(|d| d.is_dir());
    for d in dirs {
        for e in std::fs::read_dir(&d)? {
            let p = e?.path();
            if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let s = std::fs::read_to_string(&p)?;
            if let Some(f) = judge(&p, &s) {
                out.push(f);
            }
        }
    }
    out.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(out)
}

/// Split out so the self-check can drive it on a string with no file on disk.
pub fn judge(path: &Path, s: &str) -> Option<Finding> {
    let tests = s.matches("#[test]").count();
    if tests < 2 || !s.contains("remove_dir_all") || !s.contains("temp_dir()") {
        return None;
    }
    if PER_CALL.iter().any(|m| s.contains(m)) {
        return None;
    }
    // The key is the format string of the first `temp_dir().join(format!(...))`.
    let i = s.find("temp_dir()")?;
    let j = s[i..].find("format!")? + i;
    let q = s[j..].find('"')? + j + 1;
    let end = s[q..].find('"')? + q;
    let key = s[q..end].to_string();
    // Judge the ARGUMENTS, not the braces.
    //
    // The first version asked whether the `format!` call contained a `{`, which
    // is true of every single-line format call ever written -- so a key
    // interpolating ONLY `process::id()` looked variable and slipped through.
    // `verilog_imported_enum.rs` was exactly that shape, and a probe asserting
    // the directory was fresh fired on it 8 runs out of 8 while this detector
    // reported nothing.
    //
    // A pid varies per PROCESS, and every test in a binary shares one. An
    // argument derived from the input (`src.len()`) varies per input, and two
    // inputs can agree. Neither varies per CALL, which is what a scratch
    // directory that its own user deletes requires.
    let args_end = s[end..].find(';').map(|k| k + end).unwrap_or(s.len());
    let args = &s[end + 1..args_end];
    // Inline captures count as arguments. `format!("x-{tag}")` has no argument
    // list at all, and reading only what follows the string would call it
    // constant -- `backend_behaviour.rs` is that shape, with a distinct `tag`
    // per test, and flagging it would be a false conviction.
    let inline: Vec<String> = key
        .split('{')
        .skip(1)
        .filter_map(|r| r.split('}').next())
        .map(|c| c.split(':').next().unwrap_or(c).trim().to_string())
        .filter(|c| !c.is_empty())
        .collect();
    let per_call = args
        .split(',')
        .map(str::trim)
        .map(str::to_string)
        .chain(inline)
        .filter(|a| !a.is_empty() && a != ")" && a != "))")
        .any(|a| !a.contains("process::id()") && !a.contains(".len()"));
    if per_call {
        return None;
    }
    Some(Finding {
        file: path.to_path_buf(),
        tests,
        key,
    })
}

pub fn advice() -> String {
    [
        "  Fix: an AtomicUsize counter AND the pid. Not either one, and not",
        "  any property of the input (two inputs can agree).",
        "",
        "  The two are for different collisions, and this used to say \"not",
        "  the pid\". Measured on bootstrap/tests/verilog_real_arithmetic.rs,",
        "  release build, four arms:",
        "",
        "      key                  1 process, 4 threads   16 processes",
        "      neither                        6 / 150         41 / 64",
        "      process::id only               7 / 150          0 / 64",
        "      counter only                   0 / 150         29 / 64",
        "      both                           0 / 150          0 / 64",
        "",
        "  The counter separates the THREADS of one run; the pid separates",
        "  concurrent RUNS, which is not hypothetical -- two agents, two",
        "  worktrees, or a `cargo test` beside a manual run share $TMPDIR.",
        "  This detector only looks for the first collision. The middle two",
        "  rows are what each half alone costs.",
    ]
    .join("\n")
}

pub fn run(gate: bool, self_check: bool) -> Result<()> {
    if self_check {
        return run_self_check();
    }
    let root = crate::find_trinity_root()?;
    let found = scan(&root)?;

    println!();
    println!("  test binaries whose tests share one scratch directory");
    println!();
    if found.is_empty() {
        println!("      none");
    }
    for f in &found {
        println!(
            "      {:<44} {} tests   key {}",
            f.file.strip_prefix(&root).unwrap_or(&f.file).display(),
            f.tests,
            f.key
        );
    }
    println!();
    println!("  A shared scratch directory is not a slow test, it is a WRONG one:");
    println!("  each test deletes the whole directory, so one erases the input");
    println!("  another is mid-read of and the assertion reads an empty result.");
    println!("  It passes the first time it runs. A green run does not clear it --");
    println!("  print the paths of a single run and count the distinct ones.");
    println!();
    println!("{}", advice());
    println!();

    if gate && !found.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

/// Negative control: the gate must SEE a planted collision, and must not fire on
/// the fixed shape. A gate that cannot fail is not a gate.
fn run_self_check() -> Result<()> {
    let bad = r#"
        #[test] fn a() {}
        #[test] fn b() {}
        fn g(src: &str) {
            let dir = std::env::temp_dir().join(format!("x-{}-{}", std::process::id(), src.len()));
            std::fs::remove_dir_all(&dir);
        }
    "#;
    let good = r#"
        #[test] fn a() {}
        #[test] fn b() {}
        fn g(src: &str) {
            static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!("x-{}-{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
            std::fs::remove_dir_all(&dir);
        }
    "#;
    let one = r#"
        #[test] fn a() {}
        fn g(src: &str) {
            let dir = std::env::temp_dir().join(format!("x-{}-{}", std::process::id(), src.len()));
            std::fs::remove_dir_all(&dir);
        }
    "#;
    // Inline capture: no argument list at all, and `tag` differs per test.
    // Reading only what follows the format string calls this constant and
    // convicts a clean file.
    let inline = r#"
        #[test] fn a() {}
        #[test] fn b() {}
        fn g(tag: &str) {
            let dir = std::env::temp_dir().join(format!("x-{tag}"));
            std::fs::remove_dir_all(&dir);
        }
    "#;
    // Key interpolating ONLY the pid: every test in a binary shares it.
    let pid_only = r#"
        #[test] fn a() {}
        #[test] fn b() {}
        fn g() {
            let dir = std::env::temp_dir().join(format!("x-{}", std::process::id()));
            std::fs::remove_dir_all(&dir);
        }
    "#;
    let p = Path::new("probe.rs");
    let mut bad_seen = judge(p, bad).is_some();
    let good_seen = judge(p, good).is_some();
    let one_seen = judge(p, one).is_some();
    let inline_seen = judge(p, inline).is_some();
    let pid_only_seen = judge(p, pid_only).is_some();

    println!();
    println!("  self-check (negative control)");
    println!();
    println!("      planted collision seen          {}", yn(bad_seen));
    println!("      counter-keyed NOT flagged       {}", yn(!good_seen));
    println!("      single-test file NOT flagged    {}", yn(!one_seen));
    println!("      inline-capture key NOT flagged  {}", yn(!inline_seen));
    println!("      pid-only key seen               {}", yn(pid_only_seen));
    println!();

    if !bad_seen || good_seen || one_seen || inline_seen || !pid_only_seen {
        println!("  THE CONTROL FAILED. This gate cannot be trusted to see the defect");
        println!("  it exists for, so a clean run of it claims nothing.");
        println!();
        bad_seen = false;
    }
    if !bad_seen {
        std::process::exit(1);
    }
    Ok(())
}

fn yn(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "NO"
    }
}

#[cfg(test)]
mod advice_tests {
    /// The advice is the deliverable, so it is pinned like one. It used to read
    /// "an AtomicUsize counter, NOT the pid", and the four-arm table it now
    /// carries is what that cost: counter-only left 29 of 64 concurrent runs
    /// failing. Both components are required, and this asserts the text says so.
    #[test]
    fn the_advice_asks_for_both_components_and_not_one_instead_of_the_other() {
        let a = super::advice();
        assert!(a.contains("AtomicUsize"), "the per-call half is missing:\n{a}");
        assert!(a.contains("pid"), "the per-process half is missing:\n{a}");
        assert!(
            a.contains("AND"),
            "both are required; the text must not offer a choice:\n{a}"
        );
        // The exact sentence that was wrong. Anywhere in the text, in any
        // casing, it is the old advice coming back.
        let lower = a.to_lowercase();
        assert!(
            !lower.contains("not by the pid") && !lower.contains("not the pid"),
            "the old advice is back:\n{a}"
        );
    }

    /// The numbers are the reason the sentence changed, so losing them turns the
    /// advice back into an assertion. All four arms, or none of this is evidence.
    #[test]
    fn the_advice_carries_the_measurement_that_settled_it() {
        let a = super::advice();
        for arm in ["6 / 150", "41 / 64", "0 / 150", "29 / 64", "0 / 64"] {
            assert!(a.contains(arm), "missing the {arm} arm:\n{a}");
        }
    }
}

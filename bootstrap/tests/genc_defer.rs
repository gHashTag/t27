//! #7352: gen-c wrote `defer S;` as the comment `/* unsupported: StmtExpr */;`,
//! exited 0, and the deferred statement never ran. A test that depended on it
//! trapped at run time; one that did not passed while the cleanup it claimed
//! never happened.
//!
//! gen-c now writes the deferred statement at every exit of its block, the way
//! Zig runs it: at the fall-through end, before a `return` (after the returned
//! value is evaluated), before a `break` or `continue` of the loop whose body
//! declared it, innermost block first and each block in reverse order of
//! declaration. `errdefer` is refused by name and line, never dropped.
//!
//! Every test compiles the generated C with `-DT27_TEST_MAIN` and RUNS it: the
//! defect compiled cleanly, so only running the C shows the call happened.
//!
//! Part of #5980.

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let d = std::env::temp_dir().join(format!(
            "t27c-genc-defer-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("create scratch directory");
        Self(d)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn gen_c_raw(s: &Scratch, src: &str) -> Output {
    let p = s.0.join("in.t27");
    std::fs::write(&p, src).expect("write spec");
    Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&p)
        .output()
        .expect("run t27c gen-c")
}

fn gen_c(s: &Scratch, src: &str) -> String {
    let out = gen_c_raw(s, src);
    assert!(
        out.status.success(),
        "gen-c failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn cc_present() -> bool {
    Command::new("cc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Generate C for `src`, refuse an `unsupported` comment, then compile it with
/// the built-in test runner and run it. Returns the C. The run is skipped
/// where there is no cc; the text checks are not.
fn gen_and_run(src: &str) -> String {
    let s = Scratch::new();
    let c = gen_c(&s, src);
    assert!(
        !c.contains("unsupported"),
        "gen-c wrote a statement as `unsupported`:\n{}",
        c
    );
    if !cc_present() {
        eprintln!("cc not found; skipping the run");
        return c;
    }
    let cp = s.0.join("out.c");
    std::fs::write(&cp, &c).expect("write C");
    let bin = s.0.join("out");
    let built = Command::new("cc")
        .args(["-std=gnu11", "-DT27_TEST_MAIN"])
        .arg(&cp)
        .arg("-o")
        .arg(&bin)
        .output()
        .expect("run cc");
    assert!(
        built.status.success(),
        "cc refused the generated C:\n{}\n--- C ---\n{}",
        String::from_utf8_lossy(&built.stderr),
        c
    );
    let ran = Command::new(&bin).output().expect("run the generated C");
    assert!(
        ran.status.success(),
        "the generated C failed its own tests ({}):\n{}{}\n--- C ---\n{}",
        ran.status,
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr),
        c
    );
    c
}

/// The issue's reproduction, verbatim.
const ISSUE_REPRO: &str = "module m;\nvar n: i64 = 0;\nfn inc() -> void { n = n + 1; }\n\
fn f() -> void {\n    defer inc();\n}\ntest t { f(); assert(n == 1); }\n";

#[test]
fn the_issue_repro_runs_the_deferred_call() {
    gen_and_run(ISSUE_REPRO);
}

/// `a` then `b` appended as digits: 21 means `b` ran first.
const TRACE: &str = "module m;\nvar trace: i64 = 0;\n\
fn a() -> void { trace = trace * 10 + 1; }\n\
fn b() -> void { trace = trace * 10 + 2; }\n";

#[test]
fn two_defers_run_in_reverse_order() {
    gen_and_run(&format!(
        "{TRACE}fn f() -> void {{\n    defer a();\n    defer b();\n}}\n\
         test t {{ f(); assert(trace == 21); }}\n"
    ));
}

#[test]
fn an_inner_block_runs_before_the_outer_one_on_return() {
    gen_and_run(&format!(
        "{TRACE}fn f(x: i64) -> void {{\n    defer a();\n    if (x > 0) {{\n        defer b();\n        return;\n    }}\n}}\n\
         test t {{ f(1); assert(trace == 21); f(0); assert(trace == 211); }}\n"
    ));
}

const COUNTER: &str = "module m;\nvar n: i64 = 0;\nfn inc() -> void { n = n + 1; }\n";

#[test]
fn an_early_return_runs_the_deferred_call() {
    gen_and_run(&format!(
        "{COUNTER}fn g(x: i64) -> i64 {{\n    defer inc();\n    if (x > 0) {{\n        return x;\n    }}\n    return 0;\n}}\n\
         test t {{ assert(g(5) == 5); assert(n == 1); assert(g(0) == 0); assert(n == 2); }}\n"
    ));
}

#[test]
fn the_returned_value_is_read_before_the_deferred_call_runs() {
    gen_and_run(
        "module m;\nvar n: i64 = 0;\nfn reset() -> void { n = 0; }\n\
         fn h() -> i64 {\n    n = 7;\n    defer reset();\n    return n;\n}\n\
         test t { assert(h() == 7); assert(n == 0); }\n",
    );
}

#[test]
fn break_runs_the_loop_body_defer() {
    gen_and_run(&format!(
        "{COUNTER}fn f() -> i64 {{\n    var i: i64 = 0;\n    while (i < 10) {{\n        defer inc();\n        i = i + 1;\n        if (i == 3) {{\n            break;\n        }}\n    }}\n    return i;\n}}\n\
         test t {{ assert(f() == 3); assert(n == 3); }}\n"
    ));
}

#[test]
fn continue_runs_the_loop_body_defer() {
    gen_and_run(&format!(
        "{COUNTER}fn f() -> i64 {{\n    var i: i64 = 0;\n    var skipped: i64 = 0;\n    while (i < 4) {{\n        defer inc();\n        i = i + 1;\n        if (i < 10) {{\n            continue;\n        }}\n        skipped = 1;\n    }}\n    return skipped;\n}}\n\
         test t {{ assert(f() == 0); assert(n == 4); }}\n"
    ));
}

#[test]
fn a_loop_defer_does_not_run_on_the_fn_return_twice() {
    // The loop body's scope is closed when the loop ends, so the `return`
    // after it runs only the fn's own defer.
    gen_and_run(&format!(
        "{TRACE}fn f() -> void {{\n    defer a();\n    var i: i64 = 0;\n    while (i < 2) {{\n        defer b();\n        i = i + 1;\n    }}\n    return;\n}}\n\
         test t {{ f(); assert(trace == 221); }}\n"
    ));
}

#[test]
fn errdefer_is_refused_by_name_and_line() {
    let s = Scratch::new();
    let out = gen_c_raw(
        &s,
        "module m;\nvar n: i64 = 0;\nfn inc() -> void { n = n + 1; }\n\
         fn f() -> void {\n    errdefer inc();\n}\n",
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "gen-c accepted an errdefer it cannot lower:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        stderr.contains("`errdefer` at line 5"),
        "the refusal must name `errdefer` and its line:\n{}",
        stderr
    );
}

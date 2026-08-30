//! A statement that lowered to nothing became a bare `;`, and Zig has no empty
//! statement.
//!
//!     /tmp/x.zig:3:5: error: expected statement, found ';'
//!
//! So one such line does not merely add noise — it kills the whole file, and
//! every check inside it with it. Measured: **491 bare `;` lines across 81
//! generating specs**.
//!
//! Two source forms reach the same emitted construct. A `bench` block's prose
//! body arrives as a childless `StmtExpr` through `gen_bench_block`'s fallback;
//! and `defer <call>;` in a test body arrives WITH a child that `gen_expr`
//! renders as nothing — the parser keeps it deliberately, because "a dropped
//! `defer` silently removes a release, a close or a free".
//!
//! The remedy is the one the invariant path beside it already uses (T43): say
//! the body was not lowered rather than emit something that claims to be a
//! statement. Verilog already emits benches as comments; Zig was the only
//! backend where a bare `;` is fatal rather than merely useless.
//!
//! Measured per spec, both directions, zig 0.16.0, no timeouts:
//!
//!     zig test --test-no-exec    165 -> 190   +25   regressions 0
//!     zig build-obj              282 -> 308   +26   regressions 0
//!
//! Bare `;` lines: 491 -> 0. Notices emitted: 491. One for one — nothing lost
//! and nothing invented.
//!
//! Replacing a bench body with a notice is equivalent to deleting it, which is
//! normally the inflation trap. It is sound here and the justification was
//! re-derived rather than inherited: **780 `bench_*` functions are defined in
//! the generated corpus and 0 have a call site**, and `zig test` runs only
//! `test {}` blocks, so emptying them cannot lose an executed check.

use std::io::Write;
use std::process::Command;

fn gen_zig(src: &str) -> String {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("t27-baresemi-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("m.t27");
    let mut f = std::fs::File::create(&path).expect("write spec");
    f.write_all(src.as_bytes()).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen")
        .arg(&path)
        .output()
        .expect("run t27c");
    let _ = std::fs::remove_dir_all(&dir);
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// The real corpus spelling, taken from `compiler/cli/gen.t27:499`. A `bench`
/// takes no braces, and `target: < 100ms` is the prose line that becomes a
/// childless `StmtExpr` — which is where the bare `;` came from.
///
/// The first version of this fixture invented `bench "b" { measure: ... }` from
/// the issue's prose. It does not parse, `t27c` exits 1, and the test asserted
/// on an empty string. Read the construct out of the corpus, not out of the
/// report about it.
const WITH_BENCH: &str = "module m\n\nfn f(x: u32) -> u32 {\n    return x + 1\n}\n\nbench f_latency\n    target: < 100ms\n    _ = f;\n";

/// The second source form reaching the same emitted construct: `defer <call>;`
/// in a test body. The parser keeps it deliberately — "a dropped `defer`
/// silently removes a release, a close or a free" — as a StmtExpr carrying a
/// statement child that `gen_expr` renders as nothing.
const WITH_DEFER: &str = "module m\n\nfn free(x: u32) -> u32 {\n    return x\n}\n\ntest t {\n    const v = 1;\n    defer free(v);\n    assert_eq(v, 1);\n}\n";

/// The whole point: no line in the output may be a lone `;`.
#[test]
fn no_line_is_a_bare_semicolon() {
    let z = gen_zig(WITH_BENCH);
    let offenders: Vec<_> = z.lines().filter(|l| l.trim() == ";").collect();
    assert!(
        offenders.is_empty(),
        "a bare `;` is not a Zig statement:\n{z}"
    );
}

/// And what replaced it says what was dropped, rather than leaving a silent gap.
#[test]
fn the_notice_names_what_was_not_lowered() {
    let z = gen_zig(WITH_BENCH);
    assert!(
        z.contains("NOT LOWERED:"),
        "expected a notice where the statement was:\n{z}"
    );
    assert!(
        z.contains("(T43)"),
        "the notice should carry the same tag as the invariant path beside it:\n{z}"
    );
}

/// A statement that DOES lower keeps its semicolon.
///
/// The first version of this test used `y = x + 1`, which reaches `StmtAssign`
/// and never touches the arm under test. It **survived the mutation that drops
/// every semicolon**: a test of an adjacent path that happens to hold.
///
/// A bare CALL statement is the one that goes through `StmtExpr` with a
/// non-empty rendering, so this is the case that separates "rendered nothing"
/// from "rendered something" — and it is the case the mutant destroys.
#[test]
fn a_call_statement_keeps_its_semicolon_and_is_not_a_notice() {
    let src = "module m\n\nfn side(x: u32) -> u32 {\n    return x\n}\n\ntest t {\n    side(1);\n    assert_eq(side(2), 2);\n}\n";
    let z = gen_zig(src);
    assert!(
        z.lines().any(|l| l.trim() == "side(1);"),
        "a bare call statement lost its semicolon or became a notice:\n{z}"
    );
    assert!(
        !z.contains("NOT LOWERED: ExprCall"),
        "a call that renders perfectly well was reported as not lowered:\n{z}"
    );
}

/// The notice must be a COMMENT. A notice emitted as code would replace one
/// syntax error with another, and the file would still not compile.
#[test]
fn the_notice_is_a_comment() {
    let z = gen_zig(WITH_BENCH);
    for line in z.lines().filter(|l| l.contains("NOT LOWERED:")) {
        assert!(
            line.trim_start().starts_with("//"),
            "the notice must be commented out, got: {line}"
        );
    }
}

/// Both source forms are covered, and the notice distinguishes them: a childless
/// statement is labelled `empty statement`, a statement whose child rendered to
/// nothing is labelled by that child's kind. Corpus-wide the split is 475 and 16,
/// summing to the 491 bare `;` lines the fix removed.
#[test]
fn the_defer_form_is_covered_too_and_labelled_differently() {
    let bench = gen_zig(WITH_BENCH);
    let defer = gen_zig(WITH_DEFER);
    for (name, z) in [("bench", &bench), ("defer", &defer)] {
        assert!(
            !z.lines().any(|l| l.trim() == ";"),
            "a bare `;` survived the {name} form:\n{z}"
        );
    }
    // A childless statement has no child to name; one whose CHILD rendered to
    // nothing does. A label that said the same thing for both would send the
    // reader to the wrong construct, and it survived the first mutation run.
    assert!(
        bench.contains("NOT LOWERED: empty statement"),
        "the childless form should say so:\n{bench}"
    );
    let defer_label = defer
        .lines()
        .find(|l| l.contains("NOT LOWERED:"))
        .unwrap_or_else(|| panic!("no notice in the defer form:\n{defer}"));
    assert!(
        !defer_label.contains("empty statement"),
        "a statement WITH a child must not be labelled as empty: {defer_label}"
    );
}

/// End to end: the generated file is accepted by Zig. Everything above is a
/// statement about the text; this is the statement about the language, and it
/// is the one that failed before.
#[test]
fn zig_accepts_the_generated_file() {
    let zig_ok = Command::new("zig")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !zig_ok {
        eprintln!("zig not on PATH -- SKIPPED, and saying so rather than passing silently");
        return;
    }
    let z = gen_zig(WITH_BENCH);
    let dir = std::env::temp_dir().join(format!("t27-baresemi-zig-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let f = dir.join("m.zig");
    std::fs::write(&f, &z).expect("write zig");
    let out = Command::new("zig")
        .args(["build-obj", "-fno-emit-bin"])
        .arg(&f)
        .current_dir(&dir)
        .env("ZIG_GLOBAL_CACHE_DIR", dir.join("cache"))
        .output()
        .expect("run zig");
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success(),
        "zig rejected the generated file:\n{err}\n--- source ---\n{z}"
    );
}

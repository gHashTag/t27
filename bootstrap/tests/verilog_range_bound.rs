// =============================================================================
// A range is not a bound.
//
// `gen_verilog_for_stmt` wrote the whole iterable where the loop's comparison
// belongs, so `for (0..1000) |_| { … }` became
//
//     for (i = 0; i < (0 .. 1000); i = i + 1) begin
//
// which `iverilog` answers with `syntax error`. Measured by regenerating the
// corpus with the stock compiler and with this one, counting files whose `for (`
// line carries a `..`: 36 -> 0 in the simulation path, 5 -> 0 synthesizable.
//
// #2849 solved exactly this for the C emitter and recorded the trap in its own
// comment: the range is an `ExprBinary` whose `extra_op` is `".."`, NOT the
// `ExprRange` variant, which is declared in `NodeKind` and constructed nowhere.
// The repair did not travel. Rust tests the same shape; Zig writes the range
// verbatim and is correct, because Zig has ranges.
//
// Closes #2997
// =============================================================================

use std::io::Write;
use std::process::Command;

/// One module with all four shapes, so a single emission answers every question
/// and no answer can be read off another's absence:
///   * a range with a NAMED capture   -- the name must survive
///   * a range with `_`               -- the counter must match its declaration
///   * a range whose end is an EXPRESSION, not a literal
///   * a non-range iterable           -- must keep the old lowering
const SPEC: &str = r#"module RangeBound {
    fn sum_to(n: u32) -> u32 {
        var acc: u32 = 0;
        for (0..n) |k| {
            acc = acc + k;
        }
        return acc;
    }

    fn count_anon() -> u32 {
        var c: u32 = 0;
        for (0..5) |_| {
            c = c + 1;
        }
        return c;
    }

    fn count_from(lo: u32, hi: u32) -> u32 {
        var c: u32 = 0;
        for (lo..hi) |j| {
            c = c + 1;
        }
        return c;
    }

    fn total(data: [u32; 4]) -> u32 {
        var s: u32 = 0;
        for (data) |x| {
            s = s + x;
        }
        return s;
    }

    fn sum_binary(lo: u32, hi: u32) -> u32 {
        var s: u32 = 0;
        for (lo + hi) |b| {
            s = s + 1;
        }
        return s;
    }
}

test sum_to_five_is_ten {
    assert_eq(sum_to(5), 10);
}

test count_anon_is_five {
    assert_eq(count_anon(), 5);
}

test count_from_two_to_seven_is_five {
    assert_eq(count_from(2, 7), 5);
}
"#;

fn emit(subcommand: &str) -> String {
    let bin = env!("CARGO_BIN_EXE_t27c");
    // Keyed per subcommand: `icarus-simulate` writes its scratch file into the
    // shared temp dir under the spec's BASENAME, so two probes sharing a stem
    // overwrite each other's Verilog.
    let spec_path = std::env::temp_dir().join(format!("t27_range_bound_{subcommand}.t27"));
    let mut f = std::fs::File::create(&spec_path).expect("create probe spec");
    f.write_all(SPEC.as_bytes()).expect("write probe spec");
    drop(f);

    let out = Command::new(bin)
        .args([subcommand, spec_path.to_str().expect("utf-8 path")])
        .output()
        .expect("run t27c");
    assert!(
        out.status.success(),
        "t27c {subcommand} exited {:?}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Every `for (` line in the emitted module, so an assertion cannot be satisfied
/// by a line belonging to some other construct.
fn for_lines(verilog: &str) -> Vec<String> {
    verilog
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("for ("))
        .map(str::to_string)
        .collect()
}

#[test]
fn no_range_literal_reaches_a_loop_bound() {
    let verilog = emit("gen-verilog");
    for l in for_lines(&verilog) {
        assert!(
            !l.contains(".."),
            "a range reached a bound -- iverilog answers `syntax error`: {l}"
        );
    }
}

#[test]
fn a_named_capture_keeps_its_spelling() {
    let verilog = emit("gen-verilog");
    let lines = for_lines(&verilog);
    assert!(
        lines.iter().any(|l| l.contains("for (k = 0; k < n;")),
        "the body reads `k`, so the counter must be `k`; got:\n{lines:#?}"
    );
}

/// The counter must be the identifier that was DECLARED for this loop, and the
/// declaration is hoisted to the top of the function body by a different
/// function. The first version of this change renamed a `_` capture to
/// `__t27_i` at the loop and left the declaration saying `integer _;` --
/// `iverilog` answered `register `__t27_i' unknown`, and the corpus could not
/// have shown it, because all 36 carrier files already failed to elaborate on
/// the very defect being repaired.
#[test]
fn the_counter_is_the_identifier_that_was_declared() {
    let verilog = emit("gen-verilog");
    for l in for_lines(&verilog) {
        // `for (<v> = ...` -- pull <v> out and require a matching declaration.
        let v = l
            .trim_start_matches("for (")
            .split(|c: char| c == ' ' || c == '=')
            .next()
            .unwrap_or("")
            .to_string();
        assert!(!v.is_empty(), "could not read a counter out of: {l}");
        assert!(
            verilog.contains(&format!("integer {v};")),
            "the loop uses `{v}` and nothing declares it; line was: {l}"
        );
    }
}

#[test]
fn the_start_is_the_ranges_start_not_zero() {
    // `for (lo..hi)` must not be lowered as `for (j = 0; j < hi; …)`. The old
    // code always started at 0 because it never looked at the range at all.
    let verilog = emit("gen-verilog");
    let lines = for_lines(&verilog);
    assert!(
        lines.iter().any(|l| l.contains("for (j = lo; j < hi;")),
        "a range that does not start at zero must keep its start; got:\n{lines:#?}"
    );
}

#[test]
fn a_non_range_iterable_keeps_the_old_lowering() {
    // The guard must be narrow. Anything that is not an `ExprBinary` with
    // `extra_op == ".."` still goes through the iterable path, which announces
    // itself in a comment.
    let verilog = emit("gen-verilog");
    assert!(
        verilog.contains("// for-each over iterable"),
        "the non-range path must still exist; emitted:\n{verilog}"
    );
}

/// The guard has three clauses and this is the only test that reaches the third.
///
/// `for (data) |x|` iterates an IDENTIFIER -- zero children -- so a mutation
/// that drops the `extra_op == ".."` test and keeps only `children.len() == 2`
/// leaves it untouched, and the first version of this file could not tell the
/// two apart. `lo + hi` is a two-child `ExprBinary` that is NOT a range: under
/// that mutation it lowers as `for (b = lo; b < hi; …)`, silently iterating
/// something the source never asked for.
#[test]
fn a_two_child_binary_that_is_not_a_range_is_not_treated_as_one() {
    let verilog = emit("gen-verilog");
    let lines = for_lines(&verilog);
    assert!(
        lines
            .iter()
            .any(|l| l.contains("for (b = 0; b < (lo + hi);")),
        "`lo + hi` is not a range and must keep the iterable lowering; got:\n{lines:#?}"
    );
    assert!(
        !lines.iter().any(|l| l.contains("for (b = lo;")),
        "`lo + hi` was read as the range `lo .. hi`; got:\n{lines:#?}"
    );
}

/// The end this was fixed for: the spec's own declared tests, run.
///
/// `icarus-simulate` is the only ruler in this repository that RUNS generated
/// Verilog. Before this change the probe did not elaborate at all.
#[test]
fn the_specs_own_tests_pass_under_the_simulator() {
    // Ask the OS whether the simulator exists rather than matching a phrase in
    // whatever the failure printed -- a message is the tool's to change and
    // PATH is not (ci-gates 426).
    if Command::new("iverilog").arg("-V").output().is_err() {
        eprintln!("iverilog is not on PATH; skipping the runtime leg (nothing is claimed)");
        return;
    }

    let bin = env!("CARGO_BIN_EXE_t27c");
    let spec_path = std::env::temp_dir().join("t27_range_bound_sim.t27");
    let mut f = std::fs::File::create(&spec_path).expect("create probe spec");
    f.write_all(SPEC.as_bytes()).expect("write probe spec");
    drop(f);

    let out = Command::new(bin)
        .args(["icarus-simulate", spec_path.to_str().expect("utf-8 path")])
        .output()
        .expect("run t27c icarus-simulate");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    for t in [
        "sum_to_five_is_ten",
        "count_anon_is_five",
        "count_from_two_to_seven_is_five",
    ] {
        assert!(
            log.contains(&format!("[TEST] {t} : PASSED")),
            "{t} did not pass; log was:\n{log}"
        );
    }
}

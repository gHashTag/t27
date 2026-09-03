// =============================================================================
// A real value must not be routed through an integer container in gen-verilog.
//
// Two sites, one class, both found by pointing the repository's own deep ruler
// (`t27c icarus-simulate`) at a spec for the first time:
//
//   1. `gen_verilog_expr` sent EVERY `*` through `__mul_noop`, the shift-and-add
//      ladder added by #741 so that synthesizable RTL carries no `*` operator
//      (rule R-SI-1). The ladder is declared `input [63:0]`, so a float operand
//      is rounded before use: `__mul_noop(0.3, 10.0)` is 0, and `ewma_step`
//      returned 5.0 where C, Rust, Zig and hand arithmetic all say 6.5.
//
//   2. A `given` binding was declared `reg [63:0] e;` whatever its initializer
//      returned, so `e = ewma_step(0.5, 0.5, 1.0)` stored 1 instead of 0.75 and
//      the test reported FAILED against a function that was by then correct.
//
// R-SI-1 governs SYNTHESIZABLE RTL. Verilog `real` is a simulation-only type no
// synthesis flow accepts, so a real multiply is outside that rule's subject.
// The integer path is unchanged, and `verilog_r_si_1.rs` is the control for it.
//
// Refs #741
// =============================================================================

use std::io::Write;
use std::process::Command;

/// Float parameters, a float literal, and an integer multiply in one module, so
/// one emission answers both questions and neither answer can be read off the
/// other's absence.
const SPEC: &str = r#"module RealArith {
    fn ewma(est: f64, alpha: f64, sample: f64) -> f64 {
        return (alpha * sample) + ((1.0 - alpha) * est);
    }

    fn scale_int(row: u32, cols: u32) -> u32 {
        return row * cols;
    }

    fn two_literals() -> f64 {
        return 2.5 * 4.0;
    }
}

test real_binding_keeps_its_fraction
    given e = ewma(0.5, 0.5, 1.0)
    then e == 0.75

test int_binding_stays_integer
    given n = scale_int(3, 4)
    then n == 12
"#;

/// Emit Verilog for `SPEC` with the given subcommand.
///
/// It PANICS rather than returning when the binary fails. A test that returns
/// quietly on a broken front end reports PASSED while measuring nothing -- the
/// exact shape this file exists to catch in the emitter.

/// Two axes, and a key needs BOTH. Measured on this file, release build:
///
/// | key                | one process, 4 threads | 16 concurrent processes |
/// |--------------------|------------------------|-------------------------|
/// | neither            |               6 / 150  |                41 / 64  |
/// | `process::id` only |               7 / 150  |                 0 / 64  |
/// | counter only       |               0 / 150  |                29 / 64  |
/// | both               |               0 / 150  |                 0 / 64  |
///
/// The counter separates the THREADS of one run -- six tests here call
/// `emit("gen-verilog")`, so six writers share one path. The pid separates
/// concurrent RUNS, which is not hypothetical: two agents, two worktrees, or a
/// `cargo test` beside a manual run all share `$TMPDIR`.
///
/// `tri harness scratch` advises "an AtomicUsize counter, not the pid". The
/// first half is right and the second is what the middle row of that table
/// costs.
fn unique() -> String {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

fn emit(subcommand: &str) -> String {
    let bin = env!("CARGO_BIN_EXE_t27c");
    // Keyed by subcommand: `icarus-simulate` writes its scratch file into the
    // shared temp dir under the spec's BASENAME, so two probes sharing a stem
    // overwrite each other's Verilog.
    let spec_path = std::env::temp_dir().join(format!("t27_real_arith_{subcommand}_{}.t27", unique()));
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

/// The body of `ewma`, so an assertion cannot be satisfied by text belonging to
/// another function in the same module.
fn ewma_body(verilog: &str) -> &str {
    let start = verilog
        .find("function real ewma;")
        .expect("ewma must be emitted as a real function");
    let rest = &verilog[start..];
    let end = rest.find("endfunction").expect("ewma must be closed");
    &rest[..end]
}

fn scale_int_body(verilog: &str) -> &str {
    let start = verilog
        .find("function")
        .and_then(|_| verilog.find("scale_int;"))
        .expect("scale_int must be emitted");
    let rest = &verilog[start..];
    let end = rest.find("endfunction").expect("scale_int must be closed");
    &rest[..end]
}

#[test]
fn float_multiply_uses_the_operator_not_the_integer_ladder() {
    let verilog = emit("gen-verilog");
    let body = ewma_body(&verilog);
    assert!(
        !body.contains("__mul_noop("),
        "a real multiply must not go through the integer ladder; body was:\n{body}"
    );
    assert!(
        body.contains("(alpha * sample)"),
        "expected the operator form for a real multiply; body was:\n{body}"
    );
}

#[test]
fn float_literal_operand_also_counts_as_real() {
    let verilog = emit("gen-verilog");
    let body = ewma_body(&verilog);
    // `(1.0 - alpha) * est`: the left operand is a parenthesised expression, so
    // this only holds if the real test recurses through it.
    assert!(
        body.contains("* est)"),
        "the second multiply must use the operator too; body was:\n{body}"
    );
}

/// Neither operand is an identifier, so nothing but the literal's own spelling
/// can decide this one. Without it `2.5 * 4.0` goes through the ladder as
/// `round(2.5) * round(4.0)` = 8 instead of 10 -- and the first version of this
/// file did not reach the branch at all: a mutation that disabled the literal
/// arm left all six tests green.
#[test]
fn a_multiply_of_two_float_literals_is_real() {
    let verilog = emit("gen-verilog");
    assert!(
        verilog.contains("two_literals = (2.5 * 4.0)"),
        "a literal-only real multiply must use the operator; emitted:\n{verilog}"
    );
    assert!(
        !verilog.contains("__mul_noop(2.5"),
        "the ladder rounds both literals; emitted:\n{verilog}"
    );
}

#[test]
fn integer_multiply_still_uses_the_ladder() {
    let verilog = emit("gen-verilog");
    let body = scale_int_body(&verilog);
    assert!(
        body.contains("__mul_noop("),
        "R-SI-1 still governs integer multiplication; body was:\n{body}"
    );
    assert!(
        !body.contains("row * cols"),
        "an integer multiply must not emit a bare operator; body was:\n{body}"
    );
}

#[test]
fn real_given_binding_is_declared_real() {
    let verilog = emit("gen-verilog-for-simulation");
    assert!(
        verilog.contains("real e; // t27#1948 let binding"),
        "a binding of a real-returning call must be `real`, not an integer reg"
    );
    assert!(
        !verilog.contains("reg [63:0] e;"),
        "an integer reg rounds 0.75 to 1 and turns a correct function into a FAILED test"
    );
}

#[test]
fn integer_given_binding_is_still_a_reg() {
    let verilog = emit("gen-verilog-for-simulation");
    assert!(
        verilog.contains("n; // t27#1948 let binding"),
        "the integer binding must still be declared"
    );
    assert!(
        !verilog.contains("real n;"),
        "an integer binding must not become real; that is the other direction of the same defect"
    );
}

/// The end the two fixes were made for: the spec's own declared tests.
///
/// `icarus-simulate` is the only ruler in this repository that RUNS generated
/// Verilog. Before these two changes this probe reported FAILED on the real
/// binding and PASSED on the integer one -- and exited 1.
#[test]
fn the_specs_own_tests_pass_under_the_simulator() {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let spec_path = std::env::temp_dir().join(format!("t27_real_arith_sim_{}.t27", unique()));
    let mut f = std::fs::File::create(&spec_path).expect("create probe spec");
    f.write_all(SPEC.as_bytes()).expect("write probe spec");
    drop(f);

    // Ask the OS whether the simulator exists, rather than matching a phrase in
    // whatever the failure printed. The first version of this guard tested for
    // `"not found"`; the runner says `No such file or directory (os error 2)`,
    // so the guard never fired and the test failed on a machine with no
    // simulator -- a skip path written for one environment and never executed
    // in it. A guard clause you have not run is a comment.
    if Command::new("iverilog").arg("-V").output().is_err() {
        eprintln!("iverilog is not on PATH; skipping the runtime leg (nothing is claimed)");
        return;
    }

    let out = Command::new(bin)
        .args(["icarus-simulate", spec_path.to_str().expect("utf-8 path")])
        .output()
        .expect("run t27c icarus-simulate");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        log.contains("[TEST] real_binding_keeps_its_fraction : PASSED"),
        "the real binding must survive the round trip; log was:\n{log}"
    );
    assert!(
        !log.contains("real_binding_keeps_its_fraction : FAILED"),
        "log was:\n{log}"
    );
}

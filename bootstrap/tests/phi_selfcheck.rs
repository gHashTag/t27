//! Wave 35 -- R-SC-1 regression tests for the phi-invariant golden-identity
//! self-check emitter.
//!
//! Strategy: shell out to the built t27c binary via `CARGO_BIN_EXE_t27c`,
//! run `gen-phi-selfcheck` with assorted tolerances / wrap modes, capture
//! stdout, and assert structural invariants on the emitted SystemVerilog
//! text. No HDL toolchain is required to run these tests; IEEE-1800
//! conformance of the emitted block is downstream of the formatter, which
//! has dedicated unit tests in `bootstrap/src/phi_selfcheck.rs`.
//!
//! Closes #758.

use std::process::Command;

fn run_gen_phi_selfcheck(args: &[&str]) -> String {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-phi-selfcheck")
        .args(args)
        .output()
        .expect("failed to spawn t27c gen-phi-selfcheck");
    assert!(
        output.status.success(),
        "t27c gen-phi-selfcheck exited with {:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("t27c gen-phi-selfcheck produced non-UTF-8 output")
}

#[test]
fn emits_default_snippet_with_phi_localparam() {
    let v = run_gen_phi_selfcheck(&[]);
    assert!(
        v.contains("localparam real PHI = 1.6180339887498948482;"),
        "missing PHI localparam: {}",
        v
    );
    assert!(
        v.contains("localparam real GOLDEN_IDENTITY = PHI * PHI + 1.0 / (PHI * PHI);"),
        "missing GOLDEN_IDENTITY localparam"
    );
}

#[test]
fn emits_initial_begin_block_with_fatal() {
    let v = run_gen_phi_selfcheck(&[]);
    assert!(v.contains("initial begin"), "missing initial begin");
    assert!(v.contains("$fatal(1,"), "missing $fatal call");
    assert!(
        v.contains("Golden Identity violated"),
        "missing fatal message"
    );
    // Must close the initial block.
    assert!(v.lines().any(|l| l.trim() == "end"), "missing initial-block end");
}

#[test]
fn default_tolerance_window_is_2_99_to_3_01() {
    let v = run_gen_phi_selfcheck(&[]);
    assert!(v.contains("2.990000"), "missing lower bound: {}", v);
    assert!(v.contains("3.010000"), "missing upper bound: {}", v);
}

#[test]
fn custom_tolerance_is_propagated() {
    let v = run_gen_phi_selfcheck(&["--tolerance", "0.005"]);
    assert!(v.contains("2.995000"), "missing custom lower bound: {}", v);
    assert!(v.contains("3.005000"), "missing custom upper bound: {}", v);
}

#[test]
fn zero_tolerance_falls_back_to_default() {
    let v = run_gen_phi_selfcheck(&["--tolerance", "0"]);
    assert!(v.contains("2.990000"), "expected fallback lower bound");
    assert!(v.contains("3.010000"), "expected fallback upper bound");
}

#[test]
fn negative_tolerance_falls_back_to_default() {
    // Use `=` form so clap doesn't treat `-0.5` as a short flag bundle.
    let v = run_gen_phi_selfcheck(&["--tolerance=-0.5"]);
    assert!(v.contains("2.990000"), "expected fallback lower bound");
    assert!(v.contains("3.010000"), "expected fallback upper bound");
}

#[test]
fn bare_snippet_has_no_module_or_formal_guard() {
    let v = run_gen_phi_selfcheck(&[]);
    assert!(!v.contains("`ifdef FORMAL"), "bare mode must not emit FORMAL guard");
    assert!(!v.contains("module "), "bare mode must not emit module header");
    assert!(!v.contains("endmodule"), "bare mode must not emit endmodule");
}

#[test]
fn wrap_emits_formal_guarded_module() {
    let v = run_gen_phi_selfcheck(&["--wrap", "phi_top"]);
    assert!(v.starts_with("`ifdef FORMAL"), "wrap must open with FORMAL guard");
    assert!(v.contains("module phi_top ();"), "wrap must declare module");
    assert!(v.contains("endmodule"), "wrap must close module");
    assert!(
        v.trim_end().ends_with("`endif // FORMAL"),
        "wrap must close FORMAL guard"
    );
    // The phi snippet must still be present inside the wrapper.
    assert!(v.contains("localparam real PHI = 1.6180339887498948482;"));
    assert!(v.contains("$fatal(1,"));
}

#[test]
fn wrap_indents_inner_body_with_four_spaces() {
    let v = run_gen_phi_selfcheck(&["--wrap", "phi_top"]);
    let mut saw_indented_phi = false;
    let mut saw_indented_initial = false;
    for line in v.lines() {
        if line.starts_with("    localparam real PHI") {
            saw_indented_phi = true;
        }
        if line.starts_with("    initial begin") {
            saw_indented_initial = true;
        }
    }
    assert!(saw_indented_phi, "expected indented PHI localparam");
    assert!(saw_indented_initial, "expected indented initial begin");
}

#[test]
fn output_is_ascii_only() {
    let bare = run_gen_phi_selfcheck(&[]);
    assert!(bare.is_ascii(), "bare output must be ASCII (L3)");
    let wrapped = run_gen_phi_selfcheck(&["--wrap", "phi_top"]);
    assert!(wrapped.is_ascii(), "wrapped output must be ASCII (L3)");
}

#[test]
fn output_file_is_written_when_requested() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "t27_phi_selfcheck_test_{}.sv",
        std::process::id()
    ));
    let path_str = path.to_string_lossy().to_string();
    let bin = env!("CARGO_BIN_EXE_t27c");
    let status = Command::new(bin)
        .arg("gen-phi-selfcheck")
        .arg("--output")
        .arg(&path_str)
        .status()
        .expect("failed to spawn t27c gen-phi-selfcheck (output)");
    assert!(status.success(), "expected success when writing output");
    let contents = std::fs::read_to_string(&path).expect("output file should exist");
    assert!(contents.contains("localparam real PHI = 1.6180339887498948482;"));
    assert!(contents.contains("$fatal(1,"));
    let _ = std::fs::remove_file(&path);
}

//! Wave 32 -- R-TS-1 regression tests for the trit stdlib emitter.
//!
//! Strategy: shell out to the built t27c binary via `CARGO_BIN_EXE_t27c`,
//! run `gen-trit-stdlib`, capture stdout, and assert structural truth-table
//! invariants on the emitted Verilog text. No HDL toolchain (Yosys, iverilog)
//! is required to run these tests; the synthesizability of the output is
//! validated by the FPGA E2E CI pipeline (fpga-synthesis job) when the file
//! is later wired into any spec emit. For now, these tests guarantee that the
//! emitter produces canonical, structurally correct Verilog.
//!
//! Closes #751.

use std::process::Command;

fn run_gen_trit_stdlib() -> String {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-trit-stdlib")
        .output()
        .expect("failed to spawn t27c gen-trit-stdlib");
    assert!(
        output.status.success(),
        "t27c gen-trit-stdlib exited with {:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("t27c gen-trit-stdlib produced non-UTF-8 output")
}

#[test]
fn emits_all_seven_modules_via_cli() {
    let v = run_gen_trit_stdlib();
    for name in [
        "module trit_not (",
        "module trit_and (",
        "module trit_or (",
        "module trit_half_adder (",
        "module trit_full_adder (",
        "module trit_multiply (",
        "module trit3_add (",
    ] {
        assert!(v.contains(name), "missing module header: {}", name);
    }
}

#[test]
fn encoding_constants_are_canonical_via_cli() {
    let v = run_gen_trit_stdlib();
    // Balanced ternary encoding must be exactly 2'b00 / 2'b01 / 2'b10.
    // This is a load-bearing invariant: any future change here breaks every
    // module that imports the stdlib.
    assert!(v.contains("TRIT_N = 2'b00"), "TRIT_N must encode to 2'b00 (-1)");
    assert!(v.contains("TRIT_Z = 2'b01"), "TRIT_Z must encode to 2'b01 (0)");
    assert!(v.contains("TRIT_P = 2'b10"), "TRIT_P must encode to 2'b10 (+1)");
    // 2'b11 must never appear as an active mux target -- it is the reserved
    // invalid encoding. (We allow it in comments, but not in `assign`s.)
    for line in v.lines() {
        let code = line.split("//").next().unwrap_or("");
        assert!(
            !code.contains("2'b11"),
            "trit stdlib must not use reserved encoding 2'b11 in code: {}",
            line
        );
    }
}

#[test]
fn trit_not_swaps_negative_and_positive() {
    let v = run_gen_trit_stdlib();
    // Locate the trit_not module body and check the truth-table assignments.
    let body = extract_module(&v, "trit_not")
        .expect("trit_not module not found");
    assert!(
        body.contains("(a == TRIT_N) ? TRIT_P"),
        "trit_not must map TRIT_N (-1) to TRIT_P (+1): {}",
        body
    );
    assert!(
        body.contains("(a == TRIT_P) ? TRIT_N"),
        "trit_not must map TRIT_P (+1) to TRIT_N (-1): {}",
        body
    );
    assert!(
        body.contains("(a == TRIT_Z) ? TRIT_Z"),
        "trit_not must map TRIT_Z (0) to TRIT_Z (0): {}",
        body
    );
}

#[test]
fn trit_and_implements_kleene_min() {
    let v = run_gen_trit_stdlib();
    let body = extract_module(&v, "trit_and").expect("trit_and module not found");
    // Kleene AND = min: if either operand is -1, output -1; else if either is 0, output 0; else +1.
    assert!(
        body.contains("(a == TRIT_N || b == TRIT_N) ? TRIT_N"),
        "trit_and must short-circuit to TRIT_N when either operand is TRIT_N (Kleene min)"
    );
    assert!(
        body.contains("(a == TRIT_Z || b == TRIT_Z) ? TRIT_Z"),
        "trit_and must produce TRIT_Z when either operand is TRIT_Z and neither is TRIT_N"
    );
}

#[test]
fn trit_or_implements_kleene_max() {
    let v = run_gen_trit_stdlib();
    let body = extract_module(&v, "trit_or").expect("trit_or module not found");
    // Kleene OR = max: if either is +1, output +1; else if either is 0, output 0; else -1.
    assert!(
        body.contains("(a == TRIT_P || b == TRIT_P) ? TRIT_P"),
        "trit_or must short-circuit to TRIT_P when either operand is TRIT_P (Kleene max)"
    );
    assert!(
        body.contains("(a == TRIT_Z || b == TRIT_Z) ? TRIT_Z"),
        "trit_or must produce TRIT_Z when either operand is TRIT_Z and neither is TRIT_P"
    );
}

#[test]
fn trit_half_adder_handles_carry_overflow() {
    let v = run_gen_trit_stdlib();
    let body =
        extract_module(&v, "trit_half_adder").expect("trit_half_adder module not found");
    // The two overflow cases are -1 + -1 = -2 -> (sum=+1, carry=-1)
    // and +1 + +1 = +2 -> (sum=-1, carry=+1). The implementation maps the
    // signed sum {-2, -1, 0, +1, +2} back to balanced ternary.
    assert!(
        body.contains("(total == -3'sd2) ? TRIT_P"),
        "trit_half_adder sum for total=-2 must wrap to TRIT_P (+1): {}",
        body
    );
    assert!(
        body.contains("(total ==  3'sd2) ? TRIT_N"),
        "trit_half_adder sum for total=+2 must wrap to TRIT_N (-1): {}",
        body
    );
    assert!(
        body.contains("(total == -3'sd2) ? TRIT_N"),
        "trit_half_adder carry for total=-2 must be TRIT_N (-1): {}",
        body
    );
    assert!(
        body.contains("(total ==  3'sd2) ? TRIT_P"),
        "trit_half_adder carry for total=+2 must be TRIT_P (+1): {}",
        body
    );
}

#[test]
fn trit_full_adder_uses_two_half_adders_and_or_combine() {
    let v = run_gen_trit_stdlib();
    let body =
        extract_module(&v, "trit_full_adder").expect("trit_full_adder module not found");
    // Exactly 2 half-adder instances (`ha1`, `ha2`) and one trit_or combine.
    let ha_count = body.matches("trit_half_adder ha").count();
    assert_eq!(
        ha_count, 2,
        "trit_full_adder must instantiate exactly 2 half adders, got {}: {}",
        ha_count, body
    );
    assert!(
        body.contains("trit_or carry_combine"),
        "trit_full_adder must combine half-adder carries via trit_or"
    );
}

#[test]
fn trit_multiply_uses_sign_comparison_not_arithmetic() {
    let v = run_gen_trit_stdlib();
    let body = extract_module(&v, "trit_multiply").expect("trit_multiply module not found");
    // No actual multiplier -- result derived from zero-check and sign-compare.
    // (This is the key insight that makes trit_multiply free in LUTs on FPGA.)
    assert!(
        body.contains("(a_zero || b_zero) ? TRIT_Z"),
        "trit_multiply must short-circuit to TRIT_Z when either operand is zero"
    );
    assert!(
        body.contains("same_sign          ? TRIT_P"),
        "trit_multiply must return TRIT_P when signs match (both nonzero)"
    );
    assert!(
        !body.contains(" * "),
        "trit_multiply must not contain the Verilog `*` operator: {}",
        body
    );
}

#[test]
fn trit3_add_chains_three_full_adders() {
    let v = run_gen_trit_stdlib();
    let body = extract_module(&v, "trit3_add").expect("trit3_add module not found");
    let fa_count = body.matches("trit_full_adder fa").count();
    assert_eq!(
        fa_count, 3,
        "trit3_add must instantiate exactly 3 full adders, got {}: {}",
        fa_count, body
    );
    // Carry chain: cin of fa0 is TRIT_Z, fa1.cin = fa0.cout, fa2.cin = fa1.cout.
    assert!(body.contains(".cin(TRIT_Z)"), "trit3_add fa0 must seed carry with TRIT_Z");
    assert!(body.contains(".cin(c0)"), "trit3_add fa1 must take its carry-in from fa0");
    assert!(body.contains(".cin(c1)"), "trit3_add fa2 must take its carry-in from fa1");
}

#[test]
fn output_is_self_contained_and_balanced() {
    let v = run_gen_trit_stdlib();
    // `timescale on top, `default_nettype none/wire band.
    assert!(v.contains("`timescale 1ns / 1ps"));
    assert!(v.contains("`default_nettype none"), "must disable implicit nets at top");
    assert!(v.contains("`default_nettype wire"), "must restore implicit nets at bottom");
    // No syntactic refs to unknown sigs/specs -- the stdlib is hermetic.
    // (It only depends on its own module names: trit_not/and/or used by full_adder, etc.)
    // Module count: exactly 7.
    let module_count = v.matches("\nmodule ").count() + if v.starts_with("module ") { 1 } else { 0 };
    assert_eq!(module_count, 7, "expected exactly 7 modules, got {}", module_count);
    let endmodule_count = v.matches("endmodule").count();
    assert_eq!(
        endmodule_count, 7,
        "expected exactly 7 endmodule keywords, got {}",
        endmodule_count
    );
}

/// Extract the body of a named module (between `module <name> (` and the
/// matching `endmodule`) for focused assertions.
fn extract_module<'a>(verilog: &'a str, name: &str) -> Option<&'a str> {
    let header = format!("module {} (", name);
    let start = verilog.find(&header)?;
    let after = &verilog[start..];
    let end_offset = after.find("\nendmodule")?;
    Some(&after[..end_offset])
}

//! Wave 32 -- R-TS-1 regression tests for the trit stdlib emitter.
//! Wave 33 -- R-TS-2 extends with 27-trit MAC primitives.
//!
//! Strategy: shell out to the built t27c binary via `CARGO_BIN_EXE_t27c`,
//! run `gen-trit-stdlib`, capture stdout, and assert structural truth-table
//! invariants on the emitted Verilog text. No HDL toolchain (Yosys, iverilog)
//! is required to run these tests; the synthesizability of the output is
//! validated by the FPGA E2E CI pipeline (fpga-synthesis job) when the file
//! is later wired into any spec emit. For now, these tests guarantee that the
//! emitter produces canonical, structurally correct Verilog.
//!
//! Closes #751 (Wave 32). Closes #754 (Wave 33).

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
fn emits_all_eleven_modules_via_cli() {
    let v = run_gen_trit_stdlib();
    for name in [
        // Wave 32 base primitives (7)
        "module trit_not (",
        "module trit_and (",
        "module trit_or (",
        "module trit_half_adder (",
        "module trit_full_adder (",
        "module trit_multiply (",
        "module trit3_add (",
        // Wave 33 MAC primitives (4)
        "module trit_compare (",
        "module trit27_parallel_multiply (",
        "module adder_tree_27 (",
        "module trit27_dot_product (",
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
        body.contains("c1 + c2"),
        "trit_full_adder must combine half-adder carries via signed addition"
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
        !body.contains("*"),
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
    // Module count: exactly 11 (7 base + 4 MAC).
    let module_count = v.matches("\nmodule ").count() + if v.starts_with("module ") { 1 } else { 0 };
    assert_eq!(module_count, 11, "expected exactly 11 modules, got {}", module_count);
    let endmodule_count = v.matches("endmodule").count();
    assert_eq!(
        endmodule_count, 11,
        "expected exactly 11 endmodule keywords, got {}",
        endmodule_count
    );
}

// ============================================================================
// Wave 33 (R-TS-2) -- MAC primitive regression tests
// ============================================================================

#[test]
fn trit_compare_uses_direct_unsigned_ordering() {
    let v = run_gen_trit_stdlib();
    let body = extract_module(&v, "trit_compare").expect("trit_compare module not found");
    // The encoding N(00) < Z(01) < P(10) matches balanced-ternary ordering
    // exactly, so trit_compare uses a direct unsigned `<` -- no LUT-heavy
    // sign decode is necessary.
    assert!(
        body.contains("(a == b) ? TRIT_Z"),
        "trit_compare must produce TRIT_Z when a == b: {}",
        body
    );
    assert!(
        body.contains("(a <  b) ? TRIT_N"),
        "trit_compare must produce TRIT_N when a < b (encoding ordering): {}",
        body
    );
    // No arithmetic decode -- compare must not contain any signed `'sd` literal.
    assert!(
        !body.contains("'sd"),
        "trit_compare must use pure encoding-comparison, no signed arithmetic: {}",
        body
    );
}

#[test]
fn trit27_parallel_multiply_is_27_lane_simd() {
    let v = run_gen_trit_stdlib();
    let body = extract_module(&v, "trit27_parallel_multiply")
        .expect("trit27_parallel_multiply module not found");
    // 54-bit ports (27 trits * 2 bits per trit).
    assert!(body.contains("input  wire [53:0] a"));
    assert!(body.contains("input  wire [53:0] b"));
    assert!(body.contains("output wire [53:0] result"));
    // Genvar-driven SIMD with +: part-select.
    assert!(body.contains("genvar i;"), "must use genvar i for SIMD generation");
    assert!(
        body.contains("for (i = 0; i < 27; i = i + 1) begin : mult_gen"),
        "must iterate exactly 27 lanes"
    );
    assert!(body.contains("a[i*2 +: 2]"), "must use +: part-select on a");
    assert!(body.contains("b[i*2 +: 2]"), "must use +: part-select on b");
    assert!(body.contains("result[i*2 +: 2]"), "must use +: part-select on result");
    // No real multipliers -- pure sign comparison per lane.
    assert!(
        !body.contains(" * "),
        "trit27_parallel_multiply must not use the `*` operator: {}",
        body
    );
    assert!(body.contains("same_sign"), "must derive lane result from sign comparison");
}

#[test]
fn adder_tree_27_has_three_reduction_levels() {
    let v = run_gen_trit_stdlib();
    let body =
        extract_module(&v, "adder_tree_27").expect("adder_tree_27 module not found");
    // 3-level tree: decode (27 trits to signed [1:0]), then 27 -> 9 -> 3 -> 1.
    assert!(
        body.contains("wire signed [1:0] val [0:26];"),
        "adder_tree_27 must decode to a 27-entry signed [1:0] array"
    );
    assert!(
        body.contains("wire signed [2:0] l1 [0:8];"),
        "adder_tree_27 must have a 9-entry signed [2:0] level1 array"
    );
    // FIVE bits, not four. This assertion previously demanded `signed [3:0]`
    // and so actively pinned a defect: a 4-bit signed value spans [-8, +7] and
    // cannot hold this level's range of [-9, +9], so any group of nine trits
    // summing to +8, +9 or -9 wrapped by 16. The first exhaustive proof of the
    // primitive (Wave 628) found the tree returning -14 for a vector whose
    // balanced sum is +2. A test that asserts a width without checking the
    // range it must cover is a test that protects the bug. Prop. 80.
    assert!(
        body.contains("wire signed [4:0] l2 [0:2];"),
        "adder_tree_27 must have a 3-entry signed [4:0] level2 array -- \
         four bits cannot hold [-9, +9]"
    );
    // Three explicit level-2 reductions and one final level-3 sum.
    assert!(body.contains("assign l2[0] = l1[0] + l1[1] + l1[2];"));
    assert!(body.contains("assign l2[1] = l1[3] + l1[4] + l1[5];"));
    assert!(body.contains("assign l2[2] = l1[6] + l1[7] + l1[8];"));
    assert!(body.contains("assign sum = l2[0] + l2[1] + l2[2];"));
    // Output width: signed [5:0] (range -27..+27 fits in 6 bits signed).
    assert!(body.contains("output wire signed [5:0] sum"));
}

#[test]
fn trit27_dot_product_composes_mac_pipeline() {
    let v = run_gen_trit_stdlib();
    let body = extract_module(&v, "trit27_dot_product")
        .expect("trit27_dot_product module not found");
    // Step 1: parallel multiply (named `mult_unit`).
    assert!(
        body.contains("trit27_parallel_multiply mult_unit"),
        "trit27_dot_product must instantiate trit27_parallel_multiply as mult_unit"
    );
    // Step 2: adder tree (named `tree`).
    assert!(
        body.contains("adder_tree_27 tree"),
        "trit27_dot_product must instantiate adder_tree_27 as tree"
    );
    // Port wiring: input_vec and weight_vec feed mult_unit; products feed tree.
    assert!(body.contains(".a(input_vec)"));
    assert!(body.contains(".b(weight_vec)"));
    assert!(body.contains(".trits(products)"));
    assert!(body.contains(".sum(result)"));
    // Output is signed [5:0] -- final MAC result.
    assert!(body.contains("output wire signed [5:0] result"));
    // The dot product must NOT use the Verilog `*` operator anywhere.
    assert!(
        !body.contains(" * "),
        "trit27_dot_product must remain multiplier-free: {}",
        body
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

// ============================================================================
// Check for the spec-first GF-T16 MAC (specs/ternary/gft_dot2.t27, #1764 + GF-T):
// y = a1*b1 + a2*b2 in the ternary-native GoldenFloat format that was verified
// bit-exact ON SILICON (AX7203, gft_dot2 3/3). The hand-written RTL noted that
// t27c gen-verilog could not emit this (interleaved reg decls -- fixed by #1741);
// this test proves the spec-first realization is bit-exact to that silicon-proven
// RTL over random inputs. The reference modules (gft_dot2/gft_mul/gft_add) are
// embedded verbatim so the check is self-contained. Skips without iverilog/vvp.
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn t27c() -> &'static str { env!("CARGO_BIN_EXE_t27c") }
fn spec_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("specs").join("ternary").join("gft_dot2.t27")
}
fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_gft_{}_{}", std::process::id(), label));
    if dir.exists() { let _ = fs::remove_dir_all(&dir); }
    dir
}
fn tool_available(tool: &str) -> bool {
    Command::new(tool).arg("-V").output().map(|o| o.status.success()).unwrap_or(false)
}

// Silicon-proven reference RTL (trinity-fpga/build/gft_dot2), embedded verbatim.
const REFERENCE_RTL: &str = r#####"
`timescale 1ns / 1ps
`default_nettype none
// ============================================================================
// gft_dot2 -- 2-term GF-T16 dot product: y = a1*b1 + a2*b2, the multiply-accumulate
// kernel at the heart of every matmul / attention / inference layer. Pure composition
// of the silicon-proven gft_mul with gft_add (both realizations of tri_gft_arith /
// tri_gft_add) -- no new arithmetic, just the MAC wiring. Combinational.
//
// This is the hardware twin of the NUMERICAL dot-product advantage measured in
// tests/gft_task_accuracy.rs: there GF-T16 owns the wide-dynamic-range dot product on
// paper; here the same dot product is computed in GF-T16 hardware, bit-exact to spec.
//
// Operands/result are packed GF-T16 magnitudes: [ offset:15..9 (7b) | mant:8..0 (9b) ],
// value = (1 + mant/512) * 2^(offset-40).
// ============================================================================
module gft_dot2 (
    input  wire [15:0] a1,
    input  wire [15:0] b1,
    input  wire [15:0] a2,
    input  wire [15:0] b2,
    output wire [15:0] y
);
    // term 1 = a1 * b1
    wire [31:0] p1_off, p1_mant;
    gft_mul #(.BIAS(40), .OFFSET_MAX(80), .MANT_ONE(512)) u_m1 (
        .a_off({25'd0, a1[15:9]}), .a_mant({23'd0, a1[8:0]}),
        .b_off({25'd0, b1[15:9]}), .b_mant({23'd0, b1[8:0]}),
        .out_off(p1_off), .out_mant(p1_mant));

    // term 2 = a2 * b2
    wire [31:0] p2_off, p2_mant;
    gft_mul #(.BIAS(40), .OFFSET_MAX(80), .MANT_ONE(512)) u_m2 (
        .a_off({25'd0, a2[15:9]}), .a_mant({23'd0, a2[8:0]}),
        .b_off({25'd0, b2[15:9]}), .b_mant({23'd0, b2[8:0]}),
        .out_off(p2_off), .out_mant(p2_mant));

    // accumulate: term1 + term2 (same-sign GF-T add)
    wire [31:0] y_off, y_mant;
    gft_add #(.OFFSET_MAX(80), .MANT_ONE(512), .SIG_BITS(10)) u_acc (
        .a_off(p1_off), .a_mant(p1_mant),
        .b_off(p2_off), .b_mant(p2_mant),
        .out_off(y_off), .out_mant(y_mant));

    assign y = {y_off[6:0], y_mant[8:0]};
endmodule
`default_nettype wire

`timescale 1ns / 1ps
`default_nettype none
// ============================================================================
// gft_mul -- GF-T ladder multiplier (balanced-ternary exponent).
//
// Verified realization of specs/tri_gft_arith.t27's gft_mul_offset_full_p +
// gft_mul_mant_p + gft_mul_mant_carry_p -- the SAME spec the over-wire verifier
// runs (trinet_compute_over_mesh / trinet_rung_verify). SSOT is the .t27; this
// .v is the synthesizable realization (as fpga/gf16/gf16_mul.v is for GF16).
//
// t27c gen-verilog cannot emit this directly yet: it interleaves `reg`
// declarations with statements inside begin/end blocks (illegal Verilog; iverilog
// rejects it). Tracked upstream; this hand-transcription keeps the exact logic
// with legal declaration ordering, gated by an iverilog KAT sweep below.
//
// Combinational. Parametric per rung; GF-T16 defaults (bias 40, offset_max 80,
// mant_one 512). GF-T8 = (13, 26, 16); GF-T4 = (4, 8, 2); GF-T32 uses wider mant.
// ============================================================================
module gft_mul #(
    parameter [31:0] BIAS       = 40,
    parameter [31:0] OFFSET_MAX = 80,
    parameter [31:0] MANT_ONE   = 512
) (
    input  wire [31:0] a_off,
    input  wire [31:0] a_mant,
    input  wire [31:0] b_off,
    input  wire [31:0] b_mant,
    output wire [31:0] out_off,
    output wire [31:0] out_mant
);
    // Full-precision significand product (1+M/mant_one) scaled by mant_one^2.
    wire [31:0] prod   = (MANT_ONE + a_mant) * (MANT_ONE + b_mant);
    wire [31:0] thresh = (2 * MANT_ONE) * MANT_ONE;      // one-bit renorm boundary
    wire        carry  = (prod >= thresh);               // mantissa overflow -> exp += 1

    // Exponent offset: add offsets, apply the carry, de-bias, saturate at the rung's max.
    wire [31:0] sum    = a_off + b_off + {31'd0, carry};
    wire [31:0] result = sum - BIAS;
    assign out_off = (sum < BIAS)          ? 32'd0 :
                     (result >= OFFSET_MAX) ? OFFSET_MAX : result;

    // Mantissa: renormalize by the carry (divisors are constant powers of two -> shifts).
    assign out_mant = carry ? ((prod / (2 * MANT_ONE)) - MANT_ONE)
                            : ((prod /      MANT_ONE ) - MANT_ONE);
endmodule
`default_nettype wire

`timescale 1ns / 1ps
`default_nettype none
// ============================================================================
// gft_add -- GF-T ladder adder (SAME-sign add), balanced-ternary exponent.
//
// Verified realization of specs/tri_gft_add.t27's gft_add_offset_c_p +
// gft_add_mant_c_p (via gft_add_sb_p / _offset_p / _mant_p) -- the SAME spec the
// over-wire verifier runs (trinet_rung_verify, trinet_compute_over_mesh). Align
// the smaller operand by the exponent-offset difference (barrel shift), add the
// significands, and renormalize by one carry. Combinational; parametric per rung.
// GF-T16 defaults: offset_max 80, mant_one 512, sig_bits 10 (mant_bits+1).
// ============================================================================
module gft_add #(
    parameter [31:0] OFFSET_MAX = 80,
    parameter [31:0] MANT_ONE   = 512,
    parameter [31:0] SIG_BITS   = 10
) (
    input  wire [31:0] a_off,
    input  wire [31:0] a_mant,
    input  wire [31:0] b_off,
    input  wire [31:0] b_mant,
    output wire [31:0] out_off,
    output wire [31:0] out_mant
);
    // Order operands so `hi` has the larger (or equal) exponent offset.
    wire        a_hi   = (a_off >= b_off);
    wire [31:0] hi_off = a_hi ? a_off  : b_off;
    wire [31:0] hi_m   = a_hi ? a_mant : b_mant;
    wire [31:0] lo_off = a_hi ? b_off  : a_off;
    wire [31:0] lo_m   = a_hi ? b_mant : a_mant;

    // Align the smaller significand right by the offset difference (0 if it underflows).
    wire [31:0] d  = hi_off - lo_off;
    wire [31:0] sb = (d >= SIG_BITS) ? 32'd0 : ((MANT_ONE + lo_m) >> d[4:0]);
    wire [31:0] sum = (MANT_ONE + hi_m) + sb;

    // Renormalize: a significand >= 2*mant_one carries into the exponent (+1, saturate).
    wire        carry = (sum >= (2 * MANT_ONE));
    wire [31:0] e     = hi_off + 32'd1;
    assign out_off  = carry ? ((e >= OFFSET_MAX) ? OFFSET_MAX : e) : hi_off;
    assign out_mant = carry ? ((sum >> 1) - MANT_ONE) : (sum - MANT_ONE);
endmodule
`default_nettype wire

"#####;

// Drive both DUTs with random valid GF-T16 magnitudes (offset in [1,79], mant in
// [0,511]) and require the spec-first result to equal the silicon-proven RTL.
const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg [15:0] a1,b1,a2,b2; wire [15:0] y_spec, y_ref; integer i, fails, o, m;
  GftDot2  dut_spec(.clk(1'b0),.rst_n(1'b1),.en(1'b1),.a1(a1),.b1(b1),.a2(a2),.b2(b2),.ready(),.result(y_spec));
  gft_dot2 dut_ref(.a1(a1),.b1(b1),.a2(a2),.b2(b2),.y(y_ref));
  function [15:0] rnd; input integer dummy; begin
    o = 1 + ($random % 79); if (o<1) o=1; if (o>79) o=79;
    m = $random % 512; if (m<0) m=-m;
    rnd = (o<<9) | m;
  end endfunction
  initial begin
    fails=0;
    for (i=0;i<2000;i=i+1) begin
      a1=rnd(i); b1=rnd(i+1); a2=rnd(i+2); b2=rnd(i+3); #1;
      if (y_spec!==y_ref) begin fails=fails+1;
        if (fails<=5) $display("FAIL a1=%h b1=%h a2=%h b2=%h spec=%h ref=%h",a1,b1,a2,b2,y_spec,y_ref); end
    end
    if (fails==0) $display("ALL_PASS 2000"); else $display("FAILED %0d",fails);
    $finish;
  end
endmodule
"#;

#[test]
fn spec_first_gft_mac_matches_silicon_proven_rtl() {
    let gen = Command::new(t27c()).arg("gen-verilog").arg(spec_path()).output().expect("gen-verilog");
    assert!(gen.status.success(), "gen-verilog failed:\n{}", String::from_utf8_lossy(&gen.stderr));
    let verilog = String::from_utf8_lossy(&gen.stdout).into_owned();
    assert!(verilog.contains("input  wire [15:0] a1") && verilog.contains("output wire [15:0] result"),
        "GF-T MAC did not expose the a1/b1/a2/b2 -> result data interface:\n{}", verilog);
    assert!(verilog.contains("assign result = on_comb(a1, b1, a2, b2);"),
        "result port is not driven from the MAC:\n{}", verilog);

    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping GF-T cross-check");
        return;
    }
    let dir = scratch_dir("chk");
    fs::create_dir_all(&dir).expect("scratch");
    fs::write(dir.join("spec.v"), &gen.stdout).expect("spec.v");
    fs::write(dir.join("ref.v"), REFERENCE_RTL).expect("ref.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("tb.v");
    let vvp = dir.join("sim.vvp");
    let comp = Command::new("iverilog").args(["-g2012","-o",vvp.to_str().unwrap()])
        .arg(dir.join("spec.v")).arg(dir.join("ref.v")).arg(dir.join("tb.v")).output().expect("iverilog");
    assert!(comp.status.success(), "iverilog compile failed:\n{}", String::from_utf8_lossy(&comp.stderr));
    let run = Command::new("vvp").arg(&vvp).output().expect("vvp");
    let out = String::from_utf8_lossy(&run.stdout).into_owned();
    let _ = fs::remove_dir_all(&dir);
    assert!(out.contains("ALL_PASS 2000"), "spec-first GF-T MAC differs from the silicon-proven RTL:\n{}", out);
    assert!(!out.contains("FAIL"), "GF-T MAC mismatch:\n{}", out);
}

// ============================================================================
// Check for the spec-first GF-T16 matmul ROW / 2-neuron layer (specs/ternary/
// gft_layer2.t27): a shared activation (a1,a2) feeds two GF-T neurons with their
// own weight vectors; the two GF-T16 outputs are packed into one 32-bit result
// (neuron 0 in [15:0], neuron 1 in [31:16]). Proves the spec-first layer is
// bit-exact to two silicon-proven gft_dot2 instances packed, over random inputs.
// ============================================================================
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
fn t27c()->&'static str{env!("CARGO_BIN_EXE_t27c")}
fn spec_path()->PathBuf{PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("specs").join("ternary").join("gft_layer2.t27")}
fn scratch_dir(l:&str)->PathBuf{let d=env::temp_dir().join(format!("t27_gl2_{}_{}",std::process::id(),l));if d.exists(){let _=fs::remove_dir_all(&d);}d}
fn tool_available(t:&str)->bool{Command::new(t).arg("-V").output().map(|o|o.status.success()).unwrap_or(false)}
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
const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg [15:0] a1,a2,w0a,w0b,w1a,w1b; wire [31:0] y_spec; wire [15:0] n0,n1; integer i,fails,o,m;
  GftLayer2 dut(.clk(1'b0),.rst_n(1'b1),.en(1'b1),.a1(a1),.a2(a2),.w0a(w0a),.w0b(w0b),.w1a(w1a),.w1b(w1b),.ready(),.result(y_spec));
  gft_dot2 rn0(.a1(w0a),.b1(a1),.a2(w0b),.b2(a2),.y(n0));
  gft_dot2 rn1(.a1(w1a),.b1(a1),.a2(w1b),.b2(a2),.y(n1));
  wire [31:0] y_ref = {n1, n0};
  function [15:0] rnd; input integer dd; begin o=1+($random%79);if(o<1)o=1;if(o>79)o=79;m=$random%512;if(m<0)m=-m;rnd=(o<<9)|m; end endfunction
  initial begin
    fails=0;
    for(i=0;i<2000;i=i+1) begin
      a1=rnd(0);a2=rnd(0);w0a=rnd(0);w0b=rnd(0);w1a=rnd(0);w1b=rnd(0); #1;
      if(y_spec!==y_ref) begin fails=fails+1; if(fails<=5)$display("FAIL spec=%h ref=%h",y_spec,y_ref); end
    end
    if(fails==0)$display("ALL_PASS 2000"); else $display("FAILED %0d",fails);
    $finish;
  end
endmodule
"#;
#[test]
fn spec_first_gft_layer2_matches_silicon(){
    let gen=Command::new(t27c()).arg("gen-verilog").arg(spec_path()).output().expect("gen");
    assert!(gen.status.success(),"gen-verilog failed:\n{}",String::from_utf8_lossy(&gen.stderr));
    let v=String::from_utf8_lossy(&gen.stdout).into_owned();
    assert!(v.contains("output wire [31:0] result")&&v.contains("input  wire [15:0] w1b"),"missing layer interface:\n{}",v);
    if !tool_available("iverilog")||!tool_available("vvp"){eprintln!("SKIP: no iverilog/vvp");return;}
    let d=scratch_dir("chk"); fs::create_dir_all(&d).unwrap();
    fs::write(d.join("spec.v"),&gen.stdout).unwrap(); fs::write(d.join("ref.v"),REFERENCE_RTL).unwrap(); fs::write(d.join("tb.v"),TESTBENCH).unwrap();
    let vvp=d.join("s.vvp");
    let c=Command::new("iverilog").args(["-g2012","-o",vvp.to_str().unwrap()]).arg(d.join("spec.v")).arg(d.join("ref.v")).arg(d.join("tb.v")).output().unwrap();
    assert!(c.status.success(),"iverilog failed:\n{}",String::from_utf8_lossy(&c.stderr));
    let r=Command::new("vvp").arg(&vvp).output().unwrap();
    let o=String::from_utf8_lossy(&r.stdout).into_owned(); let _=fs::remove_dir_all(&d);
    assert!(o.contains("ALL_PASS 2000"),"layer differs from silicon:\n{}",o);
    assert!(!o.contains("FAIL"),"layer mismatch:\n{}",o);
}

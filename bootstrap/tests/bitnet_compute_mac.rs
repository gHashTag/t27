// ============================================================================
// Functional testbench for the BitNet ternary MAC compute stage.
//
// The BitNet HLS modules were validated only by substring asserts on emitted
// Verilog + (since #1730) a structural elaboration check. Neither proves the
// datapath COMPUTES the right numbers. This testbench drives
// `pipeline_stage2_compute` (which wraps `trit27_dot_product`) with known and
// randomized trit chunks and checks the accumulated dot product via
// iverilog + vvp.
//
// Encoding (trit_stdlib source of truth): 2'b00 = -1, 2'b01 = 0, 2'b10 = +1;
// trit i occupies bits [2i+1:2i] of the 54-bit (27-trit) chunk.
//
// This test found a real bug (#1733): `adder_tree_27` level-2 used `signed
// [3:0]` (range [-8, +7]) while a group of 9 same-sign trits sums to +/-9, so
// an all-+1 dot product read -21 instead of +27. It now also pins the exact
// overflow boundary (a level-2 group summing to +/-8) and cross-checks 200
// randomized vectors against an independent reference dot product.
//
// Skips gracefully when `iverilog`/`vvp` are not on PATH.
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn t27c() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_bitnet_mac_{}_{}", std::process::id(), label));
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
    dir
}

fn tool_available(tool: &str) -> bool {
    Command::new(tool)
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// Golden + boundary + randomized testbench.
//   {27{2'b10}} = all +1,  {27{2'b00}} = all -1,  {27{2'b01}} = all 0.
// `ref_dot` is an independent reference (decode + multiply-accumulate) that does
// NOT use the DUT's reduction tree, so a discrepancy means a real DUT bug.
const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg clk=0, rst_n=0, valid_in=0, first_chunk=0, last_chunk=0;
  reg [53:0] input_chunk=0, weight_chunk=0;
  wire valid_out, result_final;
  wire signed [15:0] result;
  integer fails=0, k, iter;
  reg [53:0] ra, rb;
  reg signed [15:0] expected;
  pipeline_stage2_compute dut(.clk(clk),.rst_n(rst_n),.valid_in(valid_in),
    .input_chunk(input_chunk),.weight_chunk(weight_chunk),
    .first_chunk(first_chunk),.last_chunk(last_chunk),
    .valid_out(valid_out),.result(result),.result_final(result_final));
  always #5 clk = ~clk;
  // Map a random integer to a valid trit encoding {Z=01, P=10, N=00}.
  function [1:0] tmap(input integer x); integer m; begin
    m = x % 3; if (m < 0) m = m + 3;
    tmap = (m==0)?2'b01:(m==1)?2'b10:2'b00; end
  endfunction
  // Independent reference dot product (does not use the DUT adder tree).
  function signed [15:0] ref_dot(input [53:0] a, input [53:0] b);
    integer j, va, vb, acc; reg [1:0] ta, tb; begin
    acc = 0;
    for (j=0;j<27;j=j+1) begin
      ta=a[j*2 +: 2]; tb=b[j*2 +: 2];
      va=(ta==2'b00)?-1:(ta==2'b10)?1:0;
      vb=(tb==2'b00)?-1:(tb==2'b10)?1:0;
      acc=acc+va*vb; end
    ref_dot=acc; end
  endfunction
  task drive(input [53:0] iv, input [53:0] wv, input f, input l); begin
    @(negedge clk); input_chunk=iv; weight_chunk=wv; valid_in=1; first_chunk=f; last_chunk=l;
    @(posedge clk); @(negedge clk); valid_in=0; end
  endtask
  task check(input signed [15:0] got, input signed [15:0] exp, input [255:0] nm); begin
    if (got!==exp) begin fails=fails+1; $display("FAIL %0s got=%0d exp=%0d",nm,got,exp); end
    else $display("PASS %0s = %0d",nm,got); end
  endtask
  initial begin
    rst_n=0; repeat(2) @(posedge clk); rst_n=1;
    // Single-chunk dot products.
    drive({27{2'b10}},{27{2'b10}},1,1); check(result,16'sd27,"allP_x_allP");
    drive({27{2'b10}},{27{2'b00}},1,1); check(result,-16'sd27,"allP_x_allN");
    drive({27{2'b00}},{27{2'b00}},1,1); check(result,16'sd27,"allN_x_allN");
    drive({27{2'b01}},{27{2'b10}},1,1); check(result,16'sd0,"allZ");
    drive({{26{2'b01}},2'b10},{27{2'b10}},1,1); check(result,16'sd1,"oneP");
    // Overflow boundary: a level-2 group summing to exactly +/-8 (the value the
    // old signed [3:0] wrapped). 8 P (or N) in trits 0..7, the rest Z.
    drive({19{2'b01}} << 16 | {8{2'b10}},{27{2'b10}},1,1); check(result,16'sd8,"eightP_boundary");
    drive({19{2'b01}} << 16 | {8{2'b00}},{27{2'b10}},1,1); check(result,-16'sd8,"eightN_boundary");
    // Multi-chunk accumulation: +27 then -27 = 0.
    drive({27{2'b10}},{27{2'b10}},1,0);
    drive({27{2'b10}},{27{2'b00}},0,1); check(result,16'sd0,"accum_p27_m27");
    // Randomized sweep vs the independent reference (fixed iverilog seed ->
    // deterministic).
    for (iter=0; iter<200; iter=iter+1) begin
      for (k=0;k<27;k=k+1) begin ra[k*2 +: 2]=tmap($random); rb[k*2 +: 2]=tmap($random); end
      expected=ref_dot(ra,rb);
      drive(ra,rb,1,1);
      if (result!==expected) begin fails=fails+1; $display("FAIL rand[%0d] got=%0d exp=%0d",iter,result,expected); end
    end
    $display("RAND_DONE 200");
    if (fails==0) $display("ALL_PASS"); else $display("FAILED %0d",fails);
    $finish;
  end
endmodule
"#;

#[test]
fn bitnet_mac_compute_golden_vectors() {
    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping BitNet MAC functional test");
        return;
    }

    let dir = scratch_dir("mac");
    fs::create_dir_all(&dir).expect("create scratch dir");

    // Emit the compute stage and the trit stdlib it depends on.
    for (sub, file) in [
        ("gen-pipeline-stage2", "compute.sv"),
        ("gen-trit-stdlib", "trit_stdlib.sv"),
    ] {
        let out = Command::new(t27c()).arg(sub).output().expect("invoke t27c");
        assert!(out.status.success(), "{} failed", sub);
        fs::write(dir.join(file), &out.stdout).unwrap_or_else(|e| panic!("write {}: {}", file, e));
    }
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    // Compile with Icarus and run.
    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("compute.sv"))
        .arg(dir.join("trit_stdlib.sv"))
        .arg(dir.join("tb.v"))
        .output()
        .expect("invoke iverilog");
    assert!(
        compile.status.success(),
        "iverilog compile failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new("vvp").arg(&vvp_path).output().expect("invoke vvp");
    let stdout = String::from_utf8_lossy(&run.stdout).into_owned();

    let _ = fs::remove_dir_all(&dir);

    assert!(
        stdout.contains("RAND_DONE 200"),
        "randomized sweep did not complete:\n{}",
        stdout
    );
    assert!(
        stdout.contains("ALL_PASS"),
        "MAC golden/boundary/random check did not all pass:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("FAIL"),
        "MAC check reported a failure:\n{}",
        stdout
    );
}

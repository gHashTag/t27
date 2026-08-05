// ============================================================================
// Functional cross-check for the spec-first BitNet neuron
// (specs/ternary/bitnet_neuron.t27, function `neuron4`): accumulate the ternary
// dot product over 4 (activation, weight) chunk pairs, then re-ternarize with a
// threshold. Its `dot27` uses a real `while` loop (enabled by the gen-verilog
// local-decl hoist fix, #1741).
//
// 200 random valid-trit chunk sets + random thresholds are fed to both the
// generated `neuron4` and an independent in-testbench reference
// (decode+multiply-accumulate over 4 chunks, then threshold), asserting
// equality via iverilog + vvp. Skips gracefully without iverilog/vvp.
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn t27c() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn spec_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("specs")
        .join("ternary")
        .join("bitnet_neuron.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_neuron_{}_{}", std::process::id(), label));
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

const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg [53:0] a0,w0,a1,w1,a2,w2,a3,w3;
  reg signed [15:0] thr;
  integer fails=0, iter, k;
  reg signed [15:0] acc; reg [7:0] exp;
  function [1:0] tmap(input integer x); integer m; begin
    m=x%3; if(m<0)m=m+3; tmap=(m==0)?2'b01:(m==1)?2'b10:2'b00; end
  endfunction
  function signed [15:0] dot_ref(input [53:0] a, input [53:0] b);
    integer j, va, vb, s; reg [1:0] ta, tb; begin
    s=0;
    for (j=0;j<27;j=j+1) begin
      ta=a[j*2 +: 2]; tb=b[j*2 +: 2];
      va=(ta==2'b00)?-1:(ta==2'b10)?1:0;
      vb=(tb==2'b00)?-1:(tb==2'b10)?1:0;
      s=s+va*vb; end
    dot_ref=s; end
  endfunction
  function [7:0] quant_ref(input signed [15:0] v, input signed [15:0] t); begin
    if (v > t) quant_ref=2; else if (v < -t) quant_ref=0; else quant_ref=1; end
  endfunction
  task rnd(output [53:0] v); integer k2; begin
    for (k2=0;k2<27;k2=k2+1) v[k2*2 +: 2]=tmap($random); end
  endtask
  initial begin
    for (iter=0; iter<200; iter=iter+1) begin
      rnd(a0); rnd(w0); rnd(a1); rnd(w1); rnd(a2); rnd(w2); rnd(a3); rnd(w3);
      thr = ($random % 60); if (thr < 0) thr = -thr;
      #1;
      acc = dot_ref(a0,w0)+dot_ref(a1,w1)+dot_ref(a2,w2)+dot_ref(a3,w3);
      exp = quant_ref(acc, thr);
      if (dut.neuron4({10'd0,a0},{10'd0,w0},{10'd0,a1},{10'd0,w1},
                      {10'd0,a2},{10'd0,w2},{10'd0,a3},{10'd0,w3}, thr) !== exp) begin
        fails=fails+1;
        if (fails<=3) $display("FAIL[%0d] got=%0d exp=%0d acc=%0d thr=%0d", iter,
          dut.neuron4({10'd0,a0},{10'd0,w0},{10'd0,a1},{10'd0,w1},
                      {10'd0,a2},{10'd0,w2},{10'd0,a3},{10'd0,w3}, thr), exp, acc, thr);
      end
    end
    if (fails==0) $display("ALL_MATCH 200"); else $display("MISMATCH %0d", fails);
    $finish;
  end
  BitnetNeuron dut(.clk(1'b0), .rst_n(1'b1), .en(1'b1), .ready());
endmodule
"#;

#[test]
fn spec_first_neuron4_matches_reference() {
    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping BitNet neuron cross-check");
        return;
    }

    let dir = scratch_dir("xcheck");
    fs::create_dir_all(&dir).expect("create scratch dir");

    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(spec_path())
        .output()
        .expect("invoke gen-verilog");
    assert!(
        gen.status.success(),
        "gen-verilog of bitnet_neuron.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    fs::write(dir.join("neuron.v"), &gen.stdout).expect("write neuron.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("neuron.v"))
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
        stdout.contains("ALL_MATCH 200"),
        "spec-first neuron4 did not match the reference on all 200 vectors:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("MISMATCH") && !stdout.contains("FAIL"),
        "spec-first neuron4 mismatch vs reference:\n{}",
        stdout
    );
}

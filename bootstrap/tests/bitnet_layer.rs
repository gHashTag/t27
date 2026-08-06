// ============================================================================
// Functional check for the 2-neuron spec-first BitNet layer
// (specs/ternary/bitnet_layer.t27, function `layer2`): two `neuronN` units over
// shared activations and per-neuron weights, packing the two output trits into
// a byte (trit0 in bits[1:0], trit1 in bits[3:2]).
//
// A hand testbench packs uniform chunks directly into the 512-bit vectors (so
// it does not depend on array-literal call args, #1749) and checks the packed
// layer output for known per-neuron dot-product/quantize outcomes. Skips
// gracefully without iverilog/vvp.
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
        .join("bitnet_layer.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_layer_{}_{}", std::process::id(), label));
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

// Output packs trit1<<2 | trit0. P=2, Z=1, N=0.
//   w0=P, w1=N (acts=P) -> (N<<2)|P = 2
//   w0=P, w1=P          -> (P<<2)|P = 10
//   w0=N, w1=P          -> (P<<2)|N = 8
//   allZ                -> (Z<<2)|Z = 5
//   nchunks=0           -> both Z   = 5
const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg [511:0] acts, w0, w1;
  localparam [53:0] P = {27{2'b10}};
  localparam [53:0] N = {27{2'b00}};
  localparam [53:0] Z = {27{2'b01}};
  integer fails=0;
  task fill(output [511:0] v, input [53:0] cv); integer k; begin
    v=0; for (k=0;k<8;k=k+1) v[k*64 +: 64]={10'd0,cv}; end
  endtask
  task chk(input [7:0] got, input [7:0] exp, input [127:0] nm); begin
    if (got!==exp) begin fails=fails+1; $display("FAIL %0s got=%0d exp=%0d",nm,got,exp); end
    else $display("PASS %0s=%0d",nm,got); end
  endtask
  initial begin
    fill(acts,P); fill(w0,P); fill(w1,N); chk(dut.layer2(acts,w0,w1,4,16'sd10), 2,  "P_N");
    fill(acts,P); fill(w0,P); fill(w1,P); chk(dut.layer2(acts,w0,w1,4,16'sd10), 10, "P_P");
    fill(acts,P); fill(w0,N); fill(w1,P); chk(dut.layer2(acts,w0,w1,4,16'sd10), 8,  "N_P");
    fill(acts,Z); fill(w0,Z); fill(w1,Z); chk(dut.layer2(acts,w0,w1,8,16'sd10), 5,  "Z_Z");
    fill(acts,P); fill(w0,P); fill(w1,P); chk(dut.layer2(acts,w0,w1,0,16'sd10), 5,  "zero_chunks");
    if (fails==0) $display("ALL_PASS"); else $display("FAILED %0d", fails);
    $finish;
  end
  BitnetLayer dut(.clk(1'b0), .rst_n(1'b1), .en(1'b1), .ready());
endmodule
"#;

#[test]
fn spec_first_layer2_packs_two_neuron_trits() {
    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping BitNet layer check");
        return;
    }

    let dir = scratch_dir("chk");
    fs::create_dir_all(&dir).expect("create scratch dir");

    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(spec_path())
        .output()
        .expect("invoke gen-verilog");
    assert!(
        gen.status.success(),
        "gen-verilog of bitnet_layer.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    fs::write(dir.join("layer.v"), &gen.stdout).expect("write layer.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("layer.v"))
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
        stdout.contains("ALL_PASS"),
        "layer2 packed-output check failed:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "layer2 mismatch:\n{}", stdout);
}

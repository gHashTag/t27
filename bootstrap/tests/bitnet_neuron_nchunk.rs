// ============================================================================
// Functional check for the parameterized N-chunk spec-first BitNet neuron
// (specs/ternary/bitnet_neuron_nchunk.t27, function `neuronN`). It loops the
// ternary dot product over the first `nchunks` (activation, weight) chunk pairs
// of `[8]u64` packed arrays, then re-ternarizes -- exercising both gen-verilog
// fixes: local-decl hoisting in a loop (#1741) and packed-array param element
// indexing (#1748).
//
// A hand testbench packs uniform chunks directly into the 512-bit vectors (so
// it does not depend on array-literal call args, which have a separate sim bug,
// #1749) and checks the neuron output against known dot-product accumulations
// across several `nchunks` and threshold settings. Skips gracefully without
// iverilog/vvp.
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
        .join("bitnet_neuron_nchunk.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_neuronN_{}_{}", std::process::id(), label));
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

// Uniform-chunk cases (P=+1, N=-1, Z=0 trits across all 27 lanes of a chunk):
//   allP x allP, nchunks=8 -> 8*27 = 216 > 10 -> P(2)
//   allP x allN, nchunks=4 -> 4*(-27) = -108 < -10 -> N(0)
//   allN x allN, nchunks=8 -> 8*27 = 216 -> P(2)
//   allZ,        nchunks=8 -> 0 -> Z(1)
//   allP x allP, nchunks=1, thr=30 -> 27 in [-30,30] -> Z(1)
//   nchunks=0 -> acc=0 -> Z(1)
const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg [511:0] acts, weights;
  localparam [53:0] P = {27{2'b10}};
  localparam [53:0] N = {27{2'b00}};
  localparam [53:0] Z = {27{2'b01}};
  integer fails=0;
  task setall(input [53:0] av, input [53:0] wv); integer c; begin
    acts=0; weights=0;
    for (c=0;c<8;c=c+1) begin acts[c*64 +: 64]={10'd0,av}; weights[c*64 +: 64]={10'd0,wv}; end
  end endtask
  task chk(input [7:0] got, input [7:0] exp, input [127:0] nm); begin
    if (got!==exp) begin fails=fails+1; $display("FAIL %0s got=%0d exp=%0d",nm,got,exp); end
    else $display("PASS %0s=%0d",nm,got); end
  endtask
  initial begin
    setall(P,P); chk(dut.neuronN(acts,weights,8,16'sd10), 2, "allP_x8");
    setall(P,N); chk(dut.neuronN(acts,weights,4,16'sd10), 0, "allP_x_N_x4");
    setall(N,N); chk(dut.neuronN(acts,weights,8,16'sd10), 2, "allN_x8");
    setall(Z,Z); chk(dut.neuronN(acts,weights,8,16'sd10), 1, "allZ");
    setall(P,P); chk(dut.neuronN(acts,weights,1,16'sd30), 1, "1chunk_band");
    setall(P,P); chk(dut.neuronN(acts,weights,0,16'sd10), 1, "0chunks");
    if (fails==0) $display("ALL_PASS"); else $display("FAILED %0d", fails);
    $finish;
  end
  BitnetNeuronN dut(.clk(1'b0), .rst_n(1'b1), .en(1'b1), .ready());
endmodule
"#;

#[test]
fn spec_first_neuron_n_accumulates_and_quantizes() {
    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping N-chunk neuron check");
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
        "gen-verilog of bitnet_neuron_nchunk.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    fs::write(dir.join("neuronN.v"), &gen.stdout).expect("write neuronN.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("neuronN.v"))
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
        "neuronN accumulation/quantize check failed:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "neuronN mismatch:\n{}", stdout);
}

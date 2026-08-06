// ============================================================================
// End-to-end check for the 2-layer spec-first BitNet inference
// (specs/ternary/bitnet_mlp.t27, function `mlp2`): layer 1 (3 neurons over the
// input activations) -> the 3 output trits are repacked into one hidden chunk
// -> layer 2 (2 single-chunk neurons) -> 2 packed output trits.
//
// A hand testbench cross-checks `mlp2` against a fully independent reference
// (both layers recomputed from scratch) over several uniform-chunk inputs, and
// pins a low-threshold case that exercises non-Z layer-2 outputs. Chunks are
// packed directly into the vectors (no array-literal call args, #1749). Skips
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
        .join("bitnet_mlp.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_mlp_{}_{}", std::process::id(), label));
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
  reg [511:0] acts, wa0, wa1, wa2;
  reg [63:0] wb0, wb1;
  localparam [53:0] P = {27{2'b10}};
  localparam [53:0] N = {27{2'b00}};
  localparam [53:0] Z = {27{2'b01}};
  integer fails=0;
  task fill(output [511:0] v, input [53:0] cv); integer k; begin
    v=0; for (k=0;k<8;k=k+1) v[k*64 +: 64]={10'd0,cv}; end
  endtask
  // Fully independent 2-layer reference.
  function [7:0] refmlp(input [511:0] a, input [511:0] wa0i, input [511:0] wa1i,
                        input [511:0] wa2i, input [63:0] wb0i, input [63:0] wb1i,
                        input [31:0] nc, input signed [15:0] thr);
    integer c, j, va, vb; reg signed [15:0] acc0,acc1,acc2,d0,d1;
    reg [7:0] q0,q1,q2,r0,r1; reg [1:0] ta,tb; reg [53:0] h; begin
    acc0=0;acc1=0;acc2=0;
    for (c=0;c<8;c=c+1) if (c<nc) begin
      for(j=0;j<27;j=j+1) begin ta=a[c*64+j*2 +:2]; tb=wa0i[c*64+j*2 +:2];
        va=(ta==0)?-1:(ta==2)?1:0; vb=(tb==0)?-1:(tb==2)?1:0; acc0=acc0+va*vb; end
      for(j=0;j<27;j=j+1) begin ta=a[c*64+j*2 +:2]; tb=wa1i[c*64+j*2 +:2];
        va=(ta==0)?-1:(ta==2)?1:0; vb=(tb==0)?-1:(tb==2)?1:0; acc1=acc1+va*vb; end
      for(j=0;j<27;j=j+1) begin ta=a[c*64+j*2 +:2]; tb=wa2i[c*64+j*2 +:2];
        va=(ta==0)?-1:(ta==2)?1:0; vb=(tb==0)?-1:(tb==2)?1:0; acc2=acc2+va*vb; end
    end
    q0=(acc0>thr)?2:(acc0<-thr)?0:1;
    q1=(acc1>thr)?2:(acc1<-thr)?0:1;
    q2=(acc2>thr)?2:(acc2<-thr)?0:1;
    h = Z; h[1:0]=q0[1:0]; h[3:2]=q1[1:0]; h[5:4]=q2[1:0];
    d0=0; for(j=0;j<27;j=j+1) begin ta=h[j*2 +:2]; tb=wb0i[j*2 +:2];
      va=(ta==0)?-1:(ta==2)?1:0; vb=(tb==0)?-1:(tb==2)?1:0; d0=d0+va*vb; end
    d1=0; for(j=0;j<27;j=j+1) begin ta=h[j*2 +:2]; tb=wb1i[j*2 +:2];
      va=(ta==0)?-1:(ta==2)?1:0; vb=(tb==0)?-1:(tb==2)?1:0; d1=d1+va*vb; end
    r0=(d0>thr)?2:(d0<-thr)?0:1;
    r1=(d1>thr)?2:(d1<-thr)?0:1;
    refmlp = (r1<<2)|r0; end
  endfunction
  reg [7:0] got, exp;
  task run(input [53:0] av, input [53:0] w0v, input [53:0] w1v, input [53:0] w2v,
           input [53:0] b0v, input [53:0] b1v, input [31:0] nc, input signed [15:0] thr,
           input [127:0] nm);
    begin
      fill(acts,av); fill(wa0,w0v); fill(wa1,w1v); fill(wa2,w2v);
      wb0={10'd0,b0v}; wb1={10'd0,b1v};
      got = dut.mlp2(acts,wa0,wa1,wa2,wb0,wb1,nc,thr);
      exp = refmlp(acts,wa0,wa1,wa2,wb0,wb1,nc,thr);
      if (got!==exp) begin fails=fails+1; $display("FAIL %0s got=%0d exp=%0d",nm,got,exp); end
      else $display("PASS %0s=%0d",nm,got);
    end
  endtask
  initial begin
    run(P,P,N,P, P,N, 4, 16'sd10, "c1");
    run(P,P,P,N, N,P, 2, 16'sd5,  "c2");
    run(Z,Z,Z,Z, Z,Z, 8, 16'sd10, "c3");
    run(N,P,N,P, P,N, 5, 16'sd7,  "c4");
    // Discriminating low-threshold case: layer2 dot=+3 > 2 -> both P -> (P<<2)|P = 10.
    run(P,P,P,P, P,P, 4, 16'sd2,  "c5_thr2");
    if (got !== 8'd10) begin fails=fails+1; $display("FAIL c5 output not P,P: %0d", got); end
    if (fails==0) $display("ALL_PASS"); else $display("FAILED %0d", fails);
    $finish;
  end
  BitnetMlp dut(.clk(1'b0), .rst_n(1'b1), .en(1'b1), .ready());
endmodule
"#;

#[test]
fn spec_first_mlp2_two_layer_inference_matches_reference() {
    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping BitNet MLP check");
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
        "gen-verilog of bitnet_mlp.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    fs::write(dir.join("mlp.v"), &gen.stdout).expect("write mlp.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("mlp.v"))
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
        "mlp2 two-layer inference did not match the reference:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "mlp2 mismatch:\n{}", stdout);
}

// ============================================================================
// End-to-end check for the 3-layer spec-first BitNet inference
// (specs/ternary/bitnet_mlp3.t27, function `mlp3`): L1 (3 neurons over the
// input) -> pack -> hidden h1 -> L2 (3 single-chunk neurons) -> pack -> hidden
// h2 -> L3 (2 single-chunk neurons) -> 2 packed output trits.
//
// Cross-checks `mlp3` against a fully independent 3-layer reference over several
// uniform-chunk inputs, including a low-threshold case where a +1 signal
// propagates through all three layers (all-P, thr=2 -> 10). Skips gracefully
// without iverilog/vvp.
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
        .join("bitnet_mlp3.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_mlp3_{}_{}", std::process::id(), label));
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
  reg [63:0] wb0,wb1,wb2,wc0,wc1;
  localparam [53:0] P={27{2'b10}}, N={27{2'b00}}, Z={27{2'b01}};
  integer fails=0;
  task fill(output [511:0] v, input [53:0] cv); integer k; begin
    v=0; for(k=0;k<8;k=k+1) v[k*64 +:64]={10'd0,cv}; end endtask
  function [15:0] dot(input [53:0] a, input [53:0] b);
    integer j,va,vb; reg[1:0] ta,tb; reg[15:0] s; begin s=0;
    for(j=0;j<27;j=j+1) begin ta=a[j*2 +:2]; tb=b[j*2 +:2];
      va=(ta==0)?-1:(ta==2)?1:0; vb=(tb==0)?-1:(tb==2)?1:0; s=s+va*vb; end
    dot=s; end endfunction
  function [7:0] q(input [15:0] v, input [15:0] t); begin
    if($signed(v)>$signed(t)) q=2; else if($signed(v)<-$signed(t)) q=0; else q=1; end endfunction
  function [53:0] pk(input [7:0] t0,t1,t2); reg[53:0] h; begin h=Z; h[1:0]=t0[1:0]; h[3:2]=t1[1:0]; h[5:4]=t2[1:0]; pk=h; end endfunction
  function [7:0] refmlp3(input [511:0] a, input[511:0] w0,w1,w2, input[63:0] b0,b1,b2,c0,c1, input[31:0] nc, input[15:0] thr);
    integer cc; reg[15:0] ac0,ac1,ac2; reg[7:0] q0,q1,q2,r0,r1,r2,s0,s1; reg[53:0] h1,h2; begin
    ac0=0;ac1=0;ac2=0;
    for(cc=0;cc<8;cc=cc+1) if(cc<nc) begin
      ac0=ac0+dot(a[cc*64 +:54], w0[cc*64 +:54]);
      ac1=ac1+dot(a[cc*64 +:54], w1[cc*64 +:54]);
      ac2=ac2+dot(a[cc*64 +:54], w2[cc*64 +:54]); end
    q0=q(ac0,thr); q1=q(ac1,thr); q2=q(ac2,thr);
    h1=pk(q0,q1,q2);
    r0=q(dot(h1,b0[53:0]),thr); r1=q(dot(h1,b1[53:0]),thr); r2=q(dot(h1,b2[53:0]),thr);
    h2=pk(r0,r1,r2);
    s0=q(dot(h2,c0[53:0]),thr); s1=q(dot(h2,c1[53:0]),thr);
    refmlp3=(s1<<2)|s0; end endfunction
  reg [7:0] got,exp;
  task run(input [53:0] av,w0v,w1v,w2v,b0v,b1v,b2v,c0v,c1v, input[31:0] nc, input[15:0] thr, input[63:0] nm); begin
    fill(acts,av);fill(wa0,w0v);fill(wa1,w1v);fill(wa2,w2v);
    wb0={10'd0,b0v};wb1={10'd0,b1v};wb2={10'd0,b2v};wc0={10'd0,c0v};wc1={10'd0,c1v};
    got=dut.mlp3(acts,wa0,wa1,wa2,wb0,wb1,wb2,wc0,wc1,nc,thr);
    exp=refmlp3(acts,wa0,wa1,wa2,wb0,wb1,wb2,wc0,wc1,nc,thr);
    if(got!==exp) begin fails=fails+1; $display("FAIL %0s got=%0d exp=%0d",nm,got,exp); end
    else $display("PASS %0s=%0d",nm,got); end endtask
  initial begin
    run(P,P,N,P, P,N,P, P,N, 4, 16'sd10, "c1");
    run(P,P,P,P, P,P,P, P,P, 4, 16'sd10, "c2_allP_hi");
    run(P,P,P,P, P,P,P, P,P, 4, 16'sd2,  "c3_allP_lo");
    run(Z,Z,Z,Z, Z,Z,Z, Z,Z, 8, 16'sd10, "c4_Z");
    run(N,P,N,P, N,P,N, P,N, 5, 16'sd3,  "c5");
    // The low-threshold all-P case must propagate +1 through all 3 layers.
    if (got !== 8'd0) ; // (last case is c5)
    fill(acts,P);fill(wa0,P);fill(wa1,P);fill(wa2,P);
    wb0={10'd0,P};wb1={10'd0,P};wb2={10'd0,P};wc0={10'd0,P};wc1={10'd0,P};
    if (dut.mlp3(acts,wa0,wa1,wa2,wb0,wb1,wb2,wc0,wc1,4,16'sd2) !== 8'd10)
      begin fails=fails+1; $display("FAIL propagate: not 10"); end
    if(fails==0) $display("ALL_PASS"); else $display("FAILED %0d",fails);
    $finish;
  end
  BitnetMlp3 dut(.clk(1'b0),.rst_n(1'b1),.en(1'b1),.ready());
endmodule
"#;

#[test]
fn spec_first_mlp3_three_layer_inference_matches_reference() {
    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping BitNet 3-layer MLP check");
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
        "gen-verilog of bitnet_mlp3.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    fs::write(dir.join("mlp3.v"), &gen.stdout).expect("write mlp3.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("mlp3.v"))
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
        "mlp3 three-layer inference did not match the reference:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "mlp3 mismatch:\n{}", stdout);
}

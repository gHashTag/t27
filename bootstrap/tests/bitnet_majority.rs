// ============================================================================
// Exhaustive check for the spec-first ternary majority gate
// (specs/ternary/bitnet_majority.t27, function `maj3`): the ternary majority of
// three trits = sign of (a + b + c), realized as a single BitNet neuron (pack ->
// dot with all-+1 -> quantize at 0). A recognizable named function computed by
// the spec-first stack.
//
// Drives all 3^3 = 27 input combinations and checks against an independent
// reference (decode {N=-1,Z=0,P=+1}, sum, sign). Skips without iverilog/vvp.
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
        .join("bitnet_majority.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_maj_{}_{}", std::process::id(), label));
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
  integer a,b,c,va,vb,vc,s,fails,n; reg [7:0] got,exp;
  function integer dec(input [7:0] t); begin dec=(t==0)?-1:(t==2)?1:0; end endfunction
  initial begin
    fails=0; n=0;
    // maj3 = sign(a + b + c)
    for (a=0;a<3;a=a+1) for (b=0;b<3;b=b+1) for (c=0;c<3;c=c+1) begin
      va=dec(a); vb=dec(b); vc=dec(c); s=va+vb+vc;
      exp = (s>0)?8'd2:(s<0)?8'd0:8'd1;
      got = dut.maj3(a[7:0], b[7:0], c[7:0]);
      n=n+1;
      if (got!==exp) begin fails=fails+1; $display("FAIL maj3 a=%0d b=%0d c=%0d got=%0d exp=%0d",a,b,c,got,exp); end
    end
    // weighted_vote = sign(a + b - c) (weights [+1,+1,-1])
    for (a=0;a<3;a=a+1) for (b=0;b<3;b=b+1) for (c=0;c<3;c=c+1) begin
      va=dec(a); vb=dec(b); vc=dec(c); s=va+vb-vc;
      exp = (s>0)?8'd2:(s<0)?8'd0:8'd1;
      got = dut.weighted_vote(a[7:0], b[7:0], c[7:0]);
      n=n+1;
      if (got!==exp) begin fails=fails+1; $display("FAIL wv a=%0d b=%0d c=%0d got=%0d exp=%0d",a,b,c,got,exp); end
    end
    if (fails==0) $display("ALL_PASS %0d", n); else $display("FAILED %0d/%0d", fails, n);
    $finish;
  end
  BitnetMajority dut(.clk(1'b0), .rst_n(1'b1), .en(1'b1), .ready());
endmodule
"#;

#[test]
fn spec_first_named_functions_exhaustive() {
    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping ternary majority check");
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
        "gen-verilog of bitnet_majority.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    fs::write(dir.join("maj.v"), &gen.stdout).expect("write maj.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("maj.v"))
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
        stdout.contains("ALL_PASS 54"),
        "maj3 did not match ternary majority on all 27+27 inputs:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "maj3 mismatch:\n{}", stdout);
}

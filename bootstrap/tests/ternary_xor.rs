// ============================================================================
// Check for the spec-first ternary XOR (specs/ternary/ternary_xor.t27,
// `ternary_xor`). XOR is NOT linearly separable, so a single neuron cannot
// compute it -- this is a genuine 2-layer network. On binary-embedded inputs
// {N=-1, P=+1} the output must be exact XOR (P if inputs differ, N if match).
//
// Drives all 3^2 = 9 (a,b) input combinations and checks against an independent
// reference that recomputes the same 2-layer construction, and asserts the four
// binary cases equal true XOR. Skips without iverilog/vvp.
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
        .join("ternary_xor.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_xor_{}_{}", std::process::id(), label));
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
  integer a,b,va,vb,h1,h2,o,fails,n; reg [7:0] got,exp;
  function integer dec(input [7:0] t); begin dec=(t==0)?-1:(t==2)?1:0; end endfunction
  function [7:0] sg(input integer v); begin sg=(v>0)?8'd2:(v<0)?8'd0:8'd1; end endfunction
  initial begin
    fails=0; n=0;
    for (a=0;a<3;a=a+1) for (b=0;b<3;b=b+1) begin
      va=dec(a); vb=dec(b);
      // reference: the same 2-layer construction
      h1 = (va+vb-1 > 0) ? 1 : (va+vb-1 < 0) ? -1 : 0;   // sign(a+b-1)
      h2 = (va+vb+1 > 0) ? 1 : (va+vb+1 < 0) ? -1 : 0;   // sign(a+b+1)
      o  = h2 + (-h1) - 1;                                // h2 AND NOT h1
      exp = sg(o);
      got = dut.ternary_xor(a[7:0], b[7:0]);
      n=n+1;
      if (got!==exp) begin fails=fails+1; $display("FAIL a=%0d b=%0d got=%0d exp=%0d",a,b,got,exp); end
    end
    // The four BINARY cases must equal true XOR (differ -> P, match -> N).
    if (dut.ternary_xor(8'd2,8'd2)!==8'd0) begin fails=fails+1; $display("FAIL xor(P,P)!=N"); end
    if (dut.ternary_xor(8'd2,8'd0)!==8'd2) begin fails=fails+1; $display("FAIL xor(P,N)!=P"); end
    if (dut.ternary_xor(8'd0,8'd2)!==8'd2) begin fails=fails+1; $display("FAIL xor(N,P)!=P"); end
    if (dut.ternary_xor(8'd0,8'd0)!==8'd0) begin fails=fails+1; $display("FAIL xor(N,N)!=N"); end
    if (fails==0) $display("ALL_PASS %0d", n); else $display("FAILED %0d", fails);
    $finish;
  end
  TernaryXor dut(.clk(1'b0), .rst_n(1'b1), .en(1'b1), .ready());
endmodule
"#;

#[test]
fn spec_first_ternary_xor_is_a_two_layer_xor() {
    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping ternary XOR check");
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
        "gen-verilog of ternary_xor.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    fs::write(dir.join("xor.v"), &gen.stdout).expect("write xor.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("xor.v"))
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
        stdout.contains("ALL_PASS 9"),
        "ternary_xor did not match the reference / true XOR:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "ternary_xor mismatch:\n{}", stdout);
}

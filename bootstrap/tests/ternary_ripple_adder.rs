// ============================================================================
// Check for the spec-first 2-bit ternary ripple-carry adder
// (specs/ternary/ternary_ripple_adder.t27, `add2`): two full adders (each
// XOR + majority) with the carry threaded between them, over trit-embedded
// binary bits {0 -> N, 1 -> P}. Output packs sum0 in bits[1:0], sum1 in [3:2],
// carry-out in [5:4]. A real multi-bit arithmetic datapath built entirely from
// the spec-first ternary stack.
//
// Exhaustively drives all 2^4 = 16 (a1a0, b1b0) input pairs and checks against
// binary addition. Skips without iverilog/vvp.
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
        .join("ternary_ripple_adder.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_add2_{}_{}", std::process::id(), label));
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

// For each 2-bit a=(a1,a0), b=(b1,b0): s = a + b (0..6). Expected packed trits:
// (s0?P:N) | (s1?P:N)<<2 | (s2?P:N)<<4, trit code P=2, N=0. Inputs to add2 are
// trit codes: bit 1 -> 2, bit 0 -> 0.
const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  integer a0,a1,b0,b1,av,bv,s,s0,s1,s2,fails,n; reg [7:0] got,exp;
  function [7:0] tc(input integer bit_); begin tc=(bit_==1)?8'd2:8'd0; end endfunction
  initial begin
    fails=0; n=0;
    for (a0=0;a0<2;a0=a0+1) for (a1=0;a1<2;a1=a1+1)
    for (b0=0;b0<2;b0=b0+1) for (b1=0;b1<2;b1=b1+1) begin
      av = a1*2 + a0; bv = b1*2 + b0; s = av + bv;
      s0 = s & 1; s1 = (s >> 1) & 1; s2 = (s >> 2) & 1;
      exp = tc(s0) | (tc(s1) << 2) | (tc(s2) << 4);
      got = dut.add2(tc(a0), tc(a1), tc(b0), tc(b1));
      n=n+1;
      if (got!==exp) begin fails=fails+1; $display("FAIL a=%0d b=%0d got=%0d exp=%0d",av,bv,got,exp); end
    end
    if (fails==0) $display("ALL_PASS %0d", n); else $display("FAILED %0d", fails);
    $finish;
  end
  TernaryRippleAdder dut(.clk(1'b0), .rst_n(1'b1), .en(1'b1), .ready());
endmodule
"#;

#[test]
fn spec_first_add2_matches_binary_addition_exhaustive() {
    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping ripple adder check");
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
        "gen-verilog of ternary_ripple_adder.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    fs::write(dir.join("add2.v"), &gen.stdout).expect("write add2.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("add2.v"))
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
        stdout.contains("ALL_PASS 16"),
        "add2 did not match binary addition on all 16 input pairs:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "add2 mismatch:\n{}", stdout);
}

// ============================================================================
// Check for the spec-first ternary full adder (specs/ternary/ternary_full_adder
// .t27, `full_adder`). A binary full adder over trit-embedded bits {0 -> N,
// 1 -> P}: sum = a XOR b XOR cin (composed from the 2-layer XOR), carry =
// majority(a, b, cin) (a single neuron). Output packs sum in bits[1:0], carry
// in bits[3:2].
//
// Exhaustively drives all 2^3 = 8 binary inputs and checks against the arithmetic
// full-adder truth table. Skips without iverilog/vvp.
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
        .join("ternary_full_adder.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_fa_{}_{}", std::process::id(), label));
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

// For each binary (a,b,cin) with 0->N(code 0), 1->P(code 2): sum_bit = parity,
// carry_bit = (a+b+cin >= 2). Packed output = (carry_trit << 2) | sum_trit,
// trit code: 0->0(N), 1->2(P).
const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  integer a,b,c,s,cy,fails,n; reg [7:0] ea,eb,ec,got,exp,st,ct;
  initial begin
    fails=0; n=0;
    for (a=0;a<2;a=a+1) for (b=0;b<2;b=b+1) for (c=0;c<2;c=c+1) begin
      s  = (a + b + c) % 2;         // sum bit
      cy = ((a + b + c) >= 2)?1:0;  // carry bit
      st = (s==1)?8'd2:8'd0;        // trit code for the sum bit
      ct = (cy==1)?8'd2:8'd0;       // trit code for the carry bit
      exp = (ct << 2) | st;
      ea = (a==1)?8'd2:8'd0; eb = (b==1)?8'd2:8'd0; ec = (c==1)?8'd2:8'd0;
      got = dut.full_adder(ea, eb, ec);
      n=n+1;
      if (got!==exp) begin fails=fails+1; $display("FAIL a=%0d b=%0d c=%0d got=%0d exp=%0d",a,b,c,got,exp); end
    end
    if (fails==0) $display("ALL_PASS %0d", n); else $display("FAILED %0d", fails);
    $finish;
  end
  TernaryFullAdder dut(.clk(1'b0), .rst_n(1'b1), .en(1'b1), .ready());
endmodule
"#;

#[test]
fn spec_first_full_adder_matches_binary_truth_table() {
    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping full adder check");
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
        "gen-verilog of ternary_full_adder.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    fs::write(dir.join("fa.v"), &gen.stdout).expect("write fa.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("fa.v"))
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
        stdout.contains("ALL_PASS 8"),
        "full_adder did not match the binary full-adder truth table:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "full_adder mismatch:\n{}", stdout);
}

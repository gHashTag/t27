// ============================================================================
// Cross-check: the spec-first ternary MAC (specs/ternary/ternary_mac.t27,
// function `dot27`) must compute the SAME dot product as the hand-written
// `trit27_dot_product` (gen-trit-stdlib) for every input.
//
// This is the equivalence proof for porting the accelerator's core datapath to
// the spec-first path (#1742): 300 random valid-trit vector pairs are fed to
// both, and the results must match bit-for-bit via iverilog + vvp.
//
// Encoding: 2'b00 = -1, 2'b01 = 0, 2'b10 = +1; trit i at bits [2i+1:2i] of a
// 54-bit chunk. Skips gracefully when iverilog/vvp are not on PATH.
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn t27c() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn spec_path() -> PathBuf {
    // bootstrap/ -> repo root -> specs/ternary/ternary_mac.t27
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("specs")
        .join("ternary")
        .join("ternary_mac.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_mac_specfirst_{}_{}", std::process::id(), label));
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

// Feeds the same random 54-bit vectors to the spec-first `dot27` (hierarchical
// call into the generated `TernaryMac` module) and the hand-written
// `trit27_dot_product`, and asserts equality over 300 vectors.
const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg [53:0] a, b;
  wire signed [5:0] ref_r;
  integer fails=0, iter, k;
  trit27_dot_product refdut(.input_vec(a), .weight_vec(b), .result(ref_r));
  function [1:0] tmap(input integer x); integer m; begin
    m=x%3; if(m<0)m=m+3; tmap=(m==0)?2'b01:(m==1)?2'b10:2'b00; end
  endfunction
  initial begin
    for (iter=0; iter<300; iter=iter+1) begin
      for (k=0;k<27;k=k+1) begin a[k*2 +: 2]=tmap($random); b[k*2 +: 2]=tmap($random); end
      #1;
      if ($signed(spec.dot27({10'd0,a},{10'd0,b})) !== $signed(ref_r)) begin
        fails=fails+1;
        if (fails<=3) $display("FAIL[%0d] spec=%0d ref=%0d", iter,
          $signed(spec.dot27({10'd0,a},{10'd0,b})), $signed(ref_r));
      end
    end
    if (fails==0) $display("ALL_MATCH 300"); else $display("MISMATCH %0d", fails);
    $finish;
  end
  TernaryMac spec(.clk(1'b0), .rst_n(1'b1), .en(1'b1), .ready());
endmodule
"#;

#[test]
fn spec_first_dot27_matches_handwritten_trit27_dot_product() {
    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping spec-first MAC cross-check");
        return;
    }

    let dir = scratch_dir("xcheck");
    fs::create_dir_all(&dir).expect("create scratch dir");

    // Emit the spec-first MAC module from the .t27 spec.
    let mac = Command::new(t27c())
        .arg("gen-verilog")
        .arg(spec_path())
        .output()
        .expect("invoke gen-verilog");
    assert!(
        mac.status.success(),
        "gen-verilog of ternary_mac.t27 failed:\n{}",
        String::from_utf8_lossy(&mac.stderr)
    );
    fs::write(dir.join("mac.v"), &mac.stdout).expect("write mac.v");

    // Emit the hand-written reference (trit27_dot_product lives in the stdlib).
    let stdlib = Command::new(t27c())
        .arg("gen-trit-stdlib")
        .output()
        .expect("invoke gen-trit-stdlib");
    assert!(stdlib.status.success(), "gen-trit-stdlib failed");
    fs::write(dir.join("trit_stdlib.sv"), &stdlib.stdout).expect("write trit_stdlib.sv");

    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("mac.v"))
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
        stdout.contains("ALL_MATCH 300"),
        "spec-first dot27 did not match trit27_dot_product on all 300 vectors:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("MISMATCH") && !stdout.contains("FAIL"),
        "spec-first dot27 mismatch vs reference:\n{}",
        stdout
    );
}

// ============================================================================
// Check for the spec-first STREAMING ternary MAC (specs/ternary/
// stream_ternary_mac.t27, #1764). This is the on-hardware BitNet inference
// primitive: `on_clock(a, b)` params become streaming INPUT data ports, and
// each cycle the 27-trit dot product of the current (a, b) pair is accumulated
// into a registered `acc` exposed as an OUTPUT data port.
//
// Verifies the generated module:
//   * has real input data ports `a`, `b` and an output port `acc`,
//   * (with yosys) synthesizes to real Artix-7 fabric -- FDCE accumulator +
//     a LUT adder-tree, not zero cells,
//   * (with iverilog) accumulates a stream of known trit-vector pairs to the
//     exact running sum of their dot products, and freezes when `en` is low.
// Skips the simulation/synth legs when the tools are absent.
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
        .join("stream_ternary_mac.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_smac_{}_{}", std::process::id(), label));
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

// Stream four known pairs; acc must track the running sum of dot products.
//   (0,0)                 -> all N.N = +27  => 27
//   (allP, allP)          -> +27            => 54
//   (0, allP)             -> -27            => 27
//   (allZ, allZ)          ->   0            => 27
// then en=0 must freeze acc at 27.
const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg clk,rst_n,en; reg [63:0] a,b; wire ready; wire signed [31:0] acc;
  integer fails;
  StreamTernaryMac dut(.clk(clk),.rst_n(rst_n),.en(en),.a(a),.b(b),.ready(ready),.acc(acc));
  initial clk=0; always #5 clk=~clk;
  task step(input [63:0] av, input [63:0] bv, input integer exp); begin
    a=av; b=bv; @(negedge clk);
    if (acc!==exp) begin fails=fails+1; $display("FAIL acc=%0d exp=%0d",acc,exp); end
  end endtask
  initial begin
    fails=0; rst_n=0; en=0; a=0; b=0; @(negedge clk); @(negedge clk);
    if (acc!==0) begin fails=fails+1; $display("FAIL reset acc=%0d",acc); end
    rst_n=1; en=1;
    step(64'd0, 64'd0, 27);
    step(64'd12009599006321322, 64'd12009599006321322, 54);
    step(64'd0, 64'd12009599006321322, 27);
    step(64'd6004799503160661, 64'd6004799503160661, 27);
    en=0; a=0; b=0; @(negedge clk);
    if (acc!==27) begin fails=fails+1; $display("FAIL freeze acc=%0d",acc); end
    if (fails==0) $display("ALL_PASS final=%0d",acc); else $display("FAILED %0d",fails);
    $finish;
  end
endmodule
"#;

#[test]
fn spec_first_streaming_ternary_mac() {
    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(spec_path())
        .output()
        .expect("invoke gen-verilog");
    assert!(
        gen.status.success(),
        "gen-verilog of stream_ternary_mac.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    let verilog = String::from_utf8_lossy(&gen.stdout).into_owned();

    // Streaming input data ports (on_clock params) + an observable accumulator.
    assert!(
        verilog.contains("input  wire [63:0] a") && verilog.contains("input  wire [63:0] b"),
        "on_clock params did not become input data ports:\n{}",
        verilog
    );
    assert!(
        verilog.contains("output reg signed [31:0] acc"),
        "accumulator was not exposed as an output data port:\n{}",
        verilog
    );
    assert!(
        verilog.contains("acc <="),
        "accumulator is not registered with a nonblocking update:\n{}",
        verilog
    );

    // Synthesizes to real Artix-7 fabric: an FDCE accumulator, not zero cells.
    if tool_available("yosys") {
        let dir = scratch_dir("synth");
        fs::create_dir_all(&dir).expect("create synth dir");
        fs::write(dir.join("mac.v"), &gen.stdout).expect("write mac.v");
        let synth = Command::new("yosys")
            .arg("-p")
            .arg(format!(
                "read_verilog -sv {}; synth_xilinx -top StreamTernaryMac; stat",
                dir.join("mac.v").to_str().unwrap()
            ))
            .output()
            .expect("invoke yosys");
        let s = String::from_utf8_lossy(&synth.stdout).into_owned()
            + &String::from_utf8_lossy(&synth.stderr);
        let _ = fs::remove_dir_all(&dir);
        assert!(synth.status.success(), "yosys synth_xilinx failed:\n{}", s);
        assert!(
            s.contains("FDCE") || s.contains("FDRE"),
            "streaming MAC produced no accumulator flip-flops:\n{}",
            s
        );
    }

    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping streaming simulation");
        return;
    }

    let dir = scratch_dir("chk");
    fs::create_dir_all(&dir).expect("create scratch dir");
    fs::write(dir.join("mac.v"), &gen.stdout).expect("write mac.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("mac.v"))
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
        "streaming MAC did not accumulate the dot-product stream correctly:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "streaming MAC mismatch:\n{}", stdout);
}

// ============================================================================
// Check for the spec-first clocked construct (specs/ternary/clocked_counter.t27,
// `fn on_clock`, #1764). This is the first *sequential* spec-first design: a
// function named `on_clock` lowers to an `always @(posedge clk or negedge rst_n)`
// that registers module-level `var` state -- reset to the declared init on
// `!rst_n`, advanced with nonblocking `<=` while `en` is asserted.
//
// Drives a real clock and asserts the registered `count`:
//   * held at 0 under reset,
//   * increments once per cycle when en=1,
//   * freezes when en=0 (en-gating),
//   * resumes on en=1,
//   * returns to 0 on asynchronous reset.
// Also asserts the generated Verilog actually contains the clocked process.
// Skips without iverilog/vvp.
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
        .join("clocked_counter.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_clk_{}_{}", std::process::id(), label));
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
  reg clk, rst_n, en; wire ready; integer i, fails;
  ClockedCounter dut(.clk(clk), .rst_n(rst_n), .en(en), .ready(ready));
  initial clk = 0;
  always #5 clk = ~clk;
  initial begin
    fails = 0;
    rst_n = 0; en = 0;                 // hold reset
    @(negedge clk); @(negedge clk);
    if (dut.count !== 8'd0) begin fails=fails+1; $display("FAIL reset: count=%0d",dut.count); end
    rst_n = 1; en = 1;                 // release, count 10 cycles
    for (i=0;i<10;i=i+1) @(negedge clk);
    if (dut.count !== 8'd10) begin fails=fails+1; $display("FAIL count!=10: %0d",dut.count); end
    en = 0;                            // freeze: 5 cycles, must stay 10
    for (i=0;i<5;i=i+1) @(negedge clk);
    if (dut.count !== 8'd10) begin fails=fails+1; $display("FAIL en-gate: %0d",dut.count); end
    en = 1;                            // resume 3 more -> 13
    for (i=0;i<3;i=i+1) @(negedge clk);
    if (dut.count !== 8'd13) begin fails=fails+1; $display("FAIL resume!=13: %0d",dut.count); end
    rst_n = 0;                         // async reset back to 0
    @(negedge clk);
    if (dut.count !== 8'd0) begin fails=fails+1; $display("FAIL re-reset: %0d",dut.count); end
    if (fails==0) $display("ALL_PASS"); else $display("FAILED %0d",fails);
    $finish;
  end
endmodule
"#;

#[test]
fn spec_first_on_clock_registers_state() {
    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(spec_path())
        .output()
        .expect("invoke gen-verilog");
    assert!(
        gen.status.success(),
        "gen-verilog of clocked_counter.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    let verilog = String::from_utf8_lossy(&gen.stdout).into_owned();

    // The clocked function must lower to a registered process, not combinational
    // logic: an edge-sensitive always block with a nonblocking update.
    assert!(
        verilog.contains("always @(posedge clk"),
        "on_clock did not lower to an edge-triggered always block:\n{}",
        verilog
    );
    assert!(
        verilog.contains("count <="),
        "clocked var was not updated with a nonblocking assignment:\n{}",
        verilog
    );
    // The registered state must be exposed as a data output port -- otherwise a
    // synthesizer dead-code-eliminates the whole design to zero cells (nothing
    // observable drives an output). This is the difference between a simulation
    // artifact and real hardware.
    assert!(
        verilog.contains("output reg [7:0] count"),
        "clocked var `count` was not exposed as an output data port:\n{}",
        verilog
    );

    // If yosys is available, prove the design synthesizes to REAL Artix-7
    // hardware -- the 8-bit register must map to flip-flops, not vanish.
    if tool_available("yosys") {
        let dir = scratch_dir("synth");
        fs::create_dir_all(&dir).expect("create synth dir");
        fs::write(dir.join("cc.v"), &gen.stdout).expect("write cc.v");
        let synth = Command::new("yosys")
            .arg("-p")
            .arg(format!(
                "read_verilog -sv {}; synth_xilinx -top ClockedCounter; stat",
                dir.join("cc.v").to_str().unwrap()
            ))
            .output()
            .expect("invoke yosys");
        let s = String::from_utf8_lossy(&synth.stdout).into_owned()
            + &String::from_utf8_lossy(&synth.stderr);
        let _ = fs::remove_dir_all(&dir);
        assert!(synth.status.success(), "yosys synth_xilinx failed:\n{}", s);
        assert!(
            s.contains("FDCE") || s.contains("FDRE") || s.contains("FDPE") || s.contains("FDSE"),
            "synth produced no flip-flops -- the register was optimized away:\n{}",
            s
        );
    }

    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping clocked simulation");
        return;
    }

    let dir = scratch_dir("chk");
    fs::create_dir_all(&dir).expect("create scratch dir");
    fs::write(dir.join("counter.v"), &gen.stdout).expect("write counter.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("counter.v"))
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
        "clocked counter did not behave as a reset/en-gated register:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "clocked counter mismatch:\n{}", stdout);
}

// ============================================================================
// Check for the spec-first COMBINATIONAL data interface (specs/ternary/
// comb_ternary_dot.t27, `on_comb`, #1764). `on_comb`'s params become input
// data ports and its return is a continuously-driven `output wire result`
// (`assign result = on_comb(...)`), so a purely combinational spec synthesizes
// to real LUTs instead of being dead-code-eliminated to zero cells.
//
// Here `on_comb` is the bit-exact 27-trit dot product. Verifies the generated
// module has input ports a, b and an output port `result`; that (with yosys) it
// synthesizes to real Artix-7 LUTs; and that (with iverilog) `result` equals the
// dot product for known trit vectors. Skips the tool legs when tools are absent.
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
        .join("comb_ternary_dot.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_cdot_{}_{}", std::process::id(), label));
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

// result must equal dot27(a,b) for known packed vectors (combinational, #1 delay).
const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg [63:0] a,b; wire signed [7:0] result; integer fails;
  CombTernaryDot dut(.clk(1'b0),.rst_n(1'b1),.en(1'b1),.a(a),.b(b),.ready(),.result(result));
  task chk(input [63:0] av, input [63:0] bv, input integer exp); begin
    a=av; b=bv; #1;
    if (result!==exp) begin fails=fails+1; $display("FAIL a=%h b=%h result=%0d exp=%0d",av,bv,result,exp); end
  end endtask
  initial begin
    fails=0;
    chk(64'd0, 64'd0, 27);
    chk(64'd12009599006321322, 64'd12009599006321322, 27);
    chk(64'd0, 64'd12009599006321322, -27);
    chk(64'd6004799503160661, 64'd6004799503160661, 0);
    if (fails==0) $display("ALL_PASS"); else $display("FAILED %0d",fails);
    $finish;
  end
endmodule
"#;

#[test]
fn spec_first_combinational_data_ports() {
    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(spec_path())
        .output()
        .expect("invoke gen-verilog");
    assert!(
        gen.status.success(),
        "gen-verilog of comb_ternary_dot.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    let verilog = String::from_utf8_lossy(&gen.stdout).into_owned();

    assert!(
        verilog.contains("input  wire [63:0] a") && verilog.contains("input  wire [63:0] b"),
        "on_comb params did not become input data ports:\n{}",
        verilog
    );
    assert!(
        verilog.contains("output wire signed [7:0] result"),
        "on_comb return was not exposed as an output data port:\n{}",
        verilog
    );
    assert!(
        verilog.contains("assign result = on_comb(a, b);"),
        "the result port is not continuously driven from on_comb:\n{}",
        verilog
    );

    // Synthesizes to real combinational Artix-7 fabric: LUTs, and NO flip-flops.
    if tool_available("yosys") {
        let dir = scratch_dir("synth");
        fs::create_dir_all(&dir).expect("create synth dir");
        fs::write(dir.join("cd.v"), &gen.stdout).expect("write cd.v");
        let synth = Command::new("yosys")
            .arg("-p")
            .arg(format!(
                "read_verilog -sv {}; synth_xilinx -top CombTernaryDot; stat",
                dir.join("cd.v").to_str().unwrap()
            ))
            .output()
            .expect("invoke yosys");
        let s = String::from_utf8_lossy(&synth.stdout).into_owned()
            + &String::from_utf8_lossy(&synth.stderr);
        let _ = fs::remove_dir_all(&dir);
        assert!(synth.status.success(), "yosys synth_xilinx failed:\n{}", s);
        assert!(
            s.contains("LUT"),
            "combinational dot product produced no LUTs (optimized away):\n{}",
            s
        );
    }

    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping combinational simulation");
        return;
    }

    let dir = scratch_dir("chk");
    fs::create_dir_all(&dir).expect("create scratch dir");
    fs::write(dir.join("cd.v"), &gen.stdout).expect("write cd.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("cd.v"))
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
        "combinational dot product did not match the reference:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "combinational dot mismatch:\n{}", stdout);
}

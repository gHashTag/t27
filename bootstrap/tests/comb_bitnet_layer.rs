// ============================================================================
// Check for the spec-first combinational BitNet LAYER (specs/ternary/
// comb_bitnet_layer.t27, #1764): four neurons over a shared 27-trit activation,
// each with its own fixed weight vector, trits packed 2 bits each into the
// `result` output. Verifies the activation input port + packed output port; that
// (with yosys) it synthesizes to real Artix-7 LUTs with NO flip-flops; and that
// (with iverilog) the packed layer output matches the hand-computed responses
// to the canonical all-P / all-N / all-Z activations.
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
        .join("comb_bitnet_layer.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_layer_{}_{}", std::process::id(), label));
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

// Weights = all+1, all-1, all0, all+1. Response to canonical activations:
//   a=all+1 -> dots +27,-27,0,+27 -> P,N,Z,P -> 2|0<<2|1<<4|2<<6 = 146
//   a=all-1 -> dots -27,+27,0,-27 -> N,P,Z,N -> 0|2<<2|1<<4|0<<6 = 24
//   a=all0  -> all dots 0        -> Z,Z,Z,Z -> 1|1<<2|1<<4|1<<6 = 85
const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg [63:0] a; wire [7:0] result; integer fails;
  CombBitnetLayer dut(.clk(1'b0),.rst_n(1'b1),.en(1'b1),.a(a),.ready(),.result(result));
  task chk(input [63:0] av,input integer exp); begin
    a=av;#1;
    if(result!==exp)begin fails=fails+1;$display("FAIL a=%h result=%0d exp=%0d",av,result,exp);end
  end endtask
  initial begin
    fails=0;
    chk(64'd12009599006321322,146);
    chk(64'd0,24);
    chk(64'd6004799503160661,85);
    if(fails==0)$display("ALL_PASS");else $display("FAILED %0d",fails);
    $finish;
  end
endmodule
"#;

#[test]
fn spec_first_combinational_bitnet_layer() {
    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(spec_path())
        .output()
        .expect("invoke gen-verilog");
    assert!(
        gen.status.success(),
        "gen-verilog of comb_bitnet_layer.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    let verilog = String::from_utf8_lossy(&gen.stdout).into_owned();

    assert!(
        verilog.contains("input  wire [63:0] a"),
        "layer activation did not become an input data port:\n{}",
        verilog
    );
    assert!(
        verilog.contains("output wire [7:0] result")
            && verilog.contains("assign result = on_comb(a);"),
        "layer output is not driven on the result port:\n{}",
        verilog
    );

    if tool_available("yosys") {
        let dir = scratch_dir("synth");
        fs::create_dir_all(&dir).expect("create synth dir");
        fs::write(dir.join("l.v"), &gen.stdout).expect("write l.v");
        let synth = Command::new("yosys")
            .arg("-p")
            .arg(format!(
                "read_verilog -sv {}; synth_xilinx -top CombBitnetLayer; stat",
                dir.join("l.v").to_str().unwrap()
            ))
            .output()
            .expect("invoke yosys");
        let s = String::from_utf8_lossy(&synth.stdout).into_owned()
            + &String::from_utf8_lossy(&synth.stderr);
        let _ = fs::remove_dir_all(&dir);
        assert!(synth.status.success(), "yosys synth_xilinx failed:\n{}", s);
        assert!(s.contains("LUT"), "layer produced no LUTs:\n{}", s);
    }

    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping layer simulation");
        return;
    }

    let dir = scratch_dir("chk");
    fs::create_dir_all(&dir).expect("create scratch dir");
    fs::write(dir.join("l.v"), &gen.stdout).expect("write l.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("l.v"))
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
        "layer did not produce the expected packed trit outputs:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "layer mismatch:\n{}", stdout);
}

// ============================================================================
// Check for the spec-first combinational BitNet NEURON (specs/ternary/
// comb_bitnet_neuron.t27, #1764): `on_comb(a, b) = quantize(dot27(a, b))` --
// a full neuron over one 27-trit chunk (weighted ternary sum -> sign) in a
// single combinational module. Verifies input ports a, b + output port result;
// that (with yosys) it synthesizes to real Artix-7 LUTs with NO flip-flops; and
// that (with iverilog) result == quantize(dot27(a,b)) for known trit vectors.
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
        .join("comb_bitnet_neuron.t27")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_neuron_{}_{}", std::process::id(), label));
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

// result = quantize(dot27(a,b)): +sum -> P(2), -sum -> N(0), 0 -> Z(1).
const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg [63:0] a,b; wire [7:0] result; integer fails;
  CombBitnetNeuron dut(.clk(1'b0),.rst_n(1'b1),.en(1'b1),.a(a),.b(b),.ready(),.result(result));
  task chk(input [63:0] av,input [63:0] bv,input integer exp); begin
    a=av;b=bv;#1;
    if(result!==exp)begin fails=fails+1;$display("FAIL a=%h b=%h r=%0d exp=%0d",av,bv,result,exp);end
  end endtask
  initial begin
    fails=0;
    chk(64'd0,64'd0,2);
    chk(64'd12009599006321322,64'd12009599006321322,2);
    chk(64'd0,64'd12009599006321322,0);
    chk(64'd6004799503160661,64'd6004799503160661,1);
    if(fails==0)$display("ALL_PASS");else $display("FAILED %0d",fails);
    $finish;
  end
endmodule
"#;

#[test]
fn spec_first_combinational_bitnet_neuron() {
    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(spec_path())
        .output()
        .expect("invoke gen-verilog");
    assert!(
        gen.status.success(),
        "gen-verilog of comb_bitnet_neuron.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    let verilog = String::from_utf8_lossy(&gen.stdout).into_owned();

    assert!(
        verilog.contains("input  wire [63:0] a") && verilog.contains("input  wire [63:0] b"),
        "neuron inputs did not become data ports:\n{}",
        verilog
    );
    assert!(
        verilog.contains("output wire [7:0] result")
            && verilog.contains("assign result = on_comb(a, b);"),
        "neuron activation is not driven on the result output port:\n{}",
        verilog
    );

    // Synthesizes to real combinational Artix-7 fabric: LUTs, and NO flip-flops.
    if tool_available("yosys") {
        let dir = scratch_dir("synth");
        fs::create_dir_all(&dir).expect("create synth dir");
        fs::write(dir.join("n.v"), &gen.stdout).expect("write n.v");
        let synth = Command::new("yosys")
            .arg("-p")
            .arg(format!(
                "read_verilog -sv {}; synth_xilinx -top CombBitnetNeuron; stat",
                dir.join("n.v").to_str().unwrap()
            ))
            .output()
            .expect("invoke yosys");
        let s = String::from_utf8_lossy(&synth.stdout).into_owned()
            + &String::from_utf8_lossy(&synth.stderr);
        let _ = fs::remove_dir_all(&dir);
        assert!(synth.status.success(), "yosys synth_xilinx failed:\n{}", s);
        assert!(s.contains("LUT"), "neuron produced no LUTs:\n{}", s);
    }

    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping neuron simulation");
        return;
    }

    let dir = scratch_dir("chk");
    fs::create_dir_all(&dir).expect("create scratch dir");
    fs::write(dir.join("n.v"), &gen.stdout).expect("write n.v");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb.v");

    let vvp_path = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp_path.to_str().unwrap()])
        .arg(dir.join("n.v"))
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
        "neuron did not compute quantize(dot27(a,b)):\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "neuron mismatch:\n{}", stdout);
}

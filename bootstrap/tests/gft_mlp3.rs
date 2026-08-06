// ============================================================================
// Check for the spec-first 3-LAYER BitNet x GF-T MLP (specs/ternary/gft_mlp3.t27):
// 4 shared GF-T16 activations -> layer1 (3 neurons) -> re-embed -> layer2 (2
// neurons) -> re-embed -> layer3 (1 output neuron) -> output trit {N=0,Z=1,P=2}.
// Every neuron is sign(sum_i w_i * a_i) in signed GF-T (RNE). Bit-exact to the
// ideal oracle over 400 vectors (tests/gft_mlp3_vectors.txt).
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn t27c() -> &'static str { env!("CARGO_BIN_EXE_t27c") }
fn spec_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("specs").join("ternary").join("gft_mlp3.t27")
}
fn vectors_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("gft_mlp3_vectors.txt")
}
fn tool_available(t: &str) -> bool {
    Command::new(t).arg("-V").output().map(|o| o.status.success()).unwrap_or(false)
}

#[test]
fn spec_first_gft_mlp3_matches_oracle() {
    let gen = Command::new(t27c()).arg("gen-verilog").arg(spec_path()).output().expect("gen-verilog");
    assert!(gen.status.success(), "gen-verilog failed:\n{}", String::from_utf8_lossy(&gen.stderr));
    let v = String::from_utf8_lossy(&gen.stdout).into_owned();
    assert!(
        v.contains("input  wire [7:0] w1_00") && v.contains("input  wire [31:0] a3")
            && v.contains("output wire [7:0] result"),
        "mlp3 missing weight/activation -> trit interface:\n{}", v
    );

    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping mlp3 check");
        return;
    }

    let vectors = fs::read_to_string(vectors_path()).expect("read vectors");
    let n = vectors.lines().filter(|l| !l.trim().is_empty()).count();
    let dir = env::temp_dir().join(format!("t27_mlp3_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("spec.v"), &gen.stdout).unwrap();
    fs::write(dir.join("vec.txt"), &vectors).unwrap();

    let tb = format!(
        r#"`timescale 1ns/1ps
module tb;
  reg [7:0] w1_00,w1_01,w1_02,w1_03,w1_10,w1_11,w1_12,w1_13,w1_20,w1_21,w1_22,w1_23;
  reg [7:0] w2_00,w2_01,w2_02,w2_10,w2_11,w2_12,w3_0,w3_1;
  reg [31:0] a0,a1,a2,a3; wire [7:0] y; integer fails,nn,fd,code; reg [7:0] exp;
  GftMlp3 dut(.clk(1'b0),.rst_n(1'b1),.en(1'b1),
    .w1_00(w1_00),.w1_01(w1_01),.w1_02(w1_02),.w1_03(w1_03),
    .w1_10(w1_10),.w1_11(w1_11),.w1_12(w1_12),.w1_13(w1_13),
    .w1_20(w1_20),.w1_21(w1_21),.w1_22(w1_22),.w1_23(w1_23),
    .w2_00(w2_00),.w2_01(w2_01),.w2_02(w2_02),
    .w2_10(w2_10),.w2_11(w2_11),.w2_12(w2_12),
    .w3_0(w3_0),.w3_1(w3_1),
    .a0(a0),.a1(a1),.a2(a2),.a3(a3),.ready(),.result(y));
  initial begin
    fails=0; nn=0; fd=$fopen("{}","r");
    while(!$feof(fd)) begin
      code=$fscanf(fd,"%d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %h %h %h %h %d\n",
        w1_00,w1_01,w1_02,w1_03,w1_10,w1_11,w1_12,w1_13,w1_20,w1_21,w1_22,w1_23,
        w2_00,w2_01,w2_02,w2_10,w2_11,w2_12,w3_0,w3_1,a0,a1,a2,a3,exp);
      if(code==25) begin #1; nn=nn+1;
        if(y!==exp) begin fails=fails+1; if(fails<=6)$display("FAIL y=%0d exp=%0d",y,exp); end
      end end
    $fclose(fd);
    if(fails==0)$display("ALL_PASS %0d",nn); else $display("FAILED %0d/%0d",fails,nn);
    $finish;
  end
endmodule
"#,
        dir.join("vec.txt").to_str().unwrap()
    );
    fs::write(dir.join("tb.v"), tb).unwrap();

    let vvp = dir.join("sim.vvp");
    let compile = Command::new("iverilog").args(["-g2012", "-o", vvp.to_str().unwrap()])
        .arg(dir.join("spec.v")).arg(dir.join("tb.v")).output().unwrap();
    assert!(compile.status.success(), "iverilog compile failed:\n{}", String::from_utf8_lossy(&compile.stderr));
    let run = Command::new("vvp").arg(&vvp).output().unwrap();
    let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
    let _ = fs::remove_dir_all(&dir);
    assert!(stdout.contains(&format!("ALL_PASS {}", n)), "mlp3 differs from the oracle:\n{}", stdout);
    assert!(!stdout.contains("FAIL"), "mlp3 mismatch:\n{}", stdout);
}

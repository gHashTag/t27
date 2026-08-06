// ============================================================================
// Check for the spec-first COMPLETE BitNet x GF-T neuron (specs/ternary/
// gft_neuron_full.t27): ternary weights x real-valued GF-T16 activations summed
// in signed GF-T (RNE), then a sign activation quantizes the sum to a TRIT
// output {N=0,Z=1,P=2} -- layer-composable (trit in weights, trit out). Bit-exact
// to the ideal oracle over 300 vectors (tests/gft_neuron_full_vectors.txt).
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn t27c() -> &'static str { env!("CARGO_BIN_EXE_t27c") }
fn spec_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("specs").join("ternary").join("gft_neuron_full.t27")
}
fn vectors_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("gft_neuron_full_vectors.txt")
}
fn tool_available(t: &str) -> bool {
    Command::new(t).arg("-V").output().map(|o| o.status.success()).unwrap_or(false)
}

#[test]
fn spec_first_gft_neuron_full_matches_oracle() {
    let gen = Command::new(t27c()).arg("gen-verilog").arg(spec_path()).output().expect("gen-verilog");
    assert!(gen.status.success(), "gen-verilog failed:\n{}", String::from_utf8_lossy(&gen.stderr));
    let v = String::from_utf8_lossy(&gen.stdout).into_owned();
    assert!(
        v.contains("input  wire [7:0] w1") && v.contains("output wire [7:0] result"),
        "full neuron missing weight/activation -> trit interface:\n{}", v
    );

    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping full neuron check");
        return;
    }

    let vectors = fs::read_to_string(vectors_path()).expect("read vectors");
    let n = vectors.lines().filter(|l| !l.trim().is_empty()).count();
    let dir = env::temp_dir().join(format!("t27_nf_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("spec.v"), &gen.stdout).unwrap();
    fs::write(dir.join("vec.txt"), &vectors).unwrap();

    let tb = format!(
        r#"`timescale 1ns/1ps
module tb;
  reg [7:0] w1,w2,w3,w4; reg [31:0] a1,a2,a3,a4; wire [7:0] y; integer fails,nn,fd,code; reg [7:0] exp;
  GftNeuronFull dut(.clk(1'b0),.rst_n(1'b1),.en(1'b1),.w1(w1),.a1(a1),.w2(w2),.a2(a2),.w3(w3),.a3(a3),.w4(w4),.a4(a4),.ready(),.result(y));
  initial begin
    fails=0; nn=0; fd=$fopen("{}","r");
    while(!$feof(fd)) begin code=$fscanf(fd,"%d %h %d %h %d %h %d %h %d\n",w1,a1,w2,a2,w3,a3,w4,a4,exp);
      if(code==9) begin #1; nn=nn+1;
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
    assert!(stdout.contains(&format!("ALL_PASS {}", n)), "full neuron differs from the oracle:\n{}", stdout);
    assert!(!stdout.contains("FAIL"), "full neuron mismatch:\n{}", stdout);
}

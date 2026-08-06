// ============================================================================
// Check for the spec-first GF-T recip primitive (specs/ternary/gft_recip.t27):
// 1/x for a signed GF-T16 input, result a signed GF-T16. The missing building
// block for a GF-T softmax. Bit-exact to the committed integer oracle over 606
// vectors (tests/gft_recip_vectors.txt); the oracle is itself <=1 ULP vs the true
// true 1/x (exact, 0 ULP) (measured in the prototype).
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn t27c() -> &'static str { env!("CARGO_BIN_EXE_t27c") }
fn spec_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("specs").join("ternary").join("gft_recip.t27")
}
fn vectors_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("gft_recip_vectors.txt")
}
fn tool_available(t: &str) -> bool {
    Command::new(t).arg("-V").output().map(|o| o.status.success()).unwrap_or(false)
}

#[test]
fn spec_first_gft_recip_matches_oracle() {
    let gen = Command::new(t27c()).arg("gen-verilog").arg(spec_path()).output().expect("gen-verilog");
    assert!(gen.status.success(), "gen-verilog failed:\n{}", String::from_utf8_lossy(&gen.stderr));
    let v = String::from_utf8_lossy(&gen.stdout).into_owned();
    assert!(
        v.contains("input  wire [31:0] x") && v.contains("output wire [31:0] result"),
        "recip missing x -> result interface:\n{}", v
    );

    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping recip check");
        return;
    }

    let vectors = fs::read_to_string(vectors_path()).expect("read vectors");
    let n = vectors.lines().filter(|l| !l.trim().is_empty()).count();
    let dir = env::temp_dir().join(format!("t27_recip_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("spec.v"), &gen.stdout).unwrap();
    fs::write(dir.join("vec.txt"), &vectors).unwrap();

    let tb = format!(
        r#"`timescale 1ns/1ps
module tb;
  reg [31:0] x; wire [31:0] y; integer fails,nn,fd,code; reg [31:0] exp;
  GftRecip dut(.clk(1'b0),.rst_n(1'b1),.en(1'b1),.x(x),.ready(),.result(y));
  initial begin
    fails=0; nn=0; fd=$fopen("{}","r");
    while(!$feof(fd)) begin code=$fscanf(fd,"%h %h\n",x,exp);
      if(code==2) begin #1; nn=nn+1;
        if(y!==exp) begin fails=fails+1; if(fails<=6)$display("FAIL x=%h y=%h exp=%h",x,y,exp); end
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
    assert!(stdout.contains(&format!("ALL_PASS {}", n)), "recip differs from the oracle:\n{}", stdout);
    assert!(!stdout.contains("FAIL"), "recip mismatch:\n{}", stdout);
}

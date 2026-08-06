// ============================================================================
// Check for the spec-first ROUND-TO-NEAREST-EVEN GF-T16 multiply (specs/ternary/
// gft_mul_rne.t27): a GF-T mul that rounds the mantissa to nearest-even, matching
// the IDEAL oracle (trinity-fpga/conformance/gft16_ref.py) -- and therefore MORE
// ACCURATE than the truncating silicon gft_mul (which is ~1 ULP low, ~37% of
// products off-by-one). Verified bit-exact against 300 oracle-generated
// normal-range vectors (tests/gft_mul_rne_vectors.txt, ~half of which differ from
// the truncating silicon). Skips without iverilog/vvp.
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
        .join("gft_mul_rne.t27")
}

fn vectors_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("gft_mul_rne_vectors.txt")
}

fn tool_available(t: &str) -> bool {
    Command::new(t)
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn spec_first_gft_mul_rne_matches_oracle() {
    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(spec_path())
        .output()
        .expect("invoke gen-verilog");
    assert!(
        gen.status.success(),
        "gen-verilog of gft_mul_rne.t27 failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    let verilog = String::from_utf8_lossy(&gen.stdout).into_owned();
    assert!(
        verilog.contains("input  wire [15:0] a") && verilog.contains("output wire [15:0] result"),
        "GF-T RNE mul did not expose the a,b -> result interface:\n{}",
        verilog
    );

    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP: iverilog/vvp not on PATH; skipping GF-T RNE oracle check");
        return;
    }

    let vectors = fs::read_to_string(vectors_path()).expect("read oracle vectors");
    let n_vectors = vectors.lines().filter(|l| !l.trim().is_empty()).count();

    let dir = env::temp_dir().join(format!("t27_rne_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch dir");
    fs::write(dir.join("spec.v"), &gen.stdout).expect("write spec.v");
    fs::write(dir.join("vec.txt"), &vectors).expect("write vec.txt");

    let tb = format!(
        r#"`timescale 1ns/1ps
module tb;
  reg [15:0] a,b; wire [15:0] y; integer fails,n,fd,code; reg [15:0] exp;
  GftMulRne dut(.clk(1'b0),.rst_n(1'b1),.en(1'b1),.a(a),.b(b),.ready(),.result(y));
  initial begin
    fails=0; n=0; fd=$fopen("{}","r");
    while(!$feof(fd)) begin code=$fscanf(fd,"%h %h %h\n",a,b,exp);
      if(code==3) begin #1; n=n+1;
        if(y!==exp) begin fails=fails+1; if(fails<=6)$display("FAIL a=%h b=%h y=%h exp=%h",a,b,y,exp); end
      end end
    $fclose(fd);
    if(fails==0)$display("ALL_PASS %0d",n); else $display("FAILED %0d/%0d",fails,n);
    $finish;
  end
endmodule
"#,
        dir.join("vec.txt").to_str().unwrap()
    );
    fs::write(dir.join("tb.v"), tb).expect("write tb.v");

    let vvp = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp.to_str().unwrap()])
        .arg(dir.join("spec.v"))
        .arg(dir.join("tb.v"))
        .output()
        .expect("invoke iverilog");
    assert!(
        compile.status.success(),
        "iverilog compile failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new("vvp").arg(&vvp).output().expect("invoke vvp");
    let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
    let _ = fs::remove_dir_all(&dir);

    assert!(
        stdout.contains(&format!("ALL_PASS {}", n_vectors)),
        "RNE GF-T mul differs from the oracle:\n{}",
        stdout
    );
    assert!(!stdout.contains("FAIL"), "RNE GF-T mul mismatch:\n{}", stdout);
}

// ============================================================================
// #1745 regression: indexing an array-typed parameter must produce a Verilog
// element part-select, not a bit-select.
//
// A `[N]T` parameter lowers to a packed `input [N*W-1:0]` vector. Before the
// fix, `xs[i]` emitted `xs[i]` -- a single-bit select -- so it read one bit
// instead of the W-bit element (`sum_arr([1,2,3,4])` returned 1, not 10). The
// fix emits `xs[i*W +: W]`. Skips gracefully when iverilog/vvp are absent.
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn t27c() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_arr_param_{}_{}", std::process::id(), label));
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

const SPEC: &str = r#"module ArrParam;
pub fn sum_arr(xs: [4]u64) -> u64 {
    var acc : u64 = 0;
    var i : u32 = 0;
    while (i < 4) {
        acc = acc + xs[i];
        i = i + 1;
    }
    return acc;
}
endmodule
"#;

const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg [255:0] xs;
  initial begin
    xs = 0;
    xs[63:0]   = 64'd1;
    xs[127:64] = 64'd2;
    xs[191:128]= 64'd3;
    xs[255:192]= 64'd4;
    #1;
    if (dut.sum_arr(xs) === 64'd10) $display("PASS sum=%0d", dut.sum_arr(xs));
    else $display("FAIL sum=%0d exp 10", dut.sum_arr(xs));
    $finish;
  end
  ArrParam dut(.clk(1'b0), .rst_n(1'b1), .en(1'b1), .ready());
endmodule
"#;

#[test]
fn array_param_index_is_element_part_select() {
    let dir = scratch_dir("idx");
    fs::create_dir_all(&dir).expect("create scratch dir");
    let spec = dir.join("arrparam.t27");
    fs::write(&spec, SPEC).expect("write spec");

    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(&spec)
        .output()
        .expect("invoke gen-verilog");
    assert!(
        gen.status.success(),
        "gen-verilog failed:\n{}",
        String::from_utf8_lossy(&gen.stderr)
    );
    let v = String::from_utf8_lossy(&gen.stdout).into_owned();

    // The index must be an element-wide part-select, never a bare bit-select.
    assert!(
        v.contains("+: 64]"),
        "array-param index did not lower to a 64-bit part-select:\n{}",
        v
    );

    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP(sim): iverilog/vvp not on PATH");
        let _ = fs::remove_dir_all(&dir);
        return;
    }

    fs::write(dir.join("arrparam.v"), v.as_bytes()).expect("write verilog");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb");
    let vvp = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-o", vvp.to_str().unwrap()])
        .arg(dir.join("arrparam.v"))
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
        stdout.contains("PASS sum=10"),
        "array-param element indexing computed the wrong sum:\n{}",
        stdout
    );
}

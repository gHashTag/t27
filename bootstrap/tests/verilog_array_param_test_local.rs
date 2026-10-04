// ============================================================================
// #5908 regression: an array argument named inside a TEST block is not a
// module-level array, and must not be bound as one.
//
// W458/W459 bind a `[N]T` parameter to the module-level array named at its
// call sites, so the function reads the ROM by name and takes no `input` for
// it. W459 also reads the call sites inside test/invariant/bench blocks. A
// test's `given xs = [1, 2, 3, 4]` is local to the test -- `gen-verilog` does
// not lower it at all -- yet any identifier was accepted, so the function lost
// `input [127:0] xs;` and its body indexed a name declared nowhere:
//
//   error: Unable to bind wire/reg/memory `xs[i]' in `LocalArr.total...'
//
// That is how specs/fpga/ternary_isa.t27 went from 6 to 7 elaboration errors
// the day it gained such a test (#4594), and the FPGA elaboration ratchet went
// red. The fix binds only to a module-level const or var; anything else is
// passed by value (#1745 part-selects).
//
// Two controls, so neither half can pass by accident:
//   * the test-local array is passed by value: `input [127:0] xs;` is
//     declared, iverilog elaborates it, and `total({4,3,2,1}, 4)` is 10;
//   * a module-level array is still bound: no `xs` input, the body reads
//     `TABLE[i]`, which is the W458 contract and must not regress.
// Skips the iverilog half gracefully when iverilog/vvp are absent.
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn t27c() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_arr_local_{}_{}", std::process::id(), label));
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

const TEST_LOCAL: &str = r#"module LocalArr {
    fn total(xs: [4]u32, n: u32) -> u32 {
        var acc : u32 = 0;
        var i : u32 = 0;
        while (i < n) {
            acc = acc + xs[i];
            i = i + 1;
        }
        return acc;
    }

    test total_of_a_test_local_array
        given xs = [1, 2, 3, 4]
        then total(xs, 4) == 10
}
"#;

const MODULE_ARRAY: &str = r#"module ModArr {
    const TABLE : [4]u32 = [1, 2, 3, 4];

    fn total(xs: [4]u32, n: u32) -> u32 {
        var acc : u32 = 0;
        var i : u32 = 0;
        while (i < n) {
            acc = acc + xs[i];
            i = i + 1;
        }
        return acc;
    }

    test total_of_the_module_table
        then total(TABLE, 4) == 10
}
"#;

const TESTBENCH: &str = r#"`timescale 1ns/1ps
module tb;
  reg [127:0] xs;
  initial begin
    xs = 0;
    xs[31:0]   = 32'd1;
    xs[63:32]  = 32'd2;
    xs[95:64]  = 32'd3;
    xs[127:96] = 32'd4;
    #1;
    if (dut.total(xs, 32'd4) === 32'd10) $display("PASS total=%0d", dut.total(xs, 32'd4));
    else $display("FAIL total=%0d exp 10", dut.total(xs, 32'd4));
    $finish;
  end
  LocalArr dut(.clk(1'b0), .rst_n(1'b1), .en(1'b1), .ready());
endmodule
"#;

fn gen_verilog(dir: &PathBuf, name: &str, spec: &str) -> String {
    let path = dir.join(name);
    fs::write(&path, spec).expect("write spec");
    let gen = Command::new(t27c())
        .arg("gen-verilog")
        .arg(&path)
        .output()
        .expect("invoke gen-verilog");
    assert!(
        gen.status.success(),
        "gen-verilog {} failed:\n{}",
        name,
        String::from_utf8_lossy(&gen.stderr)
    );
    String::from_utf8_lossy(&gen.stdout).into_owned()
}

#[test]
fn test_local_array_argument_is_passed_by_value() {
    let dir = scratch_dir("local");
    fs::create_dir_all(&dir).expect("create scratch dir");
    let v = gen_verilog(&dir, "local.t27", TEST_LOCAL);

    assert!(
        v.contains("input [127:0] xs;"),
        "a test-local array was bound as a module array: `total` lost its \
         `input [127:0] xs;` declaration\n{}",
        v
    );

    if !tool_available("iverilog") || !tool_available("vvp") {
        eprintln!("SKIP(sim): iverilog/vvp not on PATH");
        let _ = fs::remove_dir_all(&dir);
        return;
    }

    fs::write(dir.join("local.v"), v.as_bytes()).expect("write verilog");
    fs::write(dir.join("tb.v"), TESTBENCH).expect("write tb");
    let vvp = dir.join("sim.vvp");
    let compile = Command::new("iverilog")
        .args(["-g2012", "-DSIMULATION", "-o", vvp.to_str().unwrap()])
        .arg(dir.join("local.v"))
        .arg(dir.join("tb.v"))
        .output()
        .expect("invoke iverilog");
    assert!(
        compile.status.success(),
        "iverilog could not elaborate the by-value array parameter:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("vvp").arg(&vvp).output().expect("invoke vvp");
    let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
    let _ = fs::remove_dir_all(&dir);
    assert!(
        stdout.contains("PASS total=10"),
        "the by-value array parameter computed the wrong total:\n{}",
        stdout
    );
}

#[test]
fn module_level_array_argument_is_still_bound() {
    let dir = scratch_dir("modarr");
    fs::create_dir_all(&dir).expect("create scratch dir");
    let v = gen_verilog(&dir, "modarr.t27", MODULE_ARRAY);
    let _ = fs::remove_dir_all(&dir);

    assert!(
        !v.contains("input [127:0] xs;"),
        "a module-level array is no longer bound by name (W458): `total` \
         declares `xs` as an input\n{}",
        v
    );
    assert!(
        v.contains("TABLE[i]"),
        "the bound function does not read the module array `TABLE[i]`\n{}",
        v
    );
}

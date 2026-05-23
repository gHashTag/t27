//! Wave 36a -- R-BN-1 regression tests for the BitNet weight_bram emitter.
//!
//! Strategy: shell out to the built t27c binary via `CARGO_BIN_EXE_t27c`,
//! run `gen-weight-bram` with assorted geometries and module names, capture
//! stdout, and assert structural invariants on the emitted SystemVerilog
//! text. No HDL toolchain is required to run these tests; deeper port-list
//! / functional checks are covered by inline unit tests in
//! `bootstrap/src/weight_bram.rs`.
//!
//! Closes #760.

use std::process::Command;

fn run_gen_weight_bram(args: &[&str]) -> String {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-weight-bram")
        .args(args)
        .output()
        .expect("failed to spawn t27c gen-weight-bram");
    assert!(
        output.status.success(),
        "t27c gen-weight-bram exited with {:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("t27c gen-weight-bram produced non-UTF-8 output")
}

#[test]
fn emits_default_module_header() {
    let v = run_gen_weight_bram(&[]);
    assert!(v.contains("module weight_bram #("), "missing module header: {}", v);
    assert!(v.contains("endmodule"), "missing endmodule");
}

#[test]
fn default_geometry_matches_upstream() {
    let v = run_gen_weight_bram(&[]);
    assert!(v.contains("parameter DEPTH = 4096,"));
    assert!(v.contains("parameter ADDR_WIDTH = 12"));
    assert!(v.contains("[53:0]"));
}

#[test]
fn emits_dual_address_buses() {
    let v = run_gen_weight_bram(&[]);
    assert!(v.contains("input  wire [ADDR_WIDTH-1:0] rd_addr,"));
    assert!(v.contains("input  wire [ADDR_WIDTH-1:0] wr_addr,"));
}

#[test]
fn emits_synchronous_read_with_one_cycle_latency() {
    let v = run_gen_weight_bram(&[]);
    assert!(
        v.contains("always @(posedge clk) rd_data <= mem[rd_addr];"),
        "missing 1-cycle-latency read port"
    );
}

#[test]
fn emits_synchronous_write_guarded_by_wr_en() {
    let v = run_gen_weight_bram(&[]);
    assert!(
        v.contains("always @(posedge clk) if (wr_en) mem[wr_addr] <= wr_data;"),
        "missing wr_en-guarded write port"
    );
}

#[test]
fn emits_storage_array_declaration() {
    let v = run_gen_weight_bram(&[]);
    assert!(v.contains("reg [53:0] mem [0:DEPTH-1];"));
}

#[test]
fn custom_geometry_is_honored() {
    let v = run_gen_weight_bram(&[
        "--depth", "1024",
        "--addr-width", "10",
        "--data-width", "32",
        "--module-name", "custom_bram",
    ]);
    assert!(v.contains("module custom_bram #("));
    assert!(v.contains("parameter DEPTH = 1024,"));
    assert!(v.contains("parameter ADDR_WIDTH = 10"));
    assert!(v.contains("[31:0]"));
    assert!(v.contains("reg [31:0] mem [0:DEPTH-1];"));
}

#[test]
fn zero_depth_falls_back_to_default() {
    let v = run_gen_weight_bram(&["--depth", "0"]);
    assert!(v.contains("parameter DEPTH = 4096,"));
}

#[test]
fn zero_addr_width_falls_back_to_default() {
    let v = run_gen_weight_bram(&["--addr-width", "0"]);
    assert!(v.contains("parameter ADDR_WIDTH = 12"));
}

#[test]
fn zero_data_width_falls_back_to_default() {
    let v = run_gen_weight_bram(&["--data-width", "0"]);
    assert!(v.contains("[53:0]"));
}

#[test]
fn invalid_module_name_falls_back_to_default() {
    // Starts with a digit -- not a valid Verilog identifier.
    let v = run_gen_weight_bram(&["--module-name", "9bad"]);
    assert!(v.contains("module weight_bram #("));
}

#[test]
fn output_is_ascii_only() {
    let v = run_gen_weight_bram(&[]);
    assert!(v.is_ascii(), "weight_bram output must be ASCII (L3)");
}

#[test]
fn output_file_is_written_when_requested() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "t27_weight_bram_test_{}.sv",
        std::process::id()
    ));
    let path_str = path.to_string_lossy().to_string();
    let bin = env!("CARGO_BIN_EXE_t27c");
    let status = Command::new(bin)
        .arg("gen-weight-bram")
        .arg("--output")
        .arg(&path_str)
        .status()
        .expect("failed to spawn t27c gen-weight-bram (output)");
    assert!(status.success(), "expected success when writing output");
    let contents = std::fs::read_to_string(&path).expect("output file should exist");
    assert!(contents.contains("module weight_bram #("));
    assert!(contents.contains("endmodule"));
    let _ = std::fs::remove_file(&path);
}

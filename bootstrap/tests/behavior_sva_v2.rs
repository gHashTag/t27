//! Wave 37 -- R-BV-1 integration tests for the extended behavior-DSL SVA v2 emitter.
//!
//! Strategy: create a JSON file with behavior objects, shell out to the built
//! t27c binary via `CARGO_BIN_EXE_t27c`, run `gen-behavior-sva-v2`, capture
//! stdout or file output, and assert structural invariants on the emitted SVA.
//!
//! Closes #775.

use std::fs;
use std::process::Command;

use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn write_behaviors_json(behaviors: &[(&str, &str, &str, &str)]) -> String {
    let dir = std::env::temp_dir().join("t27c_test_behavior_sva_v2");
    let _ = fs::create_dir_all(&dir);
    let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let json = format!(
        "[{}]",
        behaviors
            .iter()
            .map(|(name, given, when, then)| {
                format!(
                    r#"{{"name":"{}","given":"{}","when":"{}","then":"{}"}}"#,
                    name, given, when, then
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    );
    let path = dir.join(format!("behaviors_{}_{}.json", std::process::id(), counter));
    fs::write(&path, &json).expect("write json");
    path.to_string_lossy().to_string()
}

fn run_gen_behavior_sva_v2(
    behaviors: &[(&str, &str, &str, &str)],
) -> String {
    let json_path = write_behaviors_json(behaviors);
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-behavior-sva-v2")
        .arg("--behaviors-json")
        .arg(&json_path)
        .output()
        .expect("failed to spawn t27c gen-behavior-sva-v2");
    assert!(
        output.status.success(),
        "t27c gen-behavior-sva-v2 exited with {:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("non-UTF-8 output")
}

fn run_gen_behavior_sva_v2_to_file(
    behaviors: &[(&str, &str, &str, &str)],
) -> (String, String) {
    let json_path = write_behaviors_json(behaviors);
    let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let out_path = std::env::temp_dir()
        .join("t27c_test_behavior_sva_v2")
        .join(format!("output_{}_{}.sv", std::process::id(), counter));
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-behavior-sva-v2")
        .arg("--behaviors-json")
        .arg(&json_path)
        .arg("--output")
        .arg(&out_path)
        .output()
        .expect("failed to spawn t27c gen-behavior-sva-v2");
    assert!(
        output.status.success(),
        "t27c gen-behavior-sva-v2 exited with {:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let content = fs::read_to_string(&out_path).expect("read output file");
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (content, stderr)
}

#[test]
fn v2_multi_clause_given_emits_conjunction() {
    let v = run_gen_behavior_sva_v2(&[
        ("multi_and", "valid and ready", "posedge clk", "done"),
    ]);
    assert!(v.contains("(valid_in && ready) |->"), "expected conjunction");
    assert!(v.contains("property p_multi_and;"));
}

#[test]
fn v2_comma_separator_emits_conjunction() {
    let v = run_gen_behavior_sva_v2(&[
        ("comma_test", "running, active", "rising edge", "done"),
    ]);
    assert!(v.contains("(running && active) |->"), "expected comma conjunction");
}

#[test]
fn v2_double_amp_separator() {
    let v = run_gen_behavior_sva_v2(&[
        ("amp_test", "valid && ready", "rising", "done"),
    ]);
    assert!(v.contains("(valid_in && ready) |->"), "expected && conjunction");
}

#[test]
fn v2_three_clause_conjunction() {
    let v = run_gen_behavior_sva_v2(&[
        ("three", "valid and ready and busy", "rising", "done"),
    ]);
    assert!(
        v.contains("(valid_in && ready && busy) |->"),
        "expected three-way conjunction"
    );
}

#[test]
fn v2_delay_after_cycles() {
    let v = run_gen_behavior_sva_v2(&[
        ("delayed", "start", "posedge clk", "after 3 cycles done"),
    ]);
    assert!(v.contains("start |-> ##3 done;"), "expected ##3 delay");
}

#[test]
fn v2_delay_hash_hash_syntax() {
    let v = run_gen_behavior_sva_v2(&[
        ("hash_delay", "start", "rising", "##5 valid_out"),
    ]);
    assert!(v.contains("start |-> ##5 valid_out;"), "expected ##5 delay");
}

#[test]
fn v2_eventually_emits_s_eventually() {
    let v = run_gen_behavior_sva_v2(&[
        ("live", "start", "rising", "eventually done"),
    ]);
    assert!(
        v.contains("start |-> s_eventually done;"),
        "expected s_eventually"
    );
}

#[test]
fn v2_liveness_keyword() {
    let v = run_gen_behavior_sva_v2(&[
        ("liveness", "start", "rising", "liveness check for done"),
    ]);
    assert!(
        v.contains("start |-> s_eventually done;"),
        "expected s_eventually from liveness keyword"
    );
}

#[test]
fn v2_plain_consequent_matches_v1() {
    let v = run_gen_behavior_sva_v2(&[
        ("plain", "running", "rising edge", "increment count"),
    ]);
    assert!(
        v.contains("running |-> (count == $past(count) + 1);"),
        "expected plain consequent matching v1 vocabulary"
    );
}

#[test]
fn v2_property_assert_cover_structure() {
    let v = run_gen_behavior_sva_v2(&[
        ("struct_test", "valid and ready", "posedge clk", "done"),
    ]);
    assert!(v.contains("property p_struct_test;"));
    assert!(v.contains("endproperty"));
    assert!(v.contains("assert_0_struct_test: assert property (p_struct_test)"));
    assert!(v.contains("$error(\"Assertion failed: struct_test\")"));
    assert!(v.contains("cover_0_struct_test: cover property (p_struct_test);"));
}

#[test]
fn v2_multi_behavior_indexing() {
    let v = run_gen_behavior_sva_v2(&[
        ("first", "running", "rising", "done"),
        ("second", "valid", "falling", "after 2 cycles busy"),
        ("third", "ready", "rising", "eventually done"),
    ]);
    assert!(v.contains("assert_0_first:"));
    assert!(v.contains("assert_1_second:"));
    assert!(v.contains("assert_2_third:"));
    assert!(v.contains("cover_0_first:"));
    assert!(v.contains("cover_1_second:"));
    assert!(v.contains("cover_2_third:"));
    assert!(v.contains("##2"));
    assert!(v.contains("s_eventually"));
}

#[test]
fn v2_header_footer() {
    let v = run_gen_behavior_sva_v2(&[
        ("h", "running", "rising", "done"),
    ]);
    assert!(v.contains("`timescale 1ns / 1ps"));
    assert!(v.contains("`default_nettype none"));
    assert!(v.contains("`default_nettype wire"));
    assert!(v.contains("gen-behavior-sva-v2"));
    assert!(v.contains("Wave 37"));
}

#[test]
fn v2_header_comments_quote_clauses() {
    let v = run_gen_behavior_sva_v2(&[
        ("doc_test", "valid and ready", "posedge clk", "after 3 cycles done"),
    ]);
    assert!(v.contains("// Behavior: doc_test"));
    assert!(v.contains("// Given:    valid and ready"));
    assert!(v.contains("// When:     posedge clk"));
    assert!(v.contains("// Then:     after 3 cycles done"));
}

#[test]
fn v2_falling_edge_timing() {
    let v = run_gen_behavior_sva_v2(&[
        ("fall", "running", "falling edge", "done"),
    ]);
    assert!(v.contains("@(negedge clk) disable iff (!rst_n)"));
}

#[test]
fn v2_disable_iff_rst_n() {
    let v = run_gen_behavior_sva_v2(&[
        ("rst", "running", "rising", "done"),
    ]);
    assert!(v.contains("disable iff (!rst_n)"));
}

#[test]
fn v2_file_output() {
    let (content, stderr) = run_gen_behavior_sva_v2_to_file(&[
        ("file_out", "running", "rising", "done"),
    ]);
    assert!(content.contains("property p_file_out;"));
    assert!(stderr.contains("behavior SVA v2 written to"));
    assert!(stderr.contains("bytes"));
}

#[test]
fn v2_passthrough_unknown_signal() {
    let v = run_gen_behavior_sva_v2(&[
        ("custom", "irq_status[2]", "rising", "custom_done"),
    ]);
    assert!(v.contains("irq_status[2] |->"), "unknown given signal should passthrough");
}

#[test]
fn v2_given_reset_not() {
    let v = run_gen_behavior_sva_v2(&[
        ("rst_test", "not reset", "rising", "done"),
    ]);
    assert!(v.contains("rst_n |-> done;"));
}

#[test]
fn v2_given_fifo_not_empty() {
    let v = run_gen_behavior_sva_v2(&[
        ("fifo_test", "fifo not empty", "rising", "done"),
    ]);
    assert!(v.contains("!empty |-> done;"));
}

#[test]
fn v2_delay_with_keyword_consequent() {
    let v = run_gen_behavior_sva_v2(&[
        ("delay_inc", "start", "rising", "after 2 cycles increment count"),
    ]);
    assert!(
        v.contains("start |-> ##2 (count == $past(count) + 1);"),
        "expected delayed increment consequent"
    );
}

#[test]
fn v2_mixed_conjunction_and_delay() {
    let v = run_gen_behavior_sva_v2(&[
        ("combo", "valid and start", "rising", "after 5 cycles done"),
    ]);
    assert!(v.contains("(valid_in && start) |-> ##5 done;"));
}

#[test]
fn v2_determinism() {
    let behaviors: (&str, &str, &str, &str) = ("det", "valid and ready", "rising", "after 3 cycles done");
    let v1 = run_gen_behavior_sva_v2(&[behaviors]);
    let v2 = run_gen_behavior_sva_v2(&[behaviors]);
    assert_eq!(v1, v2, "emitter must be deterministic");
}

#[test]
fn v2_empty_given_defaults() {
    let v = run_gen_behavior_sva_v2(&[
        ("empty_g", "", "rising", "done"),
    ]);
    assert!(v.contains("1'b1 |-> done;"), "empty given defaults to 1'b1");
}

#[test]
fn v2_ascii_only_output() {
    let v = run_gen_behavior_sva_v2(&[
        ("ascii", "valid and ready", "rising", "done"),
    ]);
    assert!(v.is_ascii(), "emitted SVA must be ASCII-only (L3)");
}

fn write_minimal_t27_spec() -> String {
    let dir = std::env::temp_dir().join("t27c_test_behavior_sva_v2");
    let _ = fs::create_dir_all(&dir);
    let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = dir.join(format!("spec_{}_{}.t27", std::process::id(), counter));
    // Legal t27 only: the hardened parser (t27#1940) rejects Rust-style
    // tail expressions and bare asserts instead of silently dropping them.
    let spec = r#"module test_module;

pub fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

test test_add {
    x = add(1, 2);
    assert(x == 3, "add");
}
"#;
    fs::write(&path, spec).expect("write spec");
    path.to_string_lossy().to_string()
}

#[test]
fn gen_verilog_with_sva_appends_bind_block() {
    let spec_path = write_minimal_t27_spec();
    let json_path = write_behaviors_json(&[
        ("check_done", "running", "posedge clk", "after 2 cycles done"),
    ]);
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-verilog")
        .arg(&spec_path)
        .arg("--with-sva")
        .arg("--sva-behaviors").arg(&json_path)
        .output()
        .expect("failed to spawn t27c gen-verilog --with-sva");
    assert!(
        output.status.success(),
        "t27c gen-verilog --with-sva exited with {:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("non-UTF-8");
    assert!(stdout.contains("endmodule"), "must contain endmodule");
    assert!(stdout.contains("module test_module_sva"), "must contain SVA companion module");
    assert!(stdout.contains("bind test_module test_module_sva sva_inst"), "must contain bind statement");
    assert!(stdout.contains("property p_check_done"), "must contain SVA property");
    assert!(stdout.contains("##2 done"), "must contain ##2 delay");
}

#[test]
fn gen_verilog_without_sva_no_append() {
    let spec_path = write_minimal_t27_spec();
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-verilog")
        .arg(&spec_path)
        .output()
        .expect("failed to spawn t27c gen-verilog");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("non-UTF-8");
    assert!(stdout.contains("endmodule"));
    assert!(!stdout.contains("_sva"), "should not contain SVA block without --with-sva");
    assert!(!stdout.contains("bind"), "should not contain bind without --with-sva");
}

#[test]
fn gen_verilog_with_sva_no_behaviors_is_noop() {
    let spec_path = write_minimal_t27_spec();
    let json_path = write_behaviors_json(&[]);
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-verilog")
        .arg(&spec_path)
        .arg("--with-sva")
        .arg("--sva-behaviors").arg(&json_path)
        .output()
        .expect("failed to spawn t27c gen-verilog --with-sva");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("non-UTF-8");
    assert!(stdout.contains("endmodule"));
    assert!(!stdout.contains("_sva"), "empty behaviors should not emit SVA block");
}

#[test]
fn gen_verilog_with_sva_multi_behavior() {
    let spec_path = write_minimal_t27_spec();
    let json_path = write_behaviors_json(&[
        ("a", "running", "rising", "done"),
        ("b", "valid and ready", "posedge clk", "after 5 cycles busy"),
        ("c", "start", "rising", "eventually done"),
    ]);
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-verilog")
        .arg(&spec_path)
        .arg("--with-sva")
        .arg("--sva-behaviors").arg(&json_path)
        .output()
        .expect("failed to spawn t27c gen-verilog --with-sva");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("non-UTF-8");
    assert!(stdout.contains("property p_a"));
    assert!(stdout.contains("property p_b"));
    assert!(stdout.contains("property p_c"));
    assert!(stdout.contains("##5 busy"));
    assert!(stdout.contains("s_eventually done"));
    assert!(stdout.contains("assert_0_a"));
    assert!(stdout.contains("assert_1_b"));
    assert!(stdout.contains("assert_2_c"));
}

#[test]
fn gen_verilog_with_sva_eventually() {
    let spec_path = write_minimal_t27_spec();
    let json_path = write_behaviors_json(&[
        ("liveness", "start", "rising", "eventually done"),
    ]);
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-verilog")
        .arg(&spec_path)
        .arg("--with-sva")
        .arg("--sva-behaviors").arg(&json_path)
        .output()
        .expect("failed to spawn t27c gen-verilog --with-sva");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("non-UTF-8");
    assert!(stdout.contains("s_eventually done"));
}

#[test]
fn gen_verilog_with_sva_conjunction() {
    let spec_path = write_minimal_t27_spec();
    let json_path = write_behaviors_json(&[
        ("combo", "valid and ready and busy", "rising", "done"),
    ]);
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-verilog")
        .arg(&spec_path)
        .arg("--with-sva")
        .arg("--sva-behaviors").arg(&json_path)
        .output()
        .expect("failed to spawn t27c gen-verilog --with-sva");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("non-UTF-8");
    assert!(stdout.contains("(valid_in && ready && busy) |-> done"));
}

#[test]
fn gen_verilog_with_sva_ascii_only() {
    let spec_path = write_minimal_t27_spec();
    let json_path = write_behaviors_json(&[
        ("ascii_check", "running", "rising", "done"),
    ]);
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-verilog")
        .arg(&spec_path)
        .arg("--with-sva")
        .arg("--sva-behaviors").arg(&json_path)
        .output()
        .expect("failed to spawn t27c gen-verilog --with-sva");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("non-UTF-8");
    assert!(stdout.is_ascii(), "full output must be ASCII-only (L3)");
}

#[test]
fn gen_verilog_hir_with_sva_appends_bind_block() {
    let spec_path = write_minimal_t27_spec();
    let json_path = write_behaviors_json(&[
        ("check", "running", "posedge clk", "done"),
    ]);
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg("gen-verilog-hir")
        .arg(&spec_path)
        .arg("--with-sva")
        .arg("--sva-behaviors").arg(&json_path)
        .output()
        .expect("failed to spawn t27c gen-verilog-hir --with-sva");
    assert!(
        output.status.success(),
        "exited with {:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("non-UTF-8");
    assert!(stdout.contains("endmodule"));
    assert!(stdout.contains("module test_module_sva"));
    assert!(stdout.contains("bind test_module test_module_sva sva_inst"));
    assert!(stdout.contains("property p_check"));
}

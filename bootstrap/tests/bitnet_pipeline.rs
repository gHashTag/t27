//! Wave 36b -- R-BN-2 regression tests for the BitNet pipeline_stage2_compute
//! and layer_sequencer emitters.
//!
//! Strategy: shell out to the built t27c binary via `CARGO_BIN_EXE_t27c`,
//! invoke `gen-pipeline-stage2` and `gen-layer-sequencer`, capture stdout,
//! and assert structural invariants on the emitted SystemVerilog text. No
//! HDL toolchain is required; deeper port-list / behavioral checks are
//! covered by inline unit tests in `bootstrap/src/bitnet_pipeline.rs`.
//!
//! Closes #762.

use std::process::Command;

fn run_subcommand(subcmd: &str, args: &[&str]) -> String {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let output = Command::new(bin)
        .arg(subcmd)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn t27c {}: {}", subcmd, e));
    assert!(
        output.status.success(),
        "t27c {} exited with {:?}, stderr={}",
        subcmd,
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap_or_else(|_| panic!("t27c {} produced non-UTF-8 output", subcmd))
}

// ---- pipeline_stage2_compute --------------------------------------------

#[test]
fn stage2_emits_default_module_header() {
    let v = run_subcommand("gen-pipeline-stage2", &[]);
    assert!(v.contains("module pipeline_stage2_compute ("));
    assert!(v.contains("endmodule"));
}

#[test]
fn stage2_custom_module_name_is_honored() {
    let v = run_subcommand("gen-pipeline-stage2", &["--module-name", "my_stage"]);
    assert!(v.contains("module my_stage ("));
}

#[test]
fn stage2_invalid_module_name_falls_back_to_default() {
    let v = run_subcommand("gen-pipeline-stage2", &["--module-name", "9bad"]);
    assert!(v.contains("module pipeline_stage2_compute ("));
}

#[test]
fn stage2_instantiates_trit27_dot_product() {
    let v = run_subcommand("gen-pipeline-stage2", &[]);
    assert!(v.contains("trit27_dot_product simd"));
    assert!(v.contains(".input_vec(input_chunk)"));
    assert!(v.contains(".weight_vec(weight_chunk)"));
    assert!(v.contains(".result(dot_result)"));
}

#[test]
fn stage2_has_54bit_chunk_ports() {
    let v = run_subcommand("gen-pipeline-stage2", &[]);
    assert!(v.contains("input  wire [53:0] input_chunk,"));
    assert!(v.contains("input  wire [53:0] weight_chunk,"));
}

#[test]
fn stage2_has_signed_16bit_accumulator() {
    let v = run_subcommand("gen-pipeline-stage2", &[]);
    assert!(v.contains("reg signed [15:0] accumulator;"));
    assert!(v.contains("output reg  signed [15:0] result,"));
}

#[test]
fn stage2_resets_on_negedge_rst_n() {
    let v = run_subcommand("gen-pipeline-stage2", &[]);
    assert!(v.contains("always @(posedge clk or negedge rst_n) begin"));
    assert!(v.contains("accumulator <= 0; valid_out <= 0; result_final <= 0;"));
}

#[test]
fn stage2_accumulator_gated_by_first_chunk() {
    let v = run_subcommand("gen-pipeline-stage2", &[]);
    assert!(v.contains("accumulator <= first_chunk ? dot_result : accumulator + dot_result;"));
}

#[test]
fn stage2_valid_out_strobed_on_last_chunk() {
    let v = run_subcommand("gen-pipeline-stage2", &[]);
    assert!(v.contains("valid_out <= last_chunk;"));
    assert!(v.contains("result_final <= last_chunk;"));
}

// ---- layer_sequencer -----------------------------------------------------

#[test]
fn sequencer_emits_default_module_header() {
    let v = run_subcommand("gen-layer-sequencer", &[]);
    assert!(v.contains("module layer_sequencer ("));
    assert!(v.contains("endmodule"));
}

#[test]
fn sequencer_custom_module_name_is_honored() {
    let v = run_subcommand("gen-layer-sequencer", &["--module-name", "my_seq"]);
    assert!(v.contains("module my_seq ("));
}

#[test]
fn sequencer_invalid_module_name_falls_back_to_default() {
    let v = run_subcommand("gen-layer-sequencer", &["--module-name", "9bad"]);
    assert!(v.contains("module layer_sequencer ("));
}

#[test]
fn sequencer_declares_three_state_fsm() {
    let v = run_subcommand("gen-layer-sequencer", &[]);
    assert!(v.contains("localparam IDLE=0, RUN=1, DONE_ST=2;"));
    assert!(v.contains("reg [1:0] state;"));
}

#[test]
fn sequencer_port_list_has_neuron_chunk_counters() {
    let v = run_subcommand("gen-layer-sequencer", &[]);
    assert!(v.contains("input  wire [15:0] num_neurons,"));
    assert!(v.contains("input  wire [7:0]  num_chunks,"));
    assert!(v.contains("output reg  [15:0] neuron_id,"));
    assert!(v.contains("output reg  [7:0]  chunk_id,"));
}

#[test]
fn sequencer_emits_first_last_chunk_strobes() {
    let v = run_subcommand("gen-layer-sequencer", &[]);
    assert!(v.contains("first_chunk<=(chunk_id==0);"));
    assert!(v.contains("last_chunk<=(chunk_id==num_chunks-1);"));
}

#[test]
fn sequencer_idle_arms_on_start() {
    let v = run_subcommand("gen-layer-sequencer", &[]);
    // IDLE deasserts `done` on entry (wave-36b) and arms the FSM on `start`.
    assert!(v.contains(
        "IDLE: begin done<=0; if(start) begin state<=RUN; neuron_id<=0; chunk_id<=0; end end"
    ));
}

#[test]
fn sequencer_done_st_returns_to_idle() {
    let v = run_subcommand("gen-layer-sequencer", &[]);
    assert!(v.contains("DONE_ST: begin valid<=0; done<=1; state<=IDLE; end"));
}

// ---- shared invariants ---------------------------------------------------

#[test]
fn both_outputs_are_ascii_only() {
    let s2 = run_subcommand("gen-pipeline-stage2", &[]);
    let lq = run_subcommand("gen-layer-sequencer", &[]);
    assert!(s2.is_ascii(), "pipeline_stage2 stdout must be ASCII (L3)");
    assert!(lq.is_ascii(), "layer_sequencer stdout must be ASCII (L3)");
}

#[test]
fn stage2_output_file_is_written_when_requested() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "t27_pipeline_stage2_test_{}.sv",
        std::process::id()
    ));
    let bin = env!("CARGO_BIN_EXE_t27c");
    let status = Command::new(bin)
        .arg("gen-pipeline-stage2")
        .arg("--output")
        .arg(&path)
        .status()
        .expect("failed to spawn t27c gen-pipeline-stage2 (output)");
    assert!(status.success());
    let contents = std::fs::read_to_string(&path).expect("output file should exist");
    assert!(contents.contains("module pipeline_stage2_compute ("));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn sequencer_output_file_is_written_when_requested() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "t27_layer_sequencer_test_{}.sv",
        std::process::id()
    ));
    let bin = env!("CARGO_BIN_EXE_t27c");
    let status = Command::new(bin)
        .arg("gen-layer-sequencer")
        .arg("--output")
        .arg(&path)
        .status()
        .expect("failed to spawn t27c gen-layer-sequencer (output)");
    assert!(status.success());
    let contents = std::fs::read_to_string(&path).expect("output file should exist");
    assert!(contents.contains("module layer_sequencer ("));
    let _ = std::fs::remove_file(&path);
}

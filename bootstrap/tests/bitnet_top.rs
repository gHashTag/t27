//! Integration tests for `t27c gen-bitnet-engine-top` (Wave 36f, R-BN-6).

use std::process::Command;

fn t27c_bin() -> String {
    std::env::var("CARGO_BIN_EXE_t27c").expect("CARGO_BIN_EXE_t27c not set")
}

fn run(args: &[&str]) -> (String, String, bool) {
    let out = Command::new(t27c_bin())
        .args(args)
        .output()
        .expect("failed to execute t27c");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.success(),
    )
}

// ============================================================================
// Module name handling
// ============================================================================

#[test]
fn top_default_emits_module() {
    let (stdout, stderr, ok) = run(&["gen-bitnet-engine-top"]);
    assert!(ok, "stderr: {}", stderr);
    assert!(stdout.contains("module bitnet_engine_top ("));
    assert!(stdout.contains("endmodule"));
}

#[test]
fn top_custom_module_name() {
    let (stdout, _stderr, ok) =
        run(&["gen-bitnet-engine-top", "--module-name", "engine_top_v1"]);
    assert!(ok);
    assert!(stdout.contains("module engine_top_v1 ("));
    assert!(!stdout.contains("module bitnet_engine_top ("));
}

#[test]
fn top_invalid_module_name_falls_back() {
    for bad in &["9bad", "has space", "dash-name", ""] {
        let (stdout, _stderr, ok) =
            run(&["gen-bitnet-engine-top", "--module-name", bad]);
        assert!(ok, "command failed for invalid name `{}`", bad);
        assert!(
            stdout.contains("module bitnet_engine_top ("),
            "expected fallback for `{}`",
            bad
        );
    }
}

// ============================================================================
// Port surfaces
// ============================================================================

#[test]
fn top_control_ports_present() {
    let (stdout, _stderr, ok) = run(&["gen-bitnet-engine-top"]);
    assert!(ok);
    for port in [
        "input  wire        start,",
        "input  wire [5:0]  num_layers,",
        "input  wire [15:0] neurons_per_layer,",
        "input  wire [7:0]  chunks_per_neuron,",
        "input  wire signed [15:0] threshold,",
    ] {
        assert!(stdout.contains(port), "missing control port `{}`", port);
    }
}

#[test]
fn top_status_ports_present() {
    let (stdout, _stderr, ok) = run(&["gen-bitnet-engine-top"]);
    assert!(ok);
    assert!(stdout.contains("output wire        busy,"));
    assert!(stdout.contains("output wire        done,"));
    assert!(stdout.contains("output wire [31:0] cycle_count"));
}

#[test]
fn top_external_memory_ports_present() {
    let (stdout, _stderr, ok) = run(&["gen-bitnet-engine-top"]);
    assert!(ok);
    assert!(stdout.contains("output wire [31:0] mem_addr,"));
    assert!(stdout.contains("output wire        mem_rd_en,"));
    assert!(stdout.contains("input  wire [63:0] mem_rd_data,"));
    assert!(stdout.contains("input  wire        mem_rd_valid,"));
}

// ============================================================================
// Sub-module instantiations
// ============================================================================

#[test]
fn top_instantiates_multilayer_sequencer() {
    let (stdout, _stderr, ok) = run(&["gen-bitnet-engine-top"]);
    assert!(ok);
    assert!(stdout.contains("multilayer_sequencer seq ("));
    assert!(stdout.contains(".clk(clk), .rst_n(rst_n), .start(start), .num_layers(num_layers),"));
    assert!(stdout.contains(".inference_done(done)"));
}

#[test]
fn top_instantiates_double_buffer_ctrl() {
    let (stdout, _stderr, ok) = run(&["gen-bitnet-engine-top"]);
    assert!(ok);
    assert!(stdout.contains("double_buffer_ctrl dbl_buf ("));
    assert!(stdout.contains(".use_buffer_a(use_buffer_a)"));
    assert!(stdout.contains(".neuron_id(neuron_id[11:0])"));
}

#[test]
fn top_wires_for_sequencer_and_buffer() {
    let (stdout, _stderr, ok) = run(&["gen-bitnet-engine-top"]);
    assert!(ok);
    assert!(stdout.contains("wire [5:0] current_layer;"));
    // prefetch_done is now driven by weight_prefetch_ctrl, so it is declared
    // separately from the two control-plane wires.
    assert!(stdout.contains("wire layer_start, start_prefetch;"));
    assert!(stdout.contains("wire prefetch_done;"));
    assert!(stdout.contains("wire use_buffer_a;"));
    assert!(stdout.contains("wire [11:0] buf_read_addr, buf_write_addr;"));
}

// ============================================================================
// Cycle counter & busy
// ============================================================================

#[test]
fn top_cycle_counter_resets_on_start_and_increments_when_busy() {
    let (stdout, _stderr, ok) = run(&["gen-bitnet-engine-top"]);
    assert!(ok);
    assert!(stdout.contains("reg [31:0] cycles;"));
    assert!(stdout.contains("if (!rst_n) cycles <= 32'd0;"));
    assert!(stdout.contains("else if (start) cycles <= 32'd0;"));
    assert!(stdout.contains("else if (busy) cycles <= cycles + 32'd1;"));
    assert!(stdout.contains("assign cycle_count = cycles;"));
}

#[test]
fn top_busy_from_current_layer_or_layer_start() {
    let (stdout, _stderr, ok) = run(&["gen-bitnet-engine-top"]);
    assert!(ok);
    assert!(stdout.contains("assign busy = (current_layer != 6'd0) || layer_start;"));
}

#[test]
fn top_mem_outputs_driven_by_prefetch() {
    let (stdout, _stderr, ok) = run(&["gen-bitnet-engine-top"]);
    assert!(ok);
    // Was asserting the tie-off as the contract. The external memory port is
    // now driven by the weight prefetch controller.
    assert!(stdout.contains("assign mem_addr  = pf_araddr;"));
    assert!(stdout.contains("assign mem_rd_en = pf_arvalid;"));
}

#[test]
fn top_negedge_reset_for_counter() {
    let (stdout, _stderr, ok) = run(&["gen-bitnet-engine-top"]);
    assert!(ok);
    assert!(stdout.contains("always @(posedge clk or negedge rst_n)"));
}

// ============================================================================
// File output & determinism
// ============================================================================

#[test]
fn top_output_to_file() {
    let path = std::env::temp_dir()
        .join(format!("t27_top_out_{}.sv", std::process::id()));
    let path_s = path.to_string_lossy().to_string();
    let (_stdout, stderr, ok) = run(&[
        "gen-bitnet-engine-top",
        "--module-name",
        "top_x",
        "--output",
        &path_s,
    ]);
    assert!(ok, "stderr: {}", stderr);
    let body = std::fs::read_to_string(&path).expect("output file missing");
    assert!(body.contains("module top_x ("));
    assert!(body.trim_end().ends_with("endmodule"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn top_output_is_deterministic() {
    let (a, _e1, ok1) = run(&["gen-bitnet-engine-top", "--module-name", "t1"]);
    let (b, _e2, ok2) = run(&["gen-bitnet-engine-top", "--module-name", "t1"]);
    assert!(ok1 && ok2);
    assert_eq!(a, b, "same args must yield byte-identical Verilog");
}

#[test]
fn top_output_is_pure_ascii() {
    let (stdout, _stderr, ok) = run(&["gen-bitnet-engine-top"]);
    assert!(ok);
    assert!(stdout.is_ascii(), "emitted Verilog must be ASCII");
}

#[test]
fn top_help_lists_subcommand() {
    let (stdout, _stderr, ok) = run(&["gen-bitnet-engine-top", "--help"]);
    assert!(ok);
    assert!(stdout.contains("--module-name"));
    assert!(stdout.contains("--output"));
}

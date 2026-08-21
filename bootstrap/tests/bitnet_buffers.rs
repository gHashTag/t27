//! Integration tests for `t27c gen-double-buffer-ctrl` and
//! `t27c gen-weight-prefetch-ctrl` (Wave 36c, R-BN-3).

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
// gen-double-buffer-ctrl
// ============================================================================

#[test]
fn dbuf_default_emits_module() {
    let (stdout, stderr, ok) = run(&["gen-double-buffer-ctrl"]);
    assert!(ok, "stderr: {}", stderr);
    assert!(stdout.contains("module double_buffer_ctrl ("));
    assert!(stdout.contains("endmodule"));
}

#[test]
fn dbuf_custom_module_name() {
    let (stdout, _stderr, ok) = run(&["gen-double-buffer-ctrl", "--module-name", "my_dbuf"]);
    assert!(ok);
    assert!(stdout.contains("module my_dbuf ("));
}

#[test]
fn dbuf_invalid_name_falls_back() {
    let (stdout, _stderr, ok) =
        run(&["gen-double-buffer-ctrl", "--module-name", "9 bad name!"]);
    assert!(ok);
    assert!(stdout.contains("module double_buffer_ctrl ("));
}

#[test]
fn dbuf_has_all_ports() {
    let (stdout, _stderr, ok) = run(&["gen-double-buffer-ctrl"]);
    assert!(ok);
    for port in [
        "input  wire        clk",
        "input  wire        rst_n",
        "input  wire        layer_done",
        "input  wire [5:0]  current_layer",
        "output reg         use_buffer_a",
        "output wire [11:0] read_addr",
        "output wire [11:0] write_addr",
        "input  wire [11:0] neuron_id",
    ] {
        assert!(stdout.contains(port), "missing port `{}`", port);
    }
}

#[test]
fn dbuf_toggles_on_layer_done() {
    let (stdout, _stderr, ok) = run(&["gen-double-buffer-ctrl"]);
    assert!(ok);
    assert!(stdout.contains("if (!rst_n) use_buffer_a <= 1'b1;"));
    assert!(stdout.contains("else if (layer_done) use_buffer_a <= ~use_buffer_a;"));
}

#[test]
fn dbuf_addr_assigns() {
    let (stdout, _stderr, ok) = run(&["gen-double-buffer-ctrl"]);
    assert!(ok);
    assert!(stdout.contains("assign read_addr  = neuron_id;"));
    assert!(stdout.contains("assign write_addr = neuron_id;"));
}

#[test]
fn dbuf_output_to_file() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "t27_double_buffer_ctrl_test_{}.sv",
        std::process::id()
    ));
    let (_stdout, stderr, ok) = run(&[
        "gen-double-buffer-ctrl",
        "--output",
        path.to_str().unwrap(),
    ]);
    assert!(ok, "stderr: {}", stderr);
    let content = std::fs::read_to_string(&path).expect("read emitted file");
    assert!(content.contains("module double_buffer_ctrl ("));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn dbuf_emitted_text_is_ascii() {
    let (stdout, _stderr, ok) = run(&["gen-double-buffer-ctrl"]);
    assert!(ok);
    assert!(stdout.is_ascii(), "stdout must be ASCII");
}

// ============================================================================
// gen-weight-prefetch-ctrl
// ============================================================================

#[test]
fn prefetch_default_emits_module() {
    let (stdout, stderr, ok) = run(&["gen-weight-prefetch-ctrl"]);
    assert!(ok, "stderr: {}", stderr);
    assert!(stdout.contains("module weight_prefetch_ctrl ("));
    assert!(stdout.contains("endmodule"));
}

#[test]
fn prefetch_custom_module_name() {
    let (stdout, _stderr, ok) =
        run(&["gen-weight-prefetch-ctrl", "--module-name", "my_pfetch"]);
    assert!(ok);
    assert!(stdout.contains("module my_pfetch ("));
}

#[test]
fn prefetch_invalid_name_falls_back() {
    let (stdout, _stderr, ok) =
        run(&["gen-weight-prefetch-ctrl", "--module-name", "bad name!"]);
    assert!(ok);
    assert!(stdout.contains("module weight_prefetch_ctrl ("));
}

#[test]
fn prefetch_has_all_axi_ports() {
    let (stdout, _stderr, ok) = run(&["gen-weight-prefetch-ctrl"]);
    assert!(ok);
    for port in [
        "output reg  [31:0] axi_araddr",
        "output reg         axi_arvalid",
        "input  wire        axi_arready",
        "input  wire [63:0] axi_rdata",
        "input  wire        axi_rvalid",
        "output wire        axi_rready",
    ] {
        assert!(stdout.contains(port), "missing axi port `{}`", port);
    }
}

#[test]
fn prefetch_has_all_bram_ports() {
    let (stdout, _stderr, ok) = run(&["gen-weight-prefetch-ctrl"]);
    assert!(ok);
    for port in [
        "output reg  [11:0] bram_addr",
        "output reg  [53:0] bram_data",
        "output reg         bram_we",
    ] {
        assert!(stdout.contains(port), "missing bram port `{}`", port);
    }
}

#[test]
fn prefetch_has_control_ports() {
    let (stdout, _stderr, ok) = run(&["gen-weight-prefetch-ctrl"]);
    assert!(ok);
    for port in [
        "input  wire        start_prefetch",
        "input  wire [31:0] src_addr",
        "input  wire [15:0] num_words",
        "output reg         prefetch_active",
        "output reg         prefetch_done",
    ] {
        assert!(stdout.contains(port), "missing ctrl port `{}`", port);
    }
}

#[test]
fn prefetch_fsm_states_present() {
    let (stdout, _stderr, ok) = run(&["gen-weight-prefetch-ctrl"]);
    assert!(ok);
    assert!(stdout.contains("localparam IDLE = 2'd0, FETCH = 2'd1, DONE_ST = 2'd2;"));
    assert!(stdout.contains("IDLE: begin"));
    assert!(stdout.contains("if (start_prefetch) begin"));
    assert!(stdout.contains("FETCH: begin"));
    assert!(stdout.contains("DONE_ST: begin"));
}

/// Issue #1985: the emitted `IDLE` arm must retire `prefetch_done` before it
/// tests `start_prefetch`, so a requester sampling the flag in the cycle it
/// raises `start_prefetch` does not see the previous transaction's
/// completion. Anchored to the `IDLE` arm because the reset block also
/// contains `prefetch_done <= 1'b0;`.
#[test]
fn prefetch_done_retired_in_idle_before_start_guard() {
    let (stdout, _stderr, ok) = run(&["gen-weight-prefetch-ctrl"]);
    assert!(ok);
    let case_body = stdout
        .split_once("end else case (state)")
        .expect("FSM case statement missing")
        .1;
    let idle_arm = case_body
        .split_once("FETCH: begin")
        .expect("FETCH arm missing")
        .0;
    let clear = idle_arm
        .find("prefetch_done <= 1'b0;")
        .unwrap_or_else(|| panic!("IDLE arm never clears prefetch_done:\n{}", idle_arm));
    let guard = idle_arm
        .find("if (start_prefetch)")
        .unwrap_or_else(|| panic!("IDLE arm missing start_prefetch guard:\n{}", idle_arm));
    assert!(
        clear < guard,
        "prefetch_done must be cleared on entry to IDLE, before the \
         `if (start_prefetch)` guard (#1985). IDLE arm:\n{}",
        idle_arm
    );
}

#[test]
fn prefetch_rready_combinational() {
    let (stdout, _stderr, ok) = run(&["gen-weight-prefetch-ctrl"]);
    assert!(ok);
    assert!(stdout.contains("assign axi_rready = (state == FETCH);"));
}

#[test]
fn prefetch_advances_araddr_by_8() {
    let (stdout, _stderr, ok) = run(&["gen-weight-prefetch-ctrl"]);
    assert!(ok);
    assert!(stdout.contains("if (axi_arready) axi_araddr <= axi_araddr + 32'd8;"));
}

#[test]
fn prefetch_truncates_to_54_bits() {
    let (stdout, _stderr, ok) = run(&["gen-weight-prefetch-ctrl"]);
    assert!(ok);
    assert!(stdout.contains("bram_data <= axi_rdata[53:0];"));
}

#[test]
fn prefetch_output_to_file() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "t27_weight_prefetch_ctrl_test_{}.sv",
        std::process::id()
    ));
    let (_stdout, stderr, ok) = run(&[
        "gen-weight-prefetch-ctrl",
        "--output",
        path.to_str().unwrap(),
    ]);
    assert!(ok, "stderr: {}", stderr);
    let content = std::fs::read_to_string(&path).expect("read emitted file");
    assert!(content.contains("module weight_prefetch_ctrl ("));
    assert!(content.contains("endmodule"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn prefetch_emitted_text_is_ascii() {
    let (stdout, _stderr, ok) = run(&["gen-weight-prefetch-ctrl"]);
    assert!(ok);
    assert!(stdout.is_ascii(), "stdout must be ASCII");
}

#[test]
fn prefetch_reset_block_initializes_all_regs() {
    let (stdout, _stderr, ok) = run(&["gen-weight-prefetch-ctrl"]);
    assert!(ok);
    for line in [
        "state <= IDLE;",
        "prefetch_active <= 1'b0;",
        "prefetch_done <= 1'b0;",
        "axi_arvalid <= 1'b0;",
        "bram_we <= 1'b0;",
    ] {
        assert!(stdout.contains(line), "missing reset line `{}`", line);
    }
}

// ============================================================================
// Cross-module: both subcommands listed in help
// ============================================================================

#[test]
fn help_lists_both_new_subcommands() {
    let (stdout, _stderr, ok) = run(&["--help"]);
    assert!(ok);
    assert!(stdout.contains("gen-double-buffer-ctrl"));
    assert!(stdout.contains("gen-weight-prefetch-ctrl"));
}

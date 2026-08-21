//! Integration tests for `t27c gen-axi-lite-slave` (Wave 36d, R-BN-4).

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
// Module name & parameters
// ============================================================================

#[test]
fn axi_default_emits_module() {
    let (stdout, stderr, ok) = run(&["gen-axi-lite-slave"]);
    assert!(ok, "stderr: {}", stderr);
    assert!(stdout.contains("module axi_lite_slave #("));
    assert!(stdout.contains("parameter ADDR_WIDTH = 8,"));
    assert!(stdout.contains("parameter DATA_WIDTH = 32"));
    assert!(stdout.contains("endmodule"));
}

#[test]
fn axi_custom_module_name() {
    let (stdout, _stderr, ok) =
        run(&["gen-axi-lite-slave", "--module-name", "csr_block"]);
    assert!(ok);
    assert!(stdout.contains("module csr_block #("));
}

#[test]
fn axi_invalid_module_name_falls_back() {
    let (stdout, _stderr, ok) =
        run(&["gen-axi-lite-slave", "--module-name", "9 bad name!"]);
    assert!(ok);
    assert!(stdout.contains("module axi_lite_slave #("));
}

#[test]
fn axi_custom_parameters() {
    let (stdout, _stderr, ok) = run(&[
        "gen-axi-lite-slave",
        "--addr-width", "12",
        "--data-width", "64",
    ]);
    assert!(ok);
    assert!(stdout.contains("parameter ADDR_WIDTH = 12,"));
    assert!(stdout.contains("parameter DATA_WIDTH = 64"));
}

#[test]
fn axi_param_clamp_addr_width_too_big() {
    let (stdout, _stderr, ok) = run(&[
        "gen-axi-lite-slave",
        "--addr-width", "99",
    ]);
    assert!(ok);
    assert!(stdout.contains("parameter ADDR_WIDTH = 8,"));
}

#[test]
fn axi_param_clamp_data_width_too_big() {
    let (stdout, _stderr, ok) = run(&[
        "gen-axi-lite-slave",
        "--data-width", "999",
    ]);
    assert!(ok);
    assert!(stdout.contains("parameter DATA_WIDTH = 32"));
}

// ============================================================================
// AXI port groups
// ============================================================================

#[test]
fn axi_has_all_write_channel_ports() {
    let (stdout, _stderr, ok) = run(&["gen-axi-lite-slave"]);
    assert!(ok);
    for port in [
        "input  wire [ADDR_WIDTH-1:0]   s_axi_awaddr",
        "input  wire                    s_axi_awvalid",
        "output reg                     s_axi_awready",
        "input  wire [DATA_WIDTH-1:0]   s_axi_wdata",
        "input  wire [3:0]              s_axi_wstrb",
        "input  wire                    s_axi_wvalid",
        "output reg                     s_axi_wready",
        "output reg  [1:0]              s_axi_bresp",
        "output reg                     s_axi_bvalid",
        "input  wire                    s_axi_bready",
    ] {
        assert!(stdout.contains(port), "missing write-channel port `{}`", port);
    }
}

#[test]
fn axi_has_all_read_channel_ports() {
    let (stdout, _stderr, ok) = run(&["gen-axi-lite-slave"]);
    assert!(ok);
    for port in [
        "input  wire [ADDR_WIDTH-1:0]   s_axi_araddr",
        "input  wire                    s_axi_arvalid",
        "output reg                     s_axi_arready",
        "output reg  [DATA_WIDTH-1:0]   s_axi_rdata",
        "output reg  [1:0]              s_axi_rresp",
        "output reg                     s_axi_rvalid",
        "input  wire                    s_axi_rready",
    ] {
        assert!(stdout.contains(port), "missing read-channel port `{}`", port);
    }
}

// ============================================================================
// CSR ports
// ============================================================================

#[test]
fn axi_has_all_csr_ports() {
    let (stdout, _stderr, ok) = run(&["gen-axi-lite-slave"]);
    assert!(ok);
    for port in [
        "output reg  [31:0]             reg_ctrl",
        "input  wire [31:0]             reg_status",
        "output reg  [31:0]             reg_irq_en",
        "input  wire [31:0]             reg_irq_stat",
        "output reg  [31:0]             reg_num_layers",
        "output reg  [31:0]             reg_neurons",
        "output reg  [31:0]             reg_chunks",
        "output reg  [31:0]             reg_threshold",
        "output reg  [63:0]             reg_weight_addr",
        "output reg  [63:0]             reg_input_addr",
        "output reg  [63:0]             reg_output_addr",
        "input  wire [63:0]             reg_cycles",
    ] {
        assert!(stdout.contains(port), "missing CSR port `{}`", port);
    }
}

// ============================================================================
// CSR map -- writes
// ============================================================================

#[test]
fn axi_write_case_covers_all_writable_offsets() {
    let (stdout, _stderr, ok) = run(&["gen-axi-lite-slave"]);
    assert!(ok);
    for off in [
        "4'h0: reg_ctrl",
        "4'h2: reg_irq_en",
        "4'h4: reg_num_layers",
        "4'h5: reg_neurons",
        "4'h6: reg_chunks",
        "4'h7: reg_threshold",
        "4'h8: reg_weight_addr[31:0]",
        "4'h9: reg_weight_addr[63:32]",
        "4'hA: reg_input_addr[31:0]",
        "4'hB: reg_input_addr[63:32]",
        "4'hC: reg_output_addr[31:0]",
        "4'hD: reg_output_addr[63:32]",
    ] {
        assert!(stdout.contains(off), "missing write case `{}`", off);
    }
}

// ============================================================================
// CSR map -- reads
// ============================================================================

#[test]
fn axi_read_case_covers_all_readable_offsets() {
    let (stdout, _stderr, ok) = run(&["gen-axi-lite-slave"]);
    assert!(ok);
    for off in [
        "4'h0: s_axi_rdata <= reg_ctrl",
        "4'h1: s_axi_rdata <= reg_status",
        "4'h2: s_axi_rdata <= reg_irq_en",
        "4'h3: s_axi_rdata <= reg_irq_stat",
        "4'h4: s_axi_rdata <= reg_num_layers",
        "4'h5: s_axi_rdata <= reg_neurons",
        "4'h6: s_axi_rdata <= reg_chunks",
        "4'h7: s_axi_rdata <= reg_threshold",
        "4'h8: s_axi_rdata <= reg_weight_addr[31:0]",
        "4'h9: s_axi_rdata <= reg_weight_addr[63:32]",
        "4'hA: s_axi_rdata <= reg_input_addr[31:0]",
        "4'hB: s_axi_rdata <= reg_input_addr[63:32]",
        "4'hC: s_axi_rdata <= reg_output_addr[31:0]",
        "4'hD: s_axi_rdata <= reg_output_addr[63:32]",
        "4'hE: s_axi_rdata <= reg_cycles[31:0]",
        "4'hF: s_axi_rdata <= reg_cycles[63:32]",
    ] {
        assert!(stdout.contains(off), "missing read case `{}`", off);
    }
}

#[test]
fn axi_unmapped_read_returns_deadbeef() {
    let (stdout, _stderr, ok) = run(&["gen-axi-lite-slave"]);
    assert!(ok);
    assert!(stdout.contains("default: s_axi_rdata <= 32'hDEADBEEF;"));
}

// ============================================================================
// AXI semantics
// ============================================================================

#[test]
fn axi_responses_are_okay() {
    let (stdout, _stderr, ok) = run(&["gen-axi-lite-slave"]);
    assert!(ok);
    assert!(stdout.contains("s_axi_bvalid <= 1'b1; s_axi_bresp <= 2'b00;"));
    assert!(stdout.contains("s_axi_rvalid <= 1'b1; s_axi_rresp <= 2'b00;"));
}

#[test]
fn axi_handshake_dropbacks_present() {
    let (stdout, _stderr, ok) = run(&["gen-axi-lite-slave"]);
    assert!(ok);
    assert!(stdout.contains("if (s_axi_bvalid && s_axi_bready) begin"));
    assert!(stdout.contains("if (s_axi_rvalid && s_axi_rready) begin"));
    assert!(stdout.contains("s_axi_bvalid <= 1'b0;"));
    assert!(stdout.contains("s_axi_rvalid <= 1'b0;"));
}

/// Regression guard for the lost-response defect (issue 1968), asserted on the
/// shipped CLI output rather than on the library function.
///
/// `awready`, `wready` and `arready` were asserted at reset and never
/// deasserted. With one response register per channel, accepting a second
/// transaction while the first is unanswered merges two transactions into one
/// response beat and the master hangs. Asserting that a deassertion exists at
/// all -- not the text of the current expression -- makes any re-pinning of a
/// ready fail here regardless of how it is spelled.
#[test]
fn axi_ready_drops_while_a_response_is_owed() {
    let (stdout, _stderr, ok) = run(&["gen-axi-lite-slave"]);
    assert!(ok);
    for sig in ["s_axi_awready", "s_axi_wready", "s_axi_arready"] {
        assert!(
            stdout.contains(&format!("{} <= 1'b0", sig)),
            "`{}` is never deasserted: it is asserted at reset and left high, \
             so a second transaction is accepted while the first response is \
             still unacknowledged and one of the two responses is lost. \
             Emitted:\n{}",
            sig,
            stdout
        );
    }
}

#[test]
fn axi_reset_initializes_all_outputs() {
    let (stdout, _stderr, ok) = run(&["gen-axi-lite-slave"]);
    assert!(ok);
    // Scoped to the reset branch on purpose. Since #1968 the ready signals are
    // also assigned outside reset, so a whole-module `contains` would be
    // satisfied by the release block and would no longer notice a reset line
    // going missing.
    let start = stdout.find("if (!rst_n) begin").expect("reset branch");
    let end = stdout.find("end else begin").expect("reset branch end");
    let reset_block = &stdout[start..end];
    for line in [
        "s_axi_awready <= 1'b1;",
        "s_axi_wready <= 1'b1;",
        "s_axi_bvalid <= 1'b0;",
        "s_axi_arready <= 1'b1;",
        "s_axi_rvalid <= 1'b0;",
        "reg_ctrl <= 32'd0;",
        "reg_irq_en <= 32'd0;",
        "reg_weight_addr <= 64'd0;",
        "reg_input_addr <= 64'd0;",
        "reg_output_addr <= 64'd0;",
    ] {
        assert!(
            reset_block.contains(line),
            "missing reset line `{}` in the reset branch",
            line
        );
    }
}

// ============================================================================
// Output handling
// ============================================================================

#[test]
fn axi_output_to_file() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "t27_axi_lite_slave_test_{}.sv",
        std::process::id()
    ));
    let (_stdout, stderr, ok) = run(&[
        "gen-axi-lite-slave",
        "--output",
        path.to_str().unwrap(),
    ]);
    assert!(ok, "stderr: {}", stderr);
    let content = std::fs::read_to_string(&path).expect("read emitted file");
    assert!(content.contains("module axi_lite_slave #("));
    assert!(content.contains("endmodule"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn axi_emitted_text_is_ascii() {
    let (stdout, _stderr, ok) = run(&["gen-axi-lite-slave"]);
    assert!(ok);
    assert!(stdout.is_ascii(), "stdout must be ASCII");
}

#[test]
fn help_lists_axi_lite_slave_subcommand() {
    let (stdout, _stderr, ok) = run(&["--help"]);
    assert!(ok);
    assert!(stdout.contains("gen-axi-lite-slave"));
}

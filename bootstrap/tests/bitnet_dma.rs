//! Integration tests for `t27c gen-dma-controller` (Wave 36e, R-BN-5).

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
fn dma_default_emits_module() {
    let (stdout, stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok, "stderr: {}", stderr);
    assert!(stdout.contains("module dma_controller ("));
    assert!(stdout.contains("endmodule"));
}

#[test]
fn dma_custom_module_name() {
    let (stdout, _stderr, ok) =
        run(&["gen-dma-controller", "--module-name", "dma_ddr_to_bram"]);
    assert!(ok);
    assert!(stdout.contains("module dma_ddr_to_bram ("));
    assert!(!stdout.contains("module dma_controller ("));
}

#[test]
fn dma_invalid_module_name_falls_back() {
    for bad in &["9bad", "has space", "dash-name", ""] {
        let (stdout, _stderr, ok) =
            run(&["gen-dma-controller", "--module-name", bad]);
        assert!(ok, "command failed for invalid name `{}`", bad);
        assert!(
            stdout.contains("module dma_controller ("),
            "expected fallback for `{}`",
            bad
        );
    }
}

// ============================================================================
// FSM states
// ============================================================================

#[test]
fn dma_six_states_localparam() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    for st in [
        "localparam IDLE",
        "localparam READ_ADDR",
        "localparam READ_DATA",
        "localparam WRITE_ADDR",
        "localparam WRITE_DATA",
        "localparam DONE_ST",
    ] {
        assert!(stdout.contains(st), "missing `{}`", st);
    }
}

#[test]
fn dma_state_width_three_bits() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(stdout.contains("reg [2:0]  state;"));
}

#[test]
fn dma_idle_dispatch_on_direction() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(stdout.contains("state           <= direction ? WRITE_ADDR : READ_ADDR;"));
}

// ============================================================================
// AXI ports
// ============================================================================

#[test]
fn dma_axi_read_ports() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    for port in [
        "output reg  [63:0] m_axi_araddr,",
        "output reg  [7:0]  m_axi_arlen,",
        "output reg         m_axi_arvalid,",
        "input  wire        m_axi_arready,",
        "input  wire [63:0] m_axi_rdata,",
        "input  wire        m_axi_rlast,",
        "input  wire        m_axi_rvalid,",
        "output wire        m_axi_rready,",
    ] {
        assert!(stdout.contains(port), "missing AXI-read port `{}`", port);
    }
}

#[test]
fn dma_axi_write_ports() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    for port in [
        "output reg  [63:0] m_axi_awaddr,",
        "output reg  [7:0]  m_axi_awlen,",
        "output reg         m_axi_awvalid,",
        "input  wire        m_axi_awready,",
        "output reg  [63:0] m_axi_wdata,",
        "output reg         m_axi_wlast,",
        "output reg         m_axi_wvalid,",
        "input  wire        m_axi_wready,",
        "input  wire        m_axi_bvalid,",
        "output wire        m_axi_bready,",
    ] {
        assert!(stdout.contains(port), "missing AXI-write port `{}`", port);
    }
}

#[test]
fn dma_local_memory_interface() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(stdout.contains("output reg  [11:0] local_addr,"));
    assert!(stdout.contains("output reg  [63:0] local_wdata,"));
    assert!(stdout.contains("output reg         local_we,"));
    assert!(stdout.contains("input  wire [63:0] local_rdata"));
}

#[test]
fn dma_control_ports_present() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    for port in [
        "input  wire        start,",
        "input  wire [63:0] src_addr,",
        "input  wire [63:0] dst_addr,",
        "input  wire [31:0] length,",
        "input  wire        direction,",
        "output reg         busy,",
        "output reg         done,",
    ] {
        assert!(stdout.contains(port), "missing control port `{}`", port);
    }
}

// ============================================================================
// Handshake & burst semantics
// ============================================================================

#[test]
fn dma_continuous_assigns() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(stdout.contains("assign m_axi_rready = (state == READ_DATA);"));
    assert!(stdout.contains("assign m_axi_bready = 1'b1;"));
}

#[test]
// Renamed from `dma_burst_length_is_max`, which asserted the defect as if it
// were the contract: arlen/awlen were hardwired to 256 beats for every
// transfer while the FSM stopped after the bytes ran out, so a short transfer
// requested 256 beats and abandoned the burst. Yosys refuted
// `rready held until rlast` from a reachable state. Burst length is now
// derived from the bytes still owed.
fn dma_burst_length_is_derived_from_bytes_owed() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(!stdout.contains("m_axi_arlen  <= 8'hFF;"));
    assert!(!stdout.contains("m_axi_awlen  <= 8'hFF;"));
    assert!(stdout.contains("m_axi_arlen   <= burst_len;"));
    assert!(stdout.contains("m_axi_awlen   <= burst_len;"));
}

#[test]
fn dma_beat_decrement_by_eight_bytes() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    let occurrences = stdout.matches("bytes_remaining - 32'd8").count();
    assert!(
        occurrences >= 2,
        "expected >=2 byte-count decrements (read + write), got {}",
        occurrences
    );
}

#[test]
// wlast marks the last beat of the burst, not of the transfer.
fn dma_wlast_on_final_beat() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(stdout.contains("m_axi_wlast  <= (burst_count == m_axi_awlen);"));
}

#[test]
// The `||` in the old condition WAS the bug: leaving READ_DATA on a byte
// count rather than on rlast is exactly what abandoned the burst. The read
// path now leaves only on rlast, and chains another burst if bytes remain.
fn dma_read_terminates_only_on_rlast() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(!stdout.contains("if (m_axi_rlast || bytes_remaining <= 32'd8) state <= DONE_ST;"));
    assert!(stdout.contains("if (m_axi_rlast) begin"));
    assert!(stdout.contains("m_axi_araddr <= m_axi_araddr + burst_bytes_r;"));
}

#[test]
// This previously required `local_addr <= local_addr + 1` on BOTH paths, which
// pinned the defect rather than the behaviour: on the read path the address,
// the data and the write-enable all register together, so incrementing there
// made word N land at address N+1 -- slot 0 was never written and the last word
// wrapped over it. The read path now writes at the word's own index. On the
// write path local_addr is the READ pointer and still advances. Prop. 29.
fn dma_local_addr_walks_both_paths() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(
        stdout.contains("local_addr      <= word_index;"),
        "read path must write at the word's own index"
    );
    assert!(stdout.contains("word_index      <= word_index + 12'd1;"));
    assert!(
        stdout.contains("local_addr      <= local_addr + 12'd1;"),
        "write path advances the read pointer"
    );
}

#[test]
// A 32-bit length over a 12-bit local address wrapped and overwrote data
// already transferred, then reported success. Clamp to the address space and
// raise overflow, which the top routes to the previously tied-off error IRQ.
fn dma_clamps_oversized_length_and_reports_overflow() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(stdout.contains("output reg         overflow,"));
    assert!(stdout.contains("bytes_remaining <= (length > 32'd32768) ? 32'd32768 : length;"));
    assert!(stdout.contains("overflow        <= (length > 32'd32768);"));
}

// ============================================================================
// Reset & DONE handling
// ============================================================================

#[test]
fn dma_reset_initializes_outputs() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    for line in [
        "state          <= IDLE;",
        "busy           <= 1'b0;",
        "done           <= 1'b0;",
        "m_axi_arvalid  <= 1'b0;",
        "m_axi_awvalid  <= 1'b0;",
        "m_axi_wvalid   <= 1'b0;",
        "local_we       <= 1'b0;",
        "local_addr     <= 12'd0;",
        "bytes_remaining <= 32'd0;",
    ] {
        assert!(stdout.contains(line), "missing reset line `{}`", line);
    }
}

#[test]
fn dma_done_state_clears_busy_and_returns_idle() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(stdout.contains("busy     <= 1'b0;"));
    assert!(stdout.contains("done     <= 1'b1;"));
    assert!(stdout.contains("state    <= IDLE;"));
    // default arm for safety
    assert!(stdout.contains("default: state <= IDLE;"));
}

// ============================================================================
// File output & determinism
// ============================================================================

#[test]
fn dma_output_to_file() {
    let path = std::env::temp_dir()
        .join(format!("t27_dma_out_{}.sv", std::process::id()));
    let path_s = path.to_string_lossy().to_string();
    let (_stdout, stderr, ok) =
        run(&["gen-dma-controller", "--module-name", "dma_x", "--output", &path_s]);
    assert!(ok, "stderr: {}", stderr);
    let body = std::fs::read_to_string(&path).expect("output file missing");
    assert!(body.contains("module dma_x ("));
    assert!(body.trim_end().ends_with("endmodule"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn dma_output_is_deterministic() {
    let (a, _e1, ok1) = run(&["gen-dma-controller", "--module-name", "d1"]);
    let (b, _e2, ok2) = run(&["gen-dma-controller", "--module-name", "d1"]);
    assert!(ok1 && ok2);
    assert_eq!(a, b, "same args must yield byte-identical Verilog");
}

#[test]
fn dma_output_is_pure_ascii() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(stdout.is_ascii(), "emitted Verilog must be ASCII");
}

#[test]
fn dma_help_lists_subcommand() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller", "--help"]);
    assert!(ok);
    assert!(stdout.contains("--module-name"));
    assert!(stdout.contains("--output"));
}

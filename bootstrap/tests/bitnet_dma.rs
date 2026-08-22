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
fn dma_burst_length_is_max() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(stdout.contains("m_axi_arlen  <= 8'hFF;"));
    assert!(stdout.contains("m_axi_awlen  <= 8'hFF;"));
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
fn dma_wlast_on_final_beat() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(stdout.contains("m_axi_wlast  <= (bytes_remaining <= 32'd8);"));
}

#[test]
fn dma_rlast_or_count_terminates_read() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);
    assert!(stdout
        .contains("if (m_axi_rlast || bytes_remaining <= 32'd8) state <= DONE_ST;"));
}

/// Both transfer paths must advance the local address, but they do it by
/// different mechanisms and a single literal cannot see both.
///
/// This test used to count occurrences of `local_addr <= local_addr + 12'd1;`
/// across the whole module and require two. #2345 replaced the read path's
/// post-increment with `local_addr <= beat_index` — deliberately, because the
/// post-increment landed beat 0's data at address 1 and never wrote slot 0 —
/// so the count fell to one and the test failed. The test was stale, not the
/// emitter: the *property* it meant to hold (each path advances the
/// destination address) is still true.
///
/// It went unnoticed because `cargo test -p t27c --tests` stops at the first
/// failing target and this one is 42nd, so it never ran. See #2382.
///
/// Anchored per state arm rather than counted globally: a count over the whole
/// output cannot tell which path a match came from, which is what let the read
/// path silently lose its advance.
#[test]
fn dma_local_addr_advances_on_both_paths() {
    let (stdout, _stderr, ok) = run(&["gen-dma-controller"]);
    assert!(ok);

    let read_arm = arm(&stdout, "READ_DATA: if (m_axi_rvalid) begin", "end else local_we");
    // The read path presents the beat's own index, so beat 0 lands at address 0.
    assert!(
        read_arm.contains("local_addr      <= beat_index;"),
        "READ_DATA must present the beat index as the address, not post-increment \
         (that was #2003 — beat 0 landed at address 1 and slot 0 was never written). \
         READ_DATA arm was:\n{}",
        read_arm
    );
    assert!(
        read_arm.contains("beat_index      <= beat_index + 12'd1;"),
        "READ_DATA must advance beat_index, or every beat writes address 0. \
         READ_DATA arm was:\n{}",
        read_arm
    );

    // The write path's local_addr is a read pointer into local memory, so a
    // post-increment is correct there and must not be "fixed" to match the read path.
    let write_arm = arm(&stdout, "WRITE_DATA: begin", "WRITE_RESP");
    assert!(
        write_arm.contains("local_addr      <= local_addr + 12'd1;"),
        "WRITE_DATA must advance local_addr, or every beat reads the same word. \
         WRITE_DATA arm was:\n{}",
        write_arm
    );
}

/// Slice from `start` to the next `end_marker`, so an assertion cannot be
/// satisfied by an identical line in a different state.
fn arm<'a>(hay: &'a str, start: &str, end_marker: &str) -> &'a str {
    let from = hay
        .find(start)
        .unwrap_or_else(|| panic!("state arm not found: {start}\nin:\n{hay}"));
    let rest = &hay[from..];
    let to = rest.find(end_marker).unwrap_or(rest.len());
    &rest[..to]
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

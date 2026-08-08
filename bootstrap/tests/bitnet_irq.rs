//! Integration tests for `t27c gen-interrupt-controller` (Wave 36f, R-BN-6).

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
fn irq_default_emits_module() {
    let (stdout, stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok, "stderr: {}", stderr);
    assert!(stdout.contains("module interrupt_controller ("));
    assert!(stdout.contains("endmodule"));
}

#[test]
fn irq_custom_module_name() {
    let (stdout, _stderr, ok) =
        run(&["gen-interrupt-controller", "--module-name", "irq_ctrl"]);
    assert!(ok);
    assert!(stdout.contains("module irq_ctrl ("));
    assert!(!stdout.contains("module interrupt_controller ("));
}

#[test]
fn irq_invalid_module_name_falls_back() {
    for bad in &["9bad", "has space", "dash-name", ""] {
        let (stdout, _stderr, ok) =
            run(&["gen-interrupt-controller", "--module-name", bad]);
        assert!(ok, "command failed for invalid name `{}`", bad);
        assert!(
            stdout.contains("module interrupt_controller ("),
            "expected fallback for `{}`",
            bad
        );
    }
}

// ============================================================================
// IRQ sources and signal widths
// ============================================================================

#[test]
fn irq_three_sources_present() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    assert!(stdout.contains("input  wire        inference_done,"));
    assert!(stdout.contains("input  wire        dma_done,"));
    assert!(stdout.contains("input  wire        error,"));
}

#[test]
fn irq_enable_and_status_are_three_bits() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    assert!(stdout.contains("input  wire [2:0]  irq_enable,"));
    assert!(stdout.contains("output reg  [2:0]  irq_status,"));
}

#[test]
fn irq_output_port_present() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    assert!(stdout.contains("output wire        irq_out"));
}

#[test]
fn irq_status_read_input_present() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    assert!(stdout.contains("input  wire        status_read,"));
}

// ============================================================================
// Latch / clear / mask semantics
// ============================================================================

// Both tests below used to pin the literal text of the set-then-clear chain.
// They passed for exactly as long as the lost-interrupt race existed, and would
// have failed the moment it was fixed -- a test that asserts the shape of an
// implementation cannot notice that the implementation is wrong. Yosys proved
// the race outright: $past(inference_done) && $past(status_read) |->
// irq_status[0] == 0 held on every reachable state. See
// formal/interrupt_controller_props.sv (a_event_never_lost).

#[test]
fn irq_each_source_latches_its_bit() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    // [2]=error, [1]=dma_done, [0]=inference_done, contributed unconditionally.
    assert!(stdout.contains("| {error, dma_done, inference_done}"));
}

#[test]
fn irq_status_read_clears_latch() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    // Clear-on-read survives, but applies only to the previous value, so it
    // cannot discard a source asserted in the same cycle.
    assert!(stdout.contains("(status_read ? 3'b000 : irq_status)"));
    assert!(!stdout.contains("if (status_read)    irq_status     <= 3'b000;"));
}

#[test]
fn irq_reset_zeroes_status() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    assert!(stdout.contains("if (!rst_n) irq_status <= 3'b000;"));
}

#[test]
fn irq_out_is_or_of_masked_status() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    assert!(stdout.contains("assign irq_out = |(irq_status & irq_enable);"));
}

#[test]
fn irq_uses_negedge_reset() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    assert!(stdout.contains("always @(posedge clk or negedge rst_n)"));
}

// ============================================================================
// File output & determinism
// ============================================================================

#[test]
fn irq_output_to_file() {
    let path = std::env::temp_dir()
        .join(format!("t27_irq_out_{}.sv", std::process::id()));
    let path_s = path.to_string_lossy().to_string();
    let (_stdout, stderr, ok) = run(&[
        "gen-interrupt-controller",
        "--module-name",
        "irq_x",
        "--output",
        &path_s,
    ]);
    assert!(ok, "stderr: {}", stderr);
    let body = std::fs::read_to_string(&path).expect("output file missing");
    assert!(body.contains("module irq_x ("));
    assert!(body.trim_end().ends_with("endmodule"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn irq_output_is_deterministic() {
    let (a, _e1, ok1) = run(&["gen-interrupt-controller", "--module-name", "i1"]);
    let (b, _e2, ok2) = run(&["gen-interrupt-controller", "--module-name", "i1"]);
    assert!(ok1 && ok2);
    assert_eq!(a, b, "same args must yield byte-identical Verilog");
}

#[test]
fn irq_output_is_pure_ascii() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    assert!(stdout.is_ascii(), "emitted Verilog must be ASCII");
}

#[test]
fn irq_help_lists_subcommand() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller", "--help"]);
    assert!(ok);
    assert!(stdout.contains("--module-name"));
    assert!(stdout.contains("--output"));
}

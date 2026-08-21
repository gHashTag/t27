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

/// Count non-blocking assignments whose left-hand side is `irq_status` --
/// either the whole register or a bit-select -- returning
/// `(reset_arm, update_arm)`. A line mentioning `!rst_n` is the reset arm.
///
/// Mirrors the helper in `bootstrap/src/bitnet_irq.rs`; this crate has no
/// library target, so an integration test cannot import it.
fn count_irq_status_nba(v: &str) -> (usize, usize) {
    const LHS: &str = "irq_status";
    let mut reset = 0usize;
    let mut update = 0usize;
    for line in v.lines() {
        let code = match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        };
        let mut rest = code;
        while let Some(pos) = rest.find(LHS) {
            let after = rest[pos + LHS.len()..].trim_start();
            let after = if after.starts_with('[') {
                match after.find(']') {
                    Some(close) => after[close + 1..].trim_start(),
                    None => after,
                }
            } else {
                after
            };
            if after.starts_with("<=") {
                if code.contains("!rst_n") {
                    reset += 1;
                } else {
                    update += 1;
                }
            }
            rest = &rest[pos + LHS.len()..];
        }
    }
    (reset, update)
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

#[test]
fn irq_each_source_feeds_the_single_status_update() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    // MSB first: error=[2], dma_done=[1], inference_done=[0].
    assert!(stdout.contains("| {error, dma_done, inference_done};"));
}

#[test]
fn irq_status_read_clears_only_the_previous_value() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    // The clear selects the OLD value; this cycle's sources are OR'd on
    // top, so a read concurrent with an event does not discard it.
    assert!(stdout.contains("(status_read ? 3'b000 : irq_status)"));
}

#[test]
fn irq_status_has_exactly_one_driver_outside_reset() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    let (reset, update) = count_irq_status_nba(&stdout);
    assert_eq!(reset, 1, "expected exactly one reset assignment");
    assert_eq!(
        update, 1,
        "irq_status must have ONE non-blocking driver outside reset; two \
         or more resolve last-write-wins inside the same clocked block, \
         and the later write silently discards the earlier one (W555)"
    );
}

#[test]
fn irq_driver_count_sees_the_historical_race() {
    // Negative control for the test above: a counter that always returned
    // 1 would pass the racy design too. This is the pre-fix W36f update
    // arm verbatim -- four drivers, the clear last and unconditional.
    const RACY: &str = concat!(
        "        if (!rst_n) irq_status <= 3'b000;\n",
        "        else begin\n",
        "            if (inference_done) irq_status[0] <= 1'b1;\n",
        "            if (dma_done)       irq_status[1] <= 1'b1;\n",
        "            if (error)          irq_status[2] <= 1'b1;\n",
        "            if (status_read)    irq_status     <= 3'b000;\n",
        "        end\n",
    );
    let (reset, update) = count_irq_status_nba(RACY);
    assert_eq!(reset, 1);
    assert_eq!(
        update, 4,
        "the counter must SEE all four drivers of the historical chain"
    );
}

#[test]
fn irq_racy_chain_is_gone_from_the_emitted_module() {
    let (stdout, _stderr, ok) = run(&["gen-interrupt-controller"]);
    assert!(ok);
    for banned in [
        "if (inference_done) irq_status[0] <= 1'b1;",
        "if (dma_done)       irq_status[1] <= 1'b1;",
        "if (error)          irq_status[2] <= 1'b1;",
        "if (status_read)    irq_status     <= 3'b000;",
    ] {
        assert!(
            !stdout.contains(banned),
            "emitted module still carries the racy assignment: {}",
            banned
        );
    }
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

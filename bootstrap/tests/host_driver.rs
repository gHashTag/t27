// ============================================================================
// Wave 39 (R-HS-1) integration tests for the `t27c host-smoke` CLI and the
// host-side BitNet driver module.
//
// All tests invoke the binary via CARGO_BIN_EXE_t27c. No tempfile crate is
// used (not on the Cargo.toml).
//
// Closes #784.
// ============================================================================

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn run(args: &[&str]) -> (bool, String, String) {
    let out = Command::new(bin())
        .args(args)
        .output()
        .expect("failed to spawn t27c");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

#[test]
fn host_smoke_default_invocation_succeeds() {
    let (ok, stdout, _stderr) = run(&["host-smoke"]);
    assert!(ok, "default host-smoke should succeed");
    assert!(stdout.starts_with("OK "), "stdout = {stdout}");
}

#[test]
fn host_smoke_prints_canonical_layer_count_two() {
    let (ok, stdout, _stderr) = run(&["host-smoke"]);
    assert!(ok);
    assert!(stdout.contains("layers=2"), "stdout = {stdout}");
}

#[test]
fn host_smoke_prints_neurons_sixteen_by_default() {
    let (ok, stdout, _stderr) = run(&["host-smoke"]);
    assert!(ok);
    assert!(stdout.contains("neurons=16"), "stdout = {stdout}");
}

#[test]
fn host_smoke_prints_chunks_four_by_default() {
    let (ok, stdout, _stderr) = run(&["host-smoke"]);
    assert!(ok);
    assert!(stdout.contains("chunks=4"), "stdout = {stdout}");
}

#[test]
fn host_smoke_prints_threshold_one_by_default() {
    let (ok, stdout, _stderr) = run(&["host-smoke"]);
    assert!(ok);
    assert!(stdout.contains("threshold=1"), "stdout = {stdout}");
}

#[test]
fn host_smoke_prints_zero_weight_addr_by_default() {
    let (ok, stdout, _stderr) = run(&["host-smoke"]);
    assert!(ok);
    assert!(
        stdout.contains("weight_addr=0x0000000000000000"),
        "stdout = {stdout}"
    );
}

#[test]
fn host_smoke_accepts_custom_layers() {
    let (ok, stdout, _stderr) = run(&["host-smoke", "--num-layers", "5"]);
    assert!(ok);
    assert!(stdout.contains("layers=5"), "stdout = {stdout}");
}

#[test]
fn host_smoke_accepts_custom_neurons() {
    let (ok, stdout, _stderr) = run(&["host-smoke", "--neurons", "128"]);
    assert!(ok);
    assert!(stdout.contains("neurons=128"), "stdout = {stdout}");
}

#[test]
fn host_smoke_accepts_custom_chunks() {
    let (ok, stdout, _stderr) = run(&["host-smoke", "--chunks", "32"]);
    assert!(ok);
    assert!(stdout.contains("chunks=32"), "stdout = {stdout}");
}

#[test]
fn host_smoke_accepts_custom_threshold() {
    let (ok, stdout, _stderr) = run(&["host-smoke", "--threshold", "7"]);
    assert!(ok);
    assert!(stdout.contains("threshold=7"), "stdout = {stdout}");
}

#[test]
fn host_smoke_accepts_64bit_weight_addr() {
    let (ok, stdout, _stderr) = run(&[
        "host-smoke",
        "--weight-addr",
        "1099511627776", // 0x100_0000_0000
    ]);
    assert!(ok);
    assert!(
        stdout.contains("weight_addr=0x0000010000000000"),
        "stdout = {stdout}"
    );
}

#[test]
fn host_smoke_zero_layers_returns_error() {
    let (ok, _stdout, stderr) = run(&["host-smoke", "--num-layers", "0"]);
    assert!(!ok, "zero layers must fail");
    assert!(
        stderr.contains("configure failed") || stderr.contains("InvalidConfig"),
        "stderr = {stderr}"
    );
}

#[test]
fn host_smoke_zero_neurons_returns_error() {
    let (ok, _stdout, stderr) = run(&["host-smoke", "--neurons", "0"]);
    assert!(!ok);
    assert!(
        stderr.contains("configure failed") || stderr.contains("InvalidConfig"),
        "stderr = {stderr}"
    );
}

#[test]
fn host_smoke_zero_chunks_returns_error() {
    let (ok, _stdout, stderr) = run(&["host-smoke", "--chunks", "0"]);
    assert!(!ok);
    assert!(
        stderr.contains("configure failed") || stderr.contains("InvalidConfig"),
        "stderr = {stderr}"
    );
}

#[test]
fn host_smoke_writes_eight_csrs_total() {
    // 6 configure writes + 1 CTRL start + 1 IRQ_EN write = 8 writes.
    let (ok, stdout, _stderr) = run(&["host-smoke"]);
    assert!(ok);
    assert!(stdout.starts_with("OK 8w/"), "stdout = {stdout}");
}

#[test]
fn host_smoke_reads_at_least_ten_csrs() {
    // dump() performs 10 reads; wait_done adds >=1; total >=11.
    let (ok, stdout, _stderr) = run(&["host-smoke"]);
    assert!(ok);
    let after_slash = stdout
        .split_whitespace()
        .nth(1)
        .expect("expected 'NwMr' token");
    let reads_str = after_slash
        .split('/')
        .nth(1)
        .and_then(|s| s.strip_suffix('r'))
        .expect("expected reads suffix");
    let reads: u32 = reads_str.parse().expect("reads parse");
    assert!(reads >= 10, "expected >=10 reads, got {reads}");
}

#[test]
fn host_smoke_latches_inference_done_irq() {
    let (ok, stdout, _stderr) = run(&["host-smoke"]);
    assert!(ok);
    // IRQ_INFERENCE_DONE_MASK = 0x1, sticky-latched before wait_done.
    assert!(stdout.contains("irq_stat=0x00000001"), "stdout = {stdout}");
}

#[test]
fn host_smoke_help_lists_all_flags() {
    let (ok, stdout, _stderr) = run(&["host-smoke", "--help"]);
    assert!(ok);
    for flag in [
        "--num-layers",
        "--neurons",
        "--chunks",
        "--threshold",
        "--weight-addr",
        "--max-polls",
    ] {
        assert!(stdout.contains(flag), "missing {flag} in help: {stdout}");
    }
}

#[test]
fn host_smoke_max_polls_one_still_succeeds_when_done_preset() {
    // The helper presets STATUS.done before wait_done, so a single poll
    // is enough to satisfy completion.
    let (ok, stdout, _stderr) = run(&["host-smoke", "--max-polls", "1"]);
    assert!(ok);
    assert!(stdout.starts_with("OK "), "stdout = {stdout}");
}

#[test]
fn host_smoke_large_layer_count_still_succeeds() {
    let (ok, stdout, _stderr) = run(&["host-smoke", "--num-layers", "4294967295"]);
    assert!(ok);
    assert!(stdout.contains("layers=4294967295"), "stdout = {stdout}");
}

#[test]
fn host_smoke_combined_overrides_round_trip() {
    let (ok, stdout, _stderr) = run(&[
        "host-smoke",
        "--num-layers",
        "3",
        "--neurons",
        "64",
        "--chunks",
        "8",
        "--threshold",
        "42",
        "--weight-addr",
        "1024",
    ]);
    assert!(ok);
    for fragment in [
        "layers=3",
        "neurons=64",
        "chunks=8",
        "threshold=42",
        "weight_addr=0x0000000000000400",
    ] {
        assert!(stdout.contains(fragment), "missing {fragment}: {stdout}");
    }
}

#[test]
fn host_smoke_stdout_format_is_single_line() {
    let (ok, stdout, _stderr) = run(&["host-smoke"]);
    assert!(ok);
    // Trim trailing newline only; no internal newlines expected.
    let trimmed = stdout.trim_end_matches('\n');
    assert!(!trimmed.contains('\n'), "expected single line, got {stdout}");
}

#[test]
fn host_smoke_does_not_emit_to_stderr_on_success() {
    let (ok, _stdout, stderr) = run(&["host-smoke"]);
    assert!(ok);
    assert!(stderr.trim().is_empty(), "stderr should be empty: {stderr}");
}

#[test]
fn host_smoke_consistent_across_repeated_runs() {
    let (ok1, s1, _) = run(&["host-smoke"]);
    let (ok2, s2, _) = run(&["host-smoke"]);
    assert!(ok1 && ok2);
    assert_eq!(s1, s2, "host-smoke should be deterministic");
}

#[test]
fn host_smoke_help_mentions_wave_39() {
    let (ok, stdout, _stderr) = run(&["host-smoke", "--help"]);
    assert!(ok);
    // The doc-comment on the CLI variant mentions Wave 39.
    assert!(
        stdout.contains("Wave 39") || stdout.contains("R-HS-1"),
        "expected Wave 39 / R-HS-1 in help: {stdout}"
    );
}

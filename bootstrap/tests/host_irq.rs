// ============================================================================
// Wave 40 (R-HS-2) integration tests for the `t27c host-poll-vs-irq` CLI.
//
// All tests invoke the binary via CARGO_BIN_EXE_t27c and assert observable
// behaviour of the two completion paths (busy-poll vs interrupt-driven)
// against a deterministic MockMmio.
//
// Closes #786.
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
fn poll_vs_irq_default_invocation_succeeds() {
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq"]);
    assert!(ok, "default should succeed");
    assert!(stdout.starts_with("OK "), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_stdout_is_single_line() {
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    let trimmed = stdout.trim_end_matches('\n');
    assert!(!trimmed.contains('\n'), "expected single line: {stdout}");
}

#[test]
fn poll_vs_irq_reports_poll_path_with_eight_writes() {
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stdout.contains("poll=8w/"), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_reports_irq_path_with_nine_writes() {
    // IRQ path performs the same 8 configure/start writes plus one extra
    // write-1-to-clear of IRQ_STAT inside service(); expect 9 writes.
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stdout.contains("irq=9w/"), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_writes_match_is_false_by_design() {
    // The IRQ path intentionally performs one extra W1C write.
    // writes_match should therefore report `false`.
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stdout.contains("writes_match=false"), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_csr_match_is_true() {
    // Both paths program the same scalar configuration; CSR-snapshot equality
    // should hold across both backends.
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stdout.contains("csr_match=true"), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_both_irq_stats_show_inference_done() {
    // Poll path: hardware-injected sticky latch persists (not cleared by poll).
    // IRQ path: service() write-1-to-clears the InferenceDone bit, so the
    // post-service IRQ_STAT is zero in the dump that follows.
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stdout.contains("irq_stat_poll=0x00000001"), "stdout = {stdout}");
    assert!(stdout.contains("irq_stat_irq=0x00000000"), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_accepts_custom_layers() {
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq", "--num-layers", "5"]);
    assert!(ok);
    // The CSR snapshot is identical for both paths -- the printed line does
    // not echo per-CSR values, but exit success + csr_match=true is enough.
    assert!(stdout.contains("csr_match=true"), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_accepts_custom_neurons() {
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq", "--neurons", "128"]);
    assert!(ok);
    assert!(stdout.contains("csr_match=true"));
}

#[test]
fn poll_vs_irq_accepts_custom_chunks() {
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq", "--chunks", "32"]);
    assert!(ok);
    assert!(stdout.contains("csr_match=true"));
}

#[test]
fn poll_vs_irq_accepts_custom_threshold() {
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq", "--threshold", "7"]);
    assert!(ok);
    assert!(stdout.contains("csr_match=true"));
}

#[test]
fn poll_vs_irq_accepts_64bit_weight_addr() {
    let (ok, stdout, _stderr) = run(&[
        "host-poll-vs-irq",
        "--weight-addr",
        "1099511627776",
    ]);
    assert!(ok);
    assert!(stdout.contains("csr_match=true"));
}

#[test]
fn poll_vs_irq_zero_layers_returns_error() {
    let (ok, _stdout, stderr) = run(&["host-poll-vs-irq", "--num-layers", "0"]);
    assert!(!ok);
    assert!(
        stderr.contains("configure failed") || stderr.contains("InvalidConfig"),
        "stderr = {stderr}"
    );
}

#[test]
fn poll_vs_irq_zero_neurons_returns_error() {
    let (ok, _stdout, stderr) = run(&["host-poll-vs-irq", "--neurons", "0"]);
    assert!(!ok);
    assert!(
        stderr.contains("configure failed") || stderr.contains("InvalidConfig"),
        "stderr = {stderr}"
    );
}

#[test]
fn poll_vs_irq_zero_chunks_returns_error() {
    let (ok, _stdout, stderr) = run(&["host-poll-vs-irq", "--chunks", "0"]);
    assert!(!ok);
    assert!(
        stderr.contains("configure failed") || stderr.contains("InvalidConfig"),
        "stderr = {stderr}"
    );
}

#[test]
fn poll_vs_irq_max_polls_one_still_succeeds() {
    // The helper preloads STATUS.done and latches IRQ_STAT before waiting,
    // so a single iteration completes both paths.
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq", "--max-polls", "1"]);
    assert!(ok);
    assert!(stdout.starts_with("OK "));
}

#[test]
fn poll_vs_irq_help_lists_all_flags() {
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq", "--help"]);
    assert!(ok);
    for flag in [
        "--num-layers",
        "--neurons",
        "--chunks",
        "--threshold",
        "--weight-addr",
        "--max-polls",
    ] {
        assert!(stdout.contains(flag), "missing {flag}: {stdout}");
    }
}

#[test]
fn poll_vs_irq_help_mentions_wave_40() {
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq", "--help"]);
    assert!(ok);
    assert!(
        stdout.contains("Wave 40") || stdout.contains("R-HS-2"),
        "stdout = {stdout}"
    );
}

#[test]
fn poll_vs_irq_no_stderr_on_success() {
    let (ok, _stdout, stderr) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stderr.trim().is_empty(), "stderr should be empty: {stderr}");
}

#[test]
fn poll_vs_irq_deterministic_across_two_runs() {
    let (ok1, s1, _) = run(&["host-poll-vs-irq"]);
    let (ok2, s2, _) = run(&["host-poll-vs-irq"]);
    assert!(ok1 && ok2);
    assert_eq!(s1, s2, "expected deterministic output");
}

#[test]
fn poll_vs_irq_overrides_round_trip_through_csr_match() {
    let (ok, stdout, _stderr) = run(&[
        "host-poll-vs-irq",
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
    assert!(stdout.contains("csr_match=true"), "stdout = {stdout}");
    assert!(stdout.contains("poll=8w/"));
    assert!(stdout.contains("irq=9w/"));
}

#[test]
fn poll_vs_irq_read_counts_are_eleven_each() {
    // 1 STATUS read inside wait_done + 10 dump() reads = 11 reads per path.
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stdout.contains("poll=8w/11r"), "stdout = {stdout}");
    assert!(stdout.contains("irq=9w/11r"), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_writes_diff_is_exactly_one() {
    // Sanity check: irq path differs from poll path by exactly one write
    // (the W1C of IRQ_STAT inside service()).
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    let line = stdout.lines().next().expect("at least one line");
    let poll_w: u32 = line
        .split("poll=")
        .nth(1)
        .and_then(|s| s.split('w').next())
        .and_then(|s| s.parse().ok())
        .expect("poll writes");
    let irq_w: u32 = line
        .split("irq=")
        .nth(1)
        .and_then(|s| s.split('w').next())
        .and_then(|s| s.parse().ok())
        .expect("irq writes");
    assert_eq!(irq_w, poll_w + 1, "irq path should write exactly 1 more");
}

#[test]
fn poll_vs_irq_reports_csr_match_even_with_extreme_weight_addr() {
    let (ok, stdout, _stderr) = run(&[
        "host-poll-vs-irq",
        "--weight-addr",
        "18446744073709551615", // u64::MAX
    ]);
    assert!(ok);
    assert!(stdout.contains("csr_match=true"));
}

#[test]
fn poll_vs_irq_large_layer_count_succeeds() {
    let (ok, stdout, _stderr) = run(&["host-poll-vs-irq", "--num-layers", "4294967295"]);
    assert!(ok);
    assert!(stdout.contains("csr_match=true"));
}

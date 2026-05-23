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
fn poll_vs_irq_default_succeeds() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq"]);
    assert!(ok, "default should succeed");
    assert!(stdout.starts_with("OK "), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_prints_poll_metrics() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stdout.contains("poll="), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_prints_irq_metrics() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stdout.contains("irq="), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_writes_match_field_present() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stdout.contains("writes_match="), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_prints_irq_stat_poll() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stdout.contains("irq_stat_poll=0x"), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_prints_irq_stat_irq() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stdout.contains("irq_stat_irq=0x"), "stdout = {stdout}");
}

#[test]
fn poll_vs_irq_custom_layers() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq", "--num-layers", "5"]);
    assert!(ok);
    assert!(stdout.starts_with("OK "));
}

#[test]
fn poll_vs_irq_custom_neurons() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq", "--neurons", "128"]);
    assert!(ok);
}

#[test]
fn poll_vs_irq_custom_chunks() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq", "--chunks", "32"]);
    assert!(ok);
}

#[test]
fn poll_vs_irq_custom_threshold() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq", "--threshold", "7"]);
    assert!(ok);
}

#[test]
fn poll_vs_irq_custom_weight_addr() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq", "--weight-addr", "1099511627776"]);
    assert!(ok);
}

#[test]
fn poll_vs_irq_zero_layers_fails() {
    let (ok, _, stderr) = run(&["host-poll-vs-irq", "--num-layers", "0"]);
    assert!(!ok);
    assert!(stderr.contains("configure") || stderr.contains("InvalidConfig"));
}

#[test]
fn poll_vs_irq_zero_neurons_fails() {
    let (ok, _, _stderr) = run(&["host-poll-vs-irq", "--neurons", "0"]);
    assert!(!ok);
}

#[test]
fn poll_vs_irq_zero_chunks_fails() {
    let (ok, _, _stderr) = run(&["host-poll-vs-irq", "--chunks", "0"]);
    assert!(!ok);
}

#[test]
fn poll_vs_irq_max_polls_one_succeeds() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq", "--max-polls", "1"]);
    assert!(ok, "done is preset, 1 poll is enough");
    assert!(stdout.starts_with("OK "));
}

#[test]
fn poll_vs_irq_deterministic() {
    let (ok1, s1, _) = run(&["host-poll-vs-irq"]);
    let (ok2, s2, _) = run(&["host-poll-vs-irq"]);
    assert!(ok1 && ok2);
    assert_eq!(s1, s2, "should be deterministic");
}

#[test]
fn poll_vs_irq_single_line_output() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    let trimmed = stdout.trim_end_matches('\n');
    assert!(!trimmed.contains('\n'), "expected single line, got {stdout}");
}

#[test]
fn poll_vs_irq_no_stderr_on_success() {
    let (ok, _, stderr) = run(&["host-poll-vs-irq"]);
    assert!(ok);
    assert!(stderr.trim().is_empty(), "stderr should be empty: {stderr}");
}

#[test]
fn poll_vs_irq_combined_overrides() {
    let (ok, stdout, _) = run(&[
        "host-poll-vs-irq",
        "--num-layers", "3",
        "--neurons", "64",
        "--chunks", "8",
        "--threshold", "42",
        "--weight-addr", "1024",
    ]);
    assert!(ok);
    assert!(stdout.contains("writes_match="));
}

#[test]
fn poll_vs_irq_help_lists_all_flags() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq", "--help"]);
    assert!(ok);
    for flag in ["--num-layers", "--neurons", "--chunks", "--threshold", "--weight-addr", "--max-polls"] {
        assert!(stdout.contains(flag), "missing {flag} in help: {stdout}");
    }
}

#[test]
fn poll_vs_irq_help_mentions_wave_40() {
    let (ok, stdout, _) = run(&["host-poll-vs-irq", "--help"]);
    assert!(ok);
    assert!(stdout.contains("Wave 40") || stdout.contains("R-HS-2"), "expected Wave 40 / R-HS-2: {stdout}");
}

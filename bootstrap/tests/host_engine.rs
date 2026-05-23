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
fn inference_default_succeeds() {
    let (ok, stdout, _) = run(&["host-inference"]);
    assert!(ok, "default should succeed");
    assert!(stdout.starts_with("OK "), "stdout = {stdout}");
}

#[test]
fn inference_prints_layer_count() {
    let (ok, stdout, _) = run(&["host-inference"]);
    assert!(ok);
    assert!(stdout.contains("layers=2"), "stdout = {stdout}");
}

#[test]
fn inference_prints_completed_count() {
    let (ok, stdout, _) = run(&["host-inference"]);
    assert!(ok);
    assert!(stdout.contains("completed=2"), "stdout = {stdout}");
}

#[test]
fn inference_prints_writes() {
    let (ok, stdout, _) = run(&["host-inference"]);
    assert!(ok);
    assert!(stdout.contains("writes="), "stdout = {stdout}");
}

#[test]
fn inference_prints_reads() {
    let (ok, stdout, _) = run(&["host-inference"]);
    assert!(ok);
    assert!(stdout.contains("reads="), "stdout = {stdout}");
}

#[test]
fn inference_single_layer() {
    let (ok, stdout, _) = run(&["host-inference", "--num-layers", "1"]);
    assert!(ok);
    assert!(stdout.contains("layers=1 completed=1"), "stdout = {stdout}");
}

#[test]
fn inference_three_layers() {
    let (ok, stdout, _) = run(&["host-inference", "--num-layers", "3"]);
    assert!(ok);
    assert!(stdout.contains("layers=3 completed=3"), "stdout = {stdout}");
}

#[test]
fn inference_custom_neurons() {
    let (ok, stdout, _) = run(&["host-inference", "--neurons", "128"]);
    assert!(ok);
}

#[test]
fn inference_custom_chunks() {
    let (ok, stdout, _) = run(&["host-inference", "--chunks", "32"]);
    assert!(ok);
}

#[test]
fn inference_custom_threshold() {
    let (ok, stdout, _) = run(&["host-inference", "--threshold", "42"]);
    assert!(ok);
}

#[test]
fn inference_custom_weight_addr() {
    let (ok, stdout, _) = run(&["host-inference", "--weight-addr", "1099511627776"]);
    assert!(ok);
}

#[test]
fn inference_zero_layers_fails() {
    let (ok, _, stderr) = run(&["host-inference", "--num-layers", "0"]);
    assert!(!ok);
    assert!(stderr.contains("configure") || stderr.contains("InvalidConfig"));
}

#[test]
fn inference_zero_neurons_fails() {
    let (ok, _, _stderr) = run(&["host-inference", "--neurons", "0"]);
    assert!(!ok);
}

#[test]
fn inference_zero_chunks_fails() {
    let (ok, _, _stderr) = run(&["host-inference", "--chunks", "0"]);
    assert!(!ok);
}

#[test]
fn inference_deterministic() {
    let (ok1, s1, _) = run(&["host-inference"]);
    let (ok2, s2, _) = run(&["host-inference"]);
    assert!(ok1 && ok2);
    assert_eq!(s1, s2, "should be deterministic");
}

#[test]
fn inference_single_line_output() {
    let (ok, stdout, _) = run(&["host-inference"]);
    assert!(ok);
    let trimmed = stdout.trim_end_matches('\n');
    assert!(!trimmed.contains('\n'), "expected single line, got {stdout}");
}

#[test]
fn inference_no_stderr_on_success() {
    let (ok, _, stderr) = run(&["host-inference"]);
    assert!(ok);
    assert!(stderr.trim().is_empty(), "stderr should be empty: {stderr}");
}

#[test]
fn inference_combined_overrides() {
    let (ok, stdout, _) = run(&[
        "host-inference",
        "--num-layers", "3",
        "--neurons", "64",
        "--chunks", "8",
        "--threshold", "42",
        "--weight-addr", "1024",
    ]);
    assert!(ok);
    assert!(stdout.contains("layers=3 completed=3"));
}

#[test]
fn inference_help_lists_all_flags() {
    let (ok, stdout, _) = run(&["host-inference", "--help"]);
    assert!(ok);
    for flag in ["--num-layers", "--neurons", "--chunks", "--threshold", "--weight-addr", "--max-rounds"] {
        assert!(stdout.contains(flag), "missing {flag} in help: {stdout}");
    }
}

#[test]
fn inference_help_mentions_wave_41() {
    let (ok, stdout, _) = run(&["host-inference", "--help"]);
    assert!(ok);
    assert!(stdout.contains("Wave 41") || stdout.contains("R-HS-3"), "expected Wave 41 / R-HS-3: {stdout}");
}

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
fn perf_default_succeeds() {
    let (ok, stdout, _) = run(&["host-perf"]);
    assert!(ok, "default should succeed");
    assert!(stdout.starts_with("OK "), "stdout = {stdout}");
}

#[test]
fn perf_prints_layer_count() {
    let (ok, stdout, _) = run(&["host-perf"]);
    assert!(ok);
    assert!(stdout.contains("layers=2"), "stdout = {stdout}");
}

#[test]
fn perf_prints_neuron_count() {
    let (ok, stdout, _) = run(&["host-perf"]);
    assert!(ok);
    assert!(stdout.contains("neurons=16"), "stdout = {stdout}");
}

#[test]
fn perf_prints_chunk_count() {
    let (ok, stdout, _) = run(&["host-perf"]);
    assert!(ok);
    assert!(stdout.contains("chunks=4"), "stdout = {stdout}");
}

#[test]
fn perf_prints_total_cycles() {
    let (ok, stdout, _) = run(&["host-perf"]);
    assert!(ok);
    assert!(stdout.contains("total_cycles="), "stdout = {stdout}");
}

#[test]
fn perf_prints_weight_words() {
    let (ok, stdout, _) = run(&["host-perf"]);
    assert!(ok);
    assert!(stdout.contains("total_weight_words="), "stdout = {stdout}");
}

#[test]
fn perf_prints_bram_pct() {
    let (ok, stdout, _) = run(&["host-perf"]);
    assert!(ok);
    assert!(stdout.contains("bram_pct="), "stdout = {stdout}");
}

#[test]
fn perf_prints_dma_beats() {
    let (ok, stdout, _) = run(&["host-perf"]);
    assert!(ok);
    assert!(stdout.contains("dma_beats="), "stdout = {stdout}");
}

#[test]
fn perf_prints_throughput() {
    let (ok, stdout, _) = run(&["host-perf"]);
    assert!(ok);
    assert!(stdout.contains("throughput="), "stdout = {stdout}");
    assert!(stdout.contains("inf/s"), "stdout = {stdout}");
}

#[test]
fn perf_prints_clock_freq() {
    let (ok, stdout, _) = run(&["host-perf"]);
    assert!(ok);
    assert!(stdout.contains("MHz"), "stdout = {stdout}");
}

#[test]
fn perf_custom_layers() {
    let (ok, stdout, _) = run(&["host-perf", "--num-layers", "5"]);
    assert!(ok);
    assert!(stdout.contains("layers=5"), "stdout = {stdout}");
}

#[test]
fn perf_custom_neurons() {
    let (ok, stdout, _) = run(&["host-perf", "--neurons", "128"]);
    assert!(ok);
    assert!(stdout.contains("neurons=128"), "stdout = {stdout}");
}

#[test]
fn perf_custom_chunks() {
    let (ok, stdout, _) = run(&["host-perf", "--chunks", "32"]);
    assert!(ok);
    assert!(stdout.contains("chunks=32"), "stdout = {stdout}");
}

#[test]
fn perf_custom_clock() {
    let (ok, stdout, _) = run(&["host-perf", "--clock-mhz", "100.0"]);
    assert!(ok);
    assert!(stdout.contains("@ 100.0 MHz"), "stdout = {stdout}");
}

#[test]
fn perf_zero_layers_fails() {
    let (ok, _, stderr) = run(&["host-perf", "--num-layers", "0"]);
    assert!(!ok);
    assert!(stderr.contains("invalid config"), "stderr = {stderr}");
}

#[test]
fn perf_zero_neurons_fails() {
    let (ok, _, _stderr) = run(&["host-perf", "--neurons", "0"]);
    assert!(!ok);
}

#[test]
fn perf_zero_chunks_fails() {
    let (ok, _, _stderr) = run(&["host-perf", "--chunks", "0"]);
    assert!(!ok);
}

#[test]
fn perf_deterministic() {
    let (ok1, s1, _) = run(&["host-perf"]);
    let (ok2, s2, _) = run(&["host-perf"]);
    assert!(ok1 && ok2);
    assert_eq!(s1, s2, "should be deterministic");
}

#[test]
fn perf_single_line_output() {
    let (ok, stdout, _) = run(&["host-perf"]);
    assert!(ok);
    let trimmed = stdout.trim_end_matches('\n');
    assert!(!trimmed.contains('\n'), "expected single line, got {stdout}");
}

#[test]
fn perf_no_stderr_on_success() {
    let (ok, _, stderr) = run(&["host-perf"]);
    assert!(ok);
    assert!(stderr.trim().is_empty(), "stderr should be empty: {stderr}");
}

#[test]
fn perf_help_lists_all_flags() {
    let (ok, stdout, _) = run(&["host-perf", "--help"]);
    assert!(ok);
    for flag in ["--num-layers", "--neurons", "--chunks", "--clock-mhz"] {
        assert!(stdout.contains(flag), "missing {flag} in help: {stdout}");
    }
}

#[test]
fn perf_help_mentions_wave_42() {
    let (ok, stdout, _) = run(&["host-perf", "--help"]);
    assert!(ok);
    assert!(stdout.contains("Wave 42") || stdout.contains("R-HS-4"), "expected Wave 42 / R-HS-4: {stdout}");
}

#[test]
fn perf_cycles_increase_with_layers() {
    let (ok1, s1, _) = run(&["host-perf", "--num-layers", "1"]);
    let (ok2, s2, _) = run(&["host-perf", "--num-layers", "4"]);
    assert!(ok1 && ok2);
    let c1: u64 = extract_total_cycles(&s1);
    let c2: u64 = extract_total_cycles(&s2);
    assert!(c2 > c1, "4-layer cycles ({c2}) should exceed 1-layer ({c1})");
}

fn extract_total_cycles(s: &str) -> u64 {
    let part = s.split("total_cycles=").nth(1).unwrap_or("");
    part.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0)
}

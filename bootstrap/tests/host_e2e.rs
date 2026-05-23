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

fn run_e2e(extra: &[&str]) -> (bool, String, String) {
    let mut args = vec!["host-e2e"];
    args.extend(extra);
    run(&args)
}

#[test]
fn e2e_default_succeeds() {
    let (ok, stdout, _) = run_e2e(&[]);
    assert!(ok);
    assert!(stdout.starts_with("OK "), "stdout = {stdout}");
}

#[test]
fn e2e_prints_layers() {
    let (ok, stdout, _) = run_e2e(&[]);
    assert!(ok);
    assert!(stdout.contains("layers=2"), "stdout = {stdout}");
}

#[test]
fn e2e_prints_completed() {
    let (ok, stdout, _) = run_e2e(&[]);
    assert!(ok);
    assert!(stdout.contains("completed=2"), "stdout = {stdout}");
}

#[test]
fn e2e_prints_pattern() {
    let (ok, stdout, _) = run_e2e(&[]);
    assert!(ok);
    assert!(stdout.contains("pattern=alternating"), "stdout = {stdout}");
}

#[test]
fn e2e_prints_weight_words() {
    let (ok, stdout, _) = run_e2e(&[]);
    assert!(ok);
    assert!(stdout.contains("weight_words="), "stdout = {stdout}");
}

#[test]
fn e2e_prints_writes_reads() {
    let (ok, stdout, _) = run_e2e(&[]);
    assert!(ok);
    assert!(stdout.contains("writes="), "stdout = {stdout}");
    assert!(stdout.contains("reads="), "stdout = {stdout}");
}

#[test]
fn e2e_prints_estimated_metrics() {
    let (ok, stdout, _) = run_e2e(&[]);
    assert!(ok);
    assert!(stdout.contains("est_cycles="), "stdout = {stdout}");
    assert!(stdout.contains("est_dma="), "stdout = {stdout}");
    assert!(stdout.contains("bram="), "stdout = {stdout}");
}

#[test]
fn e2e_single_layer() {
    let (ok, stdout, _) = run_e2e(&["--num-layers=1"]);
    assert!(ok);
    assert!(stdout.contains("layers=1 completed=1"));
}

#[test]
fn e2e_custom_neurons() {
    let (ok, stdout, _) = run_e2e(&["--neurons=32"]);
    assert!(ok);
    assert!(stdout.contains("weight_words=128"), "32*4=128: {stdout}");
}

#[test]
fn e2e_all_n_pattern() {
    let (ok, stdout, _) = run_e2e(&["--pattern=all-n"]);
    assert!(ok);
    assert!(stdout.contains("pattern=all-n"));
}

#[test]
fn e2e_seeded_pattern() {
    let (ok, stdout, _) = run_e2e(&["--pattern=seeded-random:42"]);
    assert!(ok);
}

#[test]
fn e2e_invalid_pattern_fails() {
    let (ok, _, stderr) = run_e2e(&["--pattern=bogus"]);
    assert!(!ok);
    assert!(stderr.contains("invalid pattern"));
}

#[test]
fn e2e_json_valid() {
    let (ok, stdout, _) = run_e2e(&["--json"]);
    assert!(ok);
    let v: serde_json::Value = stdout.trim().parse().unwrap();
    assert_eq!(v["ok"], true);
}

#[test]
fn e2e_json_has_all_fields() {
    let (ok, stdout, _) = run_e2e(&["--json"]);
    assert!(ok);
    let v: serde_json::Value = stdout.trim().parse().unwrap();
    for key in ["layers", "neurons", "chunks", "pattern", "weight_words", "layers_completed", "total_writes", "total_reads", "estimated_cycles", "estimated_dma_beats", "bram_pct", "weight_gen_ok"] {
        assert!(v.get(key).is_some(), "missing field {key}");
    }
}

#[test]
fn e2e_deterministic() {
    let (ok1, s1, _) = run_e2e(&[]);
    let (ok2, s2, _) = run_e2e(&[]);
    assert!(ok1 && ok2);
    assert_eq!(s1, s2);
}

#[test]
fn e2e_single_line_output() {
    let (ok, stdout, _) = run_e2e(&[]);
    assert!(ok);
    let trimmed = stdout.trim_end_matches('\n');
    assert!(!trimmed.contains('\n'));
}

#[test]
fn e2e_no_stderr_on_success() {
    let (ok, _, stderr) = run_e2e(&[]);
    assert!(ok);
    assert!(stderr.trim().is_empty());
}

#[test]
fn e2e_help_lists_flags() {
    let (ok, stdout, _) = run_e2e(&["--help"]);
    assert!(ok);
    for flag in ["--num-layers", "--neurons", "--chunks", "--threshold", "--pattern", "--json"] {
        assert!(stdout.contains(flag), "missing {flag}");
    }
}

#[test]
fn e2e_help_mentions_wave_46() {
    let (ok, stdout, _) = run_e2e(&["--help"]);
    assert!(ok);
    assert!(stdout.contains("Wave 46") || stdout.contains("R-HS-6"), "expected Wave 46: {stdout}");
}

#[test]
fn e2e_json_deterministic() {
    let (ok1, s1, _) = run_e2e(&["--json"]);
    let (ok2, s2, _) = run_e2e(&["--json"]);
    assert!(ok1 && ok2);
    assert_eq!(s1, s2);
}

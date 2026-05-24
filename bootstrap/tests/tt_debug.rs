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

fn write_manifest(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    let json = r#"{"commit_hash":"a1b2c3d4","chip_slug":"euler_phi","phi_invariant_hash":"deadbeef","timestamp_utc":"2026-01-01T00:00:00Z"}"#;
    std::fs::write(&path, json).unwrap();
    path
}

fn run_wrapper(extra: &[&str]) -> (bool, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let manifest = write_manifest(dir.path(), "manifest.json");
    let manifest_str = manifest.to_str().unwrap();
    let mut args = vec!["gen-tt-debug-wrapper", "--manifest", manifest_str];
    args.extend(extra);
    run(&args)
}

#[test]
fn wrapper_default_succeeds() {
    let (ok, stdout, _) = run_wrapper(&[]);
    assert!(ok, "gen-tt-debug-wrapper should succeed");
    assert!(stdout.contains("module bitnet_engine_top_debug_wrapper"), "stdout={}", stdout);
}

#[test]
fn wrapper_has_version_localparam() {
    let (ok, stdout, _) = run_wrapper(&[]);
    assert!(ok);
    assert!(stdout.contains("VERSION_WORD"), "stdout={}", stdout);
}

#[test]
fn wrapper_has_inner_instantiation() {
    let (ok, stdout, _) = run_wrapper(&[]);
    assert!(ok);
    assert!(stdout.contains("bitnet_engine_top u_inner"), "stdout={}", stdout);
}

#[test]
fn wrapper_has_error_counters() {
    let (ok, stdout, _) = run_wrapper(&[]);
    assert!(ok);
    for sig in ["err_axi_ctr", "err_dma_ctr", "err_irq_ctr", "err_csr_ctr"] {
        assert!(stdout.contains(sig), "missing {}", sig);
    }
}

#[test]
fn wrapper_has_selftest() {
    let (ok, stdout, _) = run_wrapper(&[]);
    assert!(ok);
    assert!(stdout.contains("selftest_pass"));
    assert!(stdout.contains("selftest_fail"));
    assert!(stdout.contains("DEAD_BEEF"));
}

#[test]
fn wrapper_has_axi_read_mux() {
    let (ok, stdout, _) = run_wrapper(&[]);
    assert!(ok);
    assert!(stdout.contains("case (axi_addr)"));
    assert!(stdout.contains("32'h40:"));
}

#[test]
fn wrapper_ends_with_endmodule() {
    let (ok, stdout, _) = run_wrapper(&[]);
    assert!(ok);
    assert!(stdout.trim().ends_with("endmodule"));
}

#[test]
fn wrapper_output_is_ascii() {
    let (ok, stdout, _) = run_wrapper(&[]);
    assert!(ok);
    assert!(stdout.is_ascii());
}

#[test]
fn wrapper_custom_inner_name() {
    let (ok, stdout, _) = run_wrapper(&["--inner", "my_accel"]);
    assert!(ok);
    assert!(stdout.contains("module my_accel_debug_wrapper"));
    assert!(stdout.contains("my_accel u_inner"));
}

#[test]
fn wrapper_output_to_file() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = write_manifest(dir.path(), "manifest.json");
    let out_path = dir.path().join("wrapper.sv");
    let (ok, _, _) = run(&[
        "gen-tt-debug-wrapper",
        "--manifest", manifest.to_str().unwrap(),
        "--output", out_path.to_str().unwrap(),
    ]);
    assert!(ok);
    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("module bitnet_engine_top_debug_wrapper"));
}

#[test]
fn wrapper_missing_manifest_fails() {
    let (ok, _, stderr) = run(&[
        "gen-tt-debug-wrapper",
        "--manifest", "/nonexistent/manifest.json",
    ]);
    assert!(!ok);
    assert!(stderr.contains("cannot read manifest") || stderr.contains("No such file"));
}

#[test]
fn wrapper_invalid_manifest_json_fails() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad.json");
    std::fs::write(&bad, "not json").unwrap();
    let (ok, _, stderr) = run(&[
        "gen-tt-debug-wrapper",
        "--manifest", bad.to_str().unwrap(),
    ]);
    assert!(!ok);
    assert!(stderr.contains("cannot parse manifest"));
}

#[test]
fn wrapper_provenance_in_comments() {
    let (ok, stdout, _) = run_wrapper(&[]);
    assert!(ok);
    assert!(stdout.contains("Provenance:"));
    assert!(stdout.contains("commit="));
    assert!(stdout.contains("phi="));
}

#[test]
fn wrapper_deterministic() {
    let (_, s1, _) = run_wrapper(&[]);
    let (_, s2, _) = run_wrapper(&[]);
    assert_eq!(s1, s2);
}

#[test]
fn wrapper_event_inputs() {
    let (ok, stdout, _) = run_wrapper(&[]);
    assert!(ok);
    for cls in ["event_axi_protocol", "event_dma_underrun", "event_irq_stuck", "event_csr_bad_offset"] {
        assert!(stdout.contains(cls), "missing event input: {}", cls);
    }
}

#[test]
fn wrapper_pass_through_ports() {
    let (ok, stdout, _) = run_wrapper(&[]);
    assert!(ok);
    assert!(stdout.contains("inner_start"));
    assert!(stdout.contains("inner_busy"));
    assert!(stdout.contains("inner_done"));
    assert!(stdout.contains("inner_cycle_count"));
}

#[test]
fn wrapper_error_counter_saturating() {
    let (ok, stdout, _) = run_wrapper(&[]);
    assert!(ok);
    assert!(stdout.contains("32'hFFFFFFFF"));
}

#[test]
fn wrapper_help_lists_flags() {
    let (ok, stdout, _) = run(&["gen-tt-debug-wrapper", "--help"]);
    assert!(ok);
    for flag in ["--manifest", "--inner", "--output"] {
        assert!(stdout.contains(flag), "missing {}", flag);
    }
}

#[test]
fn wrapper_help_mentions_wave_50() {
    let (ok, stdout, _) = run(&["gen-tt-debug-wrapper", "--help"]);
    assert!(ok);
    assert!(stdout.contains("Wave 50") || stdout.contains("R-TT-3"), "stdout={}", stdout);
}

#[test]
fn wrapper_invalid_inner_uses_default() {
    let (ok, stdout, _) = run_wrapper(&["--inner", "9bad"]);
    assert!(ok);
    assert!(stdout.contains("module bitnet_engine_top_debug_wrapper"), "invalid inner should use default");
}

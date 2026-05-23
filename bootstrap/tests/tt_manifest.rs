// ============================================================================
// tt_manifest.rs -- integration tests (Wave 42, R-TT-1, Closes #792)
//
// Exercises the `t27c tt-manifest` CLI through `CARGO_BIN_EXE_t27c`.  Each
// test covers a single behavioural facet (stdout vs --output, chip variants,
// determinism, error paths, env override).  Tests avoid the `tempfile`
// crate (not in Cargo.toml) and use `std::env::temp_dir()` instead.
// ============================================================================

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn tmp_path(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "t27_tt_manifest_{}_{}.json",
        label,
        std::process::id()
    ))
}

#[test]
fn tt_manifest_phi_stdout_contains_chip_field() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "abc", "--build-time", "2026-05-23T19:42:00Z"])
        .output()
        .expect("run t27c");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"chip\": \"phi\""), "stdout: {}", s);
}

#[test]
fn tt_manifest_euler_stdout_contains_chip_field() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "euler", "--commit", "abc", "--build-time", "2026-05-23T19:42:00Z"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"chip\": \"euler\""));
}

#[test]
fn tt_manifest_gamma_stdout_contains_chip_field() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "gamma", "--commit", "abc", "--build-time", "2026-05-23T19:42:00Z"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"chip\": \"gamma\""));
}

#[test]
fn tt_manifest_unknown_chip_fails() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "delta"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("--chip parse error"), "stderr: {}", err);
}

#[test]
fn tt_manifest_uppercase_chip_accepted() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "PHI", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"chip\": \"phi\""));
}

#[test]
fn tt_manifest_output_dash_goes_to_stdout() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--output", "-", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!out.stdout.is_empty());
}

#[test]
fn tt_manifest_output_writes_file() {
    let p = tmp_path("write_file");
    let _ = std::fs::remove_file(&p);
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z", "--output", p.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let content = std::fs::read_to_string(&p).unwrap();
    assert!(content.contains("\"chip\": \"phi\""));
    std::fs::remove_file(&p).unwrap();
}

#[test]
fn tt_manifest_output_file_stderr_ok_line() {
    let p = tmp_path("stderr_ok");
    let _ = std::fs::remove_file(&p);
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "euler", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z", "--output", p.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("OK tt-manifest chip=euler"), "stderr: {}", err);
    std::fs::remove_file(&p).ok();
}

#[test]
fn tt_manifest_deterministic_same_inputs_same_bytes() {
    let a = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "abc", "--build-time", "2026-05-23T19:42:00Z", "--sva-count", "7"])
        .output().unwrap();
    let b = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "abc", "--build-time", "2026-05-23T19:42:00Z", "--sva-count", "7"])
        .output().unwrap();
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn tt_manifest_different_chip_different_output() {
    let a = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z"])
        .output().unwrap();
    let b = Command::new(bin())
        .args(["tt-manifest", "--chip", "euler", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z"])
        .output().unwrap();
    assert_ne!(a.stdout, b.stdout);
}

#[test]
fn tt_manifest_different_commit_different_output() {
    let a = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "aaa", "--build-time", "2026-05-23T19:42:00Z"])
        .output().unwrap();
    let b = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "bbb", "--build-time", "2026-05-23T19:42:00Z"])
        .output().unwrap();
    assert_ne!(a.stdout, b.stdout);
}

#[test]
fn tt_manifest_contains_all_nine_modules() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z"])
        .output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    for m in &[
        "weight_bram", "pipeline_stage2_compute", "layer_sequencer",
        "double_buffer_ctrl", "weight_prefetch_ctrl",
        "bitnet_axi_slave", "bitnet_dma", "bitnet_irq", "bitnet_engine_top",
    ] {
        assert!(s.contains(m), "module {} not found in: {}", m, s);
    }
}

#[test]
fn tt_manifest_axi_widths_present() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z"])
        .output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"addr\": 32"));
    assert!(s.contains("\"data\": 32"));
    assert!(s.contains("\"csr_aperture_bytes\": 64"));
}

#[test]
fn tt_manifest_phi_invariant_hash_present() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z"])
        .output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"phi_invariant_hash\": \"218403e344779c890f302ad2c70af21fb765060dd794d793c7eacc1ef8f80e6b\""));
}

#[test]
fn tt_manifest_sva_count_overridable() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z", "--sva-count", "42"])
        .output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"sva_count\": 42"));
}

#[test]
fn tt_manifest_sva_count_default_zero() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z"])
        .output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"sva_count\": 0"));
}

#[test]
fn tt_manifest_default_commit_env_or_unknown() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--build-time", "2026-05-23T19:42:00Z"])
        .env_remove("T27_COMMIT")
        .output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"t27_commit\": \"unknown\""), "stdout: {}", s);
}

#[test]
fn tt_manifest_env_t27_commit_used() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--build-time", "2026-05-23T19:42:00Z"])
        .env("T27_COMMIT", "from_env_abc")
        .output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"t27_commit\": \"from_env_abc\""), "stdout: {}", s);
}

#[test]
fn tt_manifest_commit_flag_overrides_env() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "flag_wins", "--build-time", "2026-05-23T19:42:00Z"])
        .env("T27_COMMIT", "env_loses")
        .output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"t27_commit\": \"flag_wins\""));
    assert!(!s.contains("env_loses"));
}

#[test]
fn tt_manifest_default_build_time_is_rfc3339_z() {
    // Just check the output contains a Z-suffixed RFC3339-like timestamp
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "x"])
        .output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    // Should contain build_time_utc:"...Z"
    assert!(s.contains("\"build_time_utc\""));
    assert!(s.contains("Z\""));
}

#[test]
fn tt_manifest_roundtrip_via_serde_json() {
    let p = tmp_path("roundtrip");
    let _ = std::fs::remove_file(&p);
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "gamma", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z", "--sva-count", "11", "--output", p.to_str().unwrap()])
        .output().unwrap();
    assert!(out.status.success());
    let content = std::fs::read_to_string(&p).unwrap();
    // Re-parse using serde_json directly via a public type would need importing
    // the crate; instead we sanity-check structural fields present.
    assert!(content.starts_with("{"));
    assert!(content.trim_end().ends_with("}"));
    assert!(content.contains("\"chip\": \"gamma\""));
    assert!(content.contains("\"sva_count\": 11"));
    std::fs::remove_file(&p).ok();
}

#[test]
fn tt_manifest_three_chip_files_pairwise_distinct() {
    let mut paths = Vec::new();
    for chip in &["phi", "euler", "gamma"] {
        let p = tmp_path(&format!("triple_{}", chip));
        let _ = std::fs::remove_file(&p);
        let out = Command::new(bin())
            .args(["tt-manifest", "--chip", chip, "--commit", "deadbeef", "--build-time", "2026-05-23T19:42:00Z", "--output", p.to_str().unwrap()])
            .output().unwrap();
        assert!(out.status.success());
        paths.push(p);
    }
    let phi = std::fs::read_to_string(&paths[0]).unwrap();
    let euler = std::fs::read_to_string(&paths[1]).unwrap();
    let gamma = std::fs::read_to_string(&paths[2]).unwrap();
    assert_ne!(phi, euler);
    assert_ne!(euler, gamma);
    assert_ne!(phi, gamma);
    for p in &paths { std::fs::remove_file(p).ok(); }
}

#[test]
fn tt_manifest_pretty_printed_multiline() {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "x", "--build-time", "2026-05-23T19:42:00Z"])
        .output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.lines().count() >= 10, "expected pretty JSON >=10 lines, got: {}", s.lines().count());
}

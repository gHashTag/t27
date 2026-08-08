// ============================================================================
// tt_profile.rs -- integration tests (Wave 45, R-TT-2, Closes #800)
//
// Exercises `t27c tt-profile` and `t27c tt-conform` through CARGO_BIN_EXE_t27c.
// Each test covers a single behavioural facet.  No `tempfile` crate.
// ============================================================================

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn tmp(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("t27_tt_profile_{}_{}.json", label, std::process::id()))
}

fn emit_profile(platform: &str, path: &std::path::Path) {
    let out = Command::new(bin())
        .args(["tt-profile", "--platform", platform, "--output", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "tt-profile {} failed: {}", platform, String::from_utf8_lossy(&out.stderr));
}

fn emit_manifest_default(path: &std::path::Path) {
    let out = Command::new(bin())
        .args(["tt-manifest", "--chip", "phi", "--commit", "abc", "--build-time", "2026-05-23T20:00:00Z", "--output", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
}

// ---- tt-profile -----------------------------------------------------------

#[test]
fn tt_profile_sky130_stdout() {
    let out = Command::new(bin()).args(["tt-profile", "--platform", "sky130"]).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"platform\": \"sky130\""));
    assert!(s.contains("\"process_node_nm\": 130"));
}

#[test]
fn tt_profile_ihp_stdout() {
    let out = Command::new(bin()).args(["tt-profile", "--platform", "ihp"]).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"platform\": \"ihp_sg13g2\""));
}

#[test]
fn tt_profile_gf180_stdout() {
    let out = Command::new(bin()).args(["tt-profile", "--platform", "gf180"]).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"platform\": \"gf180mcu\""));
    assert!(s.contains("\"process_node_nm\": 180"));
}

#[test]
fn tt_profile_unknown_platform_fails() {
    let out = Command::new(bin()).args(["tt-profile", "--platform", "tsmc7"]).output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("--platform parse error"), "stderr: {}", err);
}

#[test]
fn tt_profile_output_dash_to_stdout() {
    let out = Command::new(bin()).args(["tt-profile", "--platform", "sky130", "--output", "-"]).output().unwrap();
    assert!(out.status.success());
    assert!(!out.stdout.is_empty());
}

#[test]
fn tt_profile_writes_file() {
    let p = tmp("write_sky");
    let _ = std::fs::remove_file(&p);
    emit_profile("sky130", &p);
    let body = std::fs::read_to_string(&p).unwrap();
    assert!(body.contains("\"platform\": \"sky130\""));
    std::fs::remove_file(&p).ok();
}

#[test]
fn tt_profile_deterministic() {
    let a = Command::new(bin()).args(["tt-profile", "--platform", "sky130"]).output().unwrap();
    let b = Command::new(bin()).args(["tt-profile", "--platform", "sky130"]).output().unwrap();
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn tt_profile_three_platforms_distinct() {
    let s = Command::new(bin()).args(["tt-profile", "--platform", "sky130"]).output().unwrap().stdout;
    let i = Command::new(bin()).args(["tt-profile", "--platform", "ihp"]).output().unwrap().stdout;
    let g = Command::new(bin()).args(["tt-profile", "--platform", "gf180"]).output().unwrap().stdout;
    assert_ne!(s, i);
    assert_ne!(i, g);
    assert_ne!(s, g);
}

#[test]
fn tt_profile_uppercase_platform_accepted() {
    let out = Command::new(bin()).args(["tt-profile", "--platform", "SKY130"]).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"platform\": \"sky130\""));
}

#[test]
fn tt_profile_sky_voltage_1800() {
    let out = Command::new(bin()).args(["tt-profile", "--platform", "sky130"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"supply_voltage_mvolts\": 1800"));
}

#[test]
fn tt_profile_gf_voltage_5000() {
    let out = Command::new(bin()).args(["tt-profile", "--platform", "gf180"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"supply_voltage_mvolts\": 5000"));
}

#[test]
fn tt_profile_cell_library_sky() {
    let out = Command::new(bin()).args(["tt-profile", "--platform", "sky130"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("sky130_fd_sc_hd"));
}

// ---- tt-conform -----------------------------------------------------------

#[test]
fn tt_conform_ok_on_canonical_sky() {
    let prof = tmp("conform_sky");
    let mani = tmp("conform_mani");
    emit_profile("sky130", &prof);
    emit_manifest_default(&mani);
    let out = Command::new(bin())
        .args(["tt-conform", "--profile", prof.to_str().unwrap(), "--manifest", mani.to_str().unwrap()])
        .output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("OK conform=true reasons=0"));
    std::fs::remove_file(&prof).ok();
    std::fs::remove_file(&mani).ok();
}

#[test]
fn tt_conform_ok_on_canonical_ihp() {
    let prof = tmp("conform_ihp");
    let mani = tmp("conform_mani_ihp");
    emit_profile("ihp", &prof);
    emit_manifest_default(&mani);
    let out = Command::new(bin())
        .args(["tt-conform", "--profile", prof.to_str().unwrap(), "--manifest", mani.to_str().unwrap()])
        .output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("conform=true"));
    std::fs::remove_file(&prof).ok();
    std::fs::remove_file(&mani).ok();
}

#[test]
fn tt_conform_ok_on_canonical_gf180() {
    let prof = tmp("conform_gf");
    let mani = tmp("conform_mani_gf");
    emit_profile("gf180", &prof);
    emit_manifest_default(&mani);
    let out = Command::new(bin())
        .args(["tt-conform", "--profile", prof.to_str().unwrap(), "--manifest", mani.to_str().unwrap()])
        .output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("conform=true"));
    std::fs::remove_file(&prof).ok();
    std::fs::remove_file(&mani).ok();
}

#[test]
fn tt_conform_fails_when_axi_widths_mismatch() {
    let prof = tmp("conform_bad_prof");
    let mani = tmp("conform_bad_mani");
    emit_profile("sky130", &prof);
    // Build a manifest with bad axi data width by hand-editing JSON
    emit_manifest_default(&mani);
    let mut text = std::fs::read_to_string(&mani).unwrap();
    text = text.replace("\"data\": 32,", "\"data\": 64,");
    std::fs::write(&mani, &text).unwrap();

    let out = Command::new(bin())
        .args(["tt-conform", "--profile", prof.to_str().unwrap(), "--manifest", mani.to_str().unwrap()])
        .output().unwrap();
    assert!(!out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("conform=false"), "stdout: {}", s);
    let e = String::from_utf8_lossy(&out.stderr);
    assert!(e.contains("axi data width"), "stderr: {}", e);
    std::fs::remove_file(&prof).ok();
    std::fs::remove_file(&mani).ok();
}

#[test]
fn tt_conform_fails_when_too_many_modules() {
    let prof = tmp("conform_too_many_p");
    let mani = tmp("conform_too_many_m");
    emit_profile("gf180", &prof);
    emit_manifest_default(&mani);
    // Surgically inflate the modules array from 9 to 11 by JSON-string replace
    let text = std::fs::read_to_string(&mani).unwrap();
    let inflated = text.replace(
        "\"bitnet_engine_top\"\n  ],",
        "\"bitnet_engine_top\",\n    \"extra_one\",\n    \"extra_two\"\n  ],",
    );
    assert_ne!(text, inflated, "module list inflation didn't take effect");
    std::fs::write(&mani, &inflated).unwrap();

    let out = Command::new(bin())
        .args(["tt-conform", "--profile", prof.to_str().unwrap(), "--manifest", mani.to_str().unwrap()])
        .output().unwrap();
    assert!(!out.status.success());
    let e = String::from_utf8_lossy(&out.stderr);
    assert!(e.contains("module count"), "stderr: {}", e);
    std::fs::remove_file(&prof).ok();
    std::fs::remove_file(&mani).ok();
}

#[test]
fn tt_conform_verbose_emits_verdict_json() {
    let prof = tmp("conform_verbose_p");
    let mani = tmp("conform_verbose_m");
    emit_profile("sky130", &prof);
    emit_manifest_default(&mani);
    let out = Command::new(bin())
        .args(["tt-conform", "--profile", prof.to_str().unwrap(), "--manifest", mani.to_str().unwrap(), "--verbose"])
        .output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"ok\": true"), "stdout: {}", s);
    assert!(s.contains("\"reasons\": ["));
    std::fs::remove_file(&prof).ok();
    std::fs::remove_file(&mani).ok();
}

#[test]
fn tt_conform_missing_profile_file_fails() {
    let mani = tmp("conform_nomiss_m");
    emit_manifest_default(&mani);
    let out = Command::new(bin())
        .args(["tt-conform", "--profile", "/tmp/this_does_not_exist_t27_xyz.json", "--manifest", mani.to_str().unwrap()])
        .output().unwrap();
    assert!(!out.status.success());
    std::fs::remove_file(&mani).ok();
}

#[test]
fn tt_conform_invalid_profile_json_fails() {
    let prof = tmp("conform_badjson_p");
    let mani = tmp("conform_badjson_m");
    std::fs::write(&prof, "{not valid json}").unwrap();
    emit_manifest_default(&mani);
    let out = Command::new(bin())
        .args(["tt-conform", "--profile", prof.to_str().unwrap(), "--manifest", mani.to_str().unwrap()])
        .output().unwrap();
    assert!(!out.status.success());
    let e = String::from_utf8_lossy(&out.stderr);
    assert!(e.contains("profile parse error") || e.contains("expected"), "stderr: {}", e);
    std::fs::remove_file(&prof).ok();
    std::fs::remove_file(&mani).ok();
}

#[test]
fn tt_conform_three_platforms_canonical_all_ok() {
    let mani = tmp("conform_three_m");
    emit_manifest_default(&mani);
    for plat in &["sky130", "ihp", "gf180"] {
        let prof = tmp(&format!("conform_three_{}", plat));
        emit_profile(plat, &prof);
        let out = Command::new(bin())
            .args(["tt-conform", "--profile", prof.to_str().unwrap(), "--manifest", mani.to_str().unwrap()])
            .output().unwrap();
        assert!(out.status.success(), "{}: stderr: {}", plat, String::from_utf8_lossy(&out.stderr));
        std::fs::remove_file(&prof).ok();
    }
    std::fs::remove_file(&mani).ok();
}

#[test]
fn tt_profile_max_modules_sky_12() {
    let out = Command::new(bin()).args(["tt-profile", "--platform", "sky130"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"max_modules\": 12"));
}

#[test]
fn tt_profile_max_modules_gf_9() {
    let out = Command::new(bin()).args(["tt-profile", "--platform", "gf180"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"max_modules\": 9"));
}

#[test]
fn tt_profile_pretty_printed_multiline() {
    let out = Command::new(bin()).args(["tt-profile", "--platform", "sky130"]).output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.lines().count() >= 6);
}

#[test]
fn tt_conform_canonical_phi_euler_gamma_all_ok_on_sky() {
    let prof = tmp("triple_phi_p");
    emit_profile("sky130", &prof);
    for chip in &["phi", "euler", "gamma"] {
        let mani = tmp(&format!("triple_chip_{}", chip));
        let out = Command::new(bin())
            .args(["tt-manifest", "--chip", chip, "--commit", "abc", "--build-time", "2026-05-23T20:00:00Z", "--output", mani.to_str().unwrap()])
            .output().unwrap();
        assert!(out.status.success());
        let c = Command::new(bin())
            .args(["tt-conform", "--profile", prof.to_str().unwrap(), "--manifest", mani.to_str().unwrap()])
            .output().unwrap();
        assert!(c.status.success(), "{}: stderr: {}", chip, String::from_utf8_lossy(&c.stderr));
        std::fs::remove_file(&mani).ok();
    }
    std::fs::remove_file(&prof).ok();
}

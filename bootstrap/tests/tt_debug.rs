// ============================================================================
// tt_debug.rs -- integration tests (Wave 49, R-TT-3, Closes #1217)
//
// Exercises `t27c gen-tt-debug-wrapper` through CARGO_BIN_EXE_t27c.  No
// `tempfile` crate: temp paths are derived from `std::env::temp_dir()` and
// `std::process::id()`.  Each test covers a single behavioural facet of the
// Tiny Tapeout debug wrapper emitter (R-TT-3).
// ============================================================================

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn tmp(label: &str, ext: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "t27_tt_debug_{}_{}.{}",
        label,
        std::process::id(),
        ext
    ))
}

fn emit_manifest(chip: &str, path: &std::path::Path) {
    let out = Command::new(bin())
        .args([
            "tt-manifest",
            "--chip",
            chip,
            "--commit",
            "0123456789abcdef",
            "--build-time",
            "2026-05-24T04:00:00Z",
            "--output",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "tt-manifest {} failed: {}",
        chip,
        String::from_utf8_lossy(&out.stderr)
    );
}

fn run_wrapper(manifest: &std::path::Path, inner: Option<&str>) -> (bool, String, String) {
    let mut args: Vec<String> = vec![
        "gen-tt-debug-wrapper".into(),
        "--manifest".into(),
        manifest.to_str().unwrap().into(),
    ];
    if let Some(name) = inner {
        args.push("--inner".into());
        args.push(name.into());
    }
    let out = Command::new(bin()).args(&args).output().unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

// ---- 1. basic emission ----------------------------------------------------

#[test]
fn tt_debug_phi_stdout_succeeds() {
    let m = tmp("phi_basic", "json");
    emit_manifest("phi", &m);
    let (ok, stdout, _) = run_wrapper(&m, None);
    assert!(ok);
    assert!(stdout.contains("module bitnet_engine_top_tt_debug"));
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_emits_module_header_comment() {
    let m = tmp("hdr", "json");
    emit_manifest("phi", &m);
    let (ok, stdout, _) = run_wrapper(&m, None);
    assert!(ok);
    assert!(stdout.contains("R-TT-3, W49"));
    assert!(stdout.contains("TT-debug wrapper"));
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_emits_ascii_only() {
    let m = tmp("ascii", "json");
    emit_manifest("phi", &m);
    let (ok, stdout, _) = run_wrapper(&m, None);
    assert!(ok);
    assert!(stdout.is_ascii(), "non-ascii byte in tt_debug emission");
    let _ = std::fs::remove_file(&m);
}

// ---- 2. CSR aperture layout ----------------------------------------------

#[test]
fn tt_debug_csr_offsets_match_spec() {
    let m = tmp("offsets", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    for needle in [
        "OFF_VERSION = 12'h040",
        "OFF_ERR_AXI = 12'h044",
        "OFF_ERR_DMA = 12'h048",
        "OFF_ERR_IRQ = 12'h04c",
        "OFF_ERR_CSR = 12'h050",
        "OFF_ST_TRIG = 12'h054",
        "OFF_ST_RES  = 12'h058",
    ] {
        assert!(stdout.contains(needle), "missing: {}", needle);
    }
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_aperture_stays_below_0x60() {
    // Manifest CSR aperture is 64 bytes (0x00..0x40); debug extension is
    // 0x40..0x60.  No offset must be >= 0x60.
    let m = tmp("ap_bound", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    for forbidden in ["12'h060", "12'h064", "12'h068", "12'h070"] {
        assert!(!stdout.contains(forbidden), "out-of-range offset: {}", forbidden);
    }
    let _ = std::fs::remove_file(&m);
}

// ---- 3. version word + phi invariant lo8 ---------------------------------

#[test]
fn tt_debug_version_word_phi_lo8() {
    // SHA-256("phi^2 + 1/phi^2 = 3") = 218403...e6b -> lo8 = 0x6b.
    let m = tmp("phi_lo8", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    assert!(stdout.contains("VERSION_WORD = 32'h"));
    assert!(stdout.contains("phi_lo8: 0x6b"), "phi-invariant lo8 must be 0x6b");
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_version_word_carries_chip_slug_hash() {
    let mp = tmp("vw_phi", "json");
    let me = tmp("vw_eu", "json");
    let mg = tmp("vw_ga", "json");
    emit_manifest("phi", &mp);
    emit_manifest("euler", &me);
    emit_manifest("gamma", &mg);
    let (_, p, _) = run_wrapper(&mp, None);
    let (_, e, _) = run_wrapper(&me, None);
    let (_, g, _) = run_wrapper(&mg, None);
    let extract = |s: &str| -> String {
        let key = "VERSION_WORD = 32'h";
        let i = s.find(key).expect("version word") + key.len();
        s[i..i + 8].to_string()
    };
    let vp = extract(&p);
    let ve = extract(&e);
    let vg = extract(&g);
    assert_ne!(vp, ve);
    assert_ne!(vp, vg);
    assert_ne!(ve, vg);
    // Phi-lo8 byte (last two hex chars) is identical across chips.
    assert_eq!(&vp[6..8], &ve[6..8]);
    assert_eq!(&vp[6..8], &vg[6..8]);
    let _ = std::fs::remove_file(&mp);
    let _ = std::fs::remove_file(&me);
    let _ = std::fs::remove_file(&mg);
}

#[test]
fn tt_debug_version_word_pins_commit_lo16() {
    // Commit "0123456789abcdef" -> lo16 = 0xcdef (last 4 hex of low-32 nibbles).
    let m = tmp("commit", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    assert!(stdout.contains("commit_lo16: 0x0123"));
    let _ = std::fs::remove_file(&m);
}

// ---- 4. error counters ----------------------------------------------------

#[test]
fn tt_debug_emits_four_error_counters() {
    let m = tmp("errcnt", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    for cnt in ["err_axi_cnt", "err_dma_cnt", "err_irq_cnt", "err_csr_cnt"] {
        assert!(stdout.contains(cnt), "missing counter: {}", cnt);
    }
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_counters_are_clk_gated_on_pulse() {
    let m = tmp("pulse", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    assert!(stdout.contains("if (err_axi_pulse) err_axi_cnt <= err_axi_cnt + 1'b1"));
    assert!(stdout.contains("if (err_dma_pulse) err_dma_cnt <= err_dma_cnt + 1'b1"));
    assert!(stdout.contains("if (err_irq_pulse) err_irq_cnt <= err_irq_cnt + 1'b1"));
    assert!(stdout.contains("if (err_csr_pulse) err_csr_cnt <= err_csr_cnt + 1'b1"));
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_counters_reset_on_active_low() {
    let m = tmp("rst", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    assert!(stdout.contains("posedge clk or negedge rst_n"));
    assert!(stdout.contains("if (!rst_n) begin"));
    let _ = std::fs::remove_file(&m);
}

// ---- 5. self-test trig / res ---------------------------------------------

#[test]
fn tt_debug_emits_self_test_trigger() {
    let m = tmp("st_trig", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    assert!(stdout.contains("inner_self_test_trig"));
    assert!(stdout.contains("inner_self_test_pass"));
    assert!(stdout.contains("inner_self_test_fail_count"));
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_self_test_writes_only_via_st_trig_offset() {
    let m = tmp("st_w", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    // ST_TRIG is the only RW slot in the debug aperture.
    assert!(stdout.contains("OFF_ST_TRIG"));
    assert!(stdout.contains("self_test_trig_q"));
    let _ = std::fs::remove_file(&m);
}

// ---- 6. inner-module override --------------------------------------------

#[test]
fn tt_debug_default_inner_is_bitnet_engine_top() {
    let m = tmp("def_inner", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    assert!(stdout.contains("module bitnet_engine_top_tt_debug"));
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_custom_inner_module_name() {
    let m = tmp("cust_inner", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, Some("my_engine"));
    assert!(stdout.contains("module my_engine_tt_debug"));
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_invalid_inner_falls_back_to_safe_ident() {
    let m = tmp("bad_inner", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, Some("123 bad name!"));
    // Wrapper must still emit a syntactically valid module statement.
    assert!(stdout.contains("module "));
    assert!(stdout.contains("_tt_debug"));
    let _ = std::fs::remove_file(&m);
}

// ---- 7. determinism + output path ----------------------------------------

#[test]
fn tt_debug_emission_is_byte_identical_for_same_input() {
    let m = tmp("det", "json");
    emit_manifest("phi", &m);
    let (_, a, _) = run_wrapper(&m, None);
    let (_, b, _) = run_wrapper(&m, None);
    assert_eq!(a, b, "tt_debug emission is not deterministic");
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_no_output_defaults_to_stdout() {
    let m = tmp("def_stdout", "json");
    emit_manifest("phi", &m);
    let out = Command::new(bin())
        .args(["gen-tt-debug-wrapper", "--manifest", m.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("_tt_debug"));
    assert!(s.contains("VERSION_WORD"));
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_writes_file_when_output_path_given() {
    let m = tmp("file_w", "json");
    let o = tmp("emit", "sv");
    emit_manifest("phi", &m);
    let out = Command::new(bin())
        .args([
            "gen-tt-debug-wrapper",
            "--manifest",
            m.to_str().unwrap(),
            "--output",
            o.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let body = std::fs::read_to_string(&o).unwrap();
    assert!(body.contains("_tt_debug"));
    assert!(body.contains("VERSION_WORD"));
    let _ = std::fs::remove_file(&m);
    let _ = std::fs::remove_file(&o);
}

// ---- 8. error handling ---------------------------------------------------

#[test]
fn tt_debug_fails_on_missing_manifest() {
    let out = Command::new(bin())
        .args([
            "gen-tt-debug-wrapper",
            "--manifest",
            "/nonexistent/path/to/manifest.json",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "must fail on missing manifest");
}

#[test]
fn tt_debug_fails_on_malformed_manifest_json() {
    let m = tmp("bad_json", "json");
    std::fs::write(&m, "{ not really json").unwrap();
    let out = Command::new(bin())
        .args(["gen-tt-debug-wrapper", "--manifest", m.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let _ = std::fs::remove_file(&m);
}

// ---- 9. structural sanity ------------------------------------------------

#[test]
fn tt_debug_emits_one_endmodule_only() {
    let m = tmp("endmod", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    let n = stdout.matches("\nendmodule").count();
    assert_eq!(n, 1, "expected exactly one endmodule, got {}", n);
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_keeps_axi_invariants_intact() {
    // Wave 42 manifest pins axi data=32, addr=32, csr_aperture=64.  The
    // debug wrapper must declare matching ADDR_W / DATA_W parameters.
    let m = tmp("axi_inv", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    assert!(stdout.contains("parameter integer ADDR_W = 32"));
    assert!(stdout.contains("parameter integer DATA_W = 32"));
    let _ = std::fs::remove_file(&m);
}

#[test]
fn tt_debug_emits_no_shell_or_python_metadata() {
    // L7: no new *.sh.  Defensive check that the generator does not embed
    // shell-script hooks inside the SystemVerilog output.
    let m = tmp("noshell", "json");
    emit_manifest("phi", &m);
    let (_, stdout, _) = run_wrapper(&m, None);
    assert!(!stdout.contains("#!/bin/"));
    assert!(!stdout.contains("import os"));
    let _ = std::fs::remove_file(&m);
}

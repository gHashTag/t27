// ============================================================================
// Wave 38 (R-SI-1) integration tests for `t27c gen-bitnet-bundle`.
//
// All tests invoke the binary via CARGO_BIN_EXE_t27c, write into a unique
// directory under std::env::temp_dir() (no tempfile crate -- not on the
// Cargo.toml), and clean up after themselves.
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const EXPECTED_FILES: &[&str] = &[
    "weight_bram.sv",
    "pipeline_stage2_compute.sv",
    "layer_sequencer.sv",
    "double_buffer_ctrl.sv",
    "weight_prefetch_ctrl.sv",
    "axi_lite_slave.sv",
    "dma_controller.sv",
    "interrupt_controller.sv",
    "bitnet_engine_top.sv",
    "behavior_sva_v2.sv",
    "manifest.txt",
];

/// Unique scratch directory under temp_dir(), tagged by PID + label so
/// parallel test runs don't collide.
fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!(
        "t27_bitnet_bundle_{}_{}",
        std::process::id(),
        label
    ));
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
    dir
}

fn cleanup(dir: &PathBuf) {
    let _ = fs::remove_dir_all(dir);
}

fn run_bundle(args: &[&str]) -> (String, String, bool) {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let out = Command::new(bin)
        .args(args)
        .output()
        .expect("failed to invoke t27c");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    )
}

// ----------------------------------------------------------------------------
// Smoke + directory layout
// ----------------------------------------------------------------------------

#[test]
fn bundle_creates_output_directory_when_missing() {
    let dir = scratch_dir("creates_dir");
    assert!(!dir.exists());
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    assert!(dir.is_dir());
    cleanup(&dir);
}

#[test]
fn bundle_writes_every_canonical_file() {
    let dir = scratch_dir("eleven_files");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let count = fs::read_dir(&dir).unwrap().count();
    assert_eq!(count, 13);
    cleanup(&dir);
}

#[test]
fn bundle_writes_all_canonical_filenames() {
    let dir = scratch_dir("canonical_names");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    for fname in EXPECTED_FILES {
        let path = dir.join(fname);
        assert!(path.is_file(), "missing {}", fname);
    }
    cleanup(&dir);
}

#[test]
fn bundle_all_files_nonempty() {
    let dir = scratch_dir("nonempty");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    for fname in EXPECTED_FILES {
        let path = dir.join(fname);
        let size = fs::metadata(&path).unwrap().len();
        assert!(size > 0, "{} is empty", fname);
    }
    cleanup(&dir);
}

#[test]
fn bundle_all_files_ascii() {
    let dir = scratch_dir("ascii");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    for fname in EXPECTED_FILES {
        let path = dir.join(fname);
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.is_ascii(), "{} contains non-ASCII", fname);
    }
    cleanup(&dir);
}

// ----------------------------------------------------------------------------
// Top-name override
// ----------------------------------------------------------------------------

#[test]
fn bundle_default_top_name_in_engine_top() {
    let dir = scratch_dir("default_top");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let top = fs::read_to_string(dir.join("bitnet_engine_top.sv")).unwrap();
    assert!(top.contains("bitnet_engine_top"));
    cleanup(&dir);
}

#[test]
fn bundle_custom_top_name_propagates() {
    let dir = scratch_dir("custom_top");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--top-name",
        "my_engine_v1",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let top = fs::read_to_string(dir.join("bitnet_engine_top.sv")).unwrap();
    assert!(top.contains("my_engine_v1"));
    cleanup(&dir);
}

// ----------------------------------------------------------------------------
// AXI width overrides
// ----------------------------------------------------------------------------

#[test]
fn bundle_default_axi_widths_in_manifest() {
    let dir = scratch_dir("default_axi");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let manifest = fs::read_to_string(dir.join("manifest.txt")).unwrap();
    assert!(manifest.contains("32/32 bits"));
    cleanup(&dir);
}

#[test]
fn bundle_custom_axi_widths_in_manifest() {
    let dir = scratch_dir("custom_axi");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--axi-addr-width",
        "64",
        "--axi-data-width",
        "128",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let manifest = fs::read_to_string(dir.join("manifest.txt")).unwrap();
    assert!(manifest.contains("64/128 bits"));
    cleanup(&dir);
}

// ----------------------------------------------------------------------------
// Manifest structure
// ----------------------------------------------------------------------------

#[test]
fn bundle_manifest_mentions_wave_38() {
    let dir = scratch_dir("manifest_wave");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let manifest = fs::read_to_string(dir.join("manifest.txt")).unwrap();
    assert!(manifest.contains("Wave 38"));
    assert!(manifest.contains("R-SI-1"));
    cleanup(&dir);
}

#[test]
fn bundle_manifest_mentions_trinity_invariant() {
    let dir = scratch_dir("manifest_phi");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let manifest = fs::read_to_string(dir.join("manifest.txt")).unwrap();
    assert!(manifest.contains("phi^2 + 1/phi^2 = 3"));
    cleanup(&dir);
}

#[test]
fn bundle_manifest_lists_all_ten_sv_files() {
    let dir = scratch_dir("manifest_lists");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let manifest = fs::read_to_string(dir.join("manifest.txt")).unwrap();
    for fname in &EXPECTED_FILES[..10] {
        assert!(manifest.contains(fname), "manifest missing {}", fname);
    }
    cleanup(&dir);
}

#[test]
fn bundle_manifest_byte_sizes_match_files() {
    let dir = scratch_dir("manifest_sizes");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let manifest = fs::read_to_string(dir.join("manifest.txt")).unwrap();
    for fname in &EXPECTED_FILES[..10] {
        let size = fs::metadata(dir.join(fname)).unwrap().len();
        let needle = format!("  {}  {}", fname, size);
        assert!(
            manifest.contains(&needle),
            "manifest does not record {} size {}",
            fname,
            size
        );
    }
    cleanup(&dir);
}

// ----------------------------------------------------------------------------
// Content cross-checks (composition correctness)
// ----------------------------------------------------------------------------

#[test]
fn bundle_engine_top_instantiates_subblocks() {
    let dir = scratch_dir("top_instances");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let top = fs::read_to_string(dir.join("bitnet_engine_top.sv")).unwrap();
    // The engine-top emitter (W36f) instantiates these two sub-modules.
    assert!(top.contains("multilayer_sequencer"));
    assert!(top.contains("double_buffer_ctrl"));
    cleanup(&dir);
}

#[test]
fn bundle_sva_file_contains_property_and_assert() {
    let dir = scratch_dir("sva_struct");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let sva = fs::read_to_string(dir.join("behavior_sva_v2.sv")).unwrap();
    assert!(sva.contains("property "));
    assert!(sva.contains("assert property"));
    cleanup(&dir);
}

#[test]
fn bundle_sva_includes_s_eventually() {
    let dir = scratch_dir("sva_liveness");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let sva = fs::read_to_string(dir.join("behavior_sva_v2.sv")).unwrap();
    // The canonical behavior set includes one liveness property.
    assert!(sva.contains("s_eventually"));
    cleanup(&dir);
}

#[test]
fn bundle_sva_includes_delay_consequent() {
    let dir = scratch_dir("sva_delay");
    let (_o, _e, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    let sva = fs::read_to_string(dir.join("behavior_sva_v2.sv")).unwrap();
    // The canonical behavior set includes ##1 delay consequents.
    assert!(sva.contains("|-> ##1"));
    cleanup(&dir);
}

// ----------------------------------------------------------------------------
// Determinism + idempotency
// ----------------------------------------------------------------------------

#[test]
fn bundle_two_runs_produce_identical_files() {
    let dir_a = scratch_dir("det_a");
    let dir_b = scratch_dir("det_b");
    let (_, _, ok_a) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir_a.to_str().unwrap(),
    ]);
    let (_, _, ok_b) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir_b.to_str().unwrap(),
    ]);
    assert!(ok_a && ok_b);
    for fname in EXPECTED_FILES {
        let a = fs::read_to_string(dir_a.join(fname)).unwrap();
        let b = fs::read_to_string(dir_b.join(fname)).unwrap();
        assert_eq!(a, b, "{} differs between runs", fname);
    }
    cleanup(&dir_a);
    cleanup(&dir_b);
}

#[test]
fn bundle_overwrites_existing_directory() {
    let dir = scratch_dir("overwrite");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("stale.txt"), b"old").unwrap();
    let (_, _, ok) = run_bundle(&[
        "gen-bitnet-bundle",
        "--output-dir",
        dir.to_str().unwrap(),
    ]);
    assert!(ok);
    // Bundle files all present.
    for fname in EXPECTED_FILES {
        assert!(dir.join(fname).is_file(), "missing {}", fname);
    }
    // The stale file is untouched (this is by-design: write_bundle does not
    // wipe the directory). Document the invariant by asserting it.
    assert!(dir.join("stale.txt").exists());
    cleanup(&dir);
}

// ----------------------------------------------------------------------------
// CLI error handling
// ----------------------------------------------------------------------------

#[test]
fn bundle_missing_output_dir_arg_errors() {
    let (_, stderr, ok) = run_bundle(&["gen-bitnet-bundle"]);
    assert!(!ok);
    let lower = stderr.to_ascii_lowercase();
    assert!(lower.contains("output-dir") || lower.contains("required"));
}

#[test]
fn bundle_help_lists_top_name_flag() {
    let (stdout, _e, ok) = run_bundle(&["gen-bitnet-bundle", "--help"]);
    assert!(ok);
    assert!(stdout.contains("--top-name"));
    assert!(stdout.contains("--axi-addr-width"));
    assert!(stdout.contains("--axi-data-width"));
}

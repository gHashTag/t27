// ============================================================================
// Integration test: the assembled BitNet engine bundle must ELABORATE under
// Icarus Verilog, not merely pass string-assertions on the emitted text.
//
// Motivation: the BitNet HLS modules were validated only by substring checks on
// the generated Verilog. Nothing elaborated the *assembled* engine, so a missing
// module, a port/width mismatch, or a syntax error would stay invisible until
// hardware -- the same structural blind spot that let the stale `bitnet_top`
// asserts (#1726) sit red unnoticed. This test establishes that missing
// independent instrument (#1730).
//
// It shells out to the built `t27c` via CARGO_BIN_EXE_t27c, writes into a unique
// directory under std::env::temp_dir() (no tempfile crate on Cargo.toml), and
// skips gracefully when `iverilog` is not on PATH.
// ============================================================================

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn t27c() -> &'static str {
    env!("CARGO_BIN_EXE_t27c")
}

fn scratch_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("t27_bitnet_elab_{}_{}", std::process::id(), label));
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
    dir
}

fn iverilog_available() -> bool {
    Command::new("iverilog")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn bitnet_engine_bundle_elaborates_under_iverilog() {
    if !iverilog_available() {
        eprintln!("SKIP: iverilog not on PATH; skipping BitNet engine elaboration test");
        return;
    }

    let dir = scratch_dir("elab");
    fs::create_dir_all(&dir).expect("create scratch dir");

    // 1. Emit the full HLS bundle (9 RTL modules + SVA + manifest).
    let bundle_ok = Command::new(t27c())
        .args(["gen-bitnet-bundle", "--output-dir", dir.to_str().unwrap()])
        .status()
        .expect("failed to invoke gen-bitnet-bundle")
        .success();
    assert!(bundle_ok, "gen-bitnet-bundle failed");

    // 2. The bundle is self-contained: it now includes `trit_stdlib.sv` (which
    //    defines the `trit27_dot_product` that `pipeline_stage2_compute`
    //    instantiates), so no separate emission is needed -- the bundle
    //    directory elaborates on its own.
    assert!(
        dir.join("trit_stdlib.sv").is_file(),
        "bundle must include trit_stdlib.sv to elaborate standalone"
    );

    // 3. Collect every RTL module: all .sv except the SVA property file, which
    //    is not iverilog-parseable (SystemVerilog assertions).
    let mut rtl: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("read scratch dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map_or(false, |x| x == "sv"))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map_or(false, |n| !n.starts_with("behavior_sva"))
        })
        .collect();
    rtl.sort();
    assert!(!rtl.is_empty(), "no RTL .sv emitted into bundle");

    // 4. Elaborate rooted at the engine top. `-t null` runs the full front-end
    //    (parse + elaborate) with no code-generation backend -- a pure
    //    structural check that catches missing modules and port mismatches.
    let out = Command::new("iverilog")
        .args(["-g2012", "-t", "null", "-s", "bitnet_engine_top"])
        .args(&rtl)
        .output()
        .expect("failed to invoke iverilog");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();

    let _ = fs::remove_dir_all(&dir);

    assert!(
        out.status.success(),
        "iverilog elaboration of bitnet_engine_top failed:\n{}",
        stderr
    );
    assert!(
        !stderr.to_lowercase().contains("error"),
        "iverilog reported errors during elaboration:\n{}",
        stderr
    );
}

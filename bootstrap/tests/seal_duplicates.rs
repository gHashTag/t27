//! A spec can own more than one seal file, and `--save` wrote only one of them.
//!
//! Measured over `.trinity/seals`: **1313 seals cover 728 distinct specs, and
//! 547 specs carry more than one seal file** -- 501 of those are the same module
//! under two names, `<Module>.json` beside `<dir>_<Module>.json`, left by a
//! naming scheme that changed. Refreshing 116 stale seals through
//! `t27c seal --save` reached 58 and stopped for exactly that reason, and the
//! remaining 58 had to be rewritten by hand.
//!
//! The duplicates are not deletable: `bootstrap/src/math_compare.rs` opens
//! `.trinity/seals/PellisFormulas.json` by its bare name, and that file is one
//! of a pair. So the tool maintains them.
//!
//! Second defect in the same place, and it only hides on macOS: the derived
//! name and the tracked file can differ ONLY IN CASE -- the tool computes
//! `ar_restraint.json` where `ar_Restraint.json` is tracked. A case-insensitive
//! filesystem resolves those to one file; the case-sensitive one CI runs on
//! would grow a second seal for the same spec, and neither would look wrong.
//! 4 of the store's seals are in that state.
//!
//! The child runs with its own working directory rather than
//! `std::env::set_current_dir`, which is per-PROCESS and would race every other
//! test in this binary under the default parallel runner.

use std::fs;
use std::process::Command;

const SPEC: &str = "module dup_probe {\n    fn add(a: i32, b: i32) -> i32 { return a + b; }\n}\n";

fn scratch(tag: &str) -> std::path::PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!("t27-sealdup-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("specs/probe")).expect("specs dir");
    fs::create_dir_all(d.join(".trinity/seals")).expect("seals dir");
    fs::write(d.join("specs/probe/dup.t27"), SPEC).expect("write spec");
    d
}

fn save(dir: &std::path::Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .current_dir(dir)
        .args(["seal", "--save", "specs/probe/dup.t27"])
        .output()
        .expect("run t27c");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn rust_hash(p: &std::path::Path) -> String {
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(p).expect("read seal"))
        .expect("parse seal");
    v["gen_hash_rust"].as_str().unwrap_or_default().to_string()
}

/// A stale duplicate must be refreshed, not left behind.
#[test]
fn save_updates_every_seal_naming_the_same_spec() {
    let d = scratch("dup");
    let poisoned = serde_json::json!({
        "module": "OldName",
        "spec_path": "specs/probe/dup.t27",
        "spec_hash": "sha256:0",
        "gen_hash_zig": "sha256:0",
        "gen_hash_verilog": "sha256:0",
        "gen_hash_c": "sha256:0",
        "gen_hash_rust": "sha256:0",
        "sealed_at": "2000-01-01T00:00:00Z",
        "sealed_by": "hand",
        "ring": 12
    });
    let dup = d.join(".trinity/seals/OldName.json");
    fs::write(&dup, serde_json::to_string_pretty(&poisoned).unwrap()).expect("write dup");

    let text = save(&d);
    assert!(text.contains("Seal saved to"), "no seal written:\n{text}");
    assert!(
        text.contains("1 other seal file"),
        "the duplicate must be reported, not touched in silence:\n{text}"
    );

    let canonical = d.join(".trinity/seals/probe_dup_probe.json");
    assert!(canonical.exists(), "canonical seal missing");
    assert_ne!(rust_hash(&dup), "sha256:0", "the duplicate was left stale");
    assert_eq!(
        rust_hash(&dup),
        rust_hash(&canonical),
        "the two seals for one spec disagree"
    );
    // `module` is what the file is NAMED after; overwriting it would leave a
    // seal whose name and contents disagree.
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&dup).unwrap()).unwrap();
    assert_eq!(v["module"], "OldName", "the duplicate's own module was overwritten");
    let _ = fs::remove_dir_all(&d);
}

/// True when two names differing only in case are two files here.
///
/// On a case-INSENSITIVE filesystem the assertion below cannot fail: writing the
/// derived name simply lands on the existing file, guard or no guard. Verified
/// by mutation -- making the lookup case-sensitive leaves this test green on
/// macOS. A test that cannot fail is not evidence, so it says so and skips
/// rather than reporting a pass it did not earn. CI runs on Linux, where it
/// does discriminate.
fn filesystem_is_case_sensitive() -> bool {
    let d = std::env::temp_dir().join(format!("t27-case-probe-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    if fs::create_dir_all(&d).is_err() {
        return false;
    }
    let _ = fs::write(d.join("Aa"), "");
    let _ = fs::write(d.join("aA"), "");
    let n = fs::read_dir(&d).map(|e| e.count()).unwrap_or(1);
    let _ = fs::remove_dir_all(&d);
    n == 2
}

/// An existing file whose name differs only in case must be written, not twinned.
#[test]
fn save_writes_into_an_existing_case_variant() {
    if !filesystem_is_case_sensitive() {
        eprintln!(
            "SKIP save_writes_into_an_existing_case_variant: this filesystem is \
case-insensitive, so the assertion cannot distinguish the guard from its absence"
        );
        return;
    }
    let d = scratch("case");
    // The tool derives `probe_dup_probe.json`; this is the same name in another
    // case, which is how ar_Restraint.json sits beside a derived ar_restraint.
    let variant = d.join(".trinity/seals/Probe_Dup_Probe.json");
    let seed = serde_json::json!({
        "module": "dup_probe",
        "spec_path": "specs/probe/dup.t27",
        "spec_hash": "sha256:0",
        "gen_hash_zig": "sha256:0",
        "gen_hash_verilog": "sha256:0",
        "gen_hash_c": "sha256:0",
        "gen_hash_rust": "sha256:0",
        "sealed_at": "2000-01-01T00:00:00Z",
        "sealed_by": "hand",
        "ring": 12
    });
    fs::write(&variant, serde_json::to_string_pretty(&seed).unwrap()).expect("write variant");

    save(&d);

    let n = fs::read_dir(d.join(".trinity/seals")).unwrap().count();
    assert_eq!(
        n, 1,
        "a case-variant twin was created; the store must hold one seal here"
    );
    assert_ne!(rust_hash(&variant), "sha256:0", "the existing file was not updated");
    let _ = fs::remove_dir_all(&d);
}

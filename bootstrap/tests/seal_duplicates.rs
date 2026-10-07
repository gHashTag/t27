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
    save_as(dir, "specs/probe/dup.t27").1
}

/// `t27c seal --save <typed>` run in `dir`: whether it succeeded, and its output.
fn save_as(dir: &std::path::Path, typed: &str) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .current_dir(dir)
        .args(["seal", "--save", typed])
        .output()
        .expect("run t27c");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
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

/// A seal for `specs/probe/dup.t27` under another module name, every hash poisoned.
fn poisoned_twin() -> serde_json::Value {
    serde_json::json!({
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
    })
}

/// #6913: the seal recorded `spec_path` as typed. #6789 sealed with absolute
/// paths and committed three seals naming the sealing agent's worktree, which
/// every checker found dangling; and the twin refresh, matching `spec_path` by
/// string, skipped the repo-relative twin in silence. Every spelling of the
/// same file must record the same path and reach the same twin.
#[test]
fn save_records_the_spec_path_relative_to_the_store() {
    let d = scratch("rel");
    // Not canonicalized on purpose: on macOS the temp dir is a symlink
    // (/var -> /private/var), which a plain prefix strip would not see through.
    let abs = d.join("specs/probe/dup.t27");
    let dup = d.join(".trinity/seals/OldName.json");
    for typed in [
        abs.to_str().expect("utf-8 temp path"),
        "./specs/probe/dup.t27",
        "specs/probe/../probe/dup.t27",
    ] {
        fs::write(&dup, serde_json::to_string_pretty(&poisoned_twin()).unwrap()).expect("write dup");
        let (ok, text) = save_as(&d, typed);
        assert!(ok && text.contains("Seal saved to"), "{typed}: no seal written:\n{text}");
        let canonical = d.join(".trinity/seals/probe_dup_probe.json");
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&canonical).expect("read seal")).unwrap();
        assert_eq!(
            v["spec_path"], "specs/probe/dup.t27",
            "{typed}: the seal recorded the path as typed"
        );
        assert_ne!(rust_hash(&dup), "sha256:0", "{typed}: the repo-relative twin was skipped");
    }
    let _ = fs::remove_dir_all(&d);
}

/// A spec outside the working directory has no path this store can record, so
/// the seal is refused rather than written under a path nothing here can open.
#[test]
fn save_refuses_a_spec_outside_the_store() {
    let d = scratch("in");
    let other = scratch("out");
    let outside = other.join("specs/probe/dup.t27");
    let (ok, text) = save_as(&d, outside.to_str().expect("utf-8 temp path"));
    assert!(!ok, "a spec outside the store was sealed:\n{text}");
    assert!(
        text.contains("outside the working directory"),
        "the refusal must say why:\n{text}"
    );
    let n = fs::read_dir(d.join(".trinity/seals")).unwrap().count();
    assert_eq!(n, 0, "a seal was written for a spec outside the store");
    let _ = fs::remove_dir_all(&d);
    let _ = fs::remove_dir_all(&other);
}

//! `t27c seal --save` must not mint a seal over the spec's own failing tests.
//!
//! #5578 resealed 336 specs with `seal --save`, which refused only when a
//! backend could not GENERATE and never ran a test. 13 of those specs had tests
//! that fail -- merge_sort 0/2, mse_loss 0/3, timing_tb 9/10 -- and the
//! seal-coverage gate, which compares hashes, has reported them as holding ever
//! since (#5577).
//!
//! Three outcomes, kept apart the way `t27c test-report` keeps them apart:
//!
//!     FAIL     refused, exit 1, the failing test named, no file written
//!     pass     saved, `tests.failed == 0` in the seal
//!     BLOCKED  saved with a notice -- blocked is not failing
//!
//! The FAIL and pass cases need `zig`; without it every spec is BLOCKED, so
//! those two say SKIP rather than report a pass they did not earn. The BLOCKED
//! case takes zig off PATH on purpose, so it runs everywhere.
//!
//! Each child gets its own working directory (`current_dir`), never
//! `std::env::set_current_dir`, which is per-process and would race the other
//! tests in this binary.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PASSING: &str = "module seal_probe;\n\
fn add(a: i32, b: i32) -> i32 {\n    return a + b;\n}\n\
test \"add_works\" {\n    assert add(1, 2) == 3;\n}\n";

const FAILING: &str = "module seal_probe;\n\
fn add(a: i32, b: i32) -> i32 {\n    return a + b;\n}\n\
test \"add_works\" {\n    assert add(1, 2) == 3;\n}\n\
test \"add_is_wrong_on_purpose\" {\n    assert add(1, 2) == 4;\n}\n";

fn zig_present() -> bool {
    Command::new("zig")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn scratch(tag: &str, spec: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!("t27-sealtests-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("specs/probe")).expect("specs dir");
    fs::create_dir_all(d.join(".trinity/seals")).expect("seals dir");
    // A distinct stem per case, so test-report's working directories differ too.
    fs::write(d.join(format!("specs/probe/{tag}.t27")), spec).expect("write spec");
    d
}

fn save(dir: &Path, tag: &str, path_override: Option<&Path>, force: bool) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_t27c"));
    c.current_dir(dir).args(["seal", "--save", &format!("specs/probe/{tag}.t27")]);
    if force {
        c.arg("--force");
    }
    if let Some(p) = path_override {
        c.env("PATH", p);
    }
    c.output().expect("run t27c")
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn seals(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir.join(".trinity/seals"))
        .expect("seals dir")
        .flatten()
        .map(|e| e.path())
        .collect()
}

fn read(p: &Path) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(p).expect("read seal")).expect("parse seal")
}

#[test]
fn a_failing_test_refuses_the_seal() {
    if !zig_present() {
        eprintln!("SKIP a_failing_test_refuses_the_seal: no zig, every spec is BLOCKED");
        return;
    }
    let d = scratch("failing", FAILING);
    let out = save(&d, "failing", None, false);
    let t = text(&out);
    assert!(!out.status.success(), "a failing test must make seal --save exit non-zero:\n{t}");
    assert!(t.contains("refusing to seal"), "the refusal must say so:\n{t}");
    assert!(
        t.contains("FAIL  add_is_wrong_on_purpose"),
        "the failing test must be named:\n{t}"
    );
    assert!(!t.contains("FAIL  add_works"), "a passing test was reported as failing:\n{t}");
    assert!(seals(&d).is_empty(), "a seal was written for a spec whose test fails");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn force_records_the_failure_in_the_seal() {
    if !zig_present() {
        eprintln!("SKIP force_records_the_failure_in_the_seal: no zig");
        return;
    }
    let d = scratch("forced", FAILING);
    let out = save(&d, "forced", None, true);
    let t = text(&out);
    assert!(out.status.success(), "--force must still seal:\n{t}");
    assert!(t.contains("FAILING (--force)"), "a forced seal must warn:\n{t}");
    let s = seals(&d);
    assert_eq!(s.len(), 1, "expected one seal");
    let v = read(&s[0]);
    assert_eq!(v["tests"]["failed"], 1, "the failure must be on the record: {v}");
    assert_eq!(v["tests"]["forced"], true, "{v}");
    assert_eq!(v["tests"]["failing"][0], "add_is_wrong_on_purpose", "{v}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn a_passing_spec_is_sealed() {
    if !zig_present() {
        eprintln!("SKIP a_passing_spec_is_sealed: no zig, every spec is BLOCKED");
        return;
    }
    let d = scratch("passing", PASSING);
    let out = save(&d, "passing", None, false);
    let t = text(&out);
    assert!(out.status.success(), "a passing spec must seal:\n{t}");
    assert!(t.contains("tests 1/1 pass"), "the measurement must be printed:\n{t}");
    let s = seals(&d);
    assert_eq!(s.len(), 1, "expected one seal");
    let v = read(&s[0]);
    assert_eq!(v["tests"]["total"], 1, "{v}");
    assert_eq!(v["tests"]["failed"], 0, "{v}");
    assert_eq!(v["tests"]["forced"], false, "{v}");
    let _ = fs::remove_dir_all(&d);
}

/// Blocked is not failing: with no zig nothing ran, so nothing failed -- even
/// for a spec whose test WOULD fail. The seal is saved and says why.
#[test]
fn a_blocked_spec_is_sealed_with_a_notice() {
    let d = scratch("blocked", FAILING);
    let empty = d.join("empty-path");
    fs::create_dir_all(&empty).expect("empty PATH dir");
    let out = save(&d, "blocked", Some(&empty), false);
    let t = text(&out);
    assert!(out.status.success(), "a BLOCKED spec must still seal:\n{t}");
    assert!(t.contains("tests BLOCKED, not run"), "the notice is missing:\n{t}");
    assert!(t.contains("zig not on PATH"), "the reason is missing:\n{t}");
    let s = seals(&d);
    assert_eq!(s.len(), 1, "expected one seal");
    let v = read(&s[0]);
    assert_eq!(v["tests"]["blocked"], "zig not on PATH", "{v}");
    assert!(v["tests"].get("failed").is_none(), "a blocked seal must not claim a count: {v}");
    let _ = fs::remove_dir_all(&d);
}

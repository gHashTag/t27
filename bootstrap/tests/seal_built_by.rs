//! `seal --save` writes `built_by` -- the producer identity a silicon receipt
//! can match verbatim (#7075, prerequisite of #7072).
//!
//! receipt.t27's `producer_matches` compares producer identities verbatim:
//! matching is not graded. Before `built_by`, a seal named its producer only
//! as `sealed_by: t27c-bootstrap@<version>` -- no build commit -- so no
//! receipt's toolchain could ever equal any seal's producer, and the check the
//! contract defines was unanswerable. The fix is one producer string, defined
//! once (`producer_identity()`), carried by every NEW seal.
//!
//! This test runs with or without `zig` on PATH. Without it `seal --save`
//! refuses (#7243: the test record would describe the machine), so the save
//! passes `--force` there; `built_by` is written either way.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PASSING: &str = "module seal_built_by_probe;\n\
fn add(a: i32, b: i32) -> i32 {\n    return a + b;\n}\n\
test \"add_works\" {\n    assert add(1, 2) == 3;\n}\n";

fn scratch() -> PathBuf {
    let d = fs::canonicalize(std::env::temp_dir())
        .unwrap()
        .join(format!("t27-seal-built-by-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("specs/probe")).expect("specs dir");
    fs::create_dir_all(d.join(".trinity/seals")).expect("seals dir");
    fs::write(d.join("specs/probe/built_by_probe.t27"), PASSING).expect("write spec");
    d
}

fn save(dir: &Path) -> std::process::Output {
    let zig = Command::new("zig")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let mut c = Command::new(env!("CARGO_BIN_EXE_t27c"));
    c.current_dir(dir).args(["seal", "--save", "specs/probe/built_by_probe.t27"]);
    if !zig {
        c.arg("--force");
    }
    c.output().expect("run t27c seal --save")
}

/// The one seal file the save wrote, parsed.
fn one_seal(dir: &Path) -> serde_json::Value {
    let seals = fs::read_dir(dir.join(".trinity/seals"))
        .expect("seals dir readable")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
        .collect::<Vec<_>>();
    assert_eq!(seals.len(), 1, "one spec, one seal: {seals:?}");
    serde_json::from_str(&fs::read_to_string(&seals[0]).unwrap()).expect("seal is JSON")
}

/// Every new seal carries a producer identity a receipt can match verbatim:
/// `t27c-bootstrap@<version>+<git>`, one `@`, one `+`, and a non-empty build
/// tail (the short sha, or honestly `unknown` outside a git checkout).
#[test]
fn every_new_seal_names_its_build() {
    let dir = scratch();
    let out = save(&dir);
    assert!(
        out.status.success(),
        "seal --save failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let seal = one_seal(&dir);

    let built = seal["built_by"].as_str().unwrap_or_else(|| {
        panic!("built_by absent from seal: {seal}")
    });
    assert!(
        built.starts_with("t27c-bootstrap@"),
        "producer identity is one format, defined once: {built}"
    );
    let parts: Vec<&str> = built.splitn(3, &['@', '+'][..]).collect();
    assert_eq!(parts.len(), 3, "name@version+git: {built}");
    assert!(!parts[1].is_empty(), "version present: {built}");
    assert!(
        !parts[2].is_empty(),
        "build tail present (short sha, or unknown): {built}"
    );

    // sealed_by stays the family/version claim it always was; built_by is the
    // build claim. Two fields, two different things, neither a copy.
    let sealed_by = seal["sealed_by"].as_str().expect("sealed_by present");
    assert_eq!(
        sealed_by,
        format!("t27c-bootstrap@{}", parts[1]),
        "sealed_by remains version-only"
    );

    let _ = fs::remove_dir_all(&dir);
}

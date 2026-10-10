//! A seal records the spec, not the machine that sealed it (#7243).
//!
//! Two facts about the sealing host used to reach the test record:
//! - zig's first error line, with the temp directory `test-report` builds in
//!   (`.../t27c-test-report-<stem>-<pid>/spec.zig:...`), so two reseals of
//!   one spec with one binary wrote two different files;
//! - "zig not on PATH", written over the spec's last measured result with
//!   exit 0, so a bulk reseal on a host without zig replaced every record.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Generates in all four backends; its Zig does not compile (`u8` cannot
/// hold 300), so its seal is BLOCKED with zig's own error line.
const BLOCKED: &str = "module seal_blocked_probe;\n\
fn wide() -> u8 {\n    return 300;\n}\n\
test \"wide_is_wide\" {\n    assert wide() == 44;\n}\n";

const PASSING: &str = "module seal_passing_probe;\n\
fn add(a: i32, b: i32) -> i32 {\n    return a + b;\n}\n\
test \"add_works\" {\n    assert add(1, 2) == 3;\n}\n";

fn scratch(tag: &str, spec: &str) -> PathBuf {
    let d = fs::canonicalize(std::env::temp_dir())
        .unwrap()
        .join(format!("t27-seal-stable-{}-{}", tag, std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("specs/probe")).expect("specs dir");
    fs::create_dir_all(d.join(".trinity/seals")).expect("seals dir");
    fs::write(d.join("specs/probe/probe.t27"), spec).expect("write spec");
    d
}

fn save(dir: &Path, path_env: Option<&str>, force: bool) -> std::process::Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_t27c"));
    c.current_dir(dir).args(["seal", "--save", "specs/probe/probe.t27"]);
    if force {
        c.arg("--force");
    }
    if let Some(p) = path_env {
        c.env("PATH", p);
    }
    c.output().expect("run t27c seal --save")
}

fn seal_files(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir.join(".trinity/seals"))
        .expect("seals dir readable")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
        .collect()
}

/// The one seal the save wrote, without the one field a reseal must change.
fn seal_without_time(dir: &Path) -> serde_json::Value {
    let seals = seal_files(dir);
    assert_eq!(seals.len(), 1, "one spec, one seal: {seals:?}");
    let mut v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&seals[0]).unwrap()).expect("seal is JSON");
    v.as_object_mut().unwrap().remove("sealed_at");
    v
}

fn zig_on_path() -> bool {
    Command::new("zig")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn resealed_twice(tag: &str, spec: &str) -> (serde_json::Value, serde_json::Value) {
    let dir = scratch(tag, spec);
    let first = save(&dir, None, false);
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let a = seal_without_time(&dir);
    let second = save(&dir, None, false);
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    let b = seal_without_time(&dir);
    let _ = fs::remove_dir_all(&dir);
    (a, b)
}

/// Each `seal --save` is its own process, so before #7243 the pid in the
/// blocked line differed between the two files.
#[test]
fn a_blocked_reseal_changes_only_sealed_at() {
    if !zig_on_path() {
        eprintln!("skipped: zig not on PATH");
        return;
    }
    let (a, b) = resealed_twice("blocked", BLOCKED);
    let blocked = a["tests"]["blocked"].as_str().unwrap_or_else(|| panic!("not blocked: {a}"));
    assert!(blocked.starts_with("does not compile: spec.zig:"), "{blocked}");
    assert_eq!(a, b);
}

#[test]
fn a_passing_reseal_changes_only_sealed_at() {
    if !zig_on_path() {
        eprintln!("skipped: zig not on PATH");
        return;
    }
    let (a, b) = resealed_twice("passing", PASSING);
    assert_eq!(a["tests"]["passed"], 1, "{a}");
    assert_eq!(a, b);
}

/// An empty directory as PATH: zig cannot be found, whatever the host has.
#[test]
fn no_zig_is_refused_unless_forced() {
    let dir = scratch("nozig", PASSING);
    let empty = dir.join("empty-path");
    fs::create_dir_all(&empty).unwrap();
    let path = empty.to_str().unwrap();

    let refused = save(&dir, Some(path), false);
    assert!(!refused.status.success(), "seal --save without zig exited 0");
    let err = String::from_utf8_lossy(&refused.stderr);
    assert!(err.contains("zig not on PATH"), "{err}");
    assert!(seal_files(&dir).is_empty(), "a refused seal was written");

    let forced = save(&dir, Some(path), true);
    assert!(forced.status.success(), "{}", String::from_utf8_lossy(&forced.stderr));
    let seal = seal_without_time(&dir);
    assert_eq!(seal["tests"]["blocked"], "zig not on PATH", "{seal}");
    let _ = fs::remove_dir_all(&dir);
}

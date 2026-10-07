//! #7308: gen-c refuses an integer `switch` with no `else` arm instead of ending
//! its ternary chain with `0`. The repro is specs/compiler/genc_switch_exhaustive.t27:
//! as written it is accepted and its `test` blocks run; with the `else` arm of
//! `tens` deleted it must be refused.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "t27c-genc-switch-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("create scratch directory");
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const ISSUE_REPRO: &str = "module sw;\nfn g(x: i64) -> i64 {\n    return switch (x) {\n        1 => 10,\n        2 => 20,\n    };\n}\n";

fn spec() -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs/compiler/genc_switch_exhaustive.t27"),
    )
    .expect("read specs/compiler/genc_switch_exhaustive.t27")
}

fn gen_c(scratch: &Scratch, source: &str) -> Output {
    let spec = scratch.0.join("fixture.t27");
    std::fs::write(&spec, source).expect("write fixture");
    Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&spec)
        .output()
        .expect("run t27c gen-c")
}

fn assert_refused(out: &Output) {
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "gen-c must refuse a switch on an integer with no else arm; it wrote:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        err.contains("non-exhaustive switch"),
        "the error must name the non-exhaustive switch: {err}"
    );
}

#[test]
fn the_issue_repro_is_refused() {
    let scratch = Scratch::new();
    assert_refused(&gen_c(&scratch, ISSUE_REPRO));
}

#[test]
fn with_an_else_arm_the_bytes_are_unchanged() {
    let scratch = Scratch::new();
    let src = ISSUE_REPRO.replace("        2 => 20,\n", "        2 => 20,\n        else => 0,\n");
    assert_ne!(src, ISSUE_REPRO, "the else arm must reach the source");
    let out = gen_c(&scratch, &src);
    let c = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "gen-c failed: {}", String::from_utf8_lossy(&out.stderr));
    assert!(c.contains("(x == 1) ? (10) : (x == 2) ? (20) : 0"), "C output:\n{c}");
}

#[test]
fn deleting_the_spec_else_arm_is_refused() {
    let original = spec();
    let changed = original.replacen("            else => 0,\n", "", 1);
    assert_ne!(original, changed, "the mutation must reach the source");
    let scratch = Scratch::new();
    assert_refused(&gen_c(&scratch, &changed));
}

#[test]
fn an_underscore_arm_counts_as_the_catch_all() {
    let scratch = Scratch::new();
    let src = ISSUE_REPRO.replace("        2 => 20,\n", "        2 => 20,\n        _ => 0,\n");
    let out = gen_c(&scratch, &src);
    assert!(out.status.success(), "gen-c failed: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn the_spec_is_accepted_and_its_tests_pass() {
    let scratch = Scratch::new();
    let out = gen_c(&scratch, &spec());
    assert!(out.status.success(), "gen-c failed: {}", String::from_utf8_lossy(&out.stderr));
    if Command::new("cc").arg("--version").output().is_err() {
        eprintln!("cc not found; skipping the execution check");
        return;
    }
    let c = scratch.0.join("fixture.c");
    std::fs::write(&c, &out.stdout).expect("write generated C");
    let bin = scratch.0.join("fixture");
    let built = Command::new("cc")
        .args(["-std=gnu11", "-Werror", "-DT27_TEST_MAIN"])
        .arg(&c)
        .arg("-o")
        .arg(&bin)
        .output()
        .expect("run cc");
    assert!(
        built.status.success(),
        "generated C must compile: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new(&bin).output().expect("run the generated tests");
    assert!(
        run.status.success(),
        "the spec's tests failed under gen-c:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

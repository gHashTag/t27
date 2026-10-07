//! #7307: gen-c declares a range-`for` counter as `size_t`, the C type of `usize`.
//! The repro is specs/compiler/genc_for_counter.t27; its `test` blocks are built
//! with -DT27_TEST_MAIN and run, so an `int` counter is a failed assertion.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "t27c-genc-for-counter-{}-{}",
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

const ISSUE_REPRO: &str = "module fr;\nfn f(n: usize) -> usize {\n    var s: usize = 0;\n    for (0..n) |i| {\n        s = s + 1;\n    }\n    return s;\n}\n";

fn gen_c(scratch: &Scratch, source: &str) -> String {
    let spec = scratch.0.join("fixture.t27");
    std::fs::write(&spec, source).expect("write fixture");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&spec)
        .output()
        .expect("run t27c gen-c");
    assert!(
        out.status.success(),
        "gen-c failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("gen-c output is UTF-8")
}

fn cc_available() -> bool {
    Command::new("cc").arg("--version").output().is_ok()
}

fn build_and_run_tests(scratch: &Scratch, c: &str) -> Output {
    let path = scratch.0.join("fixture.c");
    std::fs::write(&path, c).expect("write generated C");
    let bin = scratch.0.join("fixture");
    let built = Command::new("cc")
        .args(["-std=gnu11", "-Werror", "-DT27_TEST_MAIN"])
        .arg(&path)
        .arg("-o")
        .arg(&bin)
        .output()
        .expect("run cc");
    assert!(
        built.status.success(),
        "generated C must compile: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    Command::new(&bin).output().expect("run the generated tests")
}

#[test]
fn the_issue_repro_declares_a_size_t_counter() {
    let scratch = Scratch::new();
    let c = gen_c(&scratch, ISSUE_REPRO);
    assert_eq!(
        c.matches("for (size_t i = 0; i < n; i++) {").count(),
        1,
        "C output:\n{c}"
    );
    assert_eq!(c.matches("for (int ").count(), 0, "C output:\n{c}");
}

#[test]
fn inclusive_and_discarded_captures_are_size_t_too() {
    let scratch = Scratch::new();
    let src = "module fr2;\nfn f(n: usize) -> usize {\n    var s: usize = 0;\n    for (0..=n) |i| {\n        s = s + i;\n    }\n    for (0..n) |_| {\n        s = s + 1;\n    }\n    return s;\n}\n";
    let c = gen_c(&scratch, src);
    assert!(c.contains("for (size_t i = 0; i < (n + 1); i++) {"), "C output:\n{c}");
    assert!(
        c.contains("for (size_t __t27_i = 0; __t27_i < n; __t27_i++) {"),
        "C output:\n{c}"
    );
    assert!(!c.contains("for (int "), "C output:\n{c}");
}

#[test]
fn the_spec_tests_pass_past_int_max() {
    if !cc_available() {
        eprintln!("cc not found; skipping the execution check");
        return;
    }
    let src = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs/compiler/genc_for_counter.t27"),
    )
    .expect("read specs/compiler/genc_for_counter.t27");
    let scratch = Scratch::new();
    let c = gen_c(&scratch, &src);
    let run = build_and_run_tests(&scratch, &c);
    assert!(
        run.status.success(),
        "the spec's tests failed under gen-c:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

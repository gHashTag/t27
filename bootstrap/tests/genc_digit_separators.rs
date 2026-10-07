//! #7318: gen-c drops the `_` digit separators of every number it writes, and
//! two places were missed: the dimension of an array declarator and a negated
//! literal `const`, which gen-c writes as `#define N -1_000`. cc reads the
//! rest of the number as a suffix (`invalid suffix "_0" on integer
//! constant`), so the header did not compile.
//!
//! Part of #5980: the self-hosted core refuses a `_` in exactly these places
//! until gen-c writes them without one.

mod common;

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let d = std::env::temp_dir().join(format!(
            "t27c-genc-digitsep-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("create scratch directory");
        Self(d)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn gen_c(s: &Scratch, src: &str) -> String {
    let p = s.0.join("in.t27");
    std::fs::write(&p, src).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&p)
        .output()
        .expect("run t27c gen-c");
    assert!(
        out.status.success(),
        "gen-c failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn cc_present() -> bool {
    Command::new("cc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// cc must compile the header with no error. Skipped where there is no cc.
fn assert_cc_clean(s: &Scratch, c: &str) {
    if !cc_present() {
        eprintln!("cc not found; skipping the compile check");
        return;
    }
    let p = s.0.join("out.h");
    std::fs::write(&p, c).expect("write C");
    let diags = common::cc_check(&p, &[]);
    assert_eq!(
        common::error_count(&diags),
        0,
        "cc refused the generated C:\n{}\n--- C ---\n{}",
        diags,
        c
    );
}

/// The issue's reproduction, verbatim.
const ISSUE_REPRO: &str =
    "module m;\nvar a: [1_0]u8 = [_]u8{0} ** 1_0;\nconst N: i64 = -1_000;\n";

#[test]
fn the_issue_repro_has_no_separator_left() {
    let s = Scratch::new();
    let c = gen_c(&s, ISSUE_REPRO);
    assert!(!c.contains("_0"), "a `_` separator reached C:\n{}", c);
    assert!(
        c.contains("static uint8_t a[10] = "),
        "the dimension must be written `[10]`:\n{}",
        c
    );
    assert!(
        c.lines().any(|l| l == "#define N -1000"),
        "the negated const must be written `-1000`:\n{}",
        c
    );
    assert_cc_clean(&s, &c);
}

#[test]
fn hex_and_binary_dimensions_and_negations_lose_the_separator_too() {
    let s = Scratch::new();
    let c = gen_c(
        &s,
        "module m;\nvar h: [0x1_0]u8 = [_]u8{0} ** 16;\nconst B: i64 = -0b1_0;\nconst X: i64 = -0x1_F;\n",
    );
    assert!(c.contains("static uint8_t h[0x10] = "), "hex dimension:\n{}", c);
    assert!(c.lines().any(|l| l == "#define B -0b10"), "binary negation:\n{}", c);
    assert!(c.lines().any(|l| l == "#define X -0x1F"), "hex negation:\n{}", c);
    assert!(!c.contains("0x1_") && !c.contains("0b1_"), "a separator reached C:\n{}", c);
}

#[test]
fn nested_and_local_dimensions_lose_the_separator() {
    let s = Scratch::new();
    let c = gen_c(
        &s,
        "module m;\nvar g: [1_0][2_0]u8 = undefined;\n\
         fn f() -> u8 {\n    var b: [1_6]u8 = [_]u8{0} ** 1_6;\n    return b[0];\n}\n",
    );
    assert!(c.contains("g[10][20]"), "nested module dimensions:\n{}", c);
    assert!(c.contains("uint8_t b[16]"), "local dimension:\n{}", c);
    assert!(!c.contains("1_"), "a separator reached C:\n{}", c);
    assert_cc_clean(&s, &c);
}

#[test]
fn a_named_dimension_and_a_plain_negative_are_unchanged() {
    let s = Scratch::new();
    let c = gen_c(
        &s,
        "module m;\nconst LEN: usize = 4;\nvar a: [LEN]u8 = [_]u8{0} ** LEN;\nconst M: i64 = -7;\n",
    );
    assert!(c.contains("static uint8_t a[LEN] = "), "named dimension:\n{}", c);
    assert!(c.lines().any(|l| l == "#define M -7"), "plain negative:\n{}", c);
    assert_cc_clean(&s, &c);
}

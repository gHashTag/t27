//! A bare list of numeric literals must be given an element type in C.
//!
//! `var x = [1, 2, 3]` emitted `__auto_type x = { 1, 2, 3 }` --
//! `cannot use '__auto_type' with initializer list in C`. Rust writes
//! `let mut x = [1, 2, 3]` and Zig `var x = .{ 1, 2, 3 }`; only C has no
//! inference for a brace list, so the type has to be named.
//!
//! The choice matches what a SCALAR literal already gets, so the two agree:
//! `var x = 1` emits `uint32_t x = 1` here and `var x: u32 = 1` in Zig.
//!
//! A STRING literal is an `ExprLiteral` whose `value` is the text without its
//! quotes, so `["12", "34"]` passed a digit test and was typed `uint32_t` --
//! four "incompatible pointer to integer conversion" errors in one corpus file.
//! That is what `a_list_of_strings_is_refused` holds.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(body: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-clit-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    let p = d.join("in.t27");
    std::fs::write(&p, format!("module P {{\n    fn f(v: i32) -> i32 {{ {body} return 0; }}\n}}\n"))
        .expect("write");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&p)
        .output()
        .expect("t27c");
    assert!(out.status.success(), "gen-c failed: {}", String::from_utf8_lossy(&out.stderr));
    (String::from_utf8_lossy(&out.stdout).to_string(), d)
}

fn errors(h: &str, d: &std::path::Path) -> String {
    let p = d.join("h.h");
    std::fs::write(&p, h).expect("write");
    let out = Command::new("cc")
        .args(["-std=c11", "-Wall", "-Wextra", "-Wno-unused-parameter",
               "-ferror-limit=0", "-fsyntax-only", "-x", "c"])
        .arg(&p)
        .output()
        .expect("cc");
    String::from_utf8_lossy(&out.stderr).to_string()
}

#[test]
fn an_integer_list_takes_the_same_type_a_scalar_would() {
    let (h, d) = gen_c("var x = [1, 2, 3];", "ints");
    assert!(h.contains("uint32_t x[3] = { 1, 2, 3 };"), "got:\n{h}");
    assert!(!h.contains("__auto_type x"), "and no __auto_type:\n{h}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and it compiles");
    }
    // The agreement that makes `uint32_t` the right choice rather than an
    // arbitrary one: a scalar literal already gets it.
    let (s, _) = gen_c("var y = 1;", "scalar");
    assert!(s.contains("uint32_t y = 1;"), "the scalar precedent:\n{s}");
}

#[test]
fn a_fractional_element_makes_the_list_double() {
    let (h, d) = gen_c("var x = [1.0, 2.0];", "floats");
    assert!(h.contains("double x[2] = { 1.0, 2.0 };"), "got:\n{h}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and it compiles");
    }
    // Mixed: C's own promotion says double, and so does this.
    let (m, _) = gen_c("var x = [1, 2.5];", "mixed");
    assert!(m.contains("double x[2] ="), "a mixed list is double:\n{m}");
}

#[test]
fn a_list_of_strings_is_refused() {
    // The negative control that a corpus measurement produced. A string
    // literal's `value` is its text without quotes, so `["12", "34"]` looks
    // like digits; typing it `uint32_t` gave "incompatible pointer to integer
    // conversion" four times in one file.
    let (h, _d) = gen_c(r#"var x = ["12", "34"];"#, "strings");
    assert!(
        !h.contains("uint32_t x["),
        "a list of strings must not be typed as integers:\n{h}"
    );
}

#[test]
fn a_list_of_calls_is_refused() {
    // A call has a return type this does not read, and guessing one would be
    // worse than `__auto_type` -- which is at least type-correct when it works.
    let (h, _d) = gen_c("var x = [f(1), f(2)];", "calls");
    assert!(!h.contains("uint32_t x["), "a list of calls must not be guessed at:\n{h}");
}

#[test]
fn an_annotated_list_is_unchanged() {
    // The path that already worked. The inference is only consulted when there
    // is no annotation and no type on any child.
    let (h, d) = gen_c("var x : [2]u8 = [1, 2];", "annotated");
    assert!(h.contains("uint8_t x[2] = { 1, 2 };"), "got:\n{h}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and it compiles");
    }
}

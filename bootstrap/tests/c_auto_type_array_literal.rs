//! An array literal with no annotation must not become `__auto_type x = { … }`.
//!
//!     error: cannot use '__auto_type' with initializer list in C
//!
//! 1729 of them: the largest single error class in the generated C corpus,
//! more than twice the next one. `__auto_type` is a GNU extension that takes
//! its type from the initialiser, and a brace list has no type to take.
//!
//! The repair for this existed (W699 rung 3) and required the array literal to
//! carry its own `extra_type`. `[W{...}, W{...}]` does not: the type sits on
//! the CHILD, because each element is a named struct literal. So the condition
//! is widened to look there, and the element type is taken from the first
//! element when the array has none.
//!
//! Every test compiles the header. The difference between `__auto_type w = {…}`
//! and `W w[2] = {…}` is invisible to a text assertion that only checks the
//! name.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-cauto-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    let p = d.join("in.t27");
    std::fs::write(&p, spec).expect("write");
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

const STRUCT_ELEMS: &str = r#"
module P {
    struct W { code : i32, }
    fn f(v: i32) -> i32 { var w = [W{.code=1}, W{.code=2}]; return 0; }
}
"#;

#[test]
fn an_array_of_struct_literals_gets_a_real_type() {
    let (h, d) = gen_c(STRUCT_ELEMS, "structelems");
    assert!(h.contains("W w[2] = {"), "the element type comes from the first element:\n{h}");
    assert!(!h.contains("__auto_type w"), "and `__auto_type` is gone:\n{h}");
    if cc_present() {
        let e = errors(&h, &d);
        assert!(!e.contains("error"), "and it compiles:\n{e}");
    }
}

#[test]
fn the_annotated_form_still_works() {
    // The case W699 rung 3 already handled. A widened condition could have
    // taken a different path for it.
    let (h, d) = gen_c(
        "module P {\n    fn f(v: i32) -> u8 { var x : [2]u8 = [1, 2]; return x[0]; }\n}\n",
        "annotated",
    );
    assert!(h.contains("uint8_t x[2] = {"), "got:\n{h}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and it compiles");
    }
}

#[test]
fn a_float_element_is_lowered_not_passed_through() {
    // W583, third instance: THIS arm gated on `is_primitive`, which lists only
    // the integer scalars, so `f64` would have reached C as `f64 x[2]`.
    //
    // The spelling matters and the first version of this test got it wrong: an
    // ANNOTATED local (`var x : [2]f64 = ...`) takes a different branch
    // entirely, so a mutant restoring the gate passed. `[_]f64{...}` is the
    // form that reaches here -- the one the W699 note is about.
    let (h, d) = gen_c(
        "module P {\n    fn f(v: i32) -> i32 { var x = [_]f64{1.0, 2.0}; return 0; }\n}\n",
        "float",
    );
    assert!(h.contains("double x[2] = {"), "got:\n{h}");
    assert!(!h.contains("f64 x["), "no t27 spelling:\n{h}");
    // And a GF16 element, which `is_primitive` also does not list.
    let (g, _) = gen_c(
        "module P {\n    fn f(v: i32) -> i32 { var y = [_]GF16{1, 2}; return 0; }\n}\n",
        "gf16elem",
    );
    assert!(g.contains("uint16_t y[2] = {"), "got:\n{g}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and it compiles");
    }
}

#[test]
fn a_scalar_local_still_uses_auto_type() {
    // The negative control. `__auto_type p = call();` is valid and useful --
    // the type follows the initialiser. Only a BRACE LIST has no type to
    // follow, and a change that removed `__auto_type` everywhere would pass
    // every other test in this file.
    let (h, _d) = gen_c(
        "module P {\n    fn mk(v: i32) -> i32 { return 1; }\n    fn f(v: i32) -> i32 { var p = mk(0); return p; }\n}\n",
        "scalar",
    );
    assert!(
        h.contains("__auto_type p = mk(0)"),
        "a call initialiser keeps __auto_type:\n{h}"
    );
}

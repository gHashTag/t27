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

#[test]
fn a_one_element_list_is_typed_too() {
    // The parser keeps a BARE list's elements in `extra_size` and leaves
    // `children` empty -- stated in a comment on the emitter arm, forty lines
    // from the code that reads it, and found only after `children` and
    // `value` had both been searched and both come back empty.
    //
    // 334 of the remaining class were `{ 0 }` and every one arrived this way.
    for (body, want) in [
        ("var x = [7];", "uint32_t x[1] = { 7 };"),
        ("var x = [0];", "uint32_t x[1] = { 0 };"),
        ("var x = [1.5];", "double x[1] = { 1.5 };"),
    ] {
        let (h, d) = gen_c(body, "one");
        assert!(h.contains(want), "for `{body}` expected `{want}`:\n{h}");
        if cc_present() {
            assert!(!errors(&h, &d).contains("error"), "and `{body}` must compile");
        }
    }
}

#[test]
fn the_repeat_form_takes_its_length_from_the_count() {
    // `[0; 4]` arrives as `extra_size` "0;4": the element is before the
    // semicolon and the LENGTH after it, not the number of commas.
    let (h, d) = gen_c("var x = [0; 4];", "repeat");
    assert!(h.contains("uint32_t x[4] ="), "the repeat count is the length:\n{h}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and it compiles");
    }
}

#[test]
fn a_dotted_token_is_a_float_only_when_it_is_one() {
    // The mutant that exposed both halves of this. Deleting the shape check
    // types `[1.2.3]` as `double x[1] = { 1.2.3 }` -- a malformed C literal in
    // a well-formed declaration. And the FIRST version of the check, requiring
    // digits on both sides of the dot, refused `1.`, which IS valid C: the
    // mutant was better than the guard for that input.
    for (body, tag) in [("var x = [1.2.3];", "three"), ("var a = 1; var x = [a.b];", "field")] {
        let (h, _d) = gen_c(body, tag);
        assert!(!h.contains("double x["), "`{body}` is not a float:\n{h}");
        assert!(!h.contains("uint32_t x["), "nor an integer:\n{h}");
    }
    let (h, d) = gen_c("var x = [1.];", "trailing");
    assert!(h.contains("double x[1] = { 1. };"), "`1.` is a valid C double:\n{h}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and it compiles");
    }
}

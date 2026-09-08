//! `abs(x)` in C, without asking what type `x` is.
//!
//! 389 uses in 43 specs, lowered by Rust as `(v).abs()` and by Zig as
//! `@abs(v)`, and refused here for two passes with a stated reason: C has TWO
//! of them -- `abs` for `int`, `fabs` for `double` -- and choosing without the
//! argument's type is a silent truncation, which is the shape this campaign
//! has declined four times.
//!
//! C11 answers it without the type. `_Generic` dispatches on the argument's
//! own type, evaluates it once (unlike a `(x) < 0 ? -(x) : (x)` macro, which
//! evaluates twice), and is standard in the `-std=c11` this corpus compiles
//! with. The macro is named `t27_abs` and the CALL is rewritten, so a
//! `<stdlib.h>` `abs` in scope is never shadowed.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-abs-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    let p = d.join("in.t27");
    std::fs::write(&p, spec).expect("write");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c")).arg("gen-c").arg(&p).output().expect("t27c");
    let h = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(!h.is_empty(), "gen-c produced an EMPTY header");
    (h, d)
}

fn errors(h: &str, d: &std::path::Path) -> usize {
    let p = d.join("h.h");
    std::fs::write(&p, h).expect("write");
    let out = Command::new("cc")
        .args(["-std=c11", "-ferror-limit=0", "-fsyntax-only", "-x", "c"])
        .arg(&p)
        .output()
        .expect("cc");
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| {
            let mut it = l.splitn(4, ':');
            it.next().is_some()
                && it.next().map_or(false, |s| s.trim().parse::<u32>().is_ok())
                && it.next().map_or(false, |s| s.trim().parse::<u32>().is_ok())
                && it.next().map_or(false, |s| s.trim_start().starts_with("error: "))
        })
        .count()
}

#[test]
fn abs_becomes_the_type_generic_macro() {
    let (h, d) = gen_c(
        "module A {\n    fn f(v: f64) -> f64 { var a = abs(v); return a; }\n}\n",
        "float",
    );
    assert!(h.contains("t27_abs(v)"), "the call is rewritten:\n{h}");
    assert!(h.contains("#define t27_abs(x) _Generic("), "and the macro is defined:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn the_same_spelling_serves_an_integer() {
    // The whole point of `_Generic`: one spelling, two C functions, chosen by
    // the argument rather than by the emitter guessing.
    let (h, d) = gen_c(
        "module A {\n    fn g(v: i32) -> i32 { var b = abs(v); return b; }\n}\n",
        "int",
    );
    assert!(h.contains("t27_abs(v)"), "got:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and an integer argument compiles too:\n{h}");
    }
}

#[test]
fn a_module_without_abs_gets_neither_macro_nor_headers() {
    let (h, _d) = gen_c("module A {\n    fn f(v: i32) -> i32 { return v; }\n}\n", "none");
    assert!(!h.contains("t27_abs"), "unused, so not defined:\n{h}");
    assert!(!h.contains("#include <stdlib.h>"), "and no header is pulled in:\n{h}");
}

#[test]
fn a_spec_that_declares_its_own_abs_keeps_it() {
    // THE DISCRIMINATING CASE, and the same guard the cast builtin and the
    // libm include use: a module defining `fn abs` means that function.
    let (h, d) = gen_c(
        "module A {\n    fn abs(x: i32) -> i32 { return x; }\n\
         \x20   fn f(v: i32) -> i32 { var a = abs(v); return a; }\n}\n",
        "declared",
    );
    assert!(h.contains("abs(v)") && !h.contains("t27_abs(v)"), "the spec's own abs:\n{h}");
    assert!(!h.contains("#define t27_abs"), "and no macro beside it:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it still compiles:\n{h}");
    }
}

#[test]
fn the_macro_evaluates_its_argument_once() {
    // Stated as a property of the expansion rather than as its text: the
    // obvious `((x) < 0 ? -(x) : (x))` spelling names `x` twice, and `abs(i++)`
    // would then increment twice. `_Generic` selects a FUNCTION and calls it.
    let (h, _d) = gen_c(
        "module A {\n    fn f(v: f64) -> f64 { var a = abs(v); return a; }\n}\n",
        "once",
    );
    let line = h
        .lines()
        .find(|l| l.contains("#define t27_abs"))
        .expect("the macro")
        .to_string();
    // The PARAMETER LIST is not the body: `#define t27_abs(x)` contains `(x)`
    // too, and counting the whole line gives three. The first version of this
    // assertion did exactly that and failed on a correct macro.
    let body = line.split_once("t27_abs(x)").expect("the parameter list").1;
    assert!(
        body.trim_start().starts_with("_Generic((x),"),
        "the controlling expression is a TYPE query, which C does not evaluate:\n{line}"
    );
    assert_eq!(
        body.matches("(x)").count(),
        2,
        "one unevaluated `_Generic((x), ...)` and one call `(x)` -- not a \
         repeated operand the way `((x) < 0 ? -(x) : (x))` would be:\n{line}"
    );
}

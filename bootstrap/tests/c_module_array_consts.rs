//! A module-level `[N]T` constant must put its brackets after the name.
//!
//!     static const [4]u8 A = { 1, 2, 3, 4 };
//!     error: brackets are not allowed here; to declare an array, place the
//!            brackets after the identifier
//!
//! Fourth position for the same declarator rule, and the fourth is built from
//! the same helper rather than a fourth copy: a parameter wants
//! `T x[static N]` (#3435), a struct field and now a const want `T name[N]`
//! (#3446), a local wants it too (#3448).
//!
//! The MUTABLE twin four lines above carried the same defect and was found by
//! grepping every site that builds a declarator from a type and a name --
//! fixing only the one the probe used would have left it.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-cconst-{tag}-{}-{}",
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

#[test]
fn a_const_array_declares_its_brackets_after_the_name() {
    let (h, d) = gen_c(
        "module P {\n    const A : [4]u8 = [1,2,3,4];\n    fn f(v: i32) -> u8 { return A[0]; }\n}\n",
        "const",
    );
    assert!(
        h.contains("static const uint8_t A[4] ="),
        "the declarator must be `T name[N]`:\n{h}"
    );
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and the header must compile");
    }
}

#[test]
fn a_module_var_array_does_too() {
    // The twin. It sat four lines above the site the probe found, and only a
    // grep for every declarator-building site turned it up.
    let (h, d) = gen_c(
        "module P {\n    var A : [4]u8 = [1,2,3,4];\n    fn f(v: i32) -> u8 { return A[0]; }\n}\n",
        "var",
    );
    assert!(
        h.contains("static uint8_t A[4] ="),
        "a mutable module array declares the same way:\n{h}"
    );
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and the header must compile");
    }
}

#[test]
fn a_const_sized_by_a_named_constant_keeps_the_name() {
    // `#define N 4` is emitted earlier, so the name is both valid and readable.
    let (h, d) = gen_c(
        "module P {\n    const N : usize = 4;\n    const A : [N]u8 = [1,2,3,4];\n    fn f(v: i32) -> u8 { return A[0]; }\n}\n",
        "named",
    );
    assert!(h.contains("static const uint8_t A[N] ="), "got:\n{h}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and the header must compile");
    }
}

#[test]
fn a_scalar_const_is_untouched() {
    // The negative control: scalars take the `#define` path and a rewrite that
    // caught everything would move them.
    let (h, _d) = gen_c(
        "module P {\n    const A : u8 = 3;\n    fn f(v: i32) -> u8 { return A; }\n}\n",
        "scalar",
    );
    assert!(h.contains("#define A 3"), "a scalar const stays a #define:\n{h}");
    assert!(!h.contains("static const uint8_t A["), "and gains no brackets:\n{h}");
}

#[test]
fn a_slice_const_is_sized_by_its_initialiser() {
    // This test was WRONG when first written: it asserted that "the old
    // pointer lowering must survive", and there was no pointer lowering --
    // `type_to_c` passed `[]u8` through and C received `static const []u8 A`,
    // a defect that predated this change. `T name[] = { ... }` is legal C and
    // takes its size from the list, the same rule the local position needed.
    let (h, d) = gen_c(
        "module P {\n    const A : []u8 = [1,2,3];\n    fn f(v: i32) -> u8 { return A[0]; }\n}\n",
        "slice",
    );
    assert!(
        h.contains("static const uint8_t A[] ="),
        "a slice const is sized by its initialiser:\n{h}"
    );
    assert!(!h.contains("[]u8"), "and no t27 text survives:\n{h}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and the header must compile");
    }
}

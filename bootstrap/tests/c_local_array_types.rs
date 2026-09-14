//! The element type of a LOCAL array must be lowered, not passed through.
//!
//! `var x : [4]GF16;` reached C as `GF16 x[4];` -- `use of undeclared
//! identifier 'GF16'` -- while the SAME element is `uint16_t` in a parameter
//! (`uint16_t a[static 4]`) and in a struct field (`uint16_t f[4];`).
//!
//! The gate was `is_primitive`, and this is W583 a second time. That note sits
//! on `param_type_to_c` two hundred lines away and says it exactly: the gate
//! "lists only the integer scalars -- so `f32`, `f64`, `str`, `string` and
//! `gf16` took the pass-through arm and reached C unmapped even after
//! `type_to_c` learned them". The repair never travelled to the local path.
//!
//! Every assertion here hands the header to `cc`, because the text `GF16 x[4]`
//! and `uint16_t x[4]` are equally plausible-looking and only a compiler
//! separates them.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn local_decl(ty: &str, init: Option<&str>) -> (String, String) {
    let d = std::env::temp_dir().join(format!(
        "t27c-clocal-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("create temp dir");
    let body = match init {
        Some(i) => format!("var x : {ty} = {i};"),
        None => format!("var x : {ty};"),
    };
    let p = d.join("in.t27");
    std::fs::write(
        &p,
        format!("module P {{\n    struct Pair {{ a : i32, b : i32, }}\n    fn probe(v: i32) -> i32 {{ {body} return 0; }}\n}}\n"),
    )
    .expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&p)
        .output()
        .expect("run t27c");
    assert!(out.status.success(), "gen-c failed: {}", String::from_utf8_lossy(&out.stderr));
    let h = String::from_utf8_lossy(&out.stdout).to_string();
    let line = h
        .lines()
        .find(|l| l.trim_start().starts_with(|c: char| c.is_alphabetic()) && l.contains(" x") && l.trim_end().ends_with(';'))
        .unwrap_or_else(|| panic!("no local declaration for `{ty}` in:\n{h}"))
        .trim()
        .to_string();
    (line, h)
}

fn errors(header: &str, tag: &str) -> String {
    let d = std::env::temp_dir().join(format!("t27c-clocal-cc-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    let p = d.join("h.h");
    std::fs::write(&p, header).expect("write");
    let out = Command::new("cc")
        .args(["-std=c11", "-Wall", "-Wextra", "-Wno-unused-parameter", "-fsyntax-only", "-x", "c"])
        .arg(&p)
        .output()
        .expect("cc");
    String::from_utf8_lossy(&out.stderr).to_string()
}

#[test]
fn a_gf16_array_local_uses_the_c_element_type() {
    let (line, h) = local_decl("[4]GF16", None);
    assert_eq!(line, "uint16_t x[4];", "got `{line}`");
    if cc_present() {
        let e = errors(&h, "gf16");
        assert!(!e.contains("error"), "and it must compile:\n{e}");
    }
}

#[test]
fn a_float_array_local_uses_the_c_element_type() {
    // `f32`/`f64` were the types the W583 note named first, and they were still
    // passing through here.
    for (t, want) in [("[4]f64", "double x[4];"), ("[4]f32", "float x[4];")] {
        let (line, h) = local_decl(t, None);
        assert_eq!(line, want, "for `{t}` got `{line}`");
        if cc_present() {
            assert!(!errors(&h, "float").contains("error"), "`{t}` must compile");
        }
    }
}

#[test]
fn a_declared_struct_element_still_passes_through() {
    // The negative half: `type_to_c` passes a genuinely custom type through
    // unchanged, and it must -- `Pair` IS the C spelling, via the typedef.
    // A mapping that rewrote everything would break this.
    let (line, h) = local_decl("[4]Pair", None);
    assert_eq!(line, "Pair x[4];", "got `{line}`");
    if cc_present() {
        assert!(!errors(&h, "pair").contains("error"), "and it must compile");
    }
}

#[test]
fn a_slice_local_without_an_initialiser_is_a_pointer() {
    // `uint8_t x[];` is not a definition: "definition of variable with array
    // type needs an explicit size or an initializer".
    let (line, h) = local_decl("[]u8", None);
    assert_eq!(line, "uint8_t* x;", "got `{line}`");
    if cc_present() {
        assert!(!errors(&h, "slice").contains("error"), "and it must compile");
    }
}

#[test]
fn a_slice_local_WITH_an_initialiser_keeps_the_array_form() {
    // `T x[] = { ... }` is legal C and takes its size from the list. Rewriting
    // THAT to a pointer made the one corpus file carrying the shape worse --
    // +5 errors, nothing better. This test is that measurement, kept.
    let (line, _h) = local_decl("[]i32", Some("[1, 2, 3]"));
    assert!(
        line.starts_with("int32_t x[]"),
        "an initialised slice local keeps the sized-by-initialiser array form; got `{line}`"
    );
}

#[test]
fn a_const_qualified_slice_element_is_lowered() {
    // `[]const u8` carries the qualifier inside the element, so the element
    // text is the literal "const u8" and C received `const u8* x` --
    // "unknown type name 'u8'".
    let (line, h) = local_decl("[]const u8", None);
    assert_eq!(line, "const uint8_t* x;", "got `{line}`");
    if cc_present() {
        assert!(!errors(&h, "constslice").contains("error"), "and it must compile");
    }
}

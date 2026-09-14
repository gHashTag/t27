//! A `[N]T` struct field must be STORAGE in C, not a pointer.
//!
//! `struct Holder { f : [4]u8, g : i32 }` reached C as `uint8_t* f;`. The field
//! held nothing: `h.f[0] = 1` wrote through an uninitialised pointer, and the
//! same struct measured 16 bytes in C against 8 in Rust and Zig, which both
//! store the four bytes inline. Three backends agreed and C was alone.
//!
//! A wrong SIZE is a missing check; this was a missing object. So the test
//! below does not read the emitted text alone -- it compiles a program that
//! prints `sizeof` and compares the number, because the text can look right
//! while the layout does not.
//!
//! Three positions, three different answers, and that is the point of the
//! narrowing here: a parameter wants `T x[static N]` (#3435), a field wants
//! `T f[N]`, and a return cannot be an array in C at all (#3445).

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "t27c-cfield-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("create temp dir");
    d
}

fn gen(sub: &str, spec: &str, d: &std::path::Path) -> String {
    let p = d.join("in.t27");
    std::fs::write(&p, spec).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg(sub)
        .arg(&p)
        .output()
        .expect("run t27c");
    assert!(out.status.success(), "{sub} failed: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

const HOLDER: &str = r#"
module P {
    struct Holder { f : [4]u8, g : i32, }
    fn probe(h: Holder) -> i32 { return h.g; }
}
"#;

#[test]
fn a_fixed_array_field_is_storage_not_a_pointer() {
    let d = dir("text");
    let h = gen("gen-c", HOLDER, &d);
    assert!(
        h.contains("uint8_t f[4];"),
        "the field must declare four bytes of storage:\n{h}"
    );
    assert!(
        !h.contains("uint8_t* f;"),
        "and must not be a pointer:\n{h}"
    );
}

#[test]
fn the_c_struct_is_the_size_the_other_backends_give_it() {
    // The assertion that the text one cannot make. `uint8_t* f` and
    // `uint8_t f[4]` both LOOK like a four-byte field in a diff; only the
    // number distinguishes them, and it was 16 against 8.
    if !cc_present() {
        eprintln!("SKIP the_c_struct_is_the_size_the_other_backends_give_it: no cc on PATH");
        return;
    }
    let d = dir("size");
    let h = gen("gen-c", HOLDER, &d);
    let hp = d.join("h.h");
    std::fs::write(&hp, &h).expect("write header");
    let cp = d.join("m.c");
    std::fs::write(
        &cp,
        format!(
            "#include \"{}\"\n#include <stdio.h>\nint main(void){{ printf(\"%zu\\n\", sizeof(Holder)); return 0; }}\n",
            hp.display()
        ),
    )
    .expect("write main");
    let bin = d.join("m.bin");
    let out = Command::new("cc")
        .args(["-std=c11", "-o"])
        .arg(&bin)
        .arg(&cp)
        .output()
        .expect("run cc");
    assert!(
        out.status.success(),
        "the generated struct must compile:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&bin).output().expect("run");
    let size: usize = String::from_utf8_lossy(&run.stdout).trim().parse().expect("a number");
    assert_eq!(
        size, 8,
        "four inline bytes plus an i32 is 8; a pointer plus an i32 was 16"
    );
}

#[test]
fn a_slice_field_stays_a_pointer() {
    // The negative control. `[]u8` has no compile-time length, so a pointer IS
    // the intended lowering -- and a rewrite that turned every array-looking
    // field into storage would pass the positive test above.
    let d = dir("slice");
    let h = gen(
        "gen-c",
        "module P {\n    struct S { f : []u8, g : i32, }\n    fn probe(s: S) -> i32 { return s.g; }\n}\n",
        &d,
    );
    assert!(
        h.contains("uint8_t* f;"),
        "a slice field must keep the pointer lowering:\n{h}"
    );
}

#[test]
fn a_zero_length_field_stays_a_pointer() {
    // `uint8_t f[0];` inside a struct is a GCC extension, not standard C.
    let d = dir("zero");
    let h = gen(
        "gen-c",
        "module P {\n    struct S { f : [0]u8, g : i32, }\n    fn probe(s: S) -> i32 { return s.g; }\n}\n",
        &d,
    );
    assert!(!h.contains("f[0];"), "a zero-length array field must not be emitted:\n{h}");
    if cc_present() {
        let hp = d.join("z.h");
        std::fs::write(&hp, &h).expect("write");
        let out = Command::new("cc")
            .args(["-std=c11", "-Wall", "-Wextra", "-Wno-unused-parameter", "-fsyntax-only", "-x", "c"])
            .arg(&hp)
            .output()
            .expect("cc");
        assert!(
            !String::from_utf8_lossy(&out.stderr).contains("error"),
            "and the header must still compile:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn a_nested_array_field_keeps_both_dimensions() {
    // `[2][3]u8` used to emit `[3]u8* f` -- t27 syntax inside a C header,
    // which does not parse at all (#3443). Two dimensions is ordinary C.
    let d = dir("nested");
    let h = gen(
        "gen-c",
        "module P {\n    struct S { f : [2][3]u8, g : i32, }\n    fn probe(s: S) -> i32 { return s.g; }\n}\n",
        &d,
    );
    assert!(
        h.contains("uint8_t f[2][3];"),
        "both dimensions must survive:\n{h}"
    );
}

#[test]
fn the_parameter_position_is_untouched() {
    // The narrowing is load-bearing: a parameter wants `T x[static N]`, which
    // is what carries the caller-side check (#3435). A field rewrite that
    // leaked into parameters would silently undo it.
    let d = dir("param");
    let h = gen(
        "gen-c",
        "module P {\n    fn f(a: [4]u8) -> u8 { return a[0]; }\n}\n",
        &d,
    );
    assert!(
        h.contains("uint8_t a[static 4]"),
        "a parameter keeps [static N]:\n{h}"
    );
}

#[test]
fn the_rust_spelling_field_keeps_its_own_by_value_struct() {
    // `[u8; 4]` in a field already had an answer -- the hoisted
    // `t27_arr_uint8_t_4` value struct -- and this change must not take it
    // over. The position was uncovered: a mutant deleting the `;` guard in
    // `c_array_field` SURVIVED the rest of this file, because the emptiness
    // check happens to catch that spelling too. The guard is therefore
    // redundant TODAY and kept as intent; this test is what actually pins the
    // behaviour.
    let d = dir("rustspelling");
    let h = gen(
        "gen-c",
        "module P {\n    struct S { f : [u8; 4], g : i32, }\n    fn probe(s: S) -> i32 { return s.g; }\n}\n",
        &d,
    );
    assert!(
        h.contains("t27_arr_uint8_t_4 f;"),
        "the Rust spelling keeps its by-value struct field:\n{h}"
    );
    assert!(
        !h.contains("uint8_t f[4];"),
        "and must not be rewritten as a raw array by the [N]T path:\n{h}"
    );
}

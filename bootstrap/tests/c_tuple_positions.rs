//! A tuple must reach C as its hoisted struct in EVERY position, not only as a
//! return type.
//!
//! The `t27_tuple_*` typedef already existed and `c_return_type_r` was the only
//! caller, so a tuple in a parameter, a struct field or a local reached C as
//! the t27 text:
//!
//!     int32_t probe(H h, (u8, i32) t);
//!     struct H { (u8, i32) f; };
//!     (u8, i32) x;
//!
//! none of which is C.
//!
//! BOTH HALVES WERE MISSING and only one was obvious. Teaching the use sites to
//! name the struct, without also teaching the typedef collection to emit it,
//! produced `unknown type name 't27_tuple_uint8_t_int32_t'` -- WORSE than the
//! t27 text, because it looks right. That is why every test here hands the
//! header to `cc` instead of matching on the type name.
//!
//! Corpus population is ZERO: no spec puts a tuple in these positions today.
//! Measured, and the change alters not one generated file.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-ctuple-{tag}-{}-{}",
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
        .expect("run t27c");
    assert!(out.status.success(), "gen-c failed: {}", String::from_utf8_lossy(&out.stderr));
    (String::from_utf8_lossy(&out.stdout).to_string(), d)
}

/// Compile the header and return cc's stderr.
fn compile(h: &str, d: &std::path::Path) -> String {
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

const ALL_THREE: &str = r#"
module P {
    struct H { f : (u8, i32), g : i32, }
    fn probe(h: H, t: (u8, i32)) -> i32 { var x : (u8, i32); return 0; }
}
"#;

#[test]
fn a_tuple_in_every_position_compiles() {
    // The assertion the first attempt would have failed. Naming the hoisted
    // struct is not enough; it has to exist.
    if !cc_present() {
        eprintln!("SKIP a_tuple_in_every_position_compiles: no cc on PATH");
        return;
    }
    let (h, d) = gen_c(ALL_THREE, "all");
    let e = compile(&h, &d);
    assert!(
        !e.contains("error"),
        "a tuple in a field, a parameter and a local must all compile:\n{e}"
    );
}

#[test]
fn the_hoisted_struct_is_both_named_and_defined() {
    let (h, _d) = gen_c(ALL_THREE, "typedef");
    assert!(
        h.contains("} t27_tuple_uint8_t_int32_t;"),
        "the typedef must be emitted:\n{h}"
    );
    for spot in ["t27_tuple_uint8_t_int32_t f;", "t27_tuple_uint8_t_int32_t t)", "t27_tuple_uint8_t_int32_t x;"] {
        assert!(h.contains(spot), "expected `{spot}` in:\n{h}");
    }
    assert!(
        !h.contains("(u8, i32)"),
        "and no t27 tuple text may survive into C:\n{h}"
    );
}

#[test]
fn a_tuple_return_still_works() {
    // The position that already worked. A change to the shared path could
    // have taken it away without any other test noticing.
    let (h, d) = gen_c(
        "module P {\n    fn probe(v: i32) -> (u8, i32) { return (1, 2); }\n}\n",
        "ret",
    );
    assert!(
        h.contains("t27_tuple_uint8_t_int32_t probe("),
        "the return position keeps its hoisted struct:\n{h}"
    );
    if cc_present() {
        let e = compile(&h, &d);
        assert!(!e.contains("error"), "and still compiles:\n{e}");
    }
}

#[test]
fn a_non_tuple_parenthesised_type_is_untouched() {
    // The negative control. `c_tuple_info` requires a comma, and a detector
    // that fired on any parenthesised type would rewrite things it must not.
    let (h, _d) = gen_c(
        "module P {\n    struct Pair { a : i32, b : i32, }\n    fn probe(p: Pair) -> i32 { return p.a; }\n}\n",
        "plain",
    );
    assert!(h.contains("int32_t probe(Pair p)"), "a plain struct parameter is unchanged:\n{h}");
    assert!(!h.contains("t27_tuple"), "and no tuple struct is invented:\n{h}");
}

#[test]
fn a_tuple_that_appears_ONLY_as_a_local_is_still_hoisted() {
    // Each position needs its own fixture. `ALL_THREE` uses one tuple type in
    // three places, so the typedef is collected from any one of them -- and a
    // mutant dropping LOCALS from the collection passed every other test here.
    // The subject has to be alone in the position under test.
    let (h, d) = gen_c(
        "module P {\n    fn probe(v: i32) -> i32 { var x : (u16, bool); return 0; }\n}\n",
        "localonly",
    );
    assert!(
        h.contains("} t27_tuple_uint16_t_bool;"),
        "a tuple used only as a local must still hoist its struct:\n{h}"
    );
    if cc_present() {
        let e = compile(&h, &d);
        assert!(!e.contains("error"), "and the header must compile:\n{e}");
    }
}

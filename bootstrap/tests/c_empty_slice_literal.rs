//! `var d = []u8{}` names its element type, and C was throwing it away.
//!
//! The empty slice literal reached C as `__auto_type d = { 0 }` -- the type
//! discarded AND the length changed from zero to one. **478 of these in the
//! specs**, the largest shape left in the `__auto_type` class.
//!
//! A slice is a pointer everywhere else in this backend, so an empty one is a
//! null pointer, which is also the only honest length: `T x[0]` is not ISO C,
//! and `T x[1] = {0}` would answer a question about emptiness with a one.
//!
//! THE ELEMENT TYPE MUST BE ONE C WILL KNOW. The first version trusted the
//! spec and emitted `u1* a = NULL` and `Port* ports = NULL` where no `u1` and
//! no `Port` are declared anywhere -- **two files got WORSE**, trading one
//! diagnostic for two. A lowering that names a type has to check the header
//! carries it.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-emptyslice-{tag}-{}-{}",
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
fn an_empty_slice_of_a_primitive_is_a_typed_null() {
    let (h, d) = gen_c(
        "module E {\n    fn f(v: i32) -> u32 { var d = []u8{}; return 0; }\n}\n",
        "prim",
    );
    assert!(h.contains("uint8_t* d = NULL;"), "got:\n{h}");
    assert!(!h.contains("__auto_type d"), "the type was on the literal all along:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn an_empty_slice_of_a_declared_struct_is_allowed() {
    let (h, d) = gen_c(
        "module E {\n    struct Sample { x : i32, }\n    fn f(v: i32) -> u32 { var d = []Sample{}; return 0; }\n}\n",
        "struct",
    );
    assert!(h.contains("Sample* d = NULL;"), "got:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn an_element_type_the_header_does_not_declare_is_refused() {
    // THE DISCRIMINATING CASE, and the reason two files got worse before it
    // existed. `u1` is not a C type and this module declares none, so naming it
    // trades one diagnostic for two. The loud `__auto_type` stays.
    let (h, _d) = gen_c(
        "module E {\n    fn f(v: i32) -> u32 { var a = []u1{}; return 0; }\n}\n",
        "unknown",
    );
    assert!(!h.contains("u1* a = NULL"), "an unknown element type must be refused:\n{h}");
    assert!(h.contains("__auto_type a"), "and the site stays as it was:\n{h}");
}

#[test]
fn an_undeclared_struct_element_is_refused_too() {
    // The other half of the same check: a NAME that looks like a type but which
    // this module never declares. `Port` in the corpus is exactly this -- only
    // `PortMap` exists.
    let (h, _d) = gen_c(
        "module E {\n    struct PortMap { x : i32, }\n    fn f(v: i32) -> u32 { var p = []Port{}; return 0; }\n}\n",
        "undeclared",
    );
    assert!(!h.contains("Port* p = NULL"), "an undeclared struct must be refused:\n{h}");
    assert!(h.contains("__auto_type p"), "and the site stays as it was:\n{h}");
}

#[test]
fn a_non_empty_slice_literal_still_becomes_an_array() {
    // The branch is for the EMPTY case only; the populated one already had an
    // answer and must keep it.
    let (h, d) = gen_c(
        "module E {\n    fn f(v: i32) -> u32 { var e = []i16{ 1, 2 }; return 0; }\n}\n",
        "nonempty",
    );
    assert!(h.contains("int16_t e[2] = { 1, 2 };"), "got:\n{h}");
    assert!(!h.contains("int16_t* e = NULL"), "a populated literal is not a null:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

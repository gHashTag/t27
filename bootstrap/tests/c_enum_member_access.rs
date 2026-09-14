//! `Trit.pos` is an ENUM MEMBER, and C has no `Type.member`.
//!
//! It went into C verbatim: **1373 errors across 12 files**, reported as
//! `unexpected type name 'Trit'` and as undeclared `POS`/`NEG`. The constant
//! `gen_c_enum` emits is `{TYPE}_{MEMBER}`, both upper-cased, so the two
//! spellings are brought together rather than a third one invented.
//!
//! Rust emits `Trit::pos` and Zig emits `Trit.pos`, each correct for its
//! language. C was the only backend without an answer -- the same shape as
//! every repair in this campaign.
//!
//! The discriminating case is a struct FIELD with the same name as an enum
//! member: `s.pos` must stay `s.pos`. The rewrite keys on the BASE being an
//! identifier that names a declared enum, not on the member's spelling.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-cenum-{tag}-{}-{}",
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
fn an_enum_member_becomes_the_c_constant() {
    let (h, d) = gen_c(
        "module P {\n    enum Trit { pos, neg, zero }\n    fn f(v: i32) -> i32 { var t = Trit.pos; return 0; }\n}\n",
        "member",
    );
    assert!(h.contains("__auto_type t = TRIT_POS;"), "got:\n{h}");
    assert!(!h.contains("Trit.pos"), "no dotted form may reach C:\n{h}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and it must compile");
    }
}

#[test]
fn the_constant_matches_what_the_enum_declaration_emits() {
    // Stated as an agreement rather than as the literal `TRIT_POS`: if the
    // naming rule in `gen_c_enum` ever changes, both must change together.
    let (h, _d) = gen_c(
        "module P {\n    enum Colour { red, green }\n    fn f(v: i32) -> i32 { var c = Colour.green; return 0; }\n}\n",
        "agree",
    );
    assert!(h.contains("COLOUR_GREEN,") || h.contains("COLOUR_GREEN\n"), "the declaration names it:\n{h}");
    assert!(h.contains("= COLOUR_GREEN;"), "and the use spells it the same way:\n{h}");
}

#[test]
fn a_struct_field_of_the_same_name_is_untouched() {
    // The discriminating case. A rewrite keyed on the MEMBER name would break
    // this; keying on the base being a declared enum does not.
    let (h, d) = gen_c(
        "module P {\n    struct S { pos : i32, }\n    enum Trit { pos, neg }\n    fn f(s: S) -> i32 { var t = Trit.pos; return s.pos; }\n}\n",
        "field",
    );
    assert!(h.contains("return s.pos;"), "a struct field stays a field:\n{h}");
    assert!(h.contains("= TRIT_POS;"), "and the enum member still converts:\n{h}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and it compiles");
    }
}

#[test]
fn a_field_access_on_a_variable_is_untouched() {
    // The base must be an identifier that NAMES A DECLARED ENUM. A variable
    // called `t` of enum type is not that, and `t.something` is not a member
    // constant.
    let (h, _d) = gen_c(
        "module P {\n    struct S { a : i32, }\n    fn f(s: S) -> i32 { return s.a; }\n}\n",
        "var",
    );
    assert!(h.contains("return s.a;"), "got:\n{h}");
    assert!(!h.contains("S_A"), "no enum-style constant may be invented:\n{h}");
}

#[test]
fn a_field_whose_name_is_an_enum_type_is_untouched() {
    // The case that makes the BASE check load-bearing rather than decorative.
    // A mutant keying on either side -- `contains(member) || contains(base)` --
    // survives every other test here, because no other test has a member whose
    // name is also an enum type. `s.Colour` must stay a field access.
    let (h, d) = gen_c(
        "module P {\n    enum Colour { red, green }\n    struct S { Colour : i32, }\n    fn f(s: S) -> i32 { return s.Colour; }\n}\n",
        "collide",
    );
    assert!(h.contains("return s.Colour;"), "a field named after an enum stays a field:\n{h}");
    assert!(!h.contains("S_COLOUR"), "and no constant is invented:\n{h}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and it compiles");
    }
}

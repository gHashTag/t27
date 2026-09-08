//! `TokenKind::KwFn` is the SAME enum-member reference as `Trit.pos`, spelled
//! with a path instead of a dot.
//!
//! One pass earlier the dotted spelling was fixed and the path spelling was
//! never asked about. It left **917 `X::Y` occurrences in the generated C**
//! across 65 files -- 538 of them naming an enum the translation unit itself
//! declares -- and C has no `::` in an expression at all.
//!
//! The two spellings now go through ONE helper, so a change to the naming rule
//! cannot reach one and miss the other.
//!
//! The discriminating case is a member the enum does NOT declare. 12 sites in
//! the corpus spell one (`Trit::TRUE` against an enum of pos/neg/zero).
//! Lowering those would emit a constant `gen_c_enum` never wrote, trading a
//! diagnostic that names the spec's mistake for one that hides it.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-cpath-{tag}-{}-{}",
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
fn a_path_spelled_enum_member_becomes_the_c_constant() {
    let (h, d) = gen_c(
        "module P {\n    enum TokenKind { KwFn, KwIf }\n    fn f(v: i32) -> i32 { var t = TokenKind::KwFn; return 0; }\n}\n",
        "path",
    );
    assert!(h.contains("__auto_type t = TOKENKIND_KWFN;"), "got:\n{h}");
    assert!(!h.contains("TokenKind::KwFn"), "no `::` may reach C:\n{h}");
    if cc_present() {
        assert!(!errors(&h, &d).contains("error"), "and it must compile");
    }
}

#[test]
fn both_spellings_produce_the_same_constant() {
    // The point of the shared helper. If these two ever diverge, one of the
    // call sites has been changed without the other -- which is exactly the
    // defect this test exists to prevent recurring.
    let (h, _d) = gen_c(
        "module P {\n    enum Colour { red, green }\n    fn f(v: i32) -> i32 { var a = Colour.green; var b = Colour::green; return 0; }\n}\n",
        "agree",
    );
    assert!(h.contains("__auto_type a = COLOUR_GREEN;"), "dotted:\n{h}");
    assert!(h.contains("__auto_type b = COLOUR_GREEN;"), "path:\n{h}");
}

#[test]
fn a_member_the_enum_does_not_declare_is_refused() {
    // THE DISCRIMINATING CASE. `TRIT_TRUE` is a constant `gen_c_enum` never
    // emitted, so writing it would replace a diagnostic that names the spec's
    // mistake with one that hides it. The site stays as it was written.
    let (h, _d) = gen_c(
        "module P {\n    enum Trit { pos, neg, zero }\n    fn f(v: i32) -> i32 { var t = Trit::TRUE; return 0; }\n}\n",
        "unknown-member",
    );
    assert!(!h.contains("TRIT_TRUE"), "no constant may be invented:\n{h}");
    assert!(h.contains("Trit::TRUE"), "the site is left as written:\n{h}");
}

#[test]
fn a_path_whose_base_is_not_an_enum_is_untouched() {
    // A module-qualified call is the other half of the `::` population (251 of
    // the 917 occurrences) and needs a DIFFERENT repair. Refusing it here is
    // what keeps the two apart.
    let (h, _d) = gen_c(
        "module P {\n    enum Colour { red }\n    fn f(v: i32) -> i32 { var x = other::helper(1); return 0; }\n}\n",
        "modcall",
    );
    assert!(!h.contains("OTHER_HELPER"), "a call is not a member constant:\n{h}");
    assert!(h.contains("other"), "and the callee is still named:\n{h}");
}

#[test]
fn a_path_whose_base_is_a_struct_is_untouched() {
    // A struct is not an enum, and `Point::origin` is not a member constant
    // even though it has the shape of one.
    let (h, _d) = gen_c(
        "module P {\n    struct Point { x : i32, }\n    enum Colour { red }\n    fn f(v: i32) -> i32 { var p = Point::origin; return 0; }\n}\n",
        "structbase",
    );
    assert!(!h.contains("POINT_ORIGIN"), "no constant for a struct path:\n{h}");
}

//! Locals declared with no type (gHashTag/t27#6995). The sources are the
//! conformance spec `specs/tri/t27b/conformance/untyped_local.t27` and the
//! cases below; each was run under `t27c test-report` first.

mod common;
use common::{names_ok, rejected, run};
use t27b::ir::TrapKind;

/// The conformance spec: `t27c test-report` gives 6 pass, 0 vacuous.
#[test]
fn conformance_spec_passes() {
    let src = include_str!("../../../specs/tri/t27b/conformance/untyped_local.t27");
    let ran = run(src);
    let got = names_ok(&ran);
    assert_eq!(got.len(), 6);
    assert!(got.iter().all(|(_, inv, ok)| !inv && *ok), "{:?}", got);
}

/// `var x = 4294967295;` is a `u32` in the reference, so one more overflows:
/// `t27c test-report` fails the test with "integer overflow".
#[test]
fn the_pinned_u32_overflows() {
    let src = r#"module e;

fn bump() -> u32 {
    var x = 4294967295;
    x = x + 1;
    return x;
}

test wraps {
    assert(bump() == 0);
}
"#;
    let ran = run(src);
    assert_eq!(ran.len(), 1);
    assert!(matches!(ran[0].2, Err((TrapKind::Overflow, _))), "{:?}", ran[0].2);
}

/// No width is pinned on a negative or a float literal, and Zig refuses a
/// `var` of a comptime type: both fail `t27c test-report`. t27b refuses both.
#[test]
fn a_negative_or_float_var_is_still_refused() {
    let neg = r#"module e;

fn down() -> i32 {
    var c = -1;
    c = c - 1;
    return c;
}

test t {
    assert(down() == -2);
}
"#;
    let r = rejected(neg);
    assert!(r.contains("StmtLocal") && r.contains("untyped integer"), "{}", r);
    let float = r#"module e;

fn half() -> f64 {
    var f = 1.5;
    f = f / 3.0;
    return f;
}

test t {
    assert(half() == 0.5);
}
"#;
    let r = rejected(float);
    assert!(r.contains("StmtLocal") && r.contains("untyped float"), "{}", r);
}

/// An untyped `undefined` const that is read has no defined value (the
/// reference runs it and reads whatever Zig left there), so t27b keeps
/// refusing it. One whose name is only inside a string literal is counted as
/// read by the reference's text scan, gets no `_ = c;`, and Zig refuses it
/// ("unused local constant"); t27b refuses it too.
#[test]
fn a_read_undefined_const_is_still_refused() {
    let read = r#"module e;

fn get() -> u32 {
    const u = undefined;
    return u;
}

test t {
    assert(get() == 0 or get() != 0);
}
"#;
    let r = rejected(read);
    assert!(r.contains("StmtLocal") && r.contains("neither type nor value"), "{}", r);
    let quoted = r#"module e;

fn label() -> []const u8 {
    const tag = undefined;
    return "tag";
}

test t {
    assert(label().len == 3);
}
"#;
    let r = rejected(quoted);
    assert!(r.contains("StmtLocal") && r.contains("neither type nor value"), "{}", r);
}

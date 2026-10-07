//! Array literals taken by address or as an empty slice. The sources are
//! the conformance spec `specs/tri/t27b/conformance/array_literal.t27` and a
//! refusal case; each was run under `t27c test-report` first.

mod common;
use common::{names_ok, rejected, run};

/// The conformance spec: `t27c test-report` gives 6 pass.
#[test]
fn conformance_spec_passes() {
    let src = include_str!("../../../specs/tri/t27b/conformance/array_literal.t27");
    let ran = run(src);
    let got = names_ok(&ran);
    assert_eq!(got.len(), 6);
    assert!(got.iter().all(|(_, inv, ok)| !inv && *ok), "{:?}", got);
}

/// Indexing `&[_]T{ ... }` reads a comptime element in the reference, so the
/// local is refused (`t27c test-report`: 1 pass).
#[test]
fn indexing_an_address_literal_is_refused() {
    let src = r#"module e;

fn total(xs: []const i64) -> i64 {
    var s: i64 = 0;
    for (xs) |x| {
        s += x;
    }
    return s;
}

test indexed {
    const ys = &[_]i64{ 2, 3 };
    assert(total(ys) == 5);
    assert(ys[0] == 2);
}
"#;
    let r = rejected(src);
    assert!(r.contains("ExprArrayLiteral") && r.contains("not addressable"), "{}", r);
}

/// The second conformance spec, array literals with no result type of their
/// own (`t27c test-report`: 6 pass, none vacuous).
#[test]
fn value_conformance_spec_passes() {
    let src = include_str!("../../../specs/tri/t27b/conformance/array_literal_value.t27");
    let ran = run(src);
    let got = names_ok(&ran);
    assert_eq!(got.len(), 6);
    assert!(got.iter().all(|(_, inv, ok)| !inv && *ok), "{:?}", got);
}

/// A returned literal of run-time values is a pointer into the callee's
/// frame in the reference, so it is refused (`t27c test-report`: 1 pass).
#[test]
fn a_run_time_slice_return_is_refused() {
    let src = r#"module e;

fn pair(a: f64) -> []const f64 {
    return [a, 2.0];
}

test runtime {
    const s = pair(1.0);
    assert(s.len == 2);
}
"#;
    let r = rejected(src);
    assert!(r.contains("ExprArrayLiteral(run-time slice return)"), "{}", r);
}

/// A constant returned as a mutable slice from a fn a test reaches is
/// refused: a write through it faults in the reference (`t27c test-report`:
/// 1 pass, as the test only reads its length).
#[test]
fn a_constant_returned_as_a_mutable_slice_is_refused() {
    let src = r#"module e;

fn buf() -> []f32 {
    return [0.0, 1.0];
}

test reached {
    assert(buf().len == 2);
}
"#;
    let r = rejected(src);
    assert!(r.contains("ExprArrayLiteral(constant to mutable slice)"), "{}", r);
}

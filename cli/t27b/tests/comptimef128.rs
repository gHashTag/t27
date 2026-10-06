//! Compile-time float arithmetic folded in binary128, as Zig folds
//! comptime_float (gHashTag/t27#7021). The sources are the conformance spec
//! `specs/tri/t27b/conformance/comptime_float_f128.t27` and the cases below;
//! each was run under `t27c test-report` first.

mod common;
use common::{names_ok, rejected, run};

/// The conformance spec: `t27c test-report` gives 10 pass, 0 vacuous.
#[test]
fn conformance_spec_passes() {
    let src = include_str!("../../../specs/tri/t27b/conformance/comptime_float_f128.t27");
    let ran = run(src);
    let got = names_ok(&ran);
    assert_eq!(got.len(), 10);
    assert!(got.iter().all(|(_, inv, ok)| !inv && *ok), "{:?}", got);
}

/// The shape of `specs/port/scripts/pysr_trinity_blind_test_v2.t27`: the
/// folded 1/p^3 rounds to a different f64 than the runtime one, and two
/// literals with one f64 (1 + 2^-24, an f32 midpoint) round to two f32s.
/// `t27c test-report` passes both tests.
#[test]
fn binary128_folds_and_f32_midpoints() {
    let src = r#"module e;

fn cube_inv(p: f64) -> f64 {
    return 1.0 / (p * p * p);
}

fn up() -> f32 {
    return 1.0000000596046447753906250001;
}

fn down() -> f32 {
    return 1.00000005960464477539062499;
}

test gamma_folds_in_binary128 {
    const p = 1.61803398875;
    const g = 1.0 / (p * p * p);
    assert(cube_inv(p) != g);
    assert(cube_inv(p) > g);
}

test f32_midpoints {
    assert(up() == 1.00000011920928955078125);
    assert(down() == 1.0);
}
"#;
    let ran = run(src);
    let got = names_ok(&ran);
    assert_eq!(got, vec![("gamma_folds_in_binary128", false, true), ("f32_midpoints", false, true)]);
}

/// A folded value that overflows f64, and a constant division by zero,
/// stay refused.
#[test]
fn overflow_and_division_by_zero_are_still_refused() {
    let over = r#"module e;

fn huge() -> f64 {
    return 1e300 * 1e300;
}

test t {
    assert(huge() > 0.0);
}
"#;
    let r = rejected(over);
    assert!(r.contains("ExprBinary") && r.contains("overflows f64"), "{}", r);
    let zero = r#"module e;

fn inf() -> f64 {
    return 1.5 / 0.0;
}

test t {
    assert(inf() > 0.0);
}
"#;
    let r = rejected(zero);
    assert!(r.contains("ExprBinary") && r.contains("division by zero"), "{}", r);
}

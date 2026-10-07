//! `@intCast(x)` with a result type, as t27c's Zig backend prints it. Each
//! source here was run under `t27c test-report` (the reference path) first:
//! same pass/fail per test. A value outside the result type is a safety
//! panic in Zig's Debug build, so each such test fails there; here it traps
//! at the cast. The passing cases are in
//! specs/tri/t27b/conformance/int_cast.t27.

mod common;
use common::{rejected, run};
use t27b::ir::TrapKind;

/// `t27c test-report`: `fits` passes, the other 6 fail. Every failing assert
/// expects the wrapped value, so a truncating cast would pass them.
#[test]
fn out_of_range_traps() {
    let src = r#"module a;

fn narrow(x: u32) -> u16 {
    const r: u16 = @intCast(x);
    return r;
}

fn to_i32(x: i64) -> i32 {
    return @as(i32, @intCast(x));
}

fn to_u64(x: i64) -> u64 {
    return @as(u64, @intCast(x));
}

fn to_i64(x: u64) -> i64 {
    return @as(i64, @intCast(x));
}

fn to_u32(x: i32) -> u32 {
    const r: u32 = @intCast(x);
    return r;
}

test fits {
    assert(narrow(65535) == 65535);
    assert(to_i32(-2147483648) == -2147483648);
    assert(to_u64(9223372036854775807) == 9223372036854775807);
}

test unsigned_narrowing_out_of_range {
    assert(narrow(65536) == 0);
}

test signed_narrowing_below {
    assert(to_i32(-2147483649) == 2147483647);
}

test signed_narrowing_above {
    assert(to_i32(2147483648) == -2147483648);
}

test negative_to_unsigned {
    assert(to_u64(-1) == 18446744073709551615);
}

test unsigned_to_signed_too_big {
    assert(to_i64(9223372036854775808) == -9223372036854775808);
}

test negative_to_u32 {
    assert(to_u32(-1) == 4294967295);
}
"#;
    let r = run(src);
    let got: Vec<(&str, Option<(TrapKind, u32)>)> = r.iter().map(|(n, _, o)| (n.as_str(), o.err())).collect();
    assert_eq!(
        got,
        vec![
            ("fits", None),
            ("unsigned_narrowing_out_of_range", Some((TrapKind::Cast, 4))),
            ("signed_narrowing_below", Some((TrapKind::Cast, 9))),
            ("signed_narrowing_above", Some((TrapKind::Cast, 9))),
            ("negative_to_unsigned", Some((TrapKind::Cast, 13))),
            ("unsigned_to_signed_too_big", Some((TrapKind::Cast, 17))),
            ("negative_to_u32", Some((TrapKind::Cast, 21))),
        ]
    );
}

/// A literal operand that does not fit: Zig refuses it at compile time.
#[test]
fn literal_out_of_range_is_refused() {
    let src = r#"module a;

fn id(x: u8) -> u8 {
    return x;
}

test big {
    const b: u8 = @intCast(300);
    assert(id(b) == 44);
}
"#;
    let r = rejected(src);
    assert!(r.contains("ExprCall(@intCast)") && r.contains("does not fit"), "{}", r);
}

/// No result type: Zig refuses `@intCast` there, and so does t27b.
#[test]
fn no_result_type_is_refused() {
    let src = r#"module a;

fn id(x: u64) -> u64 {
    return x;
}

test untyped {
    const b = @intCast(id(3));
    assert(b == 3);
}
"#;
    let r = rejected(src);
    assert!(r.contains("@intCast"), "{}", r);
}

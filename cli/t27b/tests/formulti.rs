//! `for (xs, ys) |x, y|`: one hidden index over several objects whose
//! lengths must be equal. Each source here was run under `t27c test-report`
//! (the reference path) first: same pass/fail per test.

mod common;
use common::{rejected, run};
use t27b::ir::TrapKind;

/// `t27c test-report`: 4 pass, `unequal_slices_panics` fails (Zig's panic
/// "for loop over objects with non-equal lengths").
#[test]
fn several_objects_walk_together() {
    let src = r#"module a;

fn dot(xs: []const i64, ys: []const i64) -> i64 {
    var s: i64 = 0;
    for (xs, ys) |x, y| {
        s += x * y;
    }
    return s;
}

fn mixed(xs: [3]i64, ys: []const i64) -> i64 {
    var s: i64 = 0;
    for (xs, ys) |x, _| {
        s += x;
    }
    return s;
}

test equal_slices {
    const a: [3]i64 = [1, 2, 3];
    const b: [3]i64 = [4, 5, 6];
    assert(dot(&a, &b) == 32);
}

test arrays_three {
    const a: [2]i64 = [1, 2];
    const b: [2]i64 = [3, 4];
    const c: [2]i64 = [5, 6];
    var s: i64 = 0;
    for (a, b, c) |x, y, z| {
        s += x * y + z;
    }
    assert(s == 22);
}

test mixed_ok {
    const a: [3]i64 = [1, 2, 3];
    const b: [3]i64 = [0, 0, 0];
    assert(mixed(a, &b) == 6);
}

test unequal_slices_panics {
    const a: [3]i64 = [1, 2, 3];
    const b: [2]i64 = [4, 5];
    assert(dot(&a, &b) == 0);
}

test after_unequal {
    assert(1 == 1);
}
"#;
    let got: Vec<_> = run(src).into_iter().map(|(n, _, o)| (n, o.map_err(|(k, _)| k))).collect();
    assert_eq!(
        got,
        vec![
            ("equal_slices".to_string(), Ok(())),
            ("arrays_three".to_string(), Ok(())),
            ("mixed_ok".to_string(), Ok(())),
            ("unequal_slices_panics".to_string(), Err(TrapKind::ForLength)),
            ("after_unequal".to_string(), Ok(())),
        ]
    );
}

/// Two different comptime lengths: `t27c test-report` is BLOCKED ("non-matching
/// for loop lengths"), so t27b refuses the loop.
#[test]
fn unequal_comptime_lengths_are_refused() {
    let src = r#"module a;

test unequal_arrays {
    const a: [3]i64 = [1, 2, 3];
    const b: [2]i64 = [4, 5];
    var s: i64 = 0;
    for (a, b) |x, y| {
        s += x + y;
    }
    assert(s == 0);
}
"#;
    let r = rejected(src);
    assert!(r.contains("StmtFor(multi-object)") && r.contains("lengths 3 and 2 differ"), "{}", r);
}

/// Captures must match the iterables one to one; a range among them is
/// refused.
#[test]
fn capture_count_and_ranges_are_refused() {
    let r = rejected("module a;\n\ntest t {\n    const a: [2]i64 = [1, 2];\n    for (a, a) |x| {\n        assert(x > 0);\n    }\n}\n");
    assert!(r.contains("2 iterables but 1 captures"), "{}", r);
}

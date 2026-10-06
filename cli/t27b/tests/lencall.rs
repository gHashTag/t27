//! `x.len()`, `a.b.len()` and `len(x)`: the `.len` field, as t27c's Zig
//! backend prints them. Each source here was run under `t27c test-report`
//! (the reference path) first: same pass/fail per test.

mod common;
use common::{names_ok, run};

/// `t27c test-report`: 3 pass, `failing_len` fails.
#[test]
fn len_calls_are_the_len_field() {
    let src = r#"module a;

struct Buf {
    data: []const u8,
    n: u32,
}

fn count(xs: [4]i64) -> usize {
    return xs.len();
}

fn arr_len() -> usize {
    const a: [3]u8 = [1, 2, 3];
    return a.len();
}

test method_len {
    const xs: [4]i64 = [4, 5, 6, 7];
    assert(count(xs) == 4);
    assert(arr_len() == 3);
}

test nested_len {
    const b = Buf{ .data = "hello", .n = 1 };
    assert(b.data.len() == 5);
}

test free_len {
    const s: []const u8 = "abc";
    assert(len(s) == 3);
}

test failing_len {
    const s: []const u8 = "abcd";
    assert(s.len() == 3);
}
"#;
    assert_eq!(
        names_ok(&run(src)),
        vec![
            ("method_len", false, true),
            ("nested_len", false, true),
            ("free_len", false, true),
            ("failing_len", false, false),
        ]
    );
}

/// A declared fn `len` is called, not read as a field (`t27c test-report`: 1 pass).
#[test]
fn a_declared_len_fn_is_called() {
    let src = r#"module a;

fn len(x: u32) -> u32 {
    return x + 1;
}

test declared_len_wins {
    assert(len(4) == 5);
}
"#;
    assert_eq!(names_ok(&run(src)), vec![("declared_len_wins", false, true)]);
}

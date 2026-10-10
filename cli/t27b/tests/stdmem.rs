//! `std.mem.eql(u8, a, b)` and `std.mem.indexOf(u8, h, n)` compared with
//! `null`, on strings. Each source here was run under `t27c test-report`
//! (the reference path): same pass/fail per test, and the refused shapes
//! stay refused by name.

mod common;
use common::{names_ok, rejected, run};

const SRC: &str = "module a;

fn has(h: []const u8, n: []const u8) -> bool {
    return std.mem.indexOf(u8, h, n) != null;
}

fn same(a: []const u8, b: []const u8) -> bool {
    return std.mem.eql(u8, a, b);
}

test found_cases {
    assert(has(\"hello world\", \"world\"));
    assert(has(\"hello world\", \"hello\"));
    assert(has(\"hello\", \"\"));
    assert(has(\"\", \"\"));
    assert(has(\"abcabd\", \"abd\"));
    assert(has(\"aaab\", \"aab\"));
}

test missing_cases {
    assert(!has(\"hello\", \"hello!\"));
    assert(!has(\"\", \"a\"));
    assert(std.mem.indexOf(u8, \"abc\", \"abd\") == null);
    assert(!has(\"abcab\", \"abd\"));
}

test eql_cases {
    assert(same(\"abc\", \"abc\"));
    assert(!same(\"abc\", \"abd\"));
    assert(!same(\"abc\", \"ab\"));
    assert(same(\"\", \"\"));
    assert(std.mem.eql(u8, \"x\", \"x\"));
}

test failing_found {
    assert(has(\"hello\", \"xyz\"));
}

test failing_eql {
    assert(same(\"a\", \"b\"));
}
";

/// `t27c test-report`: 3 pass, 2 fail (the last two).
#[test]
fn std_mem_eql_and_index_of_null() {
    assert_eq!(
        names_ok(&run(SRC)),
        vec![
            ("found_cases", false, true),
            ("missing_cases", false, true),
            ("eql_cases", false, true),
            ("failing_found", false, false),
            ("failing_eql", false, false),
        ]
    );
}

#[test]
fn std_mem_other_shapes_are_refused() {
    let head = "module a;\n\n";
    let cases: &[(&str, &str)] = &[
        ("test t { const p = std.mem.indexOf(u8, \"ab\", \"b\"); assert(p != null); }", "call to `std.mem.indexOf`"),
        ("test t { assert(std.mem.eql(u32, \"ab\", \"ab\")); }", "other than (u8, str, str)"),
        ("test t { const x: u32 = 1; assert(std.mem.eql(u8, x, \"ab\")); }", "not a string"),
    ];
    for (body, detail) in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with("t27b: unsupported construct ExprCall(std.*) at line"), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

/// The conformance spec `specs/tri/t27b/conformance/std_mem_byte_slice.t27`:
/// `t27c test-report` gives 6 pass, 0 vacuous.
#[test]
fn byte_buffer_conformance_spec_passes() {
    let src = include_str!("../../../specs/tri/t27b/conformance/std_mem_byte_slice.t27");
    let ran = run(src);
    let got = names_ok(&ran);
    assert_eq!(got.len(), 6);
    assert!(got.iter().all(|(_, inv, ok)| !inv && *ok), "{:?}", got);
}

/// Bytes in a buffer compare by content, so a wrong expectation fails
/// (`t27c test-report`: 1 pass, 2 fail).
const BUF_SRC: &str = "module b;

fn put(buf: []u8, pos: usize, s: []const u8) -> usize {
    var i: usize = 0;
    while (i < s.len) {
        buf[pos + i] = s[i];
        i += 1;
    }
    return pos + s.len;
}

test matches {
    var buf: [8]u8 = undefined;
    const n: usize = put(&buf, 0, \"abc\");
    assert(std.mem.eql(u8, buf[0:n], \"abc\"));
}

test wrong_bytes {
    var buf: [8]u8 = undefined;
    const n: usize = put(&buf, 0, \"abc\");
    assert(std.mem.eql(u8, buf[0:n], \"abd\"));
}

test wrong_whole_array {
    var buf: [2]u8 = undefined;
    const n: usize = put(&buf, 0, \"ab\");
    assert(n == 2);
    assert(std.mem.eql(u8, &buf, \"ba\"));
}
";

#[test]
fn byte_buffers_compare_by_content() {
    assert_eq!(
        names_ok(&run(BUF_SRC)),
        vec![("matches", false, true), ("wrong_bytes", false, false), ("wrong_whole_array", false, false)]
    );
}

/// Only bytes coerce to a string: a slice of, or a pointer to, any other
/// element type stays refused.
#[test]
fn non_byte_buffers_are_refused() {
    let head = "module a;\n\n";
    let cases: &[&str] = &[
        "test t { var w: [4]u32 = undefined; w[0] = 1; assert(std.mem.eql(u8, w[0:1], \"a\")); }",
        "test t { var w: [4]u32 = undefined; w[0] = 1; assert(std.mem.eql(u8, &w, \"a\")); }",
    ];
    for body in cases {
        let m = rejected(&format!("{}{}\n", head, body));
        assert!(m.starts_with("t27b: unsupported construct ExprCall(std.*) at line"), "{}: {}", body, m);
        assert!(m.contains("not a string"), "{}: {}", body, m);
    }
}

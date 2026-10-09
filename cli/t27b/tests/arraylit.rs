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

/// A write through a slice a static literal backs is undefined behaviour in
/// Zig (x86_64 Debug keeps it, LLVM faults), so it is refused at the write
/// (#7765; t27c's parser gives an assignment the line after its `;`): through
/// a callee's parameter, a struct element, an address, a slice of it. `set`
/// counts only where a test calls it. Another element type's slice is written.
#[test]
fn a_write_through_a_static_literal_is_refused() {
    let head = "module w;\n\nconst Hop = struct {\n    to: usize,\n    w: i64,\n};\n\nfn order() -> []usize {\n    return [0, 1, 2, 3];\n}\n\nfn hops() -> []Hop {\n    return []Hop{ .{ .to = 1, .w = 4 } };\n}\n\nfn set(xs: []usize, v: usize) -> void {\n    xs[0] = v;\n}\n\ntest t {\n";
    let cases = [
        ("    set(order(), 9);\n    assert(order()[0] == 9);\n}\n", 18),
        ("    var hs = hops();\n    hs[0].w += 40;\n    assert(hops()[0].w == 44);\n}\n", 23),
        ("    const p = &hops()[0];\n    p.w = 1;\n}\n", 21),
        ("    const q = order()[1..];\n    q[0] = 7;\n}\n", 23),
    ];
    for (body, line) in cases {
        let r = rejected(&format!("{}{}", head, body));
        assert!(r.contains("StmtAssign(write through an array literal)") && r.contains(&format!("line {}", line)), "{}", r);
    }
    let other = "    var b: [2]u8 = [0, 0];\n    const s: []u8 = b[0..];\n    s[1] = 3;\n    assert(order()[1] == 1 and b[1] == 3);\n}\n";
    assert_eq!(names_ok(&run(&format!("{}{}", head, other))), vec![("t", false, true)]);
}

/// Constants behind a slice field or a returned slice, mutable or not, read
/// from one static per type and value, as the reference interns them
/// (`t27c test-report`: 5 pass, none vacuous, on x86_64 and aarch64).
#[test]
fn static_conformance_spec_passes() {
    let ran = run(include_str!("../../../specs/tri/t27b/conformance/static_slice_literal.t27"));
    let got = names_ok(&ran);
    assert_eq!(got.len(), 5);
    assert!(got.iter().all(|(_, inv, ok)| !inv && *ok), "{:?}", got);
}

/// `.{}` by its result type: struct defaults, and `&.{}` as an empty slice
/// (#7735; `t27c test-report`: 4 pass, none vacuous).
#[test]
fn empty_anon_literal_spec_passes() {
    let ran = run(include_str!("../../../specs/tri/t27b/conformance/empty_anon_literal.t27"));
    let got = names_ok(&ran);
    assert_eq!(got.len(), 4);
    assert!(got.iter().all(|(_, inv, ok)| !inv && *ok), "{:?}", got);
}

/// This frame's address stored through a parameter: the reference passes it
/// only by reading a dead frame, so it is refused by name (#7735).
#[test]
fn frame_address_store_is_refused() {
    let r = rejected(include_str!("../../../specs/tri/t27b/conformance/frame_address_store.t27"));
    assert!(r.contains("StmtAssign(frame address)") && r.contains("`cell`"), "{}", r);
}

/// An untyped list local is a Zig tuple (#8050): read at constant indices,
/// and at run-time ones inside an invariant (`t27c test-report`: 2 pass).
#[test]
fn tuple_local_index_spec_passes() {
    let ran = run(include_str!("../../../specs/tri/t27b/conformance/tuple_local_index.t27"));
    let got = names_ok(&ran);
    assert_eq!(got.len(), 3);
    assert!(got.iter().all(|(_, _, ok)| *ok), "{:?}", got);
}

/// A run-time index into one in a test: Zig refuses it, so t27b does too.
#[test]
fn tuple_local_run_time_index_is_refused() {
    let r = rejected("module a;\n\nconst A : i32 = 4;\n\ntest t {\n    const v = [_]i32{A, A};\n    var i: usize = 1;\n    assert(v[i] == 4);\n}\n");
    assert!(r.contains("ExprIndex(tuple)"), "{}", r);
}

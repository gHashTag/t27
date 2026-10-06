//! The tail of single-file blockers (epic #6063): one test per construct
//! family, each checked against `t27c test-report` (the reference path) for
//! both what runs and what is refused.

mod common;
use common::{names_ok, rejected, run};
use t27b::ir::TrapKind;

// ------------------------------------------------------- type undefined

/// A port's `-> undefined` stub on a fn nothing analyzed reaches: the
/// reference compiles the file (Zig never resolves that return type) and
/// runs every test; here the fn lowers with no result and its `undefined;`
/// body is a trap no test reaches. `t27c test-report`: 1 pass.
#[test]
fn undefined_return_type_on_an_unreached_fn() {
    let src = "module a;\n\npub fn main() -> undefined {\n    undefined;\n}\n\nfn helper(x: u32) -> undefined {\n    main();\n    undefined;\n}\n\nfn twice(x: u32) -> u32 {\n    return x * 2;\n}\n\ntest t {\n    assert(twice(2) == 4);\n}\n";
    assert_eq!(names_ok(&run(src)), vec![("t", false, true)]);
}

/// The same stub called from a test: the reference's Zig does not compile
/// it (BLOCKED, "use of undefined value"), so it stays refused.
#[test]
fn undefined_return_type_on_a_reached_fn_is_refused() {
    let src = "module a;\n\nfn stub() -> undefined {\n    undefined;\n}\n\ntest t {\n    stub();\n}\n";
    let m = rejected(src);
    assert!(m.starts_with("t27b: unsupported construct type undefined at line 3"), "{}", m);
}

// ------------------------------------------- brace-invariant predicates

/// #6315: t27c emits each top-level statement of a brace invariant that is a
/// predicate (a binary or unary expression, a name, an index, a field access,
/// `true`/`false`, a call to a fn declared `-> bool`) as `assert(<expr>)`
/// inside the invariant's `comptime` block. A void call stays an action and
/// an `assert` stays an assert. Without `broken` and `lit_false`,
/// `t27c test-report` proves every invariant here; with either, its comptime
/// assert fails the reference's compile, and here that invariant fails.
/// specs/tri/t27b/conformance/brace_invariant.t27 is the dogfood spec.
#[test]
fn brace_invariant_predicates_are_asserted() {
    let src = "module bi;\n\nconst MAX: u32 = 9;\nconst ON: bool = true;\nconst FLAGS: [3]bool = [false, true, true];\n\nfn ready() -> bool {\n    return true;\n}\n\nfn note() {\n    var k: u32 = 0;\n    k += 1;\n}\n\ninvariant cmp { MAX * 2 == 18 }\n\ninvariant name { ON }\n\ninvariant neg { !FLAGS[0] }\n\ninvariant idx { FLAGS[1] }\n\ninvariant lit { true }\n\ninvariant call { ready() }\n\ninvariant many {\n    note();\n    MAX > 8;\n    assert(MAX != 0);\n    MAX < 10;\n}\n\ninvariant broken {\n    MAX > 8;\n    MAX == 8;\n    MAX < 10;\n}\n\ninvariant lit_false { false }\n";
    let r = run(src);
    assert_eq!(
        names_ok(&r),
        vec![
            ("cmp", true, true),
            ("name", true, true),
            ("neg", true, true),
            ("idx", true, true),
            ("lit", true, true),
            ("call", true, true),
            ("many", true, true),
            ("broken", true, false),
            ("lit_false", true, false),
        ]
    );
    // The front-end gives an invariant's statements no line of their own
    // (line 0), so the trap names the invariant's header line.
    assert_eq!(r[7].2, Err((TrapKind::Assert, 35)));
    assert_eq!(r[8].2, Err((TrapKind::Assert, 41)));
}

/// What the reference wraps but its Zig does not compile (BLOCKED under
/// `t27c test-report`), and what it does not wrap at all, stays refused: a
/// non-bool predicate, a bare non-bool literal, and a predicate nested in an
/// `if` (only top-level statements are wrapped). A call to a fn returning a
/// non-bool value is not a predicate either; the reference discards its
/// value, which is the `ExprCall(value ignored)` family, not this one.
#[test]
fn brace_invariant_non_predicates_are_refused() {
    let cases = [
        ("invariant i { N + 1 }\n", "t27b: unsupported construct condition at line"),
        ("invariant i { 3 }\n", "t27b: unsupported construct ExprLiteral(value ignored) statement at line"),
        ("invariant i {\n    if (N == 3) {\n        N == 3;\n    }\n}\n", "t27b: unsupported construct ExprBinary(value ignored) statement at line"),
    ];
    for (body, want) in cases {
        let m = rejected(&format!("module bn;\n\nconst N: u32 = 3;\n\n{}", body));
        assert!(m.starts_with(want), "{}: {}", body, m);
    }
}

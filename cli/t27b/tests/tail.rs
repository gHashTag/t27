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

// ------------------------------------------------ tail expressions

/// #6315: t27c's Zig backend returns the last statement of a non-void fn
/// when it is a bare expression (`zig_tail_returns`), and through an if/else
/// that is last, the last statement of each branch, at any depth.
/// `t27c test-report`: `tails` passes and `wrong_tail` fails;
/// specs/tri/t27b/conformance/value_ignored.t27 is the dogfood spec.
#[test]
fn tail_expressions_are_returned() {
    let src = "module tr;\n\nconst K: u32 = 40;\n\nfn low(w: u32) -> u32 {\n    w & 15\n}\n\nfn name(w: u32) -> u32 {\n    w\n}\n\nfn konst() -> u32 {\n    K\n}\n\nfn pick(w: u32) -> u32 {\n    if w == 0 {\n        7\n    } else {\n        if w < 10 {\n            let d: u32 = w * 2;\n            d\n        } else {\n            low(w)\n        }\n    }\n}\n\nfn small(w: u32) -> bool {\n    w < 10\n}\n\ntest tails {\n    assert(low(255) == 15);\n    assert(name(9) == 9);\n    assert(konst() == 40);\n    assert(pick(0) == 7);\n    assert(pick(4) == 8);\n    assert(pick(31) == 15);\n    assert(small(3));\n    assert(!small(30));\n}\n\ntest wrong_tail {\n    assert(pick(4) == 4);\n}\n";
    assert_eq!(names_ok(&run(src)), vec![("tails", false, true), ("wrong_tail", false, false)]);
}

// ------------------------------------------------ @intCast out of range

/// #7412: int_cast_traps.t27 names each test's verdict as fuzz_oracle.t27 reads it; an `_expect_trap` test
/// traps at an `@intCast` line, interpreter and JIT alike, where Zig's Debug build panics (`t27c
/// test-report` fails exactly those). A literal that does not fit, or no result type: refused, as by Zig.
#[test]
fn int_cast_traps_outside_its_result_type() {
    let src = include_str!("int_cast_traps.t27");
    let r = run(src);
    for (name, _, o) in &r {
        let at = o.err().map(|(k, l)| (k, src.lines().nth(l as usize - 1).is_some_and(|s| s.contains("@intCast"))));
        assert_eq!(at, name.ends_with("_expect_trap").then_some((TrapKind::Cast, true)), "{}", name);
    }
    assert_eq!(r.iter().filter(|t| t.2.is_err()).count(), 7);
    let lit = "module a;\n\nfn id(x: u64) -> u64 {\n    return x;\n}\n\ntest t {\n    const b: u8 = @intCast(300);\n    assert(b == 44);\n}\n";
    assert!(rejected(lit).contains("ExprCall(@intCast) at line 8 (`@intCast` of 300 to u8: the literal does not fit"));
    assert!(rejected(&lit.replace("b: u8 = @intCast(300)", "b = @intCast(id(3))")).contains("ExprCall(@intCast) at line 8"));
}

// ------------------------------------------------------------- @exp

/// #7391: `@exp` of an f64 calls libm.t27's exp under t27b's reserved name, so a file's own `t27b_libm_exp` is not
/// the one called, and the bits are compiler_rt's (one ulp above e) whether Zig folds the call or runs it. A
/// literal, an integer and two operands are refused (specs/tri/t27b/libm_plan.t27).
#[test]
fn exp_calls_compiler_rt_s_exp_written_in_t27() {
    let src = "module a;\n\nfn t27b_libm_exp(x: f64) -> f64 {\n    return x;\n}\n\nfn e(x: f64) -> f64 {\n    return @exp(x);\n}\n\ntest t {\n    const one: f64 = 1.0;\n    assert(e(1.0) == 2.7182818284590455);\n    assert(@exp(one) == e(1.0));\n    assert(t27b_libm_exp(1.0) == 1.0);\n}\n";
    assert_eq!(names_ok(&run(src)), vec![("t", false, true)]);
    for (arg, why) in [("1.0", "is a literal"), ("@as(i32, 1)", "not an f64"), ("x, x", "one operand")] {
        let m = rejected(&src.replace("@exp(x)", &format!("@exp({})", arg)));
        assert!(m.starts_with("t27b: unsupported construct ExprCall(@exp) at line 8") && m.contains(why), "{}", m);
    }
}

/// #7217: `@log` of an f64 calls libm.t27's log, compiler_rt's table-driven routine, bit for bit; of an f32 it is
/// refused, libm.t27 holding no logf yet (specs/tri/t27b/libm_plan.t27).
#[test]
fn log_calls_compiler_rt_s_log_written_in_t27() {
    let src = "module a;\n\nfn l(x: f64) -> f64 {\n    return @log(x);\n}\n\ntest t {\n    assert(l(2.0) == 0.6931471805599453);\n    assert(l(5.0e-324) == -744.4400719213812);\n}\n";
    assert_eq!(names_ok(&run(src)), vec![("t", false, true)]);
    let m = rejected(&src.replace("x: f64) -> f64", "x: f32) -> f32"));
    assert!(m.starts_with("t27b: unsupported construct ExprCall(@log) at line 4") && m.contains("no f32 routine"), "{}", m);
}

// ------------------------------------------------ integer constants wider than 64 bits

/// #7423: a wide constant folds where Zig folds it (specs/tri/t27b/wide_plan.t27), as under `t27c test-report`: a
/// comparison, an `as` back to 64 bits, an unused constant, an unreached signature. What Zig refuses, or what
/// needs a wide value at run time, is refused under the wide type's name.
#[test]
fn wide_integer_constants_fold_or_are_refused() {
    let head = "module a;\n\nconst A: u128 = 340282366920938463463374607431768211455;\nconst B: u256 = 18446744073709551616;\nconst C: i128 = 5;\nconst D: u128 = 0 - 1;\n\nfn g(x: u128) -> u256 {\n    return x;\n}\n\n";
    let t = |body: &str| format!("{}test t {{\n    assert({});\n}}\n", head, body);
    assert_eq!(names_ok(&run(&t("B < A and (A >> 64) as u64 == 18446744073709551615 and (0 - C) as i8 == -5"))), vec![("t", false, true)]);
    let h = format!("{}fn h(x: u64) -> bool {{\n    return A > x;\n}}\n", head);
    for (src, ty, why) in [(t("A + 1 > 0"), "u128", "overflows"), (t("B as u64 == 0"), "u256", "does not fit"), (t("A << 128 == 0"), "u128", "shift amount"),
        (t("C / 0 == 1"), "i128", "by zero"), (t("A + B > 0"), "u128", "two different"), (t("(0 - C) / 2 < 0"), "i128", "negative"),
        (t("D > 0"), "u128", "does not fit"), (h, "u128", "beside a value"), (t("g(1) == 0"), "u128", "")] {
        let m = rejected(&src);
        assert!(m.contains(&format!("type {} at line", ty)) && m.contains(why), "{}: {}", src, m);
    }
}

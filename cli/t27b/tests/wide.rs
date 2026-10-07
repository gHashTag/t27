//! Integer constants wider than 64 bits (`u128`, `i128`, `u256`, `u1024`),
//! folded at compile time as Zig folds them. Each source here was run under
//! `t27c test-report` (the reference path): same pass/fail per test, and
//! what Zig's compiler refuses, or what t27b cannot hold at run time, stays
//! refused by name.

mod common;
use common::{names_ok, rejected, run};

const SPEC: &str = include_str!("../../../specs/tri/t27b/conformance/wide_int_const.t27");

/// `t27c test-report`: 4 pass, 0 vacuous.
#[test]
fn conformance_spec_passes() {
    let ran = run(SPEC);
    let got = names_ok(&ran);
    assert_eq!(
        got,
        vec![
            ("u128_constants_fold_exactly", false, true),
            ("u128_holds_all_128_bits", false, true),
            ("i128_spans_both_signs", false, true),
            ("u256_and_u1024_fold_past_128_bits", false, true),
        ]
    );
}

/// A wrong expectation fails the test it is in, so each folded value is
/// checked, not only accepted (`t27c test-report` fails the same test).
#[test]
fn wrong_expectations_fail() {
    let mutants: &[(&str, &str, &str)] = &[
        ("wc_top_limb() == 12972", "wc_top_limb() == 12973", "u128_constants_fold_exactly"),
        ("wc_low_limb() == 8165106457226027330", "wc_low_limb() == 8165106457226027331", "u128_constants_fold_exactly"),
        ("WC_HALF + WC_HALF == WC_FULL", "WC_HALF + WC_HALF != WC_FULL", "u128_constants_fold_exactly"),
        ("return WC_HALF < WC_FULL;", "return WC_HALF > WC_FULL;", "u128_constants_fold_exactly"),
        ("wc_top_bits() == 18446744073709551615", "wc_top_bits() == 18446744073709551614", "u128_holds_all_128_bits"),
        ("wc_hex_head() == 65535", "wc_hex_head() == 65534", "u128_holds_all_128_bits"),
        ("wc_wraps_left() == 15", "wc_wraps_left() == 31", "u128_holds_all_128_bits"),
        ("return WC_MIN < 0;", "return WC_MIN > 0;", "i128_spans_both_signs"),
        ("wc_min_plus_max() == 0 - 1", "wc_min_plus_max() == 0", "i128_spans_both_signs"),
        ("wc_256_ratio() == 2", "wc_256_ratio() == 3", "u256_and_u1024_fold_past_128_bits"),
        ("wc_256_digits() == 324481", "wc_256_digits() == 324482", "u256_and_u1024_fold_past_128_bits"),
        ("wc_1024_top() == 1024", "wc_1024_top() == 512", "u256_and_u1024_fold_past_128_bits"),
        ("WC_256 * 3 - WC_256 == WC_256_TWICE", "WC_256 * 3 - WC_256 != WC_256_TWICE", "u256_and_u1024_fold_past_128_bits"),
    ];
    for (from, to, test) in mutants {
        assert_eq!(SPEC.matches(from).count(), 1, "{}", from);
        let ran = run(&SPEC.replace(from, to));
        for (name, _, ok) in names_ok(&ran) {
            assert_eq!(ok, name != *test, "`{}` -> `{}`: test {}", from, to, name);
        }
    }
}

const HEAD: &str = "module a;\n\nconst A: u128 = 340282366920938463463374607431768211455;\nconst B: u256 = 18446744073709551616;\nconst C: i128 = 5;\n\n";

fn refused(cases: &[(&str, &str, &str)]) {
    for (body, construct, detail) in cases {
        let m = rejected(&format!("{}{}\n", HEAD, body));
        assert!(m.contains(&format!("construct {} at line", construct)), "{}: {}", body, m);
        assert!(m.contains(detail), "{}: {}", body, m);
    }
}

/// Zig's compiler refuses these (`t27c test-report`: blocked, "does not
/// compile"): an overflow, a value outside its type, an `@intCast` that does
/// not fit, a shift by the full width, a division by zero.
#[test]
fn what_zig_refuses_is_refused() {
    refused(&[
        ("test t { assert(A + 1 > 0); }", "ExprBinary", "overflows u128"),
        ("test t { assert(A - A - 1 < 0); }", "ExprBinary", "overflows u128"),
        ("test t { assert(B as u64 == 0); }", "literal out of range", "does not fit in u64"),
        ("test t { assert((A >> 64) as u32 == 0); }", "literal out of range", "does not fit in u32"),
        ("test t { assert(A << 128 == 0); }", "type u128", "by 128"),
        ("test t { assert(C / 0 == 1); }", "ExprBinary", "division by zero"),
        ("fn f() -> u64 { return A; }\ntest t { assert(f() > 0); }", "type u128", ""),
    ]);
    let m = rejected("module a;\n\nconst D: u128 = 340282366920938463463374607431768211456;\ntest t { assert(D > 0); }\n");
    assert!(m.contains("literal out of range") && m.contains("does not fit in u128"), "{}", m);
    let m = rejected("module a;\n\nconst D: u128 = 0 - 1;\ntest t { assert(D > 0); }\n");
    assert!(m.contains("literal out of range"), "{}", m);
}

/// Zig compiles these (`t27c test-report`: pass), but they need a run-time
/// value wider than 64 bits, or an operation t27b does not fold: refused by
/// name, never a wrong answer.
#[test]
fn what_t27b_cannot_hold_is_refused() {
    refused(&[
        ("test t { assert(A + B > 0); }", "type u128", "between u128 and u256"),
        ("test t { assert((0 - C) / 2 < 0); }", "type i128", "negative"),
        ("fn f(x: u64) -> bool { return A > x; }\ntest t { assert(f(1)); }", "type u128", "`x`"),
        ("fn f(x: u128) -> bool { return x > 0; }\ntest t { assert(f(1)); }", "type u128", ""),
        ("test t { const x: u128 = 1; assert(x == 1); }", "type u128", ""),
    ]);
}

/// Zig compiles these and so does t27b (`t27c test-report`: pass): a
/// comparison across two wide types, and a wide constant nothing uses,
/// whatever its value (Zig analyzes a module constant only where it is
/// used).
#[test]
fn comparisons_and_unused_constants() {
    assert_eq!(names_ok(&run(&format!("{}test t {{ assert(B < A); assert(A >= B); assert(B > C); }}\n", HEAD))), vec![("t", false, true)]);
    let unused = "module a;\n\nconst D: u128 = 0 - 1;\ntest t { assert(1 == 1); }\n";
    assert_eq!(names_ok(&run(unused)), vec![("t", false, true)]);
}

/// A fn no test reaches may name a wide type: Zig never resolves its
/// signature. Calling it from a reached fn makes it reached, and refused.
#[test]
fn unreached_signatures_are_withdrawn() {
    let ok = "module a;\n\nfn g(x: u128) -> u256 { return x; }\ntest t { assert(1 == 1); }\n";
    assert_eq!(names_ok(&run(ok)), vec![("t", false, true)]);
    let m = rejected("module a;\n\nfn g(x: u128) -> bool { return x > 0; }\ntest t { assert(g(1)); }\n");
    assert!(m.contains("type u128"), "{}", m);
}

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

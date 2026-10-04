//! `--blockers`: every unsupported construct of a file, and the greedy order.

use std::collections::BTreeSet;
use std::path::Path;

use t27b::blockers::{greedy, parse_test_report, replay, Reference};
use t27b::ir::OverflowMode;
use t27b::{front, lower};

/// Four unsupported constructs in four places, one of them (`f64`) repeated,
/// and a fn whose parameter type is outside the subset called from a test.
const SRC: &str = r#"module blockers_fixture;
fn half(x: f64) -> f64 {
    return x;
}
fn widen(x: u8) -> u32 {
    return x as u32;
}
fn named(x: u32) -> u32 {
    const s = "abc";
    return x;
}
fn ok(x: u32) -> u32 {
    return x + 1;
}
fn calls_half(x: u32) -> u32 {
    var h : f64 = half(1);
    return ok(x);
}
test uses_all {
    assert_eq(ok(1), 2);
    assert_eq(widen(3), 3);
    assert_eq(half(2), 2);
}
"#;

fn constructs(rejects: &[lower::Reject]) -> BTreeSet<String> {
    rejects.iter().map(|r| r.construct.clone()).collect()
}

#[test]
fn blockers_lists_every_construct() {
    let parsed = front::parse(Path::new("blockers_fixture.t27"), SRC).expect("fixture parses");
    let all = lower::blockers(&parsed.ast, OverflowMode::Trap);
    let got = constructs(&all);
    let want: BTreeSet<String> = ["type f64", "ExprCast", "ExprLiteral(string literal)"].iter().map(|s| s.to_string()).collect();
    assert_eq!(got, want, "every construct, once each kind: {:#?}", all);
    // Cascades are suppressed: the calls to `half` (whose signature was
    // rejected) and the `f64` local do not add rejections of their own beyond
    // the type itself.
    assert!(all.iter().all(|r| !r.construct.starts_with("ExprCall")), "{:#?}", all);
    assert!(all.iter().all(|r| !r.construct.starts_with("ExprIdentifier")), "{:#?}", all);

    // The normal path is unchanged: it still rejects, and every construct it
    // reports is one `blockers` reports too.
    // (A call to a rejected fn is the normal path's cascade, named as one.)
    let first = lower::lower(&parsed.ast, OverflowMode::Trap).expect_err("fixture is outside the subset");
    let mut normal = constructs(&first);
    assert!(normal.remove("ExprCall(rejected fn)"), "{:#?}", first);
    assert!(normal.is_subset(&got), "{:#?} vs {:#?}", first, all);
}

#[test]
fn blockers_is_empty_for_a_supported_file() {
    let src = "module ok_fixture;\nfn ok(x: u32) -> u32 { return x + 1; }\ntest t { assert_eq(ok(1), 2); }\n";
    let parsed = front::parse(Path::new("ok_fixture.t27"), src).expect("parses");
    assert!(lower::blockers(&parsed.ast, OverflowMode::Trap).is_empty());
    assert!(lower::lower(&parsed.ast, OverflowMode::Trap).is_ok());
}

fn set(xs: &[&str]) -> BTreeSet<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn greedy_prefers_whole_files() {
    // "a" is needed by the most files but unlocks none alone; "b" unlocks two.
    let files = vec![
        set(&["a", "b"]),
        set(&["a", "c"]),
        set(&["a", "d"]),
        set(&["b"]),
        set(&["b"]),
        set(&[]),
    ];
    let order = greedy(&files);
    let names: Vec<&str> = order.iter().map(|s| s.construct.as_str()).collect();
    assert_eq!(names[0], "b");
    assert_eq!(order[0].unlocked, 2);
    // Then "a" unlocks the first file; "c" and "d" one each.
    assert_eq!(names[1], "a");
    assert_eq!(order[1].unlocked, 1);
    assert_eq!(order.last().unwrap().cumulative, 5);
    assert_eq!(order.iter().map(|s| s.unlocked).sum::<usize>(), 5);
    // Replaying the order over the same files gives the same cumulative counts.
    let cum: Vec<usize> = order.iter().map(|s| s.cumulative).collect();
    assert_eq!(replay(&order, &files), cum);
}

#[test]
fn greedy_breaks_a_zero_unlock_step_by_progress() {
    // No construct unlocks a file alone. "x" sits in both files that need
    // two constructs; "p".."s" each sit in one.
    let files = vec![set(&["x", "p"]), set(&["x", "q"]), set(&["r", "s"])];
    let order = greedy(&files);
    assert_eq!(order[0].construct, "x");
    assert_eq!(order[0].unlocked, 0);
    assert_eq!(order[1].unlocked, 1);
}

#[test]
fn reads_test_report_output() {
    let pass = "--- test report: a.t27 ---\n\n  tests       3\n  pass        3\n  FAIL        0\n";
    assert_eq!(parse_test_report(pass), Reference::Pass);
    let none = "--- test report: a.t27 ---\n\n  tests       0\n  pass        0\n  FAIL        0\n";
    assert_eq!(parse_test_report(none), Reference::Pass);
    let fail = "--- test report: a.t27 ---\n  FAIL  t1\n\n  tests       3\n  pass        2\n  FAIL        1\n";
    assert_eq!(parse_test_report(fail), Reference::Fail("1 of 3 tests fail".into()));
    let blocked = "--- test report: a.t27 ---\n  BLOCKED  does not compile: /tmp/x/spec.zig:7:5: error: bad\n";
    assert_eq!(
        parse_test_report(blocked),
        Reference::Blocked("does not compile: spec.zig:7:5: error: bad".into())
    );
    for r in [Reference::Pass, Reference::Fail("1 of 2 tests fail".into()), Reference::Blocked("x\ty".into()), Reference::Timeout] {
        assert_eq!(Reference::decode(&r.encode()), Some(r));
    }
}

//! `--blockers`: every unsupported construct of a file, and the greedy order.

use std::collections::BTreeSet;
use std::path::Path;

use t27b::blockers::{greedy, parse_test_report, replay, retry_timeouts_once, Reference};
use t27b::ir::OverflowMode;
use t27b::{blockers, front, lower};

/// Four unsupported constructs in four places, one of them (`f16`) repeated,
/// and a fn whose parameter type is outside the subset called from a test.
const SRC: &str = r#"module blockers_fixture;
fn half(x: f16) -> f16 {
    return x;
}
fn flag(x: u8) -> bool {
    return x as bool;
}
fn named(x: u32) -> u32 {
    const s = '\0';
    return x;
}
fn ok(x: u32) -> u32 {
    return x + 1;
}
fn calls_half(x: u32) -> u32 {
    var h : f16 = half(1);
    return ok(x);
}
test uses_all {
    assert_eq(ok(1), 2);
    assert(flag(3));
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
    let want: BTreeSet<String> = ["type f16", "ExprCast(to bool)", "ExprLiteral(char escape)"].iter().map(|s| s.to_string()).collect();
    assert_eq!(got, want, "every construct, once each kind: {:#?}", all);
    // Cascades are suppressed: the calls to `half` (whose signature was
    // rejected) and the `f16` local do not add rejections of their own beyond
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

/// `corpus` retries a timed-out file once, sequentially (#6063). A runner that
/// times out only on its first call (lab contention) ends in the retry's
/// verdict; a file that hangs every time (a genuine infinite loop) stays a
/// timeout; nothing is retried twice; a non-timeout is never re-run.
#[test]
fn a_timeout_is_retried_exactly_once_and_the_retry_decides() {
    use std::collections::HashMap;
    // (file, verdict of the parallel pass)
    let mut items = vec![
        ("contended.t27", "timeout"),
        ("passes.t27", "pass"),
        ("infinite_loop.t27", "timeout"),
        ("blocked.t27", "blocked"),
    ];
    let mut calls: HashMap<&str, usize> = HashMap::new();
    let retried = retry_timeouts_once(
        &mut items,
        |x| x.1 == "timeout",
        |x| {
            *calls.entry(x.0).or_default() += 1;
            // The fake runner: the infinite loop hangs on every call, the
            // contended file passes once it runs alone.
            x.1 = if x.0 == "infinite_loop.t27" { "timeout" } else { "pass" };
        },
    );
    assert_eq!(retried, vec![true, false, true, false]);
    assert_eq!(
        items,
        vec![
            ("contended.t27", "pass"),
            ("passes.t27", "pass"),
            ("infinite_loop.t27", "timeout"),
            ("blocked.t27", "blocked"),
        ]
    );
    assert_eq!(calls.get("contended.t27"), Some(&1));
    assert_eq!(calls.get("infinite_loop.t27"), Some(&1), "a second timeout is final");
    assert_eq!(calls.len(), 2, "only timeouts are re-run");

    // Negative control: no timeout, no run, nothing marked.
    let mut clean = vec![("a.t27", "pass"), ("b.t27", "fail")];
    let r = retry_timeouts_once(&mut clean, |x| x.1 == "timeout", |_| panic!("re-ran a non-timeout"));
    assert_eq!(r, vec![false, false]);
}

/// A timeout kills the child's whole process group -- here a grandchild that
/// holds the stdout pipe open -- and nothing outside it. The lab once lost
/// its corpus driver to `kill -KILL -<pgid>`, which procps-ng 4.0.2 reads as
/// "every process"; reaching this line at all means the caller survived.
#[test]
fn a_timeout_kills_the_group_and_spares_the_caller() {
    let t0 = std::time::Instant::now();
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c").arg("sleep 30 & wait");
    let got = blockers::run_capture(&mut cmd, std::time::Duration::from_millis(200)).unwrap();
    assert!(got.is_none(), "a timeout is Ok(None)");
    // The pipes close only once the backgrounded sleep is dead too.
    assert!(t0.elapsed() < std::time::Duration::from_secs(10), "took {:?}", t0.elapsed());
}

/// #6441: the per-test verdicts of `t27c test-report --verbose`, and nothing
/// when the list could be partial.
#[test]
fn reads_per_test_verdicts_of_test_report() {
    use t27b::blockers::parse_test_verdicts;
    let v = "--- test report: a.t27 ---\n  pass  Vec2_add\n  FAIL  Vec2_length\n  pass  Vec2_dot\n\n  tests       3\n  pass        2\n  FAIL        1\n";
    assert_eq!(
        parse_test_verdicts(v),
        Some(vec![("Vec2_add".into(), true), ("Vec2_length".into(), false), ("Vec2_dot".into(), true)])
    );
    // Without --verbose only the failures are listed: the count gives it away.
    let quiet = "--- test report: a.t27 ---\n  FAIL  t1\n\n  tests       3\n  pass        2\n  FAIL        1\n";
    assert_eq!(parse_test_verdicts(quiet), None);
    let none = "--- test report: a.t27 ---\n\n  tests       0\n  pass        0\n  FAIL        0\n";
    assert_eq!(parse_test_verdicts(none), Some(vec![]));
    let blocked = "--- test report: a.t27 ---\n  BLOCKED  does not compile: spec.zig:7:5\n";
    assert_eq!(parse_test_verdicts(blocked), None);
}

#[test]
fn t27b_verdicts_skip_invariants_and_name_repeats_like_t27c() {
    use t27b::blockers::t27b_verdicts;
    let out = "PASS a\nFAIL b: assert_eq failed at line 9 (left 1, right 2)\nINVARIANT PASS inv\nINVARIANT FAIL inv2: assert at line 3\nPASS a\nPASS a\nMISMATCH c: jit Ok, interpreter Err\nm: runtime asserts 4\nm: 3 passed, 1 failed, 4 total\n";
    assert_eq!(
        t27b_verdicts(out),
        vec![
            ("a".to_string(), true),
            ("b".to_string(), false),
            ("a__dup2".to_string(), true),
            ("a__dup3".to_string(), true)
        ]
    );
}

/// The case #6441 was filed for: both sides fail the file, so the file-level
/// comparison called it agreement, whatever test failed.
#[test]
fn a_file_both_sides_fail_agrees_only_if_the_same_tests_fail() {
    use t27b::blockers::disagreements;
    let t = |xs: &[(&str, bool)]| -> Vec<(String, bool)> { xs.iter().map(|(n, ok)| (n.to_string(), *ok)).collect() };
    let reference = t(&[("Vec2_add", true), ("Vec2_length", false), ("Vec2_dot", true)]);
    assert!(disagreements(&reference, &reference).is_empty());
    let other = t(&[("Vec2_add", false), ("Vec2_length", true), ("Vec2_dot", true)]);
    assert_eq!(
        disagreements(&other, &reference),
        vec!["Vec2_add: t27b FAIL, reference pass".to_string(), "Vec2_length: t27b pass, reference FAIL".to_string()]
    );
    // A test only one side ran is a disagreement too, never a silent skip.
    let fewer = t(&[("Vec2_add", true), ("Vec2_length", false)]);
    assert_eq!(disagreements(&fewer, &reference), vec!["Vec2_dot: t27b has no such test, reference pass".to_string()]);
    assert_eq!(disagreements(&reference, &fewer), vec!["Vec2_dot: t27b pass, reference has no such test".to_string()]);
}

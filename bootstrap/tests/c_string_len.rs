//! `s.len` on a `string` is `strlen(s)`, in both spellings.
//!
//! `string` lowers to `const char*`, which carries no length, and the specs
//! write the query three ways:
//!
//!   `s.len()`  parses as ONE `ExprCall` named `s.len`   -- 1322 in the specs
//!   `s.len`    parses as an `ExprFieldAccess`           --  687
//!   `len(s)`   parses as a call to a free function      --  142 diagnostics
//!
//! The first two are handled here through ONE helper, so a change to the rule
//! cannot reach one spelling and miss the other -- the fourth time in this
//! campaign that one rule had several spellings and only one was taught.
//!
//! The third spelling is NOT handled, and the reason is measured rather than
//! assumed: of its 302 argument shapes, **zero** are a `string` parameter (171
//! are identifiers that are not parameters, 89 are not plain identifiers, 19
//! are slices). A branch for it would be unreachable.
//!
//! WHAT IS DELIBERATELY LEFT ALONE: `xs.len` where `xs` is a slice `[]T`. A
//! slice lowers to a bare pointer, so the length is not recoverable without a
//! representation that does not exist yet (#3464). 1337 sites remain, and they
//! are the subject of that decision, not of this repair.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-strlen-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    let p = d.join("in.t27");
    std::fs::write(&p, spec).expect("write");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&p)
        .output()
        .expect("t27c");
    assert!(out.status.success(), "gen-c failed: {}", String::from_utf8_lossy(&out.stderr));
    let h = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(!h.is_empty(), "gen-c produced an EMPTY header");
    (h, d)
}

fn errors(h: &str, d: &std::path::Path) -> usize {
    let p = d.join("h.h");
    std::fs::write(&p, h).expect("write");
    let out = Command::new("cc")
        .args(["-std=c11", "-ferror-limit=0", "-fsyntax-only", "-x", "c"])
        .arg(&p)
        .output()
        .expect("cc");
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| {
            let mut it = l.splitn(4, ':');
            it.next().is_some()
                && it.next().map_or(false, |s| s.trim().parse::<u32>().is_ok())
                && it.next().map_or(false, |s| s.trim().parse::<u32>().is_ok())
                && it.next().map_or(false, |s| s.trim_start().starts_with("error: "))
        })
        .count()
}

#[test]
fn the_call_spelling_becomes_strlen() {
    let (h, d) = gen_c(
        "module S {\n    fn a(s: string) -> u32 { return s.len(); }\n}\n",
        "call",
    );
    assert!(h.contains("return strlen(s);"), "got:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn both_spellings_produce_the_same_call() {
    // The point of the shared helper. If these diverge, one call site was
    // changed without the other.
    let (h, d) = gen_c(
        "module S {\n    fn a(s: string) -> u32 { return s.len(); }\n\
         \x20   fn b(s: string) -> u32 { return s.len; }\n}\n",
        "both",
    );
    assert_eq!(
        h.matches("return strlen(s);").count(),
        2,
        "both spellings must lower the same way:\n{h}"
    );
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn a_slice_is_left_alone() {
    // THE DISCRIMINATING CASE, and the deliberate limitation. A slice lowers to
    // a bare pointer; `strlen` on it would read past the end of an array of
    // anything that is not NUL-terminated bytes. The loud form names the real
    // problem (#3464).
    let (h, _d) = gen_c(
        "module S {\n    fn c(xs: []u32) -> u32 { return xs.len(); }\n}\n",
        "slice",
    );
    assert!(h.contains("xs.len()"), "a slice keeps its own spelling:\n{h}");
    assert!(!h.contains("strlen(xs)"), "and must NOT become strlen:\n{h}");
}

#[test]
fn an_annotated_string_local_counts_too() {
    let (h, d) = gen_c(
        "module S {\n    fn d(v: i32) -> u32 { var s: string = \"abc\"; return s.len(); }\n}\n",
        "local",
    );
    assert!(h.contains("strlen(s)"), "an annotated local is a string too:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn string_h_is_included_exactly_when_it_is_used() {
    // A missing include is an undeclared function -- the very family this
    // repair shrinks -- so the include and the call site are decided by the
    // SAME two helpers, and this is the test that says so.
    let (used, _d) = gen_c(
        "module S {\n    fn a(s: string) -> u32 { return s.len(); }\n}\n",
        "inc-yes",
    );
    assert!(used.contains("#include <string.h>"), "used, so included:\n{used}");
    let (unused, _d2) = gen_c(
        "module S {\n    fn a(v: i32) -> i32 { return v; }\n}\n",
        "inc-no",
    );
    assert!(
        !unused.contains("#include <string.h>"),
        "unused, so not included:\n{unused}"
    );
}

#[test]
fn the_string_set_does_not_leak_into_a_test_block() {
    // The boundary lesson from #3477: the set is per ITEM, and there are three
    // item emitters. A test block's own `s` of another type must not acquire
    // `strlen`.
    // The test block must USE `.len` on its own `s`, or the fixture proves
    // nothing: a leaked set only shows up where the rule would fire. The first
    // version asserted `s == 3` and a mutant that dropped the reset survived it.
    let (h, _d) = gen_c(
        "module S {\n    fn a(s: string) -> u32 { return s.len(); }\n\
         \x20   test \"same name, not a string\" { var s: i32 = 3; assert(s.len() == 0); }\n}\n",
        "leak",
    );
    assert!(h.contains("return strlen(s);"), "the fn still lowers:\n{h}");
    assert!(
        h.contains("s.len()"),
        "the test block's own `s` is an i32 and keeps its spelling:\n{h}"
    );
    assert_eq!(
        h.matches("strlen(s)").count(),
        1,
        "exactly one strlen -- the function's, not the test block's:\n{h}"
    );
}

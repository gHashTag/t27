//! `assert` in modules without test blocks must include `<assert.h>`
//!
//! Found by the issue: a function that calls `assert` in a module with no `test` block
//! compiles to C that calls `assert` without including `<assert.h>`, so a C99 compiler
//! rejects it (`call to undeclared function 'assert'`). The self-hosted core refuses
//! `assert` outside a tested module (E_CALL).
//!
//! This test ensures that `assert.h` is included whenever `assert` is used, regardless
//! of whether there's a test block in the module.

mod common;

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-assert-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    let p = d.join("in.t27");
    std::fs::write(&p, spec).expect("write");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c")).arg("gen-c").arg(&p).output().expect("t27c");
    let h = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(!h.is_empty(), "gen-c produced an EMPTY header");
    (h, d)
}

fn errors(h: &str, d: &std::path::Path) -> usize {
    let p = d.join("h.h");
    std::fs::write(&p, h).expect("write");
    common::error_count(&common::cc_check(&p, &[]))
}

#[test]
fn a_module_with_assert_gets_assert_h() {
    let (h, d) = gen_c(
        "module m {\n    fn f(x: i64) -> i64 {\n        assert(x > 0);\n        return x;\n    }\n}\n",
        "assert",
    );
    assert!(h.contains("#include <assert.h>"), "assert used, so assert.h included:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile with strict C99:\n{h}");
    }
}

#[test]
fn a_module_with_assert_and_test_gets_assert_h() {
    let (h, d) = gen_c(
        "module m {\n    fn f(x: i64) -> i64 {\n        assert(x > 0);\n        return x;\n    }\n    test \"test\" {\n        assert(true);\n    }\n}\n",
        "assert_test",
    );
    assert!(h.contains("#include <assert.h>"), "assert used, so assert.h included:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile with strict C99:\n{h}");
    }
}

#[test]
fn a_module_without_assert_does_not_get_assert_h() {
    let (h, _d) = gen_c(
        "module m {\n    fn f(x: i64) -> i64 {\n        return x;\n    }\n}\n",
        "no_assert",
    );
    assert!(!h.contains("#include <assert.h>"), "assert unused, so assert.h not included:\n{h}");
}

#[test]
fn a_module_with_test_only_gets_assert_h() {
    let (h, d) = gen_c(
        "module m {\n    fn f(x: i64) -> i64 {\n        return x;\n    }\n    test \"test\" {\n        assert(true);\n    }\n}\n",
        "test_only",
    );
    assert!(h.contains("#include <assert.h>"), "test block uses assert, so assert.h included:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile with strict C99:\n{h}");
    }
}
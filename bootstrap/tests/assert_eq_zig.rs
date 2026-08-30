//! `assert_eq` in a test body reached Zig as a call to a function nothing
//! declared.
//!
//! The Zig statement emitter special-cases `assert` and `@compileAssert` and
//! lowers them itself. `assert_eq` is not in that list, so it falls through to
//! the generic call path and is written out verbatim. Zig resolves identifiers
//! in AstGen -- file-wide, before any Sema -- so the undeclared name is a hard
//! error even inside a `test` block that `zig build-obj` never analyses.
//!
//! The C backend hit the identical defect and fixed it with a macro (W583);
//! Rust never emits the call. Zig was the one left out.
//!
//! Measured, per spec, both directions, 581 generated files:
//!
//!     zig build-obj -fno-emit-bin   222 -> 282   +60   regressions 0
//!     zig test --test-no-exec       105 -> 133   +28   regressions 0
//!
//! The gap between the two rulers is the honest part of this change. 60 files
//! clear the gate the corpus harness uses; only 28 survive a ruler that
//! actually analyses test bodies. The other 32 are held by a SEPARATE emitter
//! defect -- `1 << n` lowered as `@as(u32, 1) << ...` regardless of the
//! declared type -- which the undeclared identifier was masking and which
//! `build-obj`'s laziness will keep masking.

use std::io::Write;
use std::process::Command;

fn gen_zig(src: &str) -> String {
    // Keyed by a counter, not by any property of the source: two tests whose
    // sources are the same length would otherwise share a directory that each
    // one deletes on the way out.
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("t27-asserteq-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("m.t27");
    let mut f = std::fs::File::create(&path).expect("write spec");
    f.write_all(src.as_bytes()).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen")
        .arg(&path)
        .output()
        .expect("run t27c");
    let _ = std::fs::remove_dir_all(&dir);
    String::from_utf8_lossy(&out.stdout).to_string()
}

const WITH_TEST: &str = "module m\n\nfn f() -> u8 {\n    return 1\n}\n\ntest t {\n    assert_eq(f(), 1)\n}\n";

#[test]
fn assert_eq_is_declared_before_it_is_called() {
    let z = gen_zig(WITH_TEST);
    assert!(z.contains("assert_eq("), "the call must still be emitted:\n{z}");
    let decl = z
        .find("fn assert_eq(")
        .unwrap_or_else(|| panic!("no declaration for assert_eq:\n{z}"));
    let call = z.rfind("assert_eq(").expect("the call");
    assert!(decl < call, "the declaration must precede the call:\n{z}");
}

/// The shim compares. A stub would clear the gate and write a lie into every
/// test the corpus ships -- `zig build-obj` never analyses a test body, so
/// nothing downstream would notice.
#[test]
fn the_shim_actually_compares_its_arguments() {
    let z = gen_zig(WITH_TEST);
    let body = z
        .split("fn assert_eq(")
        .nth(1)
        .and_then(|r| r.split("\n}").next())
        .expect("shim body");
    assert!(
        body.contains("a != b") && body.contains("__t27_assert_fail"),
        "the shim must compare and report, got:\n{body}"
    );
}

/// A spec with no test block gets no prelude, so the shim is not free weight in
/// every generated file.
#[test]
fn a_spec_without_tests_gets_no_shim() {
    let z = gen_zig("module m\n\nfn f() -> u8 {\n    return 1\n}\n");
    assert!(!z.contains("fn assert_eq("), "unexpected shim:\n{z}");
}

/// The prelude has ONE emitter, not one per caller.
///
/// It lived in two byte-identical copies, `gen_zig` and `gen_zig_project`. That
/// is not a cosmetic complaint: a shim added to the emitter `t27c gen` uses
/// would have left `compile_project_file` emitting the old, broken prelude, and
/// nothing that drives the CLI would have caught it. This test is structural
/// because the project path needs a whole repository on disk to run.
#[test]
fn the_zig_prelude_has_a_single_emitter() {
    let src = include_str!("../src/compiler.rs");
    assert_eq!(
        src.matches("fn assert_eq(a: anytype, b: anytype) void {").count(),
        1,
        "the shim is emitted from more than one place"
    );
    assert_eq!(
        src.matches("self.write_zig_test_prelude();").count(),
        2,
        "both Zig emitters must call the shared prelude"
    );
    assert_eq!(
        src.matches("fn write_zig_test_prelude(&mut self)").count(),
        1,
        "the shared prelude must be defined once"
    );
}

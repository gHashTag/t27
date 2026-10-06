//! The tail of single-file blockers (epic #6063): one test per construct
//! family, each checked against `t27c test-report` (the reference path) for
//! both what runs and what is refused.

mod common;
use common::{names_ok, rejected, run};

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

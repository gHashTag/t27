//! `assert (a & b) == c` is a bare assert whose condition opens with `(`.
//!
//! The statement parser sent every `assert (` to the call path, so this parsed
//! as `assert(a & b)` followed by `== c`: the condition became the call's
//! argument and the comparison took the call as its operand. Typecheck said
//! ok, and the Zig backend emitted
//!
//!     if (!(a & b)) @panic("assertion failed") == c;
//!
//! which `zig test` rejects. 13 lines in 8 specs of the corpus have this shape
//! (specs/fpga/fifo.t27, specs/numeric/e8m0.t27, ...).
//!
//! Only a binary operator after the matching `)`, on the same line, selects
//! the bare form; `assert(x);` and `assert(x, "msg")` keep their old output.

use std::io::Write;
use std::process::Command;

fn gen_zig(src: &str) -> String {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("t27-assertparen-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("assertparen.t27");
    std::fs::File::create(&path)
        .and_then(|mut f| f.write_all(src.as_bytes()))
        .expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen")
        .arg(&path)
        .output()
        .expect("run t27c");
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success(),
        "t27c gen failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// The emitted lines of the one test block, trimmed.
fn test_body(zig: &str) -> Vec<String> {
    zig.lines()
        .skip_while(|l| !l.trim_start().starts_with("test "))
        .skip(1)
        .take_while(|l| l.trim() != "}")
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn spec(stmt: &str) -> String {
    format!(
        "module assertparen;\n\
         pub const A : u32 = 4;\n\
         pub const B : u32 = 7;\n\
         test t {{\n    {}\n}}\n",
        stmt
    )
}

/// The bare form emits exactly what the call form with the same condition
/// emits. Before the fix the two differed: the bare form left `== A` or
/// `and (B > A)` hanging off the end of the assert statement.
fn assert_bare_matches_call(bare: &str, call: &str) {
    let b = test_body(&gen_zig(&spec(bare)));
    let c = test_body(&gen_zig(&spec(call)));
    assert_eq!(b, c, "`{}` does not emit what `{}` emits", bare, call);
    assert_eq!(b.len(), 1, "one statement expected: {:?}", b);
}

#[test]
fn paren_then_comparison_stays_inside_the_assert() {
    assert_bare_matches_call("assert (A & B) == A;", "assert((A & B) == A);");
}

#[test]
fn paren_then_logical_and_stays_inside_the_assert() {
    assert_bare_matches_call("assert (A < B) and (B > A);", "assert((A < B) and (B > A));");
}

#[test]
fn paren_then_arithmetic_stays_inside_the_assert() {
    assert_bare_matches_call("assert (B - A) < 4;", "assert((B - A) < 4);");
}

/// The call forms are not routed through the new path.
#[test]
fn call_forms_are_unchanged() {
    let body = test_body(&gen_zig(&spec("assert((A & B) == A);")));
    assert_eq!(body.len(), 1, "{:?}", body);
    assert!(body[0].starts_with("if (!((A & B) == A)) "), "{:?}", body);

    let with_msg = gen_zig(&spec("assert(A < B, \"a below b\");"));
    assert!(with_msg.contains("a below b"), "message dropped:\n{}", with_msg);
}

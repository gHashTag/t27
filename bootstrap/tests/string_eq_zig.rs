//! `==` / `!=` on strings is the emitter's job (#5162).
//!
//! Zig has no `==` for `[]const u8`: "operator == not allowed for type
//! '[]const u8'". The Zig emitter already lowered a comparison to
//! `std.mem.eql(u8, a, b)` when one side was a string LITERAL, a parameter
//! declared `str`, or a struct field declared `str`. It missed the shape the
//! automation specs actually write -- two module-level string constants, or a
//! string parameter against a string constant:
//!
//!     if source == SOURCE_MEETING { ... }
//!     assert(DIGEST_DOOR != TOOL)
//!     assert(mail_ball("us") == BALL_OURS)
//!
//! so `t27c test-report` reported every such spec BLOCKED (ball-board,
//! mail-push) and none of their tests ran.
//!
//! What makes a name a string is read off its DECLARATION, never inferred: a
//! module const declared `: str` (or untyped and initialised by a string
//! literal), a parameter or local declared `str`, a call to a function this
//! spec declares `-> str`. A numeric comparison must keep its plain `==`.

use std::io::Write;
use std::process::Command;

fn write_spec(src: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    // Keyed by a counter: two tests must not share a directory that each one
    // deletes on the way out.
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("t27-streq-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("streq.t27");
    let mut f = std::fs::File::create(&path).expect("write spec");
    f.write_all(src.as_bytes()).expect("write spec");
    (dir, path)
}

fn gen_zig(src: &str) -> String {
    let (dir, path) = write_spec(src);
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

const SPEC: &str = r#"module streq {
    pub const KIND : str = "automation";
    pub const OTHER : str = "mail";
    pub const UNTYPED = "automation";
    pub const LIMIT : u32 = 3;

    fn is_kind(source: str) -> bool {
        return source == KIND;
    }

    fn not_other(source: str) -> bool {
        return source != OTHER;
    }

    fn label() -> str {
        return KIND;
    }

    fn local_match(source: str) -> bool {
        let want: str = "automation";
        return source == want;
    }

    fn at_limit(n: u32) -> bool {
        return n == LIMIT;
    }

    fn not_at_limit(n: u32) -> bool {
        return n != LIMIT;
    }

    test "a literal operand" {
        assert(KIND == "automation");
    }

    test "two string consts" {
        assert(KIND != OTHER);
        assert(KIND == UNTYPED);
    }

    test "a string param against a string const" {
        assert(is_kind("automation"));
        assert(!is_kind("mail"));
        assert(not_other("automation"));
        assert(!not_other("mail"));
    }

    test "a str-returning call against a const" {
        assert(label() == KIND);
    }

    test "a str local" {
        assert(local_match("automation"));
        assert(!local_match("mail"));
    }

    test "numbers keep plain equality" {
        assert(at_limit(3));
        assert(not_at_limit(4));
        assert(LIMIT == 3);
    }
}
"#;

#[test]
fn string_const_and_param_comparisons_lower_to_mem_eql() {
    let z = gen_zig(SPEC);
    for want in [
        // a literal operand (already handled before #5162 -- kept as a guard)
        "std.mem.eql(u8, KIND, \"automation\")",
        // const vs const, both directions of the operator
        "!std.mem.eql(u8, KIND, OTHER)",
        "std.mem.eql(u8, KIND, UNTYPED)",
        // a `str` param against a module const
        "std.mem.eql(u8, source, KIND)",
        "!std.mem.eql(u8, source, OTHER)",
        // a call to a fn declared `-> str`
        "std.mem.eql(u8, label(), KIND)",
        // a local declared `str`
        "std.mem.eql(u8, source, want)",
    ] {
        assert!(z.contains(want), "expected `{want}` in:\n{z}");
    }
}

#[test]
fn numeric_comparisons_keep_plain_operators() {
    let z = gen_zig(SPEC);
    assert!(z.contains("n == LIMIT"), "numeric == must stay plain:\n{z}");
    assert!(z.contains("n != LIMIT"), "numeric != must stay plain:\n{z}");
    assert!(z.contains("LIMIT == 3"), "numeric == must stay plain:\n{z}");
    assert!(
        !z.contains("std.mem.eql(u8, n,") && !z.contains("std.mem.eql(u8, LIMIT"),
        "a numeric operand must never reach std.mem.eql:\n{z}"
    );
}

#[test]
fn a_local_str_does_not_leak_into_the_next_function() {
    // `want` is a string only inside `local_match`; a numeric `want` elsewhere
    // must keep its plain `==`.
    let src = r#"module leak {
    fn a(source: str) -> bool {
        let want: str = "x";
        return source == want;
    }
    fn b(want: u32) -> bool {
        return want == 3;
    }
}
"#;
    let z = gen_zig(src);
    assert!(z.contains("std.mem.eql(u8, source, want)"), "{z}");
    assert!(z.contains("want == 3"), "the numeric param leaked into string_names:\n{z}");
}

/// End to end: the generated tests compile and PASS under `zig test`, through
/// the same `test-report` path that reported the automation specs BLOCKED.
/// Skipped when no `zig` is on PATH, so the textual guards above still run
/// everywhere.
#[test]
fn test_report_runs_the_string_tests() {
    if Command::new("zig").arg("version").output().is_err() {
        eprintln!("zig not on PATH -- skipping the end-to-end leg");
        return;
    }
    let (dir, path) = write_spec(SPEC);
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("test-report")
        .arg(&path)
        .output()
        .expect("run t27c test-report");
    let _ = std::fs::remove_dir_all(&dir);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!text.contains("BLOCKED"), "the spec must compile:\n{text}");
    let count = |label: &str| -> Option<u32> {
        text.lines()
            .map(str::trim)
            .find(|l| l.starts_with(label))
            .and_then(|l| l[label.len()..].trim().parse().ok())
    };
    assert_eq!(count("tests"), Some(6), "{text}");
    assert_eq!(count("pass"), Some(6), "{text}");
    assert_eq!(count("FAIL"), Some(0), "{text}");
}

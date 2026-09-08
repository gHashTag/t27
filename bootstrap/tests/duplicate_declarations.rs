//! `t27c check` must say something when a spec declares one name twice.
//!
//! It was silent, and the consequence is a crate that cannot compile: the
//! backends emit every copy. `specs/ml/optimizer/adamw.t27` carries what looks
//! like a revised second copy of its own declarations -- the two
//! `AdamWConfig` bodies are not even identical, one ends `use_phi_betas` and
//! the other `phi_variant` -- and `check` exited 0.
//!
//! Two findings, not one, because a single message would be false for half the
//! cases. Measured against the real compilers:
//!
//!   struct A + struct A   C: `redefinition of 'S'`   rustc: E0428
//!                         zig: `duplicate ...`       iverilog: `already been declared`
//!   struct A + fn A       rust and C ACCEPT it (separate namespaces);
//!                         zig rejects it (one namespace per container)
//!
//! The corpus has 25 of the first and none of the second. A zero is only worth
//! reporting if the thing could have been found, so the second message has its
//! own test proving it is reachable.

use std::process::Command;

fn check(spec: &str, tag: &str) -> (i32, String) {
    let dir = std::env::temp_dir().join(format!("t27c-dupdecl-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let p = dir.join("in.t27");
    std::fs::write(&p, spec).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("check")
        .arg(&p)
        .output()
        .expect("run t27c check");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), text)
}

const DUP_STRUCT: &str = r#"
module D1 {
    struct A { x : i32, }
    struct A { y : i32, }
    fn f(a: A) -> i32 { return a.x; }
}
"#;

const CLEAN: &str = r#"
module D3 {
    struct C { x : i32, }
    fn g(c: C) -> i32 { return c.x; }
}
"#;

#[test]
fn a_name_declared_twice_is_reported_with_its_count() {
    let (_, out) = check(DUP_STRUCT, "twice");
    assert!(
        out.contains("`struct A` is declared 2 times"),
        "the finding must name the kind, the name and the count:\n{out}"
    );
    assert!(
        out.contains("every backend rejects a redeclaration"),
        "and say what the consequence is:\n{out}"
    );
}

#[test]
fn a_clean_spec_reports_neither_finding() {
    // The negative control. Without it a detector that fires on everything
    // passes the test above.
    let (rc, out) = check(CLEAN, "clean");
    assert_eq!(rc, 0, "a clean spec must still check clean:\n{out}");
    assert!(
        !out.contains("redeclaration") && !out.contains("both a type and a function"),
        "a spec declaring each name once must produce no duplicate finding:\n{out}"
    );
}

#[test]
fn a_type_and_a_function_sharing_a_name_gets_the_weaker_message() {
    // Reachability, and it is the point of the test. The corpus contains zero
    // of these; a zero from a message nothing could emit would be worthless.
    let (_, out) = check(
        r#"
module D2 {
    struct B { x : i32, }
    fn B(v: i32) -> i32 { return v; }
}
"#,
        "crossns",
    );
    assert!(
        out.contains("both a type and a function"),
        "a type and a function sharing a name must be reported:\n{out}"
    );
    assert!(
        out.contains("zig rejects this; rust and C do not"),
        "and the message must NOT claim every backend rejects it -- rust and C accept it:\n{out}"
    );
    assert!(
        !out.contains("every backend rejects"),
        "the strong claim must not be attached to the weak case:\n{out}"
    );
}

#[test]
fn a_struct_and_an_enum_share_one_namespace() {
    // Both live in the type namespace in Rust and in the tag namespace in C,
    // and both compilers reject the pair -- so this is the STRONG message, not
    // the weak one.
    let (_, out) = check(
        r#"
module D4 {
    struct E { x : i32, }
    enum E { One, Two }
    fn h(v: i32) -> i32 { return v; }
}
"#,
        "structenum",
    );
    assert!(
        out.contains("is declared 2 times") && out.contains("every backend rejects"),
        "struct and enum sharing a name is a same-namespace collision:\n{out}"
    );
}

#[test]
fn the_finding_is_a_warning_and_does_not_change_the_exit_code() {
    // Measured over the corpus before this landed: 651 specs, 573 exit 0 and
    // 78 exit 1, and not one spec's code moved. Pinned here so a later change
    // to an error is a deliberate, test-visible decision.
    let (rc, out) = check(DUP_STRUCT, "exitcode");
    assert_eq!(
        rc, 0,
        "the duplicate finding is a warning; promoting it must be deliberate:\n{out}"
    );
}

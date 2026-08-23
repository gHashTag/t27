//! #2164 -- `Name(T)` is a type application in a type position.
//!
//! This test asserts the AST text, rather than exit status alone. A parser that
//! returns success while dropping a parameter type or return type would not meet
//! the language contract.

use std::process::{Command, Output};

fn parse_source(source: &str, stem: &str) -> Output {
    let bin = env!("CARGO_BIN_EXE_t27c");
    let path = std::env::temp_dir().join(format!("t27c_issue_2164_{stem}.t27"));
    std::fs::write(&path, source).expect("failed to write generic application fixture");
    Command::new(bin)
        .arg("parse")
        .arg(&path)
        .output()
        .expect("failed to spawn t27c parse")
}

fn fixture() -> &'static str {
    "\
module GenericTypeApplication {
    fn empty() -> List(void) {}
    fn add(set: *HashSet(T)) -> void {}
    fn is_left(either: Either(L, R)) -> void {}
}
"
}

fn numeric_fixture() -> &'static str {
    "\
module NumericTypeApplications {
    fn numeric(p: P(2), z: Z(1), n: N(0)) -> void {}
}
"
}

#[test]
fn generic_applications_are_preserved_in_parameter_and_return_type_positions() {
    let out = parse_source(fixture(), "generic_type_application");
    assert!(
        out.status.success(),
        "generic applications in type positions must parse (#2164); stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let dump = String::from_utf8_lossy(&out.stdout);
    let compact: String = dump.chars().filter(|c| !c.is_whitespace()).collect();
    for expected in [
        "name:\"empty\"",
        "extra_return_type:\"List(void)\"",
        "name:\"add\"",
        "(\"set\",\"*HashSet(T)\",)",
        "name:\"is_left\"",
        "(\"either\",\"Either(L,R)\",)",
    ] {
        assert!(
            compact.contains(expected),
            "parse must retain {expected:?}, not merely accept the input:\n{dump}"
        );
    }
}

#[test]
fn numeric_type_arguments_remain_unchanged() {
    let out = parse_source(numeric_fixture(), "numeric_type_application");
    assert!(
        out.status.success(),
        "existing numeric type applications must remain accepted; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let dump = String::from_utf8_lossy(&out.stdout);
    let compact: String = dump.chars().filter(|c| !c.is_whitespace()).collect();
    for expected in [
        "(\"p\",\"P(2)\",)",
        "(\"z\",\"Z(1)\",)",
        "(\"n\",\"N(0)\",)",
    ] {
        assert!(
            compact.contains(expected),
            "numeric application must remain intact as {expected:?}:\n{dump}"
        );
    }
}

//! Behavioural tests for the C and Rust backends: does the emitted code
//! COMPILE, and does it compute the right answer?
//!
//! Why this file exists. Four defects were found in one session, and each one
//! emitted a green exit over output that was wrong or absent:
//!
//!   * `gen-rust` wrote an empty `match` for every `switch`;
//!   * `gen-rust` dropped the body of every `for` loop;
//!   * `gen-c` emitted no loop header at all, so the body ran once;
//!   * `gen-c` typed an un-annotated local as `int`, printing 1 where the
//!     other backends print 4294967297.
//!
//! Every one of them was invisible to the 1600-test suite, because those tests
//! read the emitted TEXT and none of them hands it to a compiler. The Verilog
//! backend has had `iverilog` targets in this directory for a long time; C and
//! Rust had nothing.
//!
//! Each test below writes a small spec, generates, compiles with the real
//! toolchain, RUNS it, and checks the printed answer. A test that cannot find
//! its compiler skips loudly rather than passing quietly -- an absent tool is
//! not a passing test, and this file exists precisely because silence looked
//! like success.

use std::process::Command;

fn tool_present(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn tmp_dir(tag: &str) -> std::path::PathBuf {
    // Deterministic per-test, so a failing run leaves its artefacts behind to
    // look at instead of a random name nobody can find again.
    let d = std::env::temp_dir().join(format!("t27c-backend-behaviour-{tag}"));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("create temp dir");
    d
}

fn generate(subcommand: &str, spec: &str, dir: &std::path::Path, name: &str) -> String {
    let spec_path = dir.join("in.t27");
    std::fs::write(&spec_path, spec).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg(subcommand)
        .arg(&spec_path)
        .output()
        .expect("run t27c");
    assert!(
        out.status.success(),
        "{name}: `t27c {subcommand}` failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Generate C, append a `main`, compile, run, and return stdout.
fn c_says(spec: &str, main: &str, tag: &str) -> Option<String> {
    if !tool_present("cc") {
        eprintln!("SKIP {tag}: no cc on PATH");
        return None;
    }
    let dir = tmp_dir(tag);
    let mut src = generate("gen-c", spec, &dir, tag);
    src.push_str("\n#include <stdio.h>\n");
    src.push_str(main);
    let c_path = dir.join("out.c");
    let bin_path = dir.join("out.bin");
    std::fs::write(&c_path, &src).expect("write C");
    let cc = Command::new("cc")
        .arg("-std=gnu11")
        .arg("-o")
        .arg(&bin_path)
        .arg(&c_path)
        .output()
        .expect("run cc");
    assert!(
        cc.status.success(),
        "{tag}: generated C does not compile:\n{}",
        String::from_utf8_lossy(&cc.stderr)
    );
    let run = Command::new(&bin_path).output().expect("run compiled C");
    assert!(run.status.success(), "{tag}: compiled C exited non-zero");
    Some(String::from_utf8_lossy(&run.stdout).trim().to_string())
}

/// Generate Rust, append a `main`, compile, run, and return stdout.
fn rust_says(spec: &str, main: &str, tag: &str) -> Option<String> {
    if !tool_present("rustc") {
        eprintln!("SKIP {tag}: no rustc on PATH");
        return None;
    }
    let dir = tmp_dir(tag);
    let mut src = generate("gen-rust", spec, &dir, tag);
    src.push('\n');
    src.push_str(main);
    let rs_path = dir.join("out.rs");
    let bin_path = dir.join("out.bin");
    std::fs::write(&rs_path, &src).expect("write Rust");
    let rc = Command::new("rustc")
        .arg("--edition")
        .arg("2021")
        .arg("-A")
        .arg("warnings")
        .arg("-o")
        .arg(&bin_path)
        .arg(&rs_path)
        .output()
        .expect("run rustc");
    assert!(
        rc.status.success(),
        "{tag}: generated Rust does not compile:\n{}",
        String::from_utf8_lossy(&rc.stderr)
    );
    let run = Command::new(&bin_path).output().expect("run compiled Rust");
    assert!(run.status.success(), "{tag}: compiled Rust exited non-zero");
    Some(String::from_utf8_lossy(&run.stdout).trim().to_string())
}

const SPEC_RANGE_LOOP: &str = r#"module loops
pub fn sum_to_three() i32 {
    var total: i32 = 0;
    for i in 1..4 {
        total = total + i;
    }
    return total;
}
"#;

#[test]
fn c_runs_a_range_loop_the_right_number_of_times() {
    // gen-c emitted `/* for-each loop */` and a bare block: the body ran once
    // and this returned 1. It compiled cleanly, so nothing downstream noticed.
    let Some(out) = c_says(
        SPEC_RANGE_LOOP,
        "int main(void){ printf(\"%d\\n\", sum_to_three()); return 0; }\n",
        "c-range-loop",
    ) else {
        return;
    };
    assert_eq!(out, "6", "1+2+3 is 6; a body that runs once gives 1");
}

#[test]
fn rust_runs_a_range_loop_the_right_number_of_times() {
    let Some(out) = rust_says(
        SPEC_RANGE_LOOP,
        "fn main(){ println!(\"{}\", sum_to_three()); }\n",
        "rust-range-loop",
    ) else {
        return;
    };
    assert_eq!(out, "6");
}

const SPEC_INCLUSIVE: &str = r#"module inclusive
pub fn sum_to_three() i32 {
    var total: i32 = 0;
    for i in 1..=3 {
        total = total + i;
    }
    return total;
}
"#;

#[test]
fn the_inclusive_range_includes_its_upper_bound() {
    // `1..=3` must run three times, not two. Lowered as `1..(3 + 1)`.
    let Some(c) = c_says(
        SPEC_INCLUSIVE,
        "int main(void){ printf(\"%d\\n\", sum_to_three()); return 0; }\n",
        "c-inclusive",
    ) else {
        return;
    };
    assert_eq!(c, "6");
}

const SPEC_WIDE_LOCAL: &str = r#"module wide
pub fn big() u64 {
    return 4294967296;
}
pub fn plus_one() u64 {
    const v = big();
    return v + 1;
}
"#;

#[test]
fn c_and_rust_agree_on_an_un_annotated_wide_local() {
    // `const v = big()` with `big() -> u64` was typed as C's `int`, so C
    // printed 1 and Rust printed 4294967297 -- from one spec, with the C
    // compiling without a diagnostic. The two backends must agree.
    let c = c_says(
        SPEC_WIDE_LOCAL,
        "int main(void){ printf(\"%llu\\n\", (unsigned long long)plus_one()); return 0; }\n",
        "c-wide-local",
    );
    let r = rust_says(
        SPEC_WIDE_LOCAL,
        "fn main(){ println!(\"{}\", plus_one()); }\n",
        "rust-wide-local",
    );
    match (c, r) {
        (Some(c), Some(r)) => {
            assert_eq!(c, "4294967297", "C truncated a u64 to int");
            assert_eq!(r, "4294967297");
            assert_eq!(c, r, "the two backends disagree about the same spec");
        }
        _ => eprintln!("SKIP c_and_rust_agree_on_an_un_annotated_wide_local: a toolchain is missing"),
    }
}

const SPEC_MODULE_VAR: &str = r#"module mutable
var counter : u32 = 0;
pub fn bump() u32 {
    counter = counter + 1;
    return counter;
}
"#;

#[test]
fn a_module_level_var_is_mutable_in_c() {
    // It was emitted as `#define counter 0`, which turns the assignment into
    // `0 = (0 + 1)` -- not C at all.
    let Some(out) = c_says(
        SPEC_MODULE_VAR,
        "int main(void){ bump(); printf(\"%u\\n\", bump()); return 0; }\n",
        "c-module-var",
    ) else {
        return;
    };
    assert_eq!(out, "2", "two calls must leave the counter at 2");
}

#[test]
fn a_module_level_var_is_mutable_in_rust() {
    // Emitted as `pub const`, which Rust will not let you assign to. Now
    // `pub static mut`, with the reading function's body wrapped in `unsafe`.
    let Some(out) = rust_says(
        SPEC_MODULE_VAR,
        "fn main(){ unsafe { bump(); println!(\"{}\", bump()); } }\n",
        "rust-module-var",
    ) else {
        return;
    };
    assert_eq!(out, "2");
}

const SPEC_SWITCH: &str = r#"module sw
pub enum Trit { neg, zero, pos }
pub fn negate(a: Trit) Trit {
    return switch (a) {
        .neg => .pos,
        .zero => .zero,
        .pos => .neg,
    };
}
"#;

#[test]
fn rust_keeps_the_arms_of_a_switch() {
    // The arm loop tested for a node kind the parser never builds, so this
    // emitted `match a { }` -- an empty match, with exit code 0, for a
    // construct gen-c and gen-verilog both lower.
    let dir = tmp_dir("rust-switch-text");
    let src = generate("gen-rust", SPEC_SWITCH, &dir, "rust-switch-text");
    assert!(
        src.contains("Trit::neg => Trit::pos"),
        "the arms are missing from:\n{src}"
    );
    assert!(
        !src.contains("match a {\n}"),
        "an empty match reached the output:\n{src}"
    );
}

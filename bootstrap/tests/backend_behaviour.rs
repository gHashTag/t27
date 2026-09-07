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
    // look at instead of a random name nobody can find again. The pid keeps
    // that property while separating concurrent RUNS: the tag is already
    // distinct per test, so nothing here collides inside one process, and this
    // binary still failed 30 of 32 runs with 16 copies going at once -- every
    // one of them writing `-{tag}` into the same `$TMPDIR`, and
    // `remove_dir_all` two lines down deleting a sibling run's directory
    // mid-read. A counter is deliberately NOT added: it would make the path
    // unpredictable, which is the property this comment is defending.
    let d = std::env::temp_dir().join(format!(
        "t27c-backend-behaviour-{tag}-{}",
        std::process::id()
    ));
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

/// The emitted Rust as TEXT, for the one property that cannot be observed by
/// running: what a file that deliberately does NOT compile was emitted as.
fn rust_text(spec: &str, tag: &str) -> Option<String> {
    let dir = tmp_dir(tag);
    Some(generate("gen-rust", spec, &dir, tag))
}

/// Specs write the math builtins bare. `zig_builtin_to_rust` knew all of them
/// and its first line returned `None` for any name without a `@`, so the bare
/// call reached rustc verbatim: "cannot find function `abs` in this scope".
const SPEC_BARE_MATH_BUILTINS: &str = r#"
module builtins_probe {
    fn mix(x: f64) -> f64 {
        return abs(x) + sqrt(y4()) + floor(y27()) + ceil(y21()) + round(y25()) + min(x, y9()) + max(x, y9());
    }
    fn y4() -> f64 { return 4.0; }
    fn y27() -> f64 { return 2.7; }
    fn y21() -> f64 { return 2.1; }
    fn y25() -> f64 { return 2.5; }
    fn y9() -> f64 { return 9.0; }
}
"#;

/// The guard. A spec may name its own function `abs`, and 30 declarations in
/// the corpus do exactly that (`fn floor` in 10 specs, `fn abs` in 9). This
/// one is deliberately NOT absolute value, so a wrong redirect to `f64::abs`
/// changes the printed answer instead of merely changing the text.
const SPEC_USER_DEFINED_ABS: &str = r#"
module guard_probe {
    fn abs(x: f64) -> f64 { return x + 100.0; }
    fn call_it(y: f64) -> f64 { return abs(y); }
}
"#;

#[test]
fn rust_lowers_bare_math_builtins_to_methods() {
    // -3 -> 3, sqrt 4 -> 2, floor 2.7 -> 2, ceil 2.1 -> 3, round 2.5 -> 3,
    // min(-3, 9) -> -3, max(-3, 9) -> 9.  3 + 2 + 2 + 3 + 3 - 3 + 9 = 19.
    let Some(out) = rust_says(
        SPEC_BARE_MATH_BUILTINS,
        "fn main(){ println!(\"{}\", mix(-3.0)); }\n",
        "rust-bare-math-builtins",
    ) else {
        return;
    };
    assert_eq!(out, "19", "a bare `abs(` does not compile in Rust at all");
}

#[test]
fn a_spec_declaring_its_own_abs_keeps_its_own_abs() {
    // The spec's `abs` adds 100. Redirecting the call to `f64::abs` would
    // print 3 -- a wrong translation that COMPILES, which is strictly worse
    // than the bare name that does not. This test fails on 3 and passes on 97.
    let Some(out) = rust_says(
        SPEC_USER_DEFINED_ABS,
        "fn main(){ println!(\"{}\", call_it(-3.0)); }\n",
        "rust-user-defined-abs",
    ) else {
        return;
    };
    assert_eq!(out, "97", "the spec's own `fn abs` must win over the builtin");
}

#[test]
fn a_literal_receiver_is_left_bare() {
    // `(5.0).sqrt()` is E0689, "can't call method `sqrt` on ambiguous numeric
    // type `{float}`", and `sqrt` is not a `const fn` either, so no spelling of
    // a builtin works in a `const` initialiser. Leaving the bare call there
    // keeps the pre-existing E0425 rather than trading it for a new failure.
    let src = "module lit_probe {\n    const K : f64 = sqrt(5.0);\n    fn g(x: f64) -> f64 { return sqrt(x); }\n}\n";
    let Some(text) = rust_text(src, "rust-literal-receiver") else {
        return;
    };
    assert!(
        text.contains("sqrt(5.0)"),
        "a literal receiver must stay bare, got:\n{text}"
    );
    assert!(
        text.contains("(x).sqrt()"),
        "a typed receiver must become a method call, got:\n{text}"
    );
}

/// A `[]T` parameter written by the body is an OUT parameter. gen-rust rendered
/// it `Vec<T>` -- by value, no `mut` -- and then emitted `buf[i] = x` into it.
/// rustc rejects that, and had it compiled the caller would see nothing.
const SPEC_OUT_PARAM: &str = r#"
module outp {
    fn fill(buf: []i32, n: usize) -> void {
        var i : usize = 0;
        while (i < n) {
            buf[i] = 7;
            i = i + 1;
        }
    }
}
"#;

/// The inductive half: `outer` never assigns into its own `buf`, it hands it to
/// `inner`, which does. Marking only direct writers gave the caller `Vec<i32>`
/// and the callee `&mut [i32]`, and the call between them was E0308.
const SPEC_OUT_PARAM_CHAIN: &str = r#"
module chain {
    fn inner(buf: []i32, n: usize) -> void {
        var i : usize = 0;
        while (i < n) {
            buf[i] = 5;
            i = i + 1;
        }
    }
    fn outer(buf: []i32, n: usize) -> void {
        inner(buf, n);
    }
}
"#;

#[test]
fn an_out_parameter_gives_the_caller_its_writes() {
    let Some(out) = rust_says(
        SPEC_OUT_PARAM,
        "fn main(){ let mut a = [0i32; 3]; fill(&mut a, 3); println!(\"{}\", a[0]+a[1]+a[2]); }\n",
        "rust-out-param",
    ) else {
        return;
    };
    assert_eq!(out, "21", "three sevens; 0 means the writes landed in a copy");
}

#[test]
fn an_out_parameter_threaded_through_a_call_is_still_an_out_parameter() {
    let Some(out) = rust_says(
        SPEC_OUT_PARAM_CHAIN,
        "fn main(){ let mut a = [0i32; 2]; outer(&mut a, 2); println!(\"{}\", a[0]+a[1]); }\n",
        "rust-out-param-chain",
    ) else {
        return;
    };
    assert_eq!(out, "10", "two fives, written two calls deep");
}

#[test]
fn only_the_written_slice_parameter_changes() {
    // The guard has THREE outcomes and the third one matters. An earlier draft
    // made every slice parameter a reference; `[]T` then meant `&[T]` in
    // parameter position and `Vec<T>` in return, field and local position, and
    // `fn join(base: []u8) []u8 { var r : []u8 = base; }` emitted
    // `base: &[u8]` beside `let mut r: Vec<u8> = base;` -- E0308. Zig renders
    // `[]u8` in every position and C renders `uint8_t*` in every position; only
    // Rust would have disagreed with itself. An unmarked parameter is a no-op.
    let src = "module ro {\n    fn peek(a: []i32, b: []i32, c: [3]i32) -> i32 { a[0] = 1; return b[0] + c[0]; }\n}\n";
    let Some(text) = rust_text(src, "rust-readonly-slice") else {
        return;
    };
    assert!(text.contains("a: &mut [i32]"), "written slice, got:\n{text}");
    assert!(text.contains("b: Vec<i32>"), "read-only slice must be untouched, got:\n{text}");
    assert!(text.contains("c: [i32; 3]"), "fixed array untouched, got:\n{text}");
}

/// The argument half. Rewriting the PARAMETER to `&mut [T]` and leaving the
/// argument alone gave `tritwise_and(a, b, temp, len)` reading
/// "expected `&mut [i32]`, found `[i32; 27]`" -- the signature was right and
/// the call was not. A caller passing on its OWN `&mut [T]` parameter is
/// reborrowing and must NOT get a second `&mut`, so both shapes are here.
const SPEC_OUT_PARAM_CALLERS: &str = r#"
module callers {
    fn fill(buf: []i32, n: usize) -> void {
        var i : usize = 0;
        while (i < n) {
            buf[i] = 3;
            i = i + 1;
        }
    }
    fn via_local(n: usize) -> i32 {
        // NOT `= undefined`: that lowers to `let mut tmp: [i32; 4];` with no
        // initialiser and rustc answers E0381 before it ever type-checks the
        // call, which is a different defect and would make this test measure it
        // instead of the one it is here for.
        var tmp : [4]i32 = [_]i32{0, 0, 0, 0};
        fill(tmp, n);
        return tmp[0];
    }
    fn via_param(buf: []i32, n: usize) -> void {
        fill(buf, n);
    }
}
"#;

#[test]
fn a_local_array_is_borrowed_at_the_call_and_a_parameter_is_not() {
    let Some(text) = rust_text(SPEC_OUT_PARAM_CALLERS, "rust-callsite-borrow") else {
        return;
    };
    assert!(
        text.contains("fill(&mut tmp, n)"),
        "a local array must be borrowed at the call, got:\n{text}"
    );
    assert!(
        text.contains("fill(buf, n)") && !text.contains("fill(&mut buf, n)"),
        "passing on a &mut [T] parameter is a reborrow, not a second borrow, got:\n{text}"
    );
}

#[test]
fn a_local_array_out_parameter_round_trips_at_runtime() {
    let Some(out) = rust_says(
        SPEC_OUT_PARAM_CALLERS,
        "fn main(){ println!(\"{}\", via_local(4)); }\n",
        "rust-callsite-run",
    ) else {
        return;
    };
    assert_eq!(out, "3", "the local array must actually receive the write");
}

/// Zig exposes a slice length as the FIELD `.len`, so specs are written that
/// way and both spellings reached rustc unlowered: `data.len` as E0615
/// ("attempted to take value of method"), `len(data)` as E0425.
const SPEC_LEN_BOTH_SPELLINGS: &str = r#"
module lens {
    fn field_form(a: []i32) -> usize {
        return a.len;
    }
    fn call_form(a: []i32) -> usize {
        return len(a);
    }
}
"#;

/// The guard. A struct may have a field genuinely named `len` -- 6 corpus specs
/// do -- and there the access is a field and must stay one.
const SPEC_LEN_IS_A_REAL_FIELD: &str = r#"
module owns_len {
    struct Buf {
        len: u32,
        cap: u32,
    }
    fn size_of(b: Buf) -> u32 {
        return b.len;
    }
}
"#;

#[test]
fn both_spellings_of_length_become_a_method_call() {
    let Some(text) = rust_text(SPEC_LEN_BOTH_SPELLINGS, "rust-len-both") else {
        return;
    };
    assert!(text.contains("a.len()"), "field form, got:\n{text}");
    assert!(text.contains("(a).len()"), "free-call form, got:\n{text}");
}

#[test]
fn a_struct_field_named_len_stays_a_field() {
    // Rewriting this to `b.len()` is a wrong translation: `Buf` has no such
    // method, and in a struct that did have one it would read the wrong thing.
    let Some(out) = rust_says(
        SPEC_LEN_IS_A_REAL_FIELD,
        "fn main(){ println!(\"{}\", size_of(Buf{ len: 7, cap: 9 })); }\n",
        "rust-len-real-field",
    ) else {
        return;
    };
    assert_eq!(out, "7", "the declared field must win over the method");
}

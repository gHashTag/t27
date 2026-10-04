//! A `[N]T` parameter must reach C with its SIZE, not as a bare pointer.
//!
//! `[4]u8` used to lower to `uint8_t*`, so a caller passing a two-element
//! array was silent. Rust keeps the size (`[u8; 4]`) and Zig keeps it
//! (`[4]u8`); C was the one backend that dropped it.
//!
//! The spelling that restores the check had to be MEASURED, not assumed. A
//! plain `T name[N]` parameter decays to a pointer and diagnoses NOTHING --
//! only `T name[static N]` does. So the assertions below are not on the text
//! alone: the text says the size is present, and then `cc` is asked whether a
//! short caller is actually rejected and a correct caller actually accepted.
//! A text-only test would pass on a spelling that checks nothing, which is
//! precisely the bug this file exists to prevent.

use std::collections::BTreeSet;
use std::process::Command;

fn cc_present() -> bool {
    Command::new("cc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("t27c-cstatic-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("create temp dir");
    d
}

fn gen_c(spec: &str, dir: &std::path::Path) -> String {
    let spec_path = dir.join("in.t27");
    std::fs::write(&spec_path, spec).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&spec_path)
        .output()
        .expect("run t27c");
    assert!(
        out.status.success(),
        "gen-c failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// Compile `header + caller` to an object file and return cc's diagnostics.
/// A compile that fails outright panics: that is a different result from a
/// warning.
///
/// `-c`, not `-fsyntax-only` (#4832, #5907). clang diagnoses a short
/// `[static N]` argument in its front end, so either works there; gcc does it
/// in a later pass that `-fsyntax-only` never reaches. Measured with gcc 15:
/// `-fsyntax-only` prints nothing for the short caller, `-c` prints
/// `[-Wstringop-overflow=]`.
fn diagnose(header: &str, caller: &str, tag: &str) -> String {
    let dir = tmp_dir(tag);
    let mut src = header.to_string();
    src.push('\n');
    src.push_str(caller);
    let c_path = dir.join("out.c");
    std::fs::write(&c_path, &src).expect("write C");
    let out = Command::new("cc")
        .args(["-std=c11", "-Wall", "-Wextra", "-Wno-unused-parameter", "-c", "-o"])
        .arg(dir.join("out.o"))
        .arg(&c_path)
        .output()
        .expect("run cc");
    let diag = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success() && !diag.contains("error:"),
        "{tag}: generated C did not compile ({}):\n{diag}",
        out.status
    );
    diag
}

/// The `-W...` names in a diagnostic's `[-W...]` tags.
fn warning_tags(diag: &str) -> BTreeSet<String> {
    diag.match_indices("[-W")
        .filter_map(|(i, _)| diag[i..].find(']').map(|j| diag[i + 1..i + j].to_string()))
        .collect()
}

/// The warning THIS `cc` gives a too-short `[static 4]` argument, asked of it
/// rather than named. clang calls it `-Warray-bounds` and gcc
/// `-Wstringop-overflow=`; the test used to demand clang's name, and on gcc
/// it got an empty diagnostic and failed for a reason that was not the C.
///
/// Hand-written C, two callers: one passes a 2-element array, the control
/// passes 4. The answer is the tags the first provokes and the control does
/// not, so a tag that fires on both -- `-Wunused-function` for the unused
/// static caller -- cannot be mistaken for the check.
fn short_argument_tags() -> BTreeSet<String> {
    let callee = "#include <stdint.h>\nuint8_t take4(uint8_t a[static 4]) { return a[0]; }\n";
    let short = warning_tags(&diagnose(
        callee,
        "static uint8_t c(void){ uint8_t two[2] = {0}; return take4(two); }",
        "probe-short",
    ));
    let exact = warning_tags(&diagnose(
        callee,
        "static uint8_t c(void){ uint8_t four[4] = {0}; return take4(four); }",
        "probe-exact",
    ));
    short.difference(&exact).cloned().collect()
}

const SPEC: &str = r#"
module arrp {
    const N : usize = 4;
    fn fixed(a: [4]u8, b: [N]i32) -> u8 { return a[0]; }
    fn slice(a: []u8) -> u8 { return a[0]; }
}
"#;

#[test]
fn fixed_array_param_carries_its_size() {
    let dir = tmp_dir("text");
    let h = gen_c(SPEC, &dir);
    assert!(
        h.contains("uint8_t a[static 4]"),
        "a literal size must survive into C:\n{h}"
    );
    assert!(
        h.contains("int32_t b[static N]"),
        "a const-named size must survive into C, spelled by its `#define`:\n{h}"
    );
    // The `#define` the const-named size relies on must be emitted BEFORE the
    // prototype that uses it, or `[static N]` expands to nothing.
    let def = h.find("#define N").expect("const N must reach C as a #define");
    let use_ = h.find("[static N]").expect("checked above");
    assert!(def < use_, "`#define N` must precede its use in a parameter");
}

#[test]
fn a_slice_param_stays_a_pointer() {
    // `[]u8` has no compile-time length; `[static]` would be a lie about it.
    let dir = tmp_dir("slice");
    let h = gen_c(SPEC, &dir);
    assert!(
        h.contains("uint8_t slice(uint8_t* a)"),
        "a slice must keep the pointer lowering:\n{h}"
    );
}

#[test]
fn a_short_caller_is_diagnosed_and_a_correct_one_is_not() {
    if !cc_present() {
        // An absent compiler is not a passing test. This assertion is the
        // whole point of the file: without cc, nothing here proves the
        // spelling CHECKS anything.
        eprintln!("SKIP a_short_caller_is_diagnosed_and_a_correct_one_is_not: no cc on PATH");
        return;
    }
    let tags = short_argument_tags();
    assert!(
        !tags.is_empty(),
        "this cc warns about no too-short [static 4] argument even in hand-written C, \
         so it cannot judge the generated one; nothing below would be a reading"
    );

    let dir = tmp_dir("gen");
    let h = gen_c(SPEC, &dir);

    let short = diagnose(
        &h,
        "static uint8_t c(void){ uint8_t two[2] = {0}; int32_t f[4] = {0}; return fixed(two,f); }",
        "short",
    );
    assert!(
        !warning_tags(&short).is_disjoint(&tags),
        "a 2-element array into a [static 4] parameter must be diagnosed with {tags:?}, got:\n{short}"
    );

    let exact = diagnose(
        &h,
        "static uint8_t c(void){ uint8_t f4[4] = {0}; int32_t f[4] = {0}; return fixed(f4,f); }",
        "exact",
    );
    assert!(
        warning_tags(&exact).is_disjoint(&tags),
        "a correctly-sized caller must NOT be diagnosed with {tags:?}, got:\n{exact}"
    );
}

#[test]
fn a_zero_length_param_keeps_the_pointer() {
    // `[static 0]` is not valid C: "static has no effect on zero-length arrays".
    let dir = tmp_dir("zero");
    let h = gen_c(
        "module z { fn f(a: [0]u8) -> u8 { return 0; } }",
        &dir,
    );
    assert!(
        !h.contains("[static 0]"),
        "a zero-length array must not be emitted as [static 0]:\n{h}"
    );
    if cc_present() {
        diagnose(&h, "", "zero");
    }
}

#[test]
fn a_struct_field_is_untouched() {
    // `param_type_to_c` also spells struct fields, where `[static N]` is a
    // syntax error. The narrowing to parameter position is load-bearing.
    let dir = tmp_dir("field");
    let h = gen_c(
        "module s { struct Buf { data : [4]u8, } fn f(b: Buf) -> u8 { return 0; } }",
        &dir,
    );
    let field_line = h
        .lines()
        .find(|l| l.contains("data"))
        .unwrap_or("")
        .to_string();
    assert!(
        !field_line.contains("[static"),
        "a struct field must not carry [static N]: {field_line}"
    );
    if cc_present() {
        diagnose(&h, "", "field");
    }
}

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

mod common;

use common::cc_present;
use std::process::Command;

/// How THIS `cc` can be made to diagnose an undersized `[static N]` argument.
///
/// Neither half of this is portable, and both were measured rather than
/// assumed:
///
///   clang  `-fsyntax-only`            -> `[-Warray-bounds]`
///   gcc    `-fsyntax-only`            -> SILENCE. It runs no analysis pass.
///   gcc    `-c` (even at `-O0`)       -> `[-Wstringop-overflow=]`
///
/// So the test cannot name a flag set or a warning. It asks the compiler, with
/// a hand-written reference pair that has nothing to do with t27, and keeps
/// whatever answer comes back.
struct Probe {
    flags: Vec<&'static str>,
    tag: String,
}

/// "Does this header compile at all?" -- no analysis pass required, so every
/// compiler can answer it. The tests that only need that use this directly.
const SYNTAX_ONLY: &[&str] = &[
    "-std=c11",
    "-Wall",
    "-Wextra",
    "-Wno-unused-parameter",
    "-fsyntax-only",
];

/// Deliberately undersized: a 2-element array into a `[static 4]` parameter.
const REF_SHORT: &str = "#include <stdint.h>\n\
static uint8_t ref_fixed(uint8_t a[static 4]) { return a[0]; }\n\
static uint8_t ref_call(void){ uint8_t two[2]; return ref_fixed(two); }\n\
uint8_t ref_use(void){ return ref_call(); }\n";

/// The same call, correctly sized. A flag set that flags this one too is
/// detecting nothing in particular and must not be chosen.
const REF_EXACT: &str = "#include <stdint.h>\n\
static uint8_t ref_fixed(uint8_t a[static 4]) { return a[0]; }\n\
static uint8_t ref_call(void){ uint8_t f4[4]; return ref_fixed(f4); }\n\
uint8_t ref_use(void){ return ref_call(); }\n";

/// Every `[-Wsomething]` name the compiler printed on a warning line.
///
/// All of them, not the first: at higher optimisation gcc also reports
/// `-Wuninitialized` on these very fixtures, and picking whichever line came
/// out on top would make the choice depend on diagnostic ordering.
fn warning_tags(diag: &str) -> Vec<String> {
    diag.lines()
        .filter(|l| l.contains("warning:"))
        .filter_map(|l| {
            let s = l.find("[-W")?;
            let e = l[s..].find(']')? + s;
            Some(l[s + 1..e].to_string())
        })
        .collect()
}

fn run_cc(flags: &[&str], src: &str, tag: &str) -> String {
    let dir = tmp_dir(tag);
    let c_path = dir.join("probe.c");
    std::fs::write(&c_path, src).expect("write C");
    let out = Command::new("cc")
        .args(flags)
        .arg(&c_path)
        .output()
        .expect("run cc");
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn probe() -> Option<Probe> {
    const CANDIDATES: [&[&str]; 2] = [
        SYNTAX_ONLY,
        // gcc needs to actually compile before it will look. The object goes
        // nowhere; only the diagnostics are wanted.
        &["-std=c11", "-Wall", "-Wextra", "-Wno-unused-parameter", "-c", "-o", "/dev/null"],
    ];
    for flags in CANDIDATES {
        let short = warning_tags(&run_cc(flags, REF_SHORT, "probe-short"));
        // The negative control, on the same flags. The tag we want is the one
        // the SHORT call provokes and the correctly-sized call does not; a tag
        // present in both is firing on something other than the size, and
        // would make every assertion below vacuous.
        let exact = warning_tags(&run_cc(flags, REF_EXACT, "probe-exact"));
        if let Some(tag) = short.into_iter().find(|t| !exact.contains(t)) {
            return Some(Probe { flags: flags.to_vec(), tag });
        }
    }
    None
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

/// Compile `header + caller` and return cc's diagnostics, or None if the
/// compile failed outright (which is a different result from a warning).
fn diagnose(header: &str, caller: &str, tag: &str, flags: &[&str]) -> String {
    let mut src = header.to_string();
    src.push('\n');
    src.push_str(caller);
    let diag = run_cc(flags, &src, tag);
    assert!(
        !diag.contains("error:"),
        "{tag}: generated C did not compile:\n{diag}"
    );
    diag
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
    let Some(p) = probe() else {
        // Not a skip. The file exists to prove the spelling CHECKS something,
        // and a compiler that will not diagnose a hand-written undersized
        // `[static 4]` call cannot be used to prove it. Naming the compiler
        // makes the next flag set obvious to whoever reads this.
        panic!(
            "no candidate flag set makes this cc diagnose an undersized [static N] \
             argument, so the spelling cannot be verified here.\ncc --version:\n{}",
            String::from_utf8_lossy(
                &Command::new("cc").arg("--version").output().expect("cc").stdout
            )
        );
    };

    let dir = tmp_dir("gen");
    let h = gen_c(SPEC, &dir);

    let short = diagnose(
        &h,
        "static uint8_t c(void){ uint8_t two[2]; int32_t f[4]; return fixed(two,f); }\n\
         uint8_t use_c(void){ return c(); }",
        "short",
        &p.flags,
    );
    assert!(
        short.contains(&p.tag),
        "a 2-element array into a [static 4] parameter must be diagnosed ({} with {:?}), got:\n{short}",
        p.tag,
        p.flags
    );

    let exact = diagnose(
        &h,
        "static uint8_t c(void){ uint8_t f4[4]; int32_t f[4]; return fixed(f4,f); }\n\
         uint8_t use_c(void){ return c(); }",
        "exact",
        &p.flags,
    );
    assert!(
        !exact.contains(&p.tag),
        "a correctly-sized caller must NOT be diagnosed ({}), got:\n{exact}",
        p.tag
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
        diagnose(&h, "", "zero", SYNTAX_ONLY);
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
        diagnose(&h, "", "field", SYNTAX_ONLY);
    }
}

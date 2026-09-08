//! A bare `{ 1, 2 }` is an INITIALISER, not an operand.
//!
//! C accepts it after `=` in a declaration and nowhere else, so `f({ 0 })` is
//! `expected expression` -- **194 call sites** in the corpus, the largest
//! single shape inside the largest error class (`expected expression`, 885).
//!
//! The repair is the C99 compound literal `(T[]){ ... }`, and the element type
//! comes from the CALLEE's declared parameter rather than from the elements,
//! so nothing is inferred. The cast is derived from `param_type_to_c`, which
//! means it agrees with the parameter's own declaration by construction.
//!
//! WHY ONLY THE ARGUMENT POSITION. The same cast in a `return` would hand back
//! the address of a block-scoped object: all 131 `return { ... }` sites in the
//! corpus have a POINTER return type, so repairing them there would turn a
//! loud syntax error into a silent dangling pointer (#3445). An argument's
//! compound literal lives in the caller's block, which outlives the call.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-carg-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    let p = d.join("in.t27");
    std::fs::write(&p, spec).expect("write");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&p)
        .output()
        .expect("t27c");
    assert!(out.status.success(), "gen-c failed: {}", String::from_utf8_lossy(&out.stderr));
    let h = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(!h.is_empty(), "gen-c produced an EMPTY header -- the fixture never reached the emitter");
    (h, d)
}

fn errors(h: &str, d: &std::path::Path) -> usize {
    let p = d.join("h.h");
    std::fs::write(&p, h).expect("write");
    let out = Command::new("cc")
        .args(["-std=c11", "-ferror-limit=0", "-fsyntax-only", "-x", "c"])
        .arg(&p)
        .output()
        .expect("cc");
    let text = String::from_utf8_lossy(&out.stderr).to_string();
    // The DIAGNOSTIC, not the word: clang echoes the offending source line.
    text.lines()
        .filter(|l| {
            let mut it = l.splitn(4, ':');
            it.next().is_some()
                && it.next().map_or(false, |s| s.trim().parse::<u32>().is_ok())
                && it.next().map_or(false, |s| s.trim().parse::<u32>().is_ok())
                && it.next().map_or(false, |s| s.trim_start().starts_with("error: "))
        })
        .count()
}

const DECLS: &str = "    fn sum(xs: []u32) -> u32 { return 0; }\n\
                     fn names(ns: [][]const u8) -> u32 { return 0; }\n\
                     fn fixed(a: [4]u32) -> u32 { return 0; }\n\
                     fn scalar(n: u32) -> u32 { return 0; }\n";

fn spec(body: &str) -> String {
    format!("module P {{\n{DECLS}    fn f(v: i32) -> u32 {{ {body} return 0; }}\n}}\n")
}

#[test]
fn an_array_literal_argument_gets_the_compound_literal_cast() {
    let (h, d) = gen_c(&spec("var a = sum([1, 2, 3]);"), "slice");
    assert!(h.contains("sum((uint32_t[]){ 1, 2, 3 })"), "got:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn the_cast_agrees_with_the_parameters_own_declaration() {
    // Stated as an agreement rather than as the literal `(uint8_t*[])`: the
    // cast is derived from the same mapper that wrote the parameter, so if
    // that mapping ever changes, both change together or this fails.
    let (h, _d) = gen_c(&spec("var b = names([\"x\", \"y\"]);"), "agree");
    let decl = h
        .lines()
        .find(|l| l.contains(" names(") && l.trim_end().ends_with(");"))
        .expect("prototype")
        .to_string();
    let param = decl.split('(').nth(1).unwrap().split(' ').next().unwrap().to_string();
    let elem = param.strip_suffix('*').expect("a slice parameter is a pointer");
    assert!(
        h.contains(&format!("names(({}[])", elem)),
        "the cast must spell the parameter's own element type ({elem}):\n{h}"
    );
}

#[test]
fn a_fixed_size_array_parameter_is_refused() {
    // THE DISCRIMINATING CASE for the `[]` test. `[4]u32` is a different
    // lowering -- a by-value struct in some positions -- and casting to
    // `(uint32_t[])` there would be a type error rather than a repair.
    let (h, _d) = gen_c(&spec("var c = fixed([1, 2, 3, 4]);"), "fixed");
    assert!(h.contains("fixed({ 1, 2, 3, 4 })"), "left as written:\n{h}");
    assert!(!h.contains("fixed((uint32_t[])"), "no cast for a [N]T parameter:\n{h}");
}

#[test]
fn the_literal_is_typed_by_its_own_position_not_the_first() {
    // 63 of the 194 corpus sites pass the literal as a LATER argument. A
    // repair that reads `params[0]` passes every single-argument test and is
    // wrong for all of them -- and would type this list by `n`, a scalar,
    // dropping the cast entirely.
    let (h, d) = gen_c(
        "module P {\n    fn take(n: u32, xs: []u32) -> u32 { return 0; }\n         \x20   fn f(v: i32) -> u32 { var a = take(7, [1, 2, 3]); return 0; }\n}\n",
        "pos",
    );
    assert!(h.contains("take(7, (uint32_t[]){ 1, 2, 3 })"), "got:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn a_scalar_parameter_is_untouched() {
    let (h, _d) = gen_c(&spec("var d = scalar(1);"), "scalar");
    assert!(h.contains("scalar(1)"), "got:\n{h}");
    assert!(!h.contains("scalar((uint32_t[])"), "no cast where nothing is an array:\n{h}");
}

#[test]
fn a_callee_this_module_does_not_declare_is_refused() {
    // The parameter types are per translation unit. A call into another spec
    // has no declared type to read, and inventing one from the elements is the
    // guess this repair exists to avoid.
    let (h, _d) = gen_c(&spec("var e = elsewhere([1, 2]);"), "unknown");
    assert!(!h.contains("elsewhere((uint32_t[])"), "no cast without a declaration:\n{h}");
    assert!(h.contains("elsewhere({ 1, 2 })"), "left as written:\n{h}");
}

#[test]
fn a_returned_array_literal_is_not_cast_to_a_slice() {
    // The deliberate limitation, pinned. All 131 `return { ... }` sites in the
    // corpus have a pointer return type; a compound literal there is the
    // address of a block-scoped object (#3445). A loud syntax error is better
    // than a quiet dangling pointer, so this must NOT acquire a cast by
    // someone widening the rule without reading why it is narrow.
    let (h, _d) = gen_c(
        "module P {\n    fn mk(v: i32) -> []u32 { return [1, 2, 3]; }\n}\n"
            .to_string()
            .as_str(),
        "ret",
    );
    // Positive half first, so the negative cannot pass vacuously: if the
    // returned shape ever stops being a bare brace list, this test stops
    // testing what it claims and says so.
    assert!(h.contains("return { 1, 2, 3 };"), "the fixture must still produce the shape under test:\n{h}");
    assert!(!h.contains("return (uint32_t[]){"), "a return must not become a compound literal:\n{h}");
}

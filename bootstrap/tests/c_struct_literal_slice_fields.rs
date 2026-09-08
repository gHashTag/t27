//! A slice field is a POINTER in C, and a brace list is not a pointer.
//!
//! `(WeightBank){ .data = { 1, 2, 3, 4 } }` reads
//! *incompatible integer to pointer conversion initializing* -- **199 sites in
//! 9 files**, a class whose diagnostics and distinct lines are equal, so every
//! one of them is its own site with no cascade.
//!
//! The repair is the C99 compound literal, and the element type comes from the
//! FIELD's declaration through the same helper a call argument uses.
//!
//! WHERE IT IS REFUSED, and why the position matters: a compound literal at
//! block scope dies with its block. Of the 199 sites, **197 initialise a local
//! and 4 sit inside a `return`** -- and in a `return` the cast would hand back
//! the address of a local (#3445). So a flag says whether the operand of a
//! `return` is being emitted, and there are TWO return arms in this backend:
//! the statement and the expression. The first version of the guard set the
//! flag in one of them, and the returned literal took the cast anyway.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-slicefield-{tag}-{}-{}",
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
    assert!(!h.is_empty(), "gen-c produced an EMPTY header");
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
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| {
            let mut it = l.splitn(4, ':');
            it.next().is_some()
                && it.next().map_or(false, |s| s.trim().parse::<u32>().is_ok())
                && it.next().map_or(false, |s| s.trim().parse::<u32>().is_ok())
                && it.next().map_or(false, |s| s.trim_start().starts_with("error: "))
        })
        .count()
}

const DECL: &str = "    struct Bank { depth : u32, data : []i32, fixed : [2]u32, }\n";

#[test]
fn a_slice_field_gets_the_compound_literal_cast() {
    let (h, d) = gen_c(
        &format!("module B {{\n{DECL}    fn f(v: i32) -> u32 {{ var b = Bank{{ .depth = 2, .data = [1, 2, 3, 4] }}; return b.depth; }}\n}}\n"),
        "local",
    );
    assert!(h.contains(".data = (int32_t[]){ 1, 2, 3, 4 }"), "got:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn the_cast_agrees_with_the_fields_own_declaration() {
    // Derived from the same mapper that wrote the field, so the two cannot
    // drift apart: whatever `data` is declared as, the cast spells its element.
    let (h, _d) = gen_c(
        &format!("module B {{\n{DECL}    fn f(v: i32) -> u32 {{ var b = Bank{{ .data = [1] }}; return b.depth; }}\n}}\n"),
        "agree",
    );
    let decl = h
        .lines()
        .find(|l| l.trim_end().ends_with("data;"))
        .expect("the field declaration")
        .trim()
        .to_string();
    let elem = decl.split_whitespace().next().unwrap().trim_end_matches('*').to_string();
    assert!(
        h.contains(&format!(".data = ({}[])", elem)),
        "the cast must spell the field's own element type ({elem}):\n{h}"
    );
}

#[test]
fn a_returned_struct_literal_is_refused() {
    // THE DISCRIMINATING CASE. 4 of the 199 corpus sites are here, and a
    // compound literal in a `return` is the address of a block-scoped object
    // (#3445). A loud diagnostic beats a quiet dangling pointer.
    let (h, _d) = gen_c(
        &format!("module B {{\n{DECL}    fn g(v: i32) -> Bank {{ return Bank{{ .depth = 1, .data = [7, 8] }}; }}\n}}\n"),
        "ret",
    );
    assert!(
        h.contains("return (Bank){ .depth = 1, .data = { 7, 8 } };"),
        "a returned struct literal keeps the bare braces:\n{h}"
    );
    assert!(!h.contains("return (Bank){ .depth = 1, .data = (int32_t[])"), "no cast in a return:\n{h}");
}

#[test]
fn a_non_slice_field_is_untouched() {
    // `[2]u32` is a fixed array, a different lowering, and a `(uint32_t[])`
    // cast there would be a type error rather than a repair.
    let (h, _d) = gen_c(
        &format!("module B {{\n{DECL}    fn f(v: i32) -> u32 {{ var b = Bank{{ .fixed = [1, 2] }}; return b.depth; }}\n}}\n"),
        "fixed",
    );
    assert!(h.contains(".fixed = { 1, 2 }"), "left as written:\n{h}");
    assert!(!h.contains(".fixed = (uint32_t[])"), "no cast for a [N]T field:\n{h}");
}

#[test]
fn a_struct_literal_nested_inside_a_return_is_also_refused() {
    // The flag has to survive nesting: a literal built INSIDE the returned
    // expression is just as block-scoped as the outer one.
    let (h, _d) = gen_c(
        &format!("module B {{\n{DECL}    struct Pair {{ a : Bank, }}\n\
         \x20   fn h(v: i32) -> Pair {{ return Pair{{ .a = Bank{{ .data = [3] }} }}; }}\n}}\n"),
        "nested",
    );
    assert!(
        !h.contains("(int32_t[]){ 3 }"),
        "a nested literal in a return must not be cast either:\n{h}"
    );
}

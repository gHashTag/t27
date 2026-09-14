//! A field of a `*T` is reached with `->`, and C is the only backend that
//! spells the two accesses differently.
//!
//! Rust and Zig both auto-dereference, so the dot travelled into C unchanged:
//! **469 diagnostics over 389 distinct lines in 52 files**, which clang answers
//! with *"is a pointer; did you mean to use '->'?"*. Splitting those lines by
//! what follows the dot gives **214 real field accesses** -- where `->` is
//! exactly the repair -- and 175 method calls (`len` 165, `push`, `length`),
//! which are a different family (#3464) and are left alone here: turning their
//! dot into an arrow moves the error without answering it.
//!
//! Of the 214, **152** have a base that is a parameter declared `*T`. That is
//! the population this rule reaches, and it is measured in the terms the rule
//! actually evaluates.
//!
//! THE DISCRIMINATING CASE is a name that is a pointer in one item and a value
//! in the next. The set is per item, and the first version cleared it in
//! `gen_c_fn` only -- so a test block's own `MemoryCell cell;` inherited the
//! `cell` of a preceding `*MemoryCell` parameter and was written `cell->scope`:
//! **+38 errors in one file**, the only regression in the corpus and the reason
//! the bench emitter (which cleared NEITHER set) is now cleared too.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-arrow-{tag}-{}-{}",
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

const TYPES: &str = "    struct Cell { scope : u8, }\n    struct Box { inner : Cell, }\n";

#[test]
fn a_field_of_a_pointer_parameter_uses_an_arrow() {
    let (h, d) = gen_c(
        &format!("module P {{\n{TYPES}    fn touch(cell: *Cell, s: u8) bool {{ cell.scope = s; return true; }}\n}}\n"),
        "arrow",
    );
    assert!(h.contains("cell->scope = s;"), "got:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn a_field_of_a_value_parameter_keeps_the_dot() {
    // THE DISCRIMINATING CASE for the pointer test: the same field, the same
    // struct, reached through a parameter that is not a pointer.
    let (h, d) = gen_c(
        &format!("module P {{\n{TYPES}    fn byvalue(cell: Cell) u8 {{ return cell.scope; }}\n}}\n"),
        "value",
    );
    assert!(h.contains("return cell.scope;"), "got:\n{h}");
    assert!(!h.contains("cell->"), "a value must not acquire an arrow:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn only_the_pointer_hop_becomes_an_arrow() {
    // `b->inner.scope`, not `b->inner->scope`. The rule keys on the base being
    // an IDENTIFIER in the pointer set; the outer access's base is a field
    // access, whose type this pass does not track.
    let (h, d) = gen_c(
        &format!("module P {{\n{TYPES}    fn nested(b: *Box) u8 {{ return b.inner.scope; }}\n}}\n"),
        "nested",
    );
    assert!(h.contains("return b->inner.scope;"), "got:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn a_field_whose_name_is_a_pointer_parameter_keeps_the_dot() {
    // The case that makes the BASE-IS-AN-IDENTIFIER check load-bearing rather
    // than decorative. A field-access node carries the FIELD's name, so a rule
    // that only asks `pointer_typed_names.contains(&b.name)` reads `b.inner`
    // as if `inner` were the base -- and `inner` is a pointer parameter here.
    // `b` is a value, so `b.inner.scope` must keep both dots.
    let (h, d) = gen_c(
        &format!("module P {{\n{TYPES}    fn collide(inner: *Cell, b: Box) u8 {{ return b.inner.scope; }}\n}}\n"),
        "collide",
    );
    assert!(h.contains("return b.inner.scope;"), "got:\n{h}");
    assert!(!h.contains("b.inner->scope"), "the field's own name must not decide the access:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn a_slice_parameter_keeps_the_dot() {
    // Deliberate. A slice is also a C pointer, but `xs.len` is not a struct
    // field at all -- it is the `len` family (#3464). An arrow would move that
    // error rather than answer it, and the loud form names the real problem.
    let (h, _d) = gen_c(
        &format!("module P {{\n{TYPES}    fn onslice(xs: []Cell) u8 {{ return xs.len; }}\n}}\n"),
        "slice",
    );
    assert!(h.contains("return xs.len;"), "got:\n{h}");
    assert!(!h.contains("xs->len"), "a slice must not acquire an arrow:\n{h}");
}

#[test]
fn the_pointer_set_does_not_leak_into_a_test_block() {
    // The regression the corpus caught: `cell` stayed in the set from the
    // preceding `*Cell` parameter, and the test block's own value-typed `cell`
    // was written `cell->scope` -- +38 errors in one file.
    let (h, d) = gen_c(
        &format!(
            "module P {{\n{TYPES}    fn touch(cell: *Cell, s: u8) bool {{ cell.scope = s; return true; }}\n\
             \x20   test \"same name, a value\" {{ var cell: Cell = Cell{{ .scope = 1 }}; assert(cell.scope == 1); }}\n}}\n"
        ),
        "leak-test",
    );
    assert!(h.contains("cell->scope = s;"), "the fn must still use an arrow:\n{h}");
    assert!(h.contains("assert((cell.scope == 1))"), "the test block's own local is a VALUE:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn the_pointer_set_does_not_leak_into_a_bench_block() {
    // The bench emitter cleared NEITHER per-item set before this change --
    // the same boundary as a fn and a test, missing the same reset.
    let (h, d) = gen_c(
        &format!(
            "module P {{\n{TYPES}    fn touch(cell: *Cell, s: u8) bool {{ cell.scope = s; return true; }}\n\
             \x20   bench \"same name, a value\" {{ var cell: Cell = Cell{{ .scope = 2 }}; var s: u8 = cell.scope; }}\n}}\n"
        ),
        "leak-bench",
    );
    // No `assert` in the bench body on purpose: `#include <assert.h>` is
    // emitted only when the module has a TEST block, so a bench that asserts
    // does not compile on its own. Latent -- all 29 specs whose bench calls
    // `assert` also have a test block -- and out of scope here.
    assert!(h.contains("cell->scope = s;"), "the fn must still use an arrow:\n{h}");
    assert!(h.contains("= cell.scope;"), "the bench block's own local is a VALUE:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

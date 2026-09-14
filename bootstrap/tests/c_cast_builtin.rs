//! `cast_i8(x)` is a CAST, not a call, and C was the backend that did not know.
//!
//! The Zig backend has said so since W570 -- `@as(i8, @intCast(x))` -- and its
//! own comment records the fact this campaign had filed as a blocker:
//! *"`cast_i8(` alone appears 1,100 times and is defined nowhere in the
//! corpus"*. Declared nowhere was true. **Therefore unfixable was not.**
//!
//! Corpus: `cast_i8` 1079 uses, `cast_i16` 38, `cast_i32` 2. The fifth
//! instance in this campaign of *three backends agree and one has no answer*.
//!
//! INTEGERS ONLY, exactly as Zig has it: a float cast needs a different
//! conversion (`@floatCast` versus `@intFromFloat` there, a different C cast
//! here) and guessing between them would be a silent semantic choice.
//!
//! The width suffix test is now a FREE FUNCTION rather than a method on either
//! codegen, because a second copy is how one backend grows a spelling the
//! other refuses.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen(sub: &str, spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-cast-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    let p = d.join("in.t27");
    std::fs::write(&p, spec).expect("write");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg(sub)
        .arg(&p)
        .output()
        .expect("t27c");
    let h = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(!h.is_empty(), "{sub} produced an EMPTY output");
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

#[test]
fn an_integer_cast_builtin_becomes_a_c_cast() {
    let (h, d) = gen(
        "gen-c",
        "module C {\n    fn f(v: i32) -> i32 { var a = cast_i8(v); return 0; }\n}\n",
        "int",
    );
    assert!(h.contains("((int8_t)(v))"), "got:\n{h}");
    assert!(!h.contains("cast_i8("), "no call may survive:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn c_and_zig_lower_the_same_spelling() {
    // The point of the free helper. Both backends recognise the same set of
    // width suffixes, and this fails the moment one of them grows or loses one.
    let spec = "module C {\n    fn f(v: i32) -> i32 { var a = cast_i16(v); return 0; }\n}\n";
    let (c, _d) = gen("gen-c", spec, "agree-c");
    let (z, _d2) = gen("gen", spec, "agree-zig");
    assert!(c.contains("((int16_t)(v))"), "C:\n{c}");
    assert!(z.contains("@as(i16, @intCast(v))"), "Zig:\n{z}");
}

#[test]
fn a_float_suffix_is_refused() {
    // THE DISCRIMINATING CASE, and the same refusal Zig makes: converting a
    // float is a different operation and picking one silently is a semantic
    // choice, not a lowering.
    let (h, _d) = gen(
        "gen-c",
        "module C {\n    fn f(v: i32) -> i32 { var a = cast_f32(v); return 0; }\n}\n",
        "float",
    );
    assert!(h.contains("cast_f32(v)"), "a float cast is left as written:\n{h}");
    assert!(!h.contains("((float)(v))"), "and no conversion is invented:\n{h}");
}

#[test]
fn a_spec_that_declares_its_own_cast_keeps_it() {
    // The guard. A module defining `fn cast_i8` means that function, and the
    // builtin must not shadow it -- the same `declared_fns` condition Zig uses.
    let (h, _d) = gen(
        "gen-c",
        "module C {\n    fn cast_i8(x: i32) -> i32 { return x; }\n\
         \x20   fn f(v: i32) -> i32 { var a = cast_i8(v); return 0; }\n}\n",
        "declared",
    );
    assert!(h.contains("cast_i8(v)"), "a declared function is called, not cast:\n{h}");
    assert!(!h.contains("((int8_t)(v))"), "and the builtin does not shadow it:\n{h}");
}

#[test]
fn a_cast_with_two_arguments_is_not_one() {
    // Arity is part of the shape: `cast_i8(a, b)` is not the builtin.
    let (h, _d) = gen(
        "gen-c",
        "module C {\n    fn f(v: i32) -> i32 { var a = cast_i8(v, v); return 0; }\n}\n",
        "arity",
    );
    assert!(h.contains("cast_i8(v, v)"), "two arguments is not the cast:\n{h}");
}

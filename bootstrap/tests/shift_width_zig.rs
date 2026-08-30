//! A shift with a runtime amount was pinned to a width guessed from the
//! literal's magnitude, not from the type the declaration states.
//!
//! Zig has no `comptime_int << runtime`, so the left operand of such a shift
//! must carry a width. With nothing else to go on the emitter used
//! `zig_int_literal_default_type`, which reads the literal's value: `1` fits in
//! u32, therefore u32. So
//!
//!     var half: i32 = 1 << d;
//!
//! became `var half: i32 = @as(u32, 1) << @intCast(d);` -- a u32 expression
//! initialising an i32, which Zig rejects.
//!
//! The declared type was in hand two lines above the expression that had to
//! guess it. Measured on the corpus: 58 emitted sites carry `@as(u32, 1) <<`,
//! and 33 of them are a declaration that states its type -- all of them `i32`.
//!
//! Per spec, both directions, zig 0.16.0, no timeouts:
//!
//!     zig test --test-no-exec    133 -> 165   +32   regressions 0
//!     zig build-obj -fno-emit-bin 282 -> 282    +0   regressions 0
//!     cc -fsyntax-only (control)  268 -> 268    +0   regressions 0
//!
//! The second line is the honest part and was predicted before the fix: the
//! corpus acceptance column is measured with `build-obj`, which resolves
//! identifiers and never Sema-analyses a body nothing references, so it cannot
//! see this defect and does not move when it is repaired. Reporting only the
//! +32 would overstate what the repository's own column will show; reporting
//! only the +0 would say a real repair did nothing.

use std::io::Write;
use std::process::Command;

fn gen_zig(src: &str) -> String {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("t27-shiftwidth-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("m.t27");
    let mut f = std::fs::File::create(&path).expect("write spec");
    f.write_all(src.as_bytes()).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen")
        .arg(&path)
        .output()
        .expect("run t27c");
    let _ = std::fs::remove_dir_all(&dir);
    String::from_utf8_lossy(&out.stdout).to_string()
}

const SIGNED: &str = "module m\n\nfn f(d: u32) -> i32 {\n    var half: i32 = 1 << d\n    return half\n}\n";

#[test]
fn the_declared_type_pins_the_shift_not_the_literal_magnitude() {
    let z = gen_zig(SIGNED);
    assert!(
        z.contains("@as(i32, 1) <<"),
        "expected the declaration's own type, got:\n{z}"
    );
    assert!(
        !z.contains("@as(u32, 1) <<"),
        "the magnitude default is still winning:\n{z}"
    );
}

/// An unsigned declaration keeps the width it states, which happens to agree
/// with the magnitude default -- so this case cannot tell the two rules apart
/// and is here to pin that it did not regress.
#[test]
fn an_unsigned_declaration_is_unchanged() {
    let z = gen_zig("module m\n\nfn f(d: u32) -> u32 {\n    var mask: u32 = 1 << d\n    return mask\n}\n");
    assert!(z.contains("@as(u32, 1) <<"), "got:\n{z}");
}

/// A wider declaration than the literal needs. The magnitude rule gives u32 for
/// `1`; the declaration says u64, and the declaration wins.
#[test]
fn a_wider_declaration_wins_over_the_magnitude() {
    let z = gen_zig("module m\n\nfn f(d: u32) -> u64 {\n    var mask: u64 = 1 << d\n    return mask\n}\n");
    assert!(z.contains("@as(u64, 1) <<"), "got:\n{z}");
}

/// A literal that states its OWN width keeps it. The suffix is more specific
/// than the surrounding declaration, and reading the declaration first would
/// undo the repair that recorded suffixes in the first place.
#[test]
fn a_suffixed_literal_beats_the_declaration() {
    let z = gen_zig("module m\n\nfn f(d: u32) -> u64 {\n    var mask: u64 = 1u32 << d\n    return mask\n}\n");
    assert!(z.contains("@as(u32, 1) <<"), "the suffix must win:\n{z}");
}

/// The hint must not leak past its declaration.
///
/// The first version of this test put a SECOND declaration after the first and
/// asserted the hint was gone. It passed under its own mutation: an untyped
/// declaration overwrites the hint with None on its way in, so removing the
/// explicit clear changed nothing and the test could not see it.
///
/// What bites is a shift that is not a declaration at all. `return 1 << e` in a
/// u32-returning function emits `@as(i32, 1)` when the previous statement's
/// i32 hint survives -- verified against the mutant, which produces exactly
/// that. The amounts differ on purpose: with the same amount twice, common
/// subexpression elimination hoists one `_cse1` and there is only one shift
/// left to be wrong about.
#[test]
fn the_hint_does_not_leak_past_its_declaration() {
    let src = "module m\n\nfn f(d: u32, e: u32) -> u32 {\n    var a: i32 = 1 << d\n    return 1 << e\n}\n";
    let z = gen_zig(src);
    assert!(
        z.contains("var a: i32 = @as(i32, 1) <<"),
        "the declaration should still be pinned to i32:\n{z}"
    );
    let ret = z
        .lines()
        .find(|l| l.trim_start().starts_with("return") && l.contains("<<"))
        .unwrap_or_else(|| panic!("no shifting return in:\n{z}"));
    assert!(
        ret.contains("@as(u32, 1)"),
        "the i32 hint leaked out of the declaration above: {ret}"
    );
}

/// A declaration whose type is not an integer must not pin the literal to it.
/// `@as(SomeStruct, 1)` does not compile, and emitting it would trade one
/// rejected file for another.
#[test]
fn a_non_integer_declaration_falls_back_to_the_magnitude() {
    let src = "module m\n\nstruct S {\n    v: u32,\n}\n\nfn f(d: u32) -> u32 {\n    var s: f64 = 1 << d\n    return 0\n}\n";
    let z = gen_zig(src);
    assert!(
        !z.contains("@as(f64, 1) <<"),
        "a float is not a width to pin a shift to:\n{z}"
    );
}

/// A literal shift amount is left alone: `1 << 3` is comptime on both sides and
/// needs no pin at all. Byte-identical output for those sites is what kept the
/// original change from moving every generated file.
#[test]
fn a_comptime_amount_is_not_pinned() {
    let z = gen_zig("module m\n\nfn f() -> i32 {\n    var half: i32 = 1 << 3\n    return half\n}\n");
    assert!(!z.contains("@as("), "a comptime shift needs no cast:\n{z}");
}

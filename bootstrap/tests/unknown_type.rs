//! A type name that resolves to nothing was accepted in silence.
//!
//! `struct S { a: NoSuchTypeAnywhere }` typechecked as
//! "Typecheck OK (0 errors, 0 warnings)". The typechecker resolved the name to
//! `TypeInfo::Custom(..)` and asked no further question, so the first reader to
//! notice was rustc -- downstream, in another language, and only for the specs
//! whose emitted Rust got far enough to be type-checked at all.
//!
//! Measured over the corpus: 62 distinct undefined type names in 61 of 651
//! specs by rustc's reckoning, and 84 names in 169 specs by this check's --
//! which sees the whole spec rather than only what the emitter managed to emit.
//! Of the headline names, `Float`, `Int` and `Bool` are declared by NO spec at
//! all, while `Trit` is declared by four and not imported by the files that
//! use it.
//!
//! Reported as warnings: across all 651 specs the exit code of `check` changes
//! for zero of them.

use std::io::Write;
use std::process::Command;

/// Typecheck a source string through the SHIPPED binary and return the unknown
/// type warnings. Through the binary, as its neighbour does: this crate has no
/// lib target, and a test that reimplemented the check would pass against a
/// compiler that never shipped it.
fn unknown_types(src: &str) -> Vec<String> {
    static SCRATCH_N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let scratch_n = SCRATCH_N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "t27-unknowntype-{}-{}",
        std::process::id(),
        scratch_n
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("m.t27");
    let mut f = std::fs::File::create(&path).expect("write spec");
    f.write_all(src.as_bytes()).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("typecheck")
        .arg(&path)
        .output()
        .expect("run t27c");
    let _ = std::fs::remove_dir_all(&dir);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    text.lines()
        .filter(|l| l.contains("unknown type"))
        .map(|l| l.to_string())
        .collect()
}

#[test]
fn a_field_naming_a_type_that_does_not_exist_is_reported() {
    let w = unknown_types("module m {\n    struct S { a: NoSuchTypeAnywhere, }\n}\n");
    assert_eq!(w.len(), 1, "expected one warning, got {w:?}");
    assert!(w[0].contains("NoSuchTypeAnywhere"), "{}", w[0]);
    assert!(w[0].contains("field `a`"), "the warning must say WHERE: {}", w[0]);
}

#[test]
fn a_parameter_and_a_return_type_are_both_checked() {
    let w = unknown_types("module m {\n    fn f(x: Missing1) -> Missing2 { }\n}\n");
    assert_eq!(w.len(), 2, "parameter and return type, got {w:?}");
}

#[test]
fn a_declared_struct_is_not_reported() {
    // The whole value of the check is that it does not cry wolf.
    let w = unknown_types(
        "module m {\n    struct Point { x: i32, }\n    fn f(p: Point) -> i32 { return p.x; }\n}\n",
    );
    assert!(w.is_empty(), "a declared type must not warn: {w:?}");
}

#[test]
fn a_type_alias_spelled_as_a_const_is_a_declaration() {
    // `pub const PackedTrit = u8;  // Type alias` and
    // `pub const Trit = enum(i8) { .. }` are type declarations written as
    // constants, and neither carries a type ANNOTATION -- which is what
    // separates them from `pub const ONE : i8 = 1`. Missing this form produced
    // 33 false warnings on specs/base/types.t27 alone.
    let w = unknown_types(
        "module m {\n    pub const Word = u8;\n    struct S { a: Word, }\n}\n",
    );
    assert!(w.is_empty(), "a const type alias must count as declared: {w:?}");
}

#[test]
fn the_language_own_keyword_spellings_are_not_unknown() {
    // TWO resolvers, and they disagree. `resolve_type_str` knows 15 spellings;
    // the emitter also knows `int`, `float` and `string`. Asking only the first
    // reported all three as unknown types in specs/ar/coa_planning.t27, which
    // the emitter lowers correctly to i32, f64 and &'static str.
    let w = unknown_types(
        "module m {\n    struct S { a: int, b: float, c: string, d: bool, }\n}\n",
    );
    assert!(w.is_empty(), "keyword spellings must not warn: {w:?}");
}

#[test]
fn an_array_of_a_declared_type_is_not_unknown() {
    // `resolve_type_str` matches whole strings, so `[]Point` and `[3]Point`
    // both fall through to `Custom("[]Point")` -- the wrapper, not the type. A
    // check reading that directly would report every array as unresolved.
    let w = unknown_types(
        "module m {\n    struct Point { x: i32, }\n    fn f(a: []Point, b: [3]Point) -> i32 { return 0; }\n}\n",
    );
    assert!(w.is_empty(), "array wrappers must be stripped: {w:?}");
}

/// The check must see what the BACKENDS compile, not what the file literally
/// holds.
///
/// `typecheck` read the raw source while every `gen-*` path resolves `use`
/// first, so a type arriving through an import read as undeclared. Measured
/// over the corpus: **1283 unknown-type warnings before, 1045 after** -- 238 of
/// them were about types the spec correctly imports. `specs/base/ternary_add.t27`
/// alone went from 10 warnings to 0.
///
/// The resolve carries the same safety contract every backend does: it may only
/// ADD declarations. When the spliced source does not parse the original is
/// used and the fallback says so -- exactly 1 of 651 specs takes that path, and
/// the exit code of `check` changes for none of them.
#[test]
fn a_type_that_arrives_through_an_import_is_not_unknown() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("bootstrap has a parent")
        .to_path_buf();
    let spec = root.join("specs/base/ternary_add.t27");
    if !spec.exists() {
        // Loudly: an absent input is not a passing test.
        eprintln!("SKIP: specs/base/ternary_add.t27 not in this tree");
        return;
    }
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("typecheck")
        .arg(&spec)
        .output()
        .expect("run t27c");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let warned: Vec<&str> = text.lines().filter(|l| l.contains("unknown type")).collect();
    assert!(
        warned.is_empty(),
        "this spec writes `use base::types;` and every type it names comes from \
         there; it warned {} times:\n{}",
        warned.len(),
        warned.join("\n")
    );
}

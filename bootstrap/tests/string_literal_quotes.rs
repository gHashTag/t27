//! The C and Rust emitters dropped the quotes on every string literal.
//!
//! The lexer strips the surrounding quotes and stores the raw text, tagging the
//! node `extra_kind == "string"`. Zig reads that tag, Verilog reads it, the
//! typechecker reads it — the C and Rust emitters never did:
//!
//!     let s = "hello world"          ->  __auto_type s = hello world;
//!     const NAME : str = "trinity"   ->  #define NAME trinity
//!                                    ->  pub const NAME: String = trinity;
//!
//! The `#define` is the worst of the three. A macro whose body is a bare word is
//! **valid C**: it lands in the output, compiles, and fails wherever it is used —
//! or expands to something else entirely.
//!
//! Measured per spec, both directions, over a snapshot of the 665-spec corpus:
//!
//!     cc -fsyntax-only -std=gnu11   268 -> 290   +22   regressions 0
//!     rustc --emit=metadata         223 -> 224    +1   regressions 0
//!
//! The C number was derived independently twice — by an adversarial sweep and by
//! this change — and agrees at 22. The Rust half had not been measured by anyone.

use std::io::Write;
use std::process::Command;

fn gen(backend: &str, src: &str) -> String {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("t27-strquote-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("m.t27");
    let mut f = std::fs::File::create(&path).expect("write spec");
    f.write_all(src.as_bytes()).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg(backend)
        .arg(&path)
        .output()
        .expect("run t27c");
    let _ = std::fs::remove_dir_all(&dir);
    String::from_utf8_lossy(&out.stdout).to_string()
}

const SRC: &str = "module m\n\nconst NAME : str = \"trinity\"\n\nfn f() -> u32 {\n    let s = \"hello world\"\n    return 1\n}\n";

#[test]
fn c_keeps_the_quotes_on_a_local() {
    let c = gen("gen-c", SRC);
    assert!(c.contains("\"hello world\""), "quotes dropped:\n{c}");
    assert!(!c.contains("= hello world;"), "bare identifier still emitted:\n{c}");
}

/// The `#define` path is a SECOND call site, and it was still wrong after the
/// expression emitter was fixed: `c_literal` takes a `&str` and so cannot see a
/// tag that lives on the node. Fixing the expression arm alone leaves this one.
#[test]
fn c_keeps_the_quotes_on_a_define() {
    let c = gen("gen-c", SRC);
    assert!(
        c.contains("#define NAME \"trinity\""),
        "a #define of a bare word is valid C and fails at its use site:\n{c}"
    );
}

#[test]
fn rust_keeps_the_quotes() {
    let r = gen("gen-rust", SRC);
    assert!(r.contains("\"hello world\""), "quotes dropped:\n{r}");
    assert!(r.contains("\"trinity\""), "const quotes dropped:\n{r}");
    assert!(!r.contains("= trinity;"), "bare identifier still emitted:\n{r}");
}

/// Zig is the control: it already read the tag, and must be byte-unaffected.
#[test]
fn zig_was_already_right_and_is_unchanged() {
    let z = gen("gen", SRC);
    assert!(z.contains("\"trinity\"") && z.contains("\"hello world\""), "got:\n{z}");
}

/// The lexer UNESCAPES as it reads, so the value arriving at the emitter holds a
/// real newline and a real quote. Writing those back raw produces a string
/// literal that spans lines, which no target accepts.
///
/// The first version of this test tried to locate "the line with the escape" and
/// asserted on it. It failed on correct output, because the line it found was
/// the wrong one -- the emitters were right and the test was not. Asserting the
/// exact expected literal is both simpler and impossible to get subtly wrong.
#[test]
fn escapes_are_written_back_escaped() {
    // Source: let s = "a\nb\"c"  -- a newline escape and an escaped quote.
    let src = "module m\n\nfn f() -> u32 {\n    let s = \"a\\nb\\\"c\"\n    return 1\n}\n";
    let expected = "\"a\\nb\\\"c\"";
    for backend in ["gen-c", "gen-rust", "gen"] {
        let out = gen(backend, src);
        assert!(
            out.contains(expected),
            "{backend} did not write {expected} back:\n{out}"
        );
        assert!(
            !out.contains("= \"a\nb"),
            "{backend} wrote a real newline inside a string literal:\n{out}"
        );
    }
}

/// A NUMBER must not gain quotes. The tag is what separates the two, and a rule
/// that quoted every literal would trade one broken backend for two.
#[test]
fn a_number_is_not_quoted() {
    let src = "module m\n\nconst N : u32 = 42\n\nfn f() -> u32 {\n    let x = 7\n    return x\n}\n";
    let c = gen("gen-c", src);
    let r = gen("gen-rust", src);
    assert!(c.contains("#define N 42"), "C quoted a number:\n{c}");
    assert!(!c.contains("\"42\"") && !c.contains("\"7\""), "C quoted a number:\n{c}");
    assert!(!r.contains("\"42\"") && !r.contains("\"7\""), "Rust quoted a number:\n{r}");
}

/// And the `_` separator handling the C emitter already had must survive: it is
/// on the same arm, and an early `return` for strings must not skip it.
#[test]
fn the_underscore_separator_rule_still_applies() {
    let src = "module m\n\nfn f() -> u32 {\n    let x = 100_000\n    return x\n}\n";
    let c = gen("gen-c", src);
    assert!(c.contains("100000"), "C separator rule lost:\n{c}");
    assert!(!c.contains("100_000"), "C emitted a Zig separator:\n{c}");
}

//! #7319: the number lexer took any run of digits, `x X b B _` (and hex
//! letters after `0x`) as one number, and gen-c wrote what it read. `010` and
//! `0_10` reached C as the OCTAL constant 8, so a test asserting `D == 10`
//! compiled and trapped; `08` and `0b102` did not compile; `1_`, `1__0` and
//! `0x_1F` were quietly normalized. Every one now fails with an error naming
//! the literal and its line, as the Zig reference does.
//!
//! #2645: `0o17` lexed as `0` followed by the identifier `o17`, so the value
//! was 0. It is now read as octal and handed on as the decimal it denotes.
//!
//! Part of #5980: the self-hosted core refuses every one of these with E_LEX.

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let d = std::env::temp_dir().join(format!(
            "t27c-genc-numlit-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("create scratch directory");
        Self(d)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn gen_c_raw(s: &Scratch, src: &str) -> Output {
    let p = s.0.join("in.t27");
    std::fs::write(&p, src).expect("write spec");
    Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&p)
        .output()
        .expect("run t27c gen-c")
}

fn spec(lit: &str) -> String {
    format!("module m;\nconst D: i64 = {};\n", lit)
}

/// gen-c over `const D: i64 = <lit>;` must succeed and write `#define D <c>`.
fn accepts(lit: &str, c: &str) {
    let s = Scratch::new();
    let out = gen_c_raw(&s, &spec(lit));
    assert!(
        out.status.success(),
        "gen-c refused the well-formed literal `{}`:\n{}",
        lit,
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    let want = format!("#define D {}", c);
    assert!(
        text.lines().any(|l| l == want),
        "`{}` must be written `{}`:\n{}",
        lit,
        want,
        text
    );
}

/// gen-c over `const D: i64 = <lit>;` must fail, naming the literal and line 2.
fn refuses(lit: &str) {
    let s = Scratch::new();
    let out = gen_c_raw(&s, &spec(lit));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "gen-c accepted the malformed literal `{}`:\n{}",
        lit,
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        stderr.contains(&format!("`{}`", lit)) && stderr.contains("line 2"),
        "the error must name `{}` and line 2:\n{}",
        lit,
        stderr
    );
}

/// The issue's seven literals, from its Reproduce loop.
#[test]
fn the_issue_literals_are_refused_by_name_and_line() {
    for lit in ["010", "0_10", "08", "0b102", "1_", "1__0", "0x_1F"] {
        refuses(lit);
    }
}

/// The issue's well-formed literals still pass, written as C reads them.
#[test]
fn the_issue_well_formed_literals_still_pass() {
    for (lit, c) in [
        ("9", "9"),
        ("10", "10"),
        ("1_000", "1000"),
        ("0x1_F", "0x1F"),
        ("0b1_0", "0b10"),
        ("0", "0"),
        ("0x00", "0x00"),
    ] {
        accepts(lit, c);
    }
}

#[test]
fn decimals_fractions_and_exponents_still_pass() {
    for (lit, c) in [("0.5", "0.5"), ("1.5e-3", "1.5e-3"), ("1e6", "1e6"), ("2.0", "2.0")] {
        let s = Scratch::new();
        let out = gen_c_raw(&s, &format!("module m;\nconst D: f64 = {};\n", lit));
        assert!(
            out.status.success(),
            "gen-c refused `{}`:\n{}",
            lit,
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            String::from_utf8_lossy(&out.stdout).contains(c),
            "`{}` must reach C",
            lit
        );
    }
}

#[test]
fn more_misplaced_separators_and_foreign_digits_are_refused() {
    for lit in ["0x1__F", "0x1F_", "0b_1", "0b12", "1_.5", "00", "007"] {
        refuses(lit);
    }
}

/// A Rust-style typed literal keeps its `_` before the suffix, and a number
/// that is not a value (a hyphenated module name segment) is not judged.
#[test]
fn a_typed_literal_and_a_module_name_segment_are_not_refused() {
    for src in [
        "module m;\nconst D: i64 = 10_i64;\n",
        "module m;\nconst D: i64 = 0x7F_i64;\n",
        "module m;\nconst D: f64 = 2.0_f64;\n",
        "module a::RUST-04::m;\nconst D: i64 = 4;\n",
    ] {
        let s = Scratch::new();
        let out = gen_c_raw(&s, src);
        assert!(
            out.status.success(),
            "gen-c refused:\n{}\n{}",
            src,
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// #2645: an octal literal is read as octal, not as `0` and a stray name.
#[test]
fn an_octal_literal_has_its_octal_value() {
    accepts("0o17", "15");
    accepts("0o1_7", "15");
    accepts("0o0", "0");
    accepts("0o777", "511");
}

#[test]
fn a_malformed_octal_literal_is_refused() {
    for lit in ["0o18", "0o_7", "0o7_", "0o"] {
        refuses(lit);
    }
}

fn cc_present() -> bool {
    Command::new("cc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// The issue's trap, end to end: the test asserts the value the spec states,
/// and the generated C must agree when it runs. Skipped where there is no cc.
#[test]
fn the_generated_c_holds_the_stated_values() {
    let s = Scratch::new();
    let out = gen_c_raw(
        &s,
        "module m;\nconst T: i64 = 10;\nconst E: i64 = 0o777;\nconst S: i64 = 1_000;\n\
         test t { assert(T == 10); assert(E == 511); assert(S == 1000); }\n",
    );
    assert!(
        out.status.success(),
        "gen-c failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    if !cc_present() {
        eprintln!("cc not found; skipping the run");
        return;
    }
    let c = String::from_utf8_lossy(&out.stdout).to_string();
    let cp = s.0.join("out.c");
    std::fs::write(&cp, &c).expect("write C");
    let bin = s.0.join("out");
    let built = Command::new("cc")
        .args(["-std=gnu11", "-DT27_TEST_MAIN"])
        .arg(&cp)
        .arg("-o")
        .arg(&bin)
        .output()
        .expect("run cc");
    assert!(
        built.status.success(),
        "cc refused the generated C:\n{}\n--- C ---\n{}",
        String::from_utf8_lossy(&built.stderr),
        c
    );
    let ran = Command::new(&bin).output().expect("run the generated C");
    assert!(
        ran.status.success(),
        "the generated C failed its own test:\n{}{}\n--- C ---\n{}",
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr),
        c
    );
}

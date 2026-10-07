//! #7353: gen-c wrote a repeat initializer as a C comment of its source and
//! then the initializer. For a nested repeat the comment held another comment,
//! and the inner `*/` closed the outer one, so the C did not compile. A
//! repeated non-zero row became `{0}` and compiled, with every element 0. A
//! non-zero inner repeat put its GNU range at the outer level, out of bounds.
//!
//! #7427: every `**` took that path, so a numeric power `a ** b` became
//! `/* repeat: a ** b */ {0}`. A `**` whose left operand is not an array
//! literal is now `t27_pow(a, b)`.
//!
//! The tests compile the generated C with `-DT27_TEST_MAIN` and RUN it: a
//! zeroed row compiled cleanly, so only running the C shows the values.
//!
//! Part of #5980.

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let d = std::env::temp_dir().join(format!(
            "t27c-genc-repeat-{}-{}",
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

fn gen_c(s: &Scratch, src: &str) -> String {
    let out = gen_c_raw(s, src);
    assert!(
        out.status.success(),
        "gen-c failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn cc_present() -> bool {
    Command::new("cc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// No `/*` may open inside a comment that is already open: C does not nest
/// them, and the first `*/` ends the outer one.
fn assert_no_nested_comment(c: &str) {
    let b = c.as_bytes();
    let mut i = 0;
    let mut open = false;
    while i + 1 < b.len() {
        let two = (b[i], b[i + 1]);
        if !open && two == (b'/', b'*') {
            open = true;
            i += 2;
            continue;
        }
        if open && two == (b'/', b'*') {
            panic!("a C comment opens inside another one:\n{}", c);
        }
        if open && two == (b'*', b'/') {
            open = false;
            i += 2;
            continue;
        }
        i += 1;
    }
}

/// Generate C for `src`, then compile it with `-std=<std>` and the built-in
/// test runner and run it. Returns the C. The run is skipped where there is no
/// cc; the text checks are not.
fn gen_and_run_std(src: &str, std: &str) -> String {
    let s = Scratch::new();
    let c = gen_c(&s, src);
    assert_no_nested_comment(&c);
    if !cc_present() {
        eprintln!("cc not found; skipping the run");
        return c;
    }
    let cp = s.0.join("out.c");
    std::fs::write(&cp, &c).expect("write C");
    let bin = s.0.join("out");
    let built = Command::new("cc")
        .arg(format!("-std={}", std))
        .arg("-DT27_TEST_MAIN")
        .arg(&cp)
        .arg("-o")
        .arg(&bin)
        .arg("-lm")
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
        "the generated C failed its own tests ({}):\n{}{}\n--- C ---\n{}",
        ran.status,
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr),
        c
    );
    c
}

fn gen_and_run(src: &str) -> String {
    gen_and_run_std(src, "gnu11")
}

/// The issue's Reproduce, verbatim, with the issue's `cc -std=c99`.
#[test]
fn the_issue_repro_keeps_the_repeated_row() {
    gen_and_run_std(
        "module m;\nvar g: [2][2]u8 = [_][2]u8{ [_]u8{1, 2} } ** 2;\ntest t { assert(g[1][1] == 2); }\n",
        "c99",
    );
}

#[test]
fn a_zero_nested_repeat_compiles_and_reads_zero() {
    let c = gen_and_run_std(
        "module m;\nvar g: [2][3]u8 = [_][3]u8{ [_]u8{0} ** 3 } ** 2;\n\
         test t { assert(g[0][0] == 0); assert(g[1][2] == 0); }\n",
        "c99",
    );
    assert!(
        c.contains("/* repeat: { { 0 } ** 3 } ** 2 */ {0}"),
        "the nested source is one comment, then `{{0}}`:\n{}",
        c
    );
}

#[test]
fn a_non_zero_inner_repeat_stays_inside_its_row() {
    gen_and_run_std(
        "module m;\nconst K: usize = 3;\nconst R: usize = 2;\n\
         var g: [R][K]i64 = [_][K]i64{ [_]i64{7} ** K } ** R;\n\
         test t { assert(g[0][0] == 7); assert(g[1][2] == 7); assert(g[1][0] == 7); }\n",
        "c99",
    );
}

/// The corpus form (specs/depin/prove.t27), as a global, plus a one-level
/// local repeat. (A local whose type is a nested array is declared wrongly by
/// gen-c for any initializer; that is a separate defect, not a repeat one.)
#[test]
fn a_16_by_16_zero_matrix_compiles() {
    gen_and_run(
        "module m;\nvar w: [16][16]u8 = [_][16]u8{ [_]u8{0} ** 16 } ** 16;\n\
         fn f() -> u8 {\n    var r: [16]u8 = [_]u8{4} ** 16;\n    \
         w[15][15] = 3;\n    return w[15][15] + w[0][0] + r[15];\n}\n\
         test t { assert(f() == 7); }\n",
    );
}

#[test]
fn a_repeated_row_of_several_rows_keeps_every_value() {
    gen_and_run(
        "module m;\nvar g: [3][2][2]u8 = [_][2][2]u8{ [_][2]u8{ [_]u8{1, 2}, [_]u8{3, 4} } } ** 3;\n\
         test t { assert(g[2][1][0] == 3); assert(g[0][0][1] == 2); assert(g[1][1][1] == 4); }\n",
    );
}

/// The shape in specs/ternary/hybrid_arithmetic.t27: several elements, a
/// literal count. It was `{0}`.
#[test]
fn a_repeat_of_several_elements_is_unrolled() {
    let c = gen_and_run(
        "module m;\nvar p: [8]i8 = [_]i8{1, 0, -1, 1} ** 2;\n\
         test t { assert(p[0] == 1); assert(p[2] == -1); assert(p[6] == -1); assert(p[7] == 1); }\n",
    );
    assert!(c.contains("{ 1, 0, -1, 1, 1, 0, -1, 1 }"), "unrolled:\n{}", c);
}

#[test]
fn several_elements_with_a_named_count_are_refused_not_zeroed() {
    let s = Scratch::new();
    let out = gen_c_raw(
        &s,
        "module m;\nconst N: usize = 2;\nvar p: [4]i8 = [_]i8{1, 2} ** N;\n",
    );
    assert!(
        !out.status.success(),
        "gen-c wrote a repeat it cannot lower:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("`**` repeat")
            && stderr.contains("initializing `p`")
            && stderr.contains("#7353"),
        "the refusal must name the repeat and the declaration it initializes:\n{}",
        stderr
    );
}

/// The element shapes the corpus repeats (specs/base/types.t27,
/// HirIpCatalog_new, gen_gradient, gen_image, array_repeat_text): an enum
/// member under a named count, a struct literal, a call, a bracket struct
/// literal, and two elements one of which is a call. Each was `{0}` before
/// #7353; none may become a refusal, and each must read its value back.
#[test]
fn enum_struct_and_call_elements_are_repeated() {
    let c = gen_and_run(
        "module m;\nconst Trit = enum(i8) {\n    neg = -1,\n    zero = 0,\n    pos = 1,\n};\n\
         const Color = struct {\n    r: u8,\n    a: u8,\n};\n\
         const Box = struct {\n    colors: [4]Color,\n    n: u8,\n};\n\
         const W: usize = 6;\n\
         fn two() -> i32 {\n    return 2;\n}\n\
         fn mk() -> Color {\n    return Color{ .r = 3, .a = 9 };\n}\n\
         fn box() -> Box {\n    return Box{ .colors = [_]Color{ Color{ .r = 0, .a = 255 } } ** 4, .n = 1 };\n}\n\
         fn f() -> i32 {\n    \
         const t: [6]Trit = [_]Trit{.pos} ** W;\n    \
         var c: [3]Color = [_]Color{mk()} ** 3;\n    \
         var d: [5]Color = [Color{ .r = 7, .a = 1 }] ** 5;\n    \
         var q: [4]i32 = [W, two()] ** 2;\n    \
         const b = box();\n    \
         var s: i32 = 0;\n    \
         if (t[5] == Trit.pos) { s = s + 1; }\n    \
         if (c[2].r == 3 and c[0].a == 9) { s = s + 10; }\n    \
         if (d[4].r == 7 and d[0].a == 1) { s = s + 100; }\n    \
         if (q[2] == 6 and q[3] == 2) { s = s + 1000; }\n    \
         if (b.colors[3].a == 255) { s = s + 10000; }\n    \
         return s;\n}\n\
         test t { assert(f() == 11111); }\n",
    );
    assert!(!c.contains("*/ {0};"), "a non-zero repeat fell to zeros:\n{}", c);
}

/// The single-level forms the self-hosted core accepts keep their bytes.
#[test]
fn a_single_level_repeat_is_written_as_before() {
    let s = Scratch::new();
    let c = gen_c(
        &s,
        "module m;\nvar a: [4]u8 = [_]u8{0} ** 4;\nvar b: [4]u8 = [_]u8{5} ** 4;\n\
         var z: [100][2]u8 = [_][2]u8{[0, 0]} ** 100;\n",
    );
    assert!(
        c.contains("static uint8_t a[4] = /* repeat: { 0 } ** 4 */ {0};"),
        "zero repeat:\n{}",
        c
    );
    assert!(
        c.contains("static uint8_t b[4] = /* repeat: { 5 } ** 4 */ { [0 ... (4) - 1] = 5 };"),
        "non-zero repeat:\n{}",
        c
    );
    assert!(c.contains("** 100 */ {0};"), "a zero bracket row:\n{}", c);
}

/// #7427: the issue's check, plus a float power and a negative base.
#[test]
fn a_numeric_power_is_a_power() {
    let c = gen_and_run(
        "module m;\nfn p(a: u32, b: u32) -> u32 { return a ** b; }\n\
         fn q(x: f64) -> f64 { return x ** 2.0; }\n\
         fn r(x: i64, e: i64) -> i64 { return x ** e; }\n\
         test t {\n    assert(p(2, 10) == 1024);\n    assert(p(3, 0) == 1);\n    \
         assert(q(3.0) == 9.0);\n    assert(r(-2, 3) == -8);\n    assert(r(-1, -3) == -1);\n    \
         assert(r(5, -1) == 0);\n}\n",
    );
    assert!(!c.contains("repeat"), "a numeric power is not a repeat:\n{}", c);
    assert!(c.contains("t27_pow(a, b)"), "the power call:\n{}", c);
}

/// The corpus uses (specs/provider/adapters.t27, specs/portable/relay_observer.t27).
#[test]
fn a_power_inside_a_larger_expression() {
    gen_and_run(
        "module m;\nfn d(base_delay: u32, attempt: u8) -> u32 {\n    const base = @as(u32, 2);\n    \
         return base_delay * (base ** @as(u32, attempt));\n}\n\
         test t { assert(d(100, 3) == 800); }\n",
    );
}

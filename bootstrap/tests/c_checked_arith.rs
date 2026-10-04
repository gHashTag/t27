//! `gen-c` arithmetic has the meaning t27 gives it, and no C undefined
//! behaviour (#5974).
//!
//! The language decided this in #1659 (docs/reports/WAVE_LOOP_573_REPORT.md
//! section 1): plain `+ - *` trap on overflow and `+% -% *%` wrap. The Zig
//! backend and t27b trap mode implement that. `gen-c` wrote the bare C
//! operators, and in C a signed overflow, a shift amount outside [0, width),
//! a zero divisor and `MIN / -1` are undefined. The t27b differential test
//! (#5905) saw `inc(INT_MAX)` pass at -O0 and trap at -O2, and `shr(1024, 40)`
//! do the opposite.
//!
//! Inside a body, every arithmetic operator now lowers to a `t27_*` helper from
//! one guarded prelude block. The shape checks always run. The behavioural
//! checks build each test with `cc` at -O0 and -O2, and with
//! `-fsanitize=undefined` where the toolchain links it. They SKIP when no C
//! compiler is on PATH.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "t27c-c-arith-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("create temp dir");
    d
}

fn gen_c(spec: &str, dir: &std::path::Path) -> String {
    let p = dir.join("in.t27");
    std::fs::write(&p, spec).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&p)
        .output()
        .expect("run t27c gen-c");
    assert!(
        out.status.success(),
        "t27c gen-c failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn cc() -> Option<&'static str> {
    ["cc", "clang", "gcc"].into_iter().find(|c| {
        Command::new(c)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

/// The reproducer from the t27b README, plus the cases it implies.
const SPEC: &str = "module Sem;

fn inc(x: i32) -> i32 {
    return x + 1;
}

fn shr(x: i32, n: i32) -> i32 {
    return x >> n;
}

fn shl(x: i32, n: i32) -> i32 {
    return x << n;
}

fn rem(a: i32, b: i32) -> i32 {
    return a % b;
}

fn div(a: i32, b: i32) -> i32 {
    return a / b;
}

fn winc(x: i32) -> i32 {
    return x +% 1;
}

fn usub(a: u32, b: u32) -> u32 {
    return a - b;
}

fn acc(x: i32, y: i32) -> i32 {
    var s: i32 = x;
    s += y;
    return s;
}

fn mask(n: u32) -> u64 {
    var m: u64 = (1 << n) - 1;
    return m;
}

const K: i32 = 3 + 4;

test inc_max { assert_eq(inc(2147483647), 0 - 2147483647 - 1); }
test shr_big { assert_eq(shr(1024, 40), 0); }
test shr_ok { assert_eq(shr(1024, 3), 128); }
test shl_neg { assert_eq(shl(1, 0 - 1), 0); }
test shl_drops_bits { assert_eq(shl(1073741824, 1), 0 - 2147483647 - 1); }
test rem_neg { assert_eq(rem(0 - 7, 2), 0 - 1); }
test div_zero { assert_eq(div(1, 0), 0); }
test div_min { assert_eq(div(0 - 2147483647 - 1, 0 - 1), 0); }
test div_trunc { assert_eq(div(0 - 7, 2), 0 - 3); }
test winc_max { assert_eq(winc(2147483647), 0 - 2147483647 - 1); }
test usub_under { assert_eq(usub(0, 1), 0); }
test acc_over { assert_eq(acc(2147483647, 1), 0); }
test acc_ok { assert_eq(acc(2, 3), 5); }
test k_folds { assert_eq(K, 7); }
test mask_wide { assert_eq(mask(52), 4503599627370495); }

endmodule
";

/// (test, traps?) -- what the Zig backend and t27b trap mode answer.
const EXPECT: &[(&str, bool)] = &[
    ("inc_max", true),
    ("shr_big", true),
    ("shr_ok", false),
    ("shl_neg", true),
    ("shl_drops_bits", false),
    ("rem_neg", false),
    ("div_zero", true),
    ("div_min", true),
    ("div_trunc", false),
    ("winc_max", false),
    ("usub_under", true),
    ("acc_over", true),
    ("acc_ok", false),
    ("k_folds", false),
    ("mask_wide", false),
];

#[test]
fn body_arithmetic_lowers_to_checked_helpers() {
    let d = tmp_dir("shape");
    let c = gen_c(SPEC, &d);
    for want in [
        "return t27_add(x, 1);",
        "return t27_shr(x, n);",
        "return t27_shl(x, n);",
        "return t27_rem(a, b);",
        "return t27_div(a, b);",
        "return t27_wadd(x, 1);",
        "return t27_sub(a, b);",
        "s = t27_add(s, y);",
        // A bare literal shifted by a runtime amount takes the declared width,
        // as `@as(u64, 1)` does in the Zig backend; as a C `int` it would trap
        // at n >= 32.
        "uint64_t m = t27_sub(t27_shl((uint64_t)1, n), 1);",
    ] {
        assert!(c.contains(want), "missing `{want}`:\n{c}");
    }
    assert_eq!(
        c.matches("#ifndef T27_ARITH_PRELUDE").count(),
        1,
        "exactly one guarded prelude:\n{c}"
    );
}

#[test]
fn literal_arithmetic_and_module_scope_stay_infix() {
    // `0 - 7` is folded by the C compiler, which diagnoses an overflow there,
    // and a module-scope initializer must be a C constant expression -- a
    // helper call is not one.
    let d = tmp_dir("infix");
    let c = gen_c(SPEC, &d);
    assert!(c.contains("rem((0 - 7), 2)"), "literal-only stays infix:\n{c}");
    let k = c
        .lines()
        .find(|l| l.contains(" K ") || l.contains(" K="))
        .unwrap_or("");
    assert!(k.contains("(3 + 4)"), "module-scope const stays infix: `{k}`\n{c}");
    assert!(!k.contains("t27_"), "no helper at module scope: `{k}`");
}

#[test]
fn no_prelude_without_arithmetic() {
    let d = tmp_dir("none");
    let c = gen_c(
        "module Plain;\n\nfn same(a: i32, b: i32) -> bool {\n    return a == b;\n}\n\ntest t { assert(same(1, 1)); }\n\nendmodule\n",
        &d,
    );
    assert!(!c.contains("T27_ARITH_PRELUDE"), "no arithmetic, no prelude:\n{c}");
}

/// Build one driver that runs a named test, then run every test in its own
/// process and return (test, trapped?).
fn run_all(cc: &str, hdr: &std::path::Path, flags: &[&str], d: &std::path::Path, tag: &str) -> Option<Vec<(String, bool, String)>> {
    let mut drv = format!(
        "#include \"{}\"\n#include <string.h>\nint main(int c, char **v) {{\n    (void)c;\n",
        hdr.display()
    );
    for (t, _) in EXPECT {
        drv.push_str(&format!(
            "    if (!strcmp(v[1], \"{t}\")) {{ test_{t}(); return 0; }}\n"
        ));
    }
    drv.push_str("    return 2;\n}\n");
    let src = d.join(format!("drv_{tag}.c"));
    std::fs::write(&src, drv).expect("write driver");
    let exe = d.join(format!("drv_{tag}"));
    let b = Command::new(cc)
        .args(["-std=gnu11", "-w"])
        .args(flags)
        .arg(&src)
        .arg("-o")
        .arg(&exe)
        .output()
        .expect("run cc");
    if !b.status.success() {
        return None;
    }
    Some(
        EXPECT
            .iter()
            .map(|(t, _)| {
                let o = Command::new(&exe).arg(t).output().expect("run driver");
                (
                    t.to_string(),
                    !o.status.success(),
                    String::from_utf8_lossy(&o.stderr).to_string(),
                )
            })
            .collect(),
    )
}

#[test]
fn same_answers_at_o0_and_o2_matching_zig_and_t27b() {
    let Some(cc) = cc() else {
        eprintln!("SKIP same_answers_at_o0_and_o2_matching_zig_and_t27b: no C compiler on PATH");
        return;
    };
    let d = tmp_dir("run");
    let hdr = d.join("sem.h");
    std::fs::write(&hdr, gen_c(SPEC, &d)).expect("write header");
    for opt in ["-O0", "-O2"] {
        let got = run_all(cc, &hdr, &[opt], &d, &opt[1..])
            .unwrap_or_else(|| panic!("{cc} {opt} failed to build the generated C"));
        for ((t, trapped, _), (_, want)) in got.iter().zip(EXPECT) {
            assert_eq!(
                trapped, want,
                "{cc} {opt}: test {t} trapped={trapped}, Zig and t27b say trap={want}"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn ubsan_reports_nothing() {
    let Some(cc) = cc() else {
        eprintln!("SKIP ubsan_reports_nothing: no C compiler on PATH");
        return;
    };
    let d = tmp_dir("ubsan");
    let hdr = d.join("sem.h");
    std::fs::write(&hdr, gen_c(SPEC, &d)).expect("write header");
    for opt in ["-O0", "-O2"] {
        let flags = [opt, "-fsanitize=undefined", "-fno-sanitize-recover=undefined"];
        let Some(got) = run_all(cc, &hdr, &flags, &d, &format!("u{}", &opt[1..])) else {
            eprintln!("SKIP ubsan_reports_nothing: {cc} cannot link -fsanitize=undefined");
            return;
        };
        for ((t, trapped, err), (_, want)) in got.iter().zip(EXPECT) {
            assert!(
                !err.contains("runtime error"),
                "{cc} {opt} UBSan reported undefined behaviour in test {t}:\n{err}"
            );
            assert_eq!(trapped, want, "{cc} {opt} UBSan build: test {t}");
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}

//! `%` on a signed integer (or a float) in the Zig backend (#5973).
//!
//! Zig refuses a plain `%` unless both operands are unsigned integers:
//! "remainder division with 'i32' and 'i32': signed integers and floats must
//! use @rem or @mod". `/` was given its repair in W593 (`@divTrunc` when an
//! operand is known signed); `%` never was, so `return a % b;` with `a: i32`
//! made the whole generated file fail to compile.
//!
//! Found by the t27b differential test (#5905). `@rem` is the truncated
//! remainder, so `rem(-7, 2) == -1` -- the answer C, Rust and t27b give.
//! `@mod` would floor and answer 1.
//!
//! The shape checks always run. The behavioural check runs only when `zig` is
//! on PATH, and says SKIP when it is not.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "t27c-zig-rem-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("create temp dir");
    d
}

fn gen_zig(spec: &str, dir: &std::path::Path) -> String {
    let p = dir.join("in.t27");
    std::fs::write(&p, spec).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen")
        .arg(&p)
        .output()
        .expect("run t27c gen");
    assert!(
        out.status.success(),
        "t27c gen failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn zig_present() -> bool {
    Command::new("zig")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

const SPEC_SIGNED: &str = "module Sem;

fn rem(a: i32, b: i32) -> i32 {
    return a % b;
}

fn urem(a: u32, b: u32) -> u32 {
    return a % b;
}

fn frem(a: f64, b: f64) -> f64 {
    return a % b;
}

test rem_neg { assert_eq(rem(0 - 7, 2), 0 - 1); }
test rem_pos { assert_eq(rem(7, 0 - 2), 1); }
test urem_plain { assert_eq(urem(7, 2), 1); }

endmodule
";

#[test]
fn signed_rem_lowers_to_at_rem() {
    let d = tmp_dir("shape");
    let z = gen_zig(SPEC_SIGNED, &d);
    assert!(z.contains("return @rem(a, b);"), "signed % must be @rem:\n{z}");
    assert!(!z.contains("@mod("), "truncated, not floored -- never @mod:\n{z}");
}

#[test]
fn unsigned_rem_stays_infix() {
    // `%` IS valid Zig for two unsigned operands; rewriting it would only
    // churn every seal whose spec takes a u32 remainder.
    let d = tmp_dir("unsigned");
    let z = gen_zig(SPEC_SIGNED, &d);
    let urem = z
        .split("fn urem(")
        .nth(1)
        .and_then(|s| s.split('}').next())
        .expect("urem is emitted");
    assert!(urem.contains("a % b"), "unsigned % stays %:\n{urem}");
    assert!(!urem.contains("@rem"), "unsigned % stays %:\n{urem}");
}

#[test]
fn float_rem_lowers_to_at_rem() {
    // The same Zig error names floats too, and @rem accepts them.
    let d = tmp_dir("float");
    let z = gen_zig(SPEC_SIGNED, &d);
    let frem = z
        .split("fn frem(")
        .nth(1)
        .and_then(|s| s.split('}').next())
        .expect("frem is emitted");
    assert!(frem.contains("@rem(a, b)"), "float % must be @rem:\n{frem}");
}

#[test]
fn signed_rem_compiles_and_truncates_under_zig_test() {
    if !zig_present() {
        eprintln!("SKIP signed_rem_compiles_and_truncates_under_zig_test: no zig on PATH");
        return;
    }
    let d = tmp_dir("run");
    let z = gen_zig(SPEC_SIGNED, &d);
    let zp = d.join("out.zig");
    std::fs::write(&zp, &z).expect("write zig");
    let out = Command::new("zig")
        .arg("test")
        .arg(&zp)
        .current_dir(&d)
        .env("ZIG_GLOBAL_CACHE_DIR", d.join("zig-global-cache"))
        .env("ZIG_LOCAL_CACHE_DIR", d.join("zig-cache"))
        .output()
        .expect("run zig test");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "zig test must compile and pass rem(-7, 2) == -1, rem(7, -2) == 1:\n{stderr}\n{z}"
    );
    let _ = std::fs::remove_dir_all(&d);
}

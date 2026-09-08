//! `sqrt`, `floor` and `round` ARE the C names, and C was passing them through.
//!
//! Found by `tools/builtin_parity_table.py` on its first run -- the reader
//! built this pass because the same shape had produced five defects, a rule
//! present in three backends and missing from the fourth. The table put six
//! rows side by side:
//!
//!     abs    392 uses, 45 specs   Rust `(v).abs()`   Zig `@abs(v)`   C: passthrough
//!     round  136 / 38             `(v).round()`      `@round(v)`     C: passthrough
//!     sqrt   100 / 25             `(v).sqrt()`       `@sqrt(v)`      C: passthrough
//!     floor   98 / 15             `(v).floor()`      `@floor(v)`     C: passthrough
//!     max     51 / 26             `(v).max(v)`       `@max(v, v)`    C: passthrough
//!     min     19 / 12             `(v).min(v)`       `@min(v, v)`    C: passthrough
//!
//! For three of them the spelling was never wrong: `sqrt(x)` IS C, and the
//! only thing missing was `<math.h>`. The whole repair is the include.
//!
//! `abs` is deliberately NOT included: C has `abs` for int and `fabs` for
//! double, and picking one without the argument's type is a silent truncation.
//! `min` and `max` are not C functions at all. Those two rows stay loud.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn cc_present() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn gen_c(spec: &str, tag: &str) -> (String, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!(
        "t27c-libm-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    let p = d.join("in.t27");
    std::fs::write(&p, spec).expect("write");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c")).arg("gen-c").arg(&p).output().expect("t27c");
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

#[test]
fn a_module_that_calls_sqrt_gets_math_h() {
    let (h, d) = gen_c(
        "module M {\n    fn f(v: f64) -> f64 { var a = sqrt(v); return a; }\n}\n",
        "sqrt",
    );
    assert!(h.contains("#include <math.h>"), "used, so included:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn all_three_names_are_recognised() {
    let (h, d) = gen_c(
        "module M {\n    fn f(v: f64) -> f64 { var a = floor(v); var b = round(v); return a; }\n}\n",
        "three",
    );
    assert!(h.contains("#include <math.h>"), "floor and round count too:\n{h}");
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must compile:\n{h}");
    }
}

#[test]
fn a_module_that_calls_none_does_not_get_it() {
    // The other half. An include nothing needs is noise in 572 headers.
    let (h, _d) = gen_c("module M {\n    fn f(v: i32) -> i32 { return v; }\n}\n", "none");
    assert!(!h.contains("#include <math.h>"), "unused, so not included:\n{h}");
}

#[test]
fn a_spec_that_declares_its_own_sqrt_does_not_get_it() {
    // THE DISCRIMINATING CASE. A module defining `fn sqrt` means that
    // function, and pulling in `<math.h>` beside it would redeclare the name
    // with a different signature -- turning a working header into a broken one.
    let (h, d) = gen_c(
        "module M {\n    fn sqrt(x: i32) -> i32 { return x; }\n\
         \x20   fn f(v: i32) -> i32 { var a = sqrt(v); return a; }\n}\n",
        "declared",
    );
    assert!(
        !h.contains("#include <math.h>"),
        "a declared `sqrt` is the spec's own:\n{h}"
    );
    if cc_present() {
        assert_eq!(errors(&h, &d), 0, "and it must still compile:\n{h}");
    }
}

#[test]
fn abs_min_and_max_stay_loud() {
    // Deliberate, and measured: `abs` is 392 uses and C has two of them
    // (`abs` for int, `fabs` for double) -- choosing without the argument's
    // type is a silent truncation. `min` and `max` are not C functions.
    let (h, _d) = gen_c(
        "module M {\n    fn f(v: f64) -> f64 { var a = abs(v); var b = max(v, v); return a; }\n}\n",
        "loud",
    );
    assert!(!h.contains("#include <math.h>"), "abs and max do not pull it in:\n{h}");
    assert!(h.contains("abs(v)"), "and they are left as written:\n{h}");
}

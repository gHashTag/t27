//! A generic struct reaches four backends and only one of them lowers it.
//!
//! `pub const Map(K, V) = struct { ... }` parses, `t27c check` said
//! `Typecheck OK (0 errors, 0 warnings)`, and the generated C does not compile.
//! The parser records the type parameters, and `collect_type_params` hands them
//! to the unknown-type check so `K` and `V` are not reported as undeclared --
//! correct in itself, and it silences the only reader that might have noticed.
//!
//! MEASURED, because the obvious message would have been false:
//!
//!   gen-rust     `pub struct Box<T>`, `pub fn get<T>(b: Box<T>)` -- COMPILES
//!   gen-c        `struct Box { ... }` without the parameter, then
//!                `get(Box(T) b)`: "unknown type name 'T'"
//!   gen-zig      `pub const Box = struct`, then `get(b: Box(T))`:
//!                "use of undeclared identifier 'T'"
//!
//! So it is not "no backend lowers generics". One does. Two drop the parameter
//! from the declaration and keep it at the use, and that inconsistency is what
//! the warning states -- with a test that holds Rust to its half.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "t27c-generic-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    d
}

fn run(sub: &str, spec: &str, d: &std::path::Path) -> (i32, String) {
    let p = d.join("in.t27");
    std::fs::write(&p, spec).expect("write");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg(sub)
        .arg(&p)
        .output()
        .expect("run t27c");
    let mut t = String::from_utf8_lossy(&out.stdout).to_string();
    t.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), t)
}

const GENERIC: &str = r#"
module P {
    pub const Box(T) = struct {
        v : T,
    }
    fn get(b: Box(T)) -> i32 { return 0; }
}
"#;

#[test]
fn a_generic_struct_is_reported_with_its_parameters() {
    let d = dir("named");
    let (_, out) = run("check", GENERIC, &d);
    assert!(
        out.contains("`Box(T)` is generic"),
        "the finding must name the type and its parameters:\n{out}"
    );
}

#[test]
fn the_message_does_not_claim_that_no_backend_lowers_it() {
    // The obvious wording -- "no backend lowers generics" -- is false, and the
    // next test proves it by compiling Rust's output. A guard's claim has to
    // hold on the narrowest case in its population.
    let d = dir("wording");
    let (_, out) = run("check", GENERIC, &d);
    assert!(
        out.contains("gen-rust lowers it"),
        "the message must credit the backend that DOES lower it:\n{out}"
    );
    assert!(
        !out.contains("every backend") && !out.contains("no backend"),
        "and must not overstate:\n{out}"
    );
}

#[test]
fn gen_rust_really_does_lower_a_generic() {
    // The half of the message that is a claim about another program. Without
    // this the wording above is just a nicer sentence.
    if Command::new("rustc").arg("--version").output().map(|o| !o.status.success()).unwrap_or(true) {
        eprintln!("SKIP gen_rust_really_does_lower_a_generic: no rustc on PATH");
        return;
    }
    let d = dir("rustlowers");
    let (_, src) = run("gen-rust", GENERIC, &d);
    assert!(src.contains("pub struct Box<T>"), "expected a generic struct:\n{src}");
    let rs = d.join("g.rs");
    std::fs::write(&rs, &src).expect("write");
    let out = Command::new("rustc")
        .args(["--edition", "2021", "--crate-type", "lib", "--crate-name", "m", "-A", "warnings"])
        .arg("--emit=metadata")
        .arg("-o")
        .arg(d.join("m.rmeta"))
        .arg(&rs)
        .output()
        .expect("rustc");
    let e = String::from_utf8_lossy(&out.stderr);
    assert!(!e.contains("error"), "gen-rust's generic output must compile:\n{e}");
}

#[test]
fn a_plain_struct_is_not_reported() {
    // The negative control: a detector that fired on every struct would pass
    // the first test.
    let d = dir("plain");
    let (rc, out) = run(
        "check",
        "module P {\n    struct Plain { a : i32, }\n    fn f(p: Plain) -> i32 { return p.a; }\n}\n",
        &d,
    );
    assert_eq!(rc, 0, "a plain spec still checks clean:\n{out}");
    assert!(!out.contains("is generic"), "and reports no generic:\n{out}");
}

#[test]
fn the_finding_is_a_warning() {
    // Measured before landing: 651 specs, 573 exit 0 and 78 exit 1, and not one
    // spec's code moved. Promoting this to an error is a separate decision.
    let d = dir("exit");
    let (rc, out) = run("check", GENERIC, &d);
    assert_eq!(rc, 0, "the generic finding is a warning:\n{out}");
}

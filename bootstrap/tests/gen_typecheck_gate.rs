//! #6446: every gen path runs `typecheck` first.
//!
//! `tri misread` found 35 spec-shape pairs that `typecheck` refused while
//! `gen-rust` printed them anyway -- a field with no type became `pub f: ,`,
//! which no Rust compiler accepts, and every gate that only asked "did gen
//! exit 0?" stayed green. A refused spec now generates nothing: the command
//! exits non-zero, stdout is empty, and stderr carries the refusal.

use std::io::Write;
use std::process::{Command, Output};

/// The `pub f: ,` defect: `f : ,` is recovered as a field with no type.
const REFUSED: &str = "module probe {\n    pub const Thing = struct {\n        ok : u8,\n        f : ,\n    };\n}\n";

/// The same struct, well formed.
const CLEAN: &str = "module probe {\n    pub const Thing = struct {\n        ok : u8,\n        f : u8,\n    };\n}\n";

fn run(src: &str, args: &[&str]) -> Output {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("t27-gengate-{}-{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("probe.t27");
    std::fs::File::create(&path)
        .and_then(|mut f| f.write_all(src.as_bytes()))
        .expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .args(args)
        .arg(&path)
        .output()
        .expect("run t27c");
    let _ = std::fs::remove_dir_all(&dir);
    out
}

fn assert_refused(out: &Output, what: &str) {
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "{what} exited 0 on a spec typecheck refuses");
    assert!(out.stdout.is_empty(), "{what} printed output for a refused spec:\n{}", String::from_utf8_lossy(&out.stdout));
    assert!(stderr.contains("typecheck refused"), "{what} stderr lacks the refusal:\n{stderr}");
    assert!(stderr.contains("`f` has no type"), "{what} stderr lacks the typecheck message:\n{stderr}");
}

#[test]
fn gen_rust_refuses_a_field_with_no_type() {
    // Before #6446 this printed `pub f: ,` and exited 0.
    assert_refused(&run(REFUSED, &["gen-rust"]), "gen-rust");
}

#[test]
fn every_gen_path_refuses_it() {
    for cmd in ["gen", "gen-c", "gen-verilog", "gen-js", "gen-ts", "gen-python"] {
        assert_refused(&run(REFUSED, &[cmd]), cmd);
    }
}

#[test]
fn typecheck_and_gen_agree() {
    let tc = run(REFUSED, &["typecheck"]);
    assert!(!tc.status.success(), "typecheck accepted the refused spec");
    for cmd in ["typecheck", "gen-rust", "gen-c"] {
        let out = run(CLEAN, &[cmd]);
        assert!(out.status.success(), "{cmd} refused the clean spec:\n{}", String::from_utf8_lossy(&out.stderr));
    }
}

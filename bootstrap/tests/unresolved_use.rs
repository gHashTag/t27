//! #7176: a `use` that `t27c gen` cannot resolve is named on stderr.
//!
//! The splice skipped a target that was not a file without a word, `gen`
//! exited 0, and the first error came from zig about an identifier
//! (`use of undeclared identifier 'LIST_END'`), never about the `use` line.
//! Through the SHIPPED binary, as `unknown_type.rs` does: a unit test alone
//! would pass against a compiler that never called the check.

use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("t27c-7176-cli-{}-{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("dir");
    d
}

/// (exit ok, stdout, stderr) of `t27c gen <p>`.
fn gen(p: &Path) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen")
        .arg(p)
        .output()
        .expect("t27c");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

const IMPORTER: &str = "module c;\nuse a::b;\n\npub fn k() -> u8 {\n    return K;\n}\n";

#[test]
fn a_use_the_splice_cannot_find_is_named_with_its_line() {
    let d = scratch("missing").join("specs/c");
    std::fs::create_dir_all(&d).expect("dir");
    let p = d.join("in.t27");
    std::fs::write(&p, "module c;\nuse a::b;\n\npub const N : u8 = 1;\n").expect("write");
    let (ok, _, err) = gen(&p);
    assert!(ok, "the exit code does not change: {err}");
    let want = format!("{}:2: use a::b resolves to no spec", p.display());
    assert!(err.contains(&want), "want {want:?} in:\n{err}");
    assert!(err.contains("nothing is spliced from it (#7176)"), "{err}");
}

#[test]
fn a_use_the_splice_finds_prints_no_warning() {
    let root = scratch("found").join("specs");
    std::fs::create_dir_all(root.join("a")).expect("dir");
    std::fs::create_dir_all(root.join("c")).expect("dir");
    std::fs::write(root.join("a/b.t27"), "module b;\npub const K : u8 = 3;\n").expect("write");
    let p = root.join("c/in.t27");
    std::fs::write(&p, IMPORTER).expect("write");
    let (ok, out, err) = gen(&p);
    assert!(ok, "{err}");
    assert!(!err.contains("#7176"), "{err}");
    assert!(out.contains("const K"), "the spliced constant is in the output:\n{out}");
}

//! A t27 `*T` parameter must reach C as `T*`.
//!
//! `param_type_to_c` has arms for `?T`, `[]T`, `[T; N]` and `[N]T`, and had none
//! for the prefix star, so the text passed through verbatim:
//!
//!     uint64_t set(*Bitmap bitmap);
//!
//! C parses `Bitmap` there as the declared NAME carrying an implicit-int type,
//! then finds a second identifier -- "type specifier missing, defaults to 'int'"
//! followed by "expected ')'". Fifty-two generated headers carried it.
//!
//! Removing the family moved `cc accepts` by zero, which is the measurement that
//! matters about this backlog rather than a disappointment: of 404 rejected
//! files only 20 are blocked by a single family, so no one family is a lever.
//! What it does move is what comes NEXT -- with pointers typed correctly, the
//! member-access defect (`.` where C needs `->`) becomes visible as its own
//! single-family file.

use std::io::Write;
use std::process::Command;

fn gen_c(src: &str) -> String {
    // Keyed by a COUNTER, not by a property of the input. The old key was
    // `(pid, src.len())`, and every test in this binary shares the pid -- so two
    // tests whose sources happen to be the same length computed the SAME
    // directory, which each of them deletes on the way out. Under the default
    // parallel runner one test erases the spec another is mid-read of, `t27c`
    // prints nothing, and the assertion reports an empty result.
    //
    // Measured with a probe asserting the directory is fresh: it fired 8 runs
    // out of 8. The collision is not occasional -- it happens every run, and
    // only the timing of the delete decides whether a test dies.
    static SCRATCH_N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let scratch_n = SCRATCH_N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("t27-ptrparam-{}-{}", std::process::id(), scratch_n));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("m.t27");
    let mut f = std::fs::File::create(&path).expect("write spec");
    f.write_all(src.as_bytes()).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-c")
        .arg(&path)
        .output()
        .expect("run t27c");
    let _ = std::fs::remove_dir_all(&dir);
    String::from_utf8_lossy(&out.stdout).to_string()
}

const SPEC: &str = "module ptr\n\nstruct Bitmap {\n    bits: u64,\n}\n\nfn set(bitmap: *Bitmap) -> u64 {\n    return 1\n}\n";

#[test]
fn a_prefix_star_parameter_becomes_a_trailing_star_in_c() {
    let c = gen_c(SPEC);
    assert!(
        c.contains("uint64_t set(Bitmap* bitmap)"),
        "expected `Bitmap* bitmap`, got:\n{}",
        c.lines().filter(|l| l.contains("set(")).collect::<Vec<_>>().join("\n")
    );
    assert!(
        !c.contains("(*Bitmap bitmap"),
        "the prefix star must not survive into C"
    );
}

/// A pointer to a slice keeps both, and in the right order.
#[test]
fn the_star_arm_recurses_rather_than_prepending_to_raw_text() {
    let c = gen_c("module p2\n\nfn f(xs: *[]u8) -> u64 {\n    return 1\n}\n");
    let line = c
        .lines()
        .find(|l| l.contains("f(") && l.contains("xs"))
        .unwrap_or_default()
        .to_string();
    assert!(
        line.contains("uint8_t** xs"),
        "a pointer to a slice is `uint8_t**`, got: {line}"
    );
}

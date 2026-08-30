//! A constant must fit the type it declares.
//!
//! `const EXP_OFFSET: u32 = 1792...173` -- 185 digits -- typechecked clean, and
//! every backend emitted the digits verbatim: Rust as `pub const EXP_OFFSET:
//! u32`, Verilog as `localparam [31:0]`. Only `cc` said anything, and it is the
//! fourth backend, so three of the four carried a 590-bit value in a 32-bit box
//! without a word.
//!
//! The declared width is a claim, the literal is a value, and nothing compared
//! them. Ten constants across five specs.

use std::io::Write;
use std::process::Command;

/// Typecheck a source string through the shipped binary and return only the
/// width errors.
///
/// Through the binary, not a library call: this crate has no lib target, and a
/// test that reimplemented the check would pass against a compiler that never
/// shipped it.
fn errors_of(src: &str) -> Vec<String> {
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
    let dir = std::env::temp_dir().join(format!(
        "t27-constwidth-{}-{}",
        std::process::id(),
        scratch_n
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("m.t27");
    let mut f = std::fs::File::create(&path).expect("write spec");
    f.write_all(src.as_bytes()).expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("typecheck")
        .arg(&path)
        .output()
        .expect("run t27c");
    let _ = std::fs::remove_dir_all(&dir);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    text.lines()
        .filter(|l| l.contains("which no"))
        .map(|l| l.to_string())
        .collect()
}

#[test]
fn a_literal_wider_than_its_declared_type_is_an_error() {
    let e = errors_of("module m\n\nconst HUGE: u32 = 99999999999999999999999999999999999999\n");
    assert_eq!(e.len(), 1, "expected one width error, got {e:?}");
    assert!(e[0].contains("38 digits"), "{}", e[0]);
    assert!(e[0].contains("u32"), "{}", e[0]);
}

/// The value that provoked this does not fit any Rust integer either.
///
/// A checker that parsed the literal into `u128` would overflow on exactly the
/// inputs it exists to reject and report nothing -- the failure would look like
/// a clean file. So the digit count is compared first.
#[test]
fn a_literal_too_wide_for_u128_is_still_rejected() {
    let big = "1".repeat(185);
    let e = errors_of(&format!("module m\n\nconst E: u32 = {big}\n"));
    assert_eq!(e.len(), 1, "185 digits must be rejected, not overflow away");
    assert!(e[0].contains("185 digits"), "{}", e[0]);
}

#[test]
fn a_literal_that_fits_is_left_alone() {
    for (ty, v) in [
        ("u8", "255"),
        ("u16", "65535"),
        ("u32", "4294967295"),
        ("u64", "18446744073709551615"),
        ("i32", "2147483647"),
    ] {
        let e = errors_of(&format!("module m\n\nconst C: {ty} = {v}\n"));
        assert!(e.is_empty(), "{ty} = {v} must be accepted, got {e:?}");
    }
}

/// Signed types hold one bit fewer.
///
/// A check that used the full width would accept `const X: i32 = 3000000000`,
/// which is the same defect with a smaller number and no compiler to catch it:
/// `cc` only complains above 2^64.
#[test]
fn a_signed_type_holds_one_bit_fewer() {
    let e = errors_of("module m\n\nconst X: i32 = 3000000000\n");
    assert_eq!(e.len(), 1, "2^31 <= 3000000000 < 2^32, so i32 cannot hold it");
    let ok = errors_of("module m\n\nconst Y: u32 = 3000000000\n");
    assert!(ok.is_empty(), "u32 can, and the same number must be accepted there");
}

/// Underscores are separators, not digits.
#[test]
fn digit_separators_do_not_count_as_width() {
    let e = errors_of("module m\n\nconst C: u32 = 4_294_967_295\n");
    assert!(e.is_empty(), "4_294_967_295 is u32::MAX, got {e:?}");
}

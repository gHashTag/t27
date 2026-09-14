//! A `GF16` parameter must be sixteen bits wide in Verilog.
//!
//! It was thirty-two, and the file already knew better: `HwType::GF16.hw_width()`
//! returns 16 and `HwType::GF16.verilog_range()` returns `[15:0]`, both with
//! passing tests. A function parameter reaches a DIFFERENT reader,
//! `type_to_width`, which had no arm for the type and took `_ => 32`.
//!
//! Two readers of one type, one of them green. The width was not a decision
//! about GF16 at all -- it was the unknown-type default, which is why the
//! negative control below matters more than any of the positive ones: an
//! unknown type must STILL be 32, or the fix is just a wider default.
//!
//! The neighbouring `f64` arm carries the same story in a comment: it "fell
//! through to the 32-bit default and silently narrowed to half its width".
//! This is that defect, in the other direction, on this project's flagship type.
//!
//! 2598 tests passed while this was wrong, so none of them was watching.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

/// The `input` declaration Verilog gives a single parameter of type `ty`.
fn input_decl(ty: &str) -> String {
    let d = std::env::temp_dir().join(format!(
        "t27c-gf16w-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("create temp dir");
    let p = d.join("w.t27");
    std::fs::write(
        &p,
        format!("module P {{\n    fn probe(x: {ty}) -> i32 {{ return 0; }}\n}}\n"),
    )
    .expect("write spec");
    let out = Command::new(env!("CARGO_BIN_EXE_t27c"))
        .arg("gen-verilog")
        .arg(&p)
        .output()
        .expect("run t27c gen-verilog");
    let text = String::from_utf8_lossy(&out.stdout);
    // The declaration line ends `probe; // -> i32`, so anchoring on a newline
    // straight after `probe;` matches nothing -- a matcher failure that reads
    // as a fact about the backend. Take the first `input` line after it.
    let after = text
        .split_once("function")
        .and_then(|(_, r)| r.split_once("probe;"))
        .map(|(_, r)| r.to_string())
        .unwrap_or_else(|| panic!("no `probe` function in gen-verilog output for `{ty}`:\n{text}"));
    after
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("input"))
        .unwrap_or_else(|| panic!("no input declaration for `{ty}`:\n{after}"))
        .to_string()
}

#[test]
fn gf16_is_as_wide_as_u16() {
    // Stated as an EQUALITY with u16 rather than as the literal `[15:0]`: the
    // claim is that GF16 is a sixteen-bit field element, and if the sixteen-bit
    // spelling ever changes both must change together.
    let gf = input_decl("GF16");
    let u16_ = input_decl("u16");
    assert_eq!(
        gf, u16_,
        "GF16 must be declared exactly as wide as u16; got GF16 `{gf}` against u16 `{u16_}`"
    );
    assert!(gf.contains("[15:0]"), "and that width is 16 bits; got `{gf}`");
}

#[test]
fn an_unknown_type_still_takes_the_thirty_two_bit_default() {
    // The negative control, and the load-bearing one. GF16 was 32 because 32 is
    // what an UNKNOWN type gets. A "fix" that widened the default would pass
    // every other test in this file.
    let unknown = input_decl("NoSuchTypeXY");
    assert!(
        unknown.contains("[31:0]"),
        "the unknown-type default must stay 32 bits; got `{unknown}`"
    );
    assert_ne!(
        unknown,
        input_decl("GF16"),
        "GF16 must no longer be indistinguishable from a type this backend has never heard of"
    );
}

#[test]
fn the_qualified_spelling_is_the_same_type() {
    // `gf16::GF16` matched no arm and took the default, so ONE type had two
    // widths depending on how the spec spelled it. Eight corpus specs use the
    // qualified form.
    let bare = input_decl("GF16");
    let qualified = input_decl("gf16::GF16");
    assert_eq!(
        bare, qualified,
        "`GF16` and `gf16::GF16` are one type and must have one width; got `{bare}` against `{qualified}`"
    );
}

#[test]
fn an_array_of_gf16_is_the_product_of_its_parts() {
    // `[4]GF16` was 32 bits TOTAL -- eight bits per element, a quarter of the
    // data -- because GF16 was not among the element types that get the
    // product treatment. Compare `[4]u16`, which was always right.
    let gf = input_decl("[4]GF16");
    let u16s = input_decl("[4]u16");
    assert_eq!(
        gf, u16s,
        "[4]GF16 must pack exactly like [4]u16; got `{gf}` against `{u16s}`"
    );
    assert!(
        gf.contains("[63:0]"),
        "four sixteen-bit elements are 64 bits; got `{gf}`"
    );
}

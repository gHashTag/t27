//! `validate-phi-f64-literals` -- compare the binary64 literals in
//! `coq/Kernel/PhiFloat.v` against independently computed IEEE 754 doubles.
//!
//! `PhiFloat.v` builds its two `binary64` values out of four bare integers:
//!
//! ```coq
//! Definition phi_mantissa : positive := 7286977268806824%positive.
//! Definition phi_exponent : Z := (-52)%Z.
//! Definition one_mantissa : positive := 4503599627370496%positive.
//! Definition one_exponent : Z := (-52)%Z.
//! ```
//!
//! Its header says "Mantissas/exponents must match
//! [scripts/validate_phi_f64.py]", but that script's only `assert` is an
//! internal decode round-trip (`scripts/validate_phi_f64.py:23`,
//! `assert verify == x`); it never opens the Coq file, compares nothing to
//! these four literals, and always exits 0. `validate-phi-identity` cannot
//! cover the gap either: it evaluates `phi^2 + phi^-2` in `f64` from its own
//! `(1.0 + 5.0_f64.sqrt()) / 2.0`, which holds regardless of what the Coq file
//! says. And `phi_f64_bounded` proves only that the pair is a *canonical*
//! binary64 -- true of a great many pairs that are not phi. So a typo in a
//! mantissa changed which real number the whole Coq development was about
//! while every gate stayed green. Issue #2324.
//!
//! This module closes that. It reads the literals out of the file and compares
//! them against parameters decoded from `f64` values computed here, in Rust,
//! from `(1.0 + sqrt 5) / 2.0` and `1.0`. Nothing is compared against a
//! constant copied out of `PhiFloat.v`, which would be a tautology; the only
//! integers written down below are the IEEE 754 field widths.
//!
//! Every mismatch is reported before exiting non-zero, rather than aborting on
//! the first: with an early abort, one corrupted literal would leave the other
//! three assertions unexercised.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;

/// Path of the Coq file, relative to the repo root. The CI step runs from the
/// workspace root, immediately after `cargo build --release -p t27c`.
pub const DEFAULT_PHIFLOAT_PATH: &str = "coq/Kernel/PhiFloat.v";

/// A finite `f64` split the way Flocq's `B754_finite` wants it: the *full*
/// significand (implicit leading bit made explicit) and an exponent already
/// shifted down by the 52 fraction bits, so `x = mantissa * 2^exponent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct F64Params {
    pub negative: bool,
    pub mantissa: u64,
    pub exponent: i64,
    pub bits: u64,
}

/// Decode a finite, normal `f64` into Flocq-style parameters.
///
/// The reconstruction assertion at the end is a self-check on this decoder, not
/// the gate: it is the same round-trip `scripts/validate_phi_f64.py` performs,
/// and on its own it proves nothing about `PhiFloat.v`. Both `mantissa` (< 2^53)
/// and the power of two are exactly representable, so the equality is exact.
pub fn decode_f64(x: f64) -> Result<F64Params> {
    let bits = x.to_bits();
    let exp_biased = ((bits >> 52) & 0x7FF) as i64;
    if exp_biased == 0 || exp_biased == 0x7FF {
        bail!("decode_f64: {x:?} (bits {bits:#018x}) is not a normal finite double");
    }
    let params = F64Params {
        negative: (bits >> 63) & 1 == 1,
        mantissa: (1u64 << 52) | (bits & 0x000F_FFFF_FFFF_FFFF),
        exponent: exp_biased - 1023 - 52,
        bits,
    };
    let round_trip = (params.mantissa as f64) * (params.exponent as f64).exp2();
    let round_trip = if params.negative { -round_trip } else { round_trip };
    if round_trip != x {
        bail!(
            "decode_f64: self-check failed for {x:?}: {} * 2^{} == {round_trip:?}",
            params.mantissa,
            params.exponent
        );
    }
    Ok(params)
}

/// The two doubles `PhiFloat.v` claims to encode, computed here.
///
/// `sqrt` is correctly rounded by IEEE 754, the addition is correctly rounded,
/// and the division by two is exact, so this is the same value on every
/// conforming platform -- and the same expression `run_validate_phi_identity`
/// (`bootstrap/src/main.rs`) uses.
fn expected_phi() -> f64 {
    (1.0 + 5.0_f64.sqrt()) / 2.0
}

/// One literal we expect to find, and the value it must have.
struct Expectation {
    /// Coq definition name.
    name: &'static str,
    /// Coq type as written in the file: part of the anchor, so retyping a
    /// definition fails the gate instead of slipping past it.
    coq_type: &'static str,
    /// Scope suffix on the right-hand side, e.g. `%positive`.
    scope: &'static str,
    /// The value required, rendered as it must appear after normalisation.
    want: i128,
    /// Where that requirement comes from, quoted in the failure message.
    provenance: String,
}

/// Locate `Definition <name> : <coq_type> := <rhs>.` and return `(line_no, rhs)`.
///
/// Anchored to the whole line. Requires *exactly one* definition: zero means
/// the gate silently checked nothing (absence is not a pass), and more than one
/// means a later shadowing definition could decide the value while this gate
/// read the earlier one.
fn find_definition<'a>(
    text: &'a str,
    path: &Path,
    name: &str,
    coq_type: &str,
) -> Result<(usize, &'a str)> {
    let pattern = format!(
        r"^\s*Definition\s+{}\s*:\s*{}\s*:=\s*(\S.*?)\s*\.\s*$",
        regex::escape(name),
        regex::escape(coq_type)
    );
    let re = Regex::new(&pattern).context("building definition regex")?;

    let hits: Vec<(usize, &str)> = text
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            re.captures(line)
                .map(|c| (i + 1, c.get(1).unwrap().as_str()))
        })
        .collect();

    match hits.len() {
        1 => Ok(hits[0]),
        0 => Err(anyhow!(
            "{}: no `Definition {} : {} := ...` found.\n  \
             The gate for issue 2324 must read this literal; a definition it cannot \
             locate is an unchecked literal, not a pass. If the definition was \
             renamed or retyped, update bootstrap/src/phi_f64_literals.rs to match.",
            path.display(),
            name,
            coq_type
        )),
        n => Err(anyhow!(
            "{}: `Definition {} : {} := ...` found {} times (lines {}); expected exactly one.\n  \
             A second definition shadows the first, so which value Coq uses is not \
             what this gate read.",
            path.display(),
            name,
            coq_type,
            n,
            hits.iter()
                .map(|(l, _)| l.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Strip the Coq scope suffix and one layer of parentheses: `(-52)%Z` -> `-52`.
fn normalise_rhs<'a>(rhs: &'a str, scope: &str) -> Result<i128> {
    let body = rhs
        .strip_suffix(scope)
        .ok_or_else(|| anyhow!("right-hand side {rhs:?} does not end in {scope:?}"))?
        .trim();
    let body = match body.strip_prefix('(') {
        Some(inner) => inner
            .strip_suffix(')')
            .ok_or_else(|| anyhow!("unbalanced parenthesis in {rhs:?}"))?
            .trim(),
        None => body,
    };
    body.parse::<i128>()
        .with_context(|| format!("right-hand side {rhs:?} is not an integer literal"))
}

/// The gate. Returns `Err` -- so `main` exits non-zero -- listing every literal
/// that disagrees with IEEE 754.
pub fn run(path: Option<&str>) -> Result<()> {
    let path = PathBuf::from(path.unwrap_or(DEFAULT_PHIFLOAT_PATH));
    let text = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "cannot read {} -- run this from the repository root, or pass --path",
            path.display()
        )
    })?;

    let phi = expected_phi();
    let phi_params = decode_f64(phi)?;
    let one_params = decode_f64(1.0_f64)?;

    let phi_src = format!(
        "fl((1 + sqrt 5) / 2) = {:?}, bits {:#018x}, hex {}",
        phi,
        phi_params.bits,
        format_hex(phi)
    );
    let one_src = format!(
        "fl(1.0) = {:?}, bits {:#018x}, hex {}",
        1.0_f64,
        one_params.bits,
        format_hex(1.0_f64)
    );

    if phi_params.negative || one_params.negative {
        bail!("computed phi or 1.0 decoded as negative; the decoder is wrong");
    }

    let expectations = [
        Expectation {
            name: "phi_mantissa",
            coq_type: "positive",
            scope: "%positive",
            want: phi_params.mantissa as i128,
            provenance: format!("full significand of {phi_src}"),
        },
        Expectation {
            name: "phi_exponent",
            coq_type: "Z",
            scope: "%Z",
            want: phi_params.exponent as i128,
            provenance: format!("unbiased exponent minus 52 of {phi_src}"),
        },
        Expectation {
            name: "one_mantissa",
            coq_type: "positive",
            scope: "%positive",
            want: one_params.mantissa as i128,
            provenance: format!("full significand of {one_src}"),
        },
        Expectation {
            name: "one_exponent",
            coq_type: "Z",
            scope: "%Z",
            want: one_params.exponent as i128,
            provenance: format!("unbiased exponent minus 52 of {one_src}"),
        },
    ];

    let mut failures: Vec<String> = Vec::new();
    let mut checked: Vec<String> = Vec::new();

    for exp in &expectations {
        let (line_no, rhs) = match find_definition(&text, &path, exp.name, exp.coq_type) {
            Ok(hit) => hit,
            Err(e) => {
                failures.push(format!("{e}"));
                continue;
            }
        };
        let got = match normalise_rhs(rhs, exp.scope) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!(
                    "{}:{}: {} := {rhs} -- {e}",
                    path.display(),
                    line_no,
                    exp.name
                ));
                continue;
            }
        };
        if got == exp.want {
            checked.push(format!(
                "  OK  {}:{:<4} {:<13} = {:<20} ({})",
                path.display(),
                line_no,
                exp.name,
                got,
                exp.provenance
            ));
        } else {
            failures.push(format!(
                "{}:{}: {} = {} but IEEE 754 binary64 requires {}\n       \
                 source of truth: {}\n       \
                 (this literal decides which real number PhiFloat.v is about; \
                 phi_f64_bounded proves only that the pair is a canonical binary64)",
                path.display(),
                line_no,
                exp.name,
                got,
                exp.want,
                exp.provenance
            ));
        }
    }

    if !failures.is_empty() {
        for line in &checked {
            eprintln!("{line}");
        }
        let mut msg = format!(
            "PhiFloat.v binary64 literals do NOT match IEEE 754 ({} of {} literals bad):",
            failures.len(),
            expectations.len()
        );
        for f in &failures {
            msg.push_str("\n  FAIL ");
            msg.push_str(f);
        }
        bail!("{msg}");
    }

    println!(
        "PHI f64 LITERAL CHECK PASSED: {} of {}'s binary64 literals match independently \
         computed IEEE 754 doubles.",
        expectations.len(),
        path.display()
    );
    for line in &checked {
        println!("{line}");
    }
    Ok(())
}

/// `f64::to_hex`-style rendering (`0x1.<frac>p<exp>`), matching what
/// `float.hex()` prints in `scripts/validate_phi_f64.py` and the
/// `phi_sq_f64_hex` field in `conformance/FORMAT-SPEC-001.json`.
fn format_hex(x: f64) -> String {
    let bits = x.to_bits();
    let exp = ((bits >> 52) & 0x7FF) as i64 - 1023;
    let frac = bits & 0x000F_FFFF_FFFF_FFFF;
    let sign = if (bits >> 63) & 1 == 1 { "-" } else { "" };
    format!("{sign}0x1.{frac:013x}p{exp:+}")
}

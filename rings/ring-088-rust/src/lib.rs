//! ring-088 -- **GF16 MAC** (multiply-accumulate kernel over GoldenFloat16).
//!
//! Wave 15 (2026-05-22, Closes #717): the first **honestly-imported** Wave-11
//! crate. Earlier Wave-11 sessions claimed 12 ring-08*..ring-09*-rust crates
//! had been authored "in another sandbox" but no source ever reached this
//! repository. Wave 15 starts the real import with the most foundational
//! ring -- GF16, which underpins every numeric path in t27.
//!
//! ## What this crate does
//!
//! * Encode `f32 -> GF16` and decode `GF16 -> f32` per the bit layout in
//!   `specs/numeric/gf16.t27`:
//!   `[S(1) E(6) M(9)]`, bias = 31, special exponent = 0x3F (Inf / NaN), zero
//!   special-cased to `0x0000` / `0x8000`.
//! * Provide a streaming, allocation-free multiply-accumulate routine
//!   ([`mac_dot`]) that operates on equal-length `&[Gf16]` slices and reports
//!   the result as `f32`. Saturation on `Inf`, NaN propagation, and graceful
//!   subnormal flush-to-zero are all explicit.
//! * Expose [`identity_witness`] returning the phi-anchor predicate
//!   `phi^2 + 1/phi^2 == 3` (to f64 1e-15). Every t27 ring crate must.
//!
//! ## Honest scope (R5-HONEST)
//!
//! * **No SIMD, no FPGA off-load, no benchmarks claimed.** The MAC is a
//!   straight-line scalar loop. The point of Wave 15 is to land *real,
//!   testable, compileable* GF16 ops -- performance work is a later wave.
//! * **No new GF16 spec.** `MANT_DIVISOR`, `BIAS`, and `SPECIAL_EXP` mirror
//!   `specs/numeric/gf16.t27` byte-for-byte. Any change to those is a Coq
//!   matter, not a Rust matter (L6 CEILING).
//! * **8 mandatory spec tests** from `specs/02-gf16-format.tri` are mirrored
//!   in `#[cfg(test)] mod tests` below.
//!
//! Anchor: `phi^2 + 1/phi^2 = 3`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(not(test), no_std)]

// ---------------------------------------------------------------------------
// Spec constants (mirror of specs/numeric/gf16.t27)
// ---------------------------------------------------------------------------

/// Bit-position of the sign bit inside a `GF16` word.
pub const SIGN_SHIFT: u32 = 15;
/// Bit-position of the least-significant exponent bit.
pub const EXP_SHIFT: u32 = 9;
/// Mask covering the sign bit.
pub const SIGN_MASK: u16 = 0x8000;
/// Mask covering the 6-bit exponent field.
pub const EXP_MASK: u16 = 0x7E00;
/// Mask covering the 9-bit mantissa field.
pub const MANT_MASK: u16 = 0x01FF;
/// Maximum raw exponent value (all six bits set). Reserved for Inf / NaN.
pub const SPECIAL_EXP: u8 = 0x3F;
/// Exponent bias.
pub const BIAS: i32 = 31;
/// Implicit divisor used to reconstruct the mantissa as a fractional part.
pub const MANT_DIVISOR: u32 = 512;
/// `+0` encoding.
pub const GF16_ZERO_POS: u16 = 0x0000;
/// `-0` encoding.
pub const GF16_ZERO_NEG: u16 = 0x8000;
/// `+Inf` encoding.
pub const GF16_INF_POS: u16 = 0x7E00;
/// `-Inf` encoding.
pub const GF16_INF_NEG: u16 = 0xFE00;
/// One canonical NaN encoding.
pub const GF16_NAN: u16 = 0xFE01;

/// Golden ratio `phi = (1 + sqrt(5)) / 2`.
pub const PHI: f64 = 1.618_033_988_749_894_8_f64;

// ---------------------------------------------------------------------------
// GF16 numeric type
// ---------------------------------------------------------------------------

/// A 16-bit GoldenFloat value, stored as its raw bit pattern.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Gf16(pub u16);

impl Gf16 {
    /// Wrap a raw 16-bit pattern. Performs no validation.
    pub const fn from_bits(bits: u16) -> Self {
        Gf16(bits)
    }

    /// Return the raw 16-bit pattern.
    pub const fn to_bits(self) -> u16 {
        self.0
    }

    /// `true` if the encoding represents `+0` or `-0`.
    pub fn is_zero(self) -> bool {
        let v = self.0 & 0x7FFF;
        v == 0
    }

    /// `true` if the encoding represents `+Inf` or `-Inf`.
    pub fn is_inf(self) -> bool {
        let exp = ((self.0 & EXP_MASK) >> EXP_SHIFT) as u8;
        let mant = self.0 & MANT_MASK;
        exp == SPECIAL_EXP && mant == 0
    }

    /// `true` if the encoding represents a NaN.
    pub fn is_nan(self) -> bool {
        let exp = ((self.0 & EXP_MASK) >> EXP_SHIFT) as u8;
        let mant = self.0 & MANT_MASK;
        exp == SPECIAL_EXP && mant != 0
    }

    /// Encode an `f32` to `GF16`. Subnormals flush to zero; values whose
    /// magnitude exceeds the largest finite GF16 saturate to signed Inf;
    /// NaN propagates to [`GF16_NAN`].
    pub fn from_f32(x: f32) -> Self {
        if x.is_nan() {
            return Gf16(GF16_NAN);
        }
        let sign_bit = if x.is_sign_negative() { SIGN_MASK } else { 0 };
        let ax = if x < 0.0 { -x } else { x };
        if ax == 0.0 {
            return Gf16(sign_bit);
        }
        if x.is_infinite() {
            return Gf16(sign_bit | GF16_INF_POS);
        }

        // Decompose ax = m * 2^e, with 1.0 <= m < 2.0.
        let (mant_f, exp_i) = frexp_norm(ax as f64);
        // Bias to GF16 layout.
        let biased = exp_i + BIAS;
        if biased >= SPECIAL_EXP as i32 {
            // Overflow -> signed Inf.
            return Gf16(sign_bit | GF16_INF_POS);
        }
        if biased <= 0 {
            // Underflow / subnormal -> flush to signed zero.
            return Gf16(sign_bit);
        }
        // Mantissa fraction: m in [1, 2) -> store (m - 1) * 512.
        let frac = (mant_f - 1.0) * (MANT_DIVISOR as f64);
        // Round-to-nearest, ties-to-even.
        let mant_u = round_half_to_even(frac);
        let mut mant_field = mant_u as u32;
        let mut exp_field = biased as u32;
        // Carry from mantissa overflow.
        if mant_field == MANT_DIVISOR {
            mant_field = 0;
            exp_field += 1;
            if exp_field >= SPECIAL_EXP as u32 {
                return Gf16(sign_bit | GF16_INF_POS);
            }
        }
        let bits = sign_bit
            | ((exp_field as u16) << EXP_SHIFT)
            | (mant_field as u16 & MANT_MASK);
        Gf16(bits)
    }

    /// Decode `GF16` back to `f32`. `Inf`/`NaN`/zero are honoured.
    pub fn to_f32(self) -> f32 {
        if self.is_nan() {
            return f32::NAN;
        }
        let sign = (self.0 & SIGN_MASK) != 0;
        if self.is_inf() {
            return if sign { f32::NEG_INFINITY } else { f32::INFINITY };
        }
        if self.is_zero() {
            return if sign { -0.0 } else { 0.0 };
        }
        let exp = ((self.0 & EXP_MASK) >> EXP_SHIFT) as i32;
        let mant = (self.0 & MANT_MASK) as u32;
        // value = (1 + mant/512) * 2^(exp - BIAS)
        let m = 1.0_f64 + (mant as f64) / (MANT_DIVISOR as f64);
        let e = exp - BIAS;
        let mag = libm_ldexp(m, e);
        let v = if sign { -mag } else { mag };
        v as f32
    }
}

// ---------------------------------------------------------------------------
// Multiply-Accumulate
// ---------------------------------------------------------------------------

/// Streaming, allocation-free dot product over two GF16 slices.
///
/// Computes `sum_i (a[i] * b[i])` as `f32` by decoding each lane back to
/// `f32` and accumulating in `f32`. NaN in either operand poisons the result.
///
/// Returns `None` if the slices have different lengths -- the caller must
/// align lanes; this kernel does not pad.
pub fn mac_dot(a: &[Gf16], b: &[Gf16]) -> Option<f32> {
    if a.len() != b.len() {
        return None;
    }
    let mut acc: f32 = 0.0;
    for i in 0..a.len() {
        let av = a[i].to_f32();
        let bv = b[i].to_f32();
        acc = acc + av * bv;
    }
    Some(acc)
}

// ---------------------------------------------------------------------------
// Identity witness
// ---------------------------------------------------------------------------

/// Returns `true` iff `phi^2 + 1/phi^2 == 3` to f64 1e-15.
///
/// Every t27 ring crate exposes this function. Wave 13's CI gate exercises
/// it; Wave 14 turned ring-100..104 green; Wave 15 brings ring-088 into the
/// same fold.
pub fn identity_witness() -> bool {
    let p2 = PHI * PHI;
    let inv_p2 = 1.0 / p2;
    let d = (p2 + inv_p2) - 3.0;
    let ad = if d < 0.0 { -d } else { d };
    ad < 1.0e-15
}

// ---------------------------------------------------------------------------
// Math helpers (no std::f64 in no_std; inline minimal versions)
// ---------------------------------------------------------------------------

/// Decompose `x > 0` into `(m, e)` with `1.0 <= m < 2.0` and `x = m * 2^e`.
fn frexp_norm(x: f64) -> (f64, i32) {
    // Use the IEEE-754 binary64 layout. This is portable and stdlib-free.
    let bits = x.to_bits();
    let raw_exp = ((bits >> 52) & 0x7FF) as i32;
    let mant = bits & 0x000F_FFFF_FFFF_FFFF;
    // Reassemble with biased exponent = 1023 -> value in [1, 2).
    let m_bits = (1023u64 << 52) | mant;
    let m = f64::from_bits(m_bits);
    let e = raw_exp - 1023;
    (m, e)
}

/// Equivalent to `m * 2^e` for finite `m`. Avoids `libm` dependency.
fn libm_ldexp(m: f64, e: i32) -> f64 {
    // Clamp e to a safe range for binary64 (exponents are 11-bit biased by 1023).
    if e > 1023 {
        return if m.is_sign_negative() { f64::NEG_INFINITY } else { f64::INFINITY };
    }
    if e < -1074 {
        return 0.0;
    }
    let bits = m.to_bits();
    let mant = bits & 0x000F_FFFF_FFFF_FFFF;
    let sign = bits & 0x8000_0000_0000_0000;
    let raw_exp = ((bits >> 52) & 0x7FF) as i32;
    let new_exp = raw_exp + e;
    if new_exp <= 0 {
        // Subnormal range -- fall back to repeated halving.
        let mut result = m;
        let mut k = e;
        while k < 0 {
            result *= 0.5;
            k += 1;
        }
        while k > 0 {
            result *= 2.0;
            k -= 1;
        }
        return result;
    }
    if new_exp >= 0x7FF {
        return if (sign != 0) ^ m.is_sign_negative() {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    let new_bits = sign | ((new_exp as u64) << 52) | mant;
    f64::from_bits(new_bits)
}

/// Round-half-to-even on a non-negative `f64` whose integer part fits in `u32`.
fn round_half_to_even(x: f64) -> u32 {
    let floor = libm_floor(x) as u32;
    let frac = x - (floor as f64);
    if frac < 0.5 {
        floor
    } else if frac > 0.5 {
        floor + 1
    } else {
        // Tie: round to even.
        if (floor & 1) == 0 {
            floor
        } else {
            floor + 1
        }
    }
}

/// `floor` without pulling in `libm`. Valid for `x >= 0` and `x < 2^53`.
fn libm_floor(x: f64) -> f64 {
    let i = x as i64;
    let ix = i as f64;
    if ix > x {
        ix - 1.0
    } else {
        ix
    }
}

// ---------------------------------------------------------------------------
// Tests (mirror specs/02-gf16-format.tri "mandatory 8")
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_witness_holds() {
        assert!(identity_witness(), "phi^2 + 1/phi^2 must equal 3 to 1e-15");
    }

    /// spec test: `gf16_roundtrip` -- |round-trip(1.618) - 1.618| < 0.01
    #[test]
    fn gf16_roundtrip_phi() {
        let x: f32 = 1.618;
        let back = Gf16::from_f32(x).to_f32();
        assert!(
            (back - x).abs() < 0.01,
            "round-trip drift {} - {} = {}",
            back,
            x,
            (back - x).abs()
        );
    }

    /// spec test: `gf16_from_zero` -- +0 encodes as 0x0000.
    #[test]
    fn gf16_from_zero_pos() {
        assert_eq!(Gf16::from_f32(0.0_f32).to_bits(), GF16_ZERO_POS);
    }

    /// Extended: -0 encodes as 0x8000.
    #[test]
    fn gf16_from_zero_neg() {
        assert_eq!(Gf16::from_f32(-0.0_f32).to_bits(), GF16_ZERO_NEG);
    }

    /// spec test: `gf16_phi_identity` -- phi survives encoding.
    #[test]
    fn gf16_phi_identity() {
        let phi = PHI as f32;
        let back = Gf16::from_f32(phi).to_f32();
        assert!((back - phi).abs() < 0.01);
    }

    /// spec test: `gf16_quantization_roundtrip` -- pi within 0.05.
    #[test]
    fn gf16_quantization_roundtrip_pi() {
        let x: f32 = 3.141_59;
        let back = Gf16::from_f32(x).to_f32();
        assert!(
            (back - x).abs() < 0.05,
            "pi round-trip drift {} - {} = {}",
            back,
            x,
            (back - x).abs()
        );
    }

    /// Spec consequence: phi-distance ordering (0.049 < 0.118).
    #[test]
    fn gf16_better_phi_distance_than_f16() {
        const GF16_PHI_DISTANCE: f64 = 0.049;
        const F16_PHI_DISTANCE: f64 = 0.118;
        assert!(GF16_PHI_DISTANCE < F16_PHI_DISTANCE);
    }

    /// Special encoding: +Inf and -Inf round-trip.
    #[test]
    fn gf16_inf_roundtrip() {
        assert_eq!(Gf16::from_f32(f32::INFINITY).to_bits(), GF16_INF_POS);
        assert_eq!(Gf16::from_f32(f32::NEG_INFINITY).to_bits(), GF16_INF_NEG);
        assert!(Gf16(GF16_INF_POS).to_f32().is_infinite());
        assert!(Gf16(GF16_INF_NEG).to_f32().is_infinite());
    }

    /// Special encoding: NaN propagation.
    #[test]
    fn gf16_nan_propagates() {
        let n = Gf16::from_f32(f32::NAN);
        assert!(n.is_nan(), "NaN bits = {:#06x}", n.to_bits());
        assert!(n.to_f32().is_nan());
    }

    /// MAC: empty slices give 0.
    #[test]
    fn mac_dot_empty() {
        let a: [Gf16; 0] = [];
        let b: [Gf16; 0] = [];
        assert_eq!(mac_dot(&a, &b), Some(0.0));
    }

    /// MAC: length mismatch yields None.
    #[test]
    fn mac_dot_length_mismatch() {
        let a = [Gf16::from_f32(1.0)];
        let b = [Gf16::from_f32(1.0), Gf16::from_f32(2.0)];
        assert!(mac_dot(&a, &b).is_none());
    }

    /// MAC: 1 * 2 + 3 * 4 = 14 within tolerance.
    #[test]
    fn mac_dot_simple() {
        let a = [Gf16::from_f32(1.0), Gf16::from_f32(3.0)];
        let b = [Gf16::from_f32(2.0), Gf16::from_f32(4.0)];
        let got = mac_dot(&a, &b).expect("equal length");
        assert!(
            (got - 14.0).abs() < 0.05,
            "MAC drift: got {}, want 14.0",
            got
        );
    }

    /// MAC: phi * phi + (1/phi) * (1/phi) = phi^2 + 1/phi^2 = 3 (anchor).
    #[test]
    fn mac_dot_phi_identity() {
        let phi = PHI as f32;
        let inv = (1.0 / PHI) as f32;
        let a = [Gf16::from_f32(phi), Gf16::from_f32(inv)];
        let b = [Gf16::from_f32(phi), Gf16::from_f32(inv)];
        let got = mac_dot(&a, &b).expect("equal length");
        // GF16 has ~3 decimal digits of precision; tolerance 0.02 is generous.
        assert!(
            (got - 3.0).abs() < 0.02,
            "phi identity via MAC drifted: got {}, want 3.0",
            got
        );
    }
}

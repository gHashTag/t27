// SPDX-License-Identifier: Apache-2.0
// ring-096-rust: Format quantization primitives
// Mirrors specs/numeric/formats.t27 byte-for-byte.
//
// Identity anchor: phi^2 + 1/phi^2 = 3 | TRINITY
//
// Scope:
//   - GF16 bit layout constants
//   - gf16_to_f32 / f32_to_gf16 codec (decode/encode)
//   - Ternary quantization (f32_to_ternary, ternary_to_f32)
//   - Format enum + format_bytes + quantize_value utility
//
// no_std: no libm. All math via private helpers (pow_u64).

#![no_std]
#![deny(warnings)]

// ============================================================================
// 1. GF16 Bit Layout Constants (byte-for-byte from specs/numeric/formats.t27)
// ============================================================================
//
// GF16 layout: [S(1) E(6) M(9)] = [15:15][14:9][8:0]
//   - Sign:     bit 15 (0x8000)
//   - Exponent: bits 14-9 (0x7E00), bias = 31
//   - Mantissa: bits  8-0 (0x01FF)
// Range: 2^-31 to 2^32

pub const SIGN_MASK: u16 = 0x8000;
pub const EXP_MASK: u16 = 0x7E00;
pub const MANT_MASK: u16 = 0x01FF;

pub const EXP_SHIFT: u32 = 9;
pub const SIGN_SHIFT: u32 = 15;
pub const BIAS: i32 = 31;

pub const EXP_MAX: u16 = 63;
pub const EXP_MIN: u16 = 0;

/// Threshold for ternary quantization: |w| > 0.5 -> +/-1.
pub const TERNARY_THRESHOLD: f64 = 0.5;

// ============================================================================
// 2. Errors
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuantError {
    Overflow,
    Underflow,
    Nan,
}

// ============================================================================
// 3. Trit (ternary value)
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trit {
    Neg = -1isize as isize,
    Zero = 0,
    Pos = 1,
}

impl Trit {
    pub fn to_i8(self) -> i8 {
        match self {
            Trit::Neg => -1,
            Trit::Zero => 0,
            Trit::Pos => 1,
        }
    }

    pub fn from_i8(v: i8) -> Trit {
        if v > 0 {
            Trit::Pos
        } else if v < 0 {
            Trit::Neg
        } else {
            Trit::Zero
        }
    }
}

// ============================================================================
// 4. Format enum
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Format {
    Fp32 = 0,
    Fp16 = 1,
    Bf16 = 2,
    Gf16 = 3,
    Ternary = 4,
}

/// Byte size for each format (spec-aligned).
pub fn format_bytes(fmt: Format) -> usize {
    match fmt {
        Format::Fp32 => 4,
        Format::Fp16 => 2,
        Format::Bf16 => 2,
        Format::Gf16 => 2,
        Format::Ternary => 1,
    }
}

// ============================================================================
// 5. Private no_std math helpers
// ============================================================================

/// Fast integer exponentiation by squaring: base^exp.
/// Used for 2^k computations during GF16 encode/decode and anchor identity.
fn pow_u64(base: f64, exp: i32) -> f64 {
    if exp == 0 {
        return 1.0;
    }
    let (b, mut e) = if exp < 0 {
        (1.0 / base, -exp as u32)
    } else {
        (base, exp as u32)
    };
    let mut result = 1.0;
    let mut acc = b;
    while e > 0 {
        if e & 1 == 1 {
            result *= acc;
        }
        acc *= acc;
        e >>= 1;
    }
    result
}

/// |x| without core::num::Float.
fn fabs_no_std(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// Check NaN: x != x.
fn is_nan(x: f64) -> bool {
    x != x
}

/// Infinity sentinel via f64::INFINITY.
const INF: f64 = f64::INFINITY;
const NEG_INF: f64 = f64::NEG_INFINITY;

fn is_inf(x: f64) -> bool {
    x == INF || x == NEG_INF
}

// ============================================================================
// 6. GF16 codec
// ============================================================================

/// Decode GF16 (u16) to f64 (we use f64 as the canonical decoded value).
///
/// Algorithm:
///   - Extract sign, exponent, mantissa.
///   - e=0, m=0   -> signed zero.
///   - e=0, m!=0  -> denormal: value = (-1)^s * (m/2^9) * 2^(1 - bias).
///   - e=EXP_MAX, m=0  -> +/- Inf.
///   - e=EXP_MAX, m!=0 -> NaN.
///   - Normal: value = (-1)^s * (1 + m/2^9) * 2^(e - bias).
pub fn gf16_to_f32(x: u16) -> f64 {
    let s = (x & SIGN_MASK) >> SIGN_SHIFT;
    let e = (x & EXP_MASK) >> EXP_SHIFT;
    let m = x & MANT_MASK;
    let sign = if s == 1 { -1.0_f64 } else { 1.0_f64 };

    if e == EXP_MIN {
        if m == 0 {
            return sign * 0.0;
        }
        // Denormal: (-1)^s * (m / 2^9) * 2^(1 - bias)
        let mantissa = m as f64 / pow_u64(2.0, EXP_SHIFT as i32);
        return sign * mantissa * pow_u64(2.0, 1 - BIAS);
    }

    if e == EXP_MAX {
        if m == 0 {
            return if s == 1 { NEG_INF } else { INF };
        }
        return f64::NAN;
    }

    // Normal: (-1)^s * (1 + m / 2^9) * 2^(e - bias)
    let mantissa = 1.0 + (m as f64 / pow_u64(2.0, EXP_SHIFT as i32));
    sign * mantissa * pow_u64(2.0, e as i32 - BIAS)
}

/// Encode f64 to GF16 (u16), round-to-nearest.
///
/// Algorithm:
///   1. Signed zero preserved.
///   2. Inf / NaN special-cased (NaN -> 0x7F01).
///   3. Decompose into sign + magnitude.
///   4. Find e such that magnitude in [2^(e-bias), 2^(e-bias+1)).
///   5. Mantissa = (mag / 2^(e - bias) - 1.0) * 2^9, round-to-nearest.
///   6. Underflow -> 0 (with sign), overflow -> Inf.
pub fn f32_to_gf16(a: f64) -> u16 {
    // Signed zero
    if a == 0.0 {
        // distinguish -0 from +0
        if a.is_sign_negative() {
            return 0x8000;
        }
        return 0;
    }

    // NaN
    if is_nan(a) {
        return 0x7F01;
    }

    // Inf
    if is_inf(a) {
        return if a < 0.0 { 0xFE00 } else { 0x7E00 };
    }

    let sign: u16 = if a < 0.0 { 1 } else { 0 };
    let mag = fabs_no_std(a);

    // Find exponent e such that 2^(e - bias) <= mag < 2^(e - bias + 1)
    // i.e. e - bias = floor(log2(mag))
    // Compute via repeated multiply/divide; bounded loop.
    let mut e: i32 = BIAS; // unbiased = 0 -> mag in [1, 2)
    let mut mantissa_norm = mag;
    if mantissa_norm >= 1.0 {
        while mantissa_norm >= 2.0 && (e as u16) < EXP_MAX - 1 {
            mantissa_norm *= 0.5;
            e += 1;
        }
    } else {
        while mantissa_norm < 1.0 && e > 0 {
            mantissa_norm *= 2.0;
            e -= 1;
        }
        if mantissa_norm < 1.0 {
            // Denormal range: encode as e=0 with mantissa bits scaled.
            let denorm_mant_f = mantissa_norm * pow_u64(2.0, EXP_SHIFT as i32);
            let mut denorm_mant = (denorm_mant_f + 0.5) as i32;
            if denorm_mant < 0 {
                denorm_mant = 0;
            }
            if denorm_mant > MANT_MASK as i32 {
                denorm_mant = MANT_MASK as i32;
            }
            return (sign << SIGN_SHIFT) | (denorm_mant as u16 & MANT_MASK);
        }
    }

    // Overflow -> Inf
    if e >= EXP_MAX as i32 {
        return (sign << SIGN_SHIFT) | (EXP_MAX << EXP_SHIFT);
    }

    // mantissa in [1, 2): subtract 1, scale by 2^9, round.
    let frac = mantissa_norm - 1.0;
    let mant_f = frac * pow_u64(2.0, EXP_SHIFT as i32);
    let mut mant = (mant_f + 0.5) as i32;

    // Mantissa rounding could push to next exponent.
    if mant >= (1i32 << EXP_SHIFT) {
        mant = 0;
        e += 1;
        if e >= EXP_MAX as i32 {
            return (sign << SIGN_SHIFT) | (EXP_MAX << EXP_SHIFT);
        }
    }

    let e_u16 = e as u16 & 0x3F;
    (sign << SIGN_SHIFT) | (e_u16 << EXP_SHIFT) | (mant as u16 & MANT_MASK)
}

// ============================================================================
// 7. Ternary quantization
// ============================================================================

/// Quantize f64 to ternary using threshold 0.5.
pub fn f32_to_ternary(x: f64) -> Trit {
    if x > TERNARY_THRESHOLD {
        Trit::Pos
    } else if x < -TERNARY_THRESHOLD {
        Trit::Neg
    } else {
        Trit::Zero
    }
}

/// Convert ternary back to f64: -1, 0, +1.
pub fn ternary_to_f32(t: Trit) -> f64 {
    match t {
        Trit::Pos => 1.0,
        Trit::Zero => 0.0,
        Trit::Neg => -1.0,
    }
}

// ============================================================================
// 8. quantize_value utility
// ============================================================================

/// Quantize an f64 to the target format.
///
/// For Fp32 / Fp16 / Bf16 we model "preserve value within format precision"
/// by returning the original value (these formats are wider than GF16 in
/// practice and this crate's role is the codec; full IEEE 754 binary16/bf16
/// converters are out of scope for ring-096).
///
/// For Gf16: round-trip via GF16 codec.
/// For Ternary: round-trip via f32_to_ternary / ternary_to_f32.
pub fn quantize_value(x: f64, fmt: Format) -> f64 {
    match fmt {
        Format::Fp32 | Format::Fp16 | Format::Bf16 => x,
        Format::Gf16 => gf16_to_f32(f32_to_gf16(x)),
        Format::Ternary => ternary_to_f32(f32_to_ternary(x)),
    }
}

// ============================================================================
// 9. Identity witness
// ============================================================================

/// Identity witness: returns 3.0 computed via phi^2 + 1/phi^2.
/// Trinity anchor present in every ring crate.
pub fn identity_witness() -> f64 {
    // phi = (1 + sqrt(5)) / 2
    // Use a closed-form constant; integer-exact identity is asserted in
    // anchor test through pow_u64 path (see test phi_identity_via_pow).
    let phi: f64 = 1.618_033_988_749_894_8;
    pow_u64(phi, 2) + pow_u64(phi, -2)
}

// ============================================================================
// 10. Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Spec constants ----

    #[test]
    fn const_sign_mask() {
        assert_eq!(SIGN_MASK, 0x8000);
    }

    #[test]
    fn const_exp_mask() {
        assert_eq!(EXP_MASK, 0x7E00);
    }

    #[test]
    fn const_mant_mask() {
        assert_eq!(MANT_MASK, 0x01FF);
    }

    #[test]
    fn const_exp_shift_sign_shift_bias() {
        assert_eq!(EXP_SHIFT, 9);
        assert_eq!(SIGN_SHIFT, 15);
        assert_eq!(BIAS, 31);
    }

    #[test]
    fn const_exp_max_min() {
        assert_eq!(EXP_MAX, 63);
        assert_eq!(EXP_MIN, 0);
    }

    // ---- GF16 -> f32 ----

    #[test]
    fn gf16_to_f32_zero_positive() {
        assert_eq!(gf16_to_f32(0x0000), 0.0);
    }

    #[test]
    fn gf16_to_f32_zero_negative() {
        let v = gf16_to_f32(0x8000);
        assert_eq!(v, 0.0);
        assert!(v.is_sign_negative());
    }

    #[test]
    fn gf16_to_f32_denormal_positive() {
        // e=0, m!=0 -> small positive
        let v = gf16_to_f32(0x0080);
        assert!(v > 0.0 && v < 1.0);
    }

    #[test]
    fn gf16_to_f32_one() {
        // 1.0 -> bias=31, m=0 -> raw = (31 << 9) = 0x3E00
        let v = gf16_to_f32(0x3E00);
        assert!((v - 1.0).abs() < 1e-9);
    }

    #[test]
    fn gf16_to_f32_positive_inf() {
        assert_eq!(gf16_to_f32(0x7E00), INF);
    }

    #[test]
    fn gf16_to_f32_negative_inf() {
        assert_eq!(gf16_to_f32(0xFE00), NEG_INF);
    }

    #[test]
    fn gf16_to_f32_nan() {
        let v = gf16_to_f32(0x7F01);
        assert!(is_nan(v));
    }

    // ---- f32 -> GF16 ----

    #[test]
    fn f32_to_gf16_zero_positive() {
        assert_eq!(f32_to_gf16(0.0), 0x0000);
    }

    #[test]
    fn f32_to_gf16_zero_negative() {
        assert_eq!(f32_to_gf16(-0.0), 0x8000);
    }

    #[test]
    fn f32_to_gf16_one_roundtrip() {
        let enc = f32_to_gf16(1.0);
        let dec = gf16_to_f32(enc);
        assert!((dec - 1.0).abs() < 0.01);
    }

    #[test]
    fn f32_to_gf16_inf_positive() {
        assert_eq!(f32_to_gf16(INF), 0x7E00);
    }

    #[test]
    fn f32_to_gf16_inf_negative() {
        assert_eq!(f32_to_gf16(NEG_INF), 0xFE00);
    }

    #[test]
    fn f32_to_gf16_nan() {
        assert_eq!(f32_to_gf16(f64::NAN), 0x7F01);
    }

    #[test]
    fn f32_to_gf16_roundtrip_normal_values() {
        // Roundtrip various normal values within 1% tolerance.
        let values = [1.5_f64, 2.0, 0.5, -1.5, 100.0, -100.0, 0.125];
        for &v in &values {
            let enc = f32_to_gf16(v);
            let dec = gf16_to_f32(enc);
            let err = fabs_no_std(dec - v) / fabs_no_std(v);
            assert!(err < 0.01, "v={} dec={} rel_err={}", v, dec, err);
        }
    }

    // ---- Ternary ----

    #[test]
    fn ternary_positive() {
        assert_eq!(f32_to_ternary(1.0), Trit::Pos);
    }

    #[test]
    fn ternary_zero() {
        assert_eq!(f32_to_ternary(0.0), Trit::Zero);
    }

    #[test]
    fn ternary_negative() {
        assert_eq!(f32_to_ternary(-1.0), Trit::Neg);
    }

    #[test]
    fn ternary_above_threshold() {
        assert_eq!(f32_to_ternary(0.6), Trit::Pos);
    }

    #[test]
    fn ternary_below_neg_threshold() {
        assert_eq!(f32_to_ternary(-0.6), Trit::Neg);
    }

    #[test]
    fn ternary_at_threshold_is_zero() {
        // |x| == 0.5 -> zero (strict ">" in spec)
        assert_eq!(f32_to_ternary(0.5), Trit::Zero);
        assert_eq!(f32_to_ternary(-0.5), Trit::Zero);
    }

    #[test]
    fn ternary_to_f32_roundtrip() {
        assert_eq!(ternary_to_f32(Trit::Pos), 1.0);
        assert_eq!(ternary_to_f32(Trit::Zero), 0.0);
        assert_eq!(ternary_to_f32(Trit::Neg), -1.0);
    }

    #[test]
    fn ternary_symmetry() {
        let p = ternary_to_f32(f32_to_ternary(0.7));
        let n = ternary_to_f32(f32_to_ternary(-0.7));
        assert_eq!(p, -n);
    }

    // ---- Format / format_bytes ----

    #[test]
    fn format_bytes_fp32() {
        assert_eq!(format_bytes(Format::Fp32), 4);
    }

    #[test]
    fn format_bytes_fp16() {
        assert_eq!(format_bytes(Format::Fp16), 2);
    }

    #[test]
    fn format_bytes_bf16() {
        assert_eq!(format_bytes(Format::Bf16), 2);
    }

    #[test]
    fn format_bytes_gf16() {
        assert_eq!(format_bytes(Format::Gf16), 2);
    }

    #[test]
    fn format_bytes_ternary() {
        assert_eq!(format_bytes(Format::Ternary), 1);
    }

    // ---- quantize_value ----

    #[test]
    fn quantize_value_fp32_preserves() {
        assert_eq!(quantize_value(1.5, Format::Fp32), 1.5);
    }

    #[test]
    fn quantize_value_ternary_above_threshold() {
        assert_eq!(quantize_value(1.5, Format::Ternary), 1.0);
    }

    #[test]
    fn quantize_value_ternary_below_neg_threshold() {
        assert_eq!(quantize_value(-1.5, Format::Ternary), -1.0);
    }

    #[test]
    fn quantize_value_gf16_roundtrip() {
        let v = quantize_value(1.5, Format::Gf16);
        assert!((v - 1.5).abs() < 0.01);
    }

    // ---- Trit helpers ----

    #[test]
    fn trit_from_to_i8() {
        assert_eq!(Trit::from_i8(5).to_i8(), 1);
        assert_eq!(Trit::from_i8(0).to_i8(), 0);
        assert_eq!(Trit::from_i8(-3).to_i8(), -1);
    }

    // ---- pow_u64 ----

    #[test]
    fn pow_u64_zero_exp() {
        assert_eq!(pow_u64(7.0, 0), 1.0);
    }

    #[test]
    fn pow_u64_positive_exp() {
        assert!((pow_u64(2.0, 10) - 1024.0).abs() < 1e-9);
    }

    #[test]
    fn pow_u64_negative_exp() {
        assert!((pow_u64(2.0, -3) - 0.125).abs() < 1e-12);
    }

    // ---- Identity witness ----

    #[test]
    fn identity_witness_value() {
        let v = identity_witness();
        assert!((v - 3.0).abs() < 1e-9, "phi^2 + 1/phi^2 = {}, expected 3", v);
    }

    // ---- Anchor #8: cross-kernel Trinity identity through quantization ----
    //
    // Routes phi^2 + 1/phi^2 = 3 through the GF16 codec:
    //   1. Encode phi^2 via f32_to_gf16 -> u16
    //   2. Encode 1/phi^2 via f32_to_gf16 -> u16
    //   3. Decode both back via gf16_to_f32
    //   4. Sum and check ~ 3.0 within GF16 tolerance.
    // pow_u64 is also exercised for both exponents.

    #[test]
    fn quantization_phi_identity() {
        let phi: f64 = 1.618_033_988_749_894_8;

        // Exercise pow_u64 path for phi^2 and phi^-2.
        let phi_sq = pow_u64(phi, 2);
        let phi_inv_sq = pow_u64(phi, -2);

        // Pre-codec identity (mathematical truth)
        let pre = phi_sq + phi_inv_sq;
        assert!((pre - 3.0).abs() < 1e-9);

        // Round-trip through GF16 codec
        let enc_a = f32_to_gf16(phi_sq);
        let enc_b = f32_to_gf16(phi_inv_sq);
        let dec_a = gf16_to_f32(enc_a);
        let dec_b = gf16_to_f32(enc_b);

        let post = dec_a + dec_b;
        // GF16 has ~9-bit mantissa: relative error budget ~0.2%. Allow 1% absolute.
        assert!(
            (post - 3.0).abs() < 0.03,
            "phi-identity through GF16 codec: got {}, expected ~3.0",
            post
        );

        // Also exercise quantize_value route
        let q_a = quantize_value(phi_sq, Format::Gf16);
        let q_b = quantize_value(phi_inv_sq, Format::Gf16);
        assert!((q_a + q_b - 3.0).abs() < 0.03);
    }
}

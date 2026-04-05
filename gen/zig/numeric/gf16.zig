// Auto-generated from specs/numeric/gf16.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf16.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");
const math = std.math;

// ============================================================================
// GF16 -- GoldenFloat16: 16-bit phi-structured floating point (PRIMARY FORMAT)
// Bit layout: [S(1) E(6) M(9)] = [15:15][14:9][8:0]
// ============================================================================

pub const GF16 = u16;

pub const SIGN_SHIFT: u8 = 15;
pub const EXP_SHIFT: u8 = 9;
pub const MANT_SHIFT: u8 = 0;

pub const SIGN_MASK: u16 = 0x8000;
pub const EXP_MASK: u16 = 0x7E00;
pub const MANT_MASK: u16 = 0x01FF;

pub const EXP_MAX: u8 = 0x3F;
pub const EXP_MIN: u8 = 0x00;

pub const BIAS: i8 = 31;
pub const SPECIAL_EXP: u8 = 0x3F;
pub const MANT_DIVISOR: u16 = 512;

pub const PHI_BIAS: u16 = 60;

// GF16 special values
pub const GF16_ZERO_POS: u16 = 0x0000;
pub const GF16_ZERO_NEG: u16 = 0x8000;
pub const GF16_INF_POS: u16 = 0x7E00;
pub const GF16_INF_NEG: u16 = 0xFE00;
pub const GF16_NAN: u16 = 0xFE01;

// ============================================================================
// Field Extraction
// ============================================================================

pub fn gf16_extract_sign(gf16: GF16) i8 {
    const bit = (gf16 >> SIGN_SHIFT) & 1;
    return if (bit != 0) -1 else 0;
}

pub fn gf16_extract_exponent(gf16: GF16) i8 {
    return @intCast((gf16 >> EXP_SHIFT) & 0x3F);
}

pub fn gf16_extract_mantissa(gf16: GF16) i16 {
    return @intCast(gf16 & MANT_MASK);
}

pub fn gf16_from_components(sign: i8, exp: i8, mant: i16) GF16 {
    const sign_bit: u16 = if (sign < 0) 1 else 0;
    return (sign_bit << SIGN_SHIFT) |
        (@as(u16, @intCast(@as(u8, @bitCast(exp)))) << EXP_SHIFT) |
        @as(u16, @intCast(@as(u16, @bitCast(@as(i16, mant)))));
}

// ============================================================================
// Classification
// ============================================================================

pub fn gf16_is_zero(gf16: GF16) bool {
    return gf16 == GF16_ZERO_POS or gf16 == GF16_ZERO_NEG;
}

pub fn gf16_is_special(gf16: GF16) bool {
    return gf16_extract_exponent(gf16) == EXP_MAX;
}

pub fn gf16_is_inf(gf16: GF16) bool {
    const exp = gf16_extract_exponent(gf16);
    const mant = gf16_extract_mantissa(gf16);
    return (exp == EXP_MAX) and (mant == 0);
}

pub fn gf16_is_nan(gf16: GF16) bool {
    const exp = gf16_extract_exponent(gf16);
    const mant = gf16_extract_mantissa(gf16);
    return (exp == EXP_MAX) and (mant != 0);
}

pub fn gf16_is_negative(gf16: GF16) bool {
    return (gf16_extract_sign(gf16) < 0) and !gf16_is_zero(gf16);
}

pub fn gf16_is_positive(gf16: GF16) bool {
    return (gf16_extract_sign(gf16) >= 0) and !gf16_is_zero(gf16);
}

pub fn gf16_is_finite(gf16: GF16) bool {
    return !gf16_is_nan(gf16) and !gf16_is_inf(gf16);
}

// ============================================================================
// Encoding / Decoding
// ============================================================================

pub fn gf16_encode_f32(value: f32) GF16 {
    if (value == 0.0) {
        return if (math.signbit(value)) GF16_ZERO_NEG else GF16_ZERO_POS;
    }

    const sign: i8 = if (value < 0.0) -1 else 0;
    const abs_value = @abs(value);

    const f32_bits: u32 = @bitCast(abs_value);
    const f32_exp_raw: u32 = (f32_bits >> 23) & 0xFF;
    const f32_mant: u32 = f32_bits & 0x7FFFFF;

    // Convert exponent bias: f32(127) to GF16(31)
    const f32_exp: i16 = @as(i16, @intCast(f32_exp_raw)) - 127;
    var gf16_exp: i16 = f32_exp + BIAS;

    if (gf16_exp < 0) gf16_exp = 0;
    if (gf16_exp > EXP_MAX) gf16_exp = EXP_MAX;

    // Scale mantissa from 23 bits to 9 bits
    var mant: u16 = @intCast(f32_mant >> 14);

    // Round-to-nearest
    const discarded = f32_mant & 0x3FFF;
    if ((discarded & 0x2000) != 0) {
        mant += 1;
        if (mant > MANT_MASK) {
            mant = 0;
            if (gf16_exp < EXP_MAX) gf16_exp += 1;
        }
    }

    return gf16_from_components(sign, @intCast(@as(u16, @bitCast(gf16_exp))), @intCast(mant));
}

pub fn gf16_decode_to_f32(gf16: GF16) f32 {
    if (gf16_is_zero(gf16)) {
        return if (gf16_extract_sign(gf16) < 0) -0.0 else 0.0;
    }

    if (gf16_is_special(gf16)) {
        const mant = gf16_extract_mantissa(gf16);
        if (mant == 0) {
            return if (gf16_extract_sign(gf16) < 0) -math.inf(f32) else math.inf(f32);
        } else {
            return math.nan(f32);
        }
    }

    const sign = gf16_extract_sign(gf16);
    const exp = gf16_extract_exponent(gf16);
    const mant = gf16_extract_mantissa(gf16);

    const sign_mult: f32 = if (sign < 0) -1.0 else 1.0;
    const mant_mult: f32 = 1.0 + @as(f32, @floatFromInt(mant)) / 512.0;
    const exp_val: i16 = @as(i16, exp) - BIAS;
    const exp_mult: f32 = math.pow(f32, 2.0, @as(f32, @floatFromInt(exp_val)));

    return sign_mult * mant_mult * exp_mult;
}

// ============================================================================
// Unary Operations
// ============================================================================

pub fn gf16_negate(gf16: GF16) GF16 {
    return gf16 ^ SIGN_MASK;
}

pub fn gf16_abs(gf16: GF16) GF16 {
    return gf16 & ~SIGN_MASK;
}

// ============================================================================
// Arithmetic Operations
// ============================================================================

pub fn gf16_add(a: GF16, b: GF16) GF16 {
    if (gf16_is_nan(a) or gf16_is_nan(b)) return GF16_NAN;
    if (gf16_is_inf(a) and gf16_is_inf(b)) {
        const a_sign = gf16_extract_sign(a);
        const b_sign = gf16_extract_sign(b);
        return if (a_sign == b_sign) a else GF16_NAN;
    }
    if (gf16_is_inf(a)) return a;
    if (gf16_is_inf(b)) return b;

    const a_val = gf16_decode_to_f32(a);
    const b_val = gf16_decode_to_f32(b);
    return gf16_encode_f32(a_val + b_val);
}

pub fn gf16_sub(a: GF16, b: GF16) GF16 {
    if (gf16_is_nan(a) or gf16_is_nan(b)) return GF16_NAN;
    if (gf16_is_inf(a) and gf16_is_inf(b)) return GF16_NAN;
    if (gf16_is_inf(a)) return a;
    if (gf16_is_inf(b)) return gf16_negate(b);

    const a_val = gf16_decode_to_f32(a);
    const b_val = gf16_decode_to_f32(b);
    return gf16_encode_f32(a_val - b_val);
}

pub fn gf16_mul(a: GF16, b: GF16) GF16 {
    if (gf16_is_nan(a) or gf16_is_nan(b)) return GF16_NAN;
    if (gf16_is_zero(a) or gf16_is_zero(b)) {
        const a_sign = gf16_extract_sign(a);
        const b_sign = gf16_extract_sign(b);
        const result_sign = a_sign ^ b_sign;
        return if (result_sign != 0) GF16_ZERO_NEG else GF16_ZERO_POS;
    }
    if (gf16_is_inf(a) or gf16_is_inf(b)) {
        const a_sign = gf16_extract_sign(a);
        const b_sign = gf16_extract_sign(b);
        const result_sign = a_sign ^ b_sign;
        return if (result_sign != 0) GF16_INF_NEG else GF16_INF_POS;
    }

    const a_val = gf16_decode_to_f32(a);
    const b_val = gf16_decode_to_f32(b);
    return gf16_encode_f32(a_val * b_val);
}

pub fn gf16_div(a: GF16, b: GF16) GF16 {
    if (gf16_is_nan(a) or gf16_is_nan(b)) return GF16_NAN;
    if (gf16_is_zero(b)) {
        const a_sign = gf16_extract_sign(a);
        return if (a_sign != 0) GF16_INF_NEG else GF16_INF_POS;
    }
    if (gf16_is_inf(a)) {
        const a_sign = gf16_extract_sign(a);
        const b_sign = gf16_extract_sign(b);
        const result_sign = a_sign ^ b_sign;
        return if (result_sign != 0) GF16_INF_NEG else GF16_INF_POS;
    }
    if (gf16_is_inf(b)) {
        const a_sign = gf16_extract_sign(a);
        const b_sign = gf16_extract_sign(b);
        const result_sign = a_sign ^ b_sign;
        return if (result_sign != 0) GF16_ZERO_NEG else GF16_ZERO_POS;
    }

    const a_val = gf16_decode_to_f32(a);
    const b_val = gf16_decode_to_f32(b);
    return gf16_encode_f32(a_val / b_val);
}

pub fn gf16_fma(a: GF16, b: GF16, c: GF16) GF16 {
    if (gf16_is_nan(a) or gf16_is_nan(b) or gf16_is_nan(c)) return GF16_NAN;
    if (gf16_is_zero(a) or gf16_is_zero(b)) return gf16_add(c, gf16_encode_f32(0.0));

    const a_val = gf16_decode_to_f32(a);
    const b_val = gf16_decode_to_f32(b);
    const c_val = gf16_decode_to_f32(c);
    return gf16_encode_f32(a_val * b_val + c_val);
}

pub fn gf16_sqrt(a: GF16) GF16 {
    if (gf16_is_nan(a)) return GF16_NAN;
    if (gf16_is_inf(a) and !gf16_is_negative(a)) return a;
    if (gf16_is_inf(a)) return GF16_NAN;
    if (gf16_is_zero(a)) return a;
    if (gf16_is_negative(a)) return GF16_NAN;

    const a_val = gf16_decode_to_f32(a);
    return gf16_encode_f32(@sqrt(a_val));
}

// ============================================================================
// Comparison Operations
// ============================================================================

pub fn gf16_eq(a: GF16, b: GF16) bool {
    if (gf16_is_nan(a) or gf16_is_nan(b)) return false;
    if (gf16_is_zero(a) and gf16_is_zero(b)) return true;
    return a == b;
}

pub fn gf16_lt(a: GF16, b: GF16) bool {
    if (gf16_is_nan(a) or gf16_is_nan(b)) return false;
    return gf16_decode_to_f32(a) < gf16_decode_to_f32(b);
}

pub fn gf16_le(a: GF16, b: GF16) bool {
    if (gf16_is_nan(a) or gf16_is_nan(b)) return false;
    return gf16_decode_to_f32(a) <= gf16_decode_to_f32(b);
}

pub fn gf16_max(a: GF16, b: GF16) GF16 {
    if (gf16_is_nan(a)) return b;
    if (gf16_is_nan(b)) return a;
    return if (gf16_decode_to_f32(a) >= gf16_decode_to_f32(b)) a else b;
}

pub fn gf16_min(a: GF16, b: GF16) GF16 {
    if (gf16_is_nan(a)) return b;
    if (gf16_is_nan(b)) return a;
    return if (gf16_decode_to_f32(a) <= gf16_decode_to_f32(b)) a else b;
}

// ============================================================================
// Rounding Operations
// ============================================================================

pub fn gf16_floor(a: GF16) GF16 {
    if (gf16_is_nan(a)) return GF16_NAN;
    if (gf16_is_inf(a) or gf16_is_zero(a)) return a;
    return gf16_encode_f32(@floor(gf16_decode_to_f32(a)));
}

pub fn gf16_ceil(a: GF16) GF16 {
    if (gf16_is_nan(a)) return GF16_NAN;
    if (gf16_is_inf(a) or gf16_is_zero(a)) return a;
    return gf16_encode_f32(@ceil(gf16_decode_to_f32(a)));
}

pub fn gf16_round(a: GF16) GF16 {
    if (gf16_is_nan(a)) return GF16_NAN;
    if (gf16_is_inf(a) or gf16_is_zero(a)) return a;
    return gf16_encode_f32(@round(gf16_decode_to_f32(a)));
}

// ============================================================================
// Phi-optimized Rounding
// ============================================================================

pub fn gf16_round_phi(value: f32) GF16 {
    if (value == 0.0) {
        return if (math.signbit(value)) GF16_ZERO_NEG else GF16_ZERO_POS;
    }

    const sign: i8 = if (value < 0.0) -1 else 0;
    const abs_value = @abs(value);

    const f32_bits: u32 = @bitCast(abs_value);
    const f32_exp_raw: u32 = (f32_bits >> 23) & 0xFF;
    const f32_mant: u32 = f32_bits & 0x7FFFFF;

    const f32_exp: i16 = @as(i16, @intCast(f32_exp_raw)) - 127;
    var gf16_exp: i16 = f32_exp + BIAS;

    if (gf16_exp < 0) gf16_exp = 0;
    if (gf16_exp > EXP_MAX) gf16_exp = EXP_MAX;

    const normalized_mant: u32 = f32_mant | 0x00800000;
    var mant: u16 = @intCast((normalized_mant >> 15) + PHI_BIAS);

    if (mant > MANT_MASK) {
        mant = 0;
        if (gf16_exp < EXP_MAX) {
            gf16_exp += 1;
        } else {
            gf16_exp = EXP_MAX;
        }
    }

    return gf16_from_components(sign, @intCast(@as(u16, @bitCast(gf16_exp))), @intCast(mant));
}

// ============================================================================
// Format Properties
// ============================================================================

pub fn gf16_max_val() f32 {
    // Max normalized: (1 + 511/512) * 2^(62-31) = ~1.998 * 2^31
    const mant_max: f32 = 1.0 + 511.0 / 512.0;
    const exp_max: f32 = @floatFromInt(@as(i16, EXP_MAX) - 1 - BIAS);
    return mant_max * math.pow(f32, 2.0, exp_max);
}

pub fn gf16_min_positive() f32 {
    // Min subnormal: 1/512 * 2^(1 - 31) = 1/512 * 2^-30
    const mant_min: f32 = 1.0 / 512.0;
    const exp_min: f32 = @floatFromInt(@as(i16, 1) - @as(i16, BIAS));
    return mant_min * math.pow(f32, 2.0, exp_min);
}

pub fn gf16_epsilon() f32 {
    return 1.0 / 512.0;
}

pub fn gf16_validate_format() bool {
    return (SIGN_MASK == 0x8000) and (EXP_MASK == 0x7E00) and (MANT_MASK == 0x01FF);
}

// ============================================================================
// Tests
// ============================================================================

test "gf16_zero_pos_encoding" {
    try std.testing.expectEqual(@as(u16, 0x0000), GF16_ZERO_POS);
}

test "gf16_zero_neg_encoding" {
    try std.testing.expectEqual(@as(u16, 0x8000), GF16_ZERO_NEG);
}

test "gf16_inf_pos_encoding" {
    try std.testing.expectEqual(@as(u16, 0x7E00), GF16_INF_POS);
}

test "gf16_is_zero_detects_zero" {
    try std.testing.expect(gf16_is_zero(GF16_ZERO_POS));
    try std.testing.expect(gf16_is_zero(GF16_ZERO_NEG));
}

test "gf16_is_zero_rejects_nonzero" {
    try std.testing.expect(!gf16_is_zero(0x0001));
}

test "gf16_is_nan_detects_nan" {
    try std.testing.expect(gf16_is_nan(GF16_NAN));
}

test "gf16_is_inf_detects_inf" {
    try std.testing.expect(gf16_is_inf(GF16_INF_POS));
    try std.testing.expect(gf16_is_inf(GF16_INF_NEG));
}

test "gf16_encode_decode_zero_roundtrip" {
    const encoded = gf16_encode_f32(0.0);
    const decoded = gf16_decode_to_f32(encoded);
    try std.testing.expectEqual(@as(f32, 0.0), decoded);
}

test "gf16_encode_decode_one_roundtrip" {
    const encoded = gf16_encode_f32(1.0);
    const decoded = gf16_decode_to_f32(encoded);
    try std.testing.expect(@abs(decoded - 1.0) < 0.01);
}

test "gf16_negate_flips_sign" {
    const pos = gf16_encode_f32(1.0);
    const neg = gf16_negate(pos);
    try std.testing.expect(gf16_decode_to_f32(neg) < 0.0);
}

test "gf16_add_basic" {
    const a = gf16_encode_f32(1.0);
    const b = gf16_encode_f32(2.0);
    const result = gf16_decode_to_f32(gf16_add(a, b));
    try std.testing.expect(@abs(result - 3.0) < 0.1);
}

test "gf16_mul_basic" {
    const a = gf16_encode_f32(2.0);
    const b = gf16_encode_f32(3.0);
    const result = gf16_decode_to_f32(gf16_mul(a, b));
    try std.testing.expect(@abs(result - 6.0) < 0.1);
}

test "gf16_div_basic" {
    const a = gf16_encode_f32(6.0);
    const b = gf16_encode_f32(2.0);
    const result = gf16_decode_to_f32(gf16_div(a, b));
    try std.testing.expect(@abs(result - 3.0) < 0.1);
}

test "gf16_eq_basic" {
    const a = gf16_encode_f32(1.0);
    const b = gf16_encode_f32(1.0);
    try std.testing.expect(gf16_eq(a, b));
}

test "gf16_validate_format_success" {
    try std.testing.expect(gf16_validate_format());
}

test "gf16_masks_cover_all_bits" {
    try std.testing.expectEqual(@as(u16, 0xFFFF), SIGN_MASK | EXP_MASK | MANT_MASK);
}

test "gf16_bias_is_31" {
    try std.testing.expectEqual(@as(i8, 31), BIAS);
}

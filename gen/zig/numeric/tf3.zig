// Auto-generated from specs/numeric/tf3.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/tf3.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");
const math = std.math;

// ============================================================================
// TF3 -- Ternary Float 3: 8-bit ternary neural network weight format
// Bit layout: [S(1) E(3) M(4)] = [7:7][6:4][3:0]
// ============================================================================

pub const TF3 = u8;

pub const SIGN_SHIFT: u8 = 7;
pub const EXP_SHIFT: u8 = 4;
pub const MANT_SHIFT: u8 = 0;

pub const SIGN_MASK: u8 = 0x80;
pub const EXP_MASK: u8 = 0x70;
pub const MANT_MASK: u8 = 0x0F;

pub const EXP_MAX: u8 = 0x07;
pub const EXP_MIN: u8 = 0x00;

pub const BIAS: i8 = 3;
pub const MANT_BITS: u8 = 4;

// TF3 special values
pub const TF3_ZERO_POS: u8 = 0x00;
pub const TF3_ZERO_NEG: u8 = 0x80;
pub const TF3_INF_POS: u8 = 0x70;
pub const TF3_INF_NEG: u8 = 0xF0;

// ============================================================================
// Field Extraction
// ============================================================================

pub fn tf3_extract_sign(tf3: TF3) i8 {
    return @intCast((tf3 & SIGN_MASK) >> SIGN_SHIFT);
}

pub fn tf3_extract_exponent(tf3: TF3) i8 {
    return @intCast((tf3 & EXP_MASK) >> EXP_SHIFT);
}

pub fn tf3_extract_mantissa(tf3: TF3) i8 {
    return @intCast(tf3 & MANT_MASK);
}

pub fn tf3_from_components(sign: i8, exp: i8, mant: i8) TF3 {
    return (@as(TF3, @intCast(@as(u8, @bitCast(sign)))) << SIGN_SHIFT) |
        (@as(TF3, @intCast(@as(u8, @bitCast(exp)))) << EXP_SHIFT) |
        (@as(TF3, @intCast(@as(u8, @bitCast(mant)))) << MANT_SHIFT);
}

// ============================================================================
// Classification
// ============================================================================

pub fn tf3_is_zero(tf3: TF3) bool {
    return tf3 == TF3_ZERO_POS or tf3 == TF3_ZERO_NEG;
}

pub fn tf3_is_inf(tf3: TF3) bool {
    const exp = tf3_extract_exponent(tf3);
    const mant = tf3_extract_mantissa(tf3);
    return exp == EXP_MAX and mant == 0;
}

pub fn tf3_is_negative(tf3: TF3) bool {
    return (tf3_extract_sign(tf3) != 0) and !tf3_is_zero(tf3);
}

pub fn tf3_is_positive(tf3: TF3) bool {
    return (tf3_extract_sign(tf3) == 0) and !tf3_is_zero(tf3);
}

// ============================================================================
// Encoding / Decoding
// ============================================================================

pub fn tf3_from_f32(value: f32) TF3 {
    if (value == 0.0) {
        return if (math.signbit(value)) TF3_ZERO_NEG else TF3_ZERO_POS;
    }

    const sign: i8 = if (value < 0.0) 1 else 0;
    const abs_value = @abs(value);
    const clamped = @min(abs_value, 8.0);

    var exp: i8 = 0;
    var scaled = clamped;
    while (scaled >= 2.0 and exp < 7) {
        scaled /= 2.0;
        exp += 1;
    }

    // Mantissa: 4 bits
    const mantissa_raw = (scaled - 1.0) * 16.0;
    const mantissa_clamped = @max(@as(f32, 0.0), @min(@as(f32, 15.0), mantissa_raw));
    const mant: i8 = @intFromFloat(@round(mantissa_clamped));

    return tf3_from_components(sign, exp + BIAS, mant);
}

pub fn tf3_to_f32(tf3: TF3) f32 {
    if (tf3_is_zero(tf3)) {
        return if (tf3_extract_sign(tf3) != 0) -0.0 else 0.0;
    }
    if (tf3_is_inf(tf3)) {
        return if (tf3_extract_sign(tf3) != 0) -math.inf(f32) else math.inf(f32);
    }

    const sign = tf3_extract_sign(tf3);
    const exp = tf3_extract_exponent(tf3);
    const mant = tf3_extract_mantissa(tf3);

    const sign_mult: f32 = if (sign != 0) -1.0 else 1.0;
    const mant_mult: f32 = 1.0 + @as(f32, @floatFromInt(mant)) / 16.0;
    const exp_val: i8 = exp - BIAS;
    const exp_mult: f32 = math.pow(f32, 2.0, @as(f32, @floatFromInt(exp_val)));

    return sign_mult * mant_mult * exp_mult;
}

// ============================================================================
// Unary Operations
// ============================================================================

pub fn tf3_negate(tf3: TF3) TF3 {
    return tf3 ^ SIGN_MASK;
}

pub fn tf3_abs(tf3: TF3) TF3 {
    return tf3 & ~SIGN_MASK;
}

// ============================================================================
// Arithmetic Operations
// ============================================================================

pub fn tf3_add(a: TF3, b: TF3) TF3 {
    if (tf3_is_inf(a)) return a;
    if (tf3_is_inf(b)) return b;
    return tf3_from_f32(tf3_to_f32(a) + tf3_to_f32(b));
}

pub fn tf3_sub(a: TF3, b: TF3) TF3 {
    if (tf3_is_inf(a)) return a;
    if (tf3_is_inf(b)) return if (tf3_is_negative(b)) TF3_INF_POS else TF3_INF_NEG;
    return tf3_from_f32(tf3_to_f32(a) - tf3_to_f32(b));
}

pub fn tf3_mul(a: TF3, b: TF3) TF3 {
    if (tf3_is_zero(a) or tf3_is_zero(b)) {
        const rs = tf3_extract_sign(a) ^ tf3_extract_sign(b);
        return if (rs != 0) TF3_ZERO_NEG else TF3_ZERO_POS;
    }
    if (tf3_is_inf(a) or tf3_is_inf(b)) {
        const rs = tf3_extract_sign(a) ^ tf3_extract_sign(b);
        return if (rs != 0) TF3_INF_NEG else TF3_INF_POS;
    }
    return tf3_from_f32(tf3_to_f32(a) * tf3_to_f32(b));
}

pub fn tf3_div(a: TF3, b: TF3) TF3 {
    if (tf3_is_zero(b)) {
        return if (tf3_extract_sign(a) != 0) TF3_INF_NEG else TF3_INF_POS;
    }
    if (tf3_is_inf(a)) {
        const rs = tf3_extract_sign(a) ^ tf3_extract_sign(b);
        return if (rs != 0) TF3_INF_NEG else TF3_INF_POS;
    }
    if (tf3_is_inf(b)) {
        const rs = tf3_extract_sign(a) ^ tf3_extract_sign(b);
        return if (rs != 0) TF3_ZERO_NEG else TF3_ZERO_POS;
    }
    return tf3_from_f32(tf3_to_f32(a) / tf3_to_f32(b));
}

// ============================================================================
// Comparison
// ============================================================================

pub fn tf3_eq(a: TF3, b: TF3) bool {
    if (tf3_is_zero(a) and tf3_is_zero(b)) return true;
    return a == b;
}

pub fn tf3_lt(a: TF3, b: TF3) bool {
    return tf3_to_f32(a) < tf3_to_f32(b);
}

pub fn tf3_max(a: TF3, b: TF3) TF3 {
    return if (tf3_to_f32(a) >= tf3_to_f32(b)) a else b;
}

pub fn tf3_min(a: TF3, b: TF3) TF3 {
    return if (tf3_to_f32(a) <= tf3_to_f32(b)) a else b;
}

// ============================================================================
// Format Properties
// ============================================================================

pub fn tf3_validate_format() bool {
    return (SIGN_MASK | EXP_MASK | MANT_MASK) == 0xFF;
}

// ============================================================================
// Tests
// ============================================================================

test "tf3_is_zero_detects_zero" {
    try std.testing.expect(tf3_is_zero(TF3_ZERO_POS));
}

test "tf3_is_zero_detects_negative_zero" {
    try std.testing.expect(tf3_is_zero(TF3_ZERO_NEG));
}

test "tf3_is_zero_rejects_nonzero" {
    try std.testing.expect(!tf3_is_zero(0x01));
}

test "tf3_inf_positive_encoding" {
    const exp = tf3_extract_exponent(TF3_INF_POS);
    const mant = tf3_extract_mantissa(TF3_INF_POS);
    try std.testing.expectEqual(@as(i8, EXP_MAX), exp);
    try std.testing.expectEqual(@as(i8, 0), mant);
}

test "tf3_bits_masks_cover_all" {
    try std.testing.expectEqual(@as(u8, 0xFF), SIGN_MASK | EXP_MASK | MANT_MASK);
}

test "tf3_exp_bias_correct" {
    try std.testing.expectEqual(@as(i8, 3), BIAS);
}

test "tf3_mant_bits_correct" {
    try std.testing.expectEqual(@as(u8, 4), MANT_BITS);
}

test "tf3_zero_roundtrip" {
    const encoded = tf3_from_f32(0.0);
    const decoded = tf3_to_f32(encoded);
    try std.testing.expect(@abs(decoded) < 0.001);
}

test "tf3_positive_value_roundtrip" {
    const encoded = tf3_from_f32(1.5);
    const decoded = tf3_to_f32(encoded);
    try std.testing.expect(@abs(decoded - 1.5) < 0.5);
}

test "tf3_negative_value_roundtrip" {
    const encoded = tf3_from_f32(-2.5);
    const decoded = tf3_to_f32(encoded);
    try std.testing.expect(decoded < 0.0);
}

test "tf3_extract_sign_positive" {
    try std.testing.expectEqual(@as(i8, 0), tf3_extract_sign(0x00));
}

test "tf3_extract_sign_negative" {
    try std.testing.expectEqual(@as(i8, 1), tf3_extract_sign(0x80));
}

test "tf3_negate_flips_sign" {
    const pos = tf3_from_f32(1.5);
    const neg = tf3_negate(pos);
    try std.testing.expect(tf3_to_f32(neg) < 0.0);
}

test "tf3_abs_clears_sign" {
    const neg = tf3_from_f32(-1.5);
    const abs_val = tf3_abs(neg);
    try std.testing.expect(tf3_to_f32(abs_val) > 0.0);
}

test "tf3_clamps_to_max" {
    const encoded = tf3_from_f32(100.0);
    try std.testing.expect(tf3_to_f32(encoded) < 20.0);
}

test "tf3_validate_format_success" {
    try std.testing.expect(tf3_validate_format());
}

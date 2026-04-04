// Auto-generated from specs/numeric/gf8.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf8.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");
const math = std.math;

// ============================================================================
// GF8 -- GoldenFloat8: 8-bit phi-structured floating point
// Bit layout: [S|EEE|MMMM]  S:1 E:3 M:4
// ============================================================================

pub const BITS: u8 = 8;
pub const SIGN_BITS: u8 = 1;
pub const EXP_BITS: u8 = 3;
pub const MANT_BITS: u8 = 4;
pub const EXP_BIAS: u8 = 3;
pub const PHI_DISTANCE: f64 = 0.1319660112501052;
pub const MEMORY_RATIO_VS_FP32: f32 = 8.0 / 32.0;

pub const GF8 = struct {
    raw: u8,

    pub fn init(raw: u8) GF8 {
        return .{ .raw = raw };
    }
};

// ============================================================================
// Helper Functions
// ============================================================================

pub fn floor_log2(x_in: f32) i8 {
    if (x_in <= 0.0) return -128;
    var x = x_in;
    var e: i8 = 0;
    while (x >= 2.0) {
        x = x / 2.0;
        e += 1;
    }
    while (x < 1.0) {
        x = x * 2.0;
        e -= 1;
    }
    return e;
}

fn extract_mantissa(value: f32, exp: i8, mant_bits_count: u8) u8 {
    const exp_f: f32 = @floatFromInt(exp);
    const scale = math.pow(f32, 2.0, exp_f);
    const normalized = value / scale;
    const frac = normalized - 1.0;
    const max_mant: u8 = (@as(u8, 1) << @intCast(mant_bits_count)) - 1;
    const val: f32 = frac * (@as(f32, @floatFromInt(max_mant)) + 1.0);
    if (val < 0.0) return 0;
    if (val > @as(f32, @floatFromInt(max_mant))) return max_mant;
    return @intFromFloat(val);
}

fn clamp_u8(x: u8, min_val: u8, max_val: u8) u8 {
    if (x < min_val) return min_val;
    if (x > max_val) return max_val;
    return x;
}

pub fn pow_f32(base: f32, exp: f32) f32 {
    return math.pow(f32, base, exp);
}

pub fn ln_approx(x: f32) f32 {
    if (x <= 0.0) return math.nan(f32);
    if (x == 1.0) return 0.0;
    return @log(x);
}

pub fn exp_approx(x: f32) f32 {
    if (x == 0.0) return 1.0;
    return @exp(x);
}

pub fn floor_f32(x: f32) f32 {
    return @floor(x);
}

// ============================================================================
// Encoding / Decoding
// ============================================================================

pub fn encode(value: f32) GF8 {
    if (value == 0.0) return GF8.init(0);

    const sign: u8 = if (value < 0.0) 1 else 0;
    const abs_val = @abs(value);

    const exp_unbiased = floor_log2(abs_val);
    const biased_raw: i16 = @as(i16, exp_unbiased) + @as(i16, EXP_BIAS);
    const exp_biased: u8 = if (biased_raw < 0) 0 else if (biased_raw > 7) 7 else @intCast(@as(u16, @bitCast(@as(i16, @truncate(biased_raw)))));

    const mant = extract_mantissa(abs_val, exp_unbiased, MANT_BITS);

    return GF8.init((sign << 7) | (exp_biased << MANT_BITS) | mant);
}

pub fn decode(gf: GF8) f32 {
    const sign: u8 = gf.raw >> 7;
    const exp_biased: u8 = (gf.raw >> MANT_BITS) & 0x07;
    const mant: u8 = gf.raw & 0x0F;

    if (exp_biased == 0 and mant == 0) return 0.0;

    const exp_unbiased: i8 = if (exp_biased == 0)
        -@as(i8, @intCast(EXP_BIAS)) + 1
    else
        @as(i8, @intCast(exp_biased)) - @as(i8, @intCast(EXP_BIAS));

    const mant_normalized: f32 = if (exp_biased == 0)
        @as(f32, @floatFromInt(mant)) / 16.0
    else
        1.0 + @as(f32, @floatFromInt(mant)) / 16.0;

    const exp_f: f32 = @floatFromInt(exp_unbiased);
    const val = mant_normalized * math.pow(f32, 2.0, exp_f);

    return if (sign != 0) -val else val;
}

// ============================================================================
// Format Properties
// ============================================================================

pub fn max_value() f32 {
    const mant_max: f32 = 1.0 + 15.0 / 16.0;
    const exp_max: i8 = 7 - @as(i8, @intCast(EXP_BIAS));
    const exp_f: f32 = @floatFromInt(exp_max);
    return mant_max * math.pow(f32, 2.0, exp_f);
}

pub fn min_positive() f32 {
    const mant_min: f32 = 1.0 / 16.0;
    const exp_min: i8 = -@as(i8, @intCast(EXP_BIAS)) + 1;
    const exp_f: f32 = @floatFromInt(exp_min);
    return mant_min * math.pow(f32, 2.0, exp_f);
}

pub fn epsilon() f32 {
    return 1.0 / 16.0;
}

pub fn validate_format() bool {
    return (BITS == 8) and (SIGN_BITS == 1) and (EXP_BITS == 3) and (MANT_BITS == 4);
}

// ============================================================================
// Tests
// ============================================================================

test "gf8_decode_zero" {
    const gf = GF8.init(0);
    try std.testing.expectEqual(@as(f32, 0.0), decode(gf));
}

test "gf8_encode_zero_roundtrip" {
    const encoded = encode(0.0);
    const decoded = decode(encoded);
    try std.testing.expectEqual(@as(f32, 0.0), decoded);
}

test "gf8_decode_positive_value" {
    const gf = GF8.init(0b01000000);
    try std.testing.expect(decode(gf) > 0.0);
}

test "gf8_decode_negative_value" {
    const gf = GF8.init(0b10110000);
    try std.testing.expect(decode(gf) < 0.0);
}

test "gf8_bits_sum_correct" {
    try std.testing.expectEqual(BITS, SIGN_BITS + EXP_BITS + MANT_BITS);
}

test "gf8_max_value_positive" {
    try std.testing.expect(max_value() > 0.0);
}

test "gf8_min_positive_greater_than_zero" {
    try std.testing.expect(min_positive() > 0.0);
}

test "gf8_epsilon_positive" {
    try std.testing.expect(epsilon() > 0.0);
}

test "gf8_memory_ratio_vs_fp32" {
    try std.testing.expectEqual(@as(f32, 0.25), MEMORY_RATIO_VS_FP32);
}

test "gf8_validate_format_success" {
    try std.testing.expect(validate_format());
}

test "gf8_pow_zero_exponent_returns_one" {
    try std.testing.expect(@abs(pow_f32(2.0, 0.0) - 1.0) < 1e-6);
}

test "gf8_ln_approx_of_one" {
    try std.testing.expect(@abs(ln_approx(1.0)) < 1e-6);
}

test "gf8_exp_approx_zero" {
    try std.testing.expect(@abs(exp_approx(0.0) - 1.0) < 1e-6);
}

test "gf8_floor_positive" {
    try std.testing.expect(@abs(floor_f32(3.7) - 3.0) < 1e-6);
}

test "gf8_floor_negative" {
    try std.testing.expect(@abs(floor_f32(-3.2) - (-4.0)) < 1e-6);
}

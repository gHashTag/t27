// Auto-generated from specs/numeric/gf20.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf20.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");
const math = std.math;

// =====================================================================
// GoldenFloat20 -- 20-bit phi-structured floating point
// Format: [S:1 | E:7 | M:12] -- phi_distance = 0.03463
// 38% memory savings vs FP32
// =====================================================================

pub const BITS: u8 = 20;
pub const SIGN_BITS: u8 = 1;
pub const EXP_BITS: u8 = 7;
pub const MANT_BITS: u8 = 12;
pub const EXP_BIAS: u8 = 63;
pub const PHI_DISTANCE: f64 = 0.03463264154356299;
pub const MEMORY_RATIO_VS_FP32: f32 = 20.0 / 32.0; // 0.625

const EXP_MASK: u32 = (1 << EXP_BITS) - 1; // 0x7F
const MANT_MASK: u32 = (1 << MANT_BITS) - 1; // 0xFFF
const MANT_SCALE: f32 = 4096.0; // 2^12

// =====================================================================
// GF20 Type
// =====================================================================

pub const GF20 = struct {
    raw: u32, // 20-bit value stored in u32

    pub fn init(raw: u32) GF20 {
        return GF20{ .raw = raw & 0xFFFFF }; // mask to 20 bits
    }
};

// =====================================================================
// Encoding / Decoding
// =====================================================================

pub fn encode(value: f32) GF20 {
    if (value == 0.0) {
        return GF20{ .raw = 0 };
    }

    const sign: u32 = if (value < 0.0) 1 else 0;
    const abs_val: f32 = if (value < 0.0) -value else value;

    // Extract exponent (unbiased)
    const exp_unbiased = floorLog2(abs_val);
    const exp_biased_i: i32 = @as(i32, exp_unbiased) + @as(i32, EXP_BIAS);

    // Clamp exponent
    const exp_clamped: u32 = @intCast(clamp(exp_biased_i, 0, @as(i32, EXP_MASK)));

    // Extract mantissa (12 bits)
    const mant = extractMantissa(abs_val, exp_unbiased, MANT_BITS);

    return GF20{
        .raw = (sign << 19) |
            (exp_clamped << MANT_BITS) |
            mant,
    };
}

pub fn decode(gf: GF20) f32 {
    const sign: u8 = @intCast((gf.raw >> 19) & 1);
    const exp_biased: u8 = @intCast((gf.raw >> MANT_BITS) & EXP_MASK);
    const mant: u32 = gf.raw & MANT_MASK;

    // Zero
    if (exp_biased == 0 and mant == 0) {
        return 0.0;
    }

    // Exponent
    const exp_unbiased: i16 = if (exp_biased == 0)
        -@as(i16, EXP_BIAS) + 1
    else
        @as(i16, exp_biased) - @as(i16, EXP_BIAS);

    // Mantissa
    const mant_f: f32 = @floatFromInt(mant);
    const mant_normalized: f32 = if (exp_biased == 0)
        mant_f / MANT_SCALE
    else
        1.0 + mant_f / MANT_SCALE;

    const result = mant_normalized * pow2f(exp_unbiased);

    if (sign != 0) {
        return -result;
    }
    return result;
}

// =====================================================================
// Format Properties
// =====================================================================

pub fn maxValue() f32 {
    const mant_max: f32 = 1.0 + 4095.0 / MANT_SCALE;
    const exp_max: i16 = @as(i16, EXP_MASK) - @as(i16, EXP_BIAS);
    return mant_max * pow2f(exp_max);
}

pub fn minPositive() f32 {
    const mant_min: f32 = 1.0 / MANT_SCALE;
    const exp_min: i16 = -@as(i16, EXP_BIAS) + 1;
    return mant_min * pow2f(exp_min);
}

pub fn epsilon() f32 {
    return 1.0 / MANT_SCALE; // 0.000244140625
}

pub fn validateFormat() bool {
    return (BITS == SIGN_BITS + EXP_BITS + MANT_BITS) and
        (SIGN_BITS == 1) and
        (EXP_BITS == 7) and
        (MANT_BITS == 12);
}

// =====================================================================
// Helper Functions
// =====================================================================

pub fn floorLog2(x: f32) i16 {
    if (x <= 0.0) return -32768;
    var val = x;
    var exp_val: i16 = 0;
    while (val >= 2.0) {
        val = val / 2.0;
        exp_val += 1;
    }
    while (val < 1.0) {
        val = val * 2.0;
        exp_val -= 1;
    }
    return exp_val;
}

fn extractMantissa(value: f32, exp_val: i16, mant_bits: u8) u32 {
    const normalized = value / pow2f(exp_val);
    const frac = normalized - 1.0;
    const max_mant: u32 = (@as(u32, 1) << @intCast(mant_bits)) - 1;
    const max_mant_f: f32 = @floatFromInt(max_mant);
    return @intFromFloat(frac * (max_mant_f + 1.0));
}

fn clamp(x: i32, min_val: i32, max_val: i32) i32 {
    if (x < min_val) return min_val;
    if (x > max_val) return max_val;
    return x;
}

fn pow2f(exp_val: i16) f32 {
    return math.pow(f32, 2.0, @floatFromInt(exp_val));
}

pub fn powf(base: f32, exp_val: f32) f32 {
    if (exp_val == 0.0) return 1.0;
    if (base == 0.0 and exp_val > 0.0) return 0.0;
    if (base <= 0.0) return math.nan(f32);

    const is_integer = (exp_val == @floor(exp_val));
    if (is_integer) {
        var e: i32 = @intFromFloat(exp_val);
        var result: f32 = 1.0;
        var base_acc: f32 = base;
        if (e < 0) {
            e = -e;
            base_acc = 1.0 / base_acc;
        }
        while (e > 0) {
            if (@rem(e, 2) == 1) {
                result *= base_acc;
            }
            base_acc *= base_acc;
            e = @divTrunc(e, 2);
        }
        return result;
    }

    const ln_val = lnApprox(base);
    return expApprox(exp_val * ln_val);
}

pub fn lnApprox(x: f32) f32 {
    if (x <= 0.0) return math.nan(f32);
    if (x == 1.0) return 0.0;

    const t = (x - 1.0) / (x + 1.0);
    const t2 = t * t;
    const t3 = t2 * t;
    const t5 = t3 * t2;
    const t7 = t5 * t2;

    return 2.0 * (t + t3 / 3.0 + t5 / 5.0 + t7 / 7.0);
}

pub fn expApprox(x: f32) f32 {
    if (x == 0.0) return 1.0;

    var exp_x = x;
    var k: i32 = 0;

    if (exp_x > 5.0 or exp_x < -5.0) {
        k = @intFromFloat(@floor(exp_x / 5.0));
        const k_f: f32 = @floatFromInt(k);
        exp_x = exp_x - k_f * 5.0;
    }

    var result: f32 = 1.0;
    var term: f32 = 1.0;
    for (1..9) |i| {
        const i_f: f32 = @floatFromInt(i);
        term = term * exp_x / i_f;
        result += term;
    }

    if (k > 0) {
        const e5 = expApprox(5.0);
        var j: i32 = 0;
        while (j < k) : (j += 1) {
            result *= e5;
        }
    } else if (k < 0) {
        const e5 = expApprox(5.0);
        var j: i32 = k;
        while (j < 0) : (j += 1) {
            result /= e5;
        }
    }

    return result;
}

pub fn floorf(x: f32) f32 {
    return @floor(x);
}

// =====================================================================
// Tests
// =====================================================================

test "gf20_decode_zero" {
    const gf = GF20{ .raw = 0 };
    const value = decode(gf);
    try std.testing.expectEqual(@as(f32, 0.0), value);
}

test "gf20_encode_zero_roundtrip" {
    const original: f32 = 0.0;
    const encoded = encode(original);
    const decoded = decode(encoded);
    try std.testing.expectEqual(original, decoded);
}

test "gf20_bits_sum_correct" {
    const total: u8 = SIGN_BITS + EXP_BITS + MANT_BITS;
    try std.testing.expectEqual(@as(u8, BITS), total);
}

test "gf20_max_value_positive" {
    const max_val = maxValue();
    try std.testing.expect(max_val > 0.0);
}

test "gf20_min_positive_greater_than_zero" {
    const min_pos = minPositive();
    try std.testing.expect(min_pos > 0.0);
}

test "gf20_epsilon_positive" {
    const eps = epsilon();
    try std.testing.expect(eps > 0.0);
}

test "gf20_phi_distance_within_tolerance" {
    try std.testing.expect(PHI_DISTANCE < 0.04);
}

test "gf20_memory_ratio_vs_fp32" {
    try std.testing.expect(@abs(MEMORY_RATIO_VS_FP32 - 0.625) < 0.01);
}

test "gf20_validate_format_success" {
    try std.testing.expect(validateFormat());
}

test "gf20_pow_zero_exponent_returns_one" {
    const result = powf(2.0, 0.0);
    try std.testing.expect(@abs(result - 1.0) < 1e-6);
}

test "gf20_pow_one_exponent_returns_base" {
    const result = powf(5.0, 1.0);
    try std.testing.expect(@abs(result - 5.0) < 1e-6);
}

test "gf20_pow_positive_integer_exponent" {
    const result = powf(2.0, 5.0);
    try std.testing.expect(@abs(result - 32.0) < 1e-5);
}

test "gf20_pow_negative_integer_exponent" {
    const result = powf(2.0, -3.0);
    try std.testing.expect(@abs(result - 0.125) < 1e-5);
}

test "gf20_pow_fractional_exponent" {
    const result = powf(4.0, 0.5);
    try std.testing.expect(@abs(result - 2.0) < 1e-4);
}

test "gf20_pow_zero_base_positive_exponent" {
    const result = powf(0.0, 5.0);
    try std.testing.expectEqual(@as(f32, 0.0), result);
}

test "gf20_pow_one_base_any_exponent" {
    const r1 = powf(1.0, 10.0);
    const r2 = powf(1.0, -5.0);
    try std.testing.expect(@abs(r1 - 1.0) < 1e-6);
    try std.testing.expect(@abs(r2 - 1.0) < 1e-6);
}

test "gf20_ln_approx_of_one" {
    const result = lnApprox(1.0);
    try std.testing.expect(@abs(result) < 1e-6);
}

test "gf20_ln_approx_of_e" {
    const e: f32 = 2.718281828459045;
    const result = lnApprox(e);
    try std.testing.expect(@abs(result - 1.0) < 0.01);
}

test "gf20_ln_approx_negative_returns_nan" {
    const result = lnApprox(-1.0);
    try std.testing.expect(math.isNan(result));
}

test "gf20_exp_approx_zero" {
    const result = expApprox(0.0);
    try std.testing.expect(@abs(result - 1.0) < 1e-6);
}

test "gf20_exp_approx_one" {
    const e: f32 = 2.718281828459045;
    const result = expApprox(1.0);
    try std.testing.expect(@abs(result - e) < 0.01);
}

test "gf20_exp_approx_negative" {
    const result = expApprox(-1.0);
    const expected: f32 = 1.0 / 2.718281828459045;
    try std.testing.expect(@abs(result - expected) < 0.01);
}

test "gf20_floor_positive" {
    const result = floorf(3.7);
    try std.testing.expect(@abs(result - 3.0) < 1e-6);
}

test "gf20_floor_negative" {
    const result = floorf(-3.2);
    try std.testing.expect(@abs(result - (-4.0)) < 1e-6);
}

test "gf20_floor_integer" {
    const result = floorf(5.0);
    try std.testing.expect(@abs(result - 5.0) < 1e-6);
}

// Invariant tests
test "gf20_bits_constant" {
    try std.testing.expectEqual(@as(u8, 20), BITS);
}

test "gf20_sign_bits_is_one" {
    try std.testing.expectEqual(@as(u8, 1), SIGN_BITS);
}

test "gf20_exp_bits_is_seven" {
    try std.testing.expectEqual(@as(u8, 7), EXP_BITS);
}

test "gf20_mant_bits_is_twelve" {
    try std.testing.expectEqual(@as(u8, 12), MANT_BITS);
}

test "gf20_max_ge_min_positive" {
    try std.testing.expect(maxValue() >= minPositive());
}

test "gf20_phi_distance_below_threshold" {
    try std.testing.expect(PHI_DISTANCE < 0.04);
}

test "gf20_exp_bias_positive" {
    try std.testing.expect(EXP_BIAS > 0);
}

test "gf20_ln_exp_inversion" {
    const x: f32 = 2.0;
    const y = lnApprox(x);
    try std.testing.expect(@abs(expApprox(y) - x) < 0.01);
}

test "gf20_floor_monotonic" {
    try std.testing.expect(floorf(2.5) <= floorf(3.5));
}

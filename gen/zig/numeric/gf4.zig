// Auto-generated from specs/numeric/gf4.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/gf4.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");

// ============================================================================
// GF4 -- GoldenFloat4: 4-bit phi-structured floating point
// Bit layout: [S|E|MM]  S:1 E:1 M:2
// ============================================================================

pub const BITS: u8 = 4;
pub const SIGN_BITS: u8 = 1;
pub const EXP_BITS: u8 = 1;
pub const MANT_BITS: u8 = 2;
pub const EXP_BIAS: u8 = 0;
pub const PHI_DISTANCE: f64 = 0.1180339887498949;
pub const MEMORY_RATIO_VS_FP32: f32 = 4.0 / 32.0;

pub const GF4 = struct {
    raw: u4,

    pub fn init(raw: u4) GF4 {
        return .{ .raw = raw };
    }
};

// ============================================================================
// Encoding / Decoding
// ============================================================================

pub fn encode(value: f32) GF4 {
    if (value == 0.0) {
        return GF4.init(0b0000);
    }
    if (value < 0.0) {
        const pos = encode(-value).raw;
        return GF4.init(pos | 0b1000);
    }
    // Quantize to available positive values: 0.25, 0.5, 0.75, 1.0, 1.5
    if (value <= 0.375) {
        return GF4.init(0b0001); // 0.25
    } else if (value <= 0.625) {
        return GF4.init(0b0010); // 0.5
    } else if (value <= 0.875) {
        return GF4.init(0b0011); // 0.75
    } else if (value <= 1.25) {
        return GF4.init(0b0101); // 1.0
    } else {
        return GF4.init(0b0111); // 1.5
    }
}

pub fn decode(gf: GF4) f32 {
    const raw = gf.raw;
    if (raw == 0) return 0.0;

    const sign_bit: bool = (raw & 0b1000) != 0;
    const exp_bit: bool = (raw & 0b0100) != 0;
    const mant_bits: u4 = raw & 0b0011;

    const mant: f32 = @as(f32, @floatFromInt(mant_bits)) / 4.0;
    const exp_scale: f32 = if (exp_bit) 2.0 else 1.0;
    const val = mant * exp_scale;

    return if (sign_bit) -val else val;
}

// ============================================================================
// Format Properties
// ============================================================================

pub fn max_value() f32 {
    return 1.5;
}

pub fn min_positive() f32 {
    return 0.25;
}

pub fn epsilon() f32 {
    return 0.25;
}

pub fn validate_format() bool {
    return (BITS == 4) and (SIGN_BITS == 1) and (EXP_BITS == 1) and (MANT_BITS == 2);
}

// ============================================================================
// Tests
// ============================================================================

test "gf4_decode_zero" {
    const gf = GF4.init(0b0000);
    const value = decode(gf);
    try std.testing.expectEqual(@as(f32, 0.0), value);
}

test "gf4_decode_positive_max" {
    const gf = GF4.init(0b0111);
    const value = decode(gf);
    try std.testing.expectEqual(@as(f32, 1.5), value);
}

test "gf4_decode_negative" {
    const gf = GF4.init(0b1001);
    const value = decode(gf);
    try std.testing.expect(value < 0.0);
}

test "gf4_encode_zero_roundtrip" {
    const original: f32 = 0.0;
    const encoded = encode(original);
    const decoded = decode(encoded);
    try std.testing.expectEqual(original, decoded);
}

test "gf4_encode_0_25" {
    const encoded = encode(0.25);
    const decoded = decode(encoded);
    try std.testing.expect(@abs(decoded - 0.25) < 0.01);
}

test "gf4_encode_0_5" {
    const encoded = encode(0.5);
    const decoded = decode(encoded);
    try std.testing.expect(@abs(decoded - 0.5) < 0.01);
}

test "gf4_encode_0_75" {
    const encoded = encode(0.75);
    const decoded = decode(encoded);
    try std.testing.expect(@abs(decoded - 0.75) < 0.01);
}

test "gf4_encode_1_0" {
    const encoded = encode(1.0);
    const decoded = decode(encoded);
    try std.testing.expect(@abs(decoded - 1.0) < 0.01);
}

test "gf4_encode_1_5" {
    const encoded = encode(1.5);
    const decoded = decode(encoded);
    try std.testing.expect(@abs(decoded - 1.5) < 0.01);
}

test "gf4_encode_negative_values" {
    const encoded = encode(-0.5);
    const decoded = decode(encoded);
    try std.testing.expect(decoded < 0.0);
    try std.testing.expect(@abs(decoded - (-0.5)) < 0.01);
}

test "gf4_encode_clamps_to_max" {
    const encoded = encode(10.0);
    const decoded = decode(encoded);
    try std.testing.expect(decoded <= 1.5);
}

test "gf4_encode_quantization_small" {
    const encoded = encode(0.3);
    const decoded = decode(encoded);
    try std.testing.expect(@abs(decoded - 0.25) < 0.01);
}

test "gf4_max_value_is_1_5" {
    try std.testing.expectEqual(@as(f32, 1.5), max_value());
}

test "gf4_min_positive_is_0_25" {
    try std.testing.expectEqual(@as(f32, 0.25), min_positive());
}

test "gf4_bits_sum_correct" {
    const total = SIGN_BITS + EXP_BITS + MANT_BITS;
    try std.testing.expectEqual(BITS, total);
}

test "gf4_memory_ratio_vs_fp32" {
    try std.testing.expectEqual(@as(f32, 0.125), MEMORY_RATIO_VS_FP32);
}

test "gf4_validate_format_success" {
    try std.testing.expect(validate_format());
}

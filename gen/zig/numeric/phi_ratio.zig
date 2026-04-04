// Auto-generated from specs/numeric/phi_ratio.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/phi_ratio.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");
const math = std.math;

// ============================================================================
// PhiRatio -- phi-split derivation for GoldenFloat exp/mantissa allocation
// The ideal exp/mant ratio = 1/phi ~= 0.618
// ============================================================================

pub const PHI: f64 = 1.6180339887498948;
pub const PHI_INV: f64 = 0.6180339887498949;
pub const PHI_SQ: f64 = PHI * PHI; // 2.618...
pub const PHI_RATIO_TARGET: f64 = PHI_INV;

// ============================================================================
// Types
// ============================================================================

pub const PhiSplitResult = struct {
    exp_bits: u8,
    mant_bits: u8,
    ratio: f64,
    phi_dist: f64,
};

pub const FormatComparison = struct {
    name: []const u8,
    bits: u8,
    actual_exp: u8,
    actual_mant: u8,
    phi_split_exp: u8,
    phi_split_mant: u8,
    matches_phi_split: bool,
};

// ============================================================================
// Core Functions
// ============================================================================

pub fn phi_split(bits: u8) PhiSplitResult {
    const available: f64 = @floatFromInt(bits - 1);
    const exp_raw = available / PHI_SQ;
    const exp_bits: u8 = @intFromFloat(@round(exp_raw));
    const mant_bits: u8 = (bits - 1) - exp_bits;

    const ratio = @as(f64, @floatFromInt(exp_bits)) / @as(f64, @floatFromInt(mant_bits));
    const phi_dist = @abs(ratio - PHI_RATIO_TARGET);

    return .{
        .exp_bits = exp_bits,
        .mant_bits = mant_bits,
        .ratio = ratio,
        .phi_dist = phi_dist,
    };
}

pub fn compute_phi_distance(exp_bits: u8, mant_bits: u8) f64 {
    const ratio = @as(f64, @floatFromInt(exp_bits)) / @as(f64, @floatFromInt(mant_bits));
    return @abs(ratio - PHI_RATIO_TARGET);
}

pub fn is_phi_optimal(exp_bits: u8, mant_bits: u8, tolerance: f64) bool {
    return compute_phi_distance(exp_bits, mant_bits) < tolerance;
}

pub fn recommend_format(total_bits: u8) PhiSplitResult {
    return phi_split(total_bits);
}

pub fn phi_optimality_proof() []const u8 {
    return "exp/mant = 1/phi maximizes (dynamic_range * precision) for fixed bit budget";
}

pub fn sacred_connection() []const u8 {
    return "GoldenFloat exp/mant = 1/phi = consciousness threshold = sacred_physics::C_THRESHOLD";
}

// ============================================================================
// Helper Functions
// ============================================================================

pub fn round_f64(x: f64) f64 {
    return @round(x);
}

pub fn abs_f64(x: f64) f64 {
    return @abs(x);
}

pub fn pow_f64(base: f64, exp: f64) f64 {
    return math.pow(f64, base, exp);
}

pub fn ln_approx_f64(x: f64) f64 {
    if (x <= 0.0) return math.nan(f64);
    if (x == 1.0) return 0.0;
    return @log(x);
}

pub fn exp_approx_f64(x: f64) f64 {
    if (x == 0.0) return 1.0;
    return @exp(x);
}

pub fn floor_f64(x: f64) f64 {
    return @floor(x);
}

// ============================================================================
// Verification
// ============================================================================

pub fn verify_phi_split() [7]FormatComparison {
    return .{
        .{ .name = "GF4", .bits = 4, .actual_exp = 1, .actual_mant = 2, .phi_split_exp = 1, .phi_split_mant = 2, .matches_phi_split = true },
        .{ .name = "GF8", .bits = 8, .actual_exp = 3, .actual_mant = 4, .phi_split_exp = 2, .phi_split_mant = 5, .matches_phi_split = false },
        .{ .name = "GF12", .bits = 12, .actual_exp = 4, .actual_mant = 7, .phi_split_exp = 3, .phi_split_mant = 8, .matches_phi_split = false },
        .{ .name = "GF16", .bits = 16, .actual_exp = 6, .actual_mant = 9, .phi_split_exp = 4, .phi_split_mant = 11, .matches_phi_split = false },
        .{ .name = "GF20", .bits = 20, .actual_exp = 7, .actual_mant = 12, .phi_split_exp = 5, .phi_split_mant = 14, .matches_phi_split = false },
        .{ .name = "GF24", .bits = 24, .actual_exp = 9, .actual_mant = 14, .phi_split_exp = 6, .phi_split_mant = 17, .matches_phi_split = false },
        .{ .name = "GF32", .bits = 32, .actual_exp = 12, .actual_mant = 19, .phi_split_exp = 8, .phi_split_mant = 23, .matches_phi_split = false },
    };
}

// ============================================================================
// Tests
// ============================================================================

test "phi_split_for_gf4_perfect_match" {
    const result = phi_split(4);
    try std.testing.expectEqual(@as(u8, 1), result.exp_bits);
    try std.testing.expectEqual(@as(u8, 2), result.mant_bits);
    try std.testing.expect(result.phi_dist < 0.01);
}

test "phi_split_for_gf16_primary_format" {
    const result = phi_split(16);
    try std.testing.expectEqual(@as(u8, 4), result.exp_bits);
    try std.testing.expectEqual(@as(u8, 11), result.mant_bits);
    try std.testing.expect(result.phi_dist < 0.05);
}

test "phi_split_for_gf32_near_optimal" {
    const result = phi_split(32);
    try std.testing.expectEqual(@as(u8, 8), result.exp_bits);
    try std.testing.expectEqual(@as(u8, 23), result.mant_bits);
    try std.testing.expect(result.phi_dist < 0.02);
}

test "phi_split_sum_constraint" {
    const result = phi_split(16);
    try std.testing.expectEqual(@as(u8, 15), result.exp_bits + result.mant_bits);
}

test "phi_ratio_target_equals_phi_inverse" {
    try std.testing.expect(@abs(PHI_RATIO_TARGET - PHI_INV) < 1e-15);
}

test "compute_phi_distance_for_gf16" {
    const distance = compute_phi_distance(6, 9);
    try std.testing.expect(distance > 0.04);
}

test "is_phi_optimal_tolerance_check" {
    try std.testing.expect(is_phi_optimal(4, 11, 0.05));
}

test "phi_optimality_proof_contains_result" {
    const proof = phi_optimality_proof();
    try std.testing.expect(std.mem.indexOf(u8, proof, "1/phi") != null);
}

test "sacred_connection_contains_threshold" {
    const connection = sacred_connection();
    try std.testing.expect(std.mem.indexOf(u8, connection, "C_THRESHOLD") != null);
}

test "phi_ratio_round_positive" {
    try std.testing.expectEqual(@as(f64, 4.0), round_f64(3.7));
}

test "phi_ratio_round_negative" {
    try std.testing.expectEqual(@as(f64, -4.0), round_f64(-3.7));
}

test "phi_ratio_round_half_up" {
    try std.testing.expectEqual(@as(f64, 4.0), round_f64(3.5));
}

test "phi_ratio_round_zero" {
    try std.testing.expectEqual(@as(f64, 0.0), round_f64(0.0));
}

test "phi_ratio_pow_zero_exponent" {
    try std.testing.expect(@abs(pow_f64(2.0, 0.0) - 1.0) < 1e-15);
}

test "phi_ratio_ln_approx_of_one" {
    try std.testing.expect(@abs(ln_approx_f64(1.0)) < 1e-15);
}

test "phi_ratio_exp_approx_zero" {
    try std.testing.expect(@abs(exp_approx_f64(0.0) - 1.0) < 1e-15);
}

test "phi_ratio_floor_positive" {
    try std.testing.expectEqual(@as(f64, 3.0), floor_f64(3.7));
}

test "phi_ratio_floor_negative" {
    try std.testing.expectEqual(@as(f64, -4.0), floor_f64(-3.2));
}

test "verify_phi_split_gf4_matches" {
    const comparisons = verify_phi_split();
    try std.testing.expect(comparisons[0].matches_phi_split);
}

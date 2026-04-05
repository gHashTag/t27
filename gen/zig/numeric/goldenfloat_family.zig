// Auto-generated from specs/numeric/goldenfloat_family.t27
// DO NOT EDIT -- regenerate with: tri gen specs/numeric/goldenfloat_family.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");
const math = std.math;

// ============================================================================
// GoldenFloatFamily -- phi-structured floating point format registry
// Contains all 7 GF formats: GF4, GF8, GF12, GF16, GF20, GF24, GF32
// ============================================================================

pub const PHI_RATIO_TARGET: f64 = 0.6180339887498949;

pub const GoldenFloatFormat = struct {
    name: []const u8,
    bits: u8,
    sign_bits: u8,
    exp_bits: u8,
    mant_bits: u8,
    exp_mant_ratio: f64,
    phi_distance: f64,
    is_primary: bool,
};

pub const FAMILY_SIZE: usize = 7;

pub const GOLDEN_FLOAT_FAMILY: [FAMILY_SIZE]GoldenFloatFormat = .{
    .{ .name = "GF4", .bits = 4, .sign_bits = 1, .exp_bits = 1, .mant_bits = 2, .exp_mant_ratio = 0.5, .phi_distance = @abs(0.5 - PHI_RATIO_TARGET), .is_primary = false },
    .{ .name = "GF8", .bits = 8, .sign_bits = 1, .exp_bits = 3, .mant_bits = 4, .exp_mant_ratio = 0.75, .phi_distance = @abs(0.75 - PHI_RATIO_TARGET), .is_primary = false },
    .{ .name = "GF12", .bits = 12, .sign_bits = 1, .exp_bits = 4, .mant_bits = 7, .exp_mant_ratio = 0.5714285714285714, .phi_distance = @abs(0.5714285714285714 - PHI_RATIO_TARGET), .is_primary = false },
    .{ .name = "GF16", .bits = 16, .sign_bits = 1, .exp_bits = 6, .mant_bits = 9, .exp_mant_ratio = 0.6666666666666667, .phi_distance = @abs(0.6666666666666667 - PHI_RATIO_TARGET), .is_primary = true },
    .{ .name = "GF20", .bits = 20, .sign_bits = 1, .exp_bits = 7, .mant_bits = 12, .exp_mant_ratio = 0.5833333333333333, .phi_distance = @abs(0.5833333333333333 - PHI_RATIO_TARGET), .is_primary = false },
    .{ .name = "GF24", .bits = 24, .sign_bits = 1, .exp_bits = 9, .mant_bits = 14, .exp_mant_ratio = 0.6428571428571429, .phi_distance = @abs(0.6428571428571429 - PHI_RATIO_TARGET), .is_primary = false },
    .{ .name = "GF32", .bits = 32, .sign_bits = 1, .exp_bits = 12, .mant_bits = 19, .exp_mant_ratio = 0.631578947368421, .phi_distance = @abs(0.631578947368421 - PHI_RATIO_TARGET), .is_primary = false },
};

// ============================================================================
// Query Functions
// ============================================================================

pub fn get_format_by_name(name: []const u8) ?GoldenFloatFormat {
    for (GOLDEN_FLOAT_FAMILY) |fmt| {
        if (std.mem.eql(u8, fmt.name, name)) return fmt;
    }
    return null;
}

pub fn get_format_by_bits(bits: u8) ?GoldenFloatFormat {
    for (GOLDEN_FLOAT_FAMILY) |fmt| {
        if (fmt.bits == bits) return fmt;
    }
    return null;
}

pub fn get_primary_format() GoldenFloatFormat {
    return GOLDEN_FLOAT_FAMILY[3]; // GF16
}

// ============================================================================
// Utility Functions
// ============================================================================

pub fn max_value(format: GoldenFloatFormat) f64 {
    const mant_max = 2.0 - math.pow(f64, 2.0, -@as(f64, @floatFromInt(format.mant_bits)));
    const exp_max = math.pow(f64, 2.0, @as(f64, @floatFromInt(format.exp_bits))) - 1.0;
    return mant_max * math.pow(f64, 2.0, exp_max);
}

pub fn min_positive(format: GoldenFloatFormat) f64 {
    const mant_min = math.pow(f64, 2.0, -@as(f64, @floatFromInt(format.mant_bits)));
    const bias = math.pow(f64, 2.0, @as(f64, @floatFromInt(format.exp_bits)) - 1.0) - 1.0;
    return mant_min * math.pow(f64, 2.0, 1.0 - bias);
}

pub fn memory_efficiency(format: GoldenFloatFormat) f64 {
    return @as(f64, @floatFromInt(format.bits)) / 32.0;
}

// ============================================================================
// Verification
// ============================================================================

pub const VerificationReport = struct {
    all_valid: bool,
    primary_is_gf16: bool,
    phi_distances_ok: bool,
    best_phi_format: []const u8,
    best_phi_distance: f64,
    avg_phi_distance: f64,
};

pub fn verify_golden_family() VerificationReport {
    var primary_count: u8 = 0;
    var best_dist: f64 = 1.0;
    var best_name: []const u8 = "";
    var total_dist: f64 = 0.0;
    var all_bit_sums_valid = true;

    for (GOLDEN_FLOAT_FAMILY) |fmt| {
        if (fmt.is_primary) primary_count += 1;
        if (fmt.phi_distance < best_dist) {
            best_dist = fmt.phi_distance;
            best_name = fmt.name;
        }
        total_dist += fmt.phi_distance;
        if (fmt.sign_bits + fmt.exp_bits + fmt.mant_bits != fmt.bits) {
            all_bit_sums_valid = false;
        }
    }

    const avg_dist = total_dist / 7.0;

    return .{
        .all_valid = all_bit_sums_valid and (primary_count == 1),
        .primary_is_gf16 = (primary_count == 1) and GOLDEN_FLOAT_FAMILY[3].is_primary,
        .phi_distances_ok = best_dist < 0.1,
        .best_phi_format = best_name,
        .best_phi_distance = best_dist,
        .avg_phi_distance = avg_dist,
    };
}

// ============================================================================
// Tests
// ============================================================================

test "gffamily_get_format_by_name_gf16" {
    const fmt = get_format_by_name("GF16");
    try std.testing.expect(fmt != null);
    try std.testing.expectEqual(@as(u8, 16), fmt.?.bits);
}

test "gffamily_get_format_by_bits_8" {
    const fmt = get_format_by_bits(8);
    try std.testing.expect(fmt != null);
    try std.testing.expect(std.mem.eql(u8, "GF8", fmt.?.name));
}

test "gffamily_get_primary_format_is_gf16" {
    const primary = get_primary_format();
    try std.testing.expect(primary.is_primary);
    try std.testing.expect(std.mem.eql(u8, "GF16", primary.name));
}

test "gffamily_family_size_7" {
    try std.testing.expectEqual(@as(usize, 7), GOLDEN_FLOAT_FAMILY.len);
}

test "gffamily_phi_ratio_target_is_phi_inverse" {
    try std.testing.expect(@abs(PHI_RATIO_TARGET - 0.6180339887498949) < 0.000001);
}

test "gffamily_gf4_has_correct_bit_counts" {
    const fmt = get_format_by_name("GF4");
    try std.testing.expect(fmt != null);
    try std.testing.expectEqual(@as(u8, 1), fmt.?.sign_bits);
    try std.testing.expectEqual(@as(u8, 1), fmt.?.exp_bits);
    try std.testing.expectEqual(@as(u8, 2), fmt.?.mant_bits);
}

test "gffamily_gf32_has_correct_bit_counts" {
    const fmt = get_format_by_name("GF32");
    try std.testing.expect(fmt != null);
    try std.testing.expectEqual(@as(u8, 12), fmt.?.exp_bits);
    try std.testing.expectEqual(@as(u8, 19), fmt.?.mant_bits);
}

test "gffamily_only_gf16_is_primary" {
    var count: u32 = 0;
    for (GOLDEN_FLOAT_FAMILY) |fmt| {
        if (fmt.is_primary) count += 1;
    }
    try std.testing.expectEqual(@as(u32, 1), count);
}

test "gffamily_verify_primary_is_gf16" {
    const report = verify_golden_family();
    try std.testing.expect(report.primary_is_gf16);
}

test "gffamily_phi_distances_within_tolerance" {
    const report = verify_golden_family();
    try std.testing.expect(report.phi_distances_ok);
}

test "gffamily_best_phi_format_is_gf12" {
    const report = verify_golden_family();
    try std.testing.expect(std.mem.eql(u8, "GF12", report.best_phi_format));
}

test "gffamily_memory_efficiency_gf8" {
    const fmt = get_format_by_name("GF8").?;
    const eff = memory_efficiency(fmt);
    try std.testing.expect(@abs(eff - 0.25) < 0.01);
}

test "gffamily_memory_efficiency_gf16" {
    const fmt = get_format_by_name("GF16").?;
    const eff = memory_efficiency(fmt);
    try std.testing.expect(@abs(eff - 0.5) < 0.01);
}

test "gffamily_get_format_by_unknown_name" {
    try std.testing.expect(get_format_by_name("GF999") == null);
}

test "gffamily_get_format_by_unknown_bits" {
    try std.testing.expect(get_format_by_bits(100) == null);
}

test "gffamily_verify_all_valid" {
    const report = verify_golden_family();
    try std.testing.expect(report.all_valid);
}

test "gffamily_best_phi_distance_is_small" {
    const report = verify_golden_family();
    try std.testing.expect(report.best_phi_distance < 0.05);
}

test "gffamily_avg_phi_distance_reasonable" {
    const report = verify_golden_family();
    try std.testing.expect(report.avg_phi_distance > 0.0);
    try std.testing.expect(report.avg_phi_distance < 0.2);
}

test "gffamily_all_formats_sign_bits_1" {
    for (GOLDEN_FLOAT_FAMILY) |fmt| {
        try std.testing.expectEqual(@as(u8, 1), fmt.sign_bits);
    }
}

test "gffamily_all_formats_bits_sum_correct" {
    for (GOLDEN_FLOAT_FAMILY) |fmt| {
        try std.testing.expectEqual(fmt.bits, fmt.sign_bits + fmt.exp_bits + fmt.mant_bits);
    }
}

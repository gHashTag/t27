// Auto-generated from specs/base/ops.t27
// DO NOT EDIT -- regenerate with: tri gen specs/base/ops.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");

// ============================================================================
// Constants
// ============================================================================

pub const NEGONE: i8 = -1;
pub const ZERO: i8 = 0;
pub const ONE: i8 = 1;

pub const Trit = enum(i8) {
    neg = -1,
    zero = 0,
    pos = 1,
};

// ============================================================================
// Lookup Tables
// ============================================================================
// Indexed as: table[(a+1)*3 + (b+1)] where a,b in {-1,0,+1}

/// Multiplication lookup: mult_table[(a+1)*3 + (b+1)]
pub const mult_table = [9]i8{ 1, 0, -1, 0, 0, 0, -1, 0, 1 };

/// Addition lookup: add_table[(a+1)*3 + (b+1)]
pub const add_table = [9]i8{ -1, -1, 0, -1, 0, 1, 0, 1, 1 };

/// Carry lookup: carry_table[(a+1)*3 + (b+1)]
pub const carry_table = [9]i8{ 1, 0, 0, 0, 0, 0, 0, 0, -1 };

// ============================================================================
// Types
// ============================================================================

pub const AddResult = struct {
    result: Trit,
    carry_out: Trit,
};

// ============================================================================
// Functions
// ============================================================================

/// trit_multiply_table(a, b) -> Trit: Fast multiplication via lookup
pub fn trit_multiply_table(a: Trit, b: Trit) Trit {
    const idx: usize = @intCast(@as(i16, @intFromEnum(a) + 1) * 3 + (@intFromEnum(b) + 1));
    return @as(Trit, @enumFromInt(mult_table[idx]));
}

/// trit_add_table(a, b) -> Trit: Fast addition via lookup
pub fn trit_add_table(a: Trit, b: Trit) Trit {
    const idx: usize = @intCast(@as(i16, @intFromEnum(a) + 1) * 3 + (@intFromEnum(b) + 1));
    return @as(Trit, @enumFromInt(add_table[idx]));
}

/// trit_carry_table(a, b) -> Trit: Carry computation via lookup
pub fn trit_carry_table(a: Trit, b: Trit) Trit {
    const idx: usize = @intCast(@as(i16, @intFromEnum(a) + 1) * 3 + (@intFromEnum(b) + 1));
    return @as(Trit, @enumFromInt(carry_table[idx]));
}

/// trit_add_with_carry(a, b, carry_in) -> AddResult: Full ternary addition
pub fn trit_add_with_carry(a: Trit, b: Trit, carry_in: Trit) AddResult {
    var sum: i8 = @intFromEnum(a) + @intFromEnum(b);
    var carry: Trit = .zero;
    var result: Trit = .zero;

    if (sum > 1) {
        result = .neg;
        carry = .pos;
    } else if (sum < -1) {
        result = .pos;
        carry = .neg;
    } else {
        result = @as(Trit, @enumFromInt(sum));
    }

    sum = @intFromEnum(result) + @intFromEnum(carry_in);
    if (sum > 1) {
        result = .neg;
        carry = .pos;
    } else if (sum < -1) {
        result = .pos;
        carry = .neg;
    } else {
        result = @as(Trit, @enumFromInt(sum));
        carry = .zero;
    }

    return AddResult{ .result = result, .carry_out = carry };
}

/// trit_compare(a, b) -> i8: Returns -1 if a<b, 0 if a==b, +1 if a>b
pub fn trit_compare(a: Trit, b: Trit) i8 {
    if (a == b) {
        return 0;
    } else if (a == .neg or (a == .zero and b == .pos)) {
        return -1;
    } else {
        return 1;
    }
}

/// trit_negate(a) -> Trit: Trit negation
pub fn trit_negate(a: Trit) Trit {
    return switch (a) {
        .neg => .pos,
        .zero => .zero,
        .pos => .neg,
    };
}

/// trit_abs(a) -> Trit: Absolute value
pub fn trit_abs(a: Trit) Trit {
    return if (a == .neg) .pos else a;
}

/// trit_min(a, b) -> Trit: Minimum of two trits
pub fn trit_min(a: Trit, b: Trit) Trit {
    return if (a == .neg or (a == .zero and b == .pos)) a else b;
}

/// trit_max(a, b) -> Trit: Maximum of two trits
pub fn trit_max(a: Trit, b: Trit) Trit {
    return if (a == .pos or (a == .zero and b == .neg)) a else b;
}

/// trit_subtract(a, b) -> Trit: a - b using a + (-b)
pub fn trit_subtract(a: Trit, b: Trit) Trit {
    return trit_add_table(a, trit_negate(b));
}

/// trit_sign(a) -> i8: Sign of a trit
pub fn trit_sign(a: Trit) i8 {
    return @intFromEnum(a);
}

/// trit_clamp(a, min_val, max_val) -> Trit: Clamp to range
pub fn trit_clamp(a: Trit, min_val: Trit, max_val: Trit) Trit {
    if (trit_compare(a, min_val) < 0) return min_val;
    if (trit_compare(a, max_val) > 0) return max_val;
    return a;
}

/// trit_is_negative(a) -> bool: Returns true if a is -1
pub fn trit_is_negative(a: Trit) bool {
    return a == .neg;
}

/// trit_is_zero(a) -> bool: Returns true if a is 0
pub fn trit_is_zero(a: Trit) bool {
    return a == .zero;
}

/// trit_is_positive(a) -> bool: Returns true if a is +1
pub fn trit_is_positive(a: Trit) bool {
    return a == .pos;
}

/// trit_equal(a, b) -> bool: Returns true if a == b
pub fn trit_equal(a: Trit, b: Trit) bool {
    return a == b;
}

/// trit_not_equal(a, b) -> bool: Returns true if a != b
pub fn trit_not_equal(a: Trit, b: Trit) bool {
    return a != b;
}

/// trit_lt(a, b) -> bool: Less than
pub fn trit_lt(a: Trit, b: Trit) bool {
    return trit_compare(a, b) < 0;
}

/// trit_le(a, b) -> bool: Less than or equal
pub fn trit_le(a: Trit, b: Trit) bool {
    return trit_compare(a, b) <= 0;
}

/// trit_gt(a, b) -> bool: Greater than
pub fn trit_gt(a: Trit, b: Trit) bool {
    return trit_compare(a, b) > 0;
}

/// trit_ge(a, b) -> bool: Greater than or equal
pub fn trit_ge(a: Trit, b: Trit) bool {
    return trit_compare(a, b) >= 0;
}

/// trit_multiply_with_carry(a, b, carry_in) -> AddResult
pub fn trit_multiply_with_carry(a: Trit, b: Trit, carry_in: Trit) AddResult {
    const product = trit_multiply_table(a, b);
    return trit_add_with_carry(product, carry_in, .zero);
}

/// trit_reverse(a) -> Trit: Multiplicative inverse in balanced ternary
pub fn trit_reverse(a: Trit) Trit {
    return if (a == .zero) .zero else a;
}

/// trit_multiply_by_power_of_two(a, power) -> Trit
pub fn trit_multiply_by_power_of_two(a: Trit, power: u8) Trit {
    var result = a;
    var i: u8 = 1;
    while (i < power) : (i += 1) {
        const carry = trit_carry_table(result, a);
        if (carry != .zero) {
            result = if (carry == .pos) .pos else .neg;
        }
        result = trit_add_table(result, a);
    }
    return result;
}

/// trit_power(a, n) -> Trit: Raise trit to power n
pub fn trit_power(a: Trit, n: u8) Trit {
    if (n == 0) return .pos;
    if (n == 1) return a;
    if (a == .zero) return .zero;
    if (a == .pos) return .pos;
    return if (n % 2 == 1) .neg else .pos;
}

/// trit_from_bool(b) -> Trit: Convert boolean to trit
pub fn trit_from_bool(b: bool) Trit {
    return if (b) .pos else .zero;
}

/// trit_to_bool(a) -> bool: Convert trit to boolean
pub fn trit_to_bool(a: Trit) bool {
    return a == .pos;
}

/// trit_abs_diff(a, b) -> Trit: Absolute difference
pub fn trit_abs_diff(a: Trit, b: Trit) Trit {
    return if (a == b) .zero else .pos;
}

/// trit_cond_swap(cond, a, b) -> Trit: Conditional swap
pub fn trit_cond_swap(cond: Trit, a: Trit, b: Trit) Trit {
    return if (cond == .pos) b else a;
}

/// trit_is_unit(a) -> bool: Check if trit is multiplicative unit
pub fn trit_is_unit(a: Trit) bool {
    return a == .pos;
}

/// trit_is_identity(a) -> bool: Check if trit is additive identity
pub fn trit_is_identity(a: Trit) bool {
    return a == .zero;
}

/// trit_is_negated(a, b) -> bool: Check if b is negation of a
pub fn trit_is_negated(a: Trit, b: Trit) bool {
    return b == trit_negate(a);
}

// ============================================================================
// Tests
// ============================================================================

test "test_trit_multiply_table_all_combinations" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_multiply_table(.neg, .neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_multiply_table(.neg, .zero));
    try std.testing.expectEqual(@as(Trit, .neg), trit_multiply_table(.neg, .pos));
    try std.testing.expectEqual(@as(Trit, .zero), trit_multiply_table(.zero, .neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_multiply_table(.zero, .zero));
    try std.testing.expectEqual(@as(Trit, .zero), trit_multiply_table(.zero, .pos));
    try std.testing.expectEqual(@as(Trit, .neg), trit_multiply_table(.pos, .neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_multiply_table(.pos, .zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_multiply_table(.pos, .pos));
}

test "test_trit_multiply_table_commutative" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |a| {
        for (trits) |b| {
            try std.testing.expectEqual(trit_multiply_table(a, b), trit_multiply_table(b, a));
        }
    }
}

test "test_trit_add_table_neg_plus_neg" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_add_table(.neg, .neg));
}

test "test_trit_add_table_neg_plus_zero" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_add_table(.neg, .zero));
}

test "test_trit_add_table_neg_plus_pos" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_add_table(.neg, .pos));
}

test "test_trit_add_table_zero_plus_zero" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_add_table(.zero, .zero));
}

test "test_trit_add_table_zero_plus_pos" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_add_table(.zero, .pos));
}

test "test_trit_add_table_pos_plus_pos" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_add_table(.pos, .pos));
}

test "test_trit_add_table_commutative" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |a| {
        for (trits) |b| {
            try std.testing.expectEqual(trit_add_table(a, b), trit_add_table(b, a));
        }
    }
}

test "test_trit_add_table_identity_zero" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_add_table(.neg, .zero));
    try std.testing.expectEqual(@as(Trit, .zero), trit_add_table(.zero, .zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_add_table(.pos, .zero));
}

test "test_trit_carry_table_neg_plus_neg" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_carry_table(.neg, .neg));
}

test "test_trit_carry_table_neg_plus_zero" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_carry_table(.neg, .zero));
}

test "test_trit_carry_table_neg_plus_pos" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_carry_table(.neg, .pos));
}

test "test_trit_carry_table_zero_plus_zero" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_carry_table(.zero, .zero));
}

test "test_trit_carry_table_zero_plus_pos" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_carry_table(.zero, .pos));
}

test "test_trit_carry_table_pos_plus_pos" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_carry_table(.pos, .pos));
}

test "test_trit_carry_table_commutative" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |a| {
        for (trits) |b| {
            try std.testing.expectEqual(trit_carry_table(a, b), trit_carry_table(b, a));
        }
    }
}

test "test_trit_add_with_carry_no_carry" {
    const result = trit_add_with_carry(.pos, .neg, .zero);
    try std.testing.expectEqual(@as(Trit, .zero), result.result);
    try std.testing.expectEqual(@as(Trit, .zero), result.carry_out);
}

test "test_trit_add_with_carry_positive_overflow" {
    const result = trit_add_with_carry(.pos, .pos, .zero);
    try std.testing.expectEqual(@as(Trit, .neg), result.result);
    try std.testing.expectEqual(@as(Trit, .pos), result.carry_out);
}

test "test_trit_add_with_carry_negative_overflow" {
    const result = trit_add_with_carry(.neg, .neg, .zero);
    try std.testing.expectEqual(@as(Trit, .pos), result.result);
    try std.testing.expectEqual(@as(Trit, .neg), result.carry_out);
}

test "test_trit_add_with_carry_propagation" {
    const result = trit_add_with_carry(.pos, .pos, .pos);
    try std.testing.expectEqual(@as(Trit, .zero), result.result);
    try std.testing.expectEqual(@as(Trit, .pos), result.carry_out);
}

test "test_trit_add_with_carry_result_in_range" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |a| {
        for (trits) |b| {
            for (trits) |c| {
                const result = trit_add_with_carry(a, b, c);
                try std.testing.expect(@intFromEnum(result.result) >= -1 and @intFromEnum(result.result) <= 1);
            }
        }
    }
}

test "test_trit_add_with_carry_carry_in_range" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |a| {
        for (trits) |b| {
            for (trits) |c| {
                const result = trit_add_with_carry(a, b, c);
                try std.testing.expect(@intFromEnum(result.carry_out) >= -1 and @intFromEnum(result.carry_out) <= 1);
            }
        }
    }
}

test "test_trit_compare_less_than" {
    try std.testing.expectEqual(@as(i8, -1), trit_compare(.neg, .zero));
    try std.testing.expectEqual(@as(i8, -1), trit_compare(.neg, .pos));
    try std.testing.expectEqual(@as(i8, -1), trit_compare(.zero, .pos));
}

test "test_trit_compare_equal" {
    try std.testing.expectEqual(@as(i8, 0), trit_compare(.neg, .neg));
    try std.testing.expectEqual(@as(i8, 0), trit_compare(.zero, .zero));
    try std.testing.expectEqual(@as(i8, 0), trit_compare(.pos, .pos));
}

test "test_trit_compare_greater_than" {
    try std.testing.expectEqual(@as(i8, 1), trit_compare(.pos, .zero));
    try std.testing.expectEqual(@as(i8, 1), trit_compare(.pos, .neg));
    try std.testing.expectEqual(@as(i8, 1), trit_compare(.zero, .neg));
}

test "test_trit_compare_total_ordering" {
    try std.testing.expectEqual(@as(i8, -1), trit_compare(.neg, .zero));
    try std.testing.expectEqual(@as(i8, -1), trit_compare(.zero, .pos));
    try std.testing.expectEqual(@as(i8, -1), trit_compare(.neg, .pos));
}

test "test_trit_negate_involutive" {
    try std.testing.expectEqual(Trit.neg, trit_negate(trit_negate(.neg)));
    try std.testing.expectEqual(Trit.zero, trit_negate(trit_negate(.zero)));
    try std.testing.expectEqual(Trit.pos, trit_negate(trit_negate(.pos)));
}

test "test_trit_abs_non_negative" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_abs(.neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_abs(.zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_abs(.pos));
}

test "test_trit_min_returns_minimum" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_min(.neg, .pos));
    try std.testing.expectEqual(@as(Trit, .neg), trit_min(.neg, .zero));
    try std.testing.expectEqual(@as(Trit, .zero), trit_min(.zero, .pos));
    try std.testing.expectEqual(@as(Trit, .neg), trit_min(.pos, .neg));
}

test "test_trit_max_returns_maximum" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_max(.neg, .pos));
    try std.testing.expectEqual(@as(Trit, .zero), trit_max(.neg, .zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_max(.zero, .pos));
    try std.testing.expectEqual(@as(Trit, .pos), trit_max(.pos, .neg));
}

test "test_trit_min_idempotent" {
    try std.testing.expectEqual(Trit.neg, trit_min(.neg, .neg));
    try std.testing.expectEqual(Trit.zero, trit_min(.zero, .zero));
    try std.testing.expectEqual(Trit.pos, trit_min(.pos, .pos));
}

test "test_trit_max_idempotent" {
    try std.testing.expectEqual(Trit.neg, trit_max(.neg, .neg));
    try std.testing.expectEqual(Trit.zero, trit_max(.zero, .zero));
    try std.testing.expectEqual(Trit.pos, trit_max(.pos, .pos));
}

test "test_trit_subtract_neg_from_pos" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_subtract(.pos, .neg));
}

test "test_trit_subtract_pos_from_pos" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_subtract(.pos, .pos));
}

test "test_trit_subtract_neg_from_neg" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_subtract(.neg, .neg));
}

test "test_trit_subtract_all_combinations" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |a| {
        for (trits) |b| {
            try std.testing.expectEqual(trit_subtract(a, b), trit_add_table(a, trit_negate(b)));
        }
    }
}

test "test_trit_sign_negative" {
    try std.testing.expectEqual(@as(i8, -1), trit_sign(.neg));
}

test "test_trit_sign_zero" {
    try std.testing.expectEqual(@as(i8, 0), trit_sign(.zero));
}

test "test_trit_sign_positive" {
    try std.testing.expectEqual(@as(i8, 1), trit_sign(.pos));
}

test "test_trit_sign_matches_enum_value" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |a| {
        try std.testing.expectEqual(@intFromEnum(a), trit_sign(a));
    }
}

test "test_trit_clamp_below_min" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_clamp(.neg, .zero, .pos));
}

test "test_trit_clamp_above_max" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_clamp(.pos, .neg, .zero));
}

test "test_trit_clamp_in_range" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |a| {
        try std.testing.expectEqual(a, trit_clamp(a, .neg, .pos));
    }
}

test "test_trit_is_negative_true_for_neg" {
    try std.testing.expect(trit_is_negative(.neg));
}

test "test_trit_is_negative_false_for_others" {
    try std.testing.expect(!trit_is_negative(.zero));
    try std.testing.expect(!trit_is_negative(.pos));
}

test "test_trit_is_zero_true_for_zero" {
    try std.testing.expect(trit_is_zero(.zero));
}

test "test_trit_is_zero_false_for_others" {
    try std.testing.expect(!trit_is_zero(.neg));
    try std.testing.expect(!trit_is_zero(.pos));
}

test "test_trit_is_positive_true_for_pos" {
    try std.testing.expect(trit_is_positive(.pos));
}

test "test_trit_is_positive_false_for_others" {
    try std.testing.expect(!trit_is_positive(.neg));
    try std.testing.expect(!trit_is_positive(.zero));
}

test "test_trit_equal_same_values" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |a| {
        try std.testing.expect(trit_equal(a, a));
    }
}

test "test_trit_equal_different_values_false" {
    try std.testing.expect(!trit_equal(.neg, .zero));
    try std.testing.expect(!trit_equal(.zero, .pos));
    try std.testing.expect(!trit_equal(.pos, .neg));
}

test "test_trit_power_zero_exponent" {
    try std.testing.expectEqual(Trit.pos, trit_power(.neg, 0));
    try std.testing.expectEqual(Trit.pos, trit_power(.pos, 0));
}

test "test_trit_power_zero_base" {
    try std.testing.expectEqual(Trit.zero, trit_power(.zero, 1));
    try std.testing.expectEqual(Trit.zero, trit_power(.zero, 2));
    try std.testing.expectEqual(Trit.zero, trit_power(.zero, 3));
}

test "test_trit_power_one" {
    try std.testing.expectEqual(Trit.neg, trit_power(.neg, 1));
    try std.testing.expectEqual(Trit.zero, trit_power(.zero, 1));
    try std.testing.expectEqual(Trit.pos, trit_power(.pos, 1));
}

test "test_trit_power_square" {
    try std.testing.expectEqual(Trit.pos, trit_power(.neg, 2));
    try std.testing.expectEqual(Trit.zero, trit_power(.zero, 2));
    try std.testing.expectEqual(Trit.pos, trit_power(.pos, 2));
}

test "test_trit_power_cube" {
    try std.testing.expectEqual(Trit.neg, trit_power(.neg, 3));
    try std.testing.expectEqual(Trit.zero, trit_power(.zero, 3));
    try std.testing.expectEqual(Trit.pos, trit_power(.pos, 3));
}

test "test_trit_from_bool_true" {
    try std.testing.expectEqual(Trit.pos, trit_from_bool(true));
}

test "test_trit_from_bool_false" {
    try std.testing.expectEqual(Trit.zero, trit_from_bool(false));
}

test "test_trit_to_bool_positive" {
    try std.testing.expect(trit_to_bool(.pos));
}

test "test_trit_to_bool_non_positive" {
    try std.testing.expect(!trit_to_bool(.zero));
    try std.testing.expect(!trit_to_bool(.neg));
}

test "test_trit_abs_diff_equal" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |a| {
        try std.testing.expectEqual(Trit.zero, trit_abs_diff(a, a));
    }
}

test "test_trit_abs_diff_different" {
    try std.testing.expectEqual(Trit.pos, trit_abs_diff(.neg, .zero));
    try std.testing.expectEqual(Trit.pos, trit_abs_diff(.neg, .pos));
    try std.testing.expectEqual(Trit.pos, trit_abs_diff(.zero, .pos));
}

test "test_trit_abs_diff_commutative" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |a| {
        for (trits) |b| {
            try std.testing.expectEqual(trit_abs_diff(a, b), trit_abs_diff(b, a));
        }
    }
}

test "test_trit_reverse_non_zero" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_reverse(.pos));
    try std.testing.expectEqual(@as(Trit, .neg), trit_reverse(.neg));
}

test "test_trit_reverse_zero" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_reverse(.zero));
}

test "test_trit_is_negated_true_pair" {
    try std.testing.expect(trit_is_negated(.neg, .pos));
    try std.testing.expect(trit_is_negated(.pos, .neg));
    try std.testing.expect(trit_is_negated(.zero, .zero));
}

test "test_trit_is_negated_false_non_pair" {
    try std.testing.expect(!trit_is_negated(.neg, .neg));
    try std.testing.expect(!trit_is_negated(.pos, .pos));
    try std.testing.expect(!trit_is_negated(.zero, .pos));
}

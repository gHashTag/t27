// Auto-generated from specs/base/types.t27
// DO NOT EDIT -- regenerate with: tri gen specs/base/types.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");

// ============================================================================
// Constants - Trit Values
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
// Constants - PackedTrit
// ============================================================================

pub const PACKED_BITS_PER_TRIT: u8 = 2;
pub const TRITS_PER_BYTE: u8 = 8;

pub const PACKED_NEG: u8 = 2;
pub const PACKED_ZERO: u8 = 0;
pub const PACKED_ONE: u8 = 1;

pub const TRIT_MASK: u8 = 0x03;
pub const PackedTrit = u8;

// ============================================================================
// Constants - TernaryWord
// ============================================================================

pub const TRITS_PER_WORD: u8 = 27;
pub const WORD_BYTES: u8 = 5;

pub const TernaryWord = [WORD_BYTES]u8;

// ============================================================================
// Types
// ============================================================================

pub const UnpackResult = struct {
    value: Trit,
    valid: bool,
};

// ============================================================================
// Functions
// ============================================================================

/// trit_add(a, b) -> Trit: Balanced ternary addition (single trit, no carry)
pub fn trit_add(a: Trit, b: Trit) Trit {
    return switch (a) {
        .neg => switch (b) {
            .neg => .neg,
            .zero => .neg,
            .pos => .zero,
        },
        .zero => b,
        .pos => switch (b) {
            .neg => .zero,
            .zero => .pos,
            .pos => .pos,
        },
    };
}

/// trit_multiply(a, b) -> Trit: Balanced ternary multiplication
pub fn trit_multiply(a: Trit, b: Trit) Trit {
    return switch (a) {
        .neg => switch (b) {
            .neg => .pos,
            .zero => .zero,
            .pos => .neg,
        },
        .zero => .zero,
        .pos => b,
    };
}

/// trit_negate(a) -> Trit: Negate a trit
pub fn trit_negate(a: Trit) Trit {
    return switch (a) {
        .neg => .pos,
        .zero => .zero,
        .pos => .neg,
    };
}

/// trit_to_packed(trit) -> u8: Convert Trit to 2-bit packed representation
pub fn trit_to_packed(trit: Trit) u8 {
    return switch (trit) {
        .neg => PACKED_NEG,
        .zero => PACKED_ZERO,
        .pos => PACKED_ONE,
    };
}

/// packed_to_trit(packed) -> Trit: Convert 2-bit packed representation to Trit
pub fn packed_to_trit(packed: u8) Trit {
    return switch (packed & TRIT_MASK) {
        2 => .neg,
        0 => .zero,
        1 => .pos,
        else => .zero,
    };
}

/// pack_trit(trit, position, packed) -> PackedTrit: Pack a single trit at position (0-7)
pub fn pack_trit(trit: Trit, position: u8, packed: PackedTrit) PackedTrit {
    if (position >= TRITS_PER_BYTE) {
        return 0xFF;
    }

    const encoding = trit_to_packed(trit);
    const bit_pos: u3 = @intCast(position * PACKED_BITS_PER_TRIT);

    const mask: u8 = ~(@as(u8, TRIT_MASK) << bit_pos);
    var result = packed & mask;
    result |= @as(u8, encoding) << bit_pos;

    return result;
}

/// unpack_trit(position, packed) -> UnpackResult: Extract trit at position (0-7)
pub fn unpack_trit(position: u8, packed: PackedTrit) UnpackResult {
    if (position >= TRITS_PER_BYTE) {
        return UnpackResult{ .value = .zero, .valid = false };
    }

    const bit_pos: u3 = @intCast(position * PACKED_BITS_PER_TRIT);
    const encoding = (packed >> bit_pos) & TRIT_MASK;
    const value = packed_to_trit(encoding);

    return UnpackResult{ .value = value, .valid = true };
}

/// ternary_word_pack(src, count) -> TernaryWord: Pack count trits into TernaryWord
pub fn ternary_word_pack(src: []const Trit, count: u8) TernaryWord {
    if (count > TRITS_PER_WORD) {
        return [_]u8{ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF };
    }

    var result = [_]u8{ 0, 0, 0, 0, 0 };

    const n: usize = @min(count, TRITS_PER_WORD);
    for (0..n) |i| {
        const byte_idx = i / TRITS_PER_BYTE;
        const trit_pos: u8 = @intCast(i % TRITS_PER_BYTE);
        result[byte_idx] = pack_trit(src[i], trit_pos, result[byte_idx]);
    }

    return result;
}

/// ternary_word_unpack(word, count) -> array of Trit values
pub fn ternary_word_unpack(word: TernaryWord, count: u8) [TRITS_PER_WORD]Trit {
    var result: [TRITS_PER_WORD]Trit = [_]Trit{.zero} ** TRITS_PER_WORD;

    const n: usize = @min(count, TRITS_PER_WORD);
    for (0..n) |i| {
        const byte_idx = i / TRITS_PER_BYTE;
        const trit_pos: u8 = @intCast(i % TRITS_PER_BYTE);
        const unpacked = unpack_trit(trit_pos, word[byte_idx]);
        if (unpacked.valid) {
            result[i] = unpacked.value;
        }
    }

    return result;
}

/// trit_compare(a, b) -> i8: Compare two trits
pub fn trit_compare(a: Trit, b: Trit) i8 {
    if (a == b) {
        return 0;
    } else if (a == .neg or (a == .zero and b == .pos)) {
        return -1;
    } else {
        return 1;
    }
}

/// trit_min(a, b) -> Trit: Returns minimum of two trits
pub fn trit_min(a: Trit, b: Trit) Trit {
    return if (a == .neg or (a == .zero and b == .pos)) a else b;
}

/// trit_max(a, b) -> Trit: Returns maximum of two trits
pub fn trit_max(a: Trit, b: Trit) Trit {
    return if (a == .pos or (a == .zero and b == .neg)) a else b;
}

/// trit_abs(a) -> Trit: Absolute value of a trit
pub fn trit_abs(a: Trit) Trit {
    return if (a == .neg) .pos else a;
}

/// trit_from_i8(value) -> Trit: Safe conversion from i8
pub fn trit_from_i8(value: i8) Trit {
    return switch (value) {
        -1 => .neg,
        0 => .zero,
        1 => .pos,
        else => .zero,
    };
}

/// trit_and(a, b) -> Trit: Ternary AND
pub fn trit_and(a: Trit, b: Trit) Trit {
    return switch (a) {
        .pos => b,
        .zero => if (b == .zero) .zero else .neg,
        .neg => .neg,
    };
}

/// trit_or(a, b) -> Trit: Ternary OR
pub fn trit_or(a: Trit, b: Trit) Trit {
    return switch (a) {
        .pos => .pos,
        .zero => if (b == .zero) .zero else .pos,
        .neg => if (b == .neg) .neg else b,
    };
}

/// trit_xor(a, b) -> Trit: Ternary XOR
pub fn trit_xor(a: Trit, b: Trit) Trit {
    return if (a == b) (if (a == .neg) .neg else .zero) else .pos;
}

/// trit_not(a) -> Trit: Ternary NOT
pub fn trit_not(a: Trit) Trit {
    return if (a == .pos) .zero else .pos;
}

/// trit_select(condition, a, b) -> Trit: Ternary selection
pub fn trit_select(condition: Trit, a: Trit, b: Trit) Trit {
    return if (condition == .pos) a else b;
}

/// packed_trit_count(packed, value) -> u8: Count occurrences of value
pub fn packed_trit_count(packed: PackedTrit, value: Trit) u8 {
    var count: u8 = 0;
    for (0..TRITS_PER_BYTE) |i| {
        const unpacked = unpack_trit(@intCast(i), packed);
        if (unpacked.valid and unpacked.value == value) {
            count += 1;
        }
    }
    return count;
}

/// packed_trit_all_equal(packed, value) -> bool: All trits equal value
pub fn packed_trit_all_equal(packed: PackedTrit, value: Trit) bool {
    return packed_trit_count(packed, value) == TRITS_PER_BYTE;
}

/// packed_trit_is_zero(packed) -> bool: All trits are zero
pub fn packed_trit_is_zero(packed: PackedTrit) bool {
    return packed_trit_all_equal(packed, .zero);
}

/// packed_trit_is_all_same(packed) -> bool: All trits are the same
pub fn packed_trit_is_all_same(packed: PackedTrit) bool {
    const first = unpack_trit(0, packed).value;
    return packed_trit_all_equal(packed, first);
}

/// packed_trit_nand(a, b) -> PackedTrit: Element-wise NAND
pub fn packed_trit_nand(a: PackedTrit, b: PackedTrit) PackedTrit {
    var result: PackedTrit = 0;
    for (0..TRITS_PER_BYTE) |i| {
        const a_trit = unpack_trit(@intCast(i), a).value;
        const b_trit = unpack_trit(@intCast(i), b).value;
        const and_result = trit_and(a_trit, b_trit);
        const nand_result = trit_not(and_result);
        result = pack_trit(nand_result, @intCast(i), result);
    }
    return result;
}

/// packed_trit_nor(a, b) -> PackedTrit: Element-wise NOR
pub fn packed_trit_nor(a: PackedTrit, b: PackedTrit) PackedTrit {
    var result: PackedTrit = 0;
    for (0..TRITS_PER_BYTE) |i| {
        const a_trit = unpack_trit(@intCast(i), a).value;
        const b_trit = unpack_trit(@intCast(i), b).value;
        const or_result = trit_or(a_trit, b_trit);
        const nor_result = trit_not(or_result);
        result = pack_trit(nor_result, @intCast(i), result);
    }
    return result;
}

/// packed_trit_xnor(a, b) -> PackedTrit: Element-wise XNOR
pub fn packed_trit_xnor(a: PackedTrit, b: PackedTrit) PackedTrit {
    var result: PackedTrit = 0;
    for (0..TRITS_PER_BYTE) |i| {
        const a_trit = unpack_trit(@intCast(i), a).value;
        const b_trit = unpack_trit(@intCast(i), b).value;
        const xor_result = trit_xor(a_trit, b_trit);
        const xnor_result = trit_not(xor_result);
        result = pack_trit(xnor_result, @intCast(i), result);
    }
    return result;
}

/// packed_trit_shift_left(packed, shift) -> PackedTrit: Left shift
pub fn packed_trit_shift_left(packed: PackedTrit, shift: u8) PackedTrit {
    if (shift == 0) return packed;
    if (shift >= TRITS_PER_BYTE) return 0;

    var result: PackedTrit = 0;
    for (0..TRITS_PER_BYTE) |i| {
        if (i >= shift) {
            const src_pos: u8 = @intCast(i - shift);
            const src_trit = unpack_trit(src_pos, packed).value;
            result = pack_trit(src_trit, @intCast(i), result);
        }
    }
    return result;
}

/// packed_trit_shift_right(packed, shift) -> PackedTrit: Right shift
pub fn packed_trit_shift_right(packed: PackedTrit, shift: u8) PackedTrit {
    if (shift == 0) return packed;
    if (shift >= TRITS_PER_BYTE) return 0;

    var result: PackedTrit = 0;
    for (0..TRITS_PER_BYTE) |i| {
        const dst_pos: u8 = @intCast(i + shift);
        if (dst_pos < TRITS_PER_BYTE) {
            const src_trit = unpack_trit(@intCast(i), packed).value;
            result = pack_trit(src_trit, dst_pos, result);
        }
    }
    return result;
}

/// packed_trit_rotate_left(packed, rotate) -> PackedTrit: Circular left shift
pub fn packed_trit_rotate_left(packed: PackedTrit, rotate: u8) PackedTrit {
    if (rotate == 0) return packed;

    const shift = rotate % TRITS_PER_BYTE;
    var result: PackedTrit = 0;

    for (0..TRITS_PER_BYTE) |i| {
        const src_pos: u8 = @intCast((i + TRITS_PER_BYTE - shift) % TRITS_PER_BYTE);
        const src_trit = unpack_trit(src_pos, packed).value;
        result = pack_trit(src_trit, @intCast(i), result);
    }

    return result;
}

/// packed_trit_rotate_right(packed, rotate) -> PackedTrit: Circular right shift
pub fn packed_trit_rotate_right(packed: PackedTrit, rotate: u8) PackedTrit {
    if (rotate == 0) return packed;

    const shift = rotate % TRITS_PER_BYTE;
    var result: PackedTrit = 0;

    for (0..TRITS_PER_BYTE) |i| {
        const src_pos: u8 = @intCast((i + shift) % TRITS_PER_BYTE);
        const src_trit = unpack_trit(src_pos, packed).value;
        result = pack_trit(src_trit, @intCast(i), result);
    }

    return result;
}

/// ternary_word_is_zero(word) -> bool: All 27 trits are zero
pub fn ternary_word_is_zero(word: TernaryWord) bool {
    for (0..TRITS_PER_WORD) |i| {
        const byte_idx = i / TRITS_PER_BYTE;
        const trit_pos: u8 = @intCast(i % TRITS_PER_BYTE);
        const unpacked = unpack_trit(trit_pos, word[byte_idx]);
        if (unpacked.valid and unpacked.value != .zero) {
            return false;
        }
    }
    return true;
}

/// ternary_word_count(word, value) -> u8: Count occurrences of value
pub fn ternary_word_count(word: TernaryWord, value: Trit) u8 {
    var count: u8 = 0;
    for (0..TRITS_PER_WORD) |i| {
        const byte_idx = i / TRITS_PER_BYTE;
        const trit_pos: u8 = @intCast(i % TRITS_PER_BYTE);
        const unpacked = unpack_trit(trit_pos, word[byte_idx]);
        if (unpacked.valid and unpacked.value == value) {
            count += 1;
        }
    }
    return count;
}

/// ternary_word_eq(a, b) -> bool: Compare two TernaryWords for equality
pub fn ternary_word_eq(a: TernaryWord, b: TernaryWord) bool {
    for (0..WORD_BYTES) |i| {
        if (a[i] != b[i]) {
            return false;
        }
    }
    return true;
}

/// ternary_word_negate(word) -> TernaryWord: Negate all trits
pub fn ternary_word_negate(word: TernaryWord) TernaryWord {
    var result: TernaryWord = [_]u8{ 0, 0, 0, 0, 0 };
    for (0..TRITS_PER_WORD) |i| {
        const byte_idx = i / TRITS_PER_BYTE;
        const trit_pos: u8 = @intCast(i % TRITS_PER_BYTE);
        const unpacked = unpack_trit(trit_pos, word[byte_idx]);
        const negated = trit_negate(unpacked.value);
        result[byte_idx] = pack_trit(negated, trit_pos, result[byte_idx]);
    }
    return result;
}

/// ternary_word_is_all_same(word) -> bool: All 27 trits are the same value
pub fn ternary_word_is_all_same(word: TernaryWord) bool {
    const first = unpack_trit(0, word[0]).value;
    return ternary_word_count(word, first) == TRITS_PER_WORD;
}

// ============================================================================
// Tests
// ============================================================================

test "test_trit_add_neg_plus_pos_equals_zero" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_add(.neg, .pos));
}

test "test_trit_add_identity" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_add(.zero, .neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_add(.zero, .zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_add(.zero, .pos));
}

test "test_trit_mul_neg_times_neg_equals_pos" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_multiply(.neg, .neg));
}

test "test_trit_mul_zero_annihilates" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_multiply(.zero, .neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_multiply(.zero, .zero));
    try std.testing.expectEqual(@as(Trit, .zero), trit_multiply(.zero, .pos));
}

test "test_pack_unpack_roundtrip" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |trit| {
        const packed = pack_trit(trit, 3, 0);
        const unpacked = unpack_trit(3, packed);
        try std.testing.expectEqual(trit, unpacked.value);
        try std.testing.expect(unpacked.valid);
    }
}

test "test_pack_trit_all_positions" {
    for (0..8) |pos| {
        const packed = pack_trit(.pos, @intCast(pos), 0);
        const unpacked = unpack_trit(@intCast(pos), packed);
        try std.testing.expectEqual(@as(Trit, .pos), unpacked.value);
        try std.testing.expect(unpacked.valid);
    }
}

test "test_pack_trit_invalid_position_rejected" {
    const result = pack_trit(.pos, 8, 0);
    try std.testing.expectEqual(@as(PackedTrit, 0xFF), result);
}

test "test_ternary_word_pack_max_trits" {
    const src = [_]Trit{.pos} ** TRITS_PER_WORD;
    const result = ternary_word_pack(&src, TRITS_PER_WORD);
    try std.testing.expect(result[0] != 0xFF);
}

test "test_ternary_word_pack_exceeds_max" {
    const src = [_]Trit{.pos} ** (TRITS_PER_WORD + 1);
    const result = ternary_word_pack(&src, TRITS_PER_WORD + 1);
    try std.testing.expectEqual(@as(u8, 0xFF), result[0]);
}

test "test_trit_negate_neg_to_pos" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_negate(.neg));
}

test "test_trit_negate_zero_to_zero" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_negate(.zero));
}

test "test_trit_negate_pos_to_neg" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_negate(.pos));
}

test "test_trit_negate_double_negate_identity" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |trit| {
        try std.testing.expectEqual(trit, trit_negate(trit_negate(trit)));
    }
}

test "test_trit_multiply_commutative" {
    const trits = [_]Trit{ .neg, .zero, .pos };
    for (trits) |a| {
        for (trits) |b| {
            try std.testing.expectEqual(trit_multiply(a, b), trit_multiply(b, a));
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

test "test_trit_min_returns_minimum" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_min(.neg, .pos));
    try std.testing.expectEqual(@as(Trit, .neg), trit_min(.neg, .zero));
    try std.testing.expectEqual(@as(Trit, .zero), trit_min(.zero, .pos));
}

test "test_trit_max_returns_maximum" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_max(.neg, .pos));
    try std.testing.expectEqual(@as(Trit, .pos), trit_max(.zero, .pos));
    try std.testing.expectEqual(@as(Trit, .zero), trit_max(.neg, .zero));
}

test "test_trit_abs_non_negative" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_abs(.neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_abs(.zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_abs(.pos));
}

test "test_trit_to_packed_conversion" {
    try std.testing.expectEqual(@as(u8, PACKED_NEG), trit_to_packed(.neg));
    try std.testing.expectEqual(@as(u8, PACKED_ZERO), trit_to_packed(.zero));
    try std.testing.expectEqual(@as(u8, PACKED_ONE), trit_to_packed(.pos));
}

test "test_packed_to_trit_conversion" {
    try std.testing.expectEqual(@as(Trit, .neg), packed_to_trit(PACKED_NEG));
    try std.testing.expectEqual(@as(Trit, .zero), packed_to_trit(PACKED_ZERO));
    try std.testing.expectEqual(@as(Trit, .pos), packed_to_trit(PACKED_ONE));
}

test "test_ternary_word_pack_unpack_roundtrip" {
    const src = [_]Trit{ .neg, .zero, .pos, .neg, .zero, .pos };
    const packed = ternary_word_pack(&src, 6);
    const unpacked = ternary_word_unpack(packed, 6);
    for (0..6) |i| {
        try std.testing.expectEqual(src[i], unpacked[i]);
    }
}

test "test_trit_from_i8_valid_values" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_from_i8(-1));
    try std.testing.expectEqual(@as(Trit, .zero), trit_from_i8(0));
    try std.testing.expectEqual(@as(Trit, .pos), trit_from_i8(1));
}

test "test_trit_from_i8_invalid_values_clamp_to_zero" {
    try std.testing.expectEqual(@as(Trit, .zero), trit_from_i8(-2));
    try std.testing.expectEqual(@as(Trit, .zero), trit_from_i8(2));
    try std.testing.expectEqual(@as(Trit, .zero), trit_from_i8(100));
    try std.testing.expectEqual(@as(Trit, .zero), trit_from_i8(-100));
}

test "test_trit_and_truth_table" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_and(.neg, .neg));
    try std.testing.expectEqual(@as(Trit, .neg), trit_and(.neg, .zero));
    try std.testing.expectEqual(@as(Trit, .neg), trit_and(.neg, .pos));
    try std.testing.expectEqual(@as(Trit, .neg), trit_and(.zero, .neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_and(.zero, .zero));
    try std.testing.expectEqual(@as(Trit, .zero), trit_and(.zero, .pos));
    try std.testing.expectEqual(@as(Trit, .neg), trit_and(.pos, .neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_and(.pos, .zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_and(.pos, .pos));
}

test "test_trit_or_truth_table" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_or(.neg, .neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_or(.neg, .zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_or(.neg, .pos));
    try std.testing.expectEqual(@as(Trit, .zero), trit_or(.zero, .neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_or(.zero, .zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_or(.zero, .pos));
    try std.testing.expectEqual(@as(Trit, .pos), trit_or(.pos, .neg));
    try std.testing.expectEqual(@as(Trit, .pos), trit_or(.pos, .zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_or(.pos, .pos));
}

test "test_trit_xor_truth_table" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_xor(.neg, .neg));
    try std.testing.expectEqual(@as(Trit, .pos), trit_xor(.neg, .zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_xor(.neg, .pos));
    try std.testing.expectEqual(@as(Trit, .pos), trit_xor(.zero, .neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_xor(.zero, .zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_xor(.zero, .pos));
    try std.testing.expectEqual(@as(Trit, .pos), trit_xor(.pos, .neg));
    try std.testing.expectEqual(@as(Trit, .pos), trit_xor(.pos, .zero));
    try std.testing.expectEqual(@as(Trit, .zero), trit_xor(.pos, .pos));
}

test "test_trit_not_truth_table" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_not(.neg));
    try std.testing.expectEqual(@as(Trit, .pos), trit_not(.zero));
    try std.testing.expectEqual(@as(Trit, .zero), trit_not(.pos));
}

test "test_trit_select_condition_true" {
    try std.testing.expectEqual(@as(Trit, .neg), trit_select(.pos, .neg, .pos));
    try std.testing.expectEqual(@as(Trit, .zero), trit_select(.pos, .zero, .neg));
    try std.testing.expectEqual(@as(Trit, .pos), trit_select(.pos, .pos, .zero));
}

test "test_trit_select_condition_false" {
    try std.testing.expectEqual(@as(Trit, .pos), trit_select(.neg, .neg, .pos));
    try std.testing.expectEqual(@as(Trit, .neg), trit_select(.neg, .zero, .neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_select(.neg, .pos, .zero));
    try std.testing.expectEqual(@as(Trit, .pos), trit_select(.zero, .neg, .pos));
    try std.testing.expectEqual(@as(Trit, .neg), trit_select(.zero, .zero, .neg));
    try std.testing.expectEqual(@as(Trit, .zero), trit_select(.zero, .pos, .zero));
}

test "test_ternary_word_is_zero_true" {
    const word: TernaryWord = [_]u8{0} ** WORD_BYTES;
    try std.testing.expect(ternary_word_is_zero(word));
}

test "test_ternary_word_is_zero_false" {
    var word: TernaryWord = [_]u8{0} ** WORD_BYTES;
    word[0] = pack_trit(.pos, 0, word[0]);
    try std.testing.expect(!ternary_word_is_zero(word));
}

test "test_ternary_word_eq_same" {
    var word: TernaryWord = [_]u8{0} ** WORD_BYTES;
    word[0] = pack_trit(.pos, 0, word[0]);
    word[1] = pack_trit(.neg, 0, word[1]);
    try std.testing.expect(ternary_word_eq(word, word));
}

test "test_ternary_word_eq_different" {
    var word_a: TernaryWord = [_]u8{0} ** WORD_BYTES;
    var word_b: TernaryWord = [_]u8{0} ** WORD_BYTES;
    word_a[0] = pack_trit(.pos, 0, word_a[0]);
    word_b[0] = pack_trit(.neg, 0, word_b[0]);
    try std.testing.expect(!ternary_word_eq(word_a, word_b));
}

test "test_ternary_word_negate_double_identity" {
    var word: TernaryWord = [_]u8{0} ** WORD_BYTES;
    word[0] = pack_trit(.pos, 0, word[0]);
    word[1] = pack_trit(.neg, 0, word[1]);
    const double_negated = ternary_word_negate(ternary_word_negate(word));
    try std.testing.expect(ternary_word_eq(word, double_negated));
}

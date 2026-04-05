// Auto-generated from specs/vsa/ops.t27
// DO NOT EDIT -- regenerate with: tri gen specs/vsa/ops.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 28 | Module: VSAOps

const std = @import("std");
const math = std.math;

// =====================================================================
// Trit type for balanced ternary hypervectors
// =====================================================================

pub const Trit = enum(i8) {
    neg = -1,
    zero = 0,
    pos = 1,
};

// =====================================================================
// 1. Constants
// =====================================================================

pub const VSA_DIM: usize = 1024;
pub const SIMD_WIDTH: usize = 32;
pub const MAX_VECTORS: usize = 32;

pub const SIM_COSINE: u8 = 0;
pub const SIM_HAMMING: u8 = 1;
pub const SIM_DOT: u8 = 2;

pub const BIND_IDENTITY: u8 = 0;
pub const BIND_INVERT: u8 = 1;

// =====================================================================
// 2. Bind Operation
// =====================================================================

/// bind(a, b, len): XOR-like operation for balanced ternary hypervectors.
/// If a[i]==0 return b[i]; if b[i]==0 return a[i];
/// else return a[i]*b[i] (both non-zero => +1 or -1).
pub fn bind(a: []const Trit, b: []const Trit, len: usize) [VSA_DIM]Trit {
    var result: [VSA_DIM]Trit = [_]Trit{.zero} ** VSA_DIM;
    var i: usize = 0;
    while (i < len) : (i += 1) {
        const ai = a[i];
        const bi = b[i];
        if (ai == .zero) {
            result[i] = bi;
        } else if (bi == .zero) {
            result[i] = ai;
        } else {
            result[i] = if (ai == bi) Trit.pos else Trit.neg;
        }
    }
    return result;
}

/// Heap-allocated variant for dynamic lengths.
pub fn bindSlice(allocator: std.mem.Allocator, a: []const Trit, b: []const Trit, len: usize) ![]Trit {
    var result = try allocator.alloc(Trit, len);
    var i: usize = 0;
    while (i < len) : (i += 1) {
        const ai = a[i];
        const bi = b[i];
        if (ai == .zero) {
            result[i] = bi;
        } else if (bi == .zero) {
            result[i] = ai;
        } else {
            result[i] = if (ai == bi) Trit.pos else Trit.neg;
        }
    }
    return result;
}

// =====================================================================
// 3. Unbind Operation
// =====================================================================

/// unbind(bound, key, len): Inverse of bind. For XOR-like bind: unbind = bind.
pub fn unbind(bound: []const Trit, key: []const Trit, len: usize) [VSA_DIM]Trit {
    return bind(bound, key, len);
}

pub fn unbindSlice(allocator: std.mem.Allocator, bound: []const Trit, key: []const Trit, len: usize) ![]Trit {
    return bindSlice(allocator, bound, key, len);
}

// =====================================================================
// 4. Bundle Operations
// =====================================================================

/// bundle2(a, b, len): Majority vote of 2 ternary vectors.
/// If a[i]==0 return b[i]; if b[i]==0 return a[i];
/// else sign(a[i]+b[i]).
pub fn bundle2(a: []const Trit, b: []const Trit, len: usize) [VSA_DIM]Trit {
    var result: [VSA_DIM]Trit = [_]Trit{.zero} ** VSA_DIM;
    var i: usize = 0;
    while (i < len) : (i += 1) {
        const ai = a[i];
        const bi = b[i];
        if (ai == .zero) {
            result[i] = bi;
        } else if (bi == .zero) {
            result[i] = ai;
        } else {
            const sum = @as(i16, @intFromEnum(ai)) + @as(i16, @intFromEnum(bi));
            result[i] = if (sum > 0) Trit.pos else if (sum < 0) Trit.neg else Trit.zero;
        }
    }
    return result;
}

/// bundle3(a, b, c, len): Majority vote of 3 ternary vectors.
/// sum = a[i]+b[i]+c[i]; result = sign(sum).
pub fn bundle3(a: []const Trit, b: []const Trit, c: []const Trit, len: usize) [VSA_DIM]Trit {
    var result: [VSA_DIM]Trit = [_]Trit{.zero} ** VSA_DIM;
    var i: usize = 0;
    while (i < len) : (i += 1) {
        const sum = @as(i16, @intFromEnum(a[i])) + @as(i16, @intFromEnum(b[i])) + @as(i16, @intFromEnum(c[i]));
        result[i] = if (sum > 0) Trit.pos else if (sum < 0) Trit.neg else Trit.zero;
    }
    return result;
}

// =====================================================================
// 5. Similarity Operations
// =====================================================================

/// dot_product(a, b, len): Compute sum of a[i]*b[i].
pub fn dot_product(a: []const Trit, b: []const Trit, len: usize) f64 {
    var acc: i64 = 0;
    var i: usize = 0;
    while (i < len) : (i += 1) {
        const product = @as(i16, @intFromEnum(a[i])) * @as(i16, @intFromEnum(b[i]));
        acc += @as(i64, product);
    }
    return @as(f64, @floatFromInt(acc));
}

/// vector_norm(v, len): L2 norm = sqrt(count of non-zero trits).
pub fn vector_norm(v: []const Trit, len: usize) f64 {
    var nonzero_count: usize = 0;
    var i: usize = 0;
    while (i < len) : (i += 1) {
        if (v[i] != .zero) {
            nonzero_count += 1;
        }
    }
    return @sqrt(@as(f64, @floatFromInt(nonzero_count)));
}

/// hamming_distance(a, b, len): Count positions where a[i] != b[i].
pub fn hamming_distance(a: []const Trit, b: []const Trit, len: usize) usize {
    var distance: usize = 0;
    var i: usize = 0;
    while (i < len) : (i += 1) {
        if (a[i] != b[i]) {
            distance += 1;
        }
    }
    return distance;
}

/// cosine_similarity(a, b, len): (a.b) / (||a|| * ||b||).
pub fn cosine_similarity(a: []const Trit, b: []const Trit, len: usize) f64 {
    const dot = dot_product(a, b, len);
    const norm_a = vector_norm(a, len);
    const norm_b = vector_norm(b, len);
    if (norm_a == 0.0 or norm_b == 0.0) {
        return 0.0;
    }
    return dot / (norm_a * norm_b);
}

/// hamming_similarity(a, b, len): 1 - (hamming_distance / len).
pub fn hamming_similarity(a: []const Trit, b: []const Trit, len: usize) f64 {
    const dist = hamming_distance(a, b, len);
    return 1.0 - (@as(f64, @floatFromInt(dist)) / @as(f64, @floatFromInt(len)));
}

/// similarity(a, b, len, metric): Dispatch to cosine/hamming/dot.
pub fn similarity(a: []const Trit, b: []const Trit, len: usize, metric: u8) f64 {
    if (metric == SIM_COSINE) {
        return cosine_similarity(a, b, len);
    } else if (metric == SIM_HAMMING) {
        return hamming_similarity(a, b, len);
    }
    return dot_product(a, b, len);
}

// =====================================================================
// 6. Permutation Operations
// =====================================================================

/// permute(v, len, shift): Circular shift of hypervector.
pub fn permute(v: []const Trit, len: usize, shift: usize) [VSA_DIM]Trit {
    var result: [VSA_DIM]Trit = [_]Trit{.zero} ** VSA_DIM;
    const normalized_shift = shift % len;
    var i: usize = 0;
    while (i < len) : (i += 1) {
        const src_idx = (i + normalized_shift) % len;
        result[i] = v[src_idx];
    }
    return result;
}

/// encode_sequence(items, count, item_len): Position-aware bundle.
/// result = items[0] + permute(items[1],1) + permute(items[2],2) + ...
pub fn encode_sequence(items: []const []const Trit, count: usize, item_len: usize) [VSA_DIM]Trit {
    var result: [VSA_DIM]Trit = [_]Trit{.zero} ** VSA_DIM;
    // Copy items[0]
    var j: usize = 0;
    while (j < item_len) : (j += 1) {
        result[j] = items[0][j];
    }
    var i: usize = 1;
    while (i < count) : (i += 1) {
        const permuted = permute(items[i], item_len, i);
        result = bundle2(result[0..item_len], permuted[0..item_len], item_len);
        i += 0; // no-op, loop increments
    }
    return result;
}

/// probe_sequence(seq, candidate, position, len): Similarity of seq vs permute(candidate, position).
pub fn probe_sequence(seq: []const Trit, candidate: []const Trit, position: usize, len: usize) f64 {
    const permuted = permute(candidate, len, position);
    return similarity(seq, permuted[0..len], len, SIM_COSINE);
}

// =====================================================================
// Tests
// =====================================================================

test "vsa_bind_with_zeros" {
    const a = [_]Trit{ .zero, .pos, .neg };
    const b = [_]Trit{ .pos, .zero, .neg };
    const result = bind(&a, &b, 3);
    try std.testing.expectEqual(Trit.pos, result[0]);
    try std.testing.expectEqual(Trit.pos, result[1]);
    try std.testing.expectEqual(Trit.pos, result[2]);
}

test "vsa_bind_nonzero_multiply" {
    const a = [_]Trit{ .pos, .pos, .neg, .neg };
    const b = [_]Trit{ .pos, .neg, .pos, .neg };
    const result = bind(&a, &b, 4);
    try std.testing.expectEqual(Trit.pos, result[0]);
    try std.testing.expectEqual(Trit.neg, result[1]);
    try std.testing.expectEqual(Trit.neg, result[2]);
    try std.testing.expectEqual(Trit.pos, result[3]);
}

test "vsa_bundle2_with_zero" {
    const a = [_]Trit{ .zero, .pos, .neg };
    const b = [_]Trit{ .pos, .zero, .neg };
    const result = bundle2(&a, &b, 3);
    try std.testing.expectEqual(Trit.pos, result[0]);
    try std.testing.expectEqual(Trit.pos, result[1]);
    try std.testing.expectEqual(Trit.neg, result[2]);
}

test "vsa_bundle2_majority_vote" {
    const a = [_]Trit{ .pos, .neg, .pos };
    const b = [_]Trit{ .neg, .neg, .neg };
    const result = bundle2(&a, &b, 3);
    try std.testing.expectEqual(Trit.zero, result[0]);
    try std.testing.expectEqual(Trit.neg, result[1]);
    try std.testing.expectEqual(Trit.zero, result[2]);
}

test "vsa_bundle3_consensus" {
    const a = [_]Trit{ .pos, .pos, .neg };
    const b = [_]Trit{ .pos, .neg, .pos };
    const c = [_]Trit{ .pos, .pos, .pos };
    const result = bundle3(&a, &b, &c, 3);
    try std.testing.expectEqual(Trit.pos, result[0]);
    try std.testing.expectEqual(Trit.pos, result[1]);
    try std.testing.expectEqual(Trit.pos, result[2]);
}

test "vsa_dot_product_identical" {
    const a = [_]Trit{ .pos, .neg, .pos, .zero };
    const b = [_]Trit{ .pos, .neg, .pos, .zero };
    const result = dot_product(&a, &b, 4);
    try std.testing.expectEqual(@as(f64, 3.0), result);
}

test "vsa_dot_product_orthogonal" {
    const a = [_]Trit{ .pos, .neg, .zero };
    const b = [_]Trit{ .neg, .pos, .zero };
    const result = dot_product(&a, &b, 3);
    try std.testing.expectEqual(@as(f64, -2.0), result);
}

test "vsa_hamming_distance_identical" {
    const a = [_]Trit{ .pos, .neg, .zero };
    const b = [_]Trit{ .pos, .neg, .zero };
    try std.testing.expectEqual(@as(usize, 0), hamming_distance(&a, &b, 3));
}

test "vsa_hamming_distance_different" {
    const a = [_]Trit{ .pos, .pos, .pos };
    const b = [_]Trit{ .neg, .neg, .neg };
    try std.testing.expectEqual(@as(usize, 3), hamming_distance(&a, &b, 3));
}

test "vsa_vector_norm_zero_vector" {
    const v = [_]Trit{ .zero, .zero, .zero };
    try std.testing.expectEqual(@as(f64, 0.0), vector_norm(&v, 3));
}

test "vsa_vector_norm_all_nonzero" {
    const v = [_]Trit{ .pos, .neg, .pos };
    const result = vector_norm(&v, 3);
    try std.testing.expect(@abs(result - 1.732) < 0.01);
}

test "vsa_cosine_similarity_identical" {
    const a = [_]Trit{ .pos, .neg, .pos };
    const b = [_]Trit{ .pos, .neg, .pos };
    const result = cosine_similarity(&a, &b, 3);
    try std.testing.expect(@abs(result - 1.0) < 0.01);
}

test "vsa_cosine_similarity_orthogonal" {
    const a = [_]Trit{ .pos, .neg, .zero };
    const b = [_]Trit{ .neg, .pos, .zero };
    const result = cosine_similarity(&a, &b, 3);
    try std.testing.expect(@abs(result + 1.0) < 0.01);
}

test "vsa_permute_shift_by_one" {
    const v = [_]Trit{ .pos, .neg, .zero, .pos };
    const result = permute(&v, 4, 1);
    try std.testing.expectEqual(Trit.neg, result[0]);
    try std.testing.expectEqual(Trit.zero, result[1]);
    try std.testing.expectEqual(Trit.pos, result[2]);
    try std.testing.expectEqual(Trit.pos, result[3]);
}

test "vsa_permute_shift_by_len_returns_original" {
    const v = [_]Trit{ .pos, .neg, .zero };
    const result = permute(&v, 3, 3);
    try std.testing.expectEqual(Trit.pos, result[0]);
    try std.testing.expectEqual(Trit.neg, result[1]);
    try std.testing.expectEqual(Trit.zero, result[2]);
}

test "vsa_permute_shift_zero_unchanged" {
    const v = [_]Trit{ .pos, .neg, .zero };
    const result = permute(&v, 3, 0);
    try std.testing.expectEqual(Trit.pos, result[0]);
    try std.testing.expectEqual(Trit.neg, result[1]);
    try std.testing.expectEqual(Trit.zero, result[2]);
}

test "vsa_bind_unbind_identity" {
    const x = [_]Trit{ .pos, .neg, .zero, .pos };
    const key = [_]Trit{ .neg, .pos, .pos, .neg };
    const bound = bind(&x, &key, 4);
    const unbound = unbind(bound[0..4], &key, 4);
    const sim = similarity(&x, unbound[0..4], 4, SIM_COSINE);
    try std.testing.expect(sim > 0.95);
}

test "vsa_bundle2_idempotent" {
    const v = [_]Trit{ .pos, .neg, .zero };
    const result = bundle2(&v, &v, 3);
    try std.testing.expectEqual(@as(usize, 0), hamming_distance(&v, result[0..3], 3));
}

test "vsa_similarity_symmetry" {
    const a = [_]Trit{ .pos, .neg, .pos, .zero };
    const b = [_]Trit{ .neg, .pos, .neg, .pos };
    const sim_ab = similarity(&a, &b, 4, SIM_COSINE);
    const sim_ba = similarity(&b, &a, 4, SIM_COSINE);
    try std.testing.expect(@abs(sim_ab - sim_ba) < 0.0001);
}

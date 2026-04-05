// Auto-generated from specs/vsa/core.t27
// DO NOT EDIT -- regenerate with: tri gen specs/vsa/core.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: VSACore

const std = @import("std");
const math = std.math;
const ops = @import("ops.zig");

// Re-export Trit from ops
pub const Trit = ops.Trit;

// =====================================================================
// 1. Constants
// =====================================================================

pub const DIMENSION: usize = 1024;
pub const SIMILARITY_THRESHOLD: f64 = 0.15;
pub const CODEBOOK_CAPACITY: usize = 256;
pub const MAX_PREDICATE_ARGS: usize = 8;

// =====================================================================
// 2. Core Types
// =====================================================================

/// HyperVector: array of DIMENSION trits {-1, 0, +1}
pub const HyperVector = [DIMENSION]Trit;

/// Codebook: cleanup memory / item memory for nearest-neighbor lookup
pub const Codebook = struct {
    entries: [CODEBOOK_CAPACITY]HyperVector,
    labels: [CODEBOOK_CAPACITY]u32,
    count: usize,
};

/// PredicateEncoding: result of encoding a predicate-argument structure
pub const PredicateEncoding = struct {
    vector: HyperVector,
    predicate: HyperVector,
    arg_count: usize,
};

// =====================================================================
// 3. Random Hypervector Generation
// =====================================================================

/// random_hypervector(seed): Generate pseudo-random hypervector using xorshift64.
/// Each trit drawn uniformly from {-1, 0, +1}.
pub fn random_hypervector(seed: u64) HyperVector {
    var result: HyperVector = [_]Trit{.zero} ** DIMENSION;
    var state: u64 = seed;

    var i: usize = 0;
    while (i < DIMENSION) : (i += 1) {
        // xorshift64
        state = state ^ (state << 13);
        state = state ^ (state >> 7);
        state = state ^ (state << 17);

        const r = state % 3;
        result[i] = if (r == 0) Trit.neg else if (r == 1) Trit.zero else Trit.pos;
    }

    return result;
}

// =====================================================================
// 4. Predicate-Argument Encoding
// =====================================================================

/// encode_predicate(predicate, args, arg_count): Encode a predicate-argument
/// structure as a single hypervector.
/// Schema: result = bind(predicate, bundle(permute(arg[0],1), permute(arg[1],2), ...))
pub fn encode_predicate(predicate: []const Trit, args: []const []const Trit, arg_count: usize) HyperVector {
    // Permute each argument by its 1-based position
    var permuted_args: [MAX_PREDICATE_ARGS]HyperVector = undefined;
    var i: usize = 0;
    while (i < arg_count) : (i += 1) {
        permuted_args[i] = ops.permute(args[i], DIMENSION, i + 1);
    }

    // Bundle all permuted arguments via iterative majority vote
    var bundled: HyperVector = permuted_args[0];
    i = 1;
    while (i < arg_count) : (i += 1) {
        bundled = ops.bundle2(&bundled, &permuted_args[i], DIMENSION);
    }

    // Bind predicate with bundled arguments
    return ops.bind(predicate, &bundled, DIMENSION);
}

/// decode_argument(encoded, predicate, position): Retrieve the argument at a
/// given position from an encoded predicate.
/// Algorithm: unbind with predicate, then inverse-permute by position.
pub fn decode_argument(encoded: []const Trit, predicate: []const Trit, position: usize) HyperVector {
    // Unbind predicate to get bundled arguments
    const unbound = ops.unbind(encoded, predicate, DIMENSION);

    // Inverse permute to extract the argument at position
    const shift = position + 1;
    const inverse_shift = DIMENSION - (shift % DIMENSION);
    return ops.permute(&unbound, DIMENSION, inverse_shift);
}

// =====================================================================
// 5. Codebook (Cleanup Memory)
// =====================================================================

/// codebook_add(cb, vector, label): Add a labeled vector to the codebook.
/// Returns false if codebook is full.
pub fn codebook_add(cb: *Codebook, vector: []const Trit, label: u32) bool {
    if (cb.count >= CODEBOOK_CAPACITY) {
        return false;
    }

    var i: usize = 0;
    while (i < DIMENSION) : (i += 1) {
        cb.entries[cb.count][i] = vector[i];
    }
    cb.labels[cb.count] = label;
    cb.count = cb.count + 1;
    return true;
}

/// codebook_lookup(cb, query): Find the label of the most similar entry.
/// Uses cosine similarity. Returns 0xFFFFFFFF if codebook is empty.
pub fn codebook_lookup(cb: Codebook, query: []const Trit) u32 {
    if (cb.count == 0) {
        return 0xFFFFFFFF;
    }

    var best_label: u32 = cb.labels[0];
    var best_sim: f64 = -2.0;

    var i: usize = 0;
    while (i < cb.count) : (i += 1) {
        const sim = ops.cosine_similarity(query, &cb.entries[i], DIMENSION);
        if (sim > best_sim) {
            best_sim = sim;
            best_label = cb.labels[i];
        }
    }

    return best_label;
}

/// codebook_cleanup(cb, noisy): Map a noisy vector to the nearest clean entry.
/// Returns the clean vector or the input unchanged if codebook is empty or
/// best similarity is below threshold.
pub fn codebook_cleanup(cb: Codebook, noisy: []const Trit) HyperVector {
    if (cb.count == 0) {
        var result: HyperVector = [_]Trit{.zero} ** DIMENSION;
        var j: usize = 0;
        while (j < DIMENSION) : (j += 1) {
            result[j] = noisy[j];
        }
        return result;
    }

    var best_idx: usize = 0;
    var best_sim: f64 = -2.0;

    var i: usize = 0;
    while (i < cb.count) : (i += 1) {
        const sim = ops.cosine_similarity(noisy, &cb.entries[i], DIMENSION);
        if (sim > best_sim) {
            best_sim = sim;
            best_idx = i;
        }
    }

    if (best_sim < SIMILARITY_THRESHOLD) {
        var result: HyperVector = [_]Trit{.zero} ** DIMENSION;
        var j: usize = 0;
        while (j < DIMENSION) : (j += 1) {
            result[j] = noisy[j];
        }
        return result;
    }

    return cb.entries[best_idx];
}

// =====================================================================
// 6. Compositional Query Operations
// =====================================================================

/// query_role(structure, filler): Retrieve the role from a structure given
/// a known filler. structure = bind(role, filler) => role = unbind(structure, filler).
pub fn query_role(structure: []const Trit, filler: []const Trit) HyperVector {
    return ops.unbind(structure, filler, DIMENSION);
}

/// query_filler(structure, role): Retrieve the filler from a structure given
/// a known role. structure = bind(role, filler) => filler = unbind(structure, role).
pub fn query_filler(structure: []const Trit, role: []const Trit) HyperVector {
    return ops.unbind(structure, role, DIMENSION);
}

/// analogy(a, b, c): Compute d such that a:b :: c:d.
/// Algorithm: d = bind(unbind(b, a), c).
pub fn analogy(a: []const Trit, b: []const Trit, c: []const Trit) HyperVector {
    const relation = ops.unbind(b, a, DIMENSION);
    return ops.bind(&relation, c, DIMENSION);
}

// =====================================================================
// 7. Resonator Network (Factorization)
// =====================================================================

/// resonator_step: One step of resonator network -- unbind other factors,
/// lookup in codebook.
pub fn resonator_step(
    estimates: []const []const Trit,
    target: []const Trit,
    codebooks: []const Codebook,
    factor_idx: usize,
    factor_count: usize,
) HyperVector {
    // Unbind all factors except factor_idx
    var remainder: HyperVector = undefined;
    var j: usize = 0;
    while (j < DIMENSION) : (j += 1) {
        remainder[j] = target[j];
    }

    var i: usize = 0;
    while (i < factor_count) : (i += 1) {
        if (i != factor_idx) {
            const tmp = ops.unbind(&remainder, estimates[i], DIMENSION);
            remainder = tmp;
        }
    }

    // Cleanup via codebook
    return codebook_cleanup(codebooks[factor_idx], &remainder);
}

/// resonator_solve: Iterative resonator network to factorize a composed
/// hypervector. Converges when estimates stabilize.
pub fn resonator_solve(
    target: []const Trit,
    codebooks: []const Codebook,
    factor_count: usize,
    max_iters: usize,
) [MAX_PREDICATE_ARGS]HyperVector {
    // Initialize estimates with first entry of each codebook
    var estimates: [MAX_PREDICATE_ARGS]HyperVector = undefined;
    var i: usize = 0;
    while (i < factor_count) : (i += 1) {
        estimates[i] = codebooks[i].entries[0];
    }

    var iter: usize = 0;
    while (iter < max_iters) : (iter += 1) {
        var converged: bool = true;

        i = 0;
        while (i < factor_count) : (i += 1) {
            const old_estimate = estimates[i];

            // Build slice pointers for estimates
            var est_slices: [MAX_PREDICATE_ARGS][]const Trit = undefined;
            var s: usize = 0;
            while (s < factor_count) : (s += 1) {
                est_slices[s] = &estimates[s];
            }

            estimates[i] = resonator_step(
                est_slices[0..factor_count],
                target,
                codebooks,
                i,
                factor_count,
            );

            const dist = ops.hamming_distance(&old_estimate, &estimates[i], DIMENSION);
            if (dist > 0) {
                converged = false;
            }
        }

        if (converged) {
            break;
        }
    }

    return estimates;
}

// =====================================================================
// Tests
// =====================================================================

test "vsa_core_bind_self_inverse" {
    const a = random_hypervector(42);
    const b = random_hypervector(43);
    const bound = ops.bind(&a, &b, DIMENSION);
    const recovered = ops.unbind(&bound, &b, DIMENSION);
    const sim = ops.cosine_similarity(&a, &recovered, DIMENSION);
    try std.testing.expect(sim > 0.99);
}

test "vsa_core_bundle_preserves_components" {
    const a = random_hypervector(1);
    const b = random_hypervector(2);
    const c = random_hypervector(3);
    const bundled = ops.bundle3(&a, &b, &c, DIMENSION);
    const sim_a = ops.cosine_similarity(&bundled, &a, DIMENSION);
    const sim_b = ops.cosine_similarity(&bundled, &b, DIMENSION);
    const sim_c = ops.cosine_similarity(&bundled, &c, DIMENSION);
    try std.testing.expect(sim_a > SIMILARITY_THRESHOLD);
    try std.testing.expect(sim_b > SIMILARITY_THRESHOLD);
    try std.testing.expect(sim_c > SIMILARITY_THRESHOLD);
}

test "vsa_core_orthogonality_random_vectors" {
    const a = random_hypervector(100);
    const b = random_hypervector(200);
    const sim = ops.cosine_similarity(&a, &b, DIMENSION);
    try std.testing.expect(@abs(sim) < 0.1);
}

test "vsa_core_predicate_encoding_roundtrip" {
    const loves = random_hypervector(10);
    const john = random_hypervector(11);
    const mary = random_hypervector(12);
    const args = [_][]const Trit{ &loves, &john }; // placeholder
    _ = args;
    const john_slice: []const Trit = &john;
    const mary_slice: []const Trit = &mary;
    const arg_slices = [_][]const Trit{ john_slice, mary_slice };
    const encoded = encode_predicate(&loves, &arg_slices, 2);
    const recovered_john = decode_argument(&encoded, &loves, 0);
    const recovered_mary = decode_argument(&encoded, &loves, 1);
    const sim_john = ops.cosine_similarity(&recovered_john, &john, DIMENSION);
    const sim_mary = ops.cosine_similarity(&recovered_mary, &mary, DIMENSION);
    try std.testing.expect(sim_john > SIMILARITY_THRESHOLD);
    try std.testing.expect(sim_mary > SIMILARITY_THRESHOLD);
}

test "vsa_core_codebook_store_and_retrieve" {
    var cb = Codebook{
        .entries = undefined,
        .labels = undefined,
        .count = 0,
    };
    const v1 = random_hypervector(50);
    const v2 = random_hypervector(51);
    _ = codebook_add(&cb, &v1, 1);
    _ = codebook_add(&cb, &v2, 2);
    const label1 = codebook_lookup(cb, &v1);
    const label2 = codebook_lookup(cb, &v2);
    try std.testing.expectEqual(@as(u32, 1), label1);
    try std.testing.expectEqual(@as(u32, 2), label2);
}

test "vsa_core_codebook_cleanup_returns_nearest" {
    var cb = Codebook{
        .entries = undefined,
        .labels = undefined,
        .count = 0,
    };
    const clean = random_hypervector(60);
    _ = codebook_add(&cb, &clean, 1);
    const noise = random_hypervector(61);
    const noisy = ops.bundle2(&clean, &noise, DIMENSION);
    const cleaned = codebook_cleanup(cb, &noisy);
    const sim = ops.cosine_similarity(&cleaned, &clean, DIMENSION);
    try std.testing.expect(sim > SIMILARITY_THRESHOLD);
}

test "vsa_core_query_role_filler" {
    const role = random_hypervector(20);
    const filler = random_hypervector(21);
    const structure = ops.bind(&role, &filler, DIMENSION);
    const recovered_filler = query_filler(&structure, &role);
    const recovered_role = query_role(&structure, &filler);
    const sim_filler = ops.cosine_similarity(&recovered_filler, &filler, DIMENSION);
    const sim_role = ops.cosine_similarity(&recovered_role, &role, DIMENSION);
    try std.testing.expect(sim_filler > 0.99);
    try std.testing.expect(sim_role > 0.99);
}

test "vsa_core_analogy" {
    const king = random_hypervector(30);
    const queen = random_hypervector(31);
    const royalty = ops.unbind(&queen, &king, DIMENSION);
    const queen_reconstructed = ops.bind(&royalty, &king, DIMENSION);
    const sim = ops.cosine_similarity(&queen_reconstructed, &queen, DIMENSION);
    try std.testing.expect(sim > 0.99);
}

test "vsa_core_random_vectors_dense" {
    const v = random_hypervector(99);
    const norm = ops.vector_norm(&v, DIMENSION);
    try std.testing.expect(norm > 0.0);
}

test "vsa_core_permute_preserves_information" {
    const v = random_hypervector(70);
    const shifted = ops.permute(&v, DIMENSION, 5);
    const unshifted = ops.permute(&shifted, DIMENSION, DIMENSION - 5);
    const sim = ops.cosine_similarity(&v, &unshifted, DIMENSION);
    try std.testing.expect(sim > 0.99);
}

test "vsa_core_permute_decorrelates" {
    const v = random_hypervector(80);
    const shifted = ops.permute(&v, DIMENSION, 1);
    const sim = ops.cosine_similarity(&v, &shifted, DIMENSION);
    try std.testing.expect(@abs(sim) < 0.15);
}

test "vsa_core_bundle3_consensus" {
    const a = random_hypervector(90);
    const noise = random_hypervector(91);
    const bundled = ops.bundle3(&a, &a, &noise, DIMENSION);
    const sim = ops.cosine_similarity(&bundled, &a, DIMENSION);
    try std.testing.expect(sim > 0.5);
}

test "vsa_core_encode_single_arg" {
    const pred = random_hypervector(40);
    const arg0 = random_hypervector(41);
    const arg0_slice: []const Trit = &arg0;
    const arg_slices = [_][]const Trit{arg0_slice};
    const encoded = encode_predicate(&pred, &arg_slices, 1);
    const decoded = decode_argument(&encoded, &pred, 0);
    const sim = ops.cosine_similarity(&decoded, &arg0, DIMENSION);
    try std.testing.expect(sim > SIMILARITY_THRESHOLD);
}

test "vsa_core_codebook_empty_returns_sentinel" {
    const cb = Codebook{
        .entries = undefined,
        .labels = undefined,
        .count = 0,
    };
    const query = random_hypervector(55);
    const label = codebook_lookup(cb, &query);
    try std.testing.expectEqual(@as(u32, 0xFFFFFFFF), label);
}

test "vsa_core_codebook_full_rejects_add" {
    var cb = Codebook{
        .entries = undefined,
        .labels = undefined,
        .count = CODEBOOK_CAPACITY,
    };
    const v = random_hypervector(56);
    const ok = codebook_add(&cb, &v, 999);
    try std.testing.expect(!ok);
}

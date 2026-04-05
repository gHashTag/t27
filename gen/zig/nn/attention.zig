// Auto-generated from specs/nn/attention.t27
// DO NOT EDIT -- regenerate with: tri gen specs/nn/attention.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 29 | Module: SacredAttention | Multi-head attention with phi-RoPE

const std = @import("std");
const math = std.math;

// =====================================================================
// Trit type (ternary weight)
// =====================================================================

pub const Trit = enum(i8) {
    neg = -1,
    zero = 0,
    pos = 1,
};

// =====================================================================
// Constants
// =====================================================================

pub const PHI: f64 = 1.6180339887498948482;
pub const PHI_INV: f64 = 0.6180339887498948482;

pub const NUM_HEADS: usize = 3;
pub const HEAD_DIM: usize = 81; // 3^4
pub const EMBED_DIM: usize = 243; // 3 * 81
pub const CONTEXT_LEN: usize = 81;
pub const ROPE_PAIRS: usize = 40; // 81 / 2

/// Sacred scaling: phi^-3
pub const SACRED_GAMMA: f64 = PHI_INV * PHI_INV * PHI_INV; // phi^(-3) ~ 0.2360679

/// Sacred scale: 81^(-phi^-3) ~ 0.354
pub const SACRED_SCALE: f64 = blk: {
    // pow(81.0, -SACRED_GAMMA) computed at comptime
    break :blk @exp(-SACRED_GAMMA * @log(81.0));
};

pub const ATTN_CAUSAL: u8 = 0;
pub const ATTN_BIDIR: u8 = 1;
pub const ATTN_SPARSE: u8 = 2;

pub const PHASE_QUERY: u8 = 0;
pub const PHASE_KEY: u8 = 1;
pub const PHASE_VALUE: u8 = 2;
pub const PHASE_SCORE: u8 = 3;
pub const PHASE_SOFTMAX: u8 = 4;
pub const PHASE_WEIGHT: u8 = 5;

// =====================================================================
// RoPE Tables
// =====================================================================

pub const RoPETables = struct {
    cos_table: [CONTEXT_LEN * ROPE_PAIRS]f64,
    sin_table: [CONTEXT_LEN * ROPE_PAIRS]f64,
};

var rope_tables: RoPETables = .{
    .cos_table = [_]f64{0.0} ** (CONTEXT_LEN * ROPE_PAIRS),
    .sin_table = [_]f64{0.0} ** (CONTEXT_LEN * ROPE_PAIRS),
};

// =====================================================================
// Attention Buffers
// =====================================================================

pub const AttentionBuffers = struct {
    q_buffer: [EMBED_DIM]f64,
    k_buffer: [EMBED_DIM]f64,
    v_buffer: [EMBED_DIM]f64,
    scores: [NUM_HEADS * CONTEXT_LEN]f64,
    concat: [EMBED_DIM]f64,
};

pub fn initAttentionBuffers() AttentionBuffers {
    return AttentionBuffers{
        .q_buffer = [_]f64{0.0} ** EMBED_DIM,
        .k_buffer = [_]f64{0.0} ** EMBED_DIM,
        .v_buffer = [_]f64{0.0} ** EMBED_DIM,
        .scores = [_]f64{0.0} ** (NUM_HEADS * CONTEXT_LEN),
        .concat = [_]f64{0.0} ** EMBED_DIM,
    };
}

// =====================================================================
// Initialization: phi-RoPE tables
// =====================================================================

/// Initialize phi-RoPE tables.
/// theta_i = phi^(-2i/HEAD_DIM) for i=0..ROPE_PAIRS-1
/// For each position p: cos(p*theta_i), sin(p*theta_i)
pub fn sacred_attention_init() void {
    var p: usize = 0;
    while (p < CONTEXT_LEN) : (p += 1) {
        var i: usize = 0;
        while (i < ROPE_PAIRS) : (i += 1) {
            const freq_exponent: f64 = -2.0 * @as(f64, @floatFromInt(i)) / @as(f64, @floatFromInt(HEAD_DIM));
            const freq: f64 = math.pow(f64, PHI, freq_exponent);
            const angle: f64 = @as(f64, @floatFromInt(p)) * freq;

            const table_offset = p * ROPE_PAIRS + i;
            rope_tables.cos_table[table_offset] = @cos(angle);
            rope_tables.sin_table[table_offset] = @sin(angle);
        }
    }
}

// =====================================================================
// Ternary Matrix Multiplication
// =====================================================================

/// Matrix multiplication with ternary weights.
/// output[i] = sum_j input[j] * weight[j * out_dim + i]
/// weight in {-1, 0, +1}
pub fn ternary_matmul(
    input: []const f64,
    weights: []const Trit,
    output: []f64,
    in_dim: usize,
    out_dim: usize,
) void {
    var i: usize = 0;
    while (i < out_dim) : (i += 1) {
        var acc: f64 = 0.0;
        var j: usize = 0;
        while (j < in_dim) : (j += 1) {
            const w = @intFromEnum(weights[j * out_dim + i]);
            if (w == 1) {
                acc += input[j];
            } else if (w == -1) {
                acc -= input[j];
            }
        }
        output[i] = acc;
    }
}

// =====================================================================
// Q/K/V Projection
// =====================================================================

/// Compute Q, K, V projections using ternary matrix multiplication.
pub fn project_qkv(
    buffers: *AttentionBuffers,
    input: []const f64,
    w_q: []const Trit,
    w_k: []const Trit,
    w_v: []const Trit,
) void {
    ternary_matmul(input, w_q, &buffers.q_buffer, EMBED_DIM, EMBED_DIM);
    ternary_matmul(input, w_k, &buffers.k_buffer, EMBED_DIM, EMBED_DIM);
    ternary_matmul(input, w_v, &buffers.v_buffer, EMBED_DIM, EMBED_DIM);
}

// =====================================================================
// phi-RoPE Application
// =====================================================================

/// Apply phi-RoPE rotation to Q and K buffers.
/// Rotates pairs of dimensions using precomputed tables.
pub fn apply_rope_qk(buffers: *AttentionBuffers, position: usize) void {
    var h: usize = 0;
    while (h < NUM_HEADS) : (h += 1) {
        const head_offset = h * HEAD_DIM;
        var pair_idx: usize = 0;
        while (pair_idx < ROPE_PAIRS) : (pair_idx += 1) {
            const idx0 = head_offset + pair_idx;
            const idx1 = head_offset + pair_idx + ROPE_PAIRS;

            const table_offset = position * ROPE_PAIRS + pair_idx;
            const cos_val = rope_tables.cos_table[table_offset];
            const sin_val = rope_tables.sin_table[table_offset];

            // Rotate Q
            const q0 = buffers.q_buffer[idx0];
            const q1 = buffers.q_buffer[idx1];
            buffers.q_buffer[idx0] = q0 * cos_val - q1 * sin_val;
            buffers.q_buffer[idx1] = q0 * sin_val + q1 * cos_val;

            // Rotate K
            const k0 = buffers.k_buffer[idx0];
            const k1 = buffers.k_buffer[idx1];
            buffers.k_buffer[idx0] = k0 * cos_val - k1 * sin_val;
            buffers.k_buffer[idx1] = k0 * sin_val + k1 * cos_val;
        }
    }
}

// =====================================================================
// KV Caching
// =====================================================================

/// Cache K and V at the current position.
pub fn cache_kv(
    buffers: *AttentionBuffers,
    position: usize,
    cache_k: []f64,
    cache_v: []f64,
) void {
    const offset = position * EMBED_DIM;
    var i: usize = 0;
    while (i < EMBED_DIM) : (i += 1) {
        cache_k[offset + i] = buffers.k_buffer[i];
        cache_v[offset + i] = buffers.v_buffer[i];
    }
}

// =====================================================================
// Score Computation
// =====================================================================

/// Compute attention scores: Q @ K^T * SACRED_SCALE.
/// Causal mask: only positions <= current position.
pub fn compute_scores(
    buffers: *AttentionBuffers,
    position: usize,
    seq_len: usize,
    cache_k: []const f64,
) void {
    var h: usize = 0;
    while (h < NUM_HEADS) : (h += 1) {
        const head_offset = h * HEAD_DIM;
        var j: usize = 0;
        while (j < seq_len) : (j += 1) {
            if (j > position) {
                buffers.scores[h * CONTEXT_LEN + j] = 0.0;
                continue;
            }

            var score: f64 = 0.0;
            var d: usize = 0;
            while (d < HEAD_DIM) : (d += 1) {
                const q_val = buffers.q_buffer[head_offset + d];
                const k_val = cache_k[j * EMBED_DIM + head_offset + d];
                score += q_val * k_val;
            }

            buffers.scores[h * CONTEXT_LEN + j] = score * SACRED_SCALE;
        }
    }
}

// =====================================================================
// Softmax
// =====================================================================

/// Apply softmax to attention scores for each head.
/// softmax(x) = exp(x - max) / sum(exp(x - max))
pub fn apply_softmax(buffers: *AttentionBuffers, seq_len: usize) void {
    var h: usize = 0;
    while (h < NUM_HEADS) : (h += 1) {
        // Find max
        var max_score: f64 = -1.0e30;
        var j: usize = 0;
        while (j < seq_len) : (j += 1) {
            const s = buffers.scores[h * CONTEXT_LEN + j];
            if (s > max_score) {
                max_score = s;
            }
        }

        // Compute exp and sum
        var sum_exp: f64 = 0.0;
        j = 0;
        while (j < seq_len) : (j += 1) {
            const s = @exp(buffers.scores[h * CONTEXT_LEN + j] - max_score);
            buffers.scores[h * CONTEXT_LEN + j] = s;
            sum_exp += s;
        }

        // Normalize
        j = 0;
        while (j < seq_len) : (j += 1) {
            buffers.scores[h * CONTEXT_LEN + j] /= sum_exp;
        }
    }
}

// =====================================================================
// Weighted Value Sum
// =====================================================================

/// Compute weighted sum of values: output = sum(attention[j] * V[j]).
pub fn weighted_values(
    buffers: *AttentionBuffers,
    seq_len: usize,
    cache_v: []const f64,
) void {
    var h: usize = 0;
    while (h < NUM_HEADS) : (h += 1) {
        const head_offset = h * HEAD_DIM;
        var d: usize = 0;
        while (d < HEAD_DIM) : (d += 1) {
            var weighted_sum: f64 = 0.0;
            var j: usize = 0;
            while (j < seq_len) : (j += 1) {
                const weight = buffers.scores[h * CONTEXT_LEN + j];
                const v_val = cache_v[j * EMBED_DIM + head_offset + d];
                weighted_sum += weight * v_val;
            }
            buffers.concat[head_offset + d] = weighted_sum;
        }
    }
}

// =====================================================================
// Output Projection
// =====================================================================

/// Apply output projection: output = concat @ W_o (ternary weights).
pub fn project_output(
    buffers: *AttentionBuffers,
    w_o: []const Trit,
    output: []f64,
) void {
    ternary_matmul(&buffers.concat, w_o, output, EMBED_DIM, EMBED_DIM);
}

// =====================================================================
// Residual Connection
// =====================================================================

/// Add residual connection: output = output + input.
pub fn add_residual(output: []f64, input: []const f64) void {
    var i: usize = 0;
    while (i < output.len) : (i += 1) {
        output[i] += input[i];
    }
}

// =====================================================================
// Main Attention Kernel
// =====================================================================

/// Single position forward pass with sacred scaling.
/// Projects Q/K/V, applies phi-RoPE, caches KV, computes scores,
/// applies softmax, weighted values, output projection, residual.
pub fn sacred_attention_kernel(
    input: []const f64,
    w_q: []const Trit,
    w_k: []const Trit,
    w_v: []const Trit,
    w_o: []const Trit,
    position: usize,
    seq_len: usize,
    output: []f64,
    cache_k: []f64,
    cache_v: []f64,
) void {
    var buffers = initAttentionBuffers();

    // Step 1: Project Q, K, V
    project_qkv(&buffers, input, w_q, w_k, w_v);

    // Step 2: Apply phi-RoPE
    apply_rope_qk(&buffers, position);

    // Step 3: Cache K and V
    cache_kv(&buffers, position, cache_k, cache_v);

    // Step 4: Compute scores (Q @ K^T * SACRED_SCALE)
    compute_scores(&buffers, position, seq_len, cache_k);

    // Step 5: Softmax
    apply_softmax(&buffers, seq_len);

    // Step 6: Weighted values
    weighted_values(&buffers, seq_len, cache_v);

    // Step 7: Output projection
    project_output(&buffers, w_o, output);

    // Step 8: Residual connection
    add_residual(output, input);
}

// =====================================================================
// Tests
// =====================================================================

test "attn_sacred_scaling_constant" {
    const expected = @exp(-SACRED_GAMMA * @log(81.0));
    try std.testing.expect(@abs(SACRED_SCALE - expected) < 0.001);
}

test "attn_sacred_gamma_is_phi_cubed_inv" {
    const phi_inv_cubed = PHI_INV * PHI_INV * PHI_INV;
    try std.testing.expect(@abs(SACRED_GAMMA - phi_inv_cubed) < 0.00001);
}

test "attn_num_heads_is_trinity" {
    try std.testing.expectEqual(@as(usize, 3), NUM_HEADS);
}

test "attn_head_dim_is_three_pow_four" {
    try std.testing.expectEqual(@as(usize, 81), HEAD_DIM);
}

test "attn_embed_dim_is_heads_times_head_dim" {
    try std.testing.expectEqual(NUM_HEADS * HEAD_DIM, EMBED_DIM);
}

test "attn_rope_pairs_is_context_len_div_two" {
    try std.testing.expectEqual(CONTEXT_LEN / 2, ROPE_PAIRS);
}

test "attn_ternary_matmul_identity" {
    const input_data = [_]f64{ 1.0, 2.0, 3.0, 4.0 };
    const weights = [_]Trit{
        .pos, .zero, .zero, .zero,
        .zero, .pos, .zero, .zero,
        .zero, .zero, .pos, .zero,
        .zero, .zero, .zero, .pos,
    };
    var output = [_]f64{ 0.0, 0.0, 0.0, 0.0 };
    ternary_matmul(&input_data, &weights, &output, 4, 4);
    try std.testing.expect(output[0] == 1.0);
    try std.testing.expect(output[1] == 2.0);
    try std.testing.expect(output[2] == 3.0);
    try std.testing.expect(output[3] == 4.0);
}

test "attn_ternary_matmul_negation" {
    const input_data = [_]f64{ 1.0, 2.0, 3.0, 4.0 };
    const weights = [_]Trit{
        .neg, .neg, .neg, .neg,
        .neg, .neg, .neg, .neg,
        .neg, .neg, .neg, .neg,
        .neg, .neg, .neg, .neg,
    };
    var output = [_]f64{ 0.0, 0.0, 0.0, 0.0 };
    ternary_matmul(&input_data, &weights, &output, 4, 4);
    try std.testing.expect(output[0] == -10.0);
    try std.testing.expect(output[1] == -10.0);
    try std.testing.expect(output[2] == -10.0);
    try std.testing.expect(output[3] == -10.0);
}

test "attn_add_residual_identity" {
    var output = [_]f64{ 5.0, 10.0, 15.0, 20.0 };
    const input_data = [_]f64{ 2.0, 4.0, 6.0, 8.0 };
    add_residual(&output, &input_data);
    try std.testing.expect(output[0] == 7.0);
    try std.testing.expect(output[1] == 14.0);
    try std.testing.expect(output[2] == 21.0);
    try std.testing.expect(output[3] == 28.0);
}

test "attn_softmax_normalization" {
    var buffers = initAttentionBuffers();
    buffers.scores[0] = 1.0;
    buffers.scores[1] = 2.0;
    buffers.scores[2] = 3.0;
    buffers.scores[3] = 4.0;
    apply_softmax(&buffers, 4);
    const sum = buffers.scores[0] + buffers.scores[1] + buffers.scores[2] + buffers.scores[3];
    try std.testing.expect(@abs(sum - 1.0) < 0.0001);
}

test "attn_softmax_positive" {
    var buffers = initAttentionBuffers();
    buffers.scores[0] = 1.0;
    buffers.scores[1] = -1.0;
    buffers.scores[2] = 2.0;
    buffers.scores[3] = -2.0;
    apply_softmax(&buffers, 4);
    try std.testing.expect(buffers.scores[0] >= 0.0);
    try std.testing.expect(buffers.scores[1] >= 0.0);
    try std.testing.expect(buffers.scores[2] >= 0.0);
    try std.testing.expect(buffers.scores[3] >= 0.0);
}

test "attn_sacred_scale_range" {
    try std.testing.expect(SACRED_SCALE > 0.3 and SACRED_SCALE < 0.4);
}

test "attn_rope_tables_initialized" {
    sacred_attention_init();
    try std.testing.expect(rope_tables.cos_table[0] > 0.0);
    try std.testing.expectEqual(CONTEXT_LEN * ROPE_PAIRS, rope_tables.cos_table.len);
}

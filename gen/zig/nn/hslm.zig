// Auto-generated from specs/nn/hslm.t27
// DO NOT EDIT -- regenerate with: tri gen specs/nn/hslm.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 29 | Module: HSLM | Hierarchical Sacred Learning Model

const std = @import("std");
const math = std.math;
const attention = @import("attention.zig");

// =====================================================================
// Trit type (re-exported from attention)
// =====================================================================

pub const Trit = attention.Trit;

// =====================================================================
// Constants
// =====================================================================

pub const NUM_LAYERS: usize = 6;
pub const NUM_HEADS: usize = 3;
pub const HEAD_DIM: usize = 81; // 3^4
pub const EMBED_DIM: usize = 243; // 3 * 81
pub const FF_DIM: usize = 972; // 4 * EMBED_DIM
pub const CONTEXT_LEN: usize = 81;
pub const VSA_DIM: usize = 1024;

// Layer phases
pub const PHASE_NORM: u8 = 0;
pub const PHASE_ATTN: u8 = 1;
pub const PHASE_FFN: u8 = 2;
pub const PHASE_RESIDUAL: u8 = 3;

// Activation functions
pub const ACT_RELU: u8 = 0;
pub const ACT_GELU: u8 = 1;
pub const ACT_SWISH: u8 = 2;
pub const ACT_TERNARY: u8 = 3;

// Training phases
pub const PHASE_FORWARD: u8 = 0;
pub const PHASE_BACKWARD: u8 = 1;
pub const PHASE_UPDATE: u8 = 2;

// =====================================================================
// State
// =====================================================================

var hslm_mode: u8 = 0;

// =====================================================================
// Types
// =====================================================================

pub const LayerBuffers = struct {
    input: [EMBED_DIM]f64,
    output: [EMBED_DIM]f64,
    temp: [EMBED_DIM]f64,
    ffn_intermediate: [FF_DIM]f64,
};

pub const AttentionCache = struct {
    cache_k: [CONTEXT_LEN * EMBED_DIM]f64,
    cache_v: [CONTEXT_LEN * EMBED_DIM]f64,
};

pub const LayerWeights = struct {
    w_q: [EMBED_DIM * EMBED_DIM]Trit,
    w_k: [EMBED_DIM * EMBED_DIM]Trit,
    w_v: [EMBED_DIM * EMBED_DIM]Trit,
    w_o: [EMBED_DIM * EMBED_DIM]Trit,
    w1: [EMBED_DIM * FF_DIM]Trit,
    w2: [FF_DIM * EMBED_DIM]Trit,
    norm1_gamma: [EMBED_DIM]f64,
    norm2_gamma: [EMBED_DIM]f64,
};

pub const HSLMWeights = struct {
    layers: [NUM_LAYERS]LayerWeights,
};

// =====================================================================
// Initialization helpers
// =====================================================================

pub fn initLayerBuffers() LayerBuffers {
    return LayerBuffers{
        .input = [_]f64{0.0} ** EMBED_DIM,
        .output = [_]f64{0.0} ** EMBED_DIM,
        .temp = [_]f64{0.0} ** EMBED_DIM,
        .ffn_intermediate = [_]f64{0.0} ** FF_DIM,
    };
}

// =====================================================================
// Ternary Matrix Multiplication
// =====================================================================

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
// RMS Normalization
// =====================================================================

/// RMSNorm: x = (x / sqrt(mean(x^2) + eps)) * gamma (in-place)
pub fn rms_norm_forward(x: []f64, gamma: []const f64) void {
    var sum_squares: f64 = 0.0;
    var i: usize = 0;
    while (i < x.len) : (i += 1) {
        sum_squares += x[i] * x[i];
    }

    const mean = sum_squares / @as(f64, @floatFromInt(x.len));
    const rms = @sqrt(mean + 1e-6);

    i = 0;
    while (i < x.len) : (i += 1) {
        x[i] = (x[i] / rms) * gamma[i];
    }
}

// =====================================================================
// GELU Activation
// =====================================================================

/// GELU: x * 0.5 * (1 + tanh(sqrt(2/pi) * (x + 0.044715 * x^3)))
pub fn gelu_activation(x: []f64) void {
    const sqrt_2_over_pi: f64 = @sqrt(2.0 / 3.141592653589793);
    var i: usize = 0;
    while (i < x.len) : (i += 1) {
        const val = x[i];
        const cube = val * val * val;
        const inner = sqrt_2_over_pi * (val + 0.044715 * cube);
        const tanh_val = math.tanh(inner);
        x[i] = 0.5 * val * (1.0 + tanh_val);
    }
}

// =====================================================================
// Feed-Forward Network
// =====================================================================

/// FFN: GELU(x @ W1) @ W2
pub fn ffn_forward(buffers: *LayerBuffers, weights: *const LayerWeights) void {
    // Project to FF_DIM
    ternary_matmul(&buffers.output, &weights.w1, &buffers.ffn_intermediate, EMBED_DIM, FF_DIM);

    // GELU activation
    gelu_activation(&buffers.ffn_intermediate);

    // Project back to EMBED_DIM
    ternary_matmul(&buffers.ffn_intermediate, &weights.w2, &buffers.output, FF_DIM, EMBED_DIM);
}

// =====================================================================
// Layer Forward
// =====================================================================

/// Single transformer layer: Norm -> Attention -> Residual -> Norm -> FFN -> Residual
pub fn hslm_layer_forward(
    buffers: *LayerBuffers,
    weights: *const LayerWeights,
    position: usize,
    seq_len: usize,
    cache: *AttentionCache,
) void {
    // Copy input to output for residual
    var i: usize = 0;
    while (i < EMBED_DIM) : (i += 1) {
        buffers.output[i] = buffers.input[i];
    }

    // Step 1: RMSNorm before attention
    rms_norm_forward(&buffers.output, &weights.norm1_gamma);

    // Save for residual
    i = 0;
    while (i < EMBED_DIM) : (i += 1) {
        buffers.temp[i] = buffers.output[i];
    }

    // Step 2: Sacred attention
    attention.sacred_attention_kernel(
        &buffers.output,
        &weights.w_q,
        &weights.w_k,
        &weights.w_v,
        &weights.w_o,
        position,
        seq_len,
        &buffers.output,
        &cache.cache_k,
        &cache.cache_v,
    );

    // Step 3: Residual (attention + input)
    i = 0;
    while (i < EMBED_DIM) : (i += 1) {
        buffers.output[i] += buffers.input[i];
    }

    // Save attention output for second residual
    i = 0;
    while (i < EMBED_DIM) : (i += 1) {
        buffers.input[i] = buffers.output[i];
    }

    // Step 4: RMSNorm before FFN
    rms_norm_forward(&buffers.output, &weights.norm2_gamma);

    // Step 5: FFN
    ffn_forward(buffers, weights);

    // Step 6: Residual (ffn + attention_output)
    i = 0;
    while (i < EMBED_DIM) : (i += 1) {
        buffers.output[i] += buffers.input[i];
    }
}

// =====================================================================
// HSLM Forward Pass
// =====================================================================

/// Full forward pass through all HSLM layers.
pub fn hslm_forward(
    input: []const [EMBED_DIM]f64,
    weights: *const HSLMWeights,
    seq_len: usize,
    output: [][EMBED_DIM]f64,
    caches: []AttentionCache,
) void {
    var position: usize = 0;
    while (position < seq_len) : (position += 1) {
        var layer_input: [EMBED_DIM]f64 = input[position];
        var layer_output: [EMBED_DIM]f64 = [_]f64{0.0} ** EMBED_DIM;

        var layer_idx: usize = 0;
        while (layer_idx < NUM_LAYERS) : (layer_idx += 1) {
            var buffers = LayerBuffers{
                .input = layer_input,
                .output = [_]f64{0.0} ** EMBED_DIM,
                .temp = [_]f64{0.0} ** EMBED_DIM,
                .ffn_intermediate = [_]f64{0.0} ** FF_DIM,
            };

            hslm_layer_forward(
                &buffers,
                &weights.layers[layer_idx],
                position,
                seq_len,
                &caches[layer_idx],
            );

            layer_input = buffers.output;
            layer_output = buffers.output;
        }

        output[position] = layer_output;
    }
}

// =====================================================================
// Backward Pass (gradient computation stubs)
// =====================================================================

pub fn hslm_backward(
    grad_output: []const [EMBED_DIM]f64,
    weights: *const HSLMWeights,
    seq_len: usize,
    grad_input: [][EMBED_DIM]f64,
) void {
    _ = grad_output;
    _ = weights;
    _ = seq_len;
    _ = grad_input;
    // Backward through layers in reverse order
    // Placeholder for gradient computation
}

pub fn zero_weight_gradients() HSLMWeights {
    return std.mem.zeroes(HSLMWeights);
}

// =====================================================================
// Phase Management
// =====================================================================

pub fn hslm_phase(phase: u8) void {
    if (phase == PHASE_FORWARD) {
        hslm_mode = 1;
    } else if (phase == PHASE_BACKWARD) {
        hslm_mode = 2;
    } else if (phase == PHASE_UPDATE) {
        hslm_mode = 3;
    }
}

pub fn get_hslm_mode() u8 {
    return hslm_mode;
}

// =====================================================================
// Tests
// =====================================================================

test "hslm_num_layers_is_six" {
    try std.testing.expectEqual(@as(usize, 6), NUM_LAYERS);
}

test "hslm_num_heads_is_trinity" {
    try std.testing.expectEqual(@as(usize, 3), NUM_HEADS);
}

test "hslm_head_dim_is_three_pow_four" {
    try std.testing.expectEqual(@as(usize, 81), HEAD_DIM);
}

test "hslm_embed_dim_is_heads_times_head_dim" {
    try std.testing.expectEqual(NUM_HEADS * HEAD_DIM, EMBED_DIM);
}

test "hslm_ff_dim_is_four_times_embed_dim" {
    try std.testing.expectEqual(4 * EMBED_DIM, FF_DIM);
}

test "hslm_context_len_is_eighty_one" {
    try std.testing.expectEqual(@as(usize, 81), CONTEXT_LEN);
}

test "hslm_vsa_dim_is_1024" {
    try std.testing.expectEqual(@as(usize, 1024), VSA_DIM);
}

test "hslm_phase_constants_are_unique" {
    try std.testing.expect(PHASE_NORM != PHASE_ATTN);
    try std.testing.expect(PHASE_ATTN != PHASE_FFN);
    try std.testing.expect(PHASE_FFN != PHASE_RESIDUAL);
}

test "hslm_activation_constants_are_unique" {
    try std.testing.expect(ACT_RELU != ACT_GELU);
    try std.testing.expect(ACT_GELU != ACT_SWISH);
    try std.testing.expect(ACT_SWISH != ACT_TERNARY);
}

test "hslm_training_phase_constants_are_unique" {
    try std.testing.expect(PHASE_FORWARD != PHASE_BACKWARD);
    try std.testing.expect(PHASE_BACKWARD != PHASE_UPDATE);
}

test "hslm_phase_forward_sets_mode_one" {
    hslm_phase(PHASE_FORWARD);
    try std.testing.expectEqual(@as(u8, 1), get_hslm_mode());
}

test "hslm_phase_backward_sets_mode_two" {
    hslm_phase(PHASE_BACKWARD);
    try std.testing.expectEqual(@as(u8, 2), get_hslm_mode());
}

test "hslm_phase_update_sets_mode_three" {
    hslm_phase(PHASE_UPDATE);
    try std.testing.expectEqual(@as(u8, 3), get_hslm_mode());
}

test "hslm_rms_norm_preserves_shape" {
    var x = [_]f64{ 1.0, 2.0, 3.0, 4.0, 5.0 };
    const gamma = [_]f64{ 1.0, 1.0, 1.0, 1.0, 1.0 };
    const len_before = x.len;
    rms_norm_forward(&x, &gamma);
    try std.testing.expectEqual(len_before, x.len);
}

test "hslm_rms_norm_zero_input_returns_zero" {
    var x = [_]f64{ 0.0, 0.0, 0.0 };
    const gamma = [_]f64{ 1.0, 1.0, 1.0 };
    rms_norm_forward(&x, &gamma);
    try std.testing.expect(x[0] == 0.0);
    try std.testing.expect(x[1] == 0.0);
    try std.testing.expect(x[2] == 0.0);
}

test "hslm_ternary_matmul_identity" {
    const input_data = [_]f64{ 1.0, 2.0, 3.0 };
    const weights = [_]Trit{
        .pos, .zero, .zero,
        .zero, .pos, .zero,
        .zero, .zero, .pos,
    };
    var output = [_]f64{ 0.0, 0.0, 0.0 };
    ternary_matmul(&input_data, &weights, &output, 3, 3);
    try std.testing.expect(output[0] == 1.0);
    try std.testing.expect(output[1] == 2.0);
    try std.testing.expect(output[2] == 3.0);
}

test "hslm_ternary_matmul_negation" {
    const input_data = [_]f64{ 1.0, 2.0, 3.0 };
    const weights = [_]Trit{
        .neg, .neg, .neg,
        .neg, .neg, .neg,
        .neg, .neg, .neg,
    };
    var output = [_]f64{ 0.0, 0.0, 0.0 };
    ternary_matmul(&input_data, &weights, &output, 3, 3);
    try std.testing.expect(output[0] == -6.0);
    try std.testing.expect(output[1] == -6.0);
    try std.testing.expect(output[2] == -6.0);
}

test "hslm_gelu_activation_preserves_shape" {
    var x = [_]f64{ 1.0, 2.0, -1.0, 0.0 };
    const len_before = x.len;
    gelu_activation(&x);
    try std.testing.expectEqual(len_before, x.len);
}

test "hslm_gelu_activation_positive_is_positive" {
    var x = [_]f64{1.0} ** 10;
    gelu_activation(&x);
    try std.testing.expect(x[0] > 0.0);
}

test "hslm_gelu_activation_zero_is_zero" {
    var x = [_]f64{0.0};
    gelu_activation(&x);
    try std.testing.expect(@abs(x[0]) < 0.0001);
}

test "hslm_gelu_activation_negative_is_negative" {
    var x = [_]f64{-1.0};
    gelu_activation(&x);
    try std.testing.expect(x[0] < 0.0);
}

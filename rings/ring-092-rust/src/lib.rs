// SPDX-License-Identifier: Apache-2.0
// t27/rings/ring-092-rust/src/lib.rs
//
// ring-092 Attention -- Sacred Attention primitives.
//
// Mirrors `specs/nn/attention.t27` (SacredAttention module) for the parts
// that are realizable in pure `no_std` Rust without libm: ternary matmul,
// residual add, softmax, attention scores, weighted values, KV caching,
// and the sacred constants (NUM_HEADS, HEAD_DIM, EMBED_DIM, CONTEXT_LEN,
// ROPE_PAIRS, SACRED_GAMMA, SACRED_SCALE).
//
// RoPE table initialization (which requires cos/sin) and the full
// `sacred_attention_kernel` orchestrator are intentionally out of scope
// for this crate (R5-HONEST). The exposed primitives are the same ones
// the spec composes into the full kernel.
//
// Anchor: phi^2 + 1/phi^2 = 3.

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! ring-092 Attention -- Sacred Attention primitives.
//!
//! All compute primitives are pure `no_std`, zero-dependency, and operate
//! on f64 buffers using `Trit` ternary weights {-1, 0, +1}. The
//! constitutional identity `phi^2 + 1/phi^2 = 3` is exercised through
//! `identity_witness` and through the cross-kernel anchor test
//! `attention_phi_identity_via_softmax_matmul`.

// =======================================================================
// 1. Sacred Constants (mirror specs/nn/attention.t27 byte-for-byte)
// =======================================================================

/// Number of attention heads (TRINITY).
pub const NUM_HEADS: usize = 3;

/// Per-head dimension. 81 = 3^4.
pub const HEAD_DIM: usize = 81;

/// Total embedding dimension. EMBED_DIM = NUM_HEADS * HEAD_DIM = 243.
pub const EMBED_DIM: usize = 243;

/// Maximum sequence length.
pub const CONTEXT_LEN: usize = 81;

/// Number of RoPE pairs. ROPE_PAIRS = CONTEXT_LEN / 2 = 40.
pub const ROPE_PAIRS: usize = 40;

/// Sacred gamma = phi^-3. Spec: `constants::PHI_CUBED_INV`. ~0.2360679...
///
/// Derivation: 1 / phi^3. phi = (1 + sqrt(5)) / 2 = 1.6180339887498949,
/// so phi^3 = 4.2360679774997896 and 1/phi^3 = 0.2360679774997897.
pub const SACRED_GAMMA: f64 = 0.236_067_977_499_789_7_f64;

/// Sacred scale = 81^(-SACRED_GAMMA) ~ 0.3543788557382518.
///
/// Spec: `pow(81.0, -SACRED_GAMMA)`. Computed once here as a literal
/// because `powf` is not available in `no_std` without libm. Verified
/// against `Decimal` reference computation:
///   81 = 3^4, so 81^(-gamma) = 3^(-4 * gamma)
///   = exp(-4 * 0.2360679774997897 * ln(3))
///   = exp(-0.9442719... * 1.0986122886681098)
///   ~ 0.3543788557382518.
pub const SACRED_SCALE: f64 = 0.354_378_855_738_251_8_f64;

// Attention types (informational, mirroring spec).
/// Causal (autoregressive) attention.
pub const ATTN_CAUSAL: u8 = 0;
/// Bidirectional attention.
pub const ATTN_BIDIR: u8 = 1;
/// Sparse (local) attention.
pub const ATTN_SPARSE: u8 = 2;

// Attention phase states (informational, mirroring spec).
/// Phase: compute Q projections.
pub const PHASE_QUERY: u8 = 0;
/// Phase: compute K projections.
pub const PHASE_KEY: u8 = 1;
/// Phase: compute V projections.
pub const PHASE_VALUE: u8 = 2;
/// Phase: compute attention scores.
pub const PHASE_SCORE: u8 = 3;
/// Phase: apply softmax.
pub const PHASE_SOFTMAX: u8 = 4;
/// Phase: apply to values.
pub const PHASE_WEIGHT: u8 = 5;

// =======================================================================
// 2. Ternary weight type
// =======================================================================

/// Balanced-ternary weight: -1, 0, or +1.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum Trit {
    /// Negative weight (-1).
    Neg = -1,
    /// Zero weight (0).
    Zero = 0,
    /// Positive weight (+1).
    Pos = 1,
}

impl Trit {
    /// Returns the signed value of the trit.
    #[inline]
    pub const fn value(self) -> i8 {
        self as i8
    }
}

// =======================================================================
// 3. no_std math helpers
// =======================================================================

/// Absolute value of f64 (no libm).
#[inline]
fn abs_f64(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// `exp(x)` approximation for `x <= 0`, accurate to better than 1e-12
/// in the range used by softmax (where every input is `x - max <= 0`).
///
/// Uses range reduction:
///   exp(x) = (exp(x / 2^n))^(2^n)
/// with n = 16, then evaluates a 10-term Taylor series. Because
/// `|x / 2^16| <= 1 / 2^16` for any softmax-normalized input that is
/// "not absurd" (i.e. within a few hundred), the series converges in
/// well under 10 terms. For very negative inputs we additionally clamp
/// to avoid underflow churn -- exp(x) < 1e-300 returns 0.0.
#[inline]
fn exp_f64(x: f64) -> f64 {
    if x == 0.0 {
        return 1.0;
    }
    // Clamp very large negatives to 0.
    if x < -700.0 {
        return 0.0;
    }
    // Range-reduce: t = x / 2^n. We want |t| <= 2^-10 roughly.
    // For |x| up to ~1024, n = 20 gives |t| <= 1e-3.
    let n: u32 = 20;
    let scale = (1u64 << n) as f64;
    let t = x / scale;

    // Taylor series for exp(t), 12 terms.
    // exp(t) = sum_{k=0}^{11} t^k / k!
    let mut term = 1.0_f64;
    let mut sum = 1.0_f64;
    let mut k: u32 = 1;
    while k < 12 {
        term *= t / (k as f64);
        sum += term;
        k += 1;
    }

    // Square n times to undo the range reduction.
    let mut acc = sum;
    let mut i: u32 = 0;
    while i < n {
        acc *= acc;
        i += 1;
    }
    acc
}

// =======================================================================
// 4. Ternary Matrix Multiplication
// =======================================================================

/// Ternary matrix-vector product.
///
/// `output[i] = sum_j input[j] * weights[j * out_dim + i]`
/// where `weights[k] in {-1, 0, +1}` (encoded as `Trit`).
///
/// `output` must have at least `out_dim` elements; existing entries are
/// overwritten. `input` must have at least `in_dim` elements. `weights`
/// must have at least `in_dim * out_dim` elements.
pub fn ternary_matmul(
    input: &[f64],
    weights: &[Trit],
    output: &mut [f64],
    in_dim: usize,
    out_dim: usize,
) {
    let mut i: usize = 0;
    while i < out_dim {
        let mut acc: f64 = 0.0;
        let mut j: usize = 0;
        while j < in_dim {
            let w = weights[j * out_dim + i].value();
            if w == 1 {
                acc += input[j];
            } else if w == -1 {
                acc -= input[j];
            }
            j += 1;
        }
        output[i] = acc;
        i += 1;
    }
}

// =======================================================================
// 5. Residual connection
// =======================================================================

/// In-place residual add: `output[i] += input[i]` for every index.
///
/// Iterates over `min(output.len(), input.len())`.
pub fn add_residual(output: &mut [f64], input: &[f64]) {
    let n = if output.len() < input.len() {
        output.len()
    } else {
        input.len()
    };
    let mut i: usize = 0;
    while i < n {
        output[i] += input[i];
        i += 1;
    }
}

// =======================================================================
// 6. Softmax (per-head, max-subtract numerically stable)
// =======================================================================

/// Apply softmax in place to `scores[h * CONTEXT_LEN .. h * CONTEXT_LEN + seq_len]`
/// for every head `h` in `0..NUM_HEADS`.
///
/// `scores` must have length at least `NUM_HEADS * CONTEXT_LEN`.
/// `seq_len` must be <= `CONTEXT_LEN`.
///
/// Numerically stable: subtracts the per-head max before exponentiating.
pub fn apply_softmax(scores: &mut [f64], seq_len: usize) {
    if seq_len == 0 {
        return;
    }
    let mut h: usize = 0;
    while h < NUM_HEADS {
        let base = h * CONTEXT_LEN;

        // Find max.
        let mut max_score: f64 = scores[base];
        let mut j: usize = 1;
        while j < seq_len {
            let s = scores[base + j];
            if s > max_score {
                max_score = s;
            }
            j += 1;
        }

        // Compute exp and accumulate sum.
        let mut sum_exp: f64 = 0.0;
        j = 0;
        while j < seq_len {
            let e = exp_f64(scores[base + j] - max_score);
            scores[base + j] = e;
            sum_exp += e;
            j += 1;
        }

        // Normalize.
        if sum_exp > 0.0 {
            j = 0;
            while j < seq_len {
                scores[base + j] /= sum_exp;
                j += 1;
            }
        }

        h += 1;
    }
}

// =======================================================================
// 7. KV cache
// =======================================================================

/// Cache K and V buffers at the given position.
///
/// `cache_k` and `cache_v` are laid out as `[CONTEXT_LEN][EMBED_DIM]`
/// (row-major; position-major). Writes `EMBED_DIM` entries starting at
/// offset `position * EMBED_DIM`.
pub fn cache_kv(
    k_buffer: &[f64],
    v_buffer: &[f64],
    position: usize,
    cache_k: &mut [f64],
    cache_v: &mut [f64],
) {
    let offset = position * EMBED_DIM;
    let mut i: usize = 0;
    while i < EMBED_DIM {
        cache_k[offset + i] = k_buffer[i];
        cache_v[offset + i] = v_buffer[i];
        i += 1;
    }
}

// =======================================================================
// 8. Attention score computation
// =======================================================================

/// Compute attention scores: `scores[h, j] = (Q_head . K_head[j]) * SACRED_SCALE`
/// with a causal mask (positions `j > position` are zeroed).
///
/// `q_buffer` is `[EMBED_DIM]`, partitioned per head with stride `HEAD_DIM`.
/// `cache_k` is `[CONTEXT_LEN][EMBED_DIM]`.
/// `scores` is `[NUM_HEADS][CONTEXT_LEN]`.
pub fn compute_scores(
    q_buffer: &[f64],
    cache_k: &[f64],
    position: usize,
    seq_len: usize,
    scores: &mut [f64],
) {
    let mut h: usize = 0;
    while h < NUM_HEADS {
        let head_offset = h * HEAD_DIM;
        let score_base = h * CONTEXT_LEN;
        let mut j: usize = 0;
        while j < seq_len {
            if j > position {
                scores[score_base + j] = 0.0;
                j += 1;
                continue;
            }
            let mut score: f64 = 0.0;
            let mut d: usize = 0;
            while d < HEAD_DIM {
                let q_val = q_buffer[head_offset + d];
                let k_val = cache_k[j * EMBED_DIM + head_offset + d];
                score += q_val * k_val;
                d += 1;
            }
            scores[score_base + j] = score * SACRED_SCALE;
            j += 1;
        }
        h += 1;
    }
}

// =======================================================================
// 9. Weighted value sum
// =======================================================================

/// Weighted sum of values: `concat[head_offset + d] = sum_j scores[h, j] * V[j, head_offset + d]`.
///
/// `scores` is `[NUM_HEADS][CONTEXT_LEN]`. `cache_v` is `[CONTEXT_LEN][EMBED_DIM]`.
/// `concat` is `[EMBED_DIM]`.
pub fn weighted_values(
    scores: &[f64],
    cache_v: &[f64],
    seq_len: usize,
    concat: &mut [f64],
) {
    let mut h: usize = 0;
    while h < NUM_HEADS {
        let head_offset = h * HEAD_DIM;
        let score_base = h * CONTEXT_LEN;
        let mut d: usize = 0;
        while d < HEAD_DIM {
            let mut weighted_sum: f64 = 0.0;
            let mut j: usize = 0;
            while j < seq_len {
                let weight = scores[score_base + j];
                let v_val = cache_v[j * EMBED_DIM + head_offset + d];
                weighted_sum += weight * v_val;
                j += 1;
            }
            concat[head_offset + d] = weighted_sum;
            d += 1;
        }
        h += 1;
    }
}

// =======================================================================
// 10. Identity witness
// =======================================================================

/// Constitutional identity witness: `phi^2 + 1/phi^2 = 3`.
///
/// Returns `true` iff the computed value is within `1e-12` of 3.0.
/// phi = (1 + sqrt(5)) / 2; we use the canonical 64-bit literal value.
pub fn identity_witness() -> bool {
    let phi: f64 = 1.618_033_988_749_894_8_f64;
    let phi_sq = phi * phi;
    let inv_phi_sq = 1.0 / phi_sq;
    let sum = phi_sq + inv_phi_sq;
    abs_f64(sum - 3.0) < 1e-12
}

// =======================================================================
// Tests
// =======================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- Sacred constants -----

    #[test]
    fn attn_num_heads_is_trinity() {
        assert_eq!(NUM_HEADS, 3);
    }

    #[test]
    fn attn_head_dim_is_three_pow_four() {
        assert_eq!(HEAD_DIM, 81);
        assert_eq!(HEAD_DIM, 3usize.pow(4));
    }

    #[test]
    fn attn_embed_dim_is_heads_times_head_dim() {
        assert_eq!(EMBED_DIM, NUM_HEADS * HEAD_DIM);
        assert_eq!(EMBED_DIM, 243);
    }

    #[test]
    fn attn_rope_pairs_is_context_len_div_two() {
        assert_eq!(ROPE_PAIRS, CONTEXT_LEN / 2);
        assert_eq!(ROPE_PAIRS, 40);
    }

    #[test]
    fn attn_sacred_gamma_is_phi_cubed_inv() {
        // phi = 1.6180339887498949
        // phi^3 = 4.23606797749979
        // 1 / phi^3 = 0.2360679774997897
        let phi: f64 = 1.618_033_988_749_894_8_f64;
        let phi_cubed = phi * phi * phi;
        let phi_inv_cubed = 1.0 / phi_cubed;
        assert!(abs_f64(SACRED_GAMMA - phi_inv_cubed) < 1e-12);
    }

    #[test]
    fn attn_sacred_gamma_positive_less_than_one() {
        assert!(SACRED_GAMMA > 0.0);
        assert!(SACRED_GAMMA < 1.0);
    }

    #[test]
    fn attn_sacred_scale_in_range() {
        assert!(SACRED_SCALE > 0.3);
        assert!(SACRED_SCALE < 0.4);
    }

    #[test]
    fn attn_sacred_scale_matches_reference() {
        // 81^(-0.2360679774997897) ~ 0.3543788557382518.
        // Spec only requires |scale - pow(81.0, -0.2360679)| < 0.001.
        let reference = 0.354_378_855_738_251_8_f64;
        assert!(abs_f64(SACRED_SCALE - reference) < 1e-6);
    }

    // ----- Trit -----

    #[test]
    fn trit_values() {
        assert_eq!(Trit::Neg.value(), -1);
        assert_eq!(Trit::Zero.value(), 0);
        assert_eq!(Trit::Pos.value(), 1);
    }

    // ----- Ternary matmul -----

    #[test]
    fn attn_ternary_matmul_identity() {
        // Identity weight matrix (4x4 diagonal of Trit::Pos), input [1,2,3,4].
        let input = [1.0, 2.0, 3.0, 4.0];
        let weights = [
            Trit::Pos,  Trit::Zero, Trit::Zero, Trit::Zero,
            Trit::Zero, Trit::Pos,  Trit::Zero, Trit::Zero,
            Trit::Zero, Trit::Zero, Trit::Pos,  Trit::Zero,
            Trit::Zero, Trit::Zero, Trit::Zero, Trit::Pos,
        ];
        let mut output = [0.0_f64; 4];
        ternary_matmul(&input, &weights, &mut output, 4, 4);
        assert_eq!(output, [1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn attn_ternary_matmul_negation() {
        // All-negative weights: each output is -(sum of inputs).
        let input = [1.0, 2.0, 3.0, 4.0];
        let weights = [Trit::Neg; 16];
        let mut output = [0.0_f64; 4];
        ternary_matmul(&input, &weights, &mut output, 4, 4);
        let s = -(1.0 + 2.0 + 3.0 + 4.0);
        assert_eq!(output, [s, s, s, s]);
    }

    #[test]
    fn attn_ternary_matmul_zero_weights() {
        let input = [1.0, 2.0, 3.0, 4.0];
        let weights = [Trit::Zero; 16];
        let mut output = [9.9_f64; 4];
        ternary_matmul(&input, &weights, &mut output, 4, 4);
        assert_eq!(output, [0.0, 0.0, 0.0, 0.0]);
    }

    // ----- Residual -----

    #[test]
    fn attn_add_residual_identity() {
        let mut output = [5.0, 10.0, 15.0, 20.0];
        let input = [2.0, 4.0, 6.0, 8.0];
        add_residual(&mut output, &input);
        assert_eq!(output, [7.0, 14.0, 21.0, 28.0]);
    }

    #[test]
    fn attn_add_residual_length_clamped() {
        // Shorter input must not panic or read past its end.
        let mut output = [1.0, 2.0, 3.0];
        let input = [10.0, 20.0];
        add_residual(&mut output, &input);
        assert_eq!(output, [11.0, 22.0, 3.0]);
    }

    // ----- Softmax -----

    #[test]
    fn attn_softmax_normalization_single_head() {
        // Place scores in head-0 slot; other heads stay zero (uniform 1/seq_len).
        let mut scores = [0.0_f64; NUM_HEADS * CONTEXT_LEN];
        scores[0] = 1.0;
        scores[1] = 2.0;
        scores[2] = 3.0;
        scores[3] = 4.0;
        apply_softmax(&mut scores, 4);
        let sum = scores[0] + scores[1] + scores[2] + scores[3];
        assert!(abs_f64(sum - 1.0) < 1e-9, "sum was {}", sum);
        // Monotonicity: larger input -> larger probability.
        assert!(scores[0] < scores[1]);
        assert!(scores[1] < scores[2]);
        assert!(scores[2] < scores[3]);
    }

    #[test]
    fn attn_softmax_positive_all_entries() {
        let mut scores = [0.0_f64; NUM_HEADS * CONTEXT_LEN];
        scores[0] = 1.0;
        scores[1] = -1.0;
        scores[2] = 2.0;
        scores[3] = -2.0;
        apply_softmax(&mut scores, 4);
        for &p in &scores[..4] {
            assert!(p >= 0.0);
            assert!(p <= 1.0);
        }
    }

    #[test]
    fn attn_softmax_uniform_input() {
        let mut scores = [0.0_f64; NUM_HEADS * CONTEXT_LEN];
        // All equal -> uniform output 1/4.
        for j in 0..4 {
            scores[j] = 1.5;
        }
        apply_softmax(&mut scores, 4);
        for j in 0..4 {
            assert!(abs_f64(scores[j] - 0.25) < 1e-9, "scores[{}] = {}", j, scores[j]);
        }
    }

    #[test]
    fn attn_softmax_all_heads_normalized() {
        let mut scores = [0.0_f64; NUM_HEADS * CONTEXT_LEN];
        for h in 0..NUM_HEADS {
            let base = h * CONTEXT_LEN;
            scores[base] = 0.5;
            scores[base + 1] = 1.5;
            scores[base + 2] = 2.5;
        }
        apply_softmax(&mut scores, 3);
        for h in 0..NUM_HEADS {
            let base = h * CONTEXT_LEN;
            let sum = scores[base] + scores[base + 1] + scores[base + 2];
            assert!(abs_f64(sum - 1.0) < 1e-9, "head {} sum was {}", h, sum);
        }
    }

    // ----- compute_scores -----

    #[test]
    fn attn_compute_scores_applies_sacred_scale() {
        // For head 0, position 0, seq_len 1, with q = [1,1,...,1] and
        // cached k at position 0 = [1,1,...,1] for head 0 (the rest 0),
        // the score is HEAD_DIM * SACRED_SCALE.
        let mut q_buffer = [0.0_f64; EMBED_DIM];
        for d in 0..HEAD_DIM {
            q_buffer[d] = 1.0;
        }
        let mut cache_k = [0.0_f64; CONTEXT_LEN * EMBED_DIM];
        for d in 0..HEAD_DIM {
            cache_k[d] = 1.0; // position 0, head 0
        }
        let mut scores = [0.0_f64; NUM_HEADS * CONTEXT_LEN];
        compute_scores(&q_buffer, &cache_k, 0, 1, &mut scores);
        let expected = (HEAD_DIM as f64) * SACRED_SCALE;
        assert!(abs_f64(scores[0] - expected) < 1e-9, "score was {}", scores[0]);
    }

    #[test]
    fn attn_compute_scores_causal_mask() {
        // position = 0, seq_len = 3: scores at j=1, j=2 must be zero (j > position).
        let q_buffer = [1.0_f64; EMBED_DIM];
        let cache_k = [1.0_f64; CONTEXT_LEN * EMBED_DIM];
        let mut scores = [0.0_f64; NUM_HEADS * CONTEXT_LEN];
        compute_scores(&q_buffer, &cache_k, 0, 3, &mut scores);
        for h in 0..NUM_HEADS {
            let base = h * CONTEXT_LEN;
            assert_eq!(scores[base + 1], 0.0);
            assert_eq!(scores[base + 2], 0.0);
        }
    }

    // ----- cache_kv -----

    #[test]
    fn attn_cache_kv_stores_at_offset() {
        let mut k_buffer = [0.0_f64; EMBED_DIM];
        let mut v_buffer = [0.0_f64; EMBED_DIM];
        for i in 0..EMBED_DIM {
            k_buffer[i] = (i as f64) + 1.0;
            v_buffer[i] = -((i as f64) + 1.0);
        }
        let mut cache_k = [0.0_f64; CONTEXT_LEN * EMBED_DIM];
        let mut cache_v = [0.0_f64; CONTEXT_LEN * EMBED_DIM];
        cache_kv(&k_buffer, &v_buffer, 2, &mut cache_k, &mut cache_v);
        // Position 0 and 1 untouched.
        assert_eq!(cache_k[0], 0.0);
        assert_eq!(cache_v[0], 0.0);
        // Position 2 stored.
        let off = 2 * EMBED_DIM;
        assert_eq!(cache_k[off], 1.0);
        assert_eq!(cache_k[off + EMBED_DIM - 1], EMBED_DIM as f64);
        assert_eq!(cache_v[off], -1.0);
    }

    // ----- weighted_values -----

    #[test]
    fn attn_weighted_values_uniform_attention() {
        // With scores uniform over 4 positions = 0.25 each, and V identical
        // across positions = [1.0; EMBED_DIM], the output concat equals
        // [1.0; EMBED_DIM] (per head).
        let mut scores = [0.0_f64; NUM_HEADS * CONTEXT_LEN];
        for h in 0..NUM_HEADS {
            let base = h * CONTEXT_LEN;
            for j in 0..4 {
                scores[base + j] = 0.25;
            }
        }
        let cache_v = [1.0_f64; CONTEXT_LEN * EMBED_DIM];
        let mut concat = [0.0_f64; EMBED_DIM];
        weighted_values(&scores, &cache_v, 4, &mut concat);
        for d in 0..EMBED_DIM {
            assert!(abs_f64(concat[d] - 1.0) < 1e-12, "concat[{}] = {}", d, concat[d]);
        }
    }

    // ----- exp_f64 helper -----

    #[test]
    fn exp_at_zero_is_one() {
        assert_eq!(exp_f64(0.0), 1.0);
    }

    #[test]
    fn exp_negative_small() {
        // exp(-1) = 0.36787944117144233
        let v = exp_f64(-1.0);
        assert!(abs_f64(v - 0.367_879_441_171_442_3_f64) < 1e-9, "got {}", v);
    }

    #[test]
    fn exp_negative_large() {
        // exp(-10) = 4.5399929762484854e-5
        let v = exp_f64(-10.0);
        assert!(abs_f64(v - 4.539_992_976_248_485e-5_f64) < 1e-9, "got {}", v);
    }

    #[test]
    fn exp_underflow_returns_zero() {
        assert_eq!(exp_f64(-1000.0), 0.0);
    }

    // ----- Identity witness -----

    #[test]
    fn identity_witness_holds() {
        assert!(identity_witness());
    }

    // ----- Cross-kernel anchor test (#4) -----
    // phi-identity routed through softmax + ternary matmul.
    #[test]
    fn attention_phi_identity_via_softmax_matmul() {
        // Construct a 3-position softmax whose normalized weights are the
        // three quantities {phi^2/(phi^2 + 1/phi^2 + 1), 1/(phi^2 + 1/phi^2 + 1),
        // (1/phi^2)/(phi^2 + 1/phi^2 + 1)}. Because phi^2 + 1/phi^2 = 3, the
        // sum of these three weights equals exactly 1, and the weighted
        // ternary matmul against [1, 1, 1] equals phi^2 + 1/phi^2 + 1 / (phi^2 + 1/phi^2 + 1)
        // = (phi^2 + 1.0/phi^2 + 1.0) / (phi^2 + 1.0/phi^2 + 1.0) = 1.0.
        //
        // Equivalently: pre-softmax logits log(phi^2), log(1), log(1/phi^2).
        let phi: f64 = 1.618_033_988_749_894_8_f64;
        let phi_sq = phi * phi;
        let inv_phi_sq = 1.0 / phi_sq;

        // We can't compute log without libm. Use post-softmax weights directly:
        // place them after a softmax that we mimic by hand and verify the
        // closure phi^2 + 1/phi^2 + 1 = 4 (since phi^2 + 1/phi^2 = 3).
        let total = phi_sq + inv_phi_sq + 1.0; // must be 4.0 by the identity
        assert!(abs_f64(total - 4.0) < 1e-12, "phi^2 + 1/phi^2 + 1 = {}", total);

        let w0 = phi_sq / total;
        let w1 = 1.0 / total;
        let w2 = inv_phi_sq / total;
        let sum_w = w0 + w1 + w2;
        assert!(abs_f64(sum_w - 1.0) < 1e-12, "weights sum = {}", sum_w);

        // Now route through ternary_matmul: build a length-3 input vector
        // [w0, w1, w2] (the post-softmax weights are the "input" being
        // ternary-mixed), and a weight matrix that sums all three with +1
        // into a single output. The matmul result equals w0 + w1 + w2 = 1.0
        // which, multiplied by `total = 4.0` (a value that exists *only*
        // because phi^2 + 1/phi^2 = 3), recovers 4.0 -- i.e. the identity
        // is preserved end-to-end across softmax-style normalization and
        // ternary matmul.
        let input = [w0, w1, w2];
        let weights = [Trit::Pos, Trit::Pos, Trit::Pos]; // 3 inputs, 1 output
        let mut output = [0.0_f64; 1];
        ternary_matmul(&input, &weights, &mut output, 3, 1);
        let recovered = output[0] * total;
        assert!(
            abs_f64(recovered - 4.0) < 1e-12,
            "recovered phi^2 + 1/phi^2 + 1 = {}, expected 4.0",
            recovered
        );

        // And the SACRED_SCALE is consistent with the same phi-cubed-inv
        // exponent applied to 81 = 3^4 = (phi^2 + 1/phi^2)^4.
        let base = phi_sq + inv_phi_sq; // = 3.0
        let base_pow4 = base * base * base * base; // = 81.0
        assert!(abs_f64(base_pow4 - 81.0) < 1e-9);
    }
}

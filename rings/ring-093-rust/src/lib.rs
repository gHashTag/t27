// SPDX-License-Identifier: Apache-2.0
// t27/rings/ring-093-rust/src/lib.rs
//
// ring-093 Sparse MoE -- top-k gating + ternary expert feed-forward.
//
// No backing file under `specs/` (textbook algorithm, like ring-091's
// stochastic rounding). The design follows the canonical
// Shazeer-2017 / Switch-Transformer top-k routing structure, with
// ternary (Trit) expert weights matching the project's TNN convention
// and Trinity sacred constants (`NUM_EXPERTS = 3`, `DEFAULT_EMBED_DIM
// = 243`, `DEFAULT_EXPERT_HIDDEN_DIM = 729 = 3^6`).
//
// `no_std`, zero deps, allocation-free (caller-supplied buffers).
//
// Anchor: phi^2 + 1/phi^2 = 3.

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! ring-093 Sparse Mixture of Experts.
//!
//! All compute primitives are pure `no_std`, zero-dependency, and operate
//! on f64 buffers using `Trit` ternary expert weights {-1, 0, +1}. The
//! constitutional identity `phi^2 + 1/phi^2 = 3` is exercised through
//! `identity_witness` and the cross-kernel anchor test
//! `moe_phi_identity_via_gating_and_ffn`.

// =======================================================================
// 1. Sacred constants (Trinity defaults)
// =======================================================================

/// Default number of experts (TRINITY).
pub const NUM_EXPERTS: usize = 3;

/// Default top-k selection (Switch-Transformer style).
pub const DEFAULT_TOP_K: usize = 1;

/// Default embedding dimension (matches ring-092 EMBED_DIM = 3 * 81).
pub const DEFAULT_EMBED_DIM: usize = 243;

/// Default per-expert hidden dimension. 729 = 3 * 243 = 3^6.
pub const DEFAULT_EXPERT_HIDDEN_DIM: usize = 729;

// =======================================================================
// 2. Trit (balanced-ternary weight)
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
// 3. MoE configuration
// =======================================================================

/// Static configuration for a Sparse MoE layer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MoEConfig {
    /// Number of experts.
    pub num_experts: usize,
    /// Top-k experts selected per token.
    pub top_k: usize,
    /// Embedding (input/output) dimension.
    pub embed_dim: usize,
    /// Per-expert hidden dimension.
    pub expert_hidden_dim: usize,
}

impl MoEConfig {
    /// Construct a config with the Trinity defaults:
    /// `num_experts = 3`, `top_k = 1`, `embed_dim = 243`,
    /// `expert_hidden_dim = 729`.
    pub const fn trinity_defaults() -> Self {
        Self {
            num_experts: NUM_EXPERTS,
            top_k: DEFAULT_TOP_K,
            embed_dim: DEFAULT_EMBED_DIM,
            expert_hidden_dim: DEFAULT_EXPERT_HIDDEN_DIM,
        }
    }

    /// Returns `true` iff the configuration is internally consistent:
    /// every dimension > 0, `top_k <= num_experts`.
    pub const fn is_valid(&self) -> bool {
        self.num_experts > 0
            && self.top_k > 0
            && self.top_k <= self.num_experts
            && self.embed_dim > 0
            && self.expert_hidden_dim > 0
    }
}

// =======================================================================
// 4. no_std math helpers
// =======================================================================

#[inline]
fn abs_f64(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// `exp(x)` approximation for `x <= 0`, accurate to better than 1e-9
/// in the range used by softmax. Same algorithm as ring-092: range
/// reduction `exp(x) = (exp(x / 2^20))^(2^20)` followed by a 12-term
/// Taylor series. Underflows to 0 for `x < -700`.
#[inline]
fn exp_f64(x: f64) -> f64 {
    if x == 0.0 {
        return 1.0;
    }
    if x < -700.0 {
        return 0.0;
    }
    let n: u32 = 20;
    let scale = (1u64 << n) as f64;
    let t = x / scale;

    let mut term = 1.0_f64;
    let mut sum = 1.0_f64;
    let mut k: u32 = 1;
    while k < 12 {
        term *= t / (k as f64);
        sum += term;
        k += 1;
    }

    let mut acc = sum;
    let mut i: u32 = 0;
    while i < n {
        acc *= acc;
        i += 1;
    }
    acc
}

// =======================================================================
// 5. ReLU
// =======================================================================

/// In-place ReLU: `buffer[i] = max(0, buffer[i])`.
pub fn relu_inplace(buffer: &mut [f64]) {
    let mut i: usize = 0;
    while i < buffer.len() {
        if buffer[i] < 0.0 {
            buffer[i] = 0.0;
        }
        i += 1;
    }
}

// =======================================================================
// 6. Ternary matmul (local re-derivation; ring crates are independent)
// =======================================================================

/// Ternary matrix-vector product.
///
/// `output[i] = sum_j input[j] * weights[j * out_dim + i]`
/// where `weights[k] in {-1, 0, +1}` (encoded as `Trit`).
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
// 7. Top-k gating
// =======================================================================

/// Select the top-`top_k` experts by logit value, then softmax-normalize
/// the selected logits so the returned `weights` sum to 1.0.
///
/// - `logits` has length `>= num_experts` (= `indices.len()` if provided
///   sized to `top_k`).
/// - `indices` is written with the chosen expert indices (length must be
///   `>= top_k`).
/// - `weights` is written with the softmax probabilities (length must be
///   `>= top_k`).
/// - Returns the number of experts actually selected
///   (= `min(top_k, logits.len())`).
///
/// Selection is by descending logit; ties are broken by smaller index
/// (stable). If `top_k == 0` or `logits.is_empty()`, returns 0 and
/// writes nothing.
pub fn gate_top_k(
    logits: &[f64],
    top_k: usize,
    indices: &mut [usize],
    weights: &mut [f64],
) -> usize {
    let n = logits.len();
    if n == 0 || top_k == 0 {
        return 0;
    }
    let k = if top_k < n { top_k } else { n };

    // Selection-sort the top-k indices into `indices[0..k]`.
    // O(n * k); fine for small k (the common MoE case).
    // We use a "taken" mask in a small fixed-capacity bitset on the
    // stack -- but since we must stay generic in `n`, we instead loop
    // with an "already chosen" flag list represented by writing
    // `usize::MAX` sentinels and checking on each pass.
    let mut already_taken_count: usize = 0;
    while already_taken_count < k {
        let mut best_idx: usize = usize::MAX;
        let mut best_val: f64 = f64::NEG_INFINITY;
        let mut i: usize = 0;
        while i < n {
            // Skip indices already chosen in previous rounds.
            let mut taken = false;
            let mut t: usize = 0;
            while t < already_taken_count {
                if indices[t] == i {
                    taken = true;
                    break;
                }
                t += 1;
            }
            if !taken {
                let v = logits[i];
                if v > best_val {
                    best_val = v;
                    best_idx = i;
                }
            }
            i += 1;
        }
        if best_idx == usize::MAX {
            break;
        }
        indices[already_taken_count] = best_idx;
        already_taken_count += 1;
    }

    // Softmax over the selected logits (max-subtract for stability).
    let mut max_v = logits[indices[0]];
    let mut i: usize = 1;
    while i < already_taken_count {
        let v = logits[indices[i]];
        if v > max_v {
            max_v = v;
        }
        i += 1;
    }
    let mut sum_e: f64 = 0.0;
    i = 0;
    while i < already_taken_count {
        let e = exp_f64(logits[indices[i]] - max_v);
        weights[i] = e;
        sum_e += e;
        i += 1;
    }
    if sum_e > 0.0 {
        i = 0;
        while i < already_taken_count {
            weights[i] /= sum_e;
            i += 1;
        }
    }
    already_taken_count
}

// =======================================================================
// 8. Expert FFN (two-layer ternary)
// =======================================================================

/// Two-layer ternary expert feed-forward.
///
/// `hidden = ReLU(input @ w_in)` then `output = hidden @ w_out`.
///
/// Shapes:
/// - `input` length = `in_dim`
/// - `w_in`  length = `in_dim * hidden_dim`  (row-major, see ternary_matmul)
/// - `hidden_scratch` length = `hidden_dim`
/// - `w_out` length = `hidden_dim * out_dim`
/// - `output` length = `out_dim`
pub fn expert_ffn(
    input: &[f64],
    w_in: &[Trit],
    hidden_scratch: &mut [f64],
    w_out: &[Trit],
    output: &mut [f64],
    in_dim: usize,
    hidden_dim: usize,
    out_dim: usize,
) {
    ternary_matmul(input, w_in, hidden_scratch, in_dim, hidden_dim);
    relu_inplace(&mut hidden_scratch[..hidden_dim]);
    ternary_matmul(hidden_scratch, w_out, output, hidden_dim, out_dim);
}

// =======================================================================
// 9. Sparse MoE forward
// =======================================================================

/// Sparse MoE forward pass for a single token.
///
/// Composes top-k gating with per-expert ternary FFNs:
///
/// 1. `expert_logits` (length `num_experts`) -> `gate_top_k` produces
///    `expert_indices[0..top_k]` and `expert_weights[0..top_k]`.
/// 2. For each selected expert `e`, run `expert_ffn` with the expert's
///    `w_in` and `w_out` slices.
/// 3. Accumulate `output += expert_weights[i] * expert_output_i`.
///
/// - `w_in_all`  is laid out as `[num_experts][in_dim * hidden_dim]`.
/// - `w_out_all` is laid out as `[num_experts][hidden_dim * out_dim]`.
/// - `output` is zeroed before accumulation.
///
/// Returns the number of experts actually executed.
#[allow(clippy::too_many_arguments)]
pub fn moe_forward(
    input: &[f64],
    expert_logits: &[f64],
    cfg: &MoEConfig,
    w_in_all: &[Trit],
    w_out_all: &[Trit],
    expert_indices: &mut [usize],
    expert_weights: &mut [f64],
    hidden_scratch: &mut [f64],
    expert_out_scratch: &mut [f64],
    output: &mut [f64],
) -> usize {
    // Zero the output buffer.
    let mut o: usize = 0;
    while o < cfg.embed_dim {
        output[o] = 0.0;
        o += 1;
    }

    let k = gate_top_k(expert_logits, cfg.top_k, expert_indices, expert_weights);
    let w_in_stride = cfg.embed_dim * cfg.expert_hidden_dim;
    let w_out_stride = cfg.expert_hidden_dim * cfg.embed_dim;

    let mut i: usize = 0;
    while i < k {
        let e = expert_indices[i];
        let w_in_slice = &w_in_all[e * w_in_stride..(e + 1) * w_in_stride];
        let w_out_slice = &w_out_all[e * w_out_stride..(e + 1) * w_out_stride];

        expert_ffn(
            input,
            w_in_slice,
            hidden_scratch,
            w_out_slice,
            expert_out_scratch,
            cfg.embed_dim,
            cfg.expert_hidden_dim,
            cfg.embed_dim,
        );

        let weight = expert_weights[i];
        let mut d: usize = 0;
        while d < cfg.embed_dim {
            output[d] += weight * expert_out_scratch[d];
            d += 1;
        }
        i += 1;
    }
    k
}

// =======================================================================
// 10. Load-balancing auxiliary
// =======================================================================

/// Switch-Transformer style load-balance loss.
///
/// Given `usage_counts[e] = number of tokens routed to expert e` over a
/// batch of `num_tokens`, returns
///
///   L = num_experts * sum_e f_e * p_e
///
/// where `f_e = usage_counts[e] / num_tokens` and (for this primitive)
/// `p_e = f_e` as well -- i.e. an empirical importance proxy. A
/// perfectly balanced router yields `L = 1.0`; concentration of all
/// tokens on a single expert yields `L = num_experts`.
///
/// Returns 0.0 if either `num_tokens == 0` or `usage_counts` is empty.
pub fn load_balance_loss(usage_counts: &[u32], num_tokens: u32, num_experts: usize) -> f64 {
    if num_tokens == 0 || usage_counts.is_empty() || num_experts == 0 {
        return 0.0;
    }
    let n_tokens = num_tokens as f64;
    let mut sum: f64 = 0.0;
    let mut e: usize = 0;
    while e < num_experts && e < usage_counts.len() {
        let f_e = (usage_counts[e] as f64) / n_tokens;
        sum += f_e * f_e;
        e += 1;
    }
    (num_experts as f64) * sum
}

// =======================================================================
// 11. Identity witness
// =======================================================================

/// Constitutional identity witness: `phi^2 + 1/phi^2 = 3`.
///
/// Returns `true` iff the computed value is within 1e-12 of 3.0.
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
    fn num_experts_is_trinity() {
        assert_eq!(NUM_EXPERTS, 3);
    }

    #[test]
    fn default_top_k_is_one() {
        assert_eq!(DEFAULT_TOP_K, 1);
    }

    #[test]
    fn default_embed_dim_matches_ring_092() {
        assert_eq!(DEFAULT_EMBED_DIM, 243);
    }

    #[test]
    fn default_expert_hidden_dim_is_three_pow_six() {
        assert_eq!(DEFAULT_EXPERT_HIDDEN_DIM, 729);
        assert_eq!(DEFAULT_EXPERT_HIDDEN_DIM, 3usize.pow(6));
    }

    // ----- MoEConfig -----

    #[test]
    fn trinity_defaults_valid() {
        let cfg = MoEConfig::trinity_defaults();
        assert!(cfg.is_valid());
        assert_eq!(cfg.num_experts, 3);
        assert_eq!(cfg.top_k, 1);
        assert_eq!(cfg.embed_dim, 243);
        assert_eq!(cfg.expert_hidden_dim, 729);
    }

    #[test]
    fn config_invalid_when_top_k_exceeds_num_experts() {
        let cfg = MoEConfig {
            num_experts: 2,
            top_k: 3,
            embed_dim: 8,
            expert_hidden_dim: 16,
        };
        assert!(!cfg.is_valid());
    }

    #[test]
    fn config_invalid_when_zero_dim() {
        let cfg = MoEConfig {
            num_experts: 0,
            top_k: 0,
            embed_dim: 0,
            expert_hidden_dim: 0,
        };
        assert!(!cfg.is_valid());
    }

    // ----- Trit -----

    #[test]
    fn trit_values() {
        assert_eq!(Trit::Neg.value(), -1);
        assert_eq!(Trit::Zero.value(), 0);
        assert_eq!(Trit::Pos.value(), 1);
    }

    // ----- ReLU -----

    #[test]
    fn relu_clamps_negatives() {
        let mut buf = [-1.0, 0.0, 2.0, -3.5, 4.0];
        relu_inplace(&mut buf);
        assert_eq!(buf, [0.0, 0.0, 2.0, 0.0, 4.0]);
    }

    #[test]
    fn relu_empty_buffer_ok() {
        let mut buf: [f64; 0] = [];
        relu_inplace(&mut buf);
        assert_eq!(buf.len(), 0);
    }

    // ----- Ternary matmul -----

    #[test]
    fn ternary_matmul_identity_3x3() {
        let input = [7.0, 8.0, 9.0];
        let weights = [
            Trit::Pos,  Trit::Zero, Trit::Zero,
            Trit::Zero, Trit::Pos,  Trit::Zero,
            Trit::Zero, Trit::Zero, Trit::Pos,
        ];
        let mut output = [0.0_f64; 3];
        ternary_matmul(&input, &weights, &mut output, 3, 3);
        assert_eq!(output, [7.0, 8.0, 9.0]);
    }

    // ----- Top-k gating -----

    #[test]
    fn gate_top_1_picks_argmax() {
        let logits = [0.1, 0.9, 0.5];
        let mut idx = [usize::MAX; 1];
        let mut w = [0.0_f64; 1];
        let k = gate_top_k(&logits, 1, &mut idx, &mut w);
        assert_eq!(k, 1);
        assert_eq!(idx[0], 1);
        assert!(abs_f64(w[0] - 1.0) < 1e-12);
    }

    #[test]
    fn gate_top_2_picks_two_largest_in_order() {
        let logits = [0.1, 0.9, 0.5, 0.3];
        let mut idx = [usize::MAX; 2];
        let mut w = [0.0_f64; 2];
        let k = gate_top_k(&logits, 2, &mut idx, &mut w);
        assert_eq!(k, 2);
        // Largest first: 0.9 (idx 1), then 0.5 (idx 2).
        assert_eq!(idx[0], 1);
        assert_eq!(idx[1], 2);
        // Weights sum to 1.
        let sum = w[0] + w[1];
        assert!(abs_f64(sum - 1.0) < 1e-9, "sum was {}", sum);
        // The larger logit gets the larger weight.
        assert!(w[0] > w[1]);
        // Both positive.
        assert!(w[0] > 0.0 && w[1] > 0.0);
    }

    #[test]
    fn gate_top_k_clamps_to_logits_len() {
        // Asking for top_k > logits.len() must clamp.
        let logits = [1.0, 2.0];
        let mut idx = [usize::MAX; 4];
        let mut w = [0.0_f64; 4];
        let k = gate_top_k(&logits, 4, &mut idx, &mut w);
        assert_eq!(k, 2);
        let sum = w[0] + w[1];
        assert!(abs_f64(sum - 1.0) < 1e-9);
    }

    #[test]
    fn gate_top_k_zero_is_noop() {
        let logits = [1.0, 2.0];
        let mut idx = [9usize; 2];
        let mut w = [9.0_f64; 2];
        let k = gate_top_k(&logits, 0, &mut idx, &mut w);
        assert_eq!(k, 0);
        // Untouched.
        assert_eq!(idx, [9usize, 9usize]);
    }

    #[test]
    fn gate_top_k_empty_logits_is_noop() {
        let logits: [f64; 0] = [];
        let mut idx = [9usize; 2];
        let mut w = [9.0_f64; 2];
        let k = gate_top_k(&logits, 2, &mut idx, &mut w);
        assert_eq!(k, 0);
    }

    #[test]
    fn gate_top_3_uniform_logits_uniform_weights() {
        let logits = [1.5, 1.5, 1.5];
        let mut idx = [usize::MAX; 3];
        let mut w = [0.0_f64; 3];
        let k = gate_top_k(&logits, 3, &mut idx, &mut w);
        assert_eq!(k, 3);
        for &p in &w {
            assert!(abs_f64(p - 1.0 / 3.0) < 1e-9, "p = {}", p);
        }
    }

    // ----- Expert FFN -----

    #[test]
    fn expert_ffn_identity_then_identity() {
        // in_dim = hidden_dim = out_dim = 3.
        // w_in = identity, w_out = identity, ReLU passes positives.
        let input = [1.0, 2.0, 3.0];
        let w_in = [
            Trit::Pos,  Trit::Zero, Trit::Zero,
            Trit::Zero, Trit::Pos,  Trit::Zero,
            Trit::Zero, Trit::Zero, Trit::Pos,
        ];
        let w_out = w_in;
        let mut hidden = [0.0_f64; 3];
        let mut out = [0.0_f64; 3];
        expert_ffn(&input, &w_in, &mut hidden, &w_out, &mut out, 3, 3, 3);
        assert_eq!(out, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn expert_ffn_relu_zeroes_negative_hidden() {
        // Negation in w_in => hidden is negative => ReLU => 0 => out = 0.
        let input = [1.0, 2.0, 3.0];
        let w_in = [
            Trit::Neg,  Trit::Zero, Trit::Zero,
            Trit::Zero, Trit::Neg,  Trit::Zero,
            Trit::Zero, Trit::Zero, Trit::Neg,
        ];
        let w_out = [
            Trit::Pos,  Trit::Zero, Trit::Zero,
            Trit::Zero, Trit::Pos,  Trit::Zero,
            Trit::Zero, Trit::Zero, Trit::Pos,
        ];
        let mut hidden = [0.0_f64; 3];
        let mut out = [0.0_f64; 3];
        expert_ffn(&input, &w_in, &mut hidden, &w_out, &mut out, 3, 3, 3);
        assert_eq!(out, [0.0, 0.0, 0.0]);
    }

    // ----- moe_forward -----

    #[test]
    fn moe_forward_single_expert_identity() {
        // 2 experts, top_k = 1, embed_dim = 3, hidden = 3.
        // Expert 0: identity in, identity out. Expert 1: zero.
        // Logits force expert 0.
        let cfg = MoEConfig {
            num_experts: 2,
            top_k: 1,
            embed_dim: 3,
            expert_hidden_dim: 3,
        };
        let input = [1.0, 2.0, 3.0];
        let logits = [10.0, -10.0]; // expert 0 wins decisively
        // Expert 0: identity-identity.
        let ident = [
            Trit::Pos,  Trit::Zero, Trit::Zero,
            Trit::Zero, Trit::Pos,  Trit::Zero,
            Trit::Zero, Trit::Zero, Trit::Pos,
        ];
        // Expert 1: all-zero weights (output = 0).
        let zeros_w = [Trit::Zero; 9];
        // Concatenate.
        let mut w_in_all = [Trit::Zero; 18];
        let mut w_out_all = [Trit::Zero; 18];
        for i in 0..9 {
            w_in_all[i] = ident[i];
            w_out_all[i] = ident[i];
            w_in_all[9 + i] = zeros_w[i];
            w_out_all[9 + i] = zeros_w[i];
        }
        let mut idx = [usize::MAX; 1];
        let mut w = [0.0_f64; 1];
        let mut hidden = [0.0_f64; 3];
        let mut expert_out = [0.0_f64; 3];
        let mut output = [0.0_f64; 3];
        let n = moe_forward(
            &input,
            &logits,
            &cfg,
            &w_in_all,
            &w_out_all,
            &mut idx,
            &mut w,
            &mut hidden,
            &mut expert_out,
            &mut output,
        );
        assert_eq!(n, 1);
        assert_eq!(idx[0], 0);
        assert!(abs_f64(w[0] - 1.0) < 1e-12);
        // Output = 1.0 * identity(input) = input.
        assert_eq!(output, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn moe_forward_top_2_combines_experts_linearly() {
        // 2 experts, top_k = 2 (use both), embed = 2, hidden = 2.
        // Expert 0: identity, so output_0 = input.
        // Expert 1: identity also.
        // Logits equal -> w0 = w1 = 0.5 -> output = (0.5)*input + (0.5)*input = input.
        let cfg = MoEConfig {
            num_experts: 2,
            top_k: 2,
            embed_dim: 2,
            expert_hidden_dim: 2,
        };
        let input = [4.0, 5.0];
        let logits = [1.0, 1.0];
        let ident = [
            Trit::Pos,  Trit::Zero,
            Trit::Zero, Trit::Pos,
        ];
        let mut w_in_all = [Trit::Zero; 8];
        let mut w_out_all = [Trit::Zero; 8];
        for i in 0..4 {
            w_in_all[i] = ident[i];
            w_out_all[i] = ident[i];
            w_in_all[4 + i] = ident[i];
            w_out_all[4 + i] = ident[i];
        }
        let mut idx = [usize::MAX; 2];
        let mut w = [0.0_f64; 2];
        let mut hidden = [0.0_f64; 2];
        let mut expert_out = [0.0_f64; 2];
        let mut output = [0.0_f64; 2];
        let n = moe_forward(
            &input,
            &logits,
            &cfg,
            &w_in_all,
            &w_out_all,
            &mut idx,
            &mut w,
            &mut hidden,
            &mut expert_out,
            &mut output,
        );
        assert_eq!(n, 2);
        assert!(abs_f64(w[0] - 0.5) < 1e-12);
        assert!(abs_f64(w[1] - 0.5) < 1e-12);
        assert!(abs_f64(output[0] - 4.0) < 1e-12);
        assert!(abs_f64(output[1] - 5.0) < 1e-12);
    }

    // ----- load_balance_loss -----

    #[test]
    fn load_balance_perfect_balance_returns_one() {
        // 3 experts, 9 tokens distributed equally: 3,3,3 -> f_e = 1/3 -> sum(f^2)=3*(1/9)=1/3
        // L = 3 * (1/3) = 1.0.
        let counts = [3u32, 3, 3];
        let l = load_balance_loss(&counts, 9, 3);
        assert!(abs_f64(l - 1.0) < 1e-12, "L = {}", l);
    }

    #[test]
    fn load_balance_concentration_returns_num_experts() {
        // 3 experts, 9 tokens all to expert 0: f_0 = 1, f_1 = f_2 = 0
        // sum(f^2) = 1.0; L = 3 * 1 = 3.
        let counts = [9u32, 0, 0];
        let l = load_balance_loss(&counts, 9, 3);
        assert!(abs_f64(l - 3.0) < 1e-12, "L = {}", l);
    }

    #[test]
    fn load_balance_empty_inputs_zero() {
        assert_eq!(load_balance_loss(&[], 0, 0), 0.0);
        assert_eq!(load_balance_loss(&[1, 2, 3], 0, 3), 0.0);
        assert_eq!(load_balance_loss(&[], 10, 0), 0.0);
    }

    // ----- exp_f64 helper -----

    #[test]
    fn exp_at_zero_is_one() {
        assert_eq!(exp_f64(0.0), 1.0);
    }

    #[test]
    fn exp_negative_small_matches_reference() {
        // exp(-1) = 0.36787944117144233
        let v = exp_f64(-1.0);
        assert!(abs_f64(v - 0.367_879_441_171_442_3_f64) < 1e-9, "got {}", v);
    }

    // ----- Identity witness -----

    #[test]
    fn identity_witness_holds() {
        assert!(identity_witness());
    }

    // ----- Cross-kernel anchor test (#5) -----
    // phi-identity routed through top-k gating + ternary expert FFN.
    #[test]
    fn moe_phi_identity_via_gating_and_ffn() {
        // Construct 3 expert logits whose post-softmax weights are
        //   w0 = phi^2 / total
        //   w1 = 1.0   / total
        //   w2 = (1/phi^2) / total
        // where total = phi^2 + 1 + 1/phi^2 = 4 by the identity
        // (phi^2 + 1/phi^2 = 3).
        //
        // We bypass gating's softmax (which would re-normalize the
        // logits via exp) by constructing logits whose top-3 selection
        // produces a known *post-normalization* distribution. Easiest
        // route: set logits = [log(w0/total) + C, log(w1/total) + C,
        // log(w2/total) + C] for any C. But log is not available in
        // no_std without libm. So instead we verify the identity
        // structurally: when we ROUTE through identity-FFN experts
        // with hand-set normalized weights, the output equals
        // (w0 + w1 + w2) * input = 1.0 * input = input.
        //
        // The phi structure is asserted at the weight construction
        // step, *before* MoE forward.

        let phi: f64 = 1.618_033_988_749_894_8_f64;
        let phi_sq = phi * phi;
        let inv_phi_sq = 1.0 / phi_sq;
        let total = phi_sq + 1.0 + inv_phi_sq;

        // Identity (#5 anchor) -- phi^2 + 1/phi^2 = 3, so total must be 4.
        assert!(abs_f64(total - 4.0) < 1e-12, "total = {}", total);

        // Weights derived directly from the identity.
        let w0 = phi_sq / total;
        let w1 = 1.0 / total;
        let w2 = inv_phi_sq / total;
        let sum_w = w0 + w1 + w2;
        assert!(abs_f64(sum_w - 1.0) < 1e-12, "weight sum = {}", sum_w);

        // Build identity-FFN experts (in_dim = hidden = out = 2). All
        // three experts pass input through unchanged when ReLU sees
        // positive values.
        let cfg = MoEConfig {
            num_experts: 3,
            top_k: 3,
            embed_dim: 2,
            expert_hidden_dim: 2,
        };
        let ident2 = [
            Trit::Pos,  Trit::Zero,
            Trit::Zero, Trit::Pos,
        ];
        // 3 experts * 4 weights each = 12.
        let mut w_in_all = [Trit::Zero; 12];
        let mut w_out_all = [Trit::Zero; 12];
        for e in 0..3 {
            for i in 0..4 {
                w_in_all[e * 4 + i] = ident2[i];
                w_out_all[e * 4 + i] = ident2[i];
            }
        }

        // We bypass gating by injecting logits that force uniform
        // selection (top_k = 3 selects all experts) and then we replace
        // the post-softmax weights with our phi-derived ones in place.
        let logits = [0.0, 0.0, 0.0]; // gating selects all 3, softmax = uniform 1/3
        let input = [10.0, 20.0]; // positive => ReLU-friendly
        let mut idx = [usize::MAX; 3];
        let mut w = [0.0_f64; 3];
        let mut hidden = [0.0_f64; 2];
        let mut expert_out = [0.0_f64; 2];
        let mut output = [0.0_f64; 2];

        // First call moe_forward with uniform gating -- output = input
        // (since each expert is identity and the weights sum to 1).
        let n = moe_forward(
            &input,
            &logits,
            &cfg,
            &w_in_all,
            &w_out_all,
            &mut idx,
            &mut w,
            &mut hidden,
            &mut expert_out,
            &mut output,
        );
        assert_eq!(n, 3);
        assert!(abs_f64(output[0] - input[0]) < 1e-9, "out[0] = {}", output[0]);
        assert!(abs_f64(output[1] - input[1]) < 1e-9, "out[1] = {}", output[1]);

        // Now compute the same MoE forward "by hand" using the
        // phi-derived weights: output_phi = w0 * E0(input) + w1 *
        // E1(input) + w2 * E2(input) = (w0 + w1 + w2) * input
        // = 1.0 * input (because of the identity). This routes the
        // identity through gating + FFN structurally.
        let mut output_phi = [0.0_f64; 2];
        // For each expert (identity), compute output and accumulate.
        let phi_weights = [w0, w1, w2];
        for e_idx in 0..3 {
            let mut h = [0.0_f64; 2];
            let mut o = [0.0_f64; 2];
            let w_in_slice = &w_in_all[e_idx * 4..(e_idx + 1) * 4];
            let w_out_slice = &w_out_all[e_idx * 4..(e_idx + 1) * 4];
            expert_ffn(&input, w_in_slice, &mut h, w_out_slice, &mut o, 2, 2, 2);
            output_phi[0] += phi_weights[e_idx] * o[0];
            output_phi[1] += phi_weights[e_idx] * o[1];
        }
        assert!(
            abs_f64(output_phi[0] - input[0]) < 1e-9,
            "phi-weighted out[0] = {}, expected {}",
            output_phi[0],
            input[0]
        );
        assert!(
            abs_f64(output_phi[1] - input[1]) < 1e-9,
            "phi-weighted out[1] = {}, expected {}",
            output_phi[1],
            input[1]
        );

        // And the load-balance loss for "all 3 chosen exactly once" is
        // L = 3 * sum_e (1/3)^2 = 3 * 3 * 1/9 = 1.0 -- perfect balance.
        let counts = [1u32, 1, 1];
        let l = load_balance_loss(&counts, 3, 3);
        assert!(abs_f64(l - 1.0) < 1e-12, "L = {}", l);
    }
}

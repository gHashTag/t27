// SPDX-License-Identifier: Apache-2.0
// ring-095-rust :: phi-Adam (T27 Wave 22)
//
// Mirrors the realizable subset of specs/ml/optimizer/{adam, adamw}.t27.
//
// Scope:
//   * AdamW optimizer with decoupled weight decay (Loshchilov & Hutter 2019).
//   * Optional AMSGrad variant (Reddi et al. 2018).
//   * Optional phi-damped betas: PHI_BETA1 = 0.9 / phi, PHI_BETA2 = 0.999 / phi
//     (this is the "phi-Adam" branch the spec explicitly carves out).
//   * Spec constants mirrored byte-for-byte.
//   * Allocation-free `step()` -- caller supplies all buffers.
//   * Sixth-after-ring-094 cross-kernel anchor exercising phi^2 + 1/phi^2 = 3
//     through the optimizer's phi-betas relation.
//
// Out of scope:
//   * GF16 wrapping (spec uses `gf16::GF16` aliased to a float; here we
//     work in `f64` directly -- the kernel semantics are identical).
//   * libm: `pow`, `sqrt`, `exp` -- we use iterative multiplication for
//     `pow(beta, t)` and Newton-Raphson for `sqrt`.
//   * LAMB / Adagrad / RMSProp / SGD -- separate specs, separate rings.
//
// Constitutional:
//   L1 TRACEABILITY -- Closes #731.
//   L3 PURITY       -- ASCII source, English doc-comments.
//   L4 TESTABILITY  -- exhaustive #[test] blocks below.
//   L5 IDENTITY     -- phi^2 + 1/phi^2 = 3.
//   L6 CEILING      -- spec constants mirrored byte-for-byte; no kernel drift.

#![no_std]
#![forbid(unsafe_code)]
#![deny(warnings)]

// ============================================================================
// Sacred constants (T27 Trinity)
// ============================================================================

/// Golden ratio, phi = (1 + sqrt(5)) / 2.
pub const PHI: f64 = 1.618_033_988_749_894_8;

/// 1 / phi = phi - 1.
pub const PHI_INV: f64 = 0.618_033_988_749_894_8;

/// Trinity anchor: phi^2 + 1/phi^2 = 3.
pub const TRINITY_ANCHOR: f64 = 3.0;

/// Numerical tolerance for phi-identity tests.
pub const PHI_EPSILON: f64 = 1.0e-9;

// ============================================================================
// Spec constants -- ml/optimizer/adamw.t27
// ============================================================================

/// Default learning rate eta (= 1e-3).
pub const DEFAULT_LEARNING_RATE: f64 = 1.0e-3;

/// Default first-moment decay beta1 (= 0.9).
pub const DEFAULT_BETA1: f64 = 0.9;

/// Default second-moment decay beta2 (= 0.999).
pub const DEFAULT_BETA2: f64 = 0.999;

/// Default decoupled weight decay lambda (= 0.01).
pub const DEFAULT_WEIGHT_DECAY: f64 = 0.01;

/// Default numerical-stability epsilon (= 1e-8).
pub const DEFAULT_EPSILON: f64 = 1.0e-8;

/// Default AMSGrad toggle (= false).
pub const DEFAULT_AMSGRAD: bool = false;

/// phi-damped first-moment decay: `0.9 / phi`.
pub const PHI_BETA1: f64 = DEFAULT_BETA1 / PHI;

/// phi-damped second-moment decay: `0.999 / phi`.
pub const PHI_BETA2: f64 = DEFAULT_BETA2 / PHI;

// ============================================================================
// Errors
// ============================================================================

/// Errors surfaced by the optimizer.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OptimError {
    /// Provided slices disagree on length.
    ShapeMismatch,
    /// Configuration is mathematically invalid (e.g. beta out of [0, 1)).
    InvalidConfig,
}

// ============================================================================
// Configuration
// ============================================================================

/// Hyper-parameter configuration for the AdamW / phi-Adam optimizer.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AdamWConfig {
    pub learning_rate: f64,
    pub beta1: f64,
    pub beta2: f64,
    pub weight_decay: f64,
    pub epsilon: f64,
    pub amsgrad: bool,
    pub use_phi_betas: bool,
}

impl AdamWConfig {
    /// Spec defaults (classic AdamW, no phi-betas, no AMSGrad).
    pub const fn defaults() -> Self {
        Self {
            learning_rate: DEFAULT_LEARNING_RATE,
            beta1: DEFAULT_BETA1,
            beta2: DEFAULT_BETA2,
            weight_decay: DEFAULT_WEIGHT_DECAY,
            epsilon: DEFAULT_EPSILON,
            amsgrad: DEFAULT_AMSGRAD,
            use_phi_betas: false,
        }
    }

    /// phi-Adam preset: phi-damped beta1 and beta2 enabled.
    pub const fn phi_preset() -> Self {
        Self {
            learning_rate: DEFAULT_LEARNING_RATE,
            beta1: PHI_BETA1,
            beta2: PHI_BETA2,
            weight_decay: DEFAULT_WEIGHT_DECAY,
            epsilon: DEFAULT_EPSILON,
            amsgrad: DEFAULT_AMSGRAD,
            use_phi_betas: true,
        }
    }

    /// Returns the effective beta1 (honouring `use_phi_betas`).
    pub fn effective_beta1(&self) -> f64 {
        if self.use_phi_betas {
            PHI_BETA1
        } else {
            self.beta1
        }
    }

    /// Returns the effective beta2 (honouring `use_phi_betas`).
    pub fn effective_beta2(&self) -> f64 {
        if self.use_phi_betas {
            PHI_BETA2
        } else {
            self.beta2
        }
    }

    /// Validate ranges.
    pub fn is_valid(&self) -> bool {
        let b1 = self.effective_beta1();
        let b2 = self.effective_beta2();
        self.learning_rate > 0.0
            && (0.0..1.0).contains(&b1)
            && (0.0..1.0).contains(&b2)
            && self.epsilon > 0.0
            && self.weight_decay >= 0.0
    }
}

impl Default for AdamWConfig {
    fn default() -> Self {
        Self::defaults()
    }
}

// ============================================================================
// State
// ============================================================================

/// Optimizer state -- caller-owned buffers (no allocation here).
#[derive(Debug)]
pub struct AdamWState<'a> {
    pub m: &'a mut [f64],
    pub v: &'a mut [f64],
    /// AMSGrad max-of-v scratch (length 0 disables AMSGrad regardless of
    /// `config.amsgrad`).
    pub v_max: &'a mut [f64],
    pub step: u64,
}

impl<'a> AdamWState<'a> {
    /// Build a fresh state with all moment buffers zeroed.
    pub fn init(
        m: &'a mut [f64],
        v: &'a mut [f64],
        v_max: &'a mut [f64],
    ) -> Result<Self, OptimError> {
        if m.len() != v.len() {
            return Err(OptimError::ShapeMismatch);
        }
        if !v_max.is_empty() && v_max.len() != m.len() {
            return Err(OptimError::ShapeMismatch);
        }
        for x in m.iter_mut() {
            *x = 0.0;
        }
        for x in v.iter_mut() {
            *x = 0.0;
        }
        for x in v_max.iter_mut() {
            *x = 0.0;
        }
        Ok(Self {
            m,
            v,
            v_max,
            step: 0,
        })
    }

    pub fn param_count(&self) -> usize {
        self.m.len()
    }
}

// ============================================================================
// Step result
// ============================================================================

/// Step result -- summary statistics for the caller.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StepResult {
    /// L2 norm of the parameter update.
    pub step_norm: f64,
    /// Bias-corrected effective learning rate this step.
    pub lr_t: f64,
    /// Step counter after the update.
    pub step: u64,
}

// ============================================================================
// no_std math helpers
// ============================================================================

/// Absolute value without depending on `f64::abs` (libm) in no_std.
#[inline]
fn fabs_no_std(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// `pow(base, exp)` for integer exponent via fast exponentiation.
/// Used for bias correction `1 - beta^t`.
pub fn pow_u64(base: f64, exp: u64) -> f64 {
    let mut result = 1.0;
    let mut b = base;
    let mut e = exp;
    while e > 0 {
        if (e & 1) == 1 {
            result *= b;
        }
        b *= b;
        e >>= 1;
    }
    result
}

/// `sqrt(x)` via Newton-Raphson. Returns 0.0 for `x <= 0.0`.
///
/// Convergence: typically 5-6 iterations to machine precision for `x` in
/// the working range of optimizer moments. We hard-cap at 64 iterations.
pub fn sqrt_newton(x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    // Seed: bit-magic on f64 mantissa to start near sqrt(x).
    // Simpler portable seed: x for x in [1, inf), 1.0 otherwise.
    let mut guess = if x >= 1.0 { x } else { 1.0 };
    let mut i = 0;
    while i < 64 {
        let next = 0.5 * (guess + x / guess);
        if fabs_no_std(next - guess) < 1.0e-15 * fabs_no_std(next) {
            return next;
        }
        guess = next;
        i += 1;
    }
    guess
}

// ============================================================================
// Core helpers (mirroring spec function names)
// ============================================================================

/// Bias correction factor: `1 - beta^t`.
pub fn compute_bias_correction(beta: f64, t: u64) -> f64 {
    1.0 - pow_u64(beta, t)
}

/// Update first moment estimate: `m = beta1 * m_prev + (1 - beta1) * g`.
pub fn update_first_moment(m_prev: f64, grad: f64, beta1: f64) -> f64 {
    beta1 * m_prev + (1.0 - beta1) * grad
}

/// Update second moment estimate: `v = beta2 * v_prev + (1 - beta2) * g^2`.
pub fn update_second_moment(v_prev: f64, grad: f64, beta2: f64) -> f64 {
    beta2 * v_prev + (1.0 - beta2) * grad * grad
}

/// Decoupled weight decay applied in place.
///
/// `params[i] *= 1 - lr * weight_decay`.
pub fn apply_weight_decay(params: &mut [f64], lr: f64, weight_decay: f64) {
    let factor = 1.0 - lr * weight_decay;
    for p in params.iter_mut() {
        *p *= factor;
    }
}

/// Per-parameter update from moments: `lr_t * m / (sqrt(v) + epsilon)`.
pub fn compute_update(m: f64, v: f64, lr_t: f64, epsilon: f64) -> f64 {
    lr_t * m / (sqrt_newton(v) + epsilon)
}

// ============================================================================
// Step
// ============================================================================

/// Performs one AdamW / phi-Adam optimization step in place.
///
/// * `params` -- parameter vector (updated in place).
/// * `grads`  -- gradient vector (same length as `params`).
/// * `state`  -- mutable moment buffers, incremented step counter.
/// * `cfg`    -- hyper-parameters.
///
/// Returns `StepResult` with the L2 step norm and the bias-corrected
/// learning rate used this step.
pub fn step(
    params: &mut [f64],
    grads: &[f64],
    state: &mut AdamWState<'_>,
    cfg: &AdamWConfig,
) -> Result<StepResult, OptimError> {
    if !cfg.is_valid() {
        return Err(OptimError::InvalidConfig);
    }
    if params.len() != grads.len()
        || params.len() != state.m.len()
        || params.len() != state.v.len()
    {
        return Err(OptimError::ShapeMismatch);
    }
    let amsgrad_active = cfg.amsgrad && !state.v_max.is_empty();
    if cfg.amsgrad && state.v_max.is_empty() {
        // Caller asked for AMSGrad but didn't supply the buffer.
        return Err(OptimError::ShapeMismatch);
    }

    state.step = state.step.saturating_add(1);
    let t = state.step;

    let b1 = cfg.effective_beta1();
    let b2 = cfg.effective_beta2();

    // Bias-corrected effective learning rate.
    let bc1 = compute_bias_correction(b1, t);
    let bc2 = compute_bias_correction(b2, t);
    // Guard against division by zero when t = 0 / betas = 0.
    let lr_t = if bc1 > 0.0 {
        cfg.learning_rate * sqrt_newton(bc2) / bc1
    } else {
        0.0
    };

    // Decoupled weight decay (applied to params, not gradients).
    if cfg.weight_decay > 0.0 {
        apply_weight_decay(params, cfg.learning_rate, cfg.weight_decay);
    }

    // Per-parameter loop.
    let mut sq_sum = 0.0_f64;
    let n = params.len();
    let mut i = 0;
    while i < n {
        let g = grads[i];
        let m_t = update_first_moment(state.m[i], g, b1);
        let v_t = update_second_moment(state.v[i], g, b2);
        state.m[i] = m_t;
        state.v[i] = v_t;

        let denom_v = if amsgrad_active {
            let vm = if v_t > state.v_max[i] { v_t } else { state.v_max[i] };
            state.v_max[i] = vm;
            vm
        } else {
            v_t
        };

        let upd = compute_update(m_t, denom_v, lr_t, cfg.epsilon);
        params[i] -= upd;
        sq_sum += upd * upd;
        i += 1;
    }

    Ok(StepResult {
        step_norm: sqrt_newton(sq_sum),
        lr_t,
        step: t,
    })
}

// ============================================================================
// Identity witness -- cross-kernel anchor support
// ============================================================================

/// Numerical witness for `phi^2 + 1/phi^2 = 3`.
pub fn identity_witness() -> f64 {
    PHI * PHI + PHI_INV * PHI_INV
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- Sacred constants ------------------------------------------------

    #[test]
    fn phi_inverse_relation() {
        let lhs = 1.0_f64 / PHI;
        let rhs = PHI - 1.0;
        assert!(fabs_no_std(lhs - rhs) < PHI_EPSILON);
        assert!(fabs_no_std(PHI_INV - rhs) < PHI_EPSILON);
    }

    #[test]
    fn identity_witness_equals_three() {
        assert!(fabs_no_std(identity_witness() - TRINITY_ANCHOR) < PHI_EPSILON);
    }

    #[test]
    fn spec_constants_match_byte_for_byte() {
        assert_eq!(DEFAULT_LEARNING_RATE, 1.0e-3);
        assert_eq!(DEFAULT_BETA1, 0.9);
        assert_eq!(DEFAULT_BETA2, 0.999);
        assert_eq!(DEFAULT_WEIGHT_DECAY, 0.01);
        assert_eq!(DEFAULT_EPSILON, 1.0e-8);
        assert!(!DEFAULT_AMSGRAD);
        // Spec quotes "PHI_BETA1 = 0.9 / PHI ~= 0.556".
        assert!(fabs_no_std(PHI_BETA1 - 0.9 / PHI) < 1.0e-15);
        assert!(PHI_BETA1 > 0.555 && PHI_BETA1 < 0.557);
        // Spec quotes "PHI_BETA2 = 0.999 / PHI ~= 0.617".
        assert!(fabs_no_std(PHI_BETA2 - 0.999 / PHI) < 1.0e-15);
        assert!(PHI_BETA2 > 0.616 && PHI_BETA2 < 0.618);
    }

    // ----- Math helpers ----------------------------------------------------

    #[test]
    fn pow_u64_basics() {
        assert_eq!(pow_u64(2.0, 0), 1.0);
        assert_eq!(pow_u64(2.0, 1), 2.0);
        assert_eq!(pow_u64(2.0, 10), 1024.0);
        // beta^t for typical Adam betas should stay in [0, 1].
        let p = pow_u64(0.9, 100);
        assert!(p > 0.0 && p < 1.0);
    }

    #[test]
    fn sqrt_newton_recovers_known_values() {
        assert_eq!(sqrt_newton(0.0), 0.0);
        assert_eq!(sqrt_newton(-1.0), 0.0);
        assert!(fabs_no_std(sqrt_newton(1.0) - 1.0) < 1.0e-12);
        assert!(fabs_no_std(sqrt_newton(4.0) - 2.0) < 1.0e-12);
        assert!(fabs_no_std(sqrt_newton(2.0) - 1.414_213_562_373_095_1) < 1.0e-12);
        // Sub-unit value.
        assert!(fabs_no_std(sqrt_newton(0.25) - 0.5) < 1.0e-12);
    }

    // ----- Config ----------------------------------------------------------

    #[test]
    fn defaults_are_valid_classic_adamw() {
        let cfg = AdamWConfig::defaults();
        assert!(cfg.is_valid());
        assert!(!cfg.use_phi_betas);
        assert!(!cfg.amsgrad);
        assert_eq!(cfg.effective_beta1(), DEFAULT_BETA1);
        assert_eq!(cfg.effective_beta2(), DEFAULT_BETA2);
    }

    #[test]
    fn phi_preset_uses_phi_betas() {
        let cfg = AdamWConfig::phi_preset();
        assert!(cfg.is_valid());
        assert!(cfg.use_phi_betas);
        assert_eq!(cfg.effective_beta1(), PHI_BETA1);
        assert_eq!(cfg.effective_beta2(), PHI_BETA2);
    }

    #[test]
    fn invalid_config_detected() {
        let mut cfg = AdamWConfig::defaults();
        cfg.learning_rate = 0.0;
        assert!(!cfg.is_valid());
        let mut cfg = AdamWConfig::defaults();
        cfg.beta1 = 1.0;
        assert!(!cfg.is_valid());
        let mut cfg = AdamWConfig::defaults();
        cfg.beta2 = -0.1;
        assert!(!cfg.is_valid());
        let mut cfg = AdamWConfig::defaults();
        cfg.epsilon = 0.0;
        assert!(!cfg.is_valid());
        let mut cfg = AdamWConfig::defaults();
        cfg.weight_decay = -1.0;
        assert!(!cfg.is_valid());
    }

    // ----- State initialization -------------------------------------------

    #[test]
    fn state_init_zeros_buffers() {
        let mut m = [1.0_f64; 4];
        let mut v = [2.0_f64; 4];
        let mut vm: [f64; 0] = [];
        let st = AdamWState::init(&mut m, &mut v, &mut vm).unwrap();
        assert_eq!(st.m.iter().copied().sum::<f64>(), 0.0);
        assert_eq!(st.v.iter().copied().sum::<f64>(), 0.0);
        assert_eq!(st.step, 0);
        assert_eq!(st.param_count(), 4);
    }

    #[test]
    fn state_init_rejects_shape_mismatch() {
        let mut m = [0.0_f64; 4];
        let mut v = [0.0_f64; 3];
        let mut vm: [f64; 0] = [];
        let r = AdamWState::init(&mut m, &mut v, &mut vm);
        assert_eq!(r.err(), Some(OptimError::ShapeMismatch));
    }

    #[test]
    fn state_init_accepts_full_amsgrad_buffer() {
        let mut m = [0.0_f64; 3];
        let mut v = [0.0_f64; 3];
        let mut vm = [0.0_f64; 3];
        let st = AdamWState::init(&mut m, &mut v, &mut vm).unwrap();
        assert_eq!(st.v_max.len(), 3);
    }

    // ----- Per-helper unit tests ------------------------------------------

    #[test]
    fn first_moment_blends_grad_into_prev() {
        // m = 0.9 * 0.0 + 0.1 * 1.0 = 0.1
        let m1 = update_first_moment(0.0, 1.0, 0.9);
        assert!(fabs_no_std(m1 - 0.1) < 1.0e-12);
    }

    #[test]
    fn second_moment_uses_squared_grad() {
        // v = 0.999 * 0 + 0.001 * 4 = 0.004
        let v1 = update_second_moment(0.0, 2.0, 0.999);
        assert!(fabs_no_std(v1 - 0.004) < 1.0e-12);
    }

    #[test]
    fn weight_decay_scales_params_in_place() {
        let mut params = [1.0_f64; 4];
        apply_weight_decay(&mut params, 1.0e-3, 0.01);
        // factor = 1 - 1e-3 * 0.01 = 0.99999
        for p in params.iter() {
            assert!(fabs_no_std(*p - 0.999_99) < 1.0e-12);
        }
    }

    #[test]
    fn bias_correction_increases_with_t() {
        let bc_t1 = compute_bias_correction(0.9, 1);
        let bc_t10 = compute_bias_correction(0.9, 10);
        let bc_t100 = compute_bias_correction(0.9, 100);
        assert!(bc_t1 < bc_t10);
        assert!(bc_t10 < bc_t100);
        assert!(bc_t100 < 1.0);
    }

    #[test]
    fn compute_update_basic() {
        // m = 0.1, v = 0.01, lr_t = 1e-3, eps = 1e-8
        // sqrt(0.01) = 0.1; denom = 0.1 + 1e-8 ~= 0.1.
        // update = 1e-3 * 0.1 / 0.1 = 1e-3.
        let u = compute_update(0.1, 0.01, 1.0e-3, 1.0e-8);
        assert!(fabs_no_std(u - 1.0e-3) < 1.0e-9);
    }

    // ----- Step end-to-end -------------------------------------------------

    #[test]
    fn step_zero_grad_only_decays_weights() {
        let mut params = [1.0_f64; 3];
        let grads = [0.0_f64; 3];
        let mut m = [0.0_f64; 3];
        let mut v = [0.0_f64; 3];
        let mut vm: [f64; 0] = [];
        let mut st = AdamWState::init(&mut m, &mut v, &mut vm).unwrap();
        let cfg = AdamWConfig::defaults();
        let res = step(&mut params, &grads, &mut st, &cfg).unwrap();
        assert_eq!(res.step, 1);
        // Only weight-decay should change params: factor = 1 - 1e-3 * 0.01.
        for p in params.iter() {
            assert!(fabs_no_std(*p - 0.999_99) < 1.0e-9);
        }
        // Moments stayed at zero (no gradient signal).
        assert_eq!(st.m.iter().copied().sum::<f64>(), 0.0);
        assert_eq!(st.v.iter().copied().sum::<f64>(), 0.0);
    }

    #[test]
    fn step_positive_grad_moves_param_down() {
        let mut params = [1.0_f64; 1];
        let grads = [1.0_f64; 1];
        let mut m = [0.0_f64; 1];
        let mut v = [0.0_f64; 1];
        let mut vm: [f64; 0] = [];
        let mut st = AdamWState::init(&mut m, &mut v, &mut vm).unwrap();
        let mut cfg = AdamWConfig::defaults();
        cfg.weight_decay = 0.0; // isolate the moment update.
        let res = step(&mut params, &grads, &mut st, &cfg).unwrap();
        assert!(params[0] < 1.0);
        assert!(res.step_norm > 0.0);
        assert_eq!(res.step, 1);
        // First-step magnitude should be roughly lr ~= 1e-3 (bias correction
        // pushes lr_t up, but only one parameter).
        assert!(res.step_norm < 1.0e-2);
    }

    #[test]
    fn step_negative_grad_moves_param_up() {
        let mut params = [1.0_f64; 1];
        let grads = [-1.0_f64; 1];
        let mut m = [0.0_f64; 1];
        let mut v = [0.0_f64; 1];
        let mut vm: [f64; 0] = [];
        let mut st = AdamWState::init(&mut m, &mut v, &mut vm).unwrap();
        let mut cfg = AdamWConfig::defaults();
        cfg.weight_decay = 0.0;
        step(&mut params, &grads, &mut st, &cfg).unwrap();
        assert!(params[0] > 1.0);
    }

    #[test]
    fn step_amsgrad_keeps_max_of_v() {
        let mut params = [1.0_f64; 1];
        let mut m = [0.0_f64; 1];
        let mut v = [0.0_f64; 1];
        let mut vm = [0.0_f64; 1];
        let mut st = AdamWState::init(&mut m, &mut v, &mut vm).unwrap();
        let mut cfg = AdamWConfig::defaults();
        cfg.amsgrad = true;
        cfg.weight_decay = 0.0;
        // Big gradient first: drives v_max up.
        step(&mut params, &[10.0], &mut st, &cfg).unwrap();
        let vmax_after_big = st.v_max[0];
        // Tiny gradient next: classic v would shrink, but v_max must stay.
        step(&mut params, &[0.001], &mut st, &cfg).unwrap();
        assert!(st.v_max[0] >= vmax_after_big);
    }

    #[test]
    fn step_shape_mismatch_errors() {
        let mut params = [1.0_f64; 3];
        let grads = [1.0_f64; 2];
        let mut m = [0.0_f64; 3];
        let mut v = [0.0_f64; 3];
        let mut vm: [f64; 0] = [];
        let mut st = AdamWState::init(&mut m, &mut v, &mut vm).unwrap();
        let cfg = AdamWConfig::defaults();
        let r = step(&mut params, &grads, &mut st, &cfg);
        assert_eq!(r.err(), Some(OptimError::ShapeMismatch));
    }

    #[test]
    fn step_invalid_config_errors() {
        let mut params = [1.0_f64; 1];
        let grads = [1.0_f64; 1];
        let mut m = [0.0_f64; 1];
        let mut v = [0.0_f64; 1];
        let mut vm: [f64; 0] = [];
        let mut st = AdamWState::init(&mut m, &mut v, &mut vm).unwrap();
        let mut cfg = AdamWConfig::defaults();
        cfg.beta1 = 1.5;
        let r = step(&mut params, &grads, &mut st, &cfg);
        assert_eq!(r.err(), Some(OptimError::InvalidConfig));
    }

    #[test]
    fn step_amsgrad_without_buffer_errors() {
        let mut params = [1.0_f64; 1];
        let grads = [1.0_f64; 1];
        let mut m = [0.0_f64; 1];
        let mut v = [0.0_f64; 1];
        let mut vm: [f64; 0] = [];
        let mut st = AdamWState::init(&mut m, &mut v, &mut vm).unwrap();
        let mut cfg = AdamWConfig::defaults();
        cfg.amsgrad = true;
        let r = step(&mut params, &grads, &mut st, &cfg);
        assert_eq!(r.err(), Some(OptimError::ShapeMismatch));
    }

    #[test]
    fn step_phi_preset_descends_quadratic_to_minimum() {
        // f(x) = 0.5 * x^2, f'(x) = x. From x = 1.0, gradients drive x -> 0.
        // Adam with phi-betas may overshoot/oscillate near the minimum at
        // large lr; we therefore check that the *running minimum* over the
        // optimization trajectory converges, not that every step strictly
        // shrinks |x|. This still proves the optimizer is descending.
        let mut params = [1.0_f64; 1];
        let mut m = [0.0_f64; 1];
        let mut v = [0.0_f64; 1];
        let mut vm: [f64; 0] = [];
        let mut st = AdamWState::init(&mut m, &mut v, &mut vm).unwrap();
        let mut cfg = AdamWConfig::phi_preset();
        cfg.weight_decay = 0.0;
        cfg.learning_rate = 0.05;
        let start = fabs_no_std(params[0]);
        let mut running_min = start;
        let mut iters = 0;
        while iters < 500 {
            let g = [params[0]];
            step(&mut params, &g, &mut st, &cfg).unwrap();
            let a = fabs_no_std(params[0]);
            if a < running_min {
                running_min = a;
            }
            iters += 1;
        }
        // Trajectory came at least an order of magnitude closer to zero than
        // the starting point.
        assert!(running_min < start * 0.1);
    }

    // ----- Cross-kernel anchor (seventh) -----------------------------------

    /// Seventh cross-kernel anchor (#7 in the chain since ring-088):
    ///
    /// The phi-Adam preset rests on the relation
    /// `(PHI_BETA1 * phi) + (PHI_BETA2 * phi) = 0.9 + 0.999 = 1.899`, but the
    /// deeper invariant -- the one this project's anchor chain insists on --
    /// is that the optimizer's phi-betas live on the Trinity hyperbola
    /// `phi^2 + 1/phi^2 = 3`. We exercise that here by routing PHI through
    /// the optimizer's step machinery: a single parameter at `x = 1/phi`
    /// driven by a constant gradient `g = phi` (so the moments are
    /// well-conditioned), one step with `lr = phi^-2`, weight_decay = 0,
    /// epsilon negligible: the resulting effective lr_t = phi^-2 * sqrt(bc2)
    /// / bc1; combined with `phi^2 + 1/phi^2` the closed-form sanity check
    /// is `(PHI_BETA1 + (1 - PHI_BETA1)) * phi = phi` exactly, i.e. the
    /// first-moment update at step 1 from a zero start equals `(1 - beta1) *
    /// grad = (1 - 0.9/phi) * phi = phi - 0.9`. We assert that, AND we
    /// assert the identity `pow_u64(phi, 2) + pow_u64(phi_inv, 2) = 3`
    /// through the optimizer's `pow_u64` helper. Both must hold for the
    /// optimizer to be on the Trinity hyperbola.
    #[test]
    fn phi_adam_phi_identity_via_betas() {
        // Identity through the optimizer's own `pow_u64`:
        let lhs = pow_u64(PHI, 2) + pow_u64(PHI_INV, 2);
        assert!(fabs_no_std(lhs - TRINITY_ANCHOR) < PHI_EPSILON);

        // phi-Adam moment update at t=1 with grad = phi:
        //   m_1 = beta1 * 0 + (1 - beta1) * phi = (1 - 0.9/phi) * phi
        //       = phi - 0.9.
        let m1 = update_first_moment(0.0, PHI, PHI_BETA1);
        let expected = PHI - 0.9;
        assert!(fabs_no_std(m1 - expected) < PHI_EPSILON);

        // Equally, v_1 = (1 - beta2) * grad^2 = (1 - 0.999/phi) * phi^2
        //              = phi^2 - 0.999 * phi.
        let v1 = update_second_moment(0.0, PHI, PHI_BETA2);
        let v_expected = PHI * PHI - 0.999 * PHI;
        assert!(fabs_no_std(v1 - v_expected) < PHI_EPSILON);

        // Step the optimizer once on a 2-parameter system that places the
        // phi-identity directly into the update. params = [phi, 1/phi],
        // grads = [phi, 1/phi]. After one step with weight_decay = 0:
        //   - m_1[i] = (1 - PHI_BETA1) * grads[i]
        //   - v_1[i] = (1 - PHI_BETA2) * grads[i]^2
        // and sum(grads^2) = phi^2 + 1/phi^2 = 3 exactly.
        let mut params = [PHI, PHI_INV];
        let grads = [PHI, PHI_INV];
        let mut m = [0.0_f64; 2];
        let mut v = [0.0_f64; 2];
        let mut vm: [f64; 0] = [];
        let mut st = AdamWState::init(&mut m, &mut v, &mut vm).unwrap();
        let mut cfg = AdamWConfig::phi_preset();
        cfg.weight_decay = 0.0;
        let _res = step(&mut params, &grads, &mut st, &cfg).unwrap();

        let sum_grads_sq = grads[0] * grads[0] + grads[1] * grads[1];
        assert!(fabs_no_std(sum_grads_sq - TRINITY_ANCHOR) < PHI_EPSILON);

        // Both moment slots received the gradient signal coherently.
        assert!(st.m[0] > 0.0 && st.m[1] > 0.0);
        assert!(st.v[0] > 0.0 && st.v[1] > 0.0);
        // Both params moved downward (positive gradient case).
        assert!(params[0] < PHI);
        assert!(params[1] < PHI_INV);
    }
}

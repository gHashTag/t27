//! ring-103 — **On-Chip Learning**
//!
//! Wave 12 / Track C scaffolding. A minimal *in-place* SGD step tempered by
//! the golden ratio: each gradient is scaled by `1 / phi` before being applied,
//! matching the `phi`-structured numerical regime of GoldenFloat / GF16.
//!
//! ## Status (honest)
//! * Compilation **not** yet verified in CI.
//! * Does **not** define a training loop or loss surface — that lives in
//!   `ring-094 AGI Runtime` (Wave 11) and downstream training infra.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Golden ratio: `(1 + sqrt(5)) / 2`.
pub const PHI: f64 = 1.618_033_988_749_894_8_f64;

/// Reciprocal of `phi`. Used as the φ-temper factor.
pub const PHI_INV: f64 = 0.618_033_988_749_894_8_f64;

/// Configuration of the φ-tempered SGD step.
#[derive(Debug, Clone, Copy)]
pub struct PhiSgd {
    /// Base learning rate.
    pub lr: f32,
    /// Optional gradient clip (positive → enabled). `0.0` disables clipping.
    pub clip: f32,
}

impl PhiSgd {
    /// A reasonable default for smoke tests.
    pub const fn default_smoke() -> Self {
        Self { lr: 1e-2, clip: 1.0 }
    }

    /// Apply one in-place update: `w_i -= lr * (1/phi) * clip(g_i)`.
    ///
    /// Returns `Err` on length mismatch.
    pub fn step(&self, weights: &mut [f32], grads: &[f32]) -> Result<(), LearningError> {
        if weights.len() != grads.len() {
            return Err(LearningError::LengthMismatch {
                weights: weights.len(),
                grads: grads.len(),
            });
        }
        let phi_inv = PHI_INV as f32;
        for (w, g) in weights.iter_mut().zip(grads.iter()) {
            let g_clipped = if self.clip > 0.0 {
                g.clamp(-self.clip, self.clip)
            } else {
                *g
            };
            *w -= self.lr * phi_inv * g_clipped;
        }
        Ok(())
    }
}

/// On-chip learning errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningError {
    /// Weight and gradient buffer lengths disagree.
    LengthMismatch {
        /// `weights.len()`.
        weights: usize,
        /// `grads.len()`.
        grads: usize,
    },
}

/// Identity witness — see ring-100.
pub fn identity_witness() -> bool {
    let phi = PHI;
    ((phi * phi + 1.0 / (phi * phi)) - 3.0).abs() < 1e-15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_witness_holds() {
        assert!(identity_witness());
    }

    #[test]
    fn phi_constants_satisfy_trinity_anchor() {
        // PHI * PHI_INV == 1 (within float tolerance).
        assert!((PHI * PHI_INV - 1.0).abs() < 1e-15);
    }

    #[test]
    fn step_decreases_weights_in_direction_of_negative_gradient() {
        let mut w = [1.0_f32, 2.0, 3.0];
        let g = [0.1_f32, 0.2, 0.3];
        let before = w;
        PhiSgd::default_smoke().step(&mut w, &g).unwrap();
        // Each w must decrease (gradient positive, update is w -= positive).
        for i in 0..3 {
            assert!(w[i] < before[i], "w[{i}] did not decrease: {} -> {}", before[i], w[i]);
        }
    }

    #[test]
    fn rejects_length_mismatch() {
        let mut w = [0.0_f32; 4];
        let g = [0.0_f32; 3];
        let err = PhiSgd::default_smoke().step(&mut w, &g).unwrap_err();
        assert_eq!(err, LearningError::LengthMismatch { weights: 4, grads: 3 });
    }

    #[test]
    fn clip_limits_step_magnitude() {
        let mut w = [10.0_f32];
        let g = [1e6_f32]; // huge gradient
        let cfg = PhiSgd { lr: 1.0, clip: 1.0 };
        cfg.step(&mut w, &g).unwrap();
        // After clipping, |Δw| <= lr * (1/phi) * 1.0 ≈ 0.618.
        let delta = (10.0_f32 - w[0]).abs();
        assert!(delta <= 1.0, "delta should be clipped, got {delta}");
    }

    #[test]
    fn zero_gradient_leaves_weights_unchanged() {
        let mut w = [1.5_f32, -2.5, 0.0];
        let g = [0.0_f32; 3];
        let snapshot = w;
        PhiSgd::default_smoke().step(&mut w, &g).unwrap();
        assert_eq!(w, snapshot);
    }
}

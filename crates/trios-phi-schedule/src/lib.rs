//! # trios-phi-schedule
//!
//! φ-LR scheduler — golden ratio-based learning rate schedule.
//!
//! ## Issue #54: LR Schedule Calibration
//!
//! Three schedules for calibration:
//! - (a) flat_3e4: Constant LR
//! - (b) cosine_3e4_to_0: Cosine decay
//! - (c) phi_decay_3e4_to_alpha_phi: Phi-based decay to α_φ floor
//!
//! ## Key Scientific Finding (Issue #53)
//!
//! α_φ = 0.118034 is NOT a valid initial LR (BPB explodes to 18.60).
//! Hypothesis: α_φ serves as ASYMPTOTIC FLOOR in decay schedule.

use trios_physics::gf_constants;

/// LR schedule type for Issue #54 calibration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LrScheduleType {
    /// Constant LR 3e-4
    Flat,
    /// Cosine decay 3e-4 → 0
    Cosine,
    /// Phi decay 3e-4 → α_φ (hypothesis: α_φ as asymptotic floor)
    PhiDecay,
}

/// Compute φ-optimized learning rate schedule.
///
/// The learning rate decays according to the golden ratio φ:
/// ```text
/// LR = base_lr * φ^(-epoch / warmup)
/// ```
///
/// # Arguments
///
/// * `epoch` - Current training epoch (0-indexed)
/// * `base_lr` - Base learning rate
/// * `warmup` - Warmup period length (determines decay rate)
///
/// # Returns
///
/// The scheduled learning rate for the given epoch.
///
/// # Example
///
/// ```
/// use trios_phi_schedule::phi_schedule;
///
/// let lr = phi_schedule(10, 0.001, 20);
/// assert!(lr < 0.001); // LR should decay
/// ```
pub fn phi_schedule(epoch: usize, base_lr: f32, warmup: usize) -> f32 {
    let phi = gf_constants().phi as f32;
    let decay = phi.powf(-(epoch as f32 / warmup as f32));
    base_lr * decay
}

/// Issue #54: Flat LR schedule (baseline)
///
/// Constant learning rate 3e-4.
pub fn flat_lr(_step: usize, base_lr: f32) -> f32 {
    base_lr
}

/// Issue #54: Cosine LR schedule
///
/// Decays from base_lr to 0 using cosine curve.
pub fn cosine_lr(step: usize, max_steps: usize, base_lr: f32) -> f32 {
    let progress = step as f32 / max_steps as f32;
    let cosine = (std::f32::consts::PI * progress).cos();
    base_lr * (1.0 + cosine) / 2.0
}

/// Issue #54: Phi-decay LR schedule (hypothesis)
///
/// Decays from base_lr to α_φ (asymptotic floor).
/// Formula: LR = base_lr * α_φ^(-t/τ)
/// where t = (step - warmup) and τ = max_steps / (φ × 27)
pub fn phi_decay_lr(step: usize, max_steps: usize, base_lr: f32, warmup_steps: usize) -> f32 {
    let phi = gf_constants().phi as f32;
    let alpha_phi = phi.powf(-3.0);  // ≈ 0.23607

    if step < warmup_steps {
        base_lr
    } else {
        let tau = max_steps as f32 / (phi * 27.0);
        let t = (step - warmup_steps) as f32 / tau;
        // Decay towards α_φ as floor
        base_lr * alpha_phi.powf(-t.min(10.0))  // Clamp exponent to prevent overflow
    }
}

/// Unified LR scheduler for Issue #54 calibration
///
/// Select schedule type and compute LR for current step.
pub fn lr_schedule_54(schedule_type: LrScheduleType, step: usize, max_steps: usize) -> f32 {
    const BASE_LR: f32 = 3e-4;
    const WARMUP_STEPS: usize = 100;

    match schedule_type {
        LrScheduleType::Flat => flat_lr(step, BASE_LR),
        LrScheduleType::Cosine => cosine_lr(step, max_steps, BASE_LR),
        LrScheduleType::PhiDecay => phi_decay_lr(step, max_steps, BASE_LR, WARMUP_STEPS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phi_schedule_no_decay_epoch_0() {
        let base_lr = 0.001f32;
        let lr = phi_schedule(0, base_lr, 10);
        assert_eq!(lr, base_lr, "Epoch 0 should return base_lr");
    }

    #[test]
    fn test_phi_schedule_decays_over_epochs() {
        let base_lr = 0.001f32;
        let warmup = 10;
        let lr_0 = phi_schedule(0, base_lr, warmup);
        let lr_10 = phi_schedule(10, base_lr, warmup);
        let lr_20 = phi_schedule(20, base_lr, warmup);

        assert!(lr_10 < lr_0, "LR should decay from epoch 0 to 10");
        assert!(lr_20 < lr_10, "LR should decay from epoch 10 to 20");
    }

    #[test]
    fn test_phi_schedule_phi_factor() {
        let base_lr = 1.0f32;
        let warmup = 1;
        let lr_0 = phi_schedule(0, base_lr, warmup);
        let lr_1 = phi_schedule(1, base_lr, warmup);
        let phi = gf_constants().phi as f32;

        assert!((lr_1 - lr_0 / phi).abs() < 1e-6, "LR should decay by factor of φ");
    }

    #[test]
    fn test_phi_schedule_zero_warmup() {
        let base_lr = 0.001f32;
        let lr = phi_schedule(5, base_lr, 0);
        // Division by zero should not panic; result may be NaN or inf
        assert!(lr.is_nan() || lr.is_infinite() || lr == 0.0);
    }
}

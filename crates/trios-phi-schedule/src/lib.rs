//! # trios-phi-schedule
//!
//! φ-LR scheduler — golden ratio-based learning rate schedule.
//!
//! ## Formula
//!
//! ```text
//! LR = base_lr * φ^(-epoch / warmup)
//! ```
//!
//! Where φ is the golden ratio (≈1.618...).

use trios_physics::gf_constants;

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

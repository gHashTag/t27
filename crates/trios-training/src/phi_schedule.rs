//! φ-Learning Rate Schedule

const PHI: f64 = 1.618033988749895;
const ALPHA_PHI: f64 = 0.1180339887498949; // Precomputed φ^(-3)/2

pub fn phi_lr(step: usize, max_steps: usize) -> f64 {
    let tau = max_steps as f64 / (PHI * 27.0);
    ALPHA_PHI * PHI.powf(-(step as f64) / tau)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phi_lr_at_zero() {
        assert!((phi_lr(0, 10000) - ALPHA_PHI).abs() < 1e-10);
    }
}

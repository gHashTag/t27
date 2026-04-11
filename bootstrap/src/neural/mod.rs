//! Neural Runtime — Phase 2 Implementation
//!
//! φ-RoPE (Rotary Position Embedding) using PHI^(-2i/d)
//! No external ML dependencies — pure f64 arithmetic based on golden ratio.

/// Golden ratio constant
pub const PHI: f64 = 1.6180339887498948_f64;

/// φ-RoPE: Rotary Position Embedding using PHI-based theta
///
/// Traditional RoPE uses: θ_i = 10000^(-2i/d)
/// φ-RoPE uses:      θ_i = PHI^(-2i/d) for i = 0, 1, ..., dim/2
pub struct PhiRoPE {
    /// Dimension of embeddings
    pub dim: usize,
    /// Precomputed theta values = PHI^(-2i/d)
    pub thetas: Vec<f64>,
}

impl PhiRoPE {
    /// Create new φ-RoPE with given dimension
    pub fn new(dim: usize) -> Self {
        let half_dim = dim / 2;
        let mut thetas = Vec::with_capacity(half_dim);
        for i in 0..half_dim {
            // θ_i = φ^(-2i/d) where d = dim
            let exponent = -2.0 * i as f64 / dim as f64;
            let theta = PHI.powf(exponent);
            thetas.push(theta);
        }
        Self { dim, thetas }
    }

    /// Apply φ-RoPE to a vector of embeddings
    ///
    /// For each position p, rotates pairs of dimensions by theta_i
    /// where i corresponds to the dimension index (0, 2, 4, ...)
    pub fn apply(&self, x: &mut [f64], position: usize) {
        for (i, theta) in self.thetas.iter().enumerate() {
            let angle = position as f64 * theta;
            let (sin_val, cos_val) = angle.sin_cos();
            let idx = i * 2;

            // Rotate pair of dimensions (idx, idx+1)
            if idx + 1 < x.len() {
                let x0 = x[idx];
                let x1 = x[idx + 1];
                x[idx] = x0 * cos_val - x1 * sin_val;
                x[idx + 1] = x0 * sin_val + x1 * cos_val;
            }
        }
    }

    /// Get the dimension of embeddings
    pub fn dim(&self) -> usize {
        self.dim
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phi_rope_thetas() {
        let rope = PhiRoPE::new(8);
        assert_eq!(rope.dim(), 8);
        assert_eq!(rope.thetas.len(), 4);

        // θ_0 = φ^(-2*0/8) = φ^0 = 1.0
        assert!((rope.thetas[0] - 1.0).abs() < 1e-10);

        // θ_1 = φ^(-2*1/8) = φ^(-1/4)
        let theta1 = PHI.powf(-0.25); // φ^(-1/4) ≈ 0.8867
        assert!((rope.thetas[1] - theta1).abs() < 1e-10);

        // θ_2 = φ^(-2*2/8) = φ^(-1/2)
        let theta2 = PHI.powf(-0.5); // φ^(-1/2) ≈ 0.618
        assert!((rope.thetas[2] - theta2).abs() < 1e-10);

        // θ_3 = φ^(-2*3/8) = φ^(-3/4)
        let theta3 = PHI.powf(-0.75); // φ^(-3/4) ≈ 0.486
        assert!((rope.thetas[3] - theta3).abs() < 1e-10);
    }

    #[test]
    fn test_phi_rope_apply() {
        let rope = PhiRoPE::new(4);

        // Test with position 0 (no rotation)
        let mut x = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64];
        rope.apply(&mut x, 0);
        assert_eq!(x, vec![1.0, 2.0, 3.0, 4.0]);

        // Test with position 1
        let mut x = vec![1.0_f64, 0.0_f64, 1.0_f64, 0.0_f64];
        rope.apply(&mut x, 1);
        let cos_theta1 = rope.thetas[0].cos();
        let sin_theta1 = rope.thetas[0].sin();
        // x[0] = x[0]*cos - x[1]*sin = 1*cosθ - 0*sinθ = cosθ
        // x[1] = x[0]*sin + x[1]*cos = 1*sinθ + 0*cosθ = sinθ
        assert!((x[0] - cos_theta1).abs() < 1e-10);
        assert!((x[1] - sin_theta1).abs() < 1e-10);
    }

    #[test]
    fn test_phi_rope_multi_position() {
        let rope = PhiRoPE::new(4);

        // Test that different positions use different angles
        // Position 1 uses angle = 1 * θ_i for each i
        let mut x1 = vec![1.0_f64, 0.0_f64, 1.0_f64, 0.0_f64];
        rope.apply(&mut x1, 1);

        // Position 2 uses angle = 2 * θ_i for each i
        let mut x2 = vec![1.0_f64, 0.0_f64, 1.0_f64, 0.0_f64];
        rope.apply(&mut x2, 2);

        // For dims 0,1 with initial [1,0]:
        // Position 1: rotated by θ_0 = 1.0 (since θ_0 = φ^0 = 1.0)
        let cos1 = (1.0 * rope.thetas[0]).cos();  // cos(1.0)
        let sin1 = (1.0 * rope.thetas[0]).sin();  // sin(1.0)
        assert!((x1[0] - cos1).abs() < 1e-10);
        assert!((x1[1] - sin1).abs() < 1e-10);

        // Position 2: rotated by 2*θ_0 = 2.0
        let cos2 = (2.0 * rope.thetas[0]).cos();  // cos(2.0)
        let sin2 = (2.0 * rope.thetas[0]).sin();  // sin(2.0)
        assert!((x2[0] - cos2).abs() < 1e-10);
        assert!((x2[1] - sin2).abs() < 1e-10);
    }
}

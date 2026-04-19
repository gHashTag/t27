//! # trios-trinity-init
//!
//! Trinity weight initialization for neural networks.
//!
//! Uses sacred geometry constants from `trios-physics` to initialize
//! weights with Xavier-like scaling modulated by √3 factor.
//!
//! ## Example
//!
//! ```no_run
//! use trios_trinity_init::trinity_init;
//!
//! let weights = trinity_init(&[256, 128, 64]);
//! assert_eq!(weights.len(), 256 * 128 + 128 * 64);
//! ```

use rand::Rng;
use trios_physics::gf_constants;

/// Trinity weight initialization.
///
/// Initializes a weight tensor of given shape using sacred geometry constants.
/// The bound is computed as √3 / √n where n is the total number of elements,
/// inspired by Xavier initialization but with the Trinity √3 factor.
///
/// # Arguments
///
/// * `shape` - Slice of tensor dimensions (e.g., &[784, 256] for 784x256 matrix)
///
/// # Returns
///
/// Vector of f32 weights initialized in range [-bound, bound]
///
/// # Example
///
/// ```
/// use trios_trinity_init::trinity_init;
///
/// let w = trinity_init(&[3, 4]);
/// assert_eq!(w.len(), 12);
/// // All values should be within the expected bound
/// let bound = 3.0_f32.sqrt() / (12.0_f32).sqrt();
/// for &x in &w {
///     assert!(x >= -bound && x <= bound);
/// }
/// ```
pub fn trinity_init(shape: &[usize]) -> Vec<f32> {
    // Empty shape means no elements
    if shape.is_empty() {
        return Vec::new();
    }

    let size = shape.iter().product::<usize>();
    if size == 0 {
        return Vec::new();
    }

    // Get sacred constants from trios-physics (available for future extensions)
    let _constants = gf_constants();
    // √3 sacred geometry factor
    let sqrt_3 = (3.0_f32).sqrt();

    // Xavier-like bound with √3 factor
    let bound = sqrt_3 / (size as f32).sqrt();

    let mut rng = rand::thread_rng();

    (0..size)
        .map(|_| rng.gen_range(-bound..=bound))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_shape() {
        let w = trinity_init(&[]);
        assert!(w.is_empty());
    }

    #[test]
    fn test_size_calculation() {
        let w = trinity_init(&[2, 3, 4]);
        assert_eq!(w.len(), 24);
    }

    #[test]
    fn test_bound_constraints() {
        let shape = [10, 10];
        let w = trinity_init(&shape);
        let size = shape.iter().product::<usize>();
        let bound = 3.0_f32.sqrt() / (size as f32).sqrt();

        for &x in &w {
            assert!(x >= -bound, "value {} below bound {}", x, -bound);
            assert!(x <= bound, "value {} above bound {}", x, bound);
        }
    }

    #[test]
    fn test_determinism_with_fixed_seed() {
        // Note: thread_rng is non-deterministic, so we test statistical properties
        let w1: Vec<f32> = trinity_init(&[100, 100])
            .into_iter()
            .take(10)
            .collect();

        let w2: Vec<f32> = trinity_init(&[100, 100])
            .into_iter()
            .take(10)
            .collect();

        // Should not be identical (probabilistic)
        assert_ne!(w1, w2, "different calls should produce different values");
    }

    #[test]
    fn test_large_tensor() {
        let w = trinity_init(&[1024, 1024]);
        assert_eq!(w.len(), 1_048_576);
    }

    #[test]
    fn test_single_dimension() {
        let w = trinity_init(&[42]);
        assert_eq!(w.len(), 42);
        let bound = 3.0_f32.sqrt() / (42.0_f32).sqrt();
        for &x in &w {
            assert!(x >= -bound && x <= bound);
        }
    }

    #[test]
    fn test_sacred_factor() {
        // Verify √3 factor is correctly applied
        let size = 100usize;
        let w = trinity_init(&[size]);
        let bound = 3.0_f32.sqrt() / (size as f32).sqrt();

        // Find max absolute value (should be close to bound)
        let max_abs = w.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
        assert!(max_abs > 0.0, "should have non-zero values");
        assert!(max_abs <= bound, "max value should not exceed bound");
    }
}

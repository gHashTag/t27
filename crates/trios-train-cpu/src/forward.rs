//! CPU forward pass for IGLA-GF16
//!
//! Pure Rust matrix multiplication with no BLAS dependency.
//! Optimized for CPU with small batch sizes and cache-friendly access patterns.

use std::fmt;

/// Layer dimensions for IGLA-GF16
#[derive(Debug, Clone, Copy)]
pub struct LayerDims {
    pub d_model: usize,
    pub n_heads: usize,
    pub d_ffn: usize,
}

impl Default for LayerDims {
    fn default() -> Self {
        // IGLA-GF16 Fibonacci architecture
        Self {
            d_model: 144,   // Fibonacci number
            n_heads: 8,     // 2^3
            d_ffn: 233,     // Next Fibonacci number after 144
        }
    }
}

/// CPU matrix multiplication (pure Rust, no BLAS)
///
/// Computes C = A @ B where:
/// - A is (m, k)
/// - B is (k, n)
/// - C is (m, n)
///
/// # Arguments
///
/// * `a` - Input matrix A (row-major, size m*k)
/// * `b` - Input matrix B (row-major, size k*n)
/// * `c` - Output matrix C (row-major, size m*n)
/// * `m` - Rows of A / C
/// * `k` - Columns of A / rows of B (inner dimension)
/// * `n` - Columns of B / C
///
/// # Example
///
/// ```
/// use trios_train_cpu::forward::matmul;
///
/// let a = vec![1.0f32, 2.0, 3.0, 4.0]; // 2x2
/// let b = vec![2.0f32, 0.0, 1.0, 2.0]; // 2x2
/// let mut c = vec![0.0f32; 4];
///
/// matmul(&a, &b, &mut c, 2, 2, 2);
///
/// // C = [[1*2 + 2*1, 1*0 + 2*2],
/// //      [3*2 + 4*1, 3*0 + 4*2]]
/// //   = [[4, 4], [10, 8]]
/// assert_eq!(c[0], 4.0);
/// assert_eq!(c[1], 4.0);
/// assert_eq!(c[2], 10.0);
/// assert_eq!(c[3], 8.0);
/// ```
pub fn matmul(a: &[f32], b: &[f32], c: &mut [f32], m: usize, k: usize, n: usize) {
    // Simple triple-loop with cache-friendly ordering
    // This is optimized for readability; future optimizations can include:
    // - Loop tiling for cache efficiency
    // - SIMD intrinsics for ARM/AVX
    // - Parallelization with rayon

    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0f32;
            let a_row_offset = i * k;
            let c_idx = i * n + j;

            // Inner loop over k dimension
            for l in 0..k {
                // a[i, l] * b[l, j]
                // b is stored row-major, so b[l, j] is at l * n + j
                sum += unsafe {
                    // Bounds check elision: we trust caller provides valid indices
                    let a_val = *a.get_unchecked(a_row_offset + l);
                    let b_val = *b.get_unchecked(l * n + j);
                    a_val * b_val
                };
            }
            c[c_idx] = sum;
        }
    }
}

/// Vector addition: y = x + y (in-place)
///
/// # Arguments
///
/// * `x` - Input vector (size n)
/// * `y` - Input/output vector (size n), modified in-place
pub fn vec_add(x: &[f32], y: &mut [f32]) {
    assert_eq!(x.len(), y.len(), "vector dimensions must match");
    for i in 0..x.len() {
        y[i] += x[i];
    }
}

/// Vector scaling: y = x * scale
///
/// # Arguments
///
/// * `x` - Input vector (size n)
/// * `y` - Output vector (size n)
/// * `scale` - Scaling factor
pub fn vec_scale(x: &[f32], y: &mut [f32], scale: f32) {
    assert_eq!(x.len(), y.len(), "vector dimensions must match");
    for i in 0..x.len() {
        y[i] = x[i] * scale;
    }
}

/// GELU activation function
///
/// GELU(x) = x * Φ(x) where Φ is the Gaussian CDF.
/// Uses the approximation: GELU(x) ≈ 0.5 * x * (1 + tanh(√(2/π) * (x + 0.044715 * x³)))
///
/// # Arguments
///
/// * `x` - Input vector (modified in-place)
pub fn gelu(x: &mut [f32]) {
    const SQRT_2_OVER_PI: f32 = 0.797_884_6_f32;  // √(2/π)
    const BETA: f32 = 0.044715f32;

    for xi in x.iter_mut() {
        let x3 = *xi * *xi * *xi;
        let tanh_arg = SQRT_2_OVER_PI * (*xi + BETA * x3);
        let tanh_val = tanh_arg.tanh();
        *xi = 0.5 * *xi * (1.0 + tanh_val);
    }
}

/// Layer normalization (in-place)
///
/// # Arguments
///
/// * `x` - Input/output vector (size n), modified in-place
/// * `eps` - Small constant for numerical stability
pub fn layer_norm(x: &mut [f32], eps: f32) {
    let n = x.len();

    // Compute mean
    let sum: f32 = x.iter().sum();
    let mean = sum / n as f32;

    // Compute variance
    let var_sum: f32 = x.iter().map(|&xi| {
        let diff = xi - mean;
        diff * diff
    }).sum();
    let var = var_sum / n as f32;
    let std = (var + eps).sqrt();

    // Normalize
    for xi in x.iter_mut() {
        *xi = (*xi - mean) / std;
    }
}

/// Softmax activation (in-place)
///
/// # Arguments
///
/// * `x` - Input/output vector (size n), modified in-place
pub fn softmax(x: &mut [f32]) {
    // Find max for numerical stability
    let max_x = x.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

    // Compute exp and sum
    let mut sum = 0.0f32;
    for xi in x.iter_mut() {
        *xi = (*xi - max_x).exp();
        sum += *xi;
    }

    // Normalize
    let inv_sum = if sum > 0.0 { 1.0 / sum } else { 1.0 };
    for xi in x.iter_mut() {
        *xi *= inv_sum;
    }
}

/// Forward pass context for a single layer
pub struct ForwardContext {
    pub dims: LayerDims,
    pub activations: Vec<Vec<f32>>,
}

impl ForwardContext {
    pub fn new(dims: LayerDims) -> Self {
        Self {
            dims,
            activations: Vec::new(),
        }
    }

    pub fn store_activation(&mut self, activation: Vec<f32>) {
        self.activations.push(activation);
    }
}

impl fmt::Debug for ForwardContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ForwardContext")
            .field("dims", &self.dims)
            .field("num_activations", &self.activations.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matmul_2x2() {
        let a = vec![1.0f32, 2.0, 3.0, 4.0];
        let b = vec![2.0f32, 0.0, 1.0, 2.0];
        let mut c = vec![0.0f32; 4];

        matmul(&a, &b, &mut c, 2, 2, 2);

        assert!((c[0] - 4.0).abs() < 1e-6);
        assert!((c[1] - 4.0).abs() < 1e-6);
        assert!((c[2] - 10.0).abs() < 1e-6);
        assert!((c[3] - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_matmul_rectangular() {
        // A: 2x3, B: 3x4, C: 2x4
        let a = vec![
            1.0, 2.0, 3.0,
            4.0, 5.0, 6.0,
        ];
        let b = vec![
            1.0, 0.0, 1.0, 0.0,
            0.0, 1.0, 0.0, 1.0,
            1.0, 1.0, 1.0, 1.0,
        ];
        let mut c = vec![0.0f32; 8];

        matmul(&a, &b, &mut c, 2, 3, 4);

        // Row 0: [1*1+2*0+3*1, 1*0+2*1+3*1, 1*1+2*0+3*1, 1*0+2*1+3*1]
        //       = [4, 5, 4, 5]
        assert!((c[0] - 4.0).abs() < 1e-6);
        assert!((c[1] - 5.0).abs() < 1e-6);
        assert!((c[2] - 4.0).abs() < 1e-6);
        assert!((c[3] - 5.0).abs() < 1e-6);

        // Row 1: [4*1+5*0+6*1, 4*0+5*1+6*1, 4*1+5*0+6*1, 4*0+5*1+6*1]
        //       = [10, 11, 10, 11]
        assert!((c[4] - 10.0).abs() < 1e-6);
        assert!((c[5] - 11.0).abs() < 1e-6);
        assert!((c[6] - 10.0).abs() < 1e-6);
        assert!((c[7] - 11.0).abs() < 1e-6);
    }

    #[test]
    fn test_vec_add() {
        let x = vec![1.0, 2.0, 3.0];
        let mut y = vec![4.0, 5.0, 6.0];
        vec_add(&x, &mut y);
        assert_eq!(y, vec![5.0, 7.0, 9.0]);
    }

    #[test]
    fn test_vec_scale() {
        let x = vec![1.0, 2.0, 3.0];
        let mut y = vec![0.0; 3];
        vec_scale(&x, &mut y, 2.5);
        assert_eq!(y, vec![2.5, 5.0, 7.5]);
    }

    #[test]
    fn test_gelu() {
        let mut x = vec![0.0, 1.0, -1.0];
        gelu(&mut x);

        // GELU(0) ≈ 0
        assert!((x[0] - 0.0).abs() < 0.01);
        // GELU(1) ≈ 0.84... (close to input due to being in linear-ish region)
        assert!(x[1] > 0.5 && x[1] < 1.0);
        // GELU(-1) ≈ -0.15... (negative but closer to 0 than input)
        assert!(x[2] < 0.0 && x[2] > -0.5);
    }

    #[test]
    fn test_layer_norm() {
        let mut x = vec![1.0, 2.0, 3.0, 4.0];
        layer_norm(&mut x, 1e-5);

        // Mean should be ~0
        let mean: f32 = x.iter().sum::<f32>() / x.len() as f32;
        assert!(mean.abs() < 1e-5);

        // Std should be ~1
        let var: f32 = x.iter().map(|&xi| xi * xi).sum::<f32>() / x.len() as f32;
        assert!((var - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_softmax() {
        let mut x = vec![1.0, 2.0, 3.0];
        softmax(&mut x);

        // Sum should be 1
        let sum: f32 = x.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);

        // Values should be positive and ordered
        assert!(x[0] > 0.0 && x[1] > 0.0 && x[2] > 0.0);
        assert!(x[0] < x[1] && x[1] < x[2]);
    }

    #[test]
    fn test_layer_dims_default() {
        let dims = LayerDims::default();
        assert_eq!(dims.d_model, 144);
        assert_eq!(dims.n_heads, 8);
        assert_eq!(dims.d_ffn, 233);
    }
}

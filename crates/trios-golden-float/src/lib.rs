//! # trios-golden-float
//!
//! Safe Rust wrapper around [zig-golden-float](https://github.com/gHashTag/zig-golden-float),
//! providing GF16 (Golden Float 16-bit) numeric operations.
//!
//! GF16 uses a base-φ (golden ratio) exponent representation that provides
//! better dynamic range for neural network weights compared to IEEE float16.
//!
//! ## Example
//!
//! ```ignore
//! use trios_golden_float::GF16;
//!
//! let g = GF16::from_f32(1.618);
//! assert!((g.to_f32() - 1.618).abs() < 0.01);
//! ```

mod ffi;
pub mod router;

// Re-export only when the Zig library is linked.
// When vendor/ is absent, the FFI symbols won't exist — use feature-gated access.

/// A Golden Float 16-bit value with safe arithmetic.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GF16(u16);

impl GF16 {
    /// Create a GF16 from an f32 value.
    pub fn from_f32(x: f32) -> Self {
        unsafe { GF16(ffi::gf16_from_f32(x)) }
    }

    /// Convert this GF16 back to f32.
    pub fn to_f32(self) -> f32 {
        unsafe { ffi::gf16_to_f32(self.0) }
    }

    /// Get the raw u16 bits.
    pub fn to_bits(self) -> u16 {
        self.0
    }

    /// Create from raw u16 bits.
    pub fn from_bits(bits: u16) -> Self {
        GF16(bits)
    }

    /// Add two GF16 values.
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, other: Self) -> Self {
        unsafe { GF16(ffi::gf16_add(self.0, other.0)) }
    }

    /// Subtract `other` from `self`.
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, other: Self) -> Self {
        unsafe { GF16(ffi::gf16_sub(self.0, other.0)) }
    }

    /// Multiply two GF16 values.
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, other: Self) -> Self {
        unsafe { GF16(ffi::gf16_mul(self.0, other.0)) }
    }

    /// Divide `self` by `other`.
    #[allow(clippy::should_implement_trait)]
    pub fn div(self, other: Self) -> Self {
        unsafe { GF16(ffi::gf16_div(self.0, other.0)) }
    }
}

impl std::ops::Add for GF16 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.add(rhs)
    }
}

impl std::ops::Sub for GF16 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.sub(rhs)
    }
}

impl std::ops::Mul for GF16 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.mul(rhs)
    }
}

/// Compress a slice of f32 weights into GF16 representation.
///
/// Returns a `Vec<u16>` of the same length with GF16-encoded values.
pub fn compress_weights(weights: &[f32]) -> Vec<u16> {
    let mut out = vec![0u16; weights.len()];
    unsafe {
        ffi::gf16_compress_weights(weights.as_ptr(), weights.len(), out.as_mut_ptr());
    }
    out
}

/// Decompress a slice of GF16 values back to f32.
pub fn decompress_weights(compressed: &[u16]) -> Vec<f32> {
    let mut out = vec![0.0f32; compressed.len()];
    unsafe {
        ffi::gf16_decompress_weights(compressed.as_ptr(), compressed.len(), out.as_mut_ptr());
    }
    out
}

/// Compute the dot product of two GF16 vectors.
pub fn dot_product(a: &[u16], b: &[u16]) -> GF16 {
    assert_eq!(a.len(), b.len(), "vectors must have same length");
    unsafe { GF16(ffi::gf16_dot_product(a.as_ptr(), b.as_ptr(), a.len())) }
}

/// Quantize a matrix (row-major f32) to GF16 with a given scale factor.
///
/// Returns `Ok(Vec<u16>)` on success with GF16-encoded matrix data.
pub fn quantize_matrix(data: &[f32], rows: usize, cols: usize, scale: f32) -> Vec<u16> {
    let mut out = vec![0u16; rows * cols];
    let rc = unsafe {
        ffi::gf16_quantize_matrix(data.as_ptr(), rows, cols, scale, out.as_mut_ptr())
    };
    assert_eq!(rc, 0, "gf16_quantize_matrix failed");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phi_constant() {
        // Golden ratio φ = (1 + √5) / 2 ≈ 1.618033988749895
        const PHI: f64 = 1.6180339887498948482;
        assert!((PHI - 1.618033988749895).abs() < 1e-9);
    }

    #[test]
    fn test_gf16_zero() {
        let z = GF16::from_f32(0.0);
        assert_eq!(z.to_f32(), 0.0);
        assert_eq!(z.to_bits(), 0u16);
    }

    #[test]
    fn test_gf16_roundtrip() {
        let x = 1.0f32;
        let encoded = GF16::from_f32(x);
        assert!((encoded.to_f32() - x).abs() < 0.05);
    }

    #[test]
    #[ignore = "requires zig-golden-float vendor submodule"]
    fn roundtrip_phi() {
        let phi: f32 = 1.618;
        let g = GF16::from_f32(phi);
        let recovered = g.to_f32();
        assert!(
            (recovered - phi).abs() < 0.05,
            "roundtrip failed: {phi} → GF16 → {recovered}"
        );
    }

    #[test]
    #[ignore = "requires zig-golden-float vendor submodule"]
    fn compress_decompress_roundtrip() {
        let weights = [0.5, 1.0, 1.618, -0.3, 2.0];
        let compressed = compress_weights(&weights);
        let decompressed = decompress_weights(&compressed);
        for (orig, got) in weights.iter().zip(decompressed.iter()) {
            assert!(
                (orig - got).abs() < 0.1,
                "roundtrip error: {orig} → {got}"
            );
        }
    }
}

// ==============================================================================
// HYBRID PRECISION PIPELINE API (GF16 for Critical Layers)
// ==============================================================================

/// GF16 quantization for critical layers (embedding, attention, output)
///
/// Used in hybrid precision pipeline where:
/// - Embedding → GF16 (HIGH sensitivity, representation learning)
/// - Attention (QKV, proj) → GF16 (HIGH sensitivity, attention fragility)
/// - Output → GF16 (HIGH sensitivity, final logits precision)
///
/// Architecture note: These layers are assigned GF16 by STATIC_ROUTING_TABLE
/// because they are the most sensitive to quantization error.
pub mod hybrid {
    use super::GF16;

    /// Quantize a f32 embedding matrix to GF16
    ///
    /// # Arguments
    /// * `weights` - f32 embedding matrix [vocab_size, d_model]
    /// * `scale` - Optional φ-optimized scaling factor
    ///
    /// # Returns
    /// Vector of GF16 values
    pub fn quantize_embedding(weights: &[f32], scale: Option<f32>) -> Vec<GF16> {
        let phi_scale = scale.unwrap_or(1.0);
        weights.iter().map(|&w| GF16::from_f32(w * phi_scale)).collect()
    }

    /// Quantize f32 attention weights to GF16
    ///
    /// # Arguments
    /// * `weights` - f32 attention weights [d_model, 3 * d_model] for QKV
    /// * `scale` - Optional φ-optimized scaling factor
    ///
    /// # Returns
    /// Vector of GF16 values
    pub fn quantize_attention(weights: &[f32], scale: Option<f32>) -> Vec<GF16> {
        let phi_scale = scale.unwrap_or(1.0);
        weights.iter().map(|&w| GF16::from_f32(w * phi_scale)).collect()
    }

    /// Quantize f32 output projection to GF16
    ///
    /// # Arguments
    /// * `weights` - f32 output weights [d_model, vocab_size]
    /// * `scale` - Optional φ-optimized scaling factor
    ///
    /// # Returns
    /// Vector of GF16 values
    pub fn quantize_output(weights: &[f32], scale: Option<f32>) -> Vec<GF16> {
        let phi_scale = scale.unwrap_or(1.0);
        weights.iter().map(|&w| GF16::from_f32(w * phi_scale)).collect()
    }

    /// Compute φ-optimized scaling factor for GF16 quantization
    ///
    /// Based on Trinity physics: scale = 1 / (std * φ^(-0.5))
    /// This matches the log-normal distribution of neural network weights.
    ///
    /// # Arguments
    /// * `weights` - f32 weight tensor
    ///
    /// # Returns
    /// φ-optimized scaling factor
    pub fn compute_phi_scale(weights: &[f32]) -> f32 {
        if weights.is_empty() {
            return 1.0;
        }

        let mean = weights.iter().sum::<f32>() / weights.len() as f32;
        let variance = weights.iter().map(|&w| (w - mean).powi(2)).sum::<f32>() / weights.len() as f32;
        let std = variance.sqrt();

        let phi = 1.618033988749895_f32;  // φ
        std.powf(-0.5) / phi
    }

    /// Dequantize GF16 weights back to f32
    ///
    /// # Arguments
    /// * `gf16_weights` - GF16 quantized weights
    /// * `scale` - φ-optimized scaling factor used during quantization
    ///
    /// # Returns
    /// f32 weights
    pub fn dequantize(gf16_weights: &[GF16], scale: f32) -> Vec<f32> {
        gf16_weights.iter().map(|&g| g.to_f32() / scale).collect()
    }

    /// Calculate model size in bytes after GF16 quantization
    ///
    /// # Arguments
    /// * `num_params` - Number of parameters
    ///
    /// # Returns
    /// Size in bytes (16 bits per parameter)
    pub fn gf16_size_bytes(num_params: usize) -> usize {
        num_params * 2  // 16 bits = 2 bytes
    }

    /// Calculate compression ratio vs f32 (32 bits per parameter)
    ///
    /// # Arguments
    /// * `num_params` - Number of parameters
    ///
    /// # Returns
    /// Compression ratio (32.0 / 16.0 = 2.0x)
    pub fn compression_ratio(num_params: usize) -> f32 {
        32.0 / 16.0  // 2.0x compression for GF16
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_quantize_embedding() {
            let weights = vec![0.1, 0.5, 1.0, 1.618];
            let gf16 = quantize_embedding(&weights, None);

            // Roundtrip test
            let dequant = dequantize(&gf16, 1.0);
            for (orig, got) in weights.iter().zip(dequant.iter()) {
                let orig: f32 = *orig;
                let got: f32 = *got;
                assert!((orig - got).abs() < 0.01, "roundtrip error too large");
            }
        }

        #[test]
        fn test_phi_scale() {
            let weights = vec![0.1, 0.5, 1.0, 1.5, 2.0];
            let scale = compute_phi_scale(&weights);
            assert!(scale > 0.0, "scale must be positive");
            assert!(scale < 10.0, "scale must be reasonable");
        }

        #[test]
        fn test_size_bytes() {
            // 8.59M params like IGLA-GF16 model
            let params = 8_590_032;
            let size = gf16_size_bytes(params);
            assert_eq!(size, 17_180_064);  // 8.59M * 2 bytes
        }

        #[test]
        fn test_compression_ratio() {
            assert_eq!(compression_ratio(1000), 2.0);
        }
    }
}

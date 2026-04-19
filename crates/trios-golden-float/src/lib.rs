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

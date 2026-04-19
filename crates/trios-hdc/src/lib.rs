//! # trios-hdc
//!
//! Safe Rust wrapper around [zig-hdc](https://github.com/gHashTag/zig-hdc),
//! providing Hyperdimensional Computing (HDC) / Vector Symbolic Architecture (VSA)
//! operations for AI agents.
//!
//! HDC operates on high-dimensional vectors (typically D=10000) where:
//! - **Binding** (XOR/multiply) associates two vectors — invertible
//! - **Bundling** (majority/superpose) aggregates vectors — robust to noise
//! - **Permutation** (rotation) provides positional encoding
//!
//! ## Example
//!
//! ```ignore
//! use trios_hdc::HdcSpace;
//!
//! let space = HdcSpace::new(10000);
//! let a = space.random_vector();
//! let b = space.random_vector();
//! let bound = space.bind(&a, &b);
//! let sim = space.similarity(&bound, &a); // should be ~0.5
//! ```

mod ffi;

use std::marker::PhantomData;

/// Dimensionality for hypervectors. D=10000 is standard.
pub const DEFAULT_DIMENSIONS: usize = 10_000;

/// An HDC space that manages dimensionality and random vector generation.
pub struct HdcSpace {
    ptr: *mut ffi::HdcSpace,
    dimensions: usize,
}

// The Zig HDC space has no thread-local state (RNG is per-space).
unsafe impl Send for HdcSpace {}
unsafe impl Sync for HdcSpace {}

impl HdcSpace {
    /// Create a new HDC space with the given dimensionality.
    pub fn new(dimensions: usize) -> Self {
        let ptr = unsafe { ffi::hdc_space_create(dimensions) };
        assert!(!ptr.is_null(), "failed to create HDC space");
        HdcSpace { ptr, dimensions }
    }

    /// Get the dimensionality of this space.
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Generate a random hypervector.
    pub fn random_vector(&self) -> Hypervector {
        let mut data = vec![0u32; self.dimensions];
        let rc = unsafe { ffi::hdc_random_vector(self.ptr, data.as_mut_ptr()) };
        assert_eq!(rc, 0, "hdc_random_vector failed");
        Hypervector {
            data,
            _marker: PhantomData,
        }
    }

    /// Bind two hypervectors (XOR for binary VSA).
    pub fn bind(&self, a: &Hypervector, b: &Hypervector) -> Hypervector {
        let mut out = vec![0u32; self.dimensions];
        let rc = unsafe {
            ffi::hdc_bind(self.ptr, a.data.as_ptr(), b.data.as_ptr(), out.as_mut_ptr())
        };
        assert_eq!(rc, 0, "hdc_bind failed");
        Hypervector {
            data: out,
            _marker: PhantomData,
        }
    }

    /// Bundle (aggregate) two hypervectors via majority rule.
    pub fn bundle(&self, a: &Hypervector, b: &Hypervector) -> Hypervector {
        let mut out = vec![0u32; self.dimensions];
        let rc = unsafe {
            ffi::hdc_bundle(self.ptr, a.data.as_ptr(), b.data.as_ptr(), out.as_mut_ptr())
        };
        assert_eq!(rc, 0, "hdc_bundle failed");
        Hypervector {
            data: out,
            _marker: PhantomData,
        }
    }

    /// Compute cosine similarity between two hypervectors.
    pub fn similarity(&self, a: &Hypervector, b: &Hypervector) -> f64 {
        unsafe { ffi::hdc_similarity(self.ptr, a.data.as_ptr(), b.data.as_ptr()) }
    }

    /// Permute (rotate) a hypervector by `shift` positions.
    pub fn permute(&self, v: &Hypervector, shift: usize) -> Hypervector {
        let mut out = vec![0u32; self.dimensions];
        let rc = unsafe { ffi::hdc_permute(self.ptr, v.data.as_ptr(), shift, out.as_mut_ptr()) };
        assert_eq!(rc, 0, "hdc_permute failed");
        Hypervector {
            data: out,
            _marker: PhantomData,
        }
    }

    /// Encode a scalar value into a level hypervector.
    pub fn encode_level(&self, value: f64, min_val: f64, max_val: f64) -> Hypervector {
        let mut out = vec![0u32; self.dimensions];
        let rc = unsafe {
            ffi::hdc_encode_level(self.ptr, value, min_val, max_val, out.as_mut_ptr())
        };
        assert_eq!(rc, 0, "hdc_encode_level failed");
        Hypervector {
            data: out,
            _marker: PhantomData,
        }
    }

    /// Encode a record (multiple scalar values) into a single hypervector.
    pub fn encode_record(&self, values: &[f64], min_val: f64, max_val: f64) -> Hypervector {
        let mut out = vec![0u32; self.dimensions];
        let rc = unsafe {
            ffi::hdc_encode_record(
                self.ptr,
                values.as_ptr(),
                values.len(),
                min_val,
                max_val,
                out.as_mut_ptr(),
            )
        };
        assert_eq!(rc, 0, "hdc_encode_record failed");
        Hypervector {
            data: out,
            _marker: PhantomData,
        }
    }
}

impl Drop for HdcSpace {
    fn drop(&mut self) {
        unsafe { ffi::hdc_space_destroy(self.ptr) };
    }
}

/// A hypervector in an HDC space.
pub struct Hypervector {
    data: Vec<u32>,
    _marker: PhantomData<*const ()>,
}

impl Hypervector {
    /// Get the raw u32 data slice.
    pub fn as_slice(&self) -> &[u32] {
        &self.data
    }

    /// Get the dimensionality.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the hypervector is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires zig-hdc vendor submodule"]
    fn random_vectors_are_dissimilar() {
        let space = HdcSpace::new(DEFAULT_DIMENSIONS);
        let a = space.random_vector();
        let b = space.random_vector();
        let sim = space.similarity(&a, &b);
        assert!(
            sim.abs() < 0.1,
            "random vectors should be nearly orthogonal, got similarity={sim}"
        );
    }

    #[test]
    #[ignore = "requires zig-hdc vendor submodule"]
    fn bind_is_self_inverse() {
        let space = HdcSpace::new(DEFAULT_DIMENSIONS);
        let a = space.random_vector();
        let b = space.random_vector();
        let bound = space.bind(&a, &b);
        let recovered = space.bind(&bound, &b);
        let sim = space.similarity(&a, &recovered);
        assert!(
            sim > 0.9,
            "binding should be self-inverse, got similarity={sim}"
        );
    }
}

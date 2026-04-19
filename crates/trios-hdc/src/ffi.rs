//! Raw FFI declarations for zig-hdc C API.
//!
//! Hyperdimensional Computing (HDC) / Vector Symbolic Architecture (VSA)
//! operations for encoding, binding, and querying high-dimensional vectors.

use libc::{c_int, size_t};

/// Opaque handle for an HDC space (configuration + RNG state).
pub type HdcSpace = c_int;

extern "C" {
    /// Create a new HDC space with the given dimensionality (e.g. 10000).
    pub fn hdc_space_create(dimensions: size_t) -> *mut HdcSpace;

    /// Destroy an HDC space, freeing memory.
    pub fn hdc_space_destroy(space: *mut HdcSpace);

    /// Generate a random hypervector (binary or bipolar).
    /// `out` must have space for `dimensions` elements.
    pub fn hdc_random_vector(space: *mut HdcSpace, out: *mut u32) -> c_int;

    /// Bind two hypervectors (XOR for binary, multiply for bipolar).
    /// Result is written to `out`.
    pub fn hdc_bind(
        space: *mut HdcSpace,
        a: *const u32,
        b: *const u32,
        out: *mut u32,
    ) -> c_int;

    /// Bundle (superpose/aggregate) two hypervectors.
    /// Result is written to `out`.
    pub fn hdc_bundle(
        space: *mut HdcSpace,
        a: *const u32,
        b: *const u32,
        out: *mut u32,
    ) -> c_int;

    /// Compute cosine similarity between two hypervectors.
    pub fn hdc_similarity(space: *mut HdcSpace, a: *const u32, b: *const u32) -> f64;

    /// Permute (rotate) a hypervector by `shift` positions.
    pub fn hdc_permute(
        space: *mut HdcSpace,
        vec: *const u32,
        shift: size_t,
        out: *mut u32,
    ) -> c_int;

    /// Encode a scalar value into a level hypervector (for continuous values).
    pub fn hdc_encode_level(
        space: *mut HdcSpace,
        value: f64,
        min_val: f64,
        max_val: f64,
        out: *mut u32,
    ) -> c_int;

    /// Encode a record (array of scalar values) into a single hypervector.
    pub fn hdc_encode_record(
        space: *mut HdcSpace,
        values: *const f64,
        len: size_t,
        min_val: f64,
        max_val: f64,
        out: *mut u32,
    ) -> c_int;

    /// Get the dimensionality of the HDC space.
    #[allow(dead_code)]
    pub fn hdc_space_dimensions(space: *mut HdcSpace) -> size_t;
}

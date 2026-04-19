//! Raw FFI declarations for zig-golden-float C API.

use libc::{c_int, size_t};

extern "C" {
    /// Convert an f32 value to GF16 (golden float 16-bit) representation.
    pub fn gf16_from_f32(x: f32) -> u16;

    /// Convert a GF16 value back to f32.
    pub fn gf16_to_f32(x: u16) -> f32;

    /// Add two GF16 values.
    pub fn gf16_add(a: u16, b: u16) -> u16;

    /// Multiply two GF16 values.
    pub fn gf16_mul(a: u16, b: u16) -> u16;

    /// Subtract `b` from `a` in GF16.
    pub fn gf16_sub(a: u16, b: u16) -> u16;

    /// Divide `a` by `b` in GF16.
    pub fn gf16_div(a: u16, b: u16) -> u16;

    /// Compress an array of f32 weights into GF16.
    /// Returns the number of elements written to `out`.
    pub fn gf16_compress_weights(weights: *const f32, len: size_t, out: *mut u16) -> size_t;

    /// Decompress an array of GF16 values back to f32.
    pub fn gf16_decompress_weights(compressed: *const u16, len: size_t, out: *mut f32) -> size_t;

    /// Compute dot product of two GF16 vectors.
    pub fn gf16_dot_product(
        a: *const u16,
        b: *const u16,
        len: size_t,
    ) -> u16;

    /// Apply GF16 quantization to a matrix with given scale factor.
    pub fn gf16_quantize_matrix(
        data: *const f32,
        rows: size_t,
        cols: size_t,
        scale: f32,
        out: *mut u16,
    ) -> c_int;

    /// Internal: compress weights (underscore prefix for internal FFI).
    #[doc(hidden)]
    pub fn _gf16_compress_weights(weights: *const f32, len: size_t, out: *mut u16) -> size_t;

    /// Internal: decompress weights (underscore prefix for internal FFI).
    #[doc(hidden)]
    pub fn _gf16_decompress_weights(compressed: *const u16, len: size_t, out: *mut f32) -> size_t;
}

//! Raw FFI declarations for zig-sacred-geometry C API.

use libc::{c_int, size_t};

/// Result of a Beal conjecture search step.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BealCandidate {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub m: u32,
    pub n: u32,
    pub r: u32,
    /// Whether this candidate satisfies A^m + B^n = C^r with gcd(A,B,C) > 1
    pub valid: bool,
}

extern "C" {
    /// Compute φ-weighted attention matrix.
    /// `queries` and `keys` are seq_len × dim matrices (row-major).
    /// Returns attention weights matrix.
    pub fn sacred_phi_attention(
        queries: *const f64,
        keys: *const f64,
        seq_len: size_t,
        dim: size_t,
        phi_factor: f64,
        out_weights: *mut f64,
    ) -> c_int;

    /// Compute Fibonacci spiral point at parameter t.
    pub fn sacred_fibonacci_spiral(t: f64, out_x: *mut f64, out_y: *mut f64);

    /// Generate golden ratio-spaced sequence of n values in [0, 1].
    pub fn sacred_golden_sequence(n: size_t, out: *mut f64) -> c_int;

    /// Search for Beal conjecture counterexamples in range [min_base, max_base].
    /// `candidates` must have space for `max_results` entries.
    /// Returns actual number of candidates found.
    pub fn sacred_beal_search(
        min_base: u64,
        max_base: u64,
        max_exp: u32,
        candidates: *mut BealCandidate,
        max_results: size_t,
    ) -> size_t;

    /// Compute φ-dimensional bottleneck dimension for a given model width.
    /// Returns the nearest Fibonacci number that serves as bottleneck dim.
    pub fn sacred_phi_bottleneck(model_dim: size_t) -> size_t;

    /// Compute sacred geometry ratio (φ^n mod 1) for attention head spacing.
    pub fn sacred_head_spacing(n_heads: size_t, out_spacing: *mut f64) -> c_int;

}

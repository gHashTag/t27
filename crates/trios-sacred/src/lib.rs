//! # trios-sacred
//!
//! Safe Rust wrapper around [zig-sacred-geometry](https://github.com/gHashTag/zig-sacred-geometry),
//! providing sacred geometry primitives: φ-attention, Fibonacci spirals, golden sequences,
//! and Beal conjecture search.
//!
//! ## Example
//!
//! ```ignore
//! use trios_sacred::{phi_attention, golden_sequence, beal_search};
//!
//! let seq = golden_sequence(10);
//! let candidates = beal_search(2, 100, 10, 100);
//! ```

mod ffi;

pub use ffi::BealCandidate;

/// Compute φ-weighted attention matrix for given queries and keys.
///
/// - `queries`: seq_len × dim matrix (row-major)
/// - `keys`: seq_len × dim matrix (row-major)
/// - `phi_factor`: golden ratio weighting (typically 1.618)
///
/// Returns the attention weight matrix (seq_len × seq_len, row-major).
pub fn phi_attention(
    queries: &[f64],
    keys: &[f64],
    seq_len: usize,
    dim: usize,
    phi_factor: f64,
) -> Result<Vec<f64>, String> {
    let expected = seq_len * dim;
    if queries.len() != expected || keys.len() != expected {
        return Err(format!(
            "dimension mismatch: queries={}, keys={}, expected={}",
            queries.len(),
            keys.len(),
            expected
        ));
    }
    let mut out = vec![0.0f64; seq_len * seq_len];
    let rc = unsafe {
        ffi::sacred_phi_attention(
            queries.as_ptr(),
            keys.as_ptr(),
            seq_len,
            dim,
            phi_factor,
            out.as_mut_ptr(),
        )
    };
    if rc == 0 {
        Ok(out)
    } else {
        Err(format!("phi_attention failed with code {rc}"))
    }
}

/// Compute a point on the Fibonacci spiral at parameter t.
pub fn fibonacci_spiral(t: f64) -> (f64, f64) {
    let mut x = 0.0;
    let mut y = 0.0;
    unsafe { ffi::sacred_fibonacci_spiral(t, &mut x, &mut y) }
    (x, y)
}

/// Generate a golden ratio-spaced sequence of `n` values in [0, 1].
///
/// Uses the golden ratio to produce a low-discrepancy sequence.
pub fn golden_sequence(n: usize) -> Vec<f64> {
    let mut out = vec![0.0f64; n];
    unsafe {
        ffi::sacred_golden_sequence(n, out.as_mut_ptr());
    }
    out
}

/// Search for Beal conjecture counterexamples.
///
/// Searches bases in `[min_base, max_base]` with exponents up to `max_exp`.
/// Returns up to `max_results` candidates.
pub fn beal_search(
    min_base: u64,
    max_base: u64,
    max_exp: u32,
    max_results: usize,
) -> Vec<BealCandidate> {
    let mut candidates = vec![
        BealCandidate {
            a: 0,
            b: 0,
            c: 0,
            m: 0,
            n: 0,
            r: 0,
            valid: false,
        };
        max_results
    ];
    let found = unsafe {
        ffi::sacred_beal_search(
            min_base,
            max_base,
            max_exp,
            candidates.as_mut_ptr(),
            max_results,
        )
    };
    candidates.truncate(found);
    candidates
}

/// Compute the φ-dimensional bottleneck size for a model dimension.
///
/// Returns the nearest Fibonacci number ≤ model_dim that serves
/// as an optimal bottleneck dimension.
pub fn phi_bottleneck(model_dim: usize) -> usize {
    unsafe { ffi::sacred_phi_bottleneck(model_dim) }
}

/// Compute Fibonacci-based attention head spacing for `n_heads` heads.
///
/// Returns a vector of spacing factors (one per head).
pub fn head_spacing(n_heads: usize) -> Vec<f64> {
    let mut out = vec![0.0f64; n_heads];
    unsafe {
        ffi::sacred_head_spacing(n_heads, out.as_mut_ptr());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires zig-sacred-geometry vendor submodule"]
    fn golden_sequence_in_range() {
        let seq = golden_sequence(100);
        for val in &seq {
            assert!(
                *val >= 0.0 && *val <= 1.0,
                "golden sequence value out of range: {val}"
            );
        }
    }

    #[test]
    #[ignore = "requires zig-sacred-geometry vendor submodule"]
    fn phi_bottleneck_is_fibonacci() {
        let bn = phi_bottleneck(512);
        // Should be a Fibonacci number ≤ 512
        let fibs = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377];
        assert!(fibs.contains(&(bn as u32)), "bottleneck {bn} is not Fibonacci");
    }
}

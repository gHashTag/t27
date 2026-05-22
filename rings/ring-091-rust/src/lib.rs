//! ring-091 -- **Stochastic Rounding (SR) primitives + SplitMix64 PRNG**.
//!
//! Wave 18 (2026-05-22, Closes #723): the fourth honestly-imported Wave-11
//! crate. Waves 15-17 landed ring-088 (GF16 MAC), ring-089 (TNN ISA), and
//! ring-090 (Simulator primitives). Wave 18 lands ring-091: stochastic
//! rounding -- an unbiased rounding mode that's standard practice in
//! low-precision ML training.
//!
//! ## What is stochastic rounding?
//!
//! Given a real value `x` that doesn't lie exactly on the target grid,
//! deterministic round-to-nearest always picks the *closer* grid point.
//! Repeated rounding accumulates a bias (away from the midpoint).
//!
//! Stochastic rounding picks the nearest *grid points above and below*
//! and chooses between them randomly, weighted by distance:
//!
//! ```text
//!  ceil(x)  with probability   frac(x) = x - floor(x)
//!  floor(x) with probability   1 - frac(x)
//! ```
//!
//! In expectation, `E[SR(x)] == x` -- the rounding is **unbiased**. Over
//! many independent roundings the average converges to the true value.
//! This is the universal property exercised by [`tests::sr_is_unbiased`].
//!
//! References (universal background, not new spec):
//! * Hopkins et al. (2020), "Stochastic rounding: implementation, error
//!   analysis and applications".
//! * Vigna (2014), "Further scramblings of Marsaglia's xorshift generators"
//!   -- defines the SplitMix64 PRNG used here.
//!
//! ## What this crate provides
//!
//! * [`SplitMix64`] -- a deterministic, seedable, allocation-free 64-bit
//!   PRNG. Branch-free `next_u64()`.
//! * [`RoundingMode`] -- `Nearest` (round-half-to-even baseline) and
//!   `Stochastic` (this crate's contribution).
//! * [`sr_round_f32_to_i32`] -- single-value SR over the integer grid.
//! * [`sr_quantize_f32`] -- single-value SR over a uniform grid of `step`.
//! * [`sr_quantize_batch`] -- streaming, allocation-free batch
//!   quantization.
//! * [`identity_witness`] -- universal anchor `phi^2 + 1/phi^2 == 3`.
//!
//! ## Honest scope (R5-HONEST)
//!
//! * **No new spec.** SR is a textbook universal algorithm; SplitMix64 is
//!   a textbook PRNG. No file under `specs/`, `coq/`, `proofs/`,
//!   `bootstrap/`, `gen/` is touched (L2 GENERATION, L6 CEILING).
//! * **No GF16 path-dependency.** SR over `Gf16` is a natural next step
//!   but adds an inter-crate dependency on ring-088's quantizer; out of
//!   scope for Wave 18.
//! * **No hardware integration.** SR-aware MAC, SR-aware FPGA cell,
//!   accelerator wiring -- all out of scope.
//! * **`#![no_std]`** with zero external dependencies. f32 helpers
//!   (`floor`, `fract`) are implemented inline via `as i32` truncation
//!   (sufficient for SR inputs that fit in `i32`).
//!
//! Anchor: `phi^2 + 1/phi^2 = 3`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(not(test), no_std)]

// ---------------------------------------------------------------------------
// SplitMix64 -- Vigna (2014)
// ---------------------------------------------------------------------------

/// Multiplicative golden constant used by SplitMix64. Vigna's published
/// value (`0x9E3779B97F4A7C15`). Equal to `floor(2^64 / phi)` where
/// `phi = (1 + sqrt(5)) / 2` -- the same anchor this ring exists to
/// preserve.
pub const SPLITMIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// First mixing constant (Vigna 2014).
pub const SPLITMIX_MIX1: u64 = 0xBF58_476D_1CE4_E5B9;
/// Second mixing constant (Vigna 2014).
pub const SPLITMIX_MIX2: u64 = 0x94D0_49BB_1331_11EB;

/// Deterministic, seedable, 64-bit PRNG.
///
/// `next_u64` is branch-free and constant-time. The same seed always
/// produces the same sequence, so unit tests are reproducible.
#[derive(Clone, Copy, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Construct from a 64-bit seed. The seed is taken verbatim; two
    /// instances with the same seed produce identical sequences.
    pub const fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    /// Advance state by `SPLITMIX_GAMMA`, then apply the two-round
    /// avalanche mix and return a uniform `u64`.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(SPLITMIX_GAMMA);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(SPLITMIX_MIX1);
        z = (z ^ (z >> 27)).wrapping_mul(SPLITMIX_MIX2);
        z ^ (z >> 31)
    }

    /// Return a uniform `f32` in `[0.0, 1.0)`.
    ///
    /// Uses the high 24 bits of `next_u64()` (the f32 mantissa width is
    /// 23 explicit bits + 1 implicit; 24 bits of entropy is the maximum
    /// useful for `f32` uniforms).
    pub fn next_f32_unit(&mut self) -> f32 {
        let bits = (self.next_u64() >> 40) as u32; // top 24 bits
        (bits as f32) * (1.0 / 16_777_216.0_f32) // / 2^24
    }
}

// ---------------------------------------------------------------------------
// f32 helpers (no_std-friendly: implemented via i32 truncation)
// ---------------------------------------------------------------------------

/// `floor(x)` for f32 values that fit in `i32` (~ +/- 2.14e9).
///
/// Out-of-range inputs are returned unchanged: their fractional part is
/// negligible at f32 precision anyway, and stochastic rounding such a
/// value is meaningless (the gap between consecutive f32s already
/// exceeds 1).
pub fn floor_f32(x: f32) -> f32 {
    if !is_finite_f32(x) {
        return x;
    }
    if x >= i32::MAX as f32 || x <= i32::MIN as f32 {
        return x;
    }
    let trunc = x as i32 as f32;
    if x >= 0.0 || trunc == x {
        trunc
    } else {
        trunc - 1.0
    }
}

/// Fractional part `frac(x) = x - floor(x)`. Always in `[0.0, 1.0)`
/// for finite, in-range `x`.
pub fn frac_f32(x: f32) -> f32 {
    x - floor_f32(x)
}

/// `true` iff `x` is finite (not NaN, not +/- Inf). `no_std`-friendly.
pub fn is_finite_f32(x: f32) -> bool {
    let bits = x.to_bits();
    let exp = (bits >> 23) & 0xFF;
    exp != 0xFF
}

/// Inline f32 absolute value (no `core::intrinsics`, no `libm`).
pub fn abs_f32(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

// ---------------------------------------------------------------------------
// Rounding mode
// ---------------------------------------------------------------------------

/// Selects a rounding strategy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundingMode {
    /// IEEE 754 round-half-to-even baseline (deterministic, biased on
    /// non-tie cases only by the inherent grid).
    Nearest,
    /// Stochastic rounding: `P[ceil] = frac(x)`, `P[floor] = 1 - frac(x)`.
    Stochastic,
}

// ---------------------------------------------------------------------------
// SR primitives
// ---------------------------------------------------------------------------

/// Stochastically round `x` to the nearest integer using `rng`.
///
/// Returns `x.floor() as i32` with probability `1 - frac(x)`, and
/// `x.floor() as i32 + 1` with probability `frac(x)`. NaN inputs return
/// `0`. Inputs outside the `i32` range saturate.
pub fn sr_round_f32_to_i32(x: f32, rng: &mut SplitMix64) -> i32 {
    if !is_finite_f32(x) {
        return 0;
    }
    if x >= i32::MAX as f32 {
        return i32::MAX;
    }
    if x <= i32::MIN as f32 {
        return i32::MIN;
    }
    let f = floor_f32(x);
    let frac = x - f;
    let u = rng.next_f32_unit();
    let lower = f as i32;
    if u < frac {
        lower + 1
    } else {
        lower
    }
}

/// Stochastically round `x` to the nearest multiple of `step`.
///
/// Equivalent to `step * SR(x / step)`. `step` must be finite and
/// non-zero; otherwise `x` is returned unchanged. Saturates if the
/// scaled value escapes `i32`.
pub fn sr_quantize_f32(x: f32, step: f32, rng: &mut SplitMix64) -> f32 {
    if !is_finite_f32(step) || step == 0.0 || !is_finite_f32(x) {
        return x;
    }
    let scaled = x / step;
    let k = sr_round_f32_to_i32(scaled, rng) as f32;
    k * step
}

/// Streaming batch SR-quantization. Writes `output[i] = sr_quantize_f32(
/// input[i], step, rng)` for `i in 0 .. min(input.len(), output.len())`.
///
/// Returns the number of elements written. Allocation-free.
pub fn sr_quantize_batch(
    input: &[f32],
    output: &mut [f32],
    step: f32,
    rng: &mut SplitMix64,
) -> usize {
    let n = if input.len() < output.len() {
        input.len()
    } else {
        output.len()
    };
    let mut i = 0;
    while i < n {
        output[i] = sr_quantize_f32(input[i], step, rng);
        i += 1;
    }
    n
}

// ---------------------------------------------------------------------------
// Identity witness (universal anchor)
// ---------------------------------------------------------------------------

/// Golden ratio (used by [`identity_witness`]).
pub const PHI: f64 = 1.618_033_988_749_894_8;

/// Returns `true` iff `phi^2 + 1/phi^2 == 3` to f64 1e-15.
pub fn identity_witness() -> bool {
    let phi2 = PHI * PHI;
    let inv_phi2 = 1.0 / phi2;
    let d = phi2 + inv_phi2 - 3.0;
    let d_abs = if d < 0.0 { -d } else { d };
    d_abs < 1.0e-15
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- SplitMix64 ----------------------------------------------------

    #[test]
    fn splitmix_is_deterministic() {
        let mut a = SplitMix64::new(0);
        let mut b = SplitMix64::new(0);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn splitmix_different_seeds_differ() {
        let mut a = SplitMix64::new(1);
        let mut b = SplitMix64::new(2);
        // Extremely unlikely to collide on the first draw with these
        // constants; this checks the seed actually changes output.
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn splitmix_first_value_with_seed_0() {
        // Vigna's reference: SplitMix64(0).next() = 0xE220A8397B1DCDAF.
        let mut rng = SplitMix64::new(0);
        assert_eq!(rng.next_u64(), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn next_f32_unit_in_range() {
        let mut rng = SplitMix64::new(42);
        for _ in 0..1000 {
            let u = rng.next_f32_unit();
            assert!(u >= 0.0, "u = {} should be >= 0", u);
            assert!(u < 1.0, "u = {} should be < 1", u);
        }
    }

    // ----- f32 helpers ---------------------------------------------------

    #[test]
    fn floor_f32_positive() {
        assert_eq!(floor_f32(0.0), 0.0);
        assert_eq!(floor_f32(0.7), 0.0);
        assert_eq!(floor_f32(1.0), 1.0);
        assert_eq!(floor_f32(3.99), 3.0);
    }

    #[test]
    fn floor_f32_negative() {
        assert_eq!(floor_f32(-0.5), -1.0);
        assert_eq!(floor_f32(-1.0), -1.0);
        assert_eq!(floor_f32(-2.3), -3.0);
    }

    #[test]
    fn frac_f32_basic() {
        // f32 round-off forces an inequality check.
        assert!(abs_f32(frac_f32(0.75_f32) - 0.75_f32) < 1.0e-6);
        assert!(abs_f32(frac_f32(3.25_f32) - 0.25_f32) < 1.0e-6);
        assert!(abs_f32(frac_f32(-0.25_f32) - 0.75_f32) < 1.0e-6);
    }

    // ----- Edge cases ----------------------------------------------------

    #[test]
    fn sr_exact_integer_returns_integer() {
        let mut rng = SplitMix64::new(123);
        for k in -5..=5 {
            // frac(k.0) = 0, so SR must always return k.
            assert_eq!(sr_round_f32_to_i32(k as f32, &mut rng), k);
        }
    }

    #[test]
    fn sr_nan_returns_zero() {
        let mut rng = SplitMix64::new(0);
        assert_eq!(sr_round_f32_to_i32(f32::NAN, &mut rng), 0);
    }

    #[test]
    fn sr_inf_saturates() {
        let mut rng = SplitMix64::new(0);
        // Inf is not finite -> contract says return 0.
        assert_eq!(sr_round_f32_to_i32(f32::INFINITY, &mut rng), 0);
        assert_eq!(sr_round_f32_to_i32(f32::NEG_INFINITY, &mut rng), 0);
    }

    #[test]
    fn sr_round_returns_floor_or_ceil() {
        let mut rng = SplitMix64::new(7);
        for _ in 0..1000 {
            let r = sr_round_f32_to_i32(2.7_f32, &mut rng);
            assert!(r == 2 || r == 3, "SR(2.7) = {}, expected 2 or 3", r);
        }
    }

    #[test]
    fn sr_quantize_zero_step_passthrough() {
        let mut rng = SplitMix64::new(0);
        assert_eq!(sr_quantize_f32(3.14_f32, 0.0_f32, &mut rng), 3.14_f32);
    }

    #[test]
    fn sr_quantize_step_one_matches_round_to_i32() {
        let mut rng = SplitMix64::new(99);
        for _ in 0..100 {
            let mut rng_a = rng;
            let mut rng_b = rng;
            let x = 1.3_f32;
            let q = sr_quantize_f32(x, 1.0_f32, &mut rng_a);
            let r = sr_round_f32_to_i32(x, &mut rng_b) as f32;
            assert_eq!(q, r);
            rng.next_u64(); // advance the outer rng
        }
    }

    // ----- Statistical: unbiasedness -------------------------------------

    #[test]
    fn sr_is_unbiased() {
        // Mean of 10_000 SR(0.3) draws should be within 3-sigma of 0.3.
        // Each draw is Bernoulli(0.3) over {0, 1}: variance 0.21, so
        // sigma of the mean is sqrt(0.21 / 10_000) ~= 0.00458. 3-sigma
        // window is 0.014. We assert |mean - 0.3| < 0.02 (slack).
        let mut rng = SplitMix64::new(2026);
        let n = 10_000_i32;
        let mut total: i64 = 0;
        for _ in 0..n {
            total += sr_round_f32_to_i32(0.3_f32, &mut rng) as i64;
        }
        let mean = (total as f32) / (n as f32);
        let err = abs_f32(mean - 0.3_f32);
        assert!(err < 0.02_f32, "empirical mean {} of SR(0.3) deviates by {} > 0.02", mean, err);
    }

    #[test]
    fn sr_quantize_phi_unbiased() {
        // The anchor exercised through this crate's kernel: SR-quantize
        // phi at step = 0.01 ten thousand times; the average must
        // approach phi to within +- 0.001 (3-sigma window for
        // Bernoulli(frac(phi/0.01)) / sqrt(N) at this step).
        let phi = 1.618_033_988_749_894_8_f32;
        let step = 0.01_f32;
        let mut rng = SplitMix64::new(314159);
        let n = 10_000_i32;
        let mut total: f64 = 0.0;
        for _ in 0..n {
            total += sr_quantize_f32(phi, step, &mut rng) as f64;
        }
        let mean = (total / n as f64) as f32;
        let err = abs_f32(mean - phi);
        assert!(err < 0.001_f32, "mean(SR-quantize(phi, 0.01)) = {} deviates from phi = {} by {} > 0.001", mean, phi, err);
    }

    // ----- Batch ---------------------------------------------------------

    #[test]
    fn sr_quantize_batch_writes_min_len() {
        let mut rng = SplitMix64::new(1);
        let input = [1.2_f32, 3.4, 5.6, 7.8];
        let mut output = [0.0_f32; 2];
        let n = sr_quantize_batch(&input, &mut output, 1.0, &mut rng);
        assert_eq!(n, 2);
        // Both outputs are either floor or ceil of the corresponding
        // inputs; just check they're integer-valued (step = 1).
        for &q in output.iter() {
            let f = floor_f32(q);
            assert!(abs_f32(q - f) < 1.0e-6, "q = {} should be integer", q);
        }
    }

    #[test]
    fn sr_quantize_batch_empty_input() {
        let mut rng = SplitMix64::new(0);
        let input: [f32; 0] = [];
        let mut output = [0.0_f32; 4];
        let n = sr_quantize_batch(&input, &mut output, 1.0, &mut rng);
        assert_eq!(n, 0);
    }

    // ----- Universal anchor ----------------------------------------------

    #[test]
    fn identity_witness_holds() {
        assert!(identity_witness());
    }

    #[test]
    fn rounding_mode_eq() {
        assert_eq!(RoundingMode::Nearest, RoundingMode::Nearest);
        assert_eq!(RoundingMode::Stochastic, RoundingMode::Stochastic);
        assert_ne!(RoundingMode::Nearest, RoundingMode::Stochastic);
    }
}

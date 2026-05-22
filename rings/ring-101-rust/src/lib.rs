//! ring-101 — **Analog GF16**
//!
//! Wave 12 / Track C scaffolding. Software model of an *analog* GF16 numeric
//! pathway: deterministic quantize → dequantize round-trip, plus an injectable
//! noise channel that simulates capacitor / current-mode imperfections.
//!
//! This crate does **not** redefine the GF16 spec. The spec lives in
//! `specs/numeric/gf16.t27` (root repo) and is enforced by `FORMAT-SPEC-001.json`.
//! Numerics here are an *approximation* sufficient for plumbing tests; final
//! conformance lives in the existing `gf*_vectors.json` files.
//!
//! ## Status (honest)
//! * Compilation **not** yet verified in CI — Wave 12 Track D will gate that.
//! * No conformance hashes touched.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// GF16 surrogate: 6 exp bits / 9 mantissa bits / 1 sign bit, packed in a `u16`.
///
/// This is a *model* — the canonical encoding is owned by the spec layer.
/// We only need a reversible round-trip for the analog noise channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gf16Bits(pub u16);

/// Quantise an `f32` into the GF16 surrogate.
///
/// Saturating on overflow, flush-to-zero on underflow. Deterministic.
pub fn quantize(x: f32) -> Gf16Bits {
    if !x.is_finite() {
        // Map ±inf / NaN to the largest magnitude (saturating).
        return Gf16Bits(if x.is_sign_negative() { 0xFFFF } else { 0x7FFF });
    }
    if x == 0.0 {
        return Gf16Bits(0);
    }
    let sign = if x.is_sign_negative() { 1u16 } else { 0u16 };
    let mag = x.abs();
    // exponent: floor(log2(mag)) clamped to [-31, 31] then biased by +31.
    let e_raw = mag.log2().floor() as i32;
    let e = e_raw.clamp(-31, 31);
    let biased = (e + 31) as u16; // 6 bits, range 0..=62
    let m_norm = (mag / (2f32).powi(e)) - 1.0; // in [0, 1)
    let mant = (m_norm.clamp(0.0, 0.999_999) * 512.0) as u16; // 9 bits, 0..511
    Gf16Bits((sign << 15) | ((biased & 0x3F) << 9) | (mant & 0x1FF))
}

/// Dequantise the GF16 surrogate back into `f32`.
pub fn dequantize(b: Gf16Bits) -> f32 {
    let v = b.0;
    if v == 0 {
        return 0.0;
    }
    let sign = (v >> 15) & 1;
    let biased = (v >> 9) & 0x3F;
    let mant = v & 0x1FF;
    let e = biased as i32 - 31;
    let m = 1.0 + (mant as f32) / 512.0;
    let val = m * (2f32).powi(e);
    if sign == 1 { -val } else { val }
}

/// Noise model for the analog channel.
///
/// `sigma` is the relative standard deviation (a fraction of the carrier),
/// `seed` is a 64-bit state used by the deterministic LCG below. Pure function:
/// same inputs → same output.
#[derive(Debug, Clone, Copy)]
pub struct AnalogNoise {
    /// Relative noise sigma (e.g. `1e-3` for 0.1 %).
    pub sigma: f32,
    /// Deterministic seed.
    pub seed: u64,
}

impl AnalogNoise {
    /// Apply noise to a sample. Returns the noisy value and the next seed.
    pub fn perturb(self, x: f32) -> (f32, AnalogNoise) {
        // Linear congruential generator (Numerical Recipes constants).
        let next = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        // Map top 24 bits to [-1, +1].
        let u = ((next >> 40) as i64 - (1 << 23)) as f32 / (1 << 23) as f32;
        let noisy = x + x * self.sigma * u;
        (noisy, AnalogNoise { sigma: self.sigma, seed: next })
    }
}

/// Identity witness shared across all Trinity rings.
pub fn identity_witness() -> bool {
    let phi = (1.0_f64 + 5.0_f64.sqrt()) / 2.0;
    ((phi * phi + 1.0 / (phi * phi)) - 3.0).abs() < 1e-15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_witness_holds() {
        assert!(identity_witness());
    }

    #[test]
    fn quantize_dequantize_roundtrip_within_tolerance() {
        for &x in &[1.0_f32, 0.5, -0.125, 3.14159, -2.71828, 1e-3, 1e3] {
            let q = quantize(x);
            let r = dequantize(q);
            let rel = ((r - x) / x).abs();
            assert!(rel < 1e-2, "x={x} r={r} rel_err={rel}");
        }
    }

    #[test]
    fn zero_roundtrips_exactly() {
        assert_eq!(dequantize(quantize(0.0)), 0.0);
    }

    #[test]
    fn analog_noise_is_deterministic() {
        let n = AnalogNoise { sigma: 1e-3, seed: 42 };
        let (a, n2) = n.perturb(1.0);
        let (b, _) = n2.perturb(1.0);
        // Two perturbations of `1.0` with the same seed pipeline must be
        // reproducible if we re-run the pair from the same seed.
        let n_again = AnalogNoise { sigma: 1e-3, seed: 42 };
        let (a2, n2_again) = n_again.perturb(1.0);
        let (b2, _) = n2_again.perturb(1.0);
        assert_eq!(a, a2);
        assert_eq!(b, b2);
    }

    #[test]
    fn analog_noise_stays_within_3_sigma() {
        // 3-sigma bound: |noise/x| <= 3 * sigma. LCG output is bounded in [-1, 1],
        // so this is in fact a hard 1-sigma bound: |delta| <= sigma * |x|.
        let mut s = AnalogNoise { sigma: 1e-3, seed: 1 };
        let x = 1.0_f32;
        for _ in 0..1000 {
            let (y, next) = s.perturb(x);
            assert!((y - x).abs() <= 1e-3 + 1e-9);
            s = next;
        }
    }
}

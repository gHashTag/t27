//! ring-102 — **Photonic MAC**
//!
//! Wave 12 / Track C scaffolding. Models a wavelength-multiplexed MAC array:
//! `K` lanes (one per wavelength) compute `sum_k(a_k * w_k)`, each lane attenuated
//! by an insertion-loss coefficient. Pure software model — no physical units
//! beyond a dimensionless "intensity".
//!
//! ## Status (honest)
//! * Compilation **not** yet verified in CI (Wave 12 Track D).
//! * Not part of the workspace `members` — opt-in until Track D.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Per-wavelength channel with its own insertion-loss factor (`0.0..=1.0`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lane {
    /// Insertion-loss factor: `1.0` = lossless, `0.0` = completely attenuated.
    pub loss: f32,
}

impl Lane {
    /// Lossless lane.
    pub const fn ideal() -> Self {
        Self { loss: 1.0 }
    }

    /// Construct a lane with explicit loss factor, clamped into `[0, 1]`.
    pub fn new(loss: f32) -> Self {
        Self { loss: loss.clamp(0.0, 1.0) }
    }
}

/// A photonic MAC unit: `K` parallel lanes computing a weighted dot product.
#[derive(Debug, Clone)]
pub struct PhotonicMac {
    lanes: alloc_vec_polyfill::Vec<Lane>,
}

impl PhotonicMac {
    /// Build a MAC unit from a slice of lanes.
    pub fn new(lanes: &[Lane]) -> Self {
        Self {
            lanes: lanes.iter().copied().collect(),
        }
    }

    /// Number of wavelength lanes.
    pub fn k(&self) -> usize {
        self.lanes.len()
    }

    /// Compute `sum_k(a_k * w_k * loss_k)`. Inputs whose length differs from
    /// `k` return `Err`.
    pub fn dot(&self, a: &[f32], w: &[f32]) -> Result<f32, PhotonicError> {
        if a.len() != self.k() || w.len() != self.k() {
            return Err(PhotonicError::LengthMismatch {
                expected: self.k(),
                got_a: a.len(),
                got_w: w.len(),
            });
        }
        let mut acc = 0.0f32;
        for (i, lane) in self.lanes.iter().enumerate() {
            acc += a[i] * w[i] * lane.loss;
        }
        Ok(acc)
    }

    /// Average lane loss — useful for telemetry overlays.
    pub fn average_loss(&self) -> f32 {
        if self.lanes.is_empty() {
            return 0.0;
        }
        let s: f32 = self.lanes.iter().map(|l| l.loss).sum();
        s / self.lanes.len() as f32
    }
}

/// Errors raised by the photonic MAC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotonicError {
    /// Vector length disagreement against the lane count.
    LengthMismatch {
        /// Lane count of the MAC unit.
        expected: usize,
        /// Length of the activations slice.
        got_a: usize,
        /// Length of the weights slice.
        got_w: usize,
    },
}

/// Identity witness — see ring-100.
pub fn identity_witness() -> bool {
    let phi = (1.0_f64 + 5.0_f64.sqrt()) / 2.0;
    ((phi * phi + 1.0 / (phi * phi)) - 3.0).abs() < 1e-15
}

mod alloc_vec_polyfill {
    extern crate alloc;
    pub use alloc::vec::Vec;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_witness_holds() {
        assert!(identity_witness());
    }

    #[test]
    fn lossless_dot_matches_classical_dot() {
        let mac = PhotonicMac::new(&[Lane::ideal(); 4]);
        let a = [1.0, 2.0, 3.0, 4.0];
        let w = [0.5, 0.5, 0.5, 0.5];
        let got = mac.dot(&a, &w).unwrap();
        let want: f32 = a.iter().zip(w.iter()).map(|(x, y)| x * y).sum();
        assert!((got - want).abs() < 1e-6);
    }

    #[test]
    fn loss_attenuates_each_lane() {
        let mac = PhotonicMac::new(&[
            Lane::new(0.9),
            Lane::new(0.5),
            Lane::new(0.0),
            Lane::new(1.0),
        ]);
        // Lane 2 is completely lost — its contribution must be zero.
        let got = mac.dot(&[10.0, 10.0, 10.0, 10.0], &[1.0, 1.0, 1.0, 1.0]).unwrap();
        // 10*0.9 + 10*0.5 + 0 + 10*1.0 = 24
        assert!((got - 24.0).abs() < 1e-6);
        assert!((mac.average_loss() - 0.6).abs() < 1e-6);
    }

    #[test]
    fn rejects_length_mismatch() {
        let mac = PhotonicMac::new(&[Lane::ideal(); 3]);
        let err = mac.dot(&[1.0, 2.0], &[1.0, 2.0, 3.0]).unwrap_err();
        match err {
            PhotonicError::LengthMismatch { expected, got_a, got_w } => {
                assert_eq!(expected, 3);
                assert_eq!(got_a, 2);
                assert_eq!(got_w, 3);
            }
        }
    }

    #[test]
    fn loss_is_clamped_to_unit_interval() {
        assert_eq!(Lane::new(-1.0).loss, 0.0);
        assert_eq!(Lane::new(2.5).loss, 1.0);
    }
}

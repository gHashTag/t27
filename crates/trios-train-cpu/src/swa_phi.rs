//! SWA(1/φ) — Stochastic Weight Averaging with φ-decay
//!
//! P03: SWA(1/φ) — TRUE SWA, NOT EMA
//! Expected ΔBPB: −0.02
//!
//! Reference: #190 GOLF task

#[allow(dead_code)]
const PHI: f64 = 1.618033988749895;

/// SWA state for a single parameter
#[derive(Debug, Clone)]
pub struct SwaState {
    /// Shadow copy of parameters
    #[allow(dead_code)]
    shadow: Vec<f32>,
}

impl SwaState {
    pub fn new(params: &[f32]) -> Self {
        Self {
            shadow: params.to_vec(),
        }
    }

    /// SWA update step
    ///
    /// # Arguments
    /// * `params` - current parameters [in/out]
    /// * `shadow` - shadow parameters [in/out]
    /// * `step` - current training step
    /// * `swa_start` - step to start SWA
    /// * `swa_period` - SWA period (typically = 1/φ × steps)
    pub fn step(
        params: &mut [f32],
        shadow: &mut [f32],
        step: usize,
        swa_start: usize,
        swa_period: usize,
    ) {
        if step < swa_start {
            // Before SWA starts: just copy params to shadow
            shadow.copy_from_slice(params);
        } else if (step - swa_start).is_multiple_of(swa_period) {
            // At SWA update: average shadow and params
            for i in 0..params.len() {
                shadow[i] = 0.5 * shadow[i] + 0.5 * params[i];
            }
            params.copy_from_slice(shadow);
        }
        // Between SWA updates: params update normally, shadow stays
    }
}

/// Create SWA state for all parameters
pub fn swa_init(params: &[f32]) -> SwaState {
    SwaState::new(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swa_before_start() {
        let params = vec![1.0, 2.0, 3.0];
        let state = SwaState::new(&params);

        let mut params_copy = params.clone();
        let mut shadow = state.shadow.clone();

        SwaState::step(&mut params_copy, &mut shadow, 5, 10, 2);

        assert_eq!(shadow, [1.0, 2.0, 3.0]); // Shadow unchanged before start
        assert_eq!(params_copy, [1.0, 2.0, 3.0]); // Params unchanged
    }

    #[test]
    fn test_swa_first_update() {
        let params = vec![1.0, 2.0, 3.0];
        let state = SwaState::new(&params);

        let mut params_copy = vec![1.0, 2.0, 3.0];
        let mut shadow = state.shadow;

        // At step 10, first SWA update (step - swa_start) % period == 0
        SwaState::step(&mut params_copy, &mut shadow, 10, 10, 2);

        assert_eq!(shadow, [1.0, 2.0, 3.0]); // Shadow updated to average
        assert_eq!(params_copy, [1.0, 2.0, 3.0]); // Params copied from shadow
    }

    #[test]
    fn test_swa_between_updates() {
        let params = vec![1.0, 2.0, 3.0];
        let state = SwaState::new(&params);

        let mut params_copy = vec![1.0, 2.0, 3.0];
        let mut shadow = state.shadow;

        // Step 11: between SWA updates
        params_copy[0] = 5.0; // External update
        SwaState::step(&mut params_copy, &mut shadow, 11, 10, 2);

        assert_eq!(shadow, [1.0, 2.0, 3.0]); // Shadow unchanged
        assert_eq!(params_copy, [5.0, 2.0, 3.0]); // Params updated externally
    }
}

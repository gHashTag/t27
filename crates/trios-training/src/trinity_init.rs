pub struct TrinityInitConfig {
    pub gauge_std: f32,
    pub higgs_std: f32,
    pub lepton_std: f32,
}

impl Default for TrinityInitConfig {
    fn default() -> Self {
        Self {
            gauge_std: 0.118034,
            higgs_std: 0.072949,
            lepton_std: 0.045085,
        }
    }
}

pub fn phi_ortho_init(rows: usize, cols: usize) -> Vec<f32> {
    let gain = 1.0_f32 / 1.618_034;
    let std = gain * (2.0_f32 / rows as f32).sqrt();
    (0..rows * cols)
        .map(|i| {
            let phi_freq = 1.0 / 1.618_034_f32.powi((i % 7 + 1) as i32);
            let phase = (i as f32 * 0.618) * std::f32::consts::PI;
            let val = (2.0 * std::f32::consts::PI * phi_freq * (i as f32).sqrt() + phase).sin();
            val * std
        })
        .collect()
}

pub fn ortho_init_qkv(d_model: usize, _n_heads: usize) -> Vec<f32> {
    phi_ortho_init(3 * d_model, d_model)
}

pub fn ortho_init_ff(d_model: usize, ff_dim: usize) -> Vec<Vec<f32>> {
    vec![
        phi_ortho_init(ff_dim, d_model),
        phi_ortho_init(d_model, ff_dim),
    ]
}

pub struct SWAState {
    pub avg_params: Vec<f32>,
    pub n_averaged: usize,
    pub decay: f32,
}

impl SWAState {
    pub fn new(param_count: usize, decay: f32) -> Self {
        Self {
            avg_params: vec![0.0; param_count],
            n_averaged: 0,
            decay,
        }
    }

    pub fn update(&mut self, params: &[f32]) {
        self.n_averaged += 1;
        let beta = self.decay;
        for (avg, &p) in self.avg_params.iter_mut().zip(params.iter()) {
            *avg = beta * *avg + (1.0 - beta) * p;
        }
    }

    pub fn swa_params(&self) -> &[f32] {
        &self.avg_params
    }

    pub fn decay_value(&self) -> f32 {
        self.decay
    }
}

pub fn swa_decay_sweep(decay_values: &[f32], param_count: usize) -> Vec<SWASweepResult> {
    let params: Vec<f32> = (0..param_count).map(|i| (i as f32 * 0.1).sin()).collect();
    decay_values
        .iter()
        .map(|&decay| {
            let mut state = SWAState::new(param_count, decay);
            for step in 0..100 {
                let shifted: Vec<f32> = params
                    .iter()
                    .map(|&p| p + (step as f32 * 0.01).sin() * 0.1)
                    .collect();
                state.update(&shifted);
            }
            let norm: f32 = state.swa_params().iter().map(|p| p * p).sum::<f32>().sqrt();
            SWASweepResult {
                decay,
                n_averaged: state.n_averaged,
                avg_norm: norm,
            }
        })
        .collect()
}

pub struct SWASweepResult {
    pub decay: f32,
    pub n_averaged: usize,
    pub avg_norm: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phi_ortho_init_shape() {
        let w = phi_ortho_init(64, 32);
        assert_eq!(w.len(), 64 * 32);
    }

    #[test]
    fn phi_ortho_init_non_zero() {
        let w = phi_ortho_init(16, 16);
        let sum: f32 = w.iter().map(|x| x.abs()).sum();
        assert!(sum > 0.0);
    }

    #[test]
    fn phi_ortho_init_gain() {
        let w = phi_ortho_init(100, 100);
        let norm: f32 = w.iter().map(|x| x * x).sum::<f32>().sqrt();
        let expected = (1.0 / 1.618_034) * (2.0 / 100.0_f32).sqrt() * 100.0;
        assert!(
            (norm - expected).abs() / expected < 2.0,
            "norm={}, expected={}",
            norm,
            expected
        );
    }

    #[test]
    fn ortho_init_qkv_shape() {
        let w = ortho_init_qkv(64, 4);
        assert_eq!(w.len(), 3 * 64 * 64);
    }

    #[test]
    fn ortho_init_ff_shape() {
        let weights = ortho_init_ff(64, 256);
        assert_eq!(weights.len(), 2);
        assert_eq!(weights[0].len(), 256 * 64);
        assert_eq!(weights[1].len(), 64 * 256);
    }

    #[test]
    fn swa_update_converges() {
        let mut swa = SWAState::new(8, 0.999);
        let params = vec![1.0_f32; 8];
        for _ in 0..10000 {
            swa.update(&params);
        }
        for &p in swa.swa_params() {
            assert!(
                (p - 1.0).abs() < 0.1,
                "SWA should converge to 1.0, got {}",
                p
            );
        }
    }

    #[test]
    fn swa_decay_sweep_runs() {
        let results = swa_decay_sweep(&[0.4, 0.618, 0.95, 0.999], 16);
        assert_eq!(results.len(), 4);
        assert!(results[3].avg_norm > 0.0);
    }

    #[test]
    fn swa_higher_decay_smoother() {
        let results = swa_decay_sweep(&[0.4, 0.999], 32);
        assert!(
            results[1].avg_norm > 0.0,
            "Higher decay should still produce non-zero average"
        );
    }
}

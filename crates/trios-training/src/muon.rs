pub struct MuonOptimizer {
    pub momentum: Vec<f32>,
    pub lr: f32,
    pub weight_decay: f32,
    pub beta: f32,
    pub n_steps: usize,
}

impl MuonOptimizer {
    pub fn new(param_count: usize, lr: f32, weight_decay: f32) -> Self {
        Self {
            momentum: vec![0.0; param_count],
            lr,
            weight_decay,
            beta: 0.95,
            n_steps: 0,
        }
    }

    pub fn step(&mut self, params: &mut [f32], gradients: &[f32]) {
        assert_eq!(params.len(), gradients.len());
        assert_eq!(params.len(), self.momentum.len());
        self.n_steps += 1;

        let n = params.len();
        let dim = (n as f64).sqrt().ceil() as usize;
        let rows = n.div_ceil(dim);

        for (i, m) in self.momentum.iter_mut().enumerate() {
            let g = gradients.get(i).copied().unwrap_or(0.0);
            *m = self.beta * *m + g;
        }

        let mut orthogonalized = vec![0.0f32; n];
        for row in 0..rows {
            let start = row * dim;
            let end = (start + dim).min(n);
            let row_len = end - start;
            let mom_slice = &self.momentum[start..end];
            let orth_slice = &mut orthogonalized[start..end];

            let norm_sq: f32 = mom_slice.iter().map(|m| m * m).sum();
            let norm = norm_sq.sqrt().max(1e-8);

            for (src, dst) in mom_slice.iter().zip(orth_slice.iter_mut()) {
                *dst = *src / norm;
            }

            if row_len > 1 {
                let mean: f32 = orth_slice.iter().sum::<f32>() / row_len as f32;
                for val in orth_slice.iter_mut() {
                    *val -= mean;
                }
            }
        }

        for (i, p) in params.iter_mut().enumerate() {
            let wd = self.weight_decay * *p;
            *p -= self.lr * orthogonalized[i] + wd;
        }
    }

    pub fn param_count(&self) -> usize {
        self.momentum.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]

    fn muon_basic_step() {
        let mut opt = MuonOptimizer::new(4, 0.01, 0.04);
        let mut params = vec![1.0, 2.0, 3.0, 4.0];
        let grads = vec![0.1, 0.1, 0.1, 0.1];
        opt.step(&mut params, &grads);
        assert!(params[0] < 1.0);
    }

    #[test]
    fn muon_momentum_accumulates() {
        let mut opt = MuonOptimizer::new(4, 0.01, 0.0);
        let mut params = vec![0.0, 0.0, 0.0, 0.0];
        let grads = vec![1.0, -0.5, 0.3, -0.2];
        opt.step(&mut params, &grads);
        opt.step(&mut params, &grads);
        let p2 = params[0].abs();
        assert!(p2 > 0.0, "momentum should produce non-zero update");
    }

    #[test]
    fn muon_zero_grad_no_decay() {
        let mut opt = MuonOptimizer::new(4, 0.01, 0.0);
        let mut params = vec![1.0, 2.0, 3.0, 4.0];
        let grads = vec![0.0; 4];
        opt.step(&mut params, &grads);
        assert_eq!(params, vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn muon_sweep_configs() {
        for wd in [0.01, 0.04, 0.1] {
            let mut opt = MuonOptimizer::new(8, 0.01, wd);
            let mut params = vec![1.0; 8];
            let grads = vec![0.1; 8];
            for _ in 0..10 {
                opt.step(&mut params, &grads);
            }
            assert!(params[0] < 1.0);
        }
    }

    #[test]

    fn muon_param_count() {
        let opt = MuonOptimizer::new(100, 0.01, 0.04);
        assert_eq!(opt.param_count(), 100);
    }
}

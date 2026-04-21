pub struct TTTLoRA {
    pub rank: usize,
    pub input_dim: usize,
    pub output_dim: usize,
    pub down_proj: Vec<Vec<f32>>,
    pub up_proj: Vec<Vec<f32>>,
    pub lr: f32,
}

impl TTTLoRA {
    pub fn new(rank: usize, input_dim: usize, output_dim: usize) -> Self {
        let scale = (2.0_f32 / (input_dim + output_dim) as f32).sqrt() / rank as f32;
        let mut down_proj = Vec::with_capacity(rank);
        for i in 0..rank {
            let row: Vec<f32> = (0..input_dim)
                .map(|j| {
                    let phi_freq = 1.0 / 1.618_034_f32.powi(((i + j) % 7 + 1) as i32);
                    let phase = (i as f32 * 0.618) * std::f32::consts::PI;
                    (2.0 * std::f32::consts::PI * phi_freq * j as f32 + phase).sin() * scale
                })
                .collect();
            down_proj.push(row);
        }
        let mut up_proj = Vec::with_capacity(output_dim);
        for i in 0..output_dim {
            let row: Vec<f32> = (0..rank)
                .map(|j| {
                    let v = ((i * rank + j) as f64 * 0.618033988749895).fract() - 0.5;
                    v as f32 * scale * 0.1
                })
                .collect();
            up_proj.push(row);
        }
        Self {
            rank,
            input_dim,
            output_dim,
            down_proj,
            up_proj,
            lr: 1e-3,
        }
    }

    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        let latent = self.project_down(input);
        self.project_up(&latent)
    }

    pub fn ttt_update(&mut self, input: &[f32], reconstruction_target: &[f32]) {
        let latent = self.project_down(input);
        let output = self.project_up(&latent);
        let error: Vec<f32> = output
            .iter()
            .zip(reconstruction_target.iter())
            .map(|(o, t)| o - t)
            .collect();

        for (i, row) in self.up_proj.iter_mut().enumerate() {
            if i >= error.len() {
                break;
            }
            for (j, w) in row.iter_mut().enumerate() {
                if j < latent.len() {
                    *w -= self.lr * error[i] * latent[j];
                }
            }
        }

        for (i, row) in self.down_proj.iter_mut().enumerate() {
            for (j, w) in row.iter_mut().enumerate() {
                if j < input.len() {
                    let mut up_grad = 0.0f32;
                    for (k, up_row) in self.up_proj.iter().enumerate() {
                        if i < up_row.len() && k < error.len() {
                            up_grad += error[k] * up_row[i];
                        }
                    }
                    *w -= self.lr * up_grad * input[j];
                }
            }
        }
    }

    fn project_down(&self, input: &[f32]) -> Vec<f32> {
        let mut latent = vec![0.0; self.rank];
        for (i, row) in self.down_proj.iter().enumerate() {
            for (j, &w) in row.iter().enumerate() {
                if j < input.len() {
                    latent[i] += w * input[j];
                }
            }
        }
        latent
    }

    fn project_up(&self, latent: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0; self.output_dim];
        for (i, row) in self.up_proj.iter().enumerate() {
            for (j, &w) in row.iter().enumerate() {
                if j < latent.len() {
                    output[i] += w * latent[j];
                }
            }
        }
        output
    }

    pub fn param_count(&self) -> usize {
        self.rank * self.input_dim + self.output_dim * self.rank
    }
}

pub fn ttt_lora_rank_sweep(
    ranks: &[usize],
    input_dim: usize,
    output_dim: usize,
) -> Vec<LoRASweepResult> {
    let input = vec![0.5_f32; input_dim];
    let target = vec![1.0_f32; output_dim];
    let mut results = Vec::with_capacity(ranks.len());
    for &rank in ranks {
        let mut lora = TTTLoRA::new(rank, input_dim, output_dim);
        for _ in 0..50 {
            lora.ttt_update(&input, &target);
        }
        let output = lora.forward(&input);
        let mse: f32 = output
            .iter()
            .zip(target.iter())
            .map(|(o, t)| (o - t).powi(2))
            .sum::<f32>()
            / output.len() as f32;
        results.push(LoRASweepResult {
            rank,
            param_count: lora.param_count(),
            final_mse: mse,
        });
    }
    results
}

pub struct LoRASweepResult {
    pub rank: usize,
    pub param_count: usize,
    pub final_mse: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ttt_lora_shapes() {
        let lora = TTTLoRA::new(8, 64, 32);
        assert_eq!(lora.down_proj.len(), 8);
        assert_eq!(lora.down_proj[0].len(), 64);
        assert_eq!(lora.up_proj.len(), 32);
        assert_eq!(lora.up_proj[0].len(), 8);
    }

    #[test]
    fn ttt_lora_forward_dim() {
        let lora = TTTLoRA::new(4, 16, 8);
        let input = vec![1.0; 16];
        let output = lora.forward(&input);
        assert_eq!(output.len(), 8);
    }

    #[test]
    fn ttt_lora_update_reduces_error() {
        let mut lora = TTTLoRA::new(4, 16, 8);
        let input = vec![1.0; 16];
        let target = vec![1.0; 8];
        let before_mse = {
            let o = lora.forward(&input);
            o.iter()
                .zip(target.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
        };
        for _ in 0..20 {
            lora.ttt_update(&input, &target);
        }
        let after_mse = {
            let o = lora.forward(&input);
            o.iter()
                .zip(target.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
        };
        assert!(
            after_mse < before_mse,
            "TTT should reduce reconstruction error"
        );
    }

    #[test]
    fn ttt_lora_param_count() {
        let lora = TTTLoRA::new(8, 64, 32);
        assert_eq!(lora.param_count(), 8 * 64 + 32 * 8);
    }

    #[test]
    fn ttt_lora_rank_sweep_runs() {
        let results = ttt_lora_rank_sweep(&[4, 8, 16], 32, 16);
        assert_eq!(results.len(), 3);
        assert!(results[2].param_count > results[0].param_count);
    }
}

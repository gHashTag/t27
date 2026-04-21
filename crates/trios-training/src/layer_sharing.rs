pub struct LayerSharedTransformer {
    pub n_unique_layers: usize,
    pub n_iterations: usize,
    pub d_model: usize,
    pub n_heads: usize,
    pub layers: Vec<SharedLayer>,
}

pub struct SharedLayer {
    pub qkv_weight: Vec<f32>,
    pub out_weight: Vec<f32>,
    pub ff_weight1: Vec<f32>,
    pub ff_weight2: Vec<f32>,
    pub d_model: usize,
}

impl SharedLayer {
    pub fn new(d_model: usize, _n_heads: usize) -> Self {
        let scale = (2.0_f32 / d_model.max(1) as f32).sqrt();
        let qkv_weight = Self::phi_init(3 * d_model * d_model, scale);
        let out_weight = Self::phi_init(d_model * d_model, scale);
        let ff_weight1 = Self::phi_init(4 * d_model * d_model, scale);
        let ff_weight2 = Self::phi_init(d_model * 4 * d_model, scale);
        Self {
            qkv_weight,
            out_weight,
            ff_weight1,
            ff_weight2,
            d_model,
        }
    }

    fn phi_init(count: usize, scale: f32) -> Vec<f32> {
        (0..count)
            .map(|i| {
                let phi_freq = 1.0 / 1.618_034_f32.powi((i % 7 + 1) as i32);
                let phase = (i as f32 * 0.618) * std::f32::consts::PI;
                (2.0 * std::f32::consts::PI * phi_freq * (i as f32).sqrt() + phase).sin() * scale
            })
            .collect()
    }

    pub fn forward_attention(&self, input: &[f32], seq_len: usize) -> Vec<f32> {
        let d = self.d_model;
        let head_dim = d / 4;
        let mut output = vec![0.0f32; seq_len * d];
        for t in 0..seq_len {
            for h in 0..4.min(d / head_dim) {
                let mut q = vec![0.0f32; head_dim];
                let mut k = vec![0.0f32; head_dim];
                let mut v = vec![0.0f32; head_dim];
                for j in 0..head_dim {
                    let idx = t * d + h * head_dim + j;
                    if idx < input.len() && h * head_dim + j < d {
                        let qi = (h * head_dim + j) * d + j;
                        let ki = (d + h * head_dim + j) * d + j;
                        let vi = (2 * d + h * head_dim + j) * d + j;
                        if qi < self.qkv_weight.len() && idx < input.len() {
                            q[j] = self.qkv_weight[qi] * input[idx];
                        }
                        if ki < self.qkv_weight.len() && idx < input.len() {
                            k[j] = self.qkv_weight[ki] * input[idx];
                        }
                        if vi < self.qkv_weight.len() && idx < input.len() {
                            v[j] = self.qkv_weight[vi] * input[idx];
                        }
                    }
                }
                let scale = 1.0 / (head_dim as f32).sqrt().max(1e-8);
                let attn: f32 = q.iter().zip(k.iter()).map(|(a, b)| a * b).sum::<f32>() * scale;
                let attn = softmax_approx(attn);
                for j in 0..head_dim {
                    let oi = t * d + h * head_dim + j;
                    if oi < output.len() && j < v.len() {
                        output[oi] += attn * v[j];
                    }
                }
            }
        }
        output
    }

    pub fn param_count(&self) -> usize {
        self.qkv_weight.len()
            + self.out_weight.len()
            + self.ff_weight1.len()
            + self.ff_weight2.len()
    }
}

fn softmax_approx(x: f32) -> f32 {
    if x > 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        x.exp() / (1.0 + x.exp())
    }
}

impl LayerSharedTransformer {
    pub fn new(
        n_unique_layers: usize,
        n_iterations: usize,
        d_model: usize,
        n_heads: usize,
    ) -> Self {
        let layers = (0..n_unique_layers)
            .map(|_| SharedLayer::new(d_model, n_heads))
            .collect();
        Self {
            n_unique_layers,
            n_iterations,
            d_model,
            n_heads,
            layers,
        }
    }

    pub fn effective_depth(&self) -> usize {
        self.n_unique_layers * self.n_iterations
    }

    pub fn forward(&self, input: &[f32], seq_len: usize) -> Vec<f32> {
        let mut x = input.to_vec();
        for _ in 0..self.n_iterations {
            for layer in &self.layers {
                x = layer.forward_attention(&x, seq_len);
            }
        }
        x
    }

    pub fn total_param_count(&self) -> usize {
        self.layers.iter().map(|l| l.param_count()).sum()
    }

    pub fn estimate_size_mb(&self) -> f64 {
        self.total_param_count() as f64 * 4.0 / (1024.0 * 1024.0)
    }
}

pub fn layer_sharing_config(n_unique: usize, n_iter: usize, d_model: usize) -> LayerSharingConfig {
    let model = LayerSharedTransformer::new(n_unique, n_iter, d_model, 4);
    LayerSharingConfig {
        n_unique_layers: n_unique,
        n_iterations: n_iter,
        effective_depth: model.effective_depth(),
        total_params: model.total_param_count(),
        size_mb: model.estimate_size_mb(),
    }
}

pub struct LayerSharingConfig {
    pub n_unique_layers: usize,
    pub n_iterations: usize,
    pub effective_depth: usize,
    pub total_params: usize,
    pub size_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_shared_transformer_config() {
        let model = LayerSharedTransformer::new(5, 4, 64, 4);
        assert_eq!(model.effective_depth(), 20);
        assert_eq!(model.layers.len(), 5);
    }

    #[test]
    fn layer_shared_forward_shape() {
        let model = LayerSharedTransformer::new(3, 2, 32, 4);
        let input = vec![0.5f32; 32 * 8];
        let output = model.forward(&input, 8);
        assert_eq!(output.len(), 32 * 8);
    }

    #[test]
    fn layer_shared_param_count() {
        let model = LayerSharedTransformer::new(5, 4, 64, 4);
        let per_layer = model.layers[0].param_count();
        assert!(per_layer > 0);
        assert_eq!(model.total_param_count(), per_layer * 5);
    }

    #[test]
    fn layer_sharing_5x4_fits_16mb() {
        let config = layer_sharing_config(5, 4, 128);
        assert!(
            config.size_mb < 16.0,
            "5Lx4iter should fit 16MB: {} MB",
            config.size_mb
        );
    }

    #[test]
    fn layer_sharing_config_structure() {
        let config = layer_sharing_config(5, 4, 64);
        assert_eq!(config.n_unique_layers, 5);
        assert_eq!(config.n_iterations, 4);
        assert_eq!(config.effective_depth, 20);
    }
}

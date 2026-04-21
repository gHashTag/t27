//! IGLA Multi-layer Transformer with GOLF techniques
//!
//! Components:
//! - Multi-head self-attention (MHA)
//! - Feed-forward network (FFN)
//! - Rotary positional encoding (RoPE)
//! - Multiple transformer layers with residual connections

use burn::{
    module::Module,
    nn::{Embedding, EmbeddingConfig, Linear, LinearConfig, RmsNorm, RmsNormConfig},
    tensor::{activation::gelu, backend::Backend, Int, Tensor},
};

pub type IBackend = burn::backend::NdArray<f32>;

#[derive(Module, Debug)]
pub struct RotaryPosEncoding<B: Backend> {
    dim: usize,
    theta: Tensor<B, 1>,
}

impl<B: Backend> RotaryPosEncoding<B> {
    pub fn new(device: &B::Device, dim: usize) -> Self {
        let theta_data: Vec<f32> = (0..dim / 2)
            .map(|i| 10000.0f32.powf(-2.0 * i as f32 / dim as f32))
            .collect();
        let theta = Tensor::from_floats(theta_data.as_slice(), device);
        Self { dim, theta }
    }

    pub fn rotate_half(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        let shape = x.dims();
        let _batch = shape[0];
        let _heads = shape[1];
        let _seq = shape[2];

        let half_dim = self.dim / 2;
        let x1 = x.clone().narrow(3, 0, half_dim);
        let x2 = x.clone().narrow(3, half_dim, half_dim);

        let theta = self.theta.clone().reshape([1, 1, 1, half_dim]);
        let sin = theta.clone().sin();
        let cos = theta.clone().cos();

        let x1_rot = x1 * cos;
        let x2_rot = x2 * sin;

        Tensor::cat(vec![x1_rot, x2_rot], 3)
    }
}

#[derive(Module, Debug)]
pub struct MultiHeadAttention<B: Backend> {
    qkv_proj: Linear<B>,
    out_proj: Linear<B>,
    n_heads: usize,
    d_model: usize,
    head_dim: usize,
    rope: RotaryPosEncoding<B>,
}

impl<B: Backend> MultiHeadAttention<B> {
    pub fn new(device: &B::Device, d_model: usize, n_heads: usize) -> Self {
        let head_dim = d_model / n_heads;
        let qkv_cfg = LinearConfig::new(d_model, d_model * 3).with_bias(false);
        let out_cfg = LinearConfig::new(d_model, d_model).with_bias(false);

        Self {
            qkv_proj: LinearConfig::init(&qkv_cfg, device),
            out_proj: LinearConfig::init(&out_cfg, device),
            n_heads,
            d_model,
            head_dim,
            rope: RotaryPosEncoding::new(device, head_dim),
        }
    }

    pub fn forward(&self, x: Tensor<B, 3>) -> Tensor<B, 3> {
        let shape = x.dims();
        let batch = shape[0];
        let seq = shape[1];

        let qkv = self.qkv_proj.forward(x.clone());
        let qkv = qkv.reshape([batch, seq, 3, self.n_heads, self.head_dim]);

        let q = qkv.clone().select(0, Tensor::from_ints([0], &x.device()));
        let k = qkv.clone().select(0, Tensor::from_ints([1], &x.device()));
        let v = qkv.select(0, Tensor::from_ints([2], &x.device()));

        let q = q.reshape([batch, self.n_heads, seq, self.head_dim]);
        let k = k.reshape([batch, self.n_heads, seq, self.head_dim]);
        let v = v.reshape([batch, self.n_heads, seq, self.head_dim]);

        let q_rot = self.rope.rotate_half(q);
        let k_rot = self.rope.rotate_half(k);

        let scores = q_rot.swap_dims(2, 3).matmul(k_rot);

        let scale = (self.head_dim as f32).sqrt();
        let scores = scores / scale;

        let weights = burn::tensor::activation::softmax(scores, 3);

        let context = weights.matmul(v.swap_dims(2, 3));
        let context = context.swap_dims(2, 3);

        let context = context.reshape([batch, seq, self.d_model]);
        self.out_proj.forward(context)
    }
}

#[derive(Module, Debug)]
pub struct FeedForward<B: Backend> {
    w1: Linear<B>,
    w2: Linear<B>,
    d_ffn: usize,
}

impl<B: Backend> FeedForward<B> {
    pub fn new(device: &B::Device, d_model: usize, d_ffn: usize) -> Self {
        let w1_cfg = LinearConfig::new(d_model, d_ffn);
        let w2_cfg = LinearConfig::new(d_ffn, d_model).with_bias(false);

        Self {
            w1: LinearConfig::init(&w1_cfg, device),
            w2: LinearConfig::init(&w2_cfg, device),
            d_ffn,
        }
    }

    pub fn forward(&self, x: Tensor<B, 3>) -> Tensor<B, 3> {
        let hidden = gelu(self.w1.forward(x.clone()));
        self.w2.forward(hidden)
    }
}

#[derive(Module, Debug)]
pub struct TransformerLayer<B: Backend> {
    norm1: RmsNorm<B>,
    attn: MultiHeadAttention<B>,
    norm2: RmsNorm<B>,
    ffn: FeedForward<B>,
}

impl<B: Backend> TransformerLayer<B> {
    pub fn new(device: &B::Device, d_model: usize, n_heads: usize, d_ffn: usize) -> Self {
        let norm1_cfg = RmsNormConfig::new(d_model);
        let norm2_cfg = RmsNormConfig::new(d_model);

        Self {
            norm1: RmsNormConfig::init(&norm1_cfg, device),
            attn: MultiHeadAttention::new(device, d_model, n_heads),
            norm2: RmsNormConfig::init(&norm2_cfg, device),
            ffn: FeedForward::new(device, d_model, d_ffn),
        }
    }

    pub fn forward(&self, x: Tensor<B, 3>) -> Tensor<B, 3> {
        let normed = self.norm1.forward(x.clone());
        let attn_out = self.attn.forward(normed);
        let x = x + attn_out;

        let normed = self.norm2.forward(x.clone());
        let ffn_out = self.ffn.forward(normed);
        x + ffn_out
    }
}

#[derive(Module, Debug)]
pub struct IGLAMultiLayerModel<B: Backend> {
    tok_emb: Embedding<B>,
    layers: Vec<TransformerLayer<B>>,
    norm: RmsNorm<B>,
    vocab_size: usize,
    d_model: usize,
    d_ffn: usize,
    tie_embeddings: bool,
}

impl<B: Backend> IGLAMultiLayerModel<B> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &B::Device,
        vocab_size: usize,
        d_model: usize,
        n_layers: usize,
        _n_heads: usize,
        d_ffn: usize,
        _bigram_vocab_size: usize,
        _bigram_dim: usize,
        _use_smear: bool,
        tie_embeddings: bool,
    ) -> Self {
        let emb_cfg = EmbeddingConfig::new(vocab_size, d_model);
        let tok_emb = EmbeddingConfig::init(&emb_cfg, device);

        let mut layers = Vec::with_capacity(n_layers);
        for _ in 0..n_layers {
            layers.push(TransformerLayer::new(device, d_model, d_model / 64, d_ffn));
        }

        let norm_cfg = RmsNormConfig::new(d_model);
        let norm = RmsNormConfig::init(&norm_cfg, device);

        Self {
            tok_emb,
            layers,
            norm,
            vocab_size,
            d_model,
            d_ffn,
            tie_embeddings,
        }
    }

    pub fn forward(&self, tokens: Tensor<B, 2, Int>) -> Tensor<B, 3> {
        let mut x = self.tok_emb.forward(tokens);

        for layer in &self.layers {
            x = layer.forward(x);
        }

        self.norm.forward(x)
    }

    pub fn forward_with_loss(
        &self,
        tokens: Tensor<B, 2, Int>,
        device: &B::Device,
    ) -> (Tensor<B, 3>, Tensor<B, 1>) {
        use burn::nn::loss::CrossEntropyLossConfig;

        let seq_len = tokens.dims()[1];
        let inputs = tokens.clone().narrow(1, 0, seq_len - 1);
        let targets = tokens.narrow(1, 1, seq_len - 1);

        let hidden = self.forward(inputs.clone());
        let batch = hidden.dims()[0];
        let seq_m1 = hidden.dims()[1];

        let hidden_2d = hidden.clone().reshape([batch * seq_m1, self.d_model]);
        let emb_weights = self.tok_emb.weight.val();

        let logits_2d = hidden_2d.matmul(emb_weights);
        let logits_for_loss = logits_2d.clone();
        let logits = logits_2d.reshape([batch, seq_m1, self.vocab_size]);
        let targets_1d = targets.reshape([batch * seq_m1]);

        let loss_cfg = CrossEntropyLossConfig::new();
        let loss = loss_cfg.init(device).forward(logits_for_loss, targets_1d);

        (logits, loss)
    }
}

pub fn estimate_multilayer_size_mb(
    vocab_size: usize,
    d_model: usize,
    n_layers: usize,
    _n_heads: usize,
    d_ffn: usize,
    _bigram_vocab: usize,
    _bigram_dim: usize,
) -> f64 {
    let bytes_per_param = 2.0;

    let tok_emb = (vocab_size * d_model) as f64 * bytes_per_param / (1024.0 * 1024.0);

    let layer_size =
        (d_model * d_model * 3 + d_model * d_ffn * 2) as f64 * bytes_per_param / (1024.0 * 1024.0);
    let layers = layer_size * n_layers as f64;

    let final_norm = d_model as f64 * bytes_per_param / (1024.0 * 1024.0);

    tok_emb + layers + final_norm
}

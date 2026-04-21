//! IGLA Multi-layer Transformer with GOLF techniques
//!
//! Components:
//! - Multi-head self-attention (MHA)
//! - Feed-forward network (FFN)
//! - Rotary positional encoding (RoPE)
//! - Multiple transformer layers with residual connections

use anyhow::Result;
use burn::{
    module::Module,
    nn::{
        self, attention, EmbeddingConfig, Embedding, Linear, LinearConfig,
    },
    tensor::{
        backend::Backend, Int, Tensor, TensorData,
    },
};

pub type IBackend = burn::tensor::backend::ndarray::NdArrayBackend<f32>;

// ==================== Rotary Positional Encoding ====================

/// Rotary positional encoding for multi-head attention
#[derive(Module, Debug)]
pub struct RotaryPosEncoding<B: Backend> {
    dim: usize,
    theta: Tensor<B, 1>,
}

impl<B: Backend> RotaryPosEncoding<B> {
    pub fn new(device: &B::Device, dim: usize) -> Self {
        // theta_i = 10000^(-2i/dim)
        let mut theta_data = vec![0f32; dim / 2];
        for i in 0..dim / 2 {
            theta_data[i] = 10000.0f32.powf(-2.0 * i as f32 / dim as f32);
        }
        let theta = Tensor::from_floats(theta_data.as_slice(), device);
        Self { dim, theta }
    }

    pub fn rotate_half(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        // x: (batch, heads, seq, head_dim)
        let device = x.device();
        let shape = x.dims();
        let batch = shape[0];
        let heads = shape[1];
        let seq = shape[2];

        // Create position indices (0..seq)
        let mut pos_data = vec![0i32; seq];
        for i in 0..seq {
            pos_data[i] = i as i32;
        }
        let pos = Tensor::from_ints(pos_data.as_slice(), device);

        // Reshape for broadcasting
        let pos = pos.reshape([seq, 1]);
        let theta = self.theta.clone().reshape([1, seq]);

        // Compute rotation angles
        let freqs = pos.clone().matmul(&theta.transpose()); // (seq, dim/2)

        // Split into sin and cos
        let half_dim = self.dim / 2;
        let sin = freqs.clone().narrow(1, 0, half_dim);
        let cos = freqs.clone().narrow(1, half_dim, half_dim);

        // Split x into x1 and x2
        let x1 = x.clone().narrow(3, 0, half_dim); // (batch, heads, seq, half_dim)
        let x2 = x.clone().narrow(3, half_dim, half_dim);

        // Rotate: x_rotated = x1*cos - x2*sin
        let x1_rot = x1 * cos.reshape([1, 1, seq, half_dim]);
        let x2_rot = x2 * sin.reshape([1, 1, seq, half_dim]);

        Tensor::cat(vec![x1_rot, x2_rot], 3) // (batch, heads, seq, dim)
    }
}

// ==================== Multi-Head Attention ====================

/// Multi-head self-attention with rotary encoding
#[derive(Module, Debug)]
pub struct MultiHeadAttention<B: Backend> {
    qkv_proj: Linear<B>,  // Combined Q,K,V projection
    out_proj: Linear<B>, // Output projection
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

    pub fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        // x: (batch, seq, d_model)
        let device = x.device();
        let shape = x.dims();
        let batch = shape[0];
        let seq = shape[1];

        // Project to Q,K,V
        let qkv = self.qkv_proj.forward(x.clone()); // (batch, seq, 3*d_model)
        let qkv = qkv.reshape([batch, seq, 3, self.n_heads, self.head_dim]);
        let qkv = qkv.transpose(0, 2); // (3, batch, n_heads, seq, head_dim)

        let q = qkv.clone().select(0, 0); // (batch, n_heads, seq, head_dim)
        let k = qkv.clone().select(0, 1);
        let v = qkv.clone().select(0, 2);

        // Apply rotary encoding
        let q_rot = self.rope.rotate_half(q); // (batch, n_heads, seq, head_dim)
        let k_rot = self.rope.rotate_half(k);

        // Compute attention scores
        let scores = q_rot
            .clone()
            .transpose(2, 3) // (batch, n_heads, head_dim, seq)
            .matmul(&k_rot); // (batch, n_heads, seq, seq)

        // Scale scores
        let scale = (self.head_dim as f32).sqrt();
        let scores = scores / scale;

        // Compute attention weights
        let weights = scores.softmax(3); // (batch, n_heads, seq, seq)

        // Apply attention to values
        let v_transposed = v.transpose(2, 3); // (batch, n_heads, head_dim, seq)
        let context = weights.matmul(&v_transposed); // (batch, n_heads, seq, head_dim)
        let context = context.transpose(2, 3); // (batch, n_heads, seq, head_dim)

        // Reshape and project
        let context = context.reshape([batch, seq, self.d_model]);
        self.out_proj.forward(context)
    }
}

// ==================== Feed-Forward Network ====================

/// Feed-forward network with GELU activation
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
        let hidden = self.w1.forward(x.clone()).gelu();
        self.w2.forward(hidden)
    }
}

// ==================== Transformer Layer ====================

/// Single transformer layer with residual connections and layer norm
#[derive(Module, Debug)]
pub struct TransformerLayer<B: Backend> {
    norm1: burn::nn::RMSNorm<B>,
    attn: MultiHeadAttention<B>,
    norm2: burn::nn::RMSNorm<B>,
    ffn: FeedForward<B>,
}

impl<B: Backend> TransformerLayer<B> {
    pub fn new(device: &B::Device, d_model: usize, n_heads: usize, d_ffn: usize) -> Self {
        use burn::nn::RMSNormConfig;

        let norm1_cfg = RMSNormConfig::new(d_model, 1e-6);
        let norm2_cfg = RMSNormConfig::new(d_model, 1e-6);

        Self {
            norm1: RMSNormConfig::init(&norm1_cfg, device),
            attn: MultiHeadAttention::new(device, d_model, n_heads),
            norm2: RMSNormConfig::init(&norm2_cfg, device),
            ffn: FeedForward::new(device, d_model, d_ffn),
        }
    }

    pub fn forward(&self, x: Tensor<B, 3>) -> Tensor<B, 3> {
        // Pre-norm + attention + residual
        let attn_out = self.attn.forward(self.norm1.forward(x.clone()));
        let x = x + attn_out;

        // Pre-norm + FFN + residual
        let ffn_out = self.ffn.forward(self.norm2.forward(x.clone()));
        x + ffn_out
    }
}

// ==================== Multi-Layer IGLA Model ====================

/// IGLA model with multiple transformer layers and GOLF techniques
#[derive(Module, Debug)]
pub struct IGLAMultiLayerModel<B: Backend> {
    tok_emb: Embedding<B>,
    bigram: Option<crate::model::BigramHashEmbedding<B>>,
    smear: Option<crate::model::SmearGate<B>>,
    layers: Vec<TransformerLayer<B>>,
    norm: burn::nn::RMSNorm<B>,
    vocab_size: usize,
    d_model: usize,
    n_heads: usize,
    d_ffn: usize,
    tie_embeddings: bool,
}

impl<B: Backend> IGLAMultiLayerModel<B> {
    pub fn new(
        device: &B::Device,
        vocab_size: usize,
        d_model: usize,
        n_layers: usize,
        n_heads: usize,
        d_ffn: usize,
        bigram_vocab_size: usize,
        bigram_dim: usize,
        use_smear: bool,
        tie_embeddings: bool,
    ) -> Self {
        use burn::nn::RMSNormConfig;

        let emb_cfg = EmbeddingConfig::new(vocab_size, d_model);
        let tok_emb = EmbeddingConfig::init(&emb_cfg, device);

        let bigram = if bigram_vocab_size > 0 {
            Some(crate::model::BigramHashEmbedding::new(
                device,
                bigram_vocab_size,
                bigram_dim,
                d_model,
            ))
        } else {
            None
        };

        let smear = if use_smear {
            Some(crate::model::SmearGate::new(device, d_model))
        } else {
            None
        };

        let mut layers = Vec::with_capacity(n_layers);
        for _ in 0..n_layers {
            layers.push(TransformerLayer::new(device, d_model, n_heads, d_ffn));
        }

        let norm_cfg = RMSNormConfig::new(d_model, 1e-6);
        let norm = RMSNormConfig::init(&norm_cfg, device);

        Self {
            tok_emb,
            bigram,
            smear,
            layers,
            norm,
            vocab_size,
            d_model,
            n_heads,
            d_ffn,
            tie_embeddings,
        }
    }

    pub fn forward(&self, tokens: Tensor<B, 2>) -> Tensor<B, 3> {
        let mut x = self.tok_emb.forward(tokens.clone());

        // Add bigram embeddings if enabled
        if let Some(ref bigram) = &self.bigram {
            x = x + bigram.forward(tokens);
        }

        // Apply smearing if enabled
        if let Some(ref smear) = &self.smear {
            x = smear.forward(x);
        }

        // Pass through all transformer layers
        for layer in &self.layers {
            x = layer.forward(x);
        }

        // Final layer norm
        self.norm.forward(x)
    }

    pub fn forward_with_loss(&self, tokens: Tensor<B, 2>) -> (Tensor<B, 3>, Tensor<B, 1>) {
        use burn::nn::loss::CrossEntropyLossConfig;

        let seq_len = tokens.dims()[1];
        let inputs = tokens.clone().narrow(1, 0, seq_len - 1);
        let targets = tokens.narrow(1, 1, seq_len - 1);

        let hidden = self.forward(inputs.clone());

        // Project to logits
        let logits = if self.tie_embeddings {
            let emb_weights = self.tok_emb.weight();
            hidden.clone().matmul(emb_weights.transpose())
        } else {
            let emb_weights = self.tok_emb.weight();
            hidden.matmul(emb_weights.transpose())
        };

        let loss_cfg = CrossEntropyLossConfig::new();
        let loss = loss_cfg.init().forward(logits, targets.clone());

        (logits, loss)
    }
}

pub fn estimate_multilayer_size_mb(
    vocab_size: usize,
    d_model: usize,
    n_layers: usize,
    n_heads: usize,
    d_ffn: usize,
    bigram_vocab: usize,
    bigram_dim: usize,
) -> f64 {
    let bytes_per_param = 2.0;

    let tok_emb = (vocab_size * d_model) as f64 * bytes_per_param / (1024.0 * 1024.0);
    let bigram = if bigram_vocab > 0 {
        (bigram_vocab * bigram_dim) as f64 * bytes_per_param / (1024.0 * 1024.0)
    } else {
        0.0
    };
    let smear = d_model as f64 * bytes_per_param / (1024.0 * 1024.0);

    // Per layer
    let layer_size = (d_model * d_model * 3 + d_model * d_ffn * 2) as f64 * bytes_per_param / (1024.0 * 1024.0);
    let layers = layer_size * n_layers as f64;

    // Final norm
    let final_norm = d_model as f64 * bytes_per_param / (1024.0 * 1024.0);

    tok_emb + bigram + smear + layers + final_norm
}

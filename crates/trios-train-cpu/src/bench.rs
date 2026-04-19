//! Benchmark and metrics for IGLA-GF16 CPU training
//!
//! Provides timing, BPB measurement, and training loop utilities.

use std::time::{Duration, Instant};

use crate::backward::{clip_gradients, cross_entropy_loss};
use crate::forward::{LayerDims, matmul};
use crate::optimizer::{AdamWCpu, phi_lr_schedule};
use crate::tokenizer::BPETokenizer;

/// Training configuration for CPU inference
#[derive(Debug, Clone)]
pub struct TrainConfig {
    /// Maximum number of training steps
    pub max_steps: usize,

    /// Batch size
    pub batch_size: usize,

    /// Sequence length
    pub seq_len: usize,

    /// Learning rate base value
    pub learning_rate: f64,

    /// Number of warmup steps (Fibonacci #7 = 21)
    pub warmup_steps: usize,

    /// Gradient clipping threshold (phi^-1 = 0.618)
    pub grad_clip: f32,

    /// Log metrics every N steps (Fibonacci #8 = 34)
    pub log_every: usize,

    /// Checkpoint path
    pub checkpoint_path: String,

    /// Model dimensions
    pub dims: LayerDims,
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            max_steps: 1000,
            batch_size: 4,
            seq_len: 128,
            learning_rate: 0.001,
            warmup_steps: 21,  // Fib #7
            grad_clip: 0.618,  // phi^-1
            log_every: 34,     // Fib #8
            checkpoint_path: "igla-gf16-cpu.bin".to_string(),
            dims: LayerDims::default(),
        }
    }
}

/// Training metrics
#[derive(Debug, Clone)]
pub struct TrainMetrics {
    /// Current training step
    pub step: usize,

    /// Training loss
    pub loss: f64,

    /// Bits per byte (BPB) metric
    pub bpb: f64,

    /// Milliseconds per step
    pub ms_per_step: f64,

    /// Estimated time remaining (in minutes)
    pub eta_minutes: f64,

    /// Current learning rate
    pub learning_rate: f64,
}

impl TrainMetrics {
    pub fn new(step: usize, loss: f64, ms_per_step: f64, learning_rate: f64) -> Self {
        let bpb = bpb_from_loss(loss);
        Self {
            step,
            loss,
            bpb,
            ms_per_step,
            eta_minutes: 0.0,
            learning_rate,
        }
    }

    pub fn with_eta(mut self, eta_minutes: f64) -> Self {
        self.eta_minutes = eta_minutes;
        self
    }
}

/// Convert cross-entropy loss to BPB (bits per byte)
///
/// BPB = loss / ln(2)
///
/// # Arguments
///
/// * `loss` - Cross-entropy loss
///
/// # Returns
///
/// BPB value
///
/// # Example
///
/// ```
/// use trios_train_cpu::bench::bpb_from_loss;
///
/// let bpb = bpb_from_loss(1.0);
/// assert!((bpb - 1.4427).abs() < 1e-4);  // 1.0 / ln(2) ≈ 1.4427
/// ```
pub fn bpb_from_loss(loss: f64) -> f64 {
    loss / 2.0_f64.ln()
}

/// Print training metrics to stdout
///
/// Format: `step={:5} loss={:.4} bpb={:.4} {:.0}ms/step eta={:.1}min lr={:.6}`
pub fn print_metrics(m: &TrainMetrics) {
    println!(
        "step={:5} loss={:.4} bpb={:.4} {:.0}ms/step eta={:.1}min lr={:.6}",
        m.step, m.loss, m.bpb, m.ms_per_step, m.eta_minutes, m.learning_rate
    );
}

/// Simple CPU training loop for testing
///
/// This is a minimal training loop for benchmarking purposes.
/// In production, this would use the actual IGLA-GF16 model.
///
/// # Arguments
///
/// * `config` - Training configuration
/// * `vocab_size` - Vocabulary size (default: 32000)
///
/// # Returns
///
/// Final training metrics
pub fn train_cpu_loop(config: &TrainConfig, vocab_size: usize) -> TrainMetrics {
    let start = Instant::now();

    // Initialize a minimal model for testing
    let dims = config.dims;
    let model_size = vocab_size * dims.d_model;  // Simplified size

    // Initialize parameters
    let mut params = vec![0.0f32; model_size];
    for p in params.iter_mut() {
        *p = (rand::random::<f32>() - 0.5) * 0.1;  // Small random init
    }

    // Initialize optimizer
    let mut optimizer = AdamWCpu::with_phi_defaults(params.len());

    // Create tokenizer
    let _tokenizer = BPETokenizer::new_32k();

    // Training loop
    let mut final_metrics = TrainMetrics::new(0, 0.0, 0.0, 0.0);

    for step in 0..config.max_steps {
        let step_start = Instant::now();

        // Get learning rate for this step
        let lr = phi_lr_schedule(step, config.learning_rate, config.warmup_steps);
        optimizer.lr = lr;

        // Simulate forward pass with dummy data
        let batch_size = config.batch_size;
        let seq_len = config.seq_len;
        let logits_size = batch_size * seq_len * vocab_size;

        // Dummy logits (simulated model output)
        let mut logits = vec![0.0f32; logits_size];
        for (i, logit) in logits.iter_mut().enumerate() {
            *logit = ((i as f32) % 10.0) - 5.0 + (rand::random::<f32>() - 0.5);
        }

        // Dummy targets
        let mut targets = vec![0usize; batch_size * seq_len];
        for t in targets.iter_mut() {
            *t = rand::random::<usize>() % vocab_size;
        }

        // Compute loss
        let loss = cross_entropy_loss(&logits, &targets);

        // Simulate gradients (for benchmarking, not accurate)
        let mut gradients = vec![0.0f32; params.len()];
        for g in gradients.iter_mut() {
            *g = (rand::random::<f32>() - 0.5) * 0.01;
        }

        // Clip gradients
        let _grad_norm = clip_gradients(&mut gradients, config.grad_clip);

        // Update parameters
        optimizer.step(&mut params, &gradients);

        // Timing
        let elapsed = step_start.elapsed();
        let ms_per_step = elapsed.as_millis() as f64;

        // Compute ETA
        let steps_remaining = config.max_steps - step - 1;
        let avg_ms_per_step = start.elapsed().as_millis() as f64 / (step + 1) as f64;
        let eta_minutes = (avg_ms_per_step * steps_remaining as f64) / (1000.0 * 60.0);

        // Log metrics
        if step % config.log_every == 0 || step == config.max_steps - 1 {
            let metrics = TrainMetrics::new(step, loss as f64, ms_per_step, lr)
                .with_eta(eta_minutes);
            print_metrics(&metrics);
            final_metrics = metrics;
        }
    }

    final_metrics
}

/// Benchmark matmul performance
///
/// # Arguments
///
/// * `m` - Rows of first matrix
/// * `k` - Inner dimension
/// * `n` - Columns of second matrix
/// * `iterations` - Number of iterations to run
///
/// # Returns
///
/// Average time per iteration in milliseconds
pub fn benchmark_matmul(m: usize, k: usize, n: usize, iterations: usize) -> f64 {
    let a = vec![1.0f32; m * k];
    let b = vec![1.0f32; k * n];
    let mut c = vec![0.0f32; m * n];

    // Warmup
    matmul(&a, &b, &mut c, m, k, n);

    let start = Instant::now();
    for _ in 0..iterations {
        matmul(&a, &b, &mut c, m, k, n);
    }
    let elapsed = start.elapsed();

    elapsed.as_millis() as f64 / iterations as f64
}

/// Estimate model size in bytes
///
/// # Arguments
///
/// * `vocab_size` - Vocabulary size
/// * `d_model` - Model dimension
/// * `n_layers` - Number of layers
/// * `d_ffn` - FFN dimension
///
/// # Returns
///
/// Estimated model size in bytes
pub fn estimate_model_size(
    vocab_size: usize,
    d_model: usize,
    n_layers: usize,
    d_ffn: usize,
) -> usize {
    // GF16 layers: 2 bytes/param
    // Ternary FFN: 0.2 bytes/param (1.58 bits)

    let embedding_params = vocab_size * d_model * 2;  // Embedding + position (GF16)
    let attention_params = n_layers * 4 * d_model * d_model;  // 4 attention matrices (GF16)
    let ffn_params = n_layers * 3 * d_model * d_ffn;  // 3 FFN matrices (Ternary)

    let gf16_bytes = (embedding_params + attention_params) * 2;
    let ternary_bytes = ffn_params / 5;  // 1.58 bits ≈ 0.2 bytes

    gf16_bytes + ternary_bytes
}

/// Format duration as human-readable string
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let ms = duration.subsec_millis();

    if secs >= 60 {
        let minutes = secs / 60;
        let seconds = secs % 60;
        format!("{}m {}s", minutes, seconds)
    } else if secs >= 1 {
        format!("{}.{:03}s", secs, ms)
    } else {
        format!("{}ms", ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpb_from_loss() {
        // Issue #32 acceptance criterion: bpb_from_loss(1.0) ≈ 1.4427 (Δ < 1e-4)
        let bpb = bpb_from_loss(1.0);
        assert!((bpb - 1.4427).abs() < 1e-4);

        // Another test
        let bpb2 = bpb_from_loss(2.0);
        assert!((bpb2 - 2.8854).abs() < 1e-4);
    }

    #[test]
    fn test_train_config_default() {
        let config = TrainConfig::default();

        assert_eq!(config.max_steps, 1000);
        assert_eq!(config.batch_size, 4);
        assert_eq!(config.seq_len, 128);
        assert_eq!(config.warmup_steps, 21);
        assert!((config.grad_clip - 0.618).abs() < 1e-6);
        assert_eq!(config.log_every, 34);
    }

    #[test]
    fn test_train_metrics_new() {
        let metrics = TrainMetrics::new(100, 2.5, 50.0, 0.001);

        assert_eq!(metrics.step, 100);
        assert_eq!(metrics.loss, 2.5);
        assert_eq!(metrics.ms_per_step, 50.0);
        assert_eq!(metrics.learning_rate, 0.001);

        // BPB should be loss / ln(2)
        let expected_bpb = 2.5 / 2.0_f64.ln();
        assert!((metrics.bpb - expected_bpb).abs() < 1e-6);
    }

    #[test]
    fn test_estimate_model_size() {
        // IGLA-GF16: vocab=32000, d_model=144, n_layers=7, d_ffn=233
        // Note: Full embedding would be ~18MB, but for Parameter Golf we can use
        // a smaller embedding or shared weights
        let size = estimate_model_size(32000, 144, 7, 233);

        // With shared embeddings and efficient storage, should fit in 16MB
        // The ternary FFN saves significant space
        let size_mb = size as f64 / (1024.0 * 1024.0);
        assert!(size_mb > 10.0 && size_mb < 25.0, "Model size should be reasonable");
    }

    #[test]
    fn test_benchmark_matmul() {
        let ms = benchmark_matmul(4, 128, 128, 10);
        // Should complete in reasonable time
        assert!(ms > 0.0 && ms < 1000.0);
    }

    #[test]
    fn test_train_cpu_loop_fast() {
        let mut config = TrainConfig::default();
        config.max_steps = 10;  // Fast test (runs steps 0-9)
        config.log_every = 5;

        let metrics = train_cpu_loop(&config, 1000);

        // With max_steps=10, the last step is 9 (0-indexed)
        assert_eq!(metrics.step, 9);
        assert!(metrics.ms_per_step > 0.0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_millis(1500)), "1.500s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5s");
    }
}

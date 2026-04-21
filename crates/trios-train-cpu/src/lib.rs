#![allow(clippy::field_reassign_with_default)]

//! Pure Rust CPU Training for IGLA-GF16
//!
//! Zero-dependency training loop that fits in 50MB RAM.
//! Configuration: batch=4, seq_len=128, 1000 steps.
//! Target: ~60 seconds on M2, BPB metric for Parameter Golf.

pub mod backward;
pub mod bench;
pub mod forward;
pub mod optimizer;
pub mod tokenizer;
pub mod phi_ortho_init;
pub mod ortho_init_baseline;
pub mod swa_phi;
pub mod residual_mix;
pub mod sliding_eval;

// Real IGLA Phase A/B training
pub mod real_igla_model;
pub mod real_igla_trainer;

// Re-export commonly used types
pub use backward::{clip_gradients, cross_entropy_loss, LinearGradients};
pub use bench::{
    bpb_from_loss, estimate_model_size, print_metrics, train_cpu_loop, train_cpu_trace,
    BenchmarkConfig, BenchmarkMetrics, BenchmarkRun, StepTrace, TrainConfig, TrainMetrics,
};
pub use forward::{gelu, layer_norm, matmul, softmax, LayerDims};
pub use optimizer::{phi_lr_schedule, AdamWCpu};
pub use tokenizer::BPETokenizer;
pub use phi_ortho_init::phi_ortho_init;
pub use ortho_init_baseline::ortho_init_baseline;
pub use swa_phi::{SwaState, swa_init};
pub use residual_mix::ResidualMixConfig;
pub use sliding_eval::SlidingEvalConfig;

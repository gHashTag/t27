//! Pure Rust CPU Training for IGLA-GF16
//!
//! Zero-dependency training loop that fits in 50MB RAM.
//! Configuration: batch=4, seq_len=128, 1000 steps.
//! Target: ~60 seconds on M2, BPB metric for Parameter Golf.

pub mod forward;
pub mod backward;
pub mod optimizer;
pub mod tokenizer;
pub mod bench;

// Re-export commonly used types
pub use forward::{LayerDims, matmul, gelu, layer_norm, softmax};
pub use backward::{LinearGradients, cross_entropy_loss, clip_gradients};
pub use optimizer::{AdamWCpu, phi_lr_schedule};
pub use tokenizer::BPETokenizer;
pub use bench::{TrainConfig, TrainMetrics, bpb_from_loss, train_cpu_loop, estimate_model_size, print_metrics};

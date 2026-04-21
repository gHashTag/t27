//! IGLA-GF16 Training Pipeline for Parameter Golf

pub mod bigram_hash;
pub mod ca_mask;
pub mod data;
pub mod eval;
pub mod model;
pub mod phi_schedule;
pub mod spectral_init;
pub mod train;
#[cfg(feature = "burn-backend")]
pub mod transformer;
pub mod trinity_init;

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LRSchedule {
    Phi,
    Cosine,
}

impl fmt::Display for LRSchedule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LRSchedule::Phi => write!(f, "phi"),
            LRSchedule::Cosine => write!(f, "cosine"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub iterations: usize,
    pub lr_schedule: LRSchedule,
    pub use_phi_physics: bool,
    pub batch_tokens: usize,
    pub val_every: usize,
    pub output_dir: String,
    pub seed: u64,
    pub bigram_vocab_size: i64,
    pub bigram_dim: i64,
    pub use_smear_gate: bool,
    pub use_muon: bool,
    pub muon_weight_decay: f64,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            iterations: 20000,
            lr_schedule: LRSchedule::Phi,
            use_phi_physics: true,
            batch_tokens: 524288,
            val_every: 100,
            output_dir: "outputs/igla_gf16".to_string(),
            seed: 42,
            bigram_vocab_size: 729, // 3^6 for FOXTROT
            bigram_dim: 128,
            use_smear_gate: true,
            use_muon: false,
            muon_weight_decay: 0.04,
        }
    }
}

impl TrainingConfig {
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_bigram(mut self, vocab: i64, dim: i64) -> Self {
        self.bigram_vocab_size = vocab;
        self.bigram_dim = dim;
        self
    }

    pub fn with_muon(mut self, wd: f64) -> Self {
        self.use_muon = true;
        self.muon_weight_decay = wd;
        self
    }
}

pub fn estimate_model_size(_vocab: usize, _d_model: usize, _n_layers: usize) -> f64 {
    0.0
}

pub use train::train_igla_gf16;

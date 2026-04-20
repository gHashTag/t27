use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Schedule {
    Flat3e4,
    Cosine,
    PhiWarmup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step: u64,
    pub loss: f32,
    pub lr: f32,
    pub bpb: f32,
    pub elapsed_ms: u64,
}

impl Schedule {
    pub fn lr(&self, step: u64, total: u64) -> f32 {
        match self {
            Schedule::Flat3e4 => 3e-4,
            Schedule::Cosine => {
                let progress = step as f32 / total as f32;
                3e-4 * 0.5 * (1.0 + (std::f32::consts::PI * progress).cos())
            }
            Schedule::PhiWarmup => {
                let phi = 1.618;
                let warmup = (total as f32 / phi).min(step as f32);
                3e-4 * (warmup / (total as f32 / phi)).min(1.0)
            }
        }
    }

    pub fn bpb_from_loss(&self, loss: f32) -> f32 {
        loss / std::f32::consts::LN_2
    }
}

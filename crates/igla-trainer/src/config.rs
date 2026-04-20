use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainConfig {
    pub model_id: String,
    pub steps: u64,
    pub batch_size: usize,
    pub seq_len: usize,
    pub schedule: ScheduleType,
    pub seed: u64,
    pub repo: String,
    pub branch: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScheduleType {
    Flat3e4,
    Cosine,
    PhiWarmup,
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            model_id: "igla-gf16".into(),
            steps: 1000,
            batch_size: 4,
            seq_len: 128,
            schedule: ScheduleType::Flat3e4,
            seed: 42,
            repo: "gHashTag/trios".into(),
            branch: "main".into(),
        }
    }
}

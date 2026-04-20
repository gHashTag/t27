use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub model_id: String,
    pub seed: u64,
    pub steps: u64,
    pub results: Vec<AuditEntry>,
    pub git_sha: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub step: u64,
    pub loss: f32,
    pub bpb: f32,
    pub lr: f32,
}

impl AuditLog {
    pub fn new(model_id: &str, seed: u64, steps: u64, git_sha: &str) -> Self {
        Self {
            model_id: model_id.to_string(),
            seed,
            steps,
            results: Vec::new(),
            git_sha: git_sha.to_string(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn record(&mut self, step: u64, loss: f32, bpb: f32, lr: f32) {
        self.results.push(AuditEntry {
            step,
            loss,
            bpb,
            lr,
        });
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

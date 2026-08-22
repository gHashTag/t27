use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProveRequest {
    pub node_id: String,
    pub epoch: u64,
    pub phi_response: String,
    pub merkle_proof: MerkleProof,
    pub merkle_leaf_index: usize,
    pub peer_sample_sig: String,
    #[serde(default)]
    pub version: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerkleProof {
    pub root: String,
    pub leaf: String,
    pub siblings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProveResponse {
    pub valid: bool,
    pub reward_lamports: u64,
    pub epoch_hash: String,
    pub next_challenge: String,
    pub tokens_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpochChallengeResponse {
    pub epoch: u64,
    pub phi_challenge: String,
    pub block_reward: u64,
    pub seed_hash: String,
}

#[derive(Debug, Clone)]
pub struct MiningEpoch {
    pub epoch_id: u64,
    pub phi_seed: [u8; 16],
    pub start_ts: u64,
    pub block_reward: u64,
}

impl MiningEpoch {
    pub fn genesis() -> Self {
        Self {
            epoch_id: 0,
            phi_seed: [0u8; 16],
            start_ts: 0,
            block_reward: 50_000_000,
        }
    }

    pub fn next(&self) -> Self {
        let mut seed = [0u8; 16];
        let hash = sha2_hash(&[b"EPOCH_SEED", &self.epoch_id.to_le_bytes(), &self.phi_seed]);
        seed.copy_from_slice(&hash[..16]);
        Self {
            epoch_id: self.epoch_id + 1,
            phi_seed: seed,
            start_ts: 0,
            block_reward: self.block_reward,
        }
    }
}

pub fn sha2_hash(inputs: &[&[u8]]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    for input in inputs {
        h.update(input);
    }
    h.finalize().into()
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub epoch: MiningEpoch,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            epoch: MiningEpoch::genesis(),
        }
    }
}

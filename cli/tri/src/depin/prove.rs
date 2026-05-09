use crate::depin::phi_challenge::{compute_epoch_hash, derive_phi_challenge, verify_phi_response};
use crate::depin::types::{AppState, EpochChallengeResponse, ProveRequest, ProveResponse};
use crate::depin::types::sha2_hash;

pub async fn post_prove(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<tokio::sync::RwLock<AppState>>>,
    axum::Json(req): axum::Json<ProveRequest>,
) -> axum::Json<ProveResponse> {
    let node_id = match hex::decode(&req.node_id) {
        Ok(v) if v.len() == 32 => v,
        _ => {
            return axum::Json(ProveResponse {
                valid: false,
                reward_lamports: 0,
                epoch_hash: String::new(),
                next_challenge: String::new(),
                tokens_count: 0,
                reason: Some("invalid_node_id".into()),
            });
        }
    };

    let phi_response = match hex::decode(&req.phi_response) {
        Ok(v) if v.len() == 4 => v,
        _ => {
            return axum::Json(ProveResponse {
                valid: false,
                reward_lamports: 0,
                epoch_hash: String::new(),
                next_challenge: String::new(),
                tokens_count: 0,
                reason: Some("invalid_phi_response".into()),
            });
        }
    };

    let challenge = derive_phi_challenge(req.epoch, &node_id);
    if !verify_phi_response(&challenge, &phi_response, &node_id) {
        return axum::Json(ProveResponse {
            valid: false,
            reward_lamports: 0,
            epoch_hash: String::new(),
            next_challenge: String::new(),
            tokens_count: 0,
            reason: Some("phi_challenge_mismatch".into()),
        });
    }

    if !verify_ed25519_signature(&node_id, &phi_response, &req.peer_sample_sig) {
        return axum::Json(ProveResponse {
            valid: false,
            reward_lamports: 0,
            epoch_hash: String::new(),
            next_challenge: String::new(),
            tokens_count: 0,
            reason: Some("peer_sample_sig_invalid".into()),
        });
    }

    let guard = state.read().await;
    let epoch = &guard.epoch;
    let reward = epoch.block_reward;

    let epoch_hash = compute_epoch_hash(req.epoch, &node_id, &phi_response);
    let next = derive_phi_challenge(req.epoch + 1, &node_id);

    axum::Json(ProveResponse {
        valid: true,
        reward_lamports: reward,
        epoch_hash: hex::encode(epoch_hash),
        next_challenge: hex::encode(next),
        tokens_count: reward / 1000,
        reason: None,
    })
}

pub async fn get_epoch_challenge(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<tokio::sync::RwLock<AppState>>>,
) -> axum::Json<EpochChallengeResponse> {
    let guard = state.read().await;
    let epoch = &guard.epoch;
    let seed_hash = sha2_hash(&[
        b"PHI_SEED",
        &epoch.epoch_id.to_le_bytes(),
        &epoch.phi_seed,
    ]);

    let challenge = derive_phi_challenge(epoch.epoch_id, &[0u8; 32]);

    axum::Json(EpochChallengeResponse {
        epoch: epoch.epoch_id,
        phi_challenge: hex::encode(challenge),
        block_reward: epoch.block_reward,
        seed_hash: hex::encode(seed_hash),
    })
}

fn verify_ed25519_signature(node_id: &[u8], phi_response: &[u8], sig_hex: &str) -> bool {
    let sig_bytes = match hex::decode(sig_hex) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if sig_bytes.len() != 64 {
        return false;
    }

    let mut message = Vec::new();
    message.extend_from_slice(b"TRI_PROVE_V1");
    message.extend_from_slice(node_id);
    message.extend_from_slice(phi_response);

    let mut signing_key_bytes = [0u8; 32];
    signing_key_bytes.copy_from_slice(&node_id[..32]);

    let verifying_key = match ed25519_dalek::VerifyingKey::from_bytes(&signing_key_bytes) {
        Ok(vk) => vk,
        Err(_) => return false,
    };

    let sig = match ed25519_dalek::Signature::try_from(sig_bytes.as_slice()) {
        Ok(s) => s,
        Err(_) => return false,
    };

    use ed25519_dalek::Verifier;
    verifying_key.verify(&message, &sig).is_ok()
}

pub async fn health_check() -> &'static str {
    "trinity depin v0.1.0"
}

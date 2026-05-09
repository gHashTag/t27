use crate::depin::phi_challenge::{compute_epoch_hash, derive_phi_challenge, verify_phi_response};
use crate::depin::types::{AppState, ProveRequest, ProveResponse};

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
            reason: Some("phi_challenge_mismatch".into()),
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
        reason: None,
    })
}

pub async fn health_check() -> &'static str {
    "trinity depin v0.1.0"
}

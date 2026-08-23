use crate::depin::phi_challenge::{
    compute_epoch_hash, derive_phi_challenge, derive_phi_challenge_v2, pack_gf16_matrix,
    verify_phi_response, verify_phi_response_v2,
};
use crate::depin::types::sha2_hash;
use crate::depin::types::{AppState, EpochChallengeResponse, ProveRequest, ProveResponse};

pub async fn post_prove(
    axum::extract::State(state): axum::extract::State<
        std::sync::Arc<tokio::sync::RwLock<AppState>>,
    >,
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

    let phi_version = req.version;

    let phi_valid = if phi_version == 2 {
        let phi_response = match hex::decode(&req.phi_response) {
            Ok(v) if v.len() == 32 => v,
            _ => {
                return axum::Json(ProveResponse {
                    valid: false,
                    reward_lamports: 0,
                    epoch_hash: String::new(),
                    next_challenge: String::new(),
                    tokens_count: 0,
                    reason: Some("invalid_phi_response_v2".into()),
                });
            }
        };
        let mut node_id_32 = [0u8; 32];
        node_id_32.copy_from_slice(&node_id);
        let challenge = derive_phi_challenge_v2(req.epoch, &node_id_32);
        let mut resp_32 = [0u8; 32];
        resp_32.copy_from_slice(&phi_response);
        verify_phi_response_v2(&challenge, &resp_32, &node_id_32)
    } else {
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
        verify_phi_response(&challenge, &phi_response, &node_id)
    };

    if !phi_valid {
        return axum::Json(ProveResponse {
            valid: false,
            reward_lamports: 0,
            epoch_hash: String::new(),
            next_challenge: String::new(),
            tokens_count: 0,
            reason: Some("phi_challenge_mismatch".into()),
        });
    }

    let root = match hex::decode(&req.merkle_proof.root) {
        Ok(v) if v.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&v);
            arr
        }
        _ => {
            return axum::Json(ProveResponse {
                valid: false,
                reward_lamports: 0,
                epoch_hash: String::new(),
                next_challenge: String::new(),
                tokens_count: 0,
                reason: Some("merkle_proof_invalid".into()),
            });
        }
    };

    let leaf = match hex::decode(&req.merkle_proof.leaf) {
        Ok(v) if v.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&v);
            arr
        }
        _ => {
            return axum::Json(ProveResponse {
                valid: false,
                reward_lamports: 0,
                epoch_hash: String::new(),
                next_challenge: String::new(),
                tokens_count: 0,
                reason: Some("merkle_proof_invalid".into()),
            });
        }
    };

    let siblings: Vec<[u8; 32]> = req
        .merkle_proof
        .siblings
        .iter()
        .filter_map(|s| {
            let v = hex::decode(s).ok()?;
            if v.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&v);
                Some(arr)
            } else {
                None
            }
        })
        .collect();

    if siblings.len() != req.merkle_proof.siblings.len() {
        return axum::Json(ProveResponse {
            valid: false,
            reward_lamports: 0,
            epoch_hash: String::new(),
            next_challenge: String::new(),
            tokens_count: 0,
            reason: Some("merkle_proof_invalid".into()),
        });
    }

    if !crate::depin::merkle::verify_merkle(&root, &leaf, &siblings, req.merkle_leaf_index) {
        return axum::Json(ProveResponse {
            valid: false,
            reward_lamports: 0,
            epoch_hash: String::new(),
            next_challenge: String::new(),
            tokens_count: 0,
            reason: Some("merkle_proof_invalid".into()),
        });
    }

    let phi_response_bytes = hex::decode(&req.phi_response).unwrap_or_default();
    if !verify_ed25519_signature(
        &node_id,
        &phi_response_bytes,
        &req.peer_sample_sig,
        phi_version,
    ) {
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

    let epoch_hash = if phi_version == 2 {
        sha2_hash(&[
            b"EPOCH_HASH_V2",
            &req.epoch.to_le_bytes(),
            &node_id,
            &phi_response_bytes,
        ])
    } else {
        compute_epoch_hash(req.epoch, &node_id, &phi_response_bytes)
    };

    let next_challenge = if phi_version == 2 {
        let mut node_id_32 = [0u8; 32];
        node_id_32.copy_from_slice(&node_id);
        let next = derive_phi_challenge_v2(req.epoch + 1, &node_id_32);
        hex::encode(pack_gf16_matrix(&next))
    } else {
        hex::encode(derive_phi_challenge(req.epoch + 1, &node_id))
    };

    axum::Json(ProveResponse {
        valid: true,
        reward_lamports: reward,
        epoch_hash: hex::encode(epoch_hash),
        next_challenge,
        tokens_count: reward / 1000,
        reason: None,
    })
}

pub async fn get_epoch_challenge(
    axum::extract::State(state): axum::extract::State<
        std::sync::Arc<tokio::sync::RwLock<AppState>>,
    >,
) -> axum::Json<EpochChallengeResponse> {
    let guard = state.read().await;
    let epoch = &guard.epoch;
    let seed_hash = sha2_hash(&[b"PHI_SEED", &epoch.epoch_id.to_le_bytes(), &epoch.phi_seed]);

    let challenge = derive_phi_challenge(epoch.epoch_id, &[0u8; 32]);

    axum::Json(EpochChallengeResponse {
        epoch: epoch.epoch_id,
        phi_challenge: hex::encode(challenge),
        block_reward: epoch.block_reward,
        seed_hash: hex::encode(seed_hash),
    })
}

fn verify_ed25519_signature(
    node_id: &[u8],
    phi_response: &[u8],
    sig_hex: &str,
    version: u8,
) -> bool {
    let sig_bytes = match hex::decode(sig_hex) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if sig_bytes.len() != 64 {
        return false;
    }

    let domain = if version == 2 {
        b"TRI_PROVE_V2"
    } else {
        b"TRI_PROVE_V1"
    };
    let mut message = Vec::new();
    message.extend_from_slice(domain);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::depin::merkle::merkle_root;
    use crate::depin::phi_challenge::{
        compute_phi_response_v2, derive_phi_challenge, derive_phi_challenge_v2, gf16_dot4,
    };
    use crate::depin::types::{AppState, MerkleProof, ProveRequest};
    use axum::body::Body;
    use axum::routing::{get, post};
    use axum::Router;
    use ed25519_dalek::{Signer, SigningKey};
    use http_body_util::BodyExt;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tower::util::ServiceExt;

    fn build_test_app() -> Router {
        let state = Arc::new(RwLock::new(AppState::new()));
        Router::new()
            .route("/prove", post(post_prove))
            .route("/epoch-challenge", get(get_epoch_challenge))
            .route("/health", get(health_check))
            .with_state(state)
    }

    async fn call_prove(app: &Router, req: ProveRequest) -> ProveResponse {
        let body = serde_json::to_string(&req).unwrap();
        let request = axum::http::Request::builder()
            .method("POST")
            .uri("/prove")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();
        let response: axum::http::Response<axum::body::Body> =
            app.clone().oneshot(request).await.unwrap();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    fn make_valid_proof_request(epoch: u64) -> ProveRequest {
        let signing_key_bytes = [0xAA; 32];
        let signing_key = SigningKey::from_bytes(&signing_key_bytes);
        let verifying_key = signing_key.verifying_key();
        let node_id = verifying_key.to_bytes();

        let challenge = derive_phi_challenge(epoch, &node_id);
        let w: [u8; 4] = challenge[..4].try_into().unwrap();
        let x: [u8; 4] = node_id[..4].try_into().unwrap();
        let phi_response = gf16_dot4(&w, &x);

        let leaf = crate::depin::types::sha2_hash(&[&node_id, &phi_response]);
        let leaves = vec![leaf];
        let root = merkle_root(&leaves);

        let mut message = Vec::new();
        message.extend_from_slice(b"TRI_PROVE_V1");
        message.extend_from_slice(&node_id);
        message.extend_from_slice(&phi_response);
        let sig = signing_key.sign(&message);

        ProveRequest {
            node_id: hex::encode(node_id),
            epoch,
            phi_response: hex::encode(phi_response),
            merkle_proof: MerkleProof {
                root: hex::encode(root),
                leaf: hex::encode(leaf),
                siblings: vec![],
            },
            merkle_leaf_index: 0,
            peer_sample_sig: hex::encode(sig.to_bytes()),
            version: 0,
        }
    }

    #[tokio::test]
    async fn test_e2e_valid_proof() {
        let app = build_test_app();
        let req = make_valid_proof_request(0);
        let resp = call_prove(&app, req).await;
        assert!(
            resp.valid,
            "expected valid proof, got reason: {:?}",
            resp.reason
        );
        assert_eq!(resp.reward_lamports, 50_000_000);
        assert!(!resp.epoch_hash.is_empty());
        assert!(!resp.next_challenge.is_empty());
        assert_eq!(resp.tokens_count, 50_000);
        assert!(resp.reason.is_none());
    }

    #[tokio::test]
    async fn test_e2e_invalid_merkle_wrong_root() {
        let app = build_test_app();
        let mut req = make_valid_proof_request(0);
        req.merkle_proof.root = hex::encode([0xFF; 32]);
        let resp = call_prove(&app, req).await;
        assert!(!resp.valid);
        assert_eq!(resp.reason.as_deref(), Some("merkle_proof_invalid"));
    }

    #[tokio::test]
    async fn test_e2e_invalid_merkle_wrong_leaf() {
        let app = build_test_app();
        let mut req = make_valid_proof_request(0);
        req.merkle_proof.leaf = hex::encode([0xFF; 32]);
        let resp = call_prove(&app, req).await;
        assert!(!resp.valid);
        assert_eq!(resp.reason.as_deref(), Some("merkle_proof_invalid"));
    }

    #[tokio::test]
    async fn test_e2e_invalid_phi_response() {
        let app = build_test_app();
        let mut req = make_valid_proof_request(0);
        req.phi_response = hex::encode([0xFF, 0xFF, 0xFF, 0xFF]);
        let resp = call_prove(&app, req).await;
        assert!(!resp.valid);
        assert_eq!(resp.reason.as_deref(), Some("phi_challenge_mismatch"));
    }

    #[tokio::test]
    async fn test_e2e_invalid_node_id() {
        let app = build_test_app();
        let mut req = make_valid_proof_request(0);
        req.node_id = hex::encode([0u8; 16]);
        let resp = call_prove(&app, req).await;
        assert!(!resp.valid);
        assert_eq!(resp.reason.as_deref(), Some("invalid_node_id"));
    }

    #[tokio::test]
    async fn test_e2e_merkle_four_leaves() {
        let signing_key_bytes = [0xBB; 32];
        let signing_key = SigningKey::from_bytes(&signing_key_bytes);
        let verifying_key = signing_key.verifying_key();
        let node_id = verifying_key.to_bytes();
        let epoch: u64 = 0;

        let challenge = derive_phi_challenge(epoch, &node_id);
        let w: [u8; 4] = challenge[..4].try_into().unwrap();
        let x: [u8; 4] = node_id[..4].try_into().unwrap();
        let phi_response = gf16_dot4(&w, &x);

        let leaf = crate::depin::types::sha2_hash(&[&node_id, &phi_response]);
        let leaves = vec![
            crate::depin::types::sha2_hash(&[b"leaf0"]),
            crate::depin::types::sha2_hash(&[b"leaf1"]),
            leaf,
            crate::depin::types::sha2_hash(&[b"leaf3"]),
        ];
        let root = merkle_root(&leaves);
        let siblings = get_siblings(&leaves, 2);

        let mut message = Vec::new();
        message.extend_from_slice(b"TRI_PROVE_V1");
        message.extend_from_slice(&node_id);
        message.extend_from_slice(&phi_response);
        let sig = signing_key.sign(&message);

        let app = build_test_app();
        let req = ProveRequest {
            node_id: hex::encode(node_id),
            epoch,
            phi_response: hex::encode(phi_response),
            merkle_proof: MerkleProof {
                root: hex::encode(root),
                leaf: hex::encode(leaf),
                siblings: siblings.iter().map(|s| hex::encode(s)).collect(),
            },
            merkle_leaf_index: 2,
            peer_sample_sig: hex::encode(sig.to_bytes()),
            version: 0,
        };
        let resp = call_prove(&app, req).await;
        assert!(
            resp.valid,
            "expected valid proof with 4-leaf merkle tree, got reason: {:?}",
            resp.reason
        );
    }

    #[tokio::test]
    async fn test_e2e_v2_valid_proof() {
        let signing_key_bytes = [0xCC; 32];
        let signing_key = SigningKey::from_bytes(&signing_key_bytes);
        let verifying_key = signing_key.verifying_key();
        let node_id = verifying_key.to_bytes();
        let epoch: u64 = 0;

        let mut node_id_32 = [0u8; 32];
        node_id_32.copy_from_slice(&node_id);
        let challenge = derive_phi_challenge_v2(epoch, &node_id_32);
        let phi_response = compute_phi_response_v2(&challenge);

        let leaf = crate::depin::types::sha2_hash(&[&node_id, &phi_response]);
        let leaves = vec![leaf];
        let root = merkle_root(&leaves);

        let mut message = Vec::new();
        message.extend_from_slice(b"TRI_PROVE_V2");
        message.extend_from_slice(&node_id);
        message.extend_from_slice(&phi_response);
        let sig = signing_key.sign(&message);

        let app = build_test_app();
        let req = ProveRequest {
            node_id: hex::encode(node_id),
            epoch,
            phi_response: hex::encode(phi_response),
            merkle_proof: MerkleProof {
                root: hex::encode(root),
                leaf: hex::encode(leaf),
                siblings: vec![],
            },
            merkle_leaf_index: 0,
            peer_sample_sig: hex::encode(sig.to_bytes()),
            version: 2,
        };
        let resp = call_prove(&app, req).await;
        assert!(
            resp.valid,
            "expected valid V2 proof, got reason: {:?}",
            resp.reason
        );
        assert_eq!(resp.reward_lamports, 50_000_000);
        assert!(!resp.epoch_hash.is_empty());
        assert!(!resp.next_challenge.is_empty());
        assert_eq!(resp.tokens_count, 50_000);
    }

    #[tokio::test]
    async fn test_e2e_v2_wrong_response() {
        let signing_key_bytes = [0xCC; 32];
        let signing_key = SigningKey::from_bytes(&signing_key_bytes);
        let verifying_key = signing_key.verifying_key();
        let node_id = verifying_key.to_bytes();
        let epoch: u64 = 0;

        let mut message = Vec::new();
        message.extend_from_slice(b"TRI_PROVE_V2");
        message.extend_from_slice(&node_id);
        message.extend_from_slice(&[0xFF; 32]);
        let sig = signing_key.sign(&message);

        let leaf = crate::depin::types::sha2_hash(&[&node_id, &[0xFF; 32]]);
        let leaves = vec![leaf];
        let root = merkle_root(&leaves);

        let app = build_test_app();
        let req = ProveRequest {
            node_id: hex::encode(node_id),
            epoch,
            phi_response: hex::encode([0xFF; 32]),
            merkle_proof: MerkleProof {
                root: hex::encode(root),
                leaf: hex::encode(leaf),
                siblings: vec![],
            },
            merkle_leaf_index: 0,
            peer_sample_sig: hex::encode(sig.to_bytes()),
            version: 2,
        };
        let resp = call_prove(&app, req).await;
        assert!(!resp.valid);
        assert_eq!(resp.reason.as_deref(), Some("phi_challenge_mismatch"));
    }

    fn get_siblings(leaves: &[[u8; 32]], index: usize) -> Vec<[u8; 32]> {
        let mut siblings = Vec::new();
        let mut layer: Vec<[u8; 32]> = leaves.to_vec();
        let mut idx = index;
        while layer.len() > 1 {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            if sibling_idx < layer.len() {
                siblings.push(layer[sibling_idx]);
            } else {
                siblings.push(layer[idx]);
            }
            let mut next = Vec::new();
            let mut i = 0;
            while i < layer.len() {
                let left = layer[i];
                let right = if i + 1 < layer.len() {
                    layer[i + 1]
                } else {
                    left
                };
                next.push(crate::depin::merkle::hash_pair_test(&left, &right));
                i += 2;
            }
            idx /= 2;
            layer = next;
        }
        siblings
    }
}

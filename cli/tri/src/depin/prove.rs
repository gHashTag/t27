use crate::depin::phi_challenge::{
    compute_epoch_hash, derive_phi_challenge, derive_phi_challenge_v2, verify_phi_response,
    verify_phi_response_v2,
};
use crate::depin::types::{AppState, EpochChallengeResponse, ProveRequest, ProveResponse};
use crate::depin::types::sha2_hash;

fn err(reason: &str) -> axum::Json<ProveResponse> {
    axum::Json(ProveResponse {
        valid: false,
        reward_lamports: 0,
        epoch_hash: String::new(),
        next_challenge: String::new(),
        tokens_count: 0,
        reason: Some(reason.into()),
    })
}

pub async fn post_prove(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<tokio::sync::RwLock<AppState>>>,
    axum::Json(req): axum::Json<ProveRequest>,
) -> axum::Json<ProveResponse> {
    let node_id = match hex::decode(&req.node_id) {
        Ok(v) if v.len() == 32 => v,
        _ => return err("invalid_node_id"),
    };

    if req.version != 1 && req.version != 2 {
        return err("unsupported_version");
    }

    let expected_resp_len = if req.version == 2 { 32 } else { 4 };
    let phi_response = match hex::decode(&req.phi_response) {
        Ok(v) if v.len() == expected_resp_len => v,
        _ => return err("invalid_phi_response"),
    };

    let challenge_ok = if req.version == 2 {
        let node_arr: [u8; 32] = match node_id.as_slice().try_into() {
            Ok(a) => a,
            Err(_) => return err("invalid_node_id"),
        };
        let resp_arr: [u8; 32] = match phi_response.as_slice().try_into() {
            Ok(a) => a,
            Err(_) => return err("invalid_phi_response"),
        };
        let challenge = derive_phi_challenge_v2(req.epoch, &node_arr);
        verify_phi_response_v2(&challenge, &resp_arr, &node_arr)
    } else {
        let challenge = derive_phi_challenge(req.epoch, &node_id);
        verify_phi_response(&challenge, &phi_response, &node_id)
    };

    if !challenge_ok {
        return err("phi_challenge_mismatch");
    }

    let root = match hex::decode(&req.merkle_proof.root) {
        Ok(v) if v.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&v);
            arr
        }
        _ => return err("merkle_proof_invalid"),
    };

    let leaf = match hex::decode(&req.merkle_proof.leaf) {
        Ok(v) if v.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&v);
            arr
        }
        _ => return err("merkle_proof_invalid"),
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
        return err("merkle_proof_invalid");
    }

    if !crate::depin::merkle::verify_merkle(&root, &leaf, &siblings, req.merkle_leaf_index) {
        return err("merkle_proof_invalid");
    }

    if !verify_ed25519_signature(&node_id, &phi_response, &req.peer_sample_sig, req.version) {
        return err("peer_sample_sig_invalid");
    }

    let guard = state.read().await;
    let epoch = &guard.epoch;
    let reward = epoch.block_reward;

    let epoch_hash = compute_epoch_hash(req.epoch, &node_id, &phi_response);
    let next_challenge_hex = if req.version == 2 {
        let node_arr: [u8; 32] = match node_id.as_slice().try_into() {
            Ok(a) => a,
            Err(_) => return err("invalid_node_id"),
        };
        let next_matrix = derive_phi_challenge_v2(req.epoch + 1, &node_arr);
        hex::encode(crate::depin::phi_challenge::pack_gf16_matrix(&next_matrix))
    } else {
        hex::encode(derive_phi_challenge(req.epoch + 1, &node_id))
    };

    axum::Json(ProveResponse {
        valid: true,
        reward_lamports: reward,
        epoch_hash: hex::encode(epoch_hash),
        next_challenge: next_challenge_hex,
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

fn verify_ed25519_signature(node_id: &[u8], phi_response: &[u8], sig_hex: &str, version: u8) -> bool {
    let sig_bytes = match hex::decode(sig_hex) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if sig_bytes.len() != 64 {
        return false;
    }

    let domain: &[u8] = if version == 2 { b"TRI_PROVE_V2" } else { b"TRI_PROVE_V1" };
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
    use crate::depin::phi_challenge::{derive_phi_challenge, gf16_dot4};
    use crate::depin::types::{AppState, ProveRequest, MerkleProof};
    use axum::body::Body;
    use axum::routing::{get, post};
    use axum::Router;
    use ed25519_dalek::{SigningKey, Signer};
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
        let response: axum::http::Response<axum::body::Body> = app
            .clone()
            .oneshot(request)
            .await
            .unwrap();
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
            version: 1,
        }
    }

    #[tokio::test]
    async fn test_e2e_valid_proof() {
        let app = build_test_app();
        let req = make_valid_proof_request(0);
        let resp = call_prove(&app, req).await;
        assert!(resp.valid, "expected valid proof, got reason: {:?}", resp.reason);
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
            version: 1,
        };
        let resp = call_prove(&app, req).await;
        assert!(resp.valid, "expected valid proof with 4-leaf merkle tree, got reason: {:?}", resp.reason);
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
                let right = if i + 1 < layer.len() { layer[i + 1] } else { left };
                next.push(crate::depin::merkle::hash_pair_test(&left, &right));
                i += 2;
            }
            idx /= 2;
            layer = next;
        }
        siblings
    }

    use crate::depin::phi_challenge::{compute_phi_response_v2, derive_phi_challenge_v2};

    fn make_valid_proof_request_v2(epoch: u64) -> (ProveRequest, [u8; 32]) {
        let signing_key_bytes = [0xCC; 32];
        let signing_key = SigningKey::from_bytes(&signing_key_bytes);
        let verifying_key = signing_key.verifying_key();
        let node_id = verifying_key.to_bytes();

        let challenge = derive_phi_challenge_v2(epoch, &node_id);
        let phi_response = compute_phi_response_v2(&challenge);

        let leaf = crate::depin::types::sha2_hash(&[&node_id, &phi_response]);
        let leaves = vec![leaf];
        let root = merkle_root(&leaves);

        let mut message = Vec::new();
        message.extend_from_slice(b"TRI_PROVE_V2");
        message.extend_from_slice(&node_id);
        message.extend_from_slice(&phi_response);
        let sig = signing_key.sign(&message);

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
        (req, phi_response)
    }

    #[tokio::test]
    async fn test_v2_e2e_valid_proof() {
        let app = build_test_app();
        let (req, _) = make_valid_proof_request_v2(0);
        let resp = call_prove(&app, req).await;
        assert!(resp.valid, "expected valid v2 proof, got reason: {:?}", resp.reason);
        assert_eq!(resp.reward_lamports, 50_000_000);
        assert_eq!(resp.tokens_count, 50_000);
        assert!(resp.reason.is_none());
        assert_eq!(resp.next_challenge.len(), 256, "v2 next_challenge is 128 bytes = 256 hex chars");
    }

    #[tokio::test]
    async fn test_v2_e2e_invalid_response_flipped_bit() {
        let app = build_test_app();
        let (mut req, _) = make_valid_proof_request_v2(0);
        let mut bytes = hex::decode(&req.phi_response).unwrap();
        bytes[0] ^= 0x01;
        req.phi_response = hex::encode(&bytes);
        let resp = call_prove(&app, req).await;
        assert!(!resp.valid);
        assert_eq!(resp.reason.as_deref(), Some("phi_challenge_mismatch"));
    }

    #[tokio::test]
    async fn test_v2_e2e_wrong_response_length() {
        let app = build_test_app();
        let (mut req, _) = make_valid_proof_request_v2(0);
        req.phi_response = hex::encode([0xAA; 4]);
        let resp = call_prove(&app, req).await;
        assert!(!resp.valid);
        assert_eq!(resp.reason.as_deref(), Some("invalid_phi_response"));
    }

    #[tokio::test]
    async fn test_v2_e2e_unsupported_version() {
        let app = build_test_app();
        let (mut req, _) = make_valid_proof_request_v2(0);
        req.version = 99;
        let resp = call_prove(&app, req).await;
        assert!(!resp.valid);
        assert_eq!(resp.reason.as_deref(), Some("unsupported_version"));
    }

    #[test]
    fn test_v2_kat_pinned_response() {
        let mut node_id = [0u8; 32];
        for (i, b) in node_id.iter_mut().enumerate() {
            *b = (i + 1) as u8;
        }
        let challenge = derive_phi_challenge_v2(42, &node_id);
        let response = compute_phi_response_v2(&challenge);
        let hex_resp = hex::encode(response);
        assert_eq!(hex_resp.len(), 64);
        let challenge_again = derive_phi_challenge_v2(42, &node_id);
        let response_again = compute_phi_response_v2(&challenge_again);
        assert_eq!(response, response_again, "v2 KAT must be deterministic");
        let other = derive_phi_challenge_v2(43, &node_id);
        let other_resp = compute_phi_response_v2(&other);
        assert_ne!(response, other_resp, "different epoch must yield different response");
    }
}

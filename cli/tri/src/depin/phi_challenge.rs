use sha2::{Digest, Sha256};

pub fn derive_phi_challenge(epoch: u64, node_id: &[u8]) -> [u8; 16] {
    let mut h = Sha256::new();
    h.update(b"TRI_PHI_CHALLENGE_V1");
    h.update(epoch.to_le_bytes());
    h.update(node_id);
    let out = h.finalize();
    let mut challenge = [0u8; 16];
    challenge.copy_from_slice(&out[..16]);
    challenge
}

pub fn verify_phi_response(challenge: &[u8; 16], response: &[u8], node_id: &[u8]) -> bool {
    let w: [u8; 4] = match challenge[..4].try_into() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let x: [u8; 4] = match node_id.get(..4) {
        Some(s) => [s[0], s[1], s[2], s[3]],
        None => return false,
    };
    let expected = gf16_dot4(&w, &x);
    response.len() == 4 && response == expected
}

pub fn gf16_mul(a: u8, b: u8) -> u8 {
    let (mut a, mut b, mut p) = (a & 0xF, b & 0xF, 0u8);
    for _ in 0..4 {
        if b & 1 != 0 {
            p ^= a;
        }
        let carry = a & 0x8;
        a = (a << 1) & 0xF;
        if carry != 0 {
            a ^= 0x3;
        }
        b >>= 1;
    }
    p
}

pub fn gf16_dot4(w: &[u8; 4], x: &[u8; 4]) -> Vec<u8> {
    vec![gf16_mul(w[0], x[0]), gf16_mul(w[1], x[1]), gf16_mul(w[2], x[2]), gf16_mul(w[3], x[3])]
}

pub fn compute_epoch_hash(epoch: u64, node_id: &[u8], phi_response: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"EPOCH_HASH_V1");
    h.update(epoch.to_le_bytes());
    h.update(node_id);
    h.update(phi_response);
    h.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gf16_mul_identity() {
        assert_eq!(gf16_mul(1, 1), 1);
        assert_eq!(gf16_mul(0, 0xFF), 0);
        assert_eq!(gf16_mul(0xF, 1), 0xF);
    }

    #[test]
    fn test_gf16_dot4_basic() {
        let w = [1u8, 2, 3, 4];
        let x = [1u8, 1, 1, 1];
        let result = gf16_dot4(&w, &x);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_phi_challenge_deterministic() {
        let node_id = [1u8; 32];
        let c1 = derive_phi_challenge(1, &node_id);
        let c2 = derive_phi_challenge(1, &node_id);
        assert_eq!(c1, c2);
        let c3 = derive_phi_challenge(2, &node_id);
        assert_ne!(c1, c3);
    }

    #[test]
    fn test_verify_phi_response() {
        let node_id = [0xAA; 32];
        let challenge = derive_phi_challenge(1, &node_id);
        let w: [u8; 4] = challenge[..4].try_into().unwrap();
        let x: [u8; 4] = node_id[..4].try_into().unwrap();
        let response = gf16_dot4(&w, &x);
        assert!(verify_phi_response(&challenge, &response, &node_id));
    }

    #[test]
    fn test_adversarial_random_guess_fails() {
        let node_id = [0xAB; 32];
        let challenge = derive_phi_challenge(42, &node_id);
        let mut wrong = [0u8; 4];
        let mut failures = 0u32;
        for guess in 0u32..65536 {
            wrong[0] = (guess & 0xF) as u8;
            wrong[1] = ((guess >> 4) & 0xF) as u8;
            wrong[2] = ((guess >> 8) & 0xF) as u8;
            wrong[3] = ((guess >> 12) & 0xF) as u8;
            if verify_phi_response(&challenge, &wrong, &node_id) {
                failures += 1;
            }
        }
        assert_eq!(failures, 1, "exactly one valid response in 65536 guesses");
    }

    #[test]
    fn test_adversarial_wrong_epoch_fails() {
        let node_id = [0xCC; 32];
        let challenge_epoch_1 = derive_phi_challenge(1, &node_id);
        let challenge_epoch_2 = derive_phi_challenge(2, &node_id);
        let w1: [u8; 4] = challenge_epoch_1[..4].try_into().unwrap();
        let x: [u8; 4] = node_id[..4].try_into().unwrap();
        let response_for_epoch_1 = gf16_dot4(&w1, &x);
        assert!(
            !verify_phi_response(&challenge_epoch_2, &response_for_epoch_1, &node_id),
            "response for epoch 1 must fail verification for epoch 2"
        );
    }

    #[test]
    fn test_adversarial_wrong_node_fails() {
        let node_a = [0x11; 32];
        let node_b = [0x22; 32];
        let challenge = derive_phi_challenge(1, &node_a);
        let w: [u8; 4] = challenge[..4].try_into().unwrap();
        let x_a: [u8; 4] = node_a[..4].try_into().unwrap();
        let response_a = gf16_dot4(&w, &x_a);
        assert!(
            !verify_phi_response(&challenge, &response_a, &node_b),
            "response computed for node A must fail for node B"
        );
    }

    #[test]
    fn test_adversarial_preimage_resistance() {
        let node_id = [0xDD; 32];
        let c1 = derive_phi_challenge(1, &node_id);
        let c2 = derive_phi_challenge(2, &node_id);
        let c3 = derive_phi_challenge(3, &node_id);
        assert_ne!(c1[..4], c2[..4], "consecutive epochs must differ");
        assert_ne!(c2[..4], c3[..4], "consecutive epochs must differ");
        assert_ne!(c1[..4], c3[..4], "non-consecutive must also differ");
    }

    #[test]
    fn test_adversarial_commutation_nontrivial() {
        for a in 1u8..16 {
            for b in (a + 1)..16 {
                let ab = gf16_mul(a, b);
                let ba = gf16_mul(b, a);
                assert_eq!(ab, ba, "GF(2^4) mul must commute");
                if a != 1 && b != 1 {
                    assert_ne!(ab, a, "non-trivial multiplication");
                    assert_ne!(ab, b, "non-trivial multiplication");
                }
            }
        }
    }
}

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
}

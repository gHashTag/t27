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
    vec![
        gf16_mul(w[0], x[0]),
        gf16_mul(w[1], x[1]),
        gf16_mul(w[2], x[2]),
        gf16_mul(w[3], x[3]),
    ]
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

pub fn gf16_inv(a: u8) -> u8 {
    let a = a & 0xF;
    if a == 0 {
        return 0;
    }
    for x in 1u8..16 {
        if gf16_mul(a, x) == 1 {
            return x;
        }
    }
    0
}

pub fn gf16_matmul(a: &[[u8; 16]; 16], b: &[[u8; 16]; 16]) -> [[u8; 16]; 16] {
    let mut c = [[0u8; 16]; 16];
    for i in 0..16 {
        for j in 0..16 {
            let mut acc = 0u8;
            for k in 0..16 {
                acc ^= gf16_mul(a[i][k] & 0xF, b[k][j] & 0xF);
            }
            c[i][j] = acc & 0xF;
        }
    }
    c
}

pub const CHAMPION_WEIGHTS: [[u8; 16]; 16] = [
    [
        0x4, 0xF, 0xA, 0x7, 0x2, 0x8, 0x6, 0x1, 0xA, 0x2, 0x4, 0xC, 0x0, 0x6, 0x1, 0x5,
    ],
    [
        0xB, 0x7, 0x2, 0x4, 0x6, 0xA, 0x3, 0x7, 0xA, 0x3, 0xF, 0x9, 0x5, 0x1, 0xD, 0x1,
    ],
    [
        0xC, 0x7, 0x3, 0xA, 0x5, 0x2, 0x1, 0xF, 0x4, 0x2, 0x9, 0x7, 0x2, 0x9, 0x0, 0xB,
    ],
    [
        0xD, 0xE, 0x7, 0x9, 0xE, 0x2, 0x6, 0x1, 0xC, 0xF, 0x7, 0xE, 0x7, 0x6, 0x6, 0x1,
    ],
    [
        0xB, 0x7, 0x3, 0x9, 0x2, 0x4, 0xE, 0x1, 0xF, 0x5, 0x9, 0x7, 0xD, 0xB, 0x9, 0x2,
    ],
    [
        0x1, 0x5, 0x1, 0xB, 0x8, 0x2, 0x2, 0xB, 0x9, 0x9, 0x7, 0xB, 0x9, 0x9, 0x3, 0xB,
    ],
    [
        0x2, 0x1, 0xA, 0x7, 0xD, 0x1, 0x2, 0xB, 0x3, 0x7, 0x4, 0xF, 0xC, 0x7, 0x5, 0xD,
    ],
    [
        0xA, 0x8, 0xB, 0x1, 0xC, 0xA, 0x4, 0xC, 0xE, 0x5, 0x7, 0xF, 0x6, 0xA, 0xA, 0xA,
    ],
    [
        0xC, 0x9, 0x7, 0x6, 0xF, 0x4, 0x5, 0x7, 0x1, 0x2, 0xD, 0x0, 0xF, 0xE, 0x6, 0x0,
    ],
    [
        0x7, 0xE, 0xA, 0xE, 0x7, 0xB, 0x5, 0x7, 0x4, 0xC, 0xB, 0x3, 0x7, 0x4, 0xB, 0xE,
    ],
    [
        0xB, 0x8, 0x4, 0x9, 0x0, 0xE, 0x0, 0x6, 0x9, 0x5, 0x1, 0xA, 0x6, 0x5, 0x5, 0x8,
    ],
    [
        0x8, 0x2, 0xC, 0x4, 0x7, 0x6, 0x2, 0x2, 0xF, 0xA, 0xA, 0x1, 0x3, 0xD, 0x0, 0x6,
    ],
    [
        0xA, 0x4, 0x6, 0xF, 0x9, 0xC, 0x4, 0xB, 0xB, 0xD, 0x6, 0x2, 0xA, 0x5, 0x9, 0x5,
    ],
    [
        0x8, 0x6, 0xA, 0x7, 0x0, 0xC, 0x0, 0x8, 0x8, 0xF, 0x4, 0xE, 0x6, 0xA, 0x5, 0x5,
    ],
    [
        0xB, 0x5, 0x1, 0x8, 0xD, 0x8, 0x2, 0x8, 0x0, 0xE, 0xD, 0x4, 0x1, 0x0, 0x7, 0xC,
    ],
    [
        0x2, 0x3, 0xA, 0xE, 0x5, 0x5, 0xC, 0xB, 0x3, 0x8, 0x1, 0xD, 0xA, 0xA, 0x2, 0xF,
    ],
];

pub fn pack_gf16_matrix(m: &[[u8; 16]; 16]) -> [u8; 128] {
    let mut out = [0u8; 128];
    for i in 0..16 {
        for j in 0..8 {
            out[i * 8 + j] = ((m[i][j * 2] & 0xF) << 4) | (m[i][j * 2 + 1] & 0xF);
        }
    }
    out
}

pub fn derive_phi_challenge_v2(epoch: u64, node_id: &[u8; 32]) -> [[u8; 16]; 16] {
    let prefix = b"TRI_PHI_CHALLENGE_V2";
    let epoch_bytes = epoch.to_le_bytes();
    let mut matrix = [[0u8; 16]; 16];
    for i in 0u8..16 {
        let mut input = Vec::with_capacity(prefix.len() + 8 + 32 + 1);
        input.extend_from_slice(prefix);
        input.extend_from_slice(&epoch_bytes);
        input.extend_from_slice(node_id);
        input.push(i);
        let hash = Sha256::digest(&input);
        for j in 0..16 {
            matrix[i as usize][j] = (hash[j * 2] >> 4) & 0xF;
        }
    }
    matrix
}

pub fn compute_phi_response_v2(challenge: &[[u8; 16]; 16]) -> [u8; 32] {
    let product = gf16_matmul(&CHAMPION_WEIGHTS, challenge);
    let packed = pack_gf16_matrix(&product);
    Sha256::digest(&packed).into()
}

pub fn verify_phi_response_v2(
    challenge: &[[u8; 16]; 16],
    response: &[u8; 32],
    _node_id: &[u8; 32],
) -> bool {
    let expected = compute_phi_response_v2(challenge);
    expected
        .iter()
        .zip(response.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests_v2 {
    use super::*;

    #[test]
    fn test_v2_champion_weights_deterministic() {
        let prefix = b"TRI_PHI_CHAMPION_SEED_V1";
        let mut runtime = [[0u8; 16]; 16];
        for i in 0u8..16 {
            let mut data = prefix.to_vec();
            data.extend_from_slice(&(i as u64).to_le_bytes());
            let hash = Sha256::digest(&data);
            for j in 0..16 {
                runtime[i as usize][j] = (hash[j * 2] >> 4) & 0xF;
            }
        }
        assert_eq!(CHAMPION_WEIGHTS, runtime);
    }

    #[test]
    fn test_v2_champion_weights_full_rank() {
        let mut m: Vec<Vec<u8>> = CHAMPION_WEIGHTS.iter().map(|row| row.to_vec()).collect();
        for col in 0..16 {
            let pivot = (col..16)
                .find(|&r| m[r][col] != 0)
                .expect("zero pivot: matrix is singular!");
            m.swap(col, pivot);
            let inv = gf16_inv(m[col][col]);
            for j in col..16 {
                m[col][j] = gf16_mul(m[col][j], inv) & 0xF;
            }
            for r in 0..16 {
                if r != col && m[r][col] != 0 {
                    let factor = m[r][col];
                    for j in col..16 {
                        m[r][j] ^= gf16_mul(factor, m[col][j]) & 0xF;
                    }
                }
            }
        }
    }

    #[test]
    fn test_v2_deterministic() {
        let node = [0x42u8; 32];
        let c1 = derive_phi_challenge_v2(7, &node);
        let c2 = derive_phi_challenge_v2(7, &node);
        assert_eq!(c1, c2);
        let r1 = compute_phi_response_v2(&c1);
        let r2 = compute_phi_response_v2(&c2);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_v2_different_epoch_fails() {
        let node = [0x11u8; 32];
        let c0 = derive_phi_challenge_v2(0, &node);
        let c1 = derive_phi_challenge_v2(1, &node);
        let r0 = compute_phi_response_v2(&c0);
        assert!(!verify_phi_response_v2(&c1, &r0, &node));
    }

    #[test]
    fn test_v2_different_node_fails() {
        let node_a = [0xAAu8; 32];
        let node_b = [0xBBu8; 32];
        let c = derive_phi_challenge_v2(0, &node_a);
        let r = compute_phi_response_v2(&c);
        let c_b = derive_phi_challenge_v2(0, &node_b);
        assert!(!verify_phi_response_v2(&c_b, &r, &node_b));
    }

    #[test]
    fn test_v2_element_wise_spoof_fails() {
        let node = [0x55u8; 32];
        let c = derive_phi_challenge_v2(3, &node);
        let r = compute_phi_response_v2(&c);
        let mut spoof = r;
        spoof[0] ^= 0x01;
        assert!(!verify_phi_response_v2(&c, &spoof, &node));
    }

    #[test]
    fn test_v2_wrong_matrix_fails() {
        let node = [0x33u8; 32];
        let mut fake_weights = CHAMPION_WEIGHTS;
        fake_weights[0][0] ^= 0x1;
        let c = derive_phi_challenge_v2(0, &node);
        let fake_product = gf16_matmul(&fake_weights, &c);
        let packed = pack_gf16_matrix(&fake_product);
        let fake_response: [u8; 32] = Sha256::digest(&packed).into();
        assert!(!verify_phi_response_v2(&c, &fake_response, &node));
    }

    #[test]
    fn test_v2_diffusion() {
        let node = [0x77u8; 32];
        let mut c1 = derive_phi_challenge_v2(0, &node);
        let r1 = compute_phi_response_v2(&c1);
        c1[0][0] ^= 0x1;
        let r2 = compute_phi_response_v2(&c1);
        let diff_bytes = r1.iter().zip(r2.iter()).filter(|(a, b)| a != b).count();
        assert!(
            diff_bytes >= 8,
            "avalanche too weak: only {}/32 bytes changed",
            diff_bytes
        );
    }
}

use sha2::{Digest, Sha256};

pub fn merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0u8; 32];
    }
    let mut layer: Vec<[u8; 32]> = leaves.to_vec();
    while layer.len() > 1 {
        let mut next = Vec::new();
        let mut i = 0;
        while i < layer.len() {
            let left = layer[i];
            let right = if i + 1 < layer.len() {
                layer[i + 1]
            } else {
                left
            };
            next.push(hash_pair(&left, &right));
            i += 2;
        }
        layer = next;
    }
    layer[0]
}

pub fn verify_merkle(
    root: &[u8; 32],
    leaf: &[u8; 32],
    siblings: &[[u8; 32]],
    index: usize,
) -> bool {
    let mut current = *leaf;
    let mut idx = index;
    for sibling in siblings {
        if idx % 2 == 0 {
            current = hash_pair(&current, sibling);
        } else {
            current = hash_pair(sibling, &current);
        }
        idx /= 2;
    }
    current == *root
}

pub fn hash_pair_test(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    hash_pair(a, b)
}

fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(a);
    h.update(b);
    h.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_single_leaf() {
        let leaves = [[1u8; 32]];
        let root = merkle_root(&leaves);
        assert!(verify_merkle(&root, &leaves[0], &[], 0));
    }

    #[test]
    fn test_merkle_four_leaves() {
        let leaves = [
            sha2_hash(&[0u8]),
            sha2_hash(&[1u8]),
            sha2_hash(&[2u8]),
            sha2_hash(&[3u8]),
        ];
        let root = merkle_root(&leaves);
        assert!(verify_merkle(
            &root,
            &leaves[0],
            &get_siblings(&leaves, 0),
            0
        ));
        assert!(verify_merkle(
            &root,
            &leaves[3],
            &get_siblings(&leaves, 3),
            3
        ));
    }

    fn sha2_hash(input: &[u8]) -> [u8; 32] {
        sha2::Sha256::digest(input).into()
    }

    fn get_siblings(leaves: &[[u8; 32]], index: usize) -> Vec<[u8; 32]> {
        let n = leaves.len();
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
                next.push(super::hash_pair(&left, &right));
                i += 2;
            }
            idx /= 2;
            layer = next;
        }
        siblings
    }
}

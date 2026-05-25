pub struct MerkleTree;

impl MerkleTree {
    pub fn root(hashes: &[u64]) -> u64 {
        if hashes.is_empty() { return 0; }
        if hashes.len() == 1 { return hashes[0]; }
        let mut layer = hashes.to_vec();
        while layer.len() > 1 {
            let mut next = Vec::new();
            for chunk in layer.chunks(2) {
                let b = if chunk.len() == 2 { chunk[1] } else { chunk[0] };
                next.push(Self::pair_hash(chunk[0], b));
            }
            layer = next;
        }
        layer[0]
    }

    pub fn proof(hashes: &[u64], index: usize) -> Option<Vec<(u64, bool)>> {
        if index >= hashes.len() || hashes.is_empty() { return None; }
        let mut layer = hashes.to_vec();
        let mut idx = index;
        let mut proof = Vec::new();
        while layer.len() > 1 {
            let mut next = Vec::new();
            for (i, chunk) in layer.chunks(2).enumerate() {
                let b = if chunk.len() == 2 { chunk[1] } else { chunk[0] };
                next.push(Self::pair_hash(chunk[0], b));
                if i == idx / 2 {
                    let sibling = if idx % 2 == 0 { b } else { chunk[0] };
                    proof.push((sibling, idx % 2 == 1));
                }
            }
            idx /= 2;
            layer = next;
        }
        Some(proof)
    }

    pub fn verify(leaf: u64, proof: &[(u64, bool)], root: u64) -> bool {
        let mut current = leaf;
        for &(sibling, is_left) in proof {
            current = if is_left {
                Self::pair_hash(sibling, current)
            } else {
                Self::pair_hash(current, sibling)
            };
        }
        current == root
    }

    fn pair_hash(a: u64, b: u64) -> u64 {
        let mut h = a.wrapping_add(b);
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33;
        h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
        h ^= h >> 33;
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_leaf() { assert_eq!(MerkleTree::root(&[42]), 42); }

    #[test]
    fn two_leaves() {
        let r = MerkleTree::root(&[10, 20]);
        assert_ne!(r, 0);
    }

    #[test]
    fn proof_roundtrip() {
        let leaves = vec![10u64, 20, 30, 40];
        let root = MerkleTree::root(&leaves);
        for i in 0..leaves.len() {
            let proof = MerkleTree::proof(&leaves, i).unwrap();
            assert!(MerkleTree::verify(leaves[i], &proof, root));
        }
    }

    #[test]
    fn invalid_proof() {
        let leaves = vec![10u64, 20, 30, 40];
        let root = MerkleTree::root(&leaves);
        let proof = MerkleTree::proof(&leaves, 0).unwrap();
        assert!(!MerkleTree::verify(99, &proof, root));
    }

    #[test]
    fn empty() { assert_eq!(MerkleTree::root(&[]), 0); }

    #[test]
    fn proof_out_of_bounds() { assert!(MerkleTree::proof(&[1, 2, 3], 5).is_none()); }
}

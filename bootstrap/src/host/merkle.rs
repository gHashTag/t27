const FNV: u64 = 0xcbf29ce484222325;
const PRIME: u64 = 0x100000001b3;

fn hash_pair(a: u64, b: u64) -> u64 {
    let mut h = FNV;
    h ^= a; h = h.wrapping_mul(PRIME);
    h ^= b; h = h.wrapping_mul(PRIME);
    h ^= h >> 33; h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h
}

fn hash_leaf(data: &[u8]) -> u64 {
    let mut h = FNV;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(PRIME); }
    h
}

#[derive(Debug, Clone, PartialEq)]
pub enum MerkleError {
    EmptyTree,
    IndexOutOfBounds { index: usize, len: usize },
    ProofVerificationFailed,
}

impl std::fmt::Display for MerkleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MerkleError::EmptyTree => write!(f, "empty tree"),
            MerkleError::IndexOutOfBounds { index, len } =>
                write!(f, "index {index} out of bounds (len {len})"),
            MerkleError::ProofVerificationFailed => write!(f, "proof verification failed"),
        }
    }
}

impl std::error::Error for MerkleError {}

#[derive(Debug, Clone)]
pub struct ProofNode {
    pub hash: u64,
    pub is_right: bool,
}

#[derive(Debug, Clone)]
pub struct MerkleTree {
    leaves: Vec<u64>,
    layers: Vec<Vec<u64>>,
    leaf_data: Vec<Vec<u8>>,
}

impl MerkleTree {
    pub fn new() -> Self { Self { leaves: Vec::new(), layers: Vec::new(), leaf_data: Vec::new() } }

    pub fn build(&mut self, data: &[Vec<u8>]) -> u64 {
        self.leaf_data = data.to_vec();
        self.leaves = data.iter().map(|d| hash_leaf(d)).collect();
        if self.leaves.is_empty() {
            self.layers.clear();
            return 0;
        }
        let mut current = self.leaves.clone();
        self.layers = vec![self.leaves.clone()];
        while current.len() > 1 {
            let mut next = Vec::new();
            let mut i = 0;
            while i < current.len() {
                if i + 1 < current.len() {
                    next.push(hash_pair(current[i], current[i + 1]));
                } else {
                    next.push(current[i]);
                }
                i += 2;
            }
            current = next;
            self.layers.push(current.clone());
        }
        current[0]
    }

    pub fn root(&self) -> Option<u64> {
        self.layers.last().and_then(|l| l.first().copied())
    }

    pub fn proof(&self, index: usize) -> Result<(u64, Vec<ProofNode>), MerkleError> {
        if self.layers.is_empty() { return Err(MerkleError::EmptyTree); }
        if index >= self.leaves.len() { return Err(MerkleError::IndexOutOfBounds { index, len: self.leaves.len() }); }
        let mut proof_nodes = Vec::new();
        let mut idx = index;
        for layer in &self.layers[..self.layers.len() - 1] {
            let sibling = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            if sibling < layer.len() {
                proof_nodes.push(ProofNode { hash: layer[sibling], is_right: sibling % 2 == 1 });
            }
            idx /= 2;
        }
        Ok((self.leaves[index], proof_nodes))
    }

    pub fn verify(&self, leaf_hash: u64, proof: &[ProofNode], root: u64) -> bool {
        let mut current = leaf_hash;
        for node in proof {
            if node.is_right {
                current = hash_pair(current, node.hash);
            } else {
                current = hash_pair(node.hash, current);
            }
        }
        current == root
    }

    pub fn leaf_count(&self) -> usize { self.leaves.len() }
    pub fn layer_count(&self) -> usize { self.layers.len() }

    pub fn update_leaf(&mut self, index: usize, data: Vec<u8>) -> Result<u64, MerkleError> {
        if index >= self.leaves.len() { return Err(MerkleError::IndexOutOfBounds { index, len: self.leaves.len() }); }
        self.leaf_data[index] = data;
        let all_data = self.leaf_data.clone();
        self.build(&all_data);
        self.root().ok_or(MerkleError::EmptyTree)
    }
}

impl Default for MerkleTree {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tree() {
        let mt = MerkleTree::new();
        assert!(mt.root().is_none());
    }

    #[test]
    fn build_tree() {
        let mut mt = MerkleTree::new();
        let root = mt.build(&[vec![1], vec![2], vec![3], vec![4]]);
        assert_ne!(root, 0);
        assert_eq!(mt.leaf_count(), 4);
    }

    #[test]
    fn deterministic_root() {
        let mut m1 = MerkleTree::new();
        let mut m2 = MerkleTree::new();
        let data = vec![vec![1], vec![2], vec![3]];
        assert_eq!(m1.build(&data), m2.build(&data));
    }

    #[test]
    fn proof_and_verify() {
        let mut mt = MerkleTree::new();
        let root = mt.build(&[vec![10], vec![20], vec![30], vec![40]]);
        let (leaf, proof) = mt.proof(2).unwrap();
        assert!(mt.verify(leaf, &proof, root));
    }

    #[test]
    fn wrong_proof_fails() {
        let mut mt = MerkleTree::new();
        let root = mt.build(&[vec![1], vec![2], vec![3], vec![4]]);
        let (leaf, _) = mt.proof(0).unwrap();
        let bad_proof = vec![ProofNode { hash: 0, is_right: true }];
        assert!(!mt.verify(leaf, &bad_proof, root));
    }

    #[test]
    fn index_out_of_bounds() {
        let mut mt = MerkleTree::new();
        mt.build(&[vec![1], vec![2]]);
        let err = mt.proof(5).unwrap_err();
        assert!(matches!(err, MerkleError::IndexOutOfBounds { .. }));
    }

    #[test]
    fn empty_proof() {
        let mut mt = MerkleTree::new();
        let err = mt.proof(0).unwrap_err();
        assert!(matches!(err, MerkleError::EmptyTree));
    }

    #[test]
    fn update_leaf() {
        let mut mt = MerkleTree::new();
        let r1 = mt.build(&[vec![1], vec![2], vec![3]]);
        let r2 = mt.update_leaf(1, vec![99]).unwrap();
        assert_ne!(r1, r2);
    }

    #[test]
    fn single_leaf() {
        let mut mt = MerkleTree::new();
        let root = mt.build(&[vec![42]]);
        let (leaf, proof) = mt.proof(0).unwrap();
        assert!(proof.is_empty());
        assert_eq!(leaf, root);
    }

    #[test]
    fn odd_leaves() {
        let mut mt = MerkleTree::new();
        mt.build(&[vec![1], vec![2], vec![3]]);
        assert_eq!(mt.leaf_count(), 3);
        let root = mt.root().unwrap();
        assert_ne!(root, 0);
    }

    #[test]
    fn layer_count() {
        let mut mt = MerkleTree::new();
        mt.build(&[vec![1], vec![2], vec![3], vec![4]]);
        assert_eq!(mt.layer_count(), 3);
    }

    #[test]
    fn error_display() {
        assert!(MerkleError::EmptyTree.to_string().contains("empty"));
    }
}

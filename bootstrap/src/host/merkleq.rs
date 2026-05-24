use std::collections::BTreeMap;

fn fnv_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn combined_hash(a: u64, b: u64) -> u64 {
    let mut buf = [0u8; 16];
    buf[..8].copy_from_slice(&a.to_le_bytes());
    buf[8..].copy_from_slice(&b.to_le_bytes());
    fnv_hash(&buf)
}

#[derive(Debug, Clone, PartialEq)]
pub enum MqError {
    IndexOutOfRange { idx: u64, len: u64 },
    EmptyQueue,
}

impl std::fmt::Display for MqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MqError::IndexOutOfRange { idx, len } => write!(f, "index {idx} out of range (len={len})"),
            MqError::EmptyQueue => write!(f, "queue empty"),
        }
    }
}

impl std::error::Error for MqError {}

pub struct MerkleQ {
    leaves: Vec<Vec<u8>>,
    leaf_hashes: Vec<u64>,
    root_hash: u64,
    total_appended: u64,
    total_proofs: u64,
}

impl MerkleQ {
    pub fn new() -> Self { Self { leaves: Vec::new(), leaf_hashes: Vec::new(), root_hash: 0, total_appended: 0, total_proofs: 0 } }

    fn recompute_root(&mut self) {
        if self.leaf_hashes.is_empty() { self.root_hash = 0; return; }
        let mut layer: Vec<u64> = self.leaf_hashes.clone();
        while layer.len() > 1 {
            let mut next = Vec::new();
            let mut i = 0;
            while i < layer.len() {
                if i + 1 < layer.len() {
                    next.push(combined_hash(layer[i], layer[i + 1]));
                    i += 2;
                } else {
                    next.push(layer[i]);
                    i += 1;
                }
            }
            layer = next;
        }
        self.root_hash = layer[0];
    }

    pub fn append(&mut self, data: Vec<u8>) -> u64 {
        let idx = self.leaves.len() as u64;
        let h = fnv_hash(&data);
        self.leaves.push(data);
        self.leaf_hashes.push(h);
        self.recompute_root();
        self.total_appended += 1;
        idx
    }

    pub fn root_hash(&self) -> u64 { self.root_hash }

    pub fn proof(&mut self, idx: u64) -> Result<Vec<(u64, bool)>, MqError> {
        if idx >= self.leaves.len() as u64 { return Err(MqError::IndexOutOfRange { idx, len: self.leaves.len() as u64 }); }
        self.total_proofs += 1;
        let mut siblings = Vec::new();
        let mut layer: Vec<u64> = self.leaf_hashes.clone();
        let mut pos = idx as usize;
        while layer.len() > 1 {
            let mut next = Vec::new();
            let mut i = 0;
            while i < layer.len() {
                if i + 1 < layer.len() {
                    if i == pos { siblings.push((layer[i + 1], false)); }
                    else if i + 1 == pos { siblings.push((layer[i], true)); }
                    next.push(combined_hash(layer[i], layer[i + 1]));
                    i += 2;
                } else {
                    next.push(layer[i]);
                    i += 1;
                }
            }
            pos /= 2;
            layer = next;
        }
        Ok(siblings)
    }

    pub fn verify(&self, idx: u64, leaf_data: &[u8], proof: &[(u64, bool)]) -> bool {
        let mut h = fnv_hash(leaf_data);
        for (sibling, is_left) in proof {
            h = if *is_left { combined_hash(*sibling, h) } else { combined_hash(h, *sibling) };
        }
        h == self.root_hash
    }

    pub fn get(&self, idx: u64) -> Option<&[u8]> { self.leaves.get(idx as usize).map(|v| v.as_slice()) }
    pub fn len(&self) -> usize { self.leaves.len() }
    pub fn is_empty(&self) -> bool { self.leaves.is_empty() }
    pub fn total_appended(&self) -> u64 { self.total_appended }
    pub fn total_proofs(&self) -> u64 { self.total_proofs }
}

impl Default for MerkleQ {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue() { assert!(MerkleQ::new().is_empty()); }

    #[test]
    fn append_root_changes() {
        let mut q = MerkleQ::new();
        let r0 = q.root_hash();
        q.append(b"a".to_vec());
        assert_ne!(q.root_hash(), r0);
    }

    #[test]
    fn proof_verify() {
        let mut q = MerkleQ::new();
        q.append(b"leaf0".to_vec());
        q.append(b"leaf1".to_vec());
        q.append(b"leaf2".to_vec());
        let proof = q.proof(1).unwrap();
        assert!(q.verify(1, b"leaf1", &proof));
    }

    #[test]
    fn verify_wrong_data() {
        let mut q = MerkleQ::new();
        q.append(b"leaf0".to_vec());
        q.append(b"leaf1".to_vec());
        let proof = q.proof(0).unwrap();
        assert!(!q.verify(0, b"wrong", &proof));
    }

    #[test]
    fn single_element() {
        let mut q = MerkleQ::new();
        q.append(b"only".to_vec());
        let proof = q.proof(0).unwrap();
        assert!(proof.is_empty());
        assert!(q.verify(0, b"only", &proof));
    }

    #[test]
    fn index_out_of_range() {
        let mut q = MerkleQ::new();
        q.append(b"x".to_vec());
        let err = q.proof(5).unwrap_err();
        assert!(matches!(err, MqError::IndexOutOfRange { .. }));
    }

    #[test]
    fn get() {
        let mut q = MerkleQ::new();
        q.append(b"data".to_vec());
        assert_eq!(q.get(0), Some(b"data".as_slice()));
        assert_eq!(q.get(1), None);
    }

    #[test]
    fn many_elements() {
        let mut q = MerkleQ::new();
        for i in 0..16 { q.append(vec![i as u8]); }
        for i in 0..16 {
            let proof = q.proof(i).unwrap();
            assert!(q.verify(i, &[i as u8], &proof));
        }
    }

    #[test]
    fn deterministic_root() {
        let mut q1 = MerkleQ::new();
        let mut q2 = MerkleQ::new();
        for i in 0..8 {
            q1.append(vec![i]);
            q2.append(vec![i]);
        }
        assert_eq!(q1.root_hash(), q2.root_hash());
    }

    #[test]
    fn stats() {
        let mut q = MerkleQ::new();
        q.append(b"x".to_vec());
        q.proof(0).unwrap();
        assert_eq!(q.total_appended(), 1);
        assert_eq!(q.total_proofs(), 1);
    }

    #[test]
    fn error_display() { assert!(MqError::EmptyQueue.to_string().contains("empty")); }
}

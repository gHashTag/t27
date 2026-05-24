use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum HaError {
    NotFound { key: u64 },
}

impl std::fmt::Display for HaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HaError::NotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for HaError {}

const BITS: usize = 5;
const MASK: usize = (1 << BITS) - 1;
const MAX_DEPTH: usize = 64 / BITS;

fn fnv_hash(key: u64) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in key.to_le_bytes() { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

fn index_at(hash: u64, depth: usize) -> usize { ((hash >> (depth * BITS)) as usize) & MASK }

#[derive(Clone)]
enum HNode {
    Leaf { key: u64, value: Vec<u8> },
    Internal { bitmap: u32, children: Vec<HNode> },
}

pub struct HashArray {
    root: HNode,
    total_inserts: u64,
    total_lookups: u64,
    total_nodes: u64,
}

impl HashArray {
    pub fn new() -> Self { Self { root: HNode::Internal { bitmap: 0, children: Vec::new() }, total_inserts: 0, total_lookups: 0, total_nodes: 1 } }

    fn bit_pos(idx: usize) -> u32 { 1u32 << idx }

    fn insert_node(node: HNode, key: u64, value: Vec<u8>, depth: usize) -> (HNode, bool) {
        let hash = fnv_hash(key);
        let idx = index_at(hash, depth);
        match node {
            HNode::Internal { bitmap, mut children } => {
                let bit = Self::bit_pos(idx);
                if bitmap & bit == 0 {
                    let pos = (bitmap & (bit - 1)).count_ones() as usize;
                    children.insert(pos, HNode::Leaf { key, value });
                    (HNode::Internal { bitmap: bitmap | bit, children }, true)
                } else {
                    let pos = (bitmap & (bit - 1)).count_ones() as usize;
                    let child = children.remove(pos);
                    let (new_child, inserted) = Self::insert_node(child, key, value, depth + 1);
                    children.insert(pos, new_child);
                    (HNode::Internal { bitmap, children }, inserted)
                }
            }
            HNode::Leaf { key: lk, value: lv } => {
                if lk == key {
                    (HNode::Leaf { key, value }, false)
                } else {
                    let mut new_node = HNode::Internal { bitmap: 0, children: Vec::new() };
                    let lk_hash = fnv_hash(lk);
                    let lk_idx = index_at(lk_hash, depth);
                    new_node = match new_node {
                        HNode::Internal { bitmap, mut children } => {
                            let bit = Self::bit_pos(lk_idx);
                            let pos = 0;
                            children.insert(pos, HNode::Leaf { key: lk, value: lv });
                            HNode::Internal { bitmap: bitmap | bit, children }
                        }
                        _ => unreachable!(),
                    };
                    let (result, _) = Self::insert_node(new_node, key, value, depth);
                    (result, true)
                }
            }
        }
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        let (new_root, _) = Self::insert_node(self.root.clone(), key, value, 0);
        self.root = new_root;
    }

    fn get_node(node: &HNode, key: u64, depth: usize) -> Option<&[u8]> {
        let hash = fnv_hash(key);
        let idx = index_at(hash, depth);
        match node {
            HNode::Internal { bitmap, children } => {
                let bit = Self::bit_pos(idx);
                if bitmap & bit == 0 { return None; }
                let pos = (bitmap & (bit - 1)).count_ones() as usize;
                Self::get_node(&children[pos], key, depth + 1)
            }
            HNode::Leaf { key: lk, value } => {
                if *lk == key { Some(value) } else { None }
            }
        }
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        Self::get_node(&self.root, key, 0)
    }

    fn count_leaves(node: &HNode) -> usize {
        match node {
            HNode::Leaf { .. } => 1,
            HNode::Internal { children, .. } => children.iter().map(Self::count_leaves).sum(),
        }
    }

    pub fn len(&self) -> usize { Self::count_leaves(&self.root) }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

impl Default for HashArray {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ha() { assert!(HashArray::new().is_empty()); }

    #[test]
    fn insert_get() {
        let mut ha = HashArray::new();
        ha.insert(1, b"one".to_vec()); ha.insert(2, b"two".to_vec());
        assert_eq!(ha.get(1), Some(&b"one"[..]));
        assert_eq!(ha.get(2), Some(&b"two"[..]));
        assert_eq!(ha.get(3), None);
    }

    #[test]
    fn overwrite() {
        let mut ha = HashArray::new();
        ha.insert(1, b"old".to_vec()); ha.insert(1, b"new".to_vec());
        assert_eq!(ha.get(1), Some(&b"new"[..]));
        assert_eq!(ha.len(), 1);
    }

    #[test]
    fn many() {
        let mut ha = HashArray::new();
        for i in 0..200u64 { ha.insert(i, vec![i as u8]); }
        assert_eq!(ha.len(), 200);
        for i in 0..200u64 { assert_eq!(ha.get(i), Some(&[i as u8][..])); }
    }

    #[test]
    fn collisions() {
        let mut ha = HashArray::new();
        ha.insert(1, b"a".to_vec()); ha.insert(2, b"b".to_vec());
        assert_eq!(ha.get(1), Some(&b"a"[..]));
        assert_eq!(ha.get(2), Some(&b"b"[..]));
    }

    #[test]
    fn stats() {
        let mut ha = HashArray::new();
        ha.insert(1, vec![]); ha.get(1);
        assert_eq!(ha.total_inserts(), 1);
        assert_eq!(ha.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(HaError::NotFound { key: 1 }.to_string().contains("not found")); }
}

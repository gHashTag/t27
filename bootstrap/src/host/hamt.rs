use std::collections::BTreeMap;

const BITS: usize = 4;
const MASK: usize = 0xF;

pub struct Hamt {
    root: BTreeMap<usize, HamtNode>,
    total_inserts: u64,
    total_lookups: u64,
}

enum HamtNode {
    Leaf(u64, Vec<u8>),
    Inner(BTreeMap<usize, Box<HamtNode>>),
}

impl Hamt {
    pub fn new() -> Self { Self { root: BTreeMap::new(), total_inserts: 0, total_lookups: 0 } }

    fn hash(key: u64) -> u64 {
        let mut h = key.wrapping_mul(0x9e3779b97f4a7c15);
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33;
        h
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        let h = Self::hash(key);
        let idx0 = (h as usize) & MASK;
        match self.root.get_mut(&idx0) {
            None => { self.root.insert(idx0, HamtNode::Leaf(key, value)); }
            Some(node) => Self::insert_rec(node, key, value, h, BITS),
        }
    }

    fn insert_rec(node: &mut HamtNode, key: u64, value: Vec<u8>, hash: u64, shift: usize) {
        if shift >= 64 { return; }
        match node {
            HamtNode::Leaf(k, v) => {
                if *k == key { *v = value; return; }
                let old_key = *k;
                let old_val = std::mem::take(v);
                let old_hash = Self::hash(old_key);
                let old_idx = ((old_hash >> shift) as usize) & MASK;
                let new_idx = ((hash >> shift) as usize) & MASK;
                let mut inner = BTreeMap::new();
                if old_idx == new_idx {
                    let mut nested = Box::new(HamtNode::Leaf(old_key, old_val));
                    Self::insert_rec(&mut nested, key, value, hash, shift + BITS);
                    inner.insert(old_idx, nested);
                } else {
                    inner.insert(old_idx, Box::new(HamtNode::Leaf(old_key, old_val)));
                    inner.insert(new_idx, Box::new(HamtNode::Leaf(key, value)));
                }
                *node = HamtNode::Inner(inner);
            }
            HamtNode::Inner(map) => {
                let idx = ((hash >> shift) as usize) & MASK;
                match map.get_mut(&idx) {
                    Some(child) => Self::insert_rec(child, key, value, hash, shift + BITS),
                    None => { map.insert(idx, Box::new(HamtNode::Leaf(key, value))); }
                }
            }
        }
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        let h = Self::hash(key);
        let idx0 = (h as usize) & MASK;
        let node = self.root.get(&idx0)?;
        Self::get_rec(node, key, h, BITS)
    }

    fn get_rec<'a>(node: &'a HamtNode, key: u64, hash: u64, shift: usize) -> Option<&'a [u8]> {
        if shift >= 64 { return None; }
        match node {
            HamtNode::Leaf(k, v) => { if *k == key { Some(v.as_slice()) } else { None } }
            HamtNode::Inner(map) => {
                let idx = ((hash >> shift) as usize) & MASK;
                let child = map.get(&idx)?;
                Self::get_rec(child, key, hash, shift + BITS)
            }
        }
    }

    pub fn len(&self) -> usize {
        let mut count = 0;
        for node in self.root.values() { Self::count_rec(node, &mut count); }
        count
    }

    fn count_rec(node: &HamtNode, count: &mut usize) {
        match node {
            HamtNode::Leaf(_, _) => *count += 1,
            HamtNode::Inner(map) => for child in map.values() { Self::count_rec(child, count); }
        }
    }

    pub fn is_empty(&self) -> bool { self.root.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut h = Hamt::new();
        h.insert(5, b"five".to_vec());
        assert_eq!(h.get(5), Some(&b"five"[..]));
    }

    #[test]
    fn missing() { assert!(Hamt::new().get(1).is_none()); }

    #[test]
    fn many() {
        let mut h = Hamt::new();
        for i in 0..100u64 { h.insert(i, vec![i as u8]); }
        for i in 0..100u64 { assert!(h.get(i).is_some()); }
        assert_eq!(h.len(), 100);
    }

    #[test]
    fn overwrite() {
        let mut h = Hamt::new();
        h.insert(1, b"old".to_vec()); h.insert(1, b"new".to_vec());
        assert_eq!(h.get(1), Some(&b"new"[..]));
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn collisions() {
        let mut h = Hamt::new();
        for i in 0..20u64 { h.insert(i * 16, vec![]); }
        assert_eq!(h.len(), 20);
    }

    #[test]
    fn stats() {
        let mut h = Hamt::new();
        h.insert(1, vec![]); h.get(1);
        assert_eq!(h.total_inserts(), 1);
        assert_eq!(h.total_lookups(), 1);
    }
}

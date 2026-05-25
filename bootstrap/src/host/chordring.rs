use std::collections::BTreeMap;

const RING_BITS: u32 = 64;

fn ring_dist(a: u64, b: u64) -> u64 {
    if b >= a { b - a } else { u64::MAX - a + b + 1 }
}

fn in_interval(start: u64, end: u64, key: u64, include_end: bool) -> bool {
    if start < end {
        if include_end { key > start && key <= end } else { key > start && key < end }
    } else {
        if include_end { key > start || key <= end } else { key > start || key < end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChordErr {
    NodeExists { id: u64 },
    NotFound { id: u64 },
}

impl std::fmt::Display for ChordErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChordErr::NodeExists { id } => write!(f, "node {id} exists"),
            ChordErr::NotFound { id } => write!(f, "node {id} not found"),
        }
    }
}

impl std::error::Error for ChordErr {}

#[derive(Clone)]
struct Node {
    id: u64,
    data: BTreeMap<Vec<u8>, Vec<u8>>,
    fingers: [u64; RING_BITS as usize],
    successor: u64,
    predecessor: Option<u64>,
}

pub struct ChordRing {
    nodes: BTreeMap<u64, Node>,
    total_lookups: u64,
    total_joins: u64,
}

impl ChordRing {
    pub fn new() -> Self { Self { nodes: BTreeMap::new(), total_lookups: 0, total_joins: 0 } }

    pub fn join(&mut self, id: u64) -> Result<(), ChordErr> {
        self.total_joins += 1;
        if self.nodes.contains_key(&id) { return Err(ChordErr::NodeExists { id }); }
        let mut fingers = [0u64; RING_BITS as usize];
        for i in 0..RING_BITS as usize {
            let target = id.wrapping_add(1u64 << i);
            fingers[i] = self.find_successor_raw(target).unwrap_or(id);
        }
        let successor = fingers[0];
        let pred = if self.nodes.is_empty() { None } else { self.find_predecessor(successor) };
        let node = Node { id, data: BTreeMap::new(), fingers, successor, predecessor: pred };
        self.nodes.insert(id, node);
        if let Some(succ_id) = self.nodes.range(id..).next().map(|(k, _)| *k) {
            if let Some(succ) = self.nodes.get_mut(&succ_id) { succ.predecessor = Some(id); }
        }
        Ok(())
    }

    fn find_successor_raw(&self, key: u64) -> Option<u64> {
        if self.nodes.is_empty() { return None; }
        let first = *self.nodes.keys().next().unwrap();
        let mut best = first;
        let mut best_dist = ring_dist(first, key);
        for &nid in self.nodes.keys() {
            let d = ring_dist(nid, key);
            if d < best_dist { best = nid; best_dist = d; }
        }
        Some(best)
    }

    fn find_predecessor(&self, id: u64) -> Option<u64> {
        let keys: Vec<u64> = self.nodes.keys().copied().collect();
        if keys.is_empty() { return None; }
        let pos = keys.iter().position(|&k| k >= id).unwrap_or(0);
        if pos == 0 { keys.last().copied() } else { Some(keys[pos - 1]) }
    }

    pub fn lookup(&mut self, key: &[u8]) -> Option<(u64, &[u8])> {
        self.total_lookups += 1;
        let h = Self::hash_key(key);
        let node_id = self.find_successor_raw(h)?;
        let node = self.nodes.get(&node_id)?;
        node.data.get(key).map(|v| (node_id, v.as_slice()))
    }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Option<u64> {
        let h = Self::hash_key(&key);
        let node_id = self.find_successor_raw(h)?;
        let node = self.nodes.get_mut(&node_id)?;
        node.data.insert(key, value);
        Some(node_id)
    }

    fn hash_key(key: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in key { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
        h
    }

    pub fn fingers(&self, id: u64) -> Option<[u64; RING_BITS as usize]> { self.nodes.get(&id).map(|n| n.fingers) }
    pub fn successor(&self, id: u64) -> Option<u64> { self.nodes.get(&id).map(|n| n.successor) }
    pub fn predecessor(&self, id: u64) -> Option<u64> { self.nodes.get(&id).and_then(|n| n.predecessor) }
    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_joins(&self) -> u64 { self.total_joins }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_lookup() {
        let mut ring = ChordRing::new();
        ring.join(100).unwrap();
        ring.join(500).unwrap();
        ring.join(1000).unwrap();
        assert_eq!(ring.node_count(), 3);
    }

    #[test]
    fn put_lookup() {
        let mut ring = ChordRing::new();
        ring.join(100).unwrap();
        ring.put(b"hello".to_vec(), b"world".to_vec());
        let (node, val) = ring.lookup(b"hello").unwrap();
        assert_eq!(val, b"world");
    }

    #[test]
    fn duplicate_join() {
        let mut ring = ChordRing::new();
        ring.join(100).unwrap();
        assert!(ring.join(100).is_err());
    }

    #[test]
    fn finger_table() {
        let mut ring = ChordRing::new();
        ring.join(100).unwrap();
        let f = ring.fingers(100).unwrap();
        assert_eq!(f[0], 100);
    }

    #[test]
    fn many_nodes() {
        let mut ring = ChordRing::new();
        for i in 0..20u64 { ring.join(i * 50).unwrap(); }
        ring.put(b"k".to_vec(), b"v".to_vec());
        assert!(ring.lookup(b"k").is_some());
    }

    #[test]
    fn single_node() {
        let mut ring = ChordRing::new();
        ring.join(42).unwrap();
        ring.put(b"x".to_vec(), b"y".to_vec());
        let (_, v) = ring.lookup(b"x").unwrap();
        assert_eq!(v, b"y");
    }

    #[test]
    fn no_nodes_lookup() { assert!(ChordRing::new().lookup(b"x").is_none()); }

    #[test]
    fn stats() {
        let mut ring = ChordRing::new();
        ring.join(1).unwrap();
        ring.put(b"a".to_vec(), b"b".to_vec());
        ring.lookup(b"a");
        assert_eq!(ring.total_joins(), 1);
        assert_eq!(ring.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(ChordErr::NodeExists { id: 5 }.to_string().contains("exists")); }
}

use std::collections::BTreeMap;

fn fnv_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

#[derive(Debug, Clone, PartialEq)]
pub enum RhError {
    NodeNotFound { node: u64 },
    RingEmpty,
}

impl std::fmt::Display for RhError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RhError::NodeNotFound { node } => write!(f, "node {node} not found"),
            RhError::RingEmpty => write!(f, "ring empty"),
        }
    }
}

impl std::error::Error for RhError {}

pub struct RingHash {
    ring: BTreeMap<u64, u64>,
    vnodes: usize,
    nodes: BTreeMap<u64, usize>,
    total_lookups: u64,
    total_adds: u64,
    total_removes: u64,
}

impl RingHash {
    pub fn new(vnodes: usize) -> Self { Self { ring: BTreeMap::new(), vnodes, nodes: BTreeMap::new(), total_lookups: 0, total_adds: 0, total_removes: 0 } }

    pub fn add_node(&mut self, node: u64) {
        if self.nodes.contains_key(&node) { return; }
        for i in 0..self.vnodes {
            let hash = fnv_hash(&format!("{node}:{i}").as_bytes());
            self.ring.insert(hash, node);
        }
        self.nodes.insert(node, self.vnodes);
        self.total_adds += 1;
    }

    pub fn remove_node(&mut self, node: u64) -> Result<(), RhError> {
        if !self.nodes.contains_key(&node) { return Err(RhError::NodeNotFound { node }); }
        for i in 0..self.vnodes {
            let hash = fnv_hash(&format!("{node}:{i}").as_bytes());
            self.ring.remove(&hash);
        }
        self.nodes.remove(&node);
        self.total_removes += 1;
        Ok(())
    }

    pub fn lookup(&mut self, key: u64) -> Result<u64, RhError> {
        self.total_lookups += 1;
        if self.ring.is_empty() { return Err(RhError::RingEmpty); }
        let hash = fnv_hash(&key.to_le_bytes());
        match self.ring.range(hash..).next() {
            Some((_, &node)) => Ok(node),
            None => Ok(*self.ring.values().next().unwrap()),
        }
    }

    pub fn lookup_n(&mut self, key: u64, n: usize) -> Result<Vec<u64>, RhError> {
        if self.ring.is_empty() { return Err(RhError::RingEmpty); }
        let hash = fnv_hash(&key.to_le_bytes());
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let start = self.ring.range(hash..);
        let wrap = self.ring.range(..);
        for (_, &node) in start.chain(wrap) {
            if seen.insert(node) { result.push(node); }
            if result.len() >= n { break; }
        }
        Ok(result)
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn vnode_count(&self) -> usize { self.ring.len() }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_adds(&self) -> u64 { self.total_adds }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn contains_node(&self, node: u64) -> bool { self.nodes.contains_key(&node) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ring() { let r = RingHash::new(100); assert_eq!(r.node_count(), 0); }

    #[test]
    fn add_lookup() {
        let mut r = RingHash::new(100);
        r.add_node(1); r.add_node(2); r.add_node(3);
        let node = r.lookup(42).unwrap();
        assert!(node == 1 || node == 2 || node == 3);
    }

    #[test]
    fn remove() {
        let mut r = RingHash::new(50);
        r.add_node(1); r.add_node(2);
        r.remove_node(1).unwrap();
        assert_eq!(r.node_count(), 1);
        assert_eq!(r.lookup(42).unwrap(), 2);
    }

    #[test]
    fn remove_not_found() {
        let mut r = RingHash::new(10);
        let err = r.remove_node(99).unwrap_err();
        assert!(matches!(err, RhError::NodeNotFound { .. }));
    }

    #[test]
    fn empty_lookup() {
        let mut r = RingHash::new(10);
        let err = r.lookup(1).unwrap_err();
        assert!(matches!(err, RhError::RingEmpty));
    }

    #[test]
    fn lookup_n() {
        let mut r = RingHash::new(100);
        r.add_node(1); r.add_node(2); r.add_node(3);
        let nodes = r.lookup_n(42, 2).unwrap();
        assert_eq!(nodes.len(), 2);
        assert_ne!(nodes[0], nodes[1]);
    }

    #[test]
    fn distribution() {
        let mut r = RingHash::new(150);
        for n in 1..=5 { r.add_node(n); }
        let mut counts = std::collections::BTreeMap::new();
        for k in 0..5000u64 {
            *counts.entry(r.lookup(k).unwrap()).or_insert(0) += 1;
        }
        assert_eq!(counts.len(), 5);
        for &c in counts.values() { assert!(c > 500, "node got {c} keys, expected >500"); }
    }

    #[test]
    fn duplicate_add() {
        let mut r = RingHash::new(50);
        r.add_node(1); r.add_node(1);
        assert_eq!(r.node_count(), 1);
    }

    #[test]
    fn stats() {
        let mut r = RingHash::new(10);
        r.add_node(1); r.lookup(42).unwrap();
        assert_eq!(r.total_adds(), 1);
        assert_eq!(r.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(RhError::RingEmpty.to_string().contains("empty")); }
}

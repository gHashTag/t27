use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RingHashError {
    NodeExists { id: u64 },
    NodeNotFound { id: u64 },
    EmptyRing,
}

impl std::fmt::Display for RingHashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RingHashError::NodeExists { id } => write!(f, "node {id} exists"),
            RingHashError::NodeNotFound { id } => write!(f, "node {id} not found"),
            RingHashError::EmptyRing => write!(f, "ring empty"),
        }
    }
}

impl std::error::Error for RingHashError {}

fn hash_key(key: u64) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in key.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

struct Node {
    id: u64,
    weight: u32,
    virtual_count: u32,
}

pub struct RingHash {
    nodes: BTreeMap<u64, Node>,
    ring: BTreeMap<u64, u64>,
    vnodes_per_weight: u32,
    total_assignments: u64,
    total_rebalances: u64,
}

impl RingHash {
    pub fn new(vnodes_per_weight: u32) -> Self {
        Self { nodes: BTreeMap::new(), ring: BTreeMap::new(), vnodes_per_weight, total_assignments: 0, total_rebalances: 0 }
    }

    pub fn add(&mut self, id: u64, weight: u32) -> Result<(), RingHashError> {
        if self.nodes.contains_key(&id) { return Err(RingHashError::NodeExists { id }); }
        let vcount = weight * self.vnodes_per_weight;
        self.nodes.insert(id, Node { id, weight, virtual_count: vcount });
        for i in 0..vcount {
            let vhash = hash_key(id.wrapping_add((i as u64) * 1_000_003));
            self.ring.insert(vhash, id);
        }
        if self.nodes.len() > 1 { self.total_rebalances += 1; }
        Ok(())
    }

    pub fn remove(&mut self, id: u64) -> Result<(), RingHashError> {
        let node = self.nodes.remove(&id).ok_or(RingHashError::NodeNotFound { id })?;
        for i in 0..node.virtual_count {
            let vhash = hash_key(id.wrapping_add((i as u64) * 1_000_003));
            self.ring.remove(&vhash);
        }
        if !self.nodes.is_empty() { self.total_rebalances += 1; }
        Ok(())
    }

    pub fn assign(&mut self, key: u64) -> Result<u64, RingHashError> {
        if self.ring.is_empty() { return Err(RingHashError::EmptyRing); }
        let h = hash_key(key);
        self.total_assignments += 1;
        if let Some((&_, &node_id)) = self.ring.range(h..).next() {
            Ok(node_id)
        } else {
            Ok(*self.ring.values().next().unwrap())
        }
    }

    pub fn distribution(&mut self, keys: &[u64]) -> BTreeMap<u64, usize> {
        let mut dist = BTreeMap::new();
        for &k in keys {
            if let Ok(node) = self.assign(k) {
                *dist.entry(node).or_insert(0) += 1;
            }
        }
        dist
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn vnode_count(&self) -> usize { self.ring.len() }
    pub fn total_assignments(&self) -> u64 { self.total_assignments }
    pub fn total_rebalances(&self) -> u64 { self.total_rebalances }
}

impl Default for RingHash {
    fn default() -> Self { Self::new(100) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ring() { assert_eq!(RingHash::new(100).node_count(), 0); }

    #[test]
    fn add_assign() {
        let mut r = RingHash::new(10);
        r.add(1, 1).unwrap();
        let node = r.assign(42).unwrap();
        assert_eq!(node, 1);
    }

    #[test]
    fn multiple_nodes() {
        let mut r = RingHash::new(100);
        r.add(1, 1).unwrap();
        r.add(2, 1).unwrap();
        r.add(3, 1).unwrap();
        let node = r.assign(42).unwrap();
        assert!(node == 1 || node == 2 || node == 3);
    }

    #[test]
    fn distribution_spreads() {
        let mut r = RingHash::new(100);
        r.add(1, 1).unwrap(); r.add(2, 1).unwrap(); r.add(3, 1).unwrap();
        let keys: Vec<u64> = (0..1000).collect();
        let dist = r.distribution(&keys);
        assert!(dist.len() >= 2);
    }

    #[test]
    fn remove_node() {
        let mut r = RingHash::new(10);
        r.add(1, 1).unwrap(); r.add(2, 1).unwrap();
        r.remove(1).unwrap();
        assert_eq!(r.node_count(), 1);
        assert_eq!(r.assign(42).unwrap(), 2);
    }

    #[test]
    fn duplicate() {
        let mut r = RingHash::new(10);
        r.add(1, 1).unwrap();
        let err = r.add(1, 1).unwrap_err();
        assert!(matches!(err, RingHashError::NodeExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut r = RingHash::new(10);
        let err = r.remove(99).unwrap_err();
        assert!(matches!(err, RingHashError::NodeNotFound { .. }));
    }

    #[test]
    fn empty_assign() {
        let mut r = RingHash::new(10);
        let err = r.assign(1).unwrap_err();
        assert!(matches!(err, RingHashError::EmptyRing));
    }

    #[test]
    fn vnode_count() {
        let mut r = RingHash::new(10);
        r.add(1, 2).unwrap();
        assert_eq!(r.vnode_count(), 20);
    }

    #[test]
    fn stats() {
        let mut r = RingHash::new(10);
        r.add(1, 1).unwrap(); r.add(2, 1).unwrap();
        r.assign(42);
        assert_eq!(r.total_assignments(), 1);
        assert!(r.total_rebalances() >= 1);
    }

    #[test]
    fn error_display() { assert!(RingHashError::EmptyRing.to_string().contains("empty")); }
}

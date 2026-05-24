use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RingHashError {
    EmptyRing,
    NodeExists { id: u64 },
    NodeNotFound { id: u64 },
}

impl std::fmt::Display for RingHashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RingHashError::EmptyRing => write!(f, "ring is empty"),
            RingHashError::NodeExists { id } => write!(f, "node {id} exists"),
            RingHashError::NodeNotFound { id } => write!(f, "node {id} not found"),
        }
    }
}

impl std::error::Error for RingHashError {}

#[derive(Debug, Clone)]
struct VirtualNode {
    node_id: u64,
    token: u64,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: u64,
    pub vnodes: usize,
    pub weight: u32,
}

#[derive(Debug, Clone)]
pub struct RingHasher {
    ring: BTreeMap<u64, VirtualNode>,
    nodes: BTreeMap<u64, NodeInfo>,
    vnodes_per_node: usize,
}

impl RingHasher {
    pub fn new(vnodes_per_node: usize) -> Self {
        Self {
            ring: BTreeMap::new(),
            nodes: BTreeMap::new(),
            vnodes_per_node: vnodes_per_node.max(1),
        }
    }

    fn hash_vnode(node_id: u64, replica: usize) -> u64 {
        let mut h: u64 = node_id.wrapping_mul(0x5851F42D4C957F2D);
        h = h.wrapping_add(replica as u64);
        h ^= h >> 33;
        h = h.wrapping_mul(0xFF51AFD7ED558CCD);
        h ^= h >> 33;
        h = h.wrapping_mul(0xC4CEB9FE1A85EC53);
        h ^= h >> 33;
        h
    }

    fn hash_key(key: u64) -> u64 {
        let mut h = key;
        h ^= h >> 33;
        h = h.wrapping_mul(0xFF51AFD7ED558CCD);
        h ^= h >> 33;
        h = h.wrapping_mul(0xC4CEB9FE1A85EC53);
        h ^= h >> 33;
        h
    }

    pub fn add_node(&mut self, id: u64, weight: u32) -> Result<(), RingHashError> {
        if self.nodes.contains_key(&id) {
            return Err(RingHashError::NodeExists { id });
        }
        let vnodes = self.vnodes_per_node;
        for i in 0..vnodes {
            let token = Self::hash_vnode(id, i);
            self.ring.insert(token, VirtualNode { node_id: id, token });
        }
        self.nodes.insert(id, NodeInfo { id, vnodes, weight });
        Ok(())
    }

    pub fn remove_node(&mut self, id: u64) -> Result<(), RingHashError> {
        if !self.nodes.contains_key(&id) {
            return Err(RingHashError::NodeNotFound { id });
        }
        for i in 0..self.vnodes_per_node {
            let token = Self::hash_vnode(id, i);
            self.ring.remove(&token);
        }
        self.nodes.remove(&id);
        Ok(())
    }

    pub fn lookup(&self, key: u64) -> Result<u64, RingHashError> {
        if self.ring.is_empty() {
            return Err(RingHashError::EmptyRing);
        }
        let token = Self::hash_key(key);
        if let Some((&_, vn)) = self.ring.range(token..).next() {
            return Ok(vn.node_id);
        }
        Ok(self.ring.iter().next().map(|(_, vn)| vn.node_id).unwrap())
    }

    pub fn lookup_n(&self, key: u64, n: usize) -> Result<Vec<u64>, RingHashError> {
        if self.ring.is_empty() {
            return Err(RingHashError::EmptyRing);
        }
        let token = Self::hash_key(key);
        let mut seen = Vec::new();
        let mut unique = Vec::new();
        for (&_, vn) in self.ring.range(token..) {
            if !seen.contains(&vn.node_id) {
                seen.push(vn.node_id);
                unique.push(vn.node_id);
                if unique.len() >= n { break; }
            }
        }
        if unique.len() < n {
            for (&_, vn) in self.ring.iter() {
                if !seen.contains(&vn.node_id) {
                    seen.push(vn.node_id);
                    unique.push(vn.node_id);
                    if unique.len() >= n { break; }
                }
            }
        }
        Ok(unique)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn ring_size(&self) -> usize {
        self.ring.len()
    }

    pub fn contains_node(&self, id: u64) -> bool {
        self.nodes.contains_key(&id)
    }

    pub fn node_info(&self, id: u64) -> Option<&NodeInfo> {
        self.nodes.get(&id)
    }

    pub fn node_ids(&self) -> Vec<u64> {
        self.nodes.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ring() {
        let rh = RingHasher::new(64);
        assert_eq!(rh.node_count(), 0);
        assert_eq!(rh.ring_size(), 0);
    }

    #[test]
    fn add_node() {
        let mut rh = RingHasher::new(4);
        rh.add_node(1, 1).unwrap();
        assert_eq!(rh.node_count(), 1);
        assert_eq!(rh.ring_size(), 4);
        assert!(rh.contains_node(1));
    }

    #[test]
    fn add_duplicate() {
        let mut rh = RingHasher::new(4);
        rh.add_node(1, 1).unwrap();
        let err = rh.add_node(1, 1).unwrap_err();
        assert!(matches!(err, RingHashError::NodeExists { .. }));
    }

    #[test]
    fn remove_node() {
        let mut rh = RingHasher::new(4);
        rh.add_node(1, 1).unwrap();
        rh.remove_node(1).unwrap();
        assert_eq!(rh.node_count(), 0);
        assert_eq!(rh.ring_size(), 0);
    }

    #[test]
    fn remove_not_found() {
        let mut rh = RingHasher::new(4);
        let err = rh.remove_node(99).unwrap_err();
        assert!(matches!(err, RingHashError::NodeNotFound { .. }));
    }

    #[test]
    fn lookup_single_node() {
        let mut rh = RingHasher::new(4);
        rh.add_node(1, 1).unwrap();
        let node = rh.lookup(42).unwrap();
        assert_eq!(node, 1);
    }

    #[test]
    fn lookup_deterministic() {
        let mut rh = RingHasher::new(64);
        rh.add_node(1, 1).unwrap();
        rh.add_node(2, 1).unwrap();
        rh.add_node(3, 1).unwrap();
        let a = rh.lookup(12345).unwrap();
        let b = rh.lookup(12345).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn lookup_empty_ring() {
        let rh: RingHasher = RingHasher::new(4);
        let err = rh.lookup(1).unwrap_err();
        assert!(matches!(err, RingHashError::EmptyRing));
    }

    #[test]
    fn lookup_n_distinct() {
        let mut rh = RingHasher::new(64);
        rh.add_node(1, 1).unwrap();
        rh.add_node(2, 1).unwrap();
        rh.add_node(3, 1).unwrap();
        let nodes = rh.lookup_n(42, 3).unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes.iter().collect::<std::collections::HashSet<_>>().len(), 3);
    }

    #[test]
    fn distribution() {
        let mut rh = RingHasher::new(128);
        rh.add_node(1, 1).unwrap();
        rh.add_node(2, 1).unwrap();
        let mut counts = BTreeMap::new();
        for k in 0..1000u64 {
            let node = rh.lookup(k).unwrap();
            *counts.entry(node).or_insert(0) += 1;
        }
        assert_eq!(counts.len(), 2);
        assert!(counts[&1] > 300 && counts[&1] < 700);
    }

    #[test]
    fn remove_changes_routing() {
        let mut rh = RingHasher::new(64);
        rh.add_node(1, 1).unwrap();
        rh.add_node(2, 1).unwrap();
        rh.remove_node(2).unwrap();
        let node = rh.lookup(42).unwrap();
        assert_eq!(node, 1);
    }

    #[test]
    fn error_display() {
        assert!(RingHashError::EmptyRing.to_string().contains("empty"));
    }
}

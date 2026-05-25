use std::collections::BTreeMap;

pub struct RingHash {
    ring: BTreeMap<u64, u64>,
    vnodes: usize,
    total_add: u64,
    total_remove: u64,
    total_lookup: u64,
}

fn hash_vnode(node: u64, vn: usize) -> u64 {
    let mut h = node.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(vn as u64);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;
    h
}

fn hash_key(key: u64) -> u64 {
    let mut h = key.wrapping_mul(0x9e3779b97f4a7c15);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h
}

impl RingHash {
    pub fn new(vnodes: usize) -> Self { Self { ring: BTreeMap::new(), vnodes: vnodes.max(1), total_add: 0, total_remove: 0, total_lookup: 0 } }

    pub fn add_node(&mut self, node: u64) {
        self.total_add += 1;
        for vn in 0..self.vnodes { self.ring.insert(hash_vnode(node, vn), node); }
    }

    pub fn remove_node(&mut self, node: u64) {
        self.total_remove += 1;
        for vn in 0..self.vnodes { self.ring.remove(&hash_vnode(node, vn)); }
    }

    pub fn lookup(&mut self, key: u64) -> Option<u64> {
        self.total_lookup += 1;
        let h = hash_key(key);
        if let Some((&_, &node)) = self.ring.range(h..).next() { return Some(node); }
        self.ring.iter().next().map(|(_, &n)| n)
    }

    pub fn node_count(&self) -> usize {
        let mut nodes = std::collections::BTreeSet::new();
        for &n in self.ring.values() { nodes.insert(n); }
        nodes.len()
    }

    pub fn ring_len(&self) -> usize { self.ring.len() }
    pub fn total_add(&self) -> u64 { self.total_add }
    pub fn total_remove(&self) -> u64 { self.total_remove }
    pub fn total_lookup(&self) -> u64 { self.total_lookup }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_lookup() {
        let mut rh = RingHash::new(100);
        rh.add_node(1);
        assert!(rh.lookup(42).is_some());
    }

    #[test]
    fn distribution() {
        let mut rh = RingHash::new(150);
        for n in 1..=5u64 { rh.add_node(n); }
        let mut counts = std::collections::BTreeMap::new();
        for k in 0..1000u64 { *counts.entry(rh.lookup(k).unwrap()).or_insert(0) += 1; }
        assert_eq!(counts.len(), 5);
        for &c in counts.values() { assert!(c > 50, "node got {c} keys"); }
    }

    #[test]
    fn remove() {
        let mut rh = RingHash::new(50);
        rh.add_node(1); rh.add_node(2);
        rh.remove_node(1);
        assert_eq!(rh.node_count(), 1);
    }

    #[test]
    fn empty() { assert!(RingHash::new(10).lookup(1).is_none()); }

    #[test]
    fn consistent() {
        let mut rh = RingHash::new(100);
        rh.add_node(1); rh.add_node(2); rh.add_node(3);
        let a = rh.lookup(42);
        rh.add_node(4);
        let b = rh.lookup(42);
        assert_eq!(a, b);
    }

    #[test]
    fn stats() {
        let mut rh = RingHash::new(10);
        rh.add_node(1); rh.lookup(1);
        assert_eq!(rh.total_add(), 1);
        assert_eq!(rh.total_lookup(), 1);
    }
}

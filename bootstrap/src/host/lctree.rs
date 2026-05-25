use std::collections::BTreeMap;

pub struct LcTree {
    parent: BTreeMap<u64, u64>,
    total_link: u64,
    total_cut: u64,
    total_query: u64,
}

impl LcTree {
    pub fn new() -> Self { Self { parent: BTreeMap::new(), total_link: 0, total_cut: 0, total_query: 0 } }

    pub fn link(&mut self, child: u64, par: u64) -> bool {
        self.total_link += 1;
        if self.parent.contains_key(&child) { return false; }
        self.parent.insert(child, par);
        true
    }

    pub fn cut(&mut self, child: u64) -> bool {
        self.total_cut += 1;
        self.parent.remove(&child).is_some()
    }

    pub fn root(&mut self, mut node: u64) -> u64 {
        self.total_query += 1;
        while let Some(&p) = self.parent.get(&node) { node = p; }
        node
    }

    pub fn connected(&mut self, a: u64, b: u64) -> bool { self.root(a) == self.root(b) }

    pub fn depth(&mut self, mut node: u64) -> usize {
        self.total_query += 1;
        let mut d = 0;
        while let Some(&p) = self.parent.get(&node) { node = p; d += 1; }
        d
    }

    pub fn lca(&mut self, mut a: u64, mut b: u64) -> u64 {
        self.total_query += 1;
        let da = self.depth_internal(a);
        let db = self.depth_internal(b);
        if da < db { std::mem::swap(&mut a, &mut b); }
        for _ in 0..da.max(db) - da.min(db) { a = self.parent[&a]; }
        while a != b { a = self.parent[&a]; b = self.parent[&b]; }
        a
    }

    fn depth_internal(&self, mut node: u64) -> usize {
        let mut d = 0;
        while let Some(&p) = self.parent.get(&node) { node = p; d += 1; }
        d
    }

    pub fn total_link(&self) -> u64 { self.total_link }
    pub fn total_cut(&self) -> u64 { self.total_cut }
    pub fn total_query(&self) -> u64 { self.total_query }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_root() {
        let mut lc = LcTree::new();
        lc.link(1, 0); lc.link(2, 1);
        assert_eq!(lc.root(2), 0);
    }

    #[test]
    fn cut() {
        let mut lc = LcTree::new();
        lc.link(1, 0); lc.link(2, 1);
        assert!(lc.cut(2));
        assert_ne!(lc.root(1), lc.root(2));
    }

    #[test]
    fn connected() {
        let mut lc = LcTree::new();
        lc.link(1, 0); lc.link(2, 1);
        assert!(lc.connected(0, 2));
        assert!(!lc.connected(0, 5));
    }

    #[test]
    fn depth() {
        let mut lc = LcTree::new();
        lc.link(1, 0); lc.link(2, 1);
        assert_eq!(lc.depth(2), 2);
        assert_eq!(lc.depth(0), 0);
    }

    #[test]
    fn lca() {
        let mut lc = LcTree::new();
        lc.link(1, 0); lc.link(2, 0); lc.link(3, 1); lc.link(4, 2);
        assert_eq!(lc.lca(3, 4), 0);
        assert_eq!(lc.lca(3, 1), 1);
    }

    #[test]
    fn dup_link() {
        let mut lc = LcTree::new();
        lc.link(1, 0);
        assert!(!lc.link(1, 2));
    }

    #[test]
    fn stats() {
        let mut lc = LcTree::new();
        lc.link(1, 0); lc.root(1); lc.cut(1);
        assert_eq!(lc.total_link(), 1);
        assert_eq!(lc.total_query(), 1);
        assert_eq!(lc.total_cut(), 1);
    }
}

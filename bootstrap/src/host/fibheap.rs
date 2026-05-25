use std::collections::BTreeMap;

#[derive(Clone)]
struct Node {
    id: u64,
    key: i64,
    degree: usize,
    marked: bool,
    parent: Option<u64>,
    child: Option<u64>,
    left: u64,
    right: u64,
}

pub struct FibHeap {
    nodes: BTreeMap<u64, Node>,
    min_id: Option<u64>,
    roots: Vec<u64>,
    total_inserts: u64,
    total_extract_mins: u64,
    total_decrease_keys: u64,
}

impl FibHeap {
    pub fn new() -> Self { Self { nodes: BTreeMap::new(), min_id: None, roots: Vec::new(), total_inserts: 0, total_extract_mins: 0, total_decrease_keys: 0 } }

    pub fn insert(&mut self, id: u64, key: i64) {
        self.total_inserts += 1;
        let node = Node { id, key, degree: 0, marked: false, parent: None, child: None, left: id, right: id };
        self.nodes.insert(id, node);
        self.roots.push(id);
        if self.min_id.is_none() || key < self.nodes[&self.min_id.unwrap()].key { self.min_id = Some(id); }
    }

    pub fn find_min(&self) -> Option<(u64, i64)> { self.min_id.map(|id| (id, self.nodes[&id].key)) }

    pub fn extract_min(&mut self) -> Option<(u64, i64)> {
        self.total_extract_mins += 1;
        let mid = self.min_id?;
        let key = self.nodes[&mid].key;
        let child = self.nodes[&mid].child;
        if let Some(cid) = child { self.add_children_to_roots(cid); }
        self.roots.retain(|&r| r != mid);
        self.nodes.remove(&mid);
        if self.roots.is_empty() { self.min_id = None; }
        else { self.consolidate(); }
        Some((mid, key))
    }

    fn add_children_to_roots(&mut self, child_id: u64) {
        let mut ids = vec![child_id];
        let mut next = self.nodes[&child_id].right;
        while next != child_id { ids.push(next); next = self.nodes[&next].right; }
        for id in ids {
            if let Some(n) = self.nodes.get_mut(&id) { n.parent = None; n.marked = false; }
            self.roots.push(id);
        }
    }

    fn consolidate(&mut self) {
        let mut deg_map: BTreeMap<usize, u64> = BTreeMap::new();
        let roots: Vec<u64> = self.roots.drain(..).collect();
        for id in roots {
            let mut x = id;
            let mut deg = self.nodes[&x].degree;
            while let Some(&y) = deg_map.get(&deg) {
                let (smaller, larger) = if self.nodes[&x].key <= self.nodes[&y].key { (x, y) } else { (y, x) };
                self.link(larger, smaller);
                deg_map.remove(&deg);
                x = smaller;
                deg += 1;
            }
            deg_map.insert(deg, x);
        }
        self.roots = deg_map.values().copied().collect();
        self.min_id = None;
        for &id in &self.roots {
            if self.min_id.is_none() || self.nodes[&id].key < self.nodes[&self.min_id.unwrap()].key { self.min_id = Some(id); }
        }
    }

    fn link(&mut self, child_id: u64, parent_id: u64) {
        if let Some(c) = self.nodes.get_mut(&child_id) { c.parent = Some(parent_id); c.marked = false; }
        let old_child = self.nodes[&parent_id].child;
        if let Some(n) = self.nodes.get_mut(&parent_id) { n.child = Some(child_id); n.degree += 1; }
        if let Some(oc) = old_child {
            self.nodes.get_mut(&child_id).unwrap().left = oc;
            self.nodes.get_mut(&child_id).unwrap().right = self.nodes[&oc].right;
            self.nodes.get_mut(&oc).unwrap().left = child_id;
        }
    }

    pub fn decrease_key(&mut self, id: u64, new_key: i64) {
        self.total_decrease_keys += 1;
        if let Some(n) = self.nodes.get_mut(&id) { n.key = new_key; }
        if let Some(pid) = self.nodes[&id].parent {
            if new_key < self.nodes[&pid].key {
                let node = &self.nodes[&id];
                let _was_marked = node.marked;
                if let Some(n) = self.nodes.get_mut(&id) { n.parent = None; n.marked = false; }
                self.roots.push(id);
            }
        }
        if self.min_id.is_none() || new_key < self.nodes[&self.min_id.unwrap()].key { self.min_id = Some(id); }
    }

    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn is_empty(&self) -> bool { self.nodes.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_extract_mins(&self) -> u64 { self.total_extract_mins }
    pub fn total_decrease_keys(&self) -> u64 { self.total_decrease_keys }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_find_min() {
        let mut fh = FibHeap::new();
        fh.insert(1, 10); fh.insert(2, 5); fh.insert(3, 20);
        assert_eq!(fh.find_min(), Some((2, 5)));
    }

    #[test]
    fn extract_min() {
        let mut fh = FibHeap::new();
        fh.insert(1, 30); fh.insert(2, 10); fh.insert(3, 20);
        assert_eq!(fh.extract_min(), Some((2, 10)));
        assert_eq!(fh.extract_min(), Some((3, 20)));
        assert_eq!(fh.len(), 1);
    }

    #[test]
    fn extract_all() {
        let mut fh = FibHeap::new();
        fh.insert(1, 3); fh.insert(2, 1); fh.insert(3, 2);
        let mut vals = Vec::new();
        while let Some((id, k)) = fh.extract_min() { vals.push((id, k)); }
        assert_eq!(vals, vec![(2, 1), (3, 2), (1, 3)]);
    }

    #[test]
    fn decrease_key() {
        let mut fh = FibHeap::new();
        fh.insert(1, 100); fh.insert(2, 50);
        fh.decrease_key(1, 10);
        assert_eq!(fh.find_min(), Some((1, 10)));
    }

    #[test]
    fn empty() { assert!(FibHeap::new().extract_min().is_none()); }

    #[test]
    fn single() {
        let mut fh = FibHeap::new();
        fh.insert(1, 42);
        assert_eq!(fh.extract_min(), Some((1, 42)));
        assert!(fh.is_empty());
    }

    #[test]
    fn stats() {
        let mut fh = FibHeap::new();
        fh.insert(1, 1); fh.extract_min();
        assert_eq!(fh.total_inserts(), 1);
        assert_eq!(fh.total_extract_mins(), 1);
    }
}

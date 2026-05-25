use std::collections::BTreeMap;

struct TNode {
    key: u64,
    value: Vec<u8>,
    prio: u8,
    left: Option<usize>,
    right: Option<usize>,
}

pub struct ZipTree {
    nodes: Vec<TNode>,
    index: BTreeMap<u64, usize>,
    root: Option<usize>,
    rank_state: u64,
    total_inserts: u64,
    total_deletes: u64,
    total_lookups: u64,
}

impl ZipTree {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), index: BTreeMap::new(), root: None, rank_state: 0x12345678,
               total_inserts: 0, total_deletes: 0, total_lookups: 0 }
    }

    fn next_prio(&mut self) -> u8 {
        self.rank_state = self.rank_state.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(1);
        (self.rank_state >> 56) as u8
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        if let Some(&idx) = self.index.get(&key) { self.nodes[idx].value = value; return; }
        let prio = self.next_prio();
        let idx = self.nodes.len();
        self.nodes.push(TNode { key, value, prio, left: None, right: None });
        self.index.insert(key, idx);
        self.root = Some(self.insert_rec(self.root, idx));
    }

    fn insert_rec(&mut self, root: Option<usize>, ni: usize) -> usize {
        let Some(ri) = root else { return ni; };
        let np = self.nodes[ni].prio;
        let rp = self.nodes[ri].prio;
        let nk = self.nodes[ni].key;
        let rk = self.nodes[ri].key;
        if np < rp {
            if nk < rk {
                let left = { let n = &self.nodes[ri]; n.left };
                let new_left = self.insert_rec(left, ni);
                self.nodes[ri].left = Some(new_left);
                self.rotate_right(ri)
            } else {
                let right = { let n = &self.nodes[ri]; n.right };
                let new_right = self.insert_rec(right, ni);
                self.nodes[ri].right = Some(new_right);
                self.rotate_left(ri)
            }
        } else {
            if nk < rk {
                self.nodes[ni].right = Some(ri);
                self.nodes[ni].left = { let n = &self.nodes[ri]; n.left };
                self.nodes[ri].left = None;
                ni
            } else {
                self.nodes[ni].left = Some(ri);
                self.nodes[ni].right = { let n = &self.nodes[ri]; n.right };
                self.nodes[ri].right = None;
                ni
            }
        }
    }

    fn rotate_right(&mut self, y: usize) -> usize {
        let x = self.nodes[y].left.unwrap();
        let xr = { let n = &self.nodes[x]; n.right };
        self.nodes[y].left = xr;
        self.nodes[x].right = Some(y);
        x
    }

    fn rotate_left(&mut self, x: usize) -> usize {
        let y = self.nodes[x].right.unwrap();
        let yl = { let n = &self.nodes[y]; n.left };
        self.nodes[x].right = yl;
        self.nodes[y].left = Some(x);
        y
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        let mut cur = self.root;
        while let Some(ci) = cur {
            let (ck, cl, cr) = { let n = &self.nodes[ci]; (n.key, n.left, n.right) };
            if ck == key { return Some(&self.nodes[ci].value); }
            cur = if key < ck { cl } else { cr };
        }
        None
    }

    pub fn contains(&mut self, key: u64) -> bool { self.get(key).is_some() }
    pub fn len(&self) -> usize { self.index.len() }
    pub fn is_empty(&self) -> bool { self.index.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut zt = ZipTree::new();
        zt.insert(5, b"five".to_vec());
        assert_eq!(zt.get(5), Some(&b"five"[..]));
    }

    #[test]
    fn missing() { let mut zt = ZipTree::new(); assert!(zt.get(1).is_none()); }

    #[test]
    fn many() {
        let mut zt = ZipTree::new();
        for i in 0..100u64 { zt.insert(i, vec![i as u8]); }
        for i in 0..100u64 { assert!(zt.contains(i)); }
        assert_eq!(zt.len(), 100);
    }

    #[test]
    fn overwrite() {
        let mut zt = ZipTree::new();
        zt.insert(1, b"old".to_vec()); zt.insert(1, b"new".to_vec());
        assert_eq!(zt.get(1), Some(&b"new"[..]));
        assert_eq!(zt.len(), 1);
    }

    #[test]
    fn reverse_insert() {
        let mut zt = ZipTree::new();
        for i in (0..50u64).rev() { zt.insert(i, vec![]); }
        assert_eq!(zt.len(), 50);
        assert!(zt.contains(0)); assert!(zt.contains(49));
    }

    #[test]
    fn stats() {
        let mut zt = ZipTree::new();
        zt.insert(1, vec![]); zt.get(1);
        assert_eq!(zt.total_inserts(), 1);
        assert_eq!(zt.total_lookups(), 1);
    }
}

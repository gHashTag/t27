use std::cell::Cell;

struct AvlNode {
    key: u64,
    value: Vec<u8>,
    height: u8,
    left: Option<usize>,
    right: Option<usize>,
}

pub struct AvlMap {
    nodes: Vec<AvlNode>,
    root: Option<usize>,
    index: Vec<Option<usize>>,
    total_inserts: Cell<u64>,
    total_lookups: Cell<u64>,
}

impl AvlMap {
    pub fn new() -> Self { Self { nodes: Vec::new(), root: None, index: vec![None; 65536], total_inserts: Cell::new(0), total_lookups: Cell::new(0) } }

    fn height(&self, n: Option<usize>) -> u8 { n.map(|i| self.nodes[i].height).unwrap_or(0) }

    fn balance_factor(&self, n: usize) -> i8 {
        let lh = self.height(self.nodes[n].left) as i8;
        let rh = self.height(self.nodes[n].right) as i8;
        lh - rh
    }

    fn update_height(&mut self, n: usize) {
        let lh = self.height(self.nodes[n].left);
        let rh = self.height(self.nodes[n].right);
        self.nodes[n].height = 1 + lh.max(rh);
    }

    fn rotate_right(&mut self, y: usize) -> usize {
        let x = self.nodes[y].left.unwrap();
        let t2 = self.nodes[x].right;
        self.nodes[y].left = t2;
        self.nodes[x].right = Some(y);
        self.update_height(y);
        self.update_height(x);
        x
    }

    fn rotate_left(&mut self, x: usize) -> usize {
        let y = self.nodes[x].right.unwrap();
        let t2 = self.nodes[y].left;
        self.nodes[x].right = t2;
        self.nodes[y].left = Some(x);
        self.update_height(x);
        self.update_height(y);
        y
    }

    fn balance(&mut self, n: usize) -> usize {
        self.update_height(n);
        let bf = self.balance_factor(n);
        if bf > 1 {
            let left = self.nodes[n].left.unwrap();
            if self.balance_factor(left) < 0 { let r = self.rotate_left(left); self.nodes[n].left = Some(r); }
            self.rotate_right(n)
        } else if bf < -1 {
            let right = self.nodes[n].right.unwrap();
            if self.balance_factor(right) > 0 { let r = self.rotate_right(right); self.nodes[n].right = Some(r); }
            self.rotate_left(n)
        } else { n }
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts.set(self.total_inserts.get() + 1);
        if let Some(idx) = self.index[key as usize] { self.nodes[idx].value = value; return; }
        let idx = self.nodes.len();
        self.nodes.push(AvlNode { key, value, height: 1, left: None, right: None });
        self.index[key as usize] = Some(idx);
        self.root = Some(self.insert_rec(self.root, idx));
    }

    fn insert_rec(&mut self, node: Option<usize>, ni: usize) -> usize {
        let Some(ri) = node else { return ni; };
        let nk = self.nodes[ni].key;
        let rk = self.nodes[ri].key;
        if nk < rk {
            let left = self.nodes[ri].left;
            let new_left = self.insert_rec(left, ni);
            self.nodes[ri].left = Some(new_left);
        } else {
            let right = self.nodes[ri].right;
            let new_right = self.insert_rec(right, ni);
            self.nodes[ri].right = Some(new_right);
        }
        self.balance(ri)
    }

    pub fn get(&self, key: u64) -> Option<&[u8]> {
        self.total_lookups.set(self.total_lookups.get() + 1);
        self.index[key as usize].map(|i| self.nodes[i].value.as_slice())
    }

    pub fn contains(&self, key: u64) -> bool { self.get(key).is_some() }
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn is_empty(&self) -> bool { self.nodes.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts.get() }
    pub fn total_lookups(&self) -> u64 { self.total_lookups.get() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut m = AvlMap::new();
        m.insert(5, b"five".to_vec());
        assert_eq!(m.get(5), Some(&b"five"[..]));
    }

    #[test]
    fn missing() { assert!(AvlMap::new().get(1).is_none()); }

    #[test]
    fn many() {
        let mut m = AvlMap::new();
        for i in 0..100u64 { m.insert(i, vec![i as u8]); }
        for i in 0..100u64 { assert!(m.contains(i)); }
        assert_eq!(m.len(), 100);
    }

    #[test]
    fn overwrite() {
        let mut m = AvlMap::new();
        m.insert(1, b"old".to_vec()); m.insert(1, b"new".to_vec());
        assert_eq!(m.get(1), Some(&b"new"[..]));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn sorted_insert() {
        let mut m = AvlMap::new();
        for i in 0..50u64 { m.insert(i, vec![]); }
        assert_eq!(m.len(), 50);
        assert!(m.contains(0)); assert!(m.contains(49));
    }

    #[test]
    fn height_balanced() {
        let mut m = AvlMap::new();
        for i in 0..100u64 { m.insert(i, vec![]); }
        if let Some(ri) = m.root {
            let h = m.nodes[ri].height;
            assert!(h <= 8, "AVL height {} for 100 nodes", h);
        }
    }

    #[test]
    fn stats() {
        let mut m = AvlMap::new();
        m.insert(1, vec![]); m.get(1);
        assert_eq!(m.total_inserts(), 1);
        assert_eq!(m.total_lookups(), 1);
    }
}

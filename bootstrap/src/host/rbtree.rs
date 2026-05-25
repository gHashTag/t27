use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Color { Red, Black }

struct RBNode {
    key: u64,
    value: Vec<u8>,
    color: Color,
    left: Option<usize>,
    right: Option<usize>,
    parent: Option<usize>,
}

pub struct RBTree {
    nodes: Vec<RBNode>,
    root: Option<usize>,
    total_inserts: u64,
    total_lookups: u64,
}

impl RBTree {
    pub fn new() -> Self { Self { nodes: Vec::new(), root: None, total_inserts: 0, total_lookups: 0 } }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        if let Some(idx) = self.find_idx(key) { self.nodes[idx].value = value; return; }
        let ni = self.nodes.len();
        let is_root = self.root.is_none();
        self.nodes.push(RBNode { key, value, color: if is_root { Color::Black } else { Color::Red }, left: None, right: None, parent: None });
        if is_root { self.root = Some(0); return; }
        let mut cur = self.root.unwrap();
        loop {
            let (child, set_right) = if key < self.nodes[cur].key { (self.nodes[cur].left, false) } else { (self.nodes[cur].right, true) };
            if let Some(c) = child {
                cur = c;
            } else {
                self.nodes[ni].parent = Some(cur);
                if set_right { self.nodes[cur].right = Some(ni); } else { self.nodes[cur].left = Some(ni); }
                break;
            }
        }
        self.fix_insert(ni);
    }

    fn fix_insert(&mut self, mut n: usize) {
        while let Some(pi) = self.nodes[n].parent {
            if self.nodes[pi].color == Color::Black { break; }
            let gp = match self.nodes[pi].parent { Some(g) => g, None => break };
            let uncle = if self.nodes[gp].left == Some(pi) { self.nodes[gp].right } else { self.nodes[gp].left };
            if let Some(ui) = uncle {
                if self.nodes[ui].color == Color::Red {
                    self.nodes[pi].color = Color::Black;
                    self.nodes[ui].color = Color::Black;
                    self.nodes[gp].color = Color::Red;
                    n = gp;
                    continue;
                }
            }
            let p_is_left = self.nodes[gp].left == Some(pi);
            let n_is_left = self.nodes[pi].left == Some(n);
            if p_is_left && !n_is_left { self.rotate_left(pi); n = pi; continue; }
            if !p_is_left && n_is_left { self.rotate_right(pi); n = pi; continue; }
            self.nodes[pi].color = Color::Black;
            self.nodes[gp].color = Color::Red;
            if p_is_left { self.rotate_right(gp); } else { self.rotate_left(gp); }
            break;
        }
        if let Some(ri) = self.root { self.nodes[ri].color = Color::Black; }
    }

    fn rotate_left(&mut self, x: usize) {
        let y = self.nodes[x].right.unwrap();
        self.nodes[x].right = self.nodes[y].left;
        if let Some(t) = self.nodes[y].left { self.nodes[t].parent = Some(x); }
        self.nodes[y].parent = self.nodes[x].parent;
        match self.nodes[x].parent {
            None => self.root = Some(y),
            Some(p) => { if self.nodes[p].left == Some(x) { self.nodes[p].left = Some(y); } else { self.nodes[p].right = Some(y); } }
        }
        self.nodes[y].left = Some(x);
        self.nodes[x].parent = Some(y);
    }

    fn rotate_right(&mut self, x: usize) {
        let y = self.nodes[x].left.unwrap();
        self.nodes[x].left = self.nodes[y].right;
        if let Some(t) = self.nodes[y].right { self.nodes[t].parent = Some(x); }
        self.nodes[y].parent = self.nodes[x].parent;
        match self.nodes[x].parent {
            None => self.root = Some(y),
            Some(p) => { if self.nodes[p].left == Some(x) { self.nodes[p].left = Some(y); } else { self.nodes[p].right = Some(y); } }
        }
        self.nodes[y].right = Some(x);
        self.nodes[x].parent = Some(y);
    }

    fn find_idx(&self, key: u64) -> Option<usize> {
        let mut cur = self.root;
        while let Some(ci) = cur {
            match key.cmp(&self.nodes[ci].key) {
                Ordering::Equal => return Some(ci),
                Ordering::Less => cur = self.nodes[ci].left,
                Ordering::Greater => cur = self.nodes[ci].right,
            }
        }
        None
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        self.find_idx(key).map(|i| self.nodes[i].value.as_slice())
    }

    pub fn contains(&mut self, key: u64) -> bool { self.get(key).is_some() }
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn is_empty(&self) -> bool { self.nodes.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }

    pub fn black_height(&self) -> usize {
        let mut h = 0;
        let mut cur = self.root;
        while let Some(ci) = cur {
            if self.nodes[ci].color == Color::Black { h += 1; }
            cur = self.nodes[ci].left;
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut t = RBTree::new();
        t.insert(5, b"five".to_vec());
        assert_eq!(t.get(5), Some(&b"five"[..]));
    }

    #[test]
    fn missing() { assert!(RBTree::new().get(1).is_none()); }

    #[test]
    fn many() {
        let mut t = RBTree::new();
        for i in 0..100u64 { t.insert(i, vec![i as u8]); }
        for i in 0..100u64 { assert!(t.contains(i)); }
        assert_eq!(t.len(), 100);
    }

    #[test]
    fn overwrite() {
        let mut t = RBTree::new();
        t.insert(1, b"old".to_vec()); t.insert(1, b"new".to_vec());
        assert_eq!(t.get(1), Some(&b"new"[..]));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn black_height_balanced() {
        let mut t = RBTree::new();
        for i in 0..100u64 { t.insert(i, vec![]); }
        let bh = t.black_height();
        assert!(bh <= 6, "black height {} for 100 nodes", bh);
    }

    #[test]
    fn root_is_black() {
        let mut t = RBTree::new();
        for i in 0..10u64 { t.insert(i, vec![]); }
        if let Some(ri) = t.root { assert_eq!(t.nodes[ri].color, Color::Black); }
    }

    #[test]
    fn stats() {
        let mut t = RBTree::new();
        t.insert(1, vec![]); t.get(1);
        assert_eq!(t.total_inserts(), 1);
        assert_eq!(t.total_lookups(), 1);
    }
}

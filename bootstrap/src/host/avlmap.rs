#[derive(Debug, Clone, PartialEq)]
pub enum AvlError {
    NotFound { key: u64 },
}

impl std::fmt::Display for AvlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AvlError::NotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for AvlError {}

struct ANode {
    key: u64,
    value: Vec<u8>,
    height: i32,
    left: Option<usize>,
    right: Option<usize>,
}

pub struct AvlMap {
    nodes: Vec<ANode>,
    root: Option<usize>,
    free: Vec<usize>,
    total_inserts: u64,
    total_removes: u64,
    total_rotations: u64,
}

impl AvlMap {
    pub fn new() -> Self { Self { nodes: Vec::new(), root: None, free: Vec::new(), total_inserts: 0, total_removes: 0, total_rotations: 0 } }

    fn alloc(&mut self, key: u64, value: Vec<u8>) -> usize {
        if let Some(idx) = self.free.pop() {
            self.nodes[idx] = ANode { key, value, height: 1, left: None, right: None };
            return idx;
        }
        let idx = self.nodes.len();
        self.nodes.push(ANode { key, value, height: 1, left: None, right: None });
        idx
    }

    fn node_height(nodes: &[ANode], idx: Option<usize>) -> i32 { idx.map(|i| nodes[i].height).unwrap_or(0) }

    fn update_height_inline(nodes: &mut [ANode], idx: usize) {
        let lh = Self::node_height(nodes, nodes[idx].left);
        let rh = Self::node_height(nodes, nodes[idx].right);
        nodes[idx].height = 1 + lh.max(rh);
    }

    fn balance_factor_inline(nodes: &[ANode], idx: usize) -> i32 {
        let lh = Self::node_height(nodes, nodes[idx].left);
        let rh = Self::node_height(nodes, nodes[idx].right);
        rh - lh
    }

    fn rotate_left(&mut self, idx: usize) -> usize {
        let r = self.nodes[idx].right.unwrap();
        self.nodes[idx].right = self.nodes[r].left;
        self.nodes[r].left = Some(idx);
        Self::update_height_inline(&mut self.nodes, idx);
        Self::update_height_inline(&mut self.nodes, r);
        self.total_rotations += 1;
        r
    }

    fn rotate_right(&mut self, idx: usize) -> usize {
        let l = self.nodes[idx].left.unwrap();
        self.nodes[idx].left = self.nodes[l].right;
        self.nodes[l].right = Some(idx);
        Self::update_height_inline(&mut self.nodes, idx);
        Self::update_height_inline(&mut self.nodes, l);
        self.total_rotations += 1;
        l
    }

    fn balance(&mut self, idx: usize) -> usize {
        Self::update_height_inline(&mut self.nodes, idx);
        let bf = Self::balance_factor_inline(&self.nodes, idx);
        if bf > 1 {
            let r = self.nodes[idx].right.unwrap();
            if Self::balance_factor_inline(&self.nodes, r) < 0 {
                let new_r = self.rotate_right(r);
                self.nodes[idx].right = Some(new_r);
            }
            return self.rotate_left(idx);
        }
        if bf < -1 {
            let l = self.nodes[idx].left.unwrap();
            if Self::balance_factor_inline(&self.nodes, l) > 0 {
                let new_l = self.rotate_left(l);
                self.nodes[idx].left = Some(new_l);
            }
            return self.rotate_right(idx);
        }
        idx
    }

    fn insert_rec(&mut self, node: Option<usize>, key: u64, value: Vec<u8>) -> usize {
        match node {
            None => { return self.alloc(key, value); }
            Some(idx) => {
                if key < self.nodes[idx].key {
                    let new_left = self.insert_rec(self.nodes[idx].left, key, value);
                    self.nodes[idx].left = Some(new_left);
                } else if key > self.nodes[idx].key {
                    let new_right = self.insert_rec(self.nodes[idx].right, key, value);
                    self.nodes[idx].right = Some(new_right);
                } else {
                    self.nodes[idx].value = value;
                    return idx;
                }
                self.balance(idx)
            }
        }
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        let new_root = self.insert_rec(self.root, key, value);
        self.root = Some(new_root);
    }

    pub fn get(&self, key: u64) -> Option<&[u8]> {
        let mut cur = self.root;
        while let Some(ci) = cur {
            if key < self.nodes[ci].key { cur = self.nodes[ci].left; }
            else if key > self.nodes[ci].key { cur = self.nodes[ci].right; }
            else { return Some(&self.nodes[ci].value); }
        }
        None
    }

    fn min_node(&self, mut idx: usize) -> usize {
        while let Some(l) = self.nodes[idx].left { idx = l; }
        idx
    }

    fn remove_rec(&mut self, node: Option<usize>, key: u64) -> Option<usize> {
        match node {
            None => None,
            Some(idx) => {
                if key < self.nodes[idx].key {
                    self.nodes[idx].left = self.remove_rec(self.nodes[idx].left, key);
                } else if key > self.nodes[idx].key {
                    self.nodes[idx].right = self.remove_rec(self.nodes[idx].right, key);
                } else {
                    match (self.nodes[idx].left, self.nodes[idx].right) {
                        (None, None) => { self.free.push(idx); return None; }
                        (Some(l), None) => { self.free.push(idx); return Some(l); }
                        (None, Some(r)) => { self.free.push(idx); return Some(r); }
                        (Some(_), Some(r)) => {
                            let succ = self.min_node(r);
                            let succ_key = self.nodes[succ].key;
                            let succ_val = self.nodes[succ].value.clone();
                            self.nodes[idx].key = succ_key;
                            self.nodes[idx].value = succ_val;
                            self.nodes[idx].right = self.remove_rec(self.nodes[idx].right, succ_key);
                        }
                    }
                }
                Some(self.balance(idx))
            }
        }
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, AvlError> {
        self.total_removes += 1;
        let value = self.get(key).ok_or(AvlError::NotFound { key })?.to_vec();
        self.root = self.remove_rec(self.root, key);
        Ok(value)
    }

    pub fn height(&self) -> i32 { self.root.map(|r| self.nodes[r].height).unwrap_or(0) }
    pub fn len(&self) -> usize { self.nodes.len() - self.free.len() }
    pub fn is_empty(&self) -> bool { self.root.is_none() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_rotations(&self) -> u64 { self.total_rotations }
}

impl Default for AvlMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_am() { assert!(AvlMap::new().is_empty()); }

    #[test]
    fn insert_get() {
        let mut am = AvlMap::new();
        am.insert(1, b"one".to_vec()); am.insert(2, b"two".to_vec());
        assert_eq!(am.get(1), Some(&b"one"[..]));
        assert_eq!(am.get(2), Some(&b"two"[..]));
    }

    #[test]
    fn overwrite() {
        let mut am = AvlMap::new();
        am.insert(1, b"old".to_vec()); am.insert(1, b"new".to_vec());
        assert_eq!(am.get(1), Some(&b"new"[..]));
        assert_eq!(am.len(), 1);
    }

    #[test]
    fn remove() {
        let mut am = AvlMap::new();
        am.insert(1, b"a".to_vec()); am.insert(2, b"b".to_vec());
        am.remove(1).unwrap();
        assert_eq!(am.get(1), None);
        assert_eq!(am.len(), 1);
    }

    #[test]
    fn remove_not_found() { assert!(AvlMap::new().remove(1).is_err()); }

    #[test]
    fn balanced_height() {
        let mut am = AvlMap::new();
        for i in 0..1024u64 { am.insert(i, vec![]); }
        let h = am.height() as f64;
        let expected = (1024f64).log2();
        assert!(h < expected * 1.5, "height {h} too large for 1024 nodes");
    }

    #[test]
    fn many() {
        let mut am = AvlMap::new();
        for i in 0..200u64 { am.insert(i, vec![i as u8]); }
        assert_eq!(am.len(), 200);
        for i in 0..200u64 { assert_eq!(am.get(i), Some(&[i as u8][..])); }
    }

    #[test]
    fn rotations() {
        let mut am = AvlMap::new();
        for i in 0..50 { am.insert(i, vec![]); }
        assert!(am.total_rotations() > 0);
    }

    #[test]
    fn remove_many() {
        let mut am = AvlMap::new();
        for i in 0..30u64 { am.insert(i, vec![]); }
        for i in 0..30u64 { am.remove(i).unwrap(); }
        assert!(am.is_empty());
    }

    #[test]
    fn stats() {
        let mut am = AvlMap::new();
        am.insert(1, vec![]);
        assert_eq!(am.total_inserts(), 1);
    }

    #[test]
    fn error_display() { assert!(AvlError::NotFound { key: 1 }.to_string().contains("not found")); }
}

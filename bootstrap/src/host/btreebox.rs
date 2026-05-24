#[derive(Debug, Clone, PartialEq)]
pub enum BtError {
    NotFound { key: u64 },
}

impl std::fmt::Display for BtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BtError::NotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for BtError {}

struct BNode {
    keys: Vec<u64>,
    values: Vec<Vec<u8>>,
    children: Vec<usize>,
    leaf: bool,
}

pub struct BTreeBox {
    nodes: Vec<BNode>,
    root: Option<usize>,
    order: usize,
    total_inserts: u64,
    total_removes: u64,
    total_splits: u64,
}

impl BTreeBox {
    pub fn new(order: usize) -> Self { Self { nodes: Vec::new(), root: None, order, total_inserts: 0, total_removes: 0, total_splits: 0 } }

    fn alloc_leaf(&mut self) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(BNode { keys: Vec::new(), values: Vec::new(), children: Vec::new(), leaf: true });
        idx
    }

    fn alloc_internal(&mut self) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(BNode { keys: Vec::new(), values: Vec::new(), children: Vec::new(), leaf: false });
        idx
    }

    fn find_pos(keys: &[u64], key: u64) -> usize {
        keys.iter().position(|&k| k >= key).unwrap_or(keys.len())
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        match self.root {
            None => {
                let idx = self.alloc_leaf();
                self.nodes[idx].keys.push(key);
                self.nodes[idx].values.push(value);
                self.root = Some(idx);
                return;
            }
            Some(r) => {
                if self.nodes[r].keys.len() == 2 * self.order - 1 {
                    let new_root = self.alloc_internal();
                    self.nodes[new_root].children.push(r);
                    self.split_child(new_root, 0);
                    self.root = Some(new_root);
                    self.insert_nonfull(new_root, key, value);
                } else {
                    self.insert_nonfull(r, key, value);
                }
            }
        }
    }

    fn insert_nonfull(&mut self, node: usize, key: u64, value: Vec<u8>) {
        if self.nodes[node].leaf {
            let pos = Self::find_pos(&self.nodes[node].keys, key);
            if pos < self.nodes[node].keys.len() && self.nodes[node].keys[pos] == key {
                self.nodes[node].values[pos] = value;
                return;
            }
            self.nodes[node].keys.insert(pos, key);
            self.nodes[node].values.insert(pos, value);
        } else {
            let pos = Self::find_pos(&self.nodes[node].keys, key);
            if pos < self.nodes[node].keys.len() && self.nodes[node].keys[pos] == key {
                self.nodes[node].values[pos] = value;
                return;
            }
            let child = self.nodes[node].children[pos];
            if self.nodes[child].keys.len() == 2 * self.order - 1 {
                self.split_child(node, pos);
                if key > self.nodes[node].keys[pos] {
                    self.insert_nonfull(self.nodes[node].children[pos + 1], key, value);
                } else {
                    self.insert_nonfull(self.nodes[node].children[pos], key, value);
                }
            } else {
                self.insert_nonfull(child, key, value);
            }
        }
    }

    fn split_child(&mut self, parent: usize, idx: usize) {
        self.total_splits += 1;
        let child = self.nodes[parent].children[idx];
        let mid = self.order - 1;
        let mid_key = self.nodes[child].keys[mid];
        let mid_val = self.nodes[child].values[mid].clone();
        let is_leaf = self.nodes[child].leaf;
        let right_keys: Vec<u64> = self.nodes[child].keys.drain((mid + 1)..).collect();
        let right_vals: Vec<Vec<u8>> = self.nodes[child].values.drain((mid + 1)..).collect();
        let right_children: Vec<usize> = if is_leaf { Vec::new() } else { self.nodes[child].children.drain((mid + 1)..).collect() };
        self.nodes[child].keys.truncate(mid);
        self.nodes[child].values.truncate(mid);
        if !is_leaf { self.nodes[child].children.truncate(mid + 1); }
        let right = if is_leaf { self.alloc_leaf() } else { self.alloc_internal() };
        self.nodes[right].keys = right_keys;
        self.nodes[right].values = right_vals;
        self.nodes[right].children = right_children;
        self.nodes[right].leaf = is_leaf;
        self.nodes[parent].keys.insert(idx, mid_key);
        self.nodes[parent].values.insert(idx, mid_val);
        self.nodes[parent].children.insert(idx + 1, right);
    }

    pub fn get(&self, key: u64) -> Option<&[u8]> {
        let mut cur = self.root?;
        loop {
            let pos = Self::find_pos(&self.nodes[cur].keys, key);
            if pos < self.nodes[cur].keys.len() && self.nodes[cur].keys[pos] == key {
                return Some(&self.nodes[cur].values[pos]);
            }
            if self.nodes[cur].leaf { return None; }
            cur = self.nodes[cur].children[pos];
        }
    }

    pub fn range(&self, min: u64, max: u64) -> Vec<(u64, &[u8])> {
        let mut result = Vec::new();
        if let Some(root) = self.root { self.range_rec(root, min, max, &mut result); }
        result
    }

    fn range_rec<'a>(&'a self, node: usize, min: u64, max: u64, result: &mut Vec<(u64, &'a [u8])>) {
        for i in 0..self.nodes[node].keys.len() {
            if !self.nodes[node].leaf && self.nodes[node].keys[i] > min {
                self.range_rec(self.nodes[node].children[i], min, max, result);
            }
            let k = self.nodes[node].keys[i];
            if k >= min && k <= max { result.push((k, &self.nodes[node].values[i])); }
        }
        if !self.nodes[node].leaf {
            let last = self.nodes[node].children.len() - 1;
            self.range_rec(self.nodes[node].children[last], min, max, result);
        }
    }

    pub fn len(&self) -> usize {
        let mut count = 0;
        if let Some(r) = self.root { self.count_rec(r, &mut count); }
        count
    }

    fn count_rec(&self, node: usize, count: &mut usize) {
        *count += self.nodes[node].keys.len();
        for &child in &self.nodes[node].children { self.count_rec(child, count); }
    }

    pub fn is_empty(&self) -> bool { self.root.is_none() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_splits(&self) -> u64 { self.total_splits }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bt() { let bt = BTreeBox::new(3); assert!(bt.is_empty()); }

    #[test]
    fn insert_get() {
        let mut bt = BTreeBox::new(3);
        for i in [30, 10, 20, 50, 40] { bt.insert(i, vec![i as u8]); }
        for i in [10, 20, 30, 40, 50] { assert_eq!(bt.get(i), Some(&[i as u8][..])); }
        assert_eq!(bt.get(25), None);
    }

    #[test]
    fn overwrite() {
        let mut bt = BTreeBox::new(3);
        bt.insert(1, b"old".to_vec()); bt.insert(1, b"new".to_vec());
        assert_eq!(bt.get(1), Some(&b"new"[..]));
    }

    #[test]
    fn splits() {
        let mut bt = BTreeBox::new(3);
        for i in 0..20u64 { bt.insert(i, vec![]); }
        assert!(bt.total_splits() > 0);
        assert_eq!(bt.len(), 20);
    }

    #[test]
    fn range() {
        let mut bt = BTreeBox::new(3);
        for i in 0..20u64 { bt.insert(i, vec![i as u8]); }
        let r: Vec<u64> = bt.range(5, 10).iter().map(|(k, _)| *k).collect();
        assert_eq!(r, vec![5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn large_order() {
        let mut bt = BTreeBox::new(10);
        for i in 0..100u64 { bt.insert(i, vec![i as u8]); }
        assert_eq!(bt.len(), 100);
        for i in 0..100u64 { assert_eq!(bt.get(i), Some(&[i as u8][..])); }
    }

    #[test]
    fn stats() {
        let mut bt = BTreeBox::new(3);
        bt.insert(1, vec![]);
        assert_eq!(bt.total_inserts(), 1);
    }

    #[test]
    fn error_display() { assert!(BtError::NotFound { key: 1 }.to_string().contains("not found")); }
}

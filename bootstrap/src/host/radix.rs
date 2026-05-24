use std::collections::BTreeMap;

#[derive(Debug, Clone)]
struct TrieNode {
    children: BTreeMap<u8, usize>,
    value: Option<Vec<u8>>,
    prefix: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RadixError {
    KeyExists { key: Vec<u8> },
    KeyNotFound { key: Vec<u8> },
}

impl std::fmt::Display for RadixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RadixError::KeyExists { key } => write!(f, "key {:?} exists", key),
            RadixError::KeyNotFound { key } => write!(f, "key {:?} not found", key),
        }
    }
}

impl std::error::Error for RadixError {}

pub struct RadixTrie {
    nodes: Vec<TrieNode>,
    count: usize,
    total_inserts: u64,
    total_deletes: u64,
}

impl RadixTrie {
    pub fn new() -> Self {
        Self { nodes: vec![TrieNode { children: BTreeMap::new(), value: None, prefix: Vec::new() }], count: 0, total_inserts: 0, total_deletes: 0 }
    }

    pub fn insert(&mut self, key: &[u8], value: Vec<u8>) -> Result<(), RadixError> {
        if self.get(key).is_some() { return Err(RadixError::KeyExists { key: key.to_vec() }); }
        self.insert_rec(0, key, value);
        self.count += 1;
        self.total_inserts += 1;
        Ok(())
    }

    fn insert_rec(&mut self, node_idx: usize, key: &[u8], value: Vec<u8>) {
        if key.is_empty() {
            self.nodes[node_idx].value = Some(value);
            return;
        }
        let first = key[0];
        if let Some(&child_idx) = self.nodes[node_idx].children.get(&first) {
            let common = self.common_prefix(key, &self.nodes[child_idx].prefix);
            let child_prefix_len = self.nodes[child_idx].prefix.len();
            if common == child_prefix_len {
                self.insert_rec(child_idx, &key[common..], value);
            } else {
                self.split_node(node_idx, child_idx, common, key, value);
            }
        } else {
            let new_idx = self.nodes.len();
            self.nodes.push(TrieNode { children: BTreeMap::new(), value: Some(value), prefix: key.to_vec() });
            self.nodes[node_idx].children.insert(first, new_idx);
        }
    }

    fn split_node(&mut self, parent_idx: usize, child_idx: usize, split_at: usize, key: &[u8], value: Vec<u8>) {
        let child_prefix = self.nodes[child_idx].prefix.clone();
        let split_prefix = child_prefix[..split_at].to_vec();
        let child_remainder = child_prefix[split_at..].to_vec();
        let key_remainder = key[split_at..].to_vec();
        let new_child_first = child_remainder[0];

        let split_idx = self.nodes.len();
        self.nodes.push(TrieNode { children: BTreeMap::new(), value: None, prefix: split_prefix });

        self.nodes[child_idx].prefix = child_remainder;
        self.nodes[split_idx].children.insert(new_child_first, child_idx);

        if key_remainder.is_empty() {
            self.nodes[split_idx].value = Some(value);
        } else {
            let leaf_first = key_remainder[0];
            let leaf_idx = self.nodes.len();
            self.nodes.push(TrieNode { children: BTreeMap::new(), value: Some(value), prefix: key_remainder });
            self.nodes[split_idx].children.insert(leaf_first, leaf_idx);
        }

        let parent_first = self.nodes[split_idx].prefix[0];
        self.nodes[parent_idx].children.remove(&child_prefix[0]);
        self.nodes[parent_idx].children.insert(parent_first, split_idx);
    }

    fn common_prefix(&self, a: &[u8], b: &[u8]) -> usize {
        a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.get_rec(0, key)
    }

    fn get_rec(&self, node_idx: usize, key: &[u8]) -> Option<&Vec<u8>> {
        if key.is_empty() { return self.nodes[node_idx].value.as_ref(); }
        let first = key[0];
        let child_idx = *self.nodes[node_idx].children.get(&first)?;
        let child_prefix = &self.nodes[child_idx].prefix;
        if key.len() < child_prefix.len() { return None; }
        if key[..child_prefix.len()] != child_prefix[..] { return None; }
        self.get_rec(child_idx, &key[child_prefix.len()..])
    }

    pub fn longest_prefix(&self, key: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        self.lpf_rec(0, key, Vec::new())
    }

    fn lpf_rec(&self, node_idx: usize, key: &[u8], path: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
        let current_best = self.nodes[node_idx].value.as_ref()
            .map(|v| (path.clone(), v.clone()));
        if key.is_empty() { return current_best; }
        let first = key[0];
        if let Some(&child_idx) = self.nodes[node_idx].children.get(&first) {
            let cp = &self.nodes[child_idx].prefix;
            if key.len() >= cp.len() && key[..cp.len()] == cp[..] {
                let mut new_path = path.clone();
                new_path.extend_from_slice(cp);
                return self.lpf_rec(child_idx, &key[cp.len()..], new_path).or(current_best);
            }
        }
        current_best
    }

    pub fn keys_with_prefix(&self, prefix: &[u8]) -> Vec<Vec<u8>> {
        let mut results = Vec::new();
        self.collect_prefix(0, prefix, Vec::new(), &mut results);
        results
    }

    fn collect_prefix(&self, node_idx: usize, prefix: &[u8], path: Vec<u8>, results: &mut Vec<Vec<u8>>) {
        if let Some(&child_idx) = self.nodes[node_idx].children.get(&prefix.get(0).unwrap_or(&0)) {
            let mut new_path = path.clone();
            new_path.extend_from_slice(&self.nodes[child_idx].prefix);
            if self.nodes[child_idx].value.is_some() {
                results.push(new_path.clone());
            }
            self.collect_all(child_idx, new_path, results);
        }
    }

    fn collect_all(&self, node_idx: usize, path: Vec<u8>, results: &mut Vec<Vec<u8>>) {
        for (&_b, &child_idx) in &self.nodes[node_idx].children {
            let mut new_path = path.clone();
            new_path.extend_from_slice(&self.nodes[child_idx].prefix);
            if self.nodes[child_idx].value.is_some() { results.push(new_path.clone()); }
            self.collect_all(child_idx, new_path, results);
        }
    }

    pub fn count(&self) -> usize { self.count }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn is_empty(&self) -> bool { self.count == 0 }
}

impl Default for RadixTrie {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_trie() { assert!(RadixTrie::new().is_empty()); }

    #[test]
    fn insert_get() {
        let mut t = RadixTrie::new();
        t.insert(b"hello", vec![1]).unwrap();
        assert_eq!(t.get(b"hello"), Some(&vec![1]));
    }

    #[test]
    fn duplicate() {
        let mut t = RadixTrie::new();
        t.insert(b"k", vec![1]).unwrap();
        let err = t.insert(b"k", vec![2]).unwrap_err();
        assert!(matches!(err, RadixError::KeyExists { .. }));
    }

    #[test]
    fn not_found() {
        let t = RadixTrie::new();
        assert!(t.get(b"nope").is_none());
    }

    #[test]
    fn common_prefix_keys() {
        let mut t = RadixTrie::new();
        t.insert(b"abc", vec![1]).unwrap();
        t.insert(b"abd", vec![2]).unwrap();
        assert_eq!(t.get(b"abc"), Some(&vec![1]));
        assert_eq!(t.get(b"abd"), Some(&vec![2]));
    }

    #[test]
    fn nested_keys() {
        let mut t = RadixTrie::new();
        t.insert(b"a", vec![1]).unwrap();
        t.insert(b"ab", vec![2]).unwrap();
        t.insert(b"abc", vec![3]).unwrap();
        assert_eq!(t.get(b"a"), Some(&vec![1]));
        assert_eq!(t.get(b"abc"), Some(&vec![3]));
    }

    #[test]
    fn longest_prefix() {
        let mut t = RadixTrie::new();
        t.insert(b"abc", vec![1]).unwrap();
        t.insert(b"abcd", vec![2]).unwrap();
        let (k, v) = t.longest_prefix(b"abcdef").unwrap();
        assert_eq!(k, b"abcd".to_vec());
        assert_eq!(v, vec![2]);
    }

    #[test]
    fn longest_prefix_partial() {
        let mut t = RadixTrie::new();
        t.insert(b"ab", vec![1]).unwrap();
        let (k, _) = t.longest_prefix(b"abc").unwrap();
        assert_eq!(k, b"ab".to_vec());
    }

    #[test]
    fn count() {
        let mut t = RadixTrie::new();
        t.insert(b"a", vec![]).unwrap();
        t.insert(b"b", vec![]).unwrap();
        assert_eq!(t.count(), 2);
    }

    #[test]
    fn empty_key() {
        let mut t = RadixTrie::new();
        t.insert(b"", vec![42]).unwrap();
        assert_eq!(t.get(b""), Some(&vec![42]));
    }

    #[test]
    fn stats() {
        let mut t = RadixTrie::new();
        t.insert(b"k", vec![]).unwrap();
        assert_eq!(t.total_inserts(), 1);
    }

    #[test]
    fn error_display() {
        assert!(RadixError::KeyNotFound { key: b"x".to_vec() }.to_string().contains("not found"));
    }
}

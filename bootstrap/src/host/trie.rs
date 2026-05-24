use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TeError {
    KeyNotFound { key: String },
}

impl std::fmt::Display for TeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeError::KeyNotFound { key } => write!(f, "key '{key}' not found"),
        }
    }
}

impl std::error::Error for TeError {}

struct Node {
    children: BTreeMap<u8, usize>,
    is_end: bool,
    value: Option<Vec<u8>>,
}

pub struct Trie {
    nodes: Vec<Node>,
    total_inserts: u64,
    total_lookups: u64,
    total_removes: u64,
    total_prefix_queries: u64,
}

impl Trie {
    pub fn new() -> Self {
        Self { nodes: vec![Node { children: BTreeMap::new(), is_end: false, value: None }], total_inserts: 0, total_lookups: 0, total_removes: 0, total_prefix_queries: 0 }
    }

    pub fn insert(&mut self, key: &[u8], value: Vec<u8>) {
        let mut cur = 0;
        for &b in key {
            let next = if let Some(&n) = self.nodes[cur].children.get(&b) {
                n
            } else {
                let n = self.nodes.len();
                self.nodes.push(Node { children: BTreeMap::new(), is_end: false, value: None });
                self.nodes[cur].children.insert(b, n);
                n
            };
            cur = next;
        }
        self.nodes[cur].is_end = true;
        self.nodes[cur].value = Some(value);
        self.total_inserts += 1;
    }

    pub fn get(&mut self, key: &[u8]) -> Option<&Vec<u8>> {
        self.total_lookups += 1;
        let node = self.find_node(key)?;
        if node.is_end { node.value.as_ref() } else { None }
    }

    fn find_node(&self, key: &[u8]) -> Option<&Node> {
        let mut cur = 0;
        for &b in key {
            cur = *self.nodes[cur].children.get(&b)?;
        }
        Some(&self.nodes[cur])
    }

    pub fn contains(&mut self, key: &[u8]) -> bool { self.get(key).is_some() }

    pub fn remove(&mut self, key: &[u8]) -> Result<Vec<u8>, TeError> {
        self.total_removes += 1;
        let mut cur = 0;
        for &b in key {
            cur = *self.nodes[cur].children.get(&b).ok_or_else(|| TeError::KeyNotFound { key: String::from_utf8_lossy(key).to_string() })?;
        }
        let node = &mut self.nodes[cur];
        if !node.is_end { return Err(TeError::KeyNotFound { key: String::from_utf8_lossy(key).to_string() }); }
        node.is_end = false;
        node.value.take().ok_or_else(|| TeError::KeyNotFound { key: String::from_utf8_lossy(key).to_string() })
    }

    pub fn starts_with(&mut self, prefix: &[u8]) -> Vec<Vec<u8>> {
        self.total_prefix_queries += 1;
        let mut results = Vec::new();
        let cur = match self.find_node(prefix) {
            Some(_) => { let mut c = 0; for &b in prefix { c = *self.nodes[c].children.get(&b).unwrap(); } c }
            None => return results,
        };
        self.collect_keys(cur, prefix.to_vec(), &mut results);
        results
    }

    fn collect_keys(&self, node_idx: usize, prefix: Vec<u8>, results: &mut Vec<Vec<u8>>) {
        if self.nodes[node_idx].is_end { results.push(prefix.clone()); }
        for (&b, &child) in &self.nodes[node_idx].children {
            let mut new_prefix = prefix.clone();
            new_prefix.push(b);
            self.collect_keys(child, new_prefix, results);
        }
    }

    pub fn key_count(&self) -> u64 { self.total_inserts - self.total_removes }
    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_prefix_queries(&self) -> u64 { self.total_prefix_queries }
}

impl Default for Trie {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_trie() { let t = Trie::new(); assert_eq!(t.node_count(), 1); }

    #[test]
    fn insert_get() {
        let mut t = Trie::new();
        t.insert(b"hello", b"world".to_vec());
        assert_eq!(t.get(b"hello"), Some(&b"world".to_vec()));
    }

    #[test]
    fn not_found() {
        let mut t = Trie::new();
        t.insert(b"hello", b"x".to_vec());
        assert_eq!(t.get(b"hell"), None);
        assert_eq!(t.get(b"hellox"), None);
    }

    #[test]
    fn prefix_search() {
        let mut t = Trie::new();
        t.insert(b"abc", vec![]); t.insert(b"abcd", vec![]); t.insert(b"abx", vec![]);
        let keys = t.starts_with(b"ab");
        assert_eq!(keys.len(), 3);
    }

    #[test]
    fn prefix_no_match() {
        let mut t = Trie::new();
        t.insert(b"hello", vec![]);
        assert!(t.starts_with(b"xyz").is_empty());
    }

    #[test]
    fn remove() {
        let mut t = Trie::new();
        t.insert(b"key", b"val".to_vec());
        let v = t.remove(b"key").unwrap();
        assert_eq!(v, b"val");
        assert_eq!(t.get(b"key"), None);
    }

    #[test]
    fn remove_missing() {
        let mut t = Trie::new();
        let err = t.remove(b"nope").unwrap_err();
        assert!(matches!(err, TeError::KeyNotFound { .. }));
    }

    #[test]
    fn contains() {
        let mut t = Trie::new();
        t.insert(b"k", b"v".to_vec());
        assert!(t.contains(b"k"));
        assert!(!t.contains(b"x"));
    }

    #[test]
    fn shared_prefix() {
        let mut t = Trie::new();
        t.insert(b"abc", b"1".to_vec());
        t.insert(b"abd", b"2".to_vec());
        assert_eq!(t.get(b"abc"), Some(&b"1".to_vec()));
        assert_eq!(t.get(b"abd"), Some(&b"2".to_vec()));
        assert!(t.node_count() < 8);
    }

    #[test]
    fn stats() {
        let mut t = Trie::new();
        t.insert(b"k", b"v".to_vec());
        t.get(b"k");
        assert_eq!(t.total_inserts(), 1);
        assert_eq!(t.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(TeError::KeyNotFound { key: "x".into() }.to_string().contains("x")); }
}

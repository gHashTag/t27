use std::collections::BTreeMap;

#[derive(Debug, Clone)]
struct TrieNode {
    children: BTreeMap<u8, usize>,
    value: Option<Vec<u8>>,
    is_end: bool,
}

pub struct TrieMap {
    nodes: Vec<TrieNode>,
    count: usize,
}

impl TrieMap {
    pub fn new() -> Self {
        Self { nodes: vec![TrieNode { children: BTreeMap::new(), value: None, is_end: false }], count: 0 }
    }

    pub fn insert(&mut self, key: &str, value: Vec<u8>) -> bool {
        let mut idx = 0;
        for &b in key.as_bytes() {
            if let Some(&child) = self.nodes[idx].children.get(&b) {
                idx = child;
            } else {
                let new_idx = self.nodes.len();
                self.nodes.push(TrieNode { children: BTreeMap::new(), value: None, is_end: false });
                self.nodes[idx].children.insert(b, new_idx);
                idx = new_idx;
            }
        }
        let was_new = !self.nodes[idx].is_end;
        self.nodes[idx].is_end = true;
        self.nodes[idx].value = Some(value);
        if was_new { self.count += 1; }
        was_new
    }

    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        let idx = self.find_node(key)?;
        if self.nodes[idx].is_end { self.nodes[idx].value.as_ref() } else { None }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.find_node(key).map(|idx| self.nodes[idx].is_end).unwrap_or(false)
    }

    pub fn remove(&mut self, key: &str) -> bool {
        if let Some(idx) = self.find_node(key) {
            if self.nodes[idx].is_end {
                self.nodes[idx].is_end = false;
                self.nodes[idx].value = None;
                self.count -= 1;
                return true;
            }
        }
        false
    }

    fn find_node(&self, key: &str) -> Option<usize> {
        let mut idx = 0;
        for &b in key.as_bytes() {
            idx = *self.nodes[idx].children.get(&b)?;
        }
        Some(idx)
    }

    pub fn starts_with(&self, prefix: &str) -> Vec<String> {
        let mut results = Vec::new();
        if let Some(idx) = self.find_node(prefix) {
            self.collect_keys(idx, prefix.to_string(), &mut results);
        }
        results
    }

    fn collect_keys(&self, idx: usize, prefix: String, results: &mut Vec<String>) {
        if self.nodes[idx].is_end { results.push(prefix.clone()); }
        for (&b, &child) in &self.nodes[idx].children {
            let mut new_prefix = prefix.clone();
            new_prefix.push(b as char);
            self.collect_keys(child, new_prefix, results);
        }
    }

    pub fn autocomplete(&self, prefix: &str, max: usize) -> Vec<String> {
        let mut all = self.starts_with(prefix);
        all.truncate(max);
        all
    }

    pub fn count(&self) -> usize { self.count }
    pub fn is_empty(&self) -> bool { self.count == 0 }

    pub fn longest_common_prefix(&self) -> String {
        if self.count == 0 { return String::new(); }
        let mut prefix = String::new();
        let mut idx = 0;
        loop {
            let children: Vec<(&u8, &usize)> = self.nodes[idx].children.iter().collect();
            if children.len() == 1 && !self.nodes[idx].is_end {
                prefix.push(*children[0].0 as char);
                idx = *children[0].1;
            } else {
                break;
            }
        }
        prefix
    }
}

impl Default for TrieMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_trie() { assert!(TrieMap::new().is_empty()); }

    #[test]
    fn insert_get() {
        let mut t = TrieMap::new();
        assert!(t.insert("hello", vec![1]));
        assert_eq!(t.get("hello"), Some(&vec![1]));
    }

    #[test]
    fn duplicate_insert() {
        let mut t = TrieMap::new();
        assert!(t.insert("k", vec![1]));
        assert!(!t.insert("k", vec![2]));
        assert_eq!(t.get("k"), Some(&vec![2]));
    }

    #[test]
    fn not_found() {
        let t = TrieMap::new();
        assert!(t.get("nope").is_none());
    }

    #[test]
    fn remove() {
        let mut t = TrieMap::new();
        t.insert("key", vec![1]);
        assert!(t.remove("key"));
        assert!(!t.contains("key"));
    }

    #[test]
    fn remove_missing() { assert!(!TrieMap::new().remove("nope")); }

    #[test]
    fn starts_with() {
        let mut t = TrieMap::new();
        t.insert("abc", vec![]); t.insert("abd", vec![]); t.insert("xyz", vec![]);
        let r = t.starts_with("ab");
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn autocomplete() {
        let mut t = TrieMap::new();
        t.insert("apple", vec![]); t.insert("app", vec![]); t.insert("application", vec![]);
        let r = t.autocomplete("app", 2);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn contains() {
        let mut t = TrieMap::new();
        t.insert("test", vec![]);
        assert!(t.contains("test"));
        assert!(!t.contains("tes"));
    }

    #[test]
    fn longest_common_prefix() {
        let mut t = TrieMap::new();
        t.insert("abc", vec![]); t.insert("abd", vec![]);
        assert_eq!(t.longest_common_prefix(), "ab");
    }

    #[test]
    fn count() {
        let mut t = TrieMap::new();
        t.insert("a", vec![]); t.insert("b", vec![]);
        assert_eq!(t.count(), 2);
        t.remove("a");
        assert_eq!(t.count(), 1);
    }

    #[test]
    fn empty_prefix() {
        let mut t = TrieMap::new();
        t.insert("a", vec![]); t.insert("b", vec![]);
        let r = t.starts_with("");
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn lcp_empty() { assert_eq!(TrieMap::new().longest_common_prefix(), ""); }
}

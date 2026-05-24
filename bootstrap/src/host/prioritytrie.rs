use std::collections::BTreeMap;

struct TrieNode {
    children: BTreeMap<u8, usize>,
    score: f64,
    is_entry: bool,
    entry_key: Option<String>,
}

pub struct PriorityTrie {
    nodes: Vec<TrieNode>,
    total_inserts: u64,
    total_queries: u64,
}

#[derive(Debug, Clone)]
pub struct ScoredEntry {
    pub key: String,
    pub score: f64,
}

impl PriorityTrie {
    pub fn new() -> Self {
        Self {
            nodes: vec![TrieNode { children: BTreeMap::new(), score: 0.0, is_entry: false, entry_key: None }],
            total_inserts: 0,
            total_queries: 0,
        }
    }

    pub fn insert(&mut self, key: &str, score: f64) {
        let mut node = 0;
        for &b in key.as_bytes() {
            let next = self.nodes[node].children.get(&b).copied();
            let next = next.unwrap_or_else(|| {
                let idx = self.nodes.len();
                self.nodes.push(TrieNode { children: BTreeMap::new(), score: 0.0, is_entry: false, entry_key: None });
                self.nodes[node].children.insert(b, idx);
                idx
            });
            node = next;
        }
        self.nodes[node].score = score;
        self.nodes[node].is_entry = true;
        self.nodes[node].entry_key = Some(key.to_string());
        self.total_inserts += 1;
    }

    pub fn get(&mut self, key: &str) -> Option<f64> {
        self.total_queries += 1;
        let node = self.follow(key)?;
        if self.nodes[node].is_entry { Some(self.nodes[node].score) } else { None }
    }

    pub fn autocomplete(&mut self, prefix: &str) -> Vec<ScoredEntry> {
        self.total_queries += 1;
        let mut results = Vec::new();
        if let Some(node) = self.follow(prefix) {
            self.collect(node, &mut results);
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn top_k(&mut self, prefix: &str, k: usize) -> Vec<ScoredEntry> {
        let mut all = self.autocomplete(prefix);
        all.truncate(k);
        all
    }

    pub fn update(&mut self, key: &str, score: f64) -> bool {
        if let Some(node) = self.follow(key) {
            if self.nodes[node].is_entry {
                self.nodes[node].score = score;
                return true;
            }
        }
        false
    }

    pub fn remove(&mut self, key: &str) -> bool {
        if let Some(node) = self.follow(key) {
            if self.nodes[node].is_entry {
                self.nodes[node].is_entry = false;
                self.nodes[node].score = 0.0;
                self.nodes[node].entry_key = None;
                return true;
            }
        }
        false
    }

    fn follow(&self, key: &str) -> Option<usize> {
        let mut node = 0;
        for &b in key.as_bytes() {
            node = *self.nodes[node].children.get(&b)?;
        }
        Some(node)
    }

    fn collect(&self, node: usize, results: &mut Vec<ScoredEntry>) {
        if self.nodes[node].is_entry {
            if let Some(ref key) = self.nodes[node].entry_key {
                results.push(ScoredEntry { key: key.clone(), score: self.nodes[node].score });
            }
        }
        for &child in self.nodes[node].children.values() {
            self.collect(child, results);
        }
    }

    pub fn len(&self) -> usize { self.nodes.iter().filter(|n| n.is_entry).count() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

impl Default for PriorityTrie {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_trie() { assert!(PriorityTrie::new().is_empty()); }

    #[test]
    fn insert_get() {
        let mut t = PriorityTrie::new();
        t.insert("hello", 1.0);
        assert_eq!(t.get("hello"), Some(1.0));
        assert_eq!(t.get("hell"), None);
    }

    #[test]
    fn autocomplete() {
        let mut t = PriorityTrie::new();
        t.insert("apple", 3.0);
        t.insert("application", 1.0);
        t.insert("apply", 2.0);
        t.insert("banana", 5.0);
        let results = t.autocomplete("app");
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].key, "apple");
    }

    #[test]
    fn top_k() {
        let mut t = PriorityTrie::new();
        t.insert("a", 5.0); t.insert("ab", 3.0); t.insert("abc", 1.0);
        let top = t.top_k("a", 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].key, "a");
    }

    #[test]
    fn update() {
        let mut t = PriorityTrie::new();
        t.insert("x", 1.0);
        assert!(t.update("x", 2.0));
        assert_eq!(t.get("x"), Some(2.0));
    }

    #[test]
    fn remove() {
        let mut t = PriorityTrie::new();
        t.insert("x", 1.0);
        assert!(t.remove("x"));
        assert!(t.get("x").is_none());
    }

    #[test]
    fn remove_nonexistent() {
        let mut t = PriorityTrie::new();
        assert!(!t.remove("x"));
    }

    #[test]
    fn empty_autocomplete() {
        let mut t = PriorityTrie::new();
        let results = t.autocomplete("x");
        assert!(results.is_empty());
    }

    #[test]
    fn len_tracking() {
        let mut t = PriorityTrie::new();
        t.insert("a", 1.0); t.insert("b", 2.0);
        assert_eq!(t.len(), 2);
        t.remove("a");
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn stats() {
        let mut t = PriorityTrie::new();
        t.insert("a", 1.0);
        t.get("a");
        assert_eq!(t.total_inserts(), 1);
        assert_eq!(t.total_queries(), 1);
    }

    #[test]
    fn update_nonexistent() {
        let mut t = PriorityTrie::new();
        assert!(!t.update("x", 1.0));
    }
}

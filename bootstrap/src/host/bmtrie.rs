use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct TrieEntry {
    pub prefix: u32,
    pub mask: u32,
    pub tag: String,
    pub priority: u32,
}

impl TrieEntry {
    pub fn new(prefix: u32, mask_len: u8, tag: &str, priority: u32) -> Self {
        let mask = if mask_len == 0 { 0u32 } else { !0u32 << (32 - mask_len) };
        Self { prefix: prefix & mask, mask, tag: tag.to_string(), priority }
    }

    pub fn matches(&self, addr: u32) -> bool {
        (addr & self.mask) == self.prefix
    }

    pub fn mask_len(&self) -> u8 {
        if self.mask == 0 { return 0; }
        self.mask.count_ones() as u8
    }
}

#[derive(Debug, Clone)]
pub struct BitmaskTrie {
    entries: BTreeMap<u32, Vec<TrieEntry>>,
    total_lookups: u64,
    total_hits: u64,
}

impl BitmaskTrie {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            total_lookups: 0,
            total_hits: 0,
        }
    }

    pub fn insert(&mut self, entry: TrieEntry) {
        let key = entry.prefix;
        self.entries.entry(key).or_default().push(entry);
    }

    pub fn remove(&mut self, prefix: u32, mask_len: u8) -> bool {
        let mask = if mask_len == 0 { 0u32 } else { !0u32 << (32 - mask_len) };
        let key = prefix & mask;
        let entry = self.entries.get_mut(&key);
        match entry {
            Some(vec) => {
                let len_before = vec.len();
                vec.retain(|e| e.prefix != key || e.mask != mask);
                let removed = vec.len() != len_before;
                if vec.is_empty() {
                    self.entries.remove(&key);
                }
                removed
            }
            None => false,
        }
    }

    pub fn lookup(&mut self, addr: u32) -> Option<&TrieEntry> {
        self.total_lookups += 1;
        let mut best: Option<&TrieEntry> = None;
        for (_, entries) in &self.entries {
            for entry in entries {
                if entry.matches(addr) {
                    match &best {
                        Some(b) if entry.priority > b.priority => best = Some(entry),
                        None => best = Some(entry),
                        _ => {}
                    }
                }
            }
        }
        if best.is_some() {
            self.total_hits += 1;
        }
        best
    }

    pub fn lookup_longest_prefix(&mut self, addr: u32) -> Option<&TrieEntry> {
        self.total_lookups += 1;
        let mut best: Option<&TrieEntry> = None;
        for (_, entries) in &self.entries {
            for entry in entries {
                if entry.matches(addr) {
                    match &best {
                        Some(b) if entry.mask_len() > b.mask_len() => best = Some(entry),
                        None => best = Some(entry),
                        _ => {}
                    }
                }
            }
        }
        if best.is_some() { self.total_hits += 1; }
        best
    }

    pub fn entry_count(&self) -> usize {
        self.entries.values().map(|v| v.len()).sum()
    }

    pub fn total_lookups(&self) -> u64 {
        self.total_lookups
    }

    pub fn total_hits(&self) -> u64 {
        self.total_hits
    }

    pub fn hit_rate(&self) -> f64 {
        if self.total_lookups == 0 { 0.0 } else { self.total_hits as f64 / self.total_lookups as f64 }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for BitmaskTrie {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trie_entry_matches() {
        let e = TrieEntry::new(0x0A000000, 8, "net_a", 1);
        assert!(e.matches(0x0A010203));
        assert!(!e.matches(0x0B000000));
    }

    #[test]
    fn trie_entry_mask_len() {
        let e = TrieEntry::new(0, 24, "x", 0);
        assert_eq!(e.mask_len(), 24);
    }

    #[test]
    fn insert_and_lookup() {
        let mut trie = BitmaskTrie::new();
        trie.insert(TrieEntry::new(0x0A000000, 8, "net_a", 1));
        let result = trie.lookup(0x0A010203);
        assert!(result.is_some());
        assert_eq!(result.unwrap().tag, "net_a");
    }

    #[test]
    fn lookup_miss() {
        let mut trie = BitmaskTrie::new();
        trie.insert(TrieEntry::new(0x0A000000, 8, "net_a", 1));
        assert!(trie.lookup(0x0B000000).is_none());
    }

    #[test]
    fn priority_selection() {
        let mut trie = BitmaskTrie::new();
        trie.insert(TrieEntry::new(0x0A000000, 8, "low", 1));
        trie.insert(TrieEntry::new(0x0A000000, 8, "high", 10));
        let result = trie.lookup(0x0A010203).unwrap();
        assert_eq!(result.tag, "high");
    }

    #[test]
    fn longest_prefix_match() {
        let mut trie = BitmaskTrie::new();
        trie.insert(TrieEntry::new(0x0A000000, 8, "short", 1));
        trie.insert(TrieEntry::new(0x0A010000, 16, "long", 1));
        let result = trie.lookup_longest_prefix(0x0A010203).unwrap();
        assert_eq!(result.tag, "long");
    }

    #[test]
    fn remove_entry() {
        let mut trie = BitmaskTrie::new();
        trie.insert(TrieEntry::new(0x0A000000, 8, "net_a", 1));
        assert!(trie.remove(0x0A000000, 8));
        assert_eq!(trie.entry_count(), 0);
    }

    #[test]
    fn remove_missing() {
        let mut trie = BitmaskTrie::new();
        assert!(!trie.remove(0, 8));
    }

    #[test]
    fn stats() {
        let mut trie = BitmaskTrie::new();
        trie.insert(TrieEntry::new(0x0A000000, 8, "a", 1));
        trie.lookup(0x0A000000);
        trie.lookup(0x0B000000);
        assert_eq!(trie.total_lookups(), 2);
        assert_eq!(trie.total_hits(), 1);
        assert!((trie.hit_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn multiple_prefixes() {
        let mut trie = BitmaskTrie::new();
        trie.insert(TrieEntry::new(0x0A000000, 8, "a", 1));
        trie.insert(TrieEntry::new(0x0B000000, 8, "b", 1));
        trie.insert(TrieEntry::new(0x0C000000, 8, "c", 1));
        assert_eq!(trie.entry_count(), 3);
        assert_eq!(trie.lookup(0x0B000001).unwrap().tag, "b");
    }

    #[test]
    fn zero_mask_matches_all() {
        let mut trie = BitmaskTrie::new();
        trie.insert(TrieEntry::new(0, 0, "default", 0));
        assert!(trie.lookup(0xFFFFFFFF).is_some());
    }

    #[test]
    fn clear() {
        let mut trie = BitmaskTrie::new();
        trie.insert(TrieEntry::new(0, 8, "x", 0));
        trie.clear();
        assert_eq!(trie.entry_count(), 0);
    }
}

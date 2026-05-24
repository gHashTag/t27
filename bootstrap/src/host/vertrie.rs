use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct VerTrieNode {
    pub version: u64,
    pub value: Option<Vec<u8>>,
    pub children: BTreeMap<u8, usize>,
}

pub struct VersionedTrie {
    versions: BTreeMap<u64, Vec<VerTrieNode>>,
    current_version: u64,
    total_inserts: u64,
    total_lookups: u64,
    total_snapshots: u64,
}

impl VersionedTrie {
    pub fn new() -> Self {
        let mut versions = BTreeMap::new();
        versions.insert(0, vec![VerTrieNode { version: 0, value: None, children: BTreeMap::new() }]);
        Self { versions, current_version: 0, total_inserts: 0, total_lookups: 0, total_snapshots: 0 }
    }

    pub fn snapshot(&mut self) -> u64 {
        let frozen_ver = self.current_version;
        let new_ver = self.current_version + 1;
        let copy = self.versions.get(&frozen_ver).cloned().unwrap();
        self.versions.insert(new_ver, copy);
        self.current_version = new_ver;
        self.total_snapshots += 1;
        frozen_ver
    }

    pub fn insert(&mut self, key: &[u8], value: Vec<u8>) {
        let nodes = self.versions.get_mut(&self.current_version).unwrap();
        let mut node = 0;
        for &b in key {
            let next = nodes[node].children.get(&b).copied();
            let next = next.unwrap_or_else(|| {
                let idx = nodes.len();
                nodes.push(VerTrieNode { version: self.current_version, value: None, children: BTreeMap::new() });
                nodes[node].children.insert(b, idx);
                idx
            });
            node = next;
        }
        nodes[node].value = Some(value);
        self.total_inserts += 1;
    }

    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.total_lookups += 1;
        self.get_at(key, self.current_version)
    }

    pub fn get_at(&self, key: &[u8], version: u64) -> Option<Vec<u8>> {
        let nodes = self.versions.get(&version)?;
        let mut node = 0;
        for &b in key {
            node = *nodes[node].children.get(&b)?;
        }
        nodes[node].value.clone()
    }

    pub fn contains(&mut self, key: &[u8]) -> bool { self.get(key).is_some() }

    pub fn diff_keys(&self, v1: u64, v2: u64) -> Vec<Vec<u8>> {
        let mut keys1 = BTreeMap::new();
        let mut keys2 = BTreeMap::new();
        if let Some(nodes) = self.versions.get(&v1) { self.collect_keys(nodes, 0, &mut Vec::new(), &mut keys1); }
        if let Some(nodes) = self.versions.get(&v2) { self.collect_keys(nodes, 0, &mut Vec::new(), &mut keys2); }
        let mut diff = Vec::new();
        for (k, v) in &keys1 {
            if keys2.get(k) != Some(v) { diff.push(k.clone()); }
        }
        for (k, v) in &keys2 {
            if !keys1.contains_key(k) { diff.push(k.clone()); }
            else if keys1.get(k) != Some(v) && !diff.contains(k) { diff.push(k.clone()); }
        }
        diff
    }

    fn collect_keys(&self, nodes: &[VerTrieNode], idx: usize, path: &mut Vec<u8>, result: &mut BTreeMap<Vec<u8>, Vec<u8>>) {
        if let Some(ref v) = nodes[idx].value {
            result.insert(path.clone(), v.clone());
        }
        for (&b, &child) in &nodes[idx].children {
            path.push(b);
            self.collect_keys(nodes, child, path, result);
            path.pop();
        }
    }

    pub fn version(&self) -> u64 { self.current_version }
    pub fn version_count(&self) -> usize { self.versions.len() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_snapshots(&self) -> u64 { self.total_snapshots }
}

impl Default for VersionedTrie {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_trie() { let vt = VersionedTrie::new(); assert_eq!(vt.version(), 0); }

    #[test]
    fn insert_get() {
        let mut vt = VersionedTrie::new();
        vt.insert(b"hello", b"world".to_vec());
        assert_eq!(vt.get(b"hello"), Some(b"world".to_vec()));
    }

    #[test]
    fn snapshot_isolation() {
        let mut vt = VersionedTrie::new();
        vt.insert(b"key", b"v1".to_vec());
        let v1 = vt.snapshot();
        vt.insert(b"key", b"v2".to_vec());
        assert_eq!(vt.get_at(b"key", v1), Some(b"v1".to_vec()));
        assert_eq!(vt.get(b"key"), Some(b"v2".to_vec()));
    }

    #[test]
    fn diff_keys() {
        let mut vt = VersionedTrie::new();
        vt.insert(b"a", b"1".to_vec());
        let v1 = vt.snapshot();
        vt.insert(b"b", b"2".to_vec());
        vt.insert(b"a", b"3".to_vec());
        let diff = vt.diff_keys(v1, vt.version());
        assert!(diff.contains(&b"a".to_vec()));
        assert!(diff.contains(&b"b".to_vec()));
    }

    #[test]
    fn contains() {
        let mut vt = VersionedTrie::new();
        vt.insert(b"k", b"v".to_vec());
        assert!(vt.contains(b"k"));
        assert!(!vt.contains(b"x"));
    }

    #[test]
    fn get_at_missing_version() {
        let vt = VersionedTrie::new();
        assert!(vt.get_at(b"k", 99).is_none());
    }

    #[test]
    fn missing_key() {
        let mut vt = VersionedTrie::new();
        vt.insert(b"hello", b"world".to_vec());
        assert!(vt.get(b"hell").is_none());
    }

    #[test]
    fn multiple_snapshots() {
        let mut vt = VersionedTrie::new();
        vt.insert(b"a", b"1".to_vec());
        let v1 = vt.snapshot();
        vt.insert(b"b", b"2".to_vec());
        let v2 = vt.snapshot();
        vt.insert(b"c", b"3".to_vec());
        assert_eq!(vt.version_count(), 3);
        assert_eq!(vt.get_at(b"c", v1), None);
        assert_eq!(vt.get_at(b"c", v2), None);
        assert_eq!(vt.get(b"c"), Some(b"3".to_vec()));
    }

    #[test]
    fn no_diff_same_version() {
        let mut vt = VersionedTrie::new();
        vt.insert(b"a", b"1".to_vec());
        let diff = vt.diff_keys(0, 0);
        assert!(diff.is_empty());
    }

    #[test]
    fn stats() {
        let mut vt = VersionedTrie::new();
        vt.insert(b"a", b"1".to_vec());
        vt.snapshot();
        vt.get(b"a");
        assert_eq!(vt.total_inserts(), 1);
        assert_eq!(vt.total_snapshots(), 1);
        assert_eq!(vt.total_lookups(), 1);
    }
}

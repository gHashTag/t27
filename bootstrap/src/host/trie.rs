use std::collections::BTreeMap;

pub struct Trie {
    children: BTreeMap<u8, Box<Trie>>,
    is_end: bool,
    count: usize,
}

impl Trie {
    pub fn new() -> Self { Self { children: BTreeMap::new(), is_end: false, count: 0 } }

    pub fn insert(&mut self, key: &[u8]) -> bool {
        let mut node = &mut *self;
        for &b in key {
            node = node.children.entry(b).or_insert_with(|| Box::new(Self::new()));
        }
        if node.is_end { return false; }
        node.is_end = true;
        self.count += 1;
        true
    }

    pub fn contains(&self, key: &[u8]) -> bool {
        let mut node = self;
        for &b in key {
            match node.children.get(&b) {
                Some(n) => node = n,
                None => return false,
            }
        }
        node.is_end
    }

    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        let mut node = self;
        for &b in prefix {
            match node.children.get(&b) {
                Some(n) => node = n,
                None => return false,
            }
        }
        true
    }

    pub fn remove(&mut self, key: &[u8]) -> bool {
        Self::remove_rec(self, key, 0).map(|r| { if r { self.count -= 1; } r }).unwrap_or(false)
    }

    fn remove_rec(node: &mut Self, key: &[u8], depth: usize) -> Option<bool> {
        if depth == key.len() {
            if node.is_end { node.is_end = false; return Some(true); }
            return Some(false);
        }
        let b = key[depth];
        node.children.get_mut(&b)?;
        let removed = Self::remove_rec(node.children.get_mut(&b).unwrap(), key, depth + 1)?;
        if !removed { return Some(false); }
        let child = node.children.get(&b).unwrap();
        if !child.is_end && child.children.is_empty() { node.children.remove(&b); }
        Some(true)
    }

    pub fn len(&self) -> usize { self.count }
    pub fn is_empty(&self) -> bool { self.count == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_lookup() {
        let mut t = Trie::new();
        assert!(t.insert(b"hello"));
        assert!(t.contains(b"hello"));
        assert!(!t.contains(b"hell"));
    }

    #[test]
    fn no_double_insert() {
        let mut t = Trie::new();
        assert!(t.insert(b"abc"));
        assert!(!t.insert(b"abc"));
    }

    #[test]
    fn prefix() {
        let mut t = Trie::new();
        t.insert(b"apple");
        t.insert(b"application");
        assert!(t.starts_with(b"app"));
        assert!(!t.starts_with(b"apt"));
    }

    #[test]
    fn remove() {
        let mut t = Trie::new();
        t.insert(b"hello");
        assert!(t.remove(b"hello"));
        assert!(!t.contains(b"hello"));
        assert!(!t.remove(b"hello"));
    }

    #[test]
    fn count() {
        let mut t = Trie::new();
        t.insert(b"a"); t.insert(b"ab"); t.insert(b"abc");
        assert_eq!(t.len(), 3);
    }

    #[test]
    fn empty() {
        let t = Trie::new();
        assert!(t.is_empty());
        assert!(!t.contains(b""));
    }
}

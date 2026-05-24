#[derive(Debug, Clone, PartialEq)]
pub enum SpError {
    NotFound { key: u64 },
}

impl std::fmt::Display for SpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpError::NotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for SpError {}

struct SNode {
    key: u64,
    value: Vec<u8>,
    parent: Option<usize>,
    left: Option<usize>,
    right: Option<usize>,
}

pub struct SplayMap {
    nodes: Vec<SNode>,
    root: Option<usize>,
    free: Vec<usize>,
    access_count: u64,
    total_inserts: u64,
    total_removes: u64,
    total_rotations: u64,
}

impl SplayMap {
    pub fn new() -> Self { Self { nodes: Vec::new(), root: None, free: Vec::new(), access_count: 0, total_inserts: 0, total_removes: 0, total_rotations: 0 } }

    fn alloc(&mut self, key: u64, value: Vec<u8>) -> usize {
        if let Some(idx) = self.free.pop() {
            self.nodes[idx] = SNode { key, value, parent: None, left: None, right: None };
            return idx;
        }
        let idx = self.nodes.len();
        self.nodes.push(SNode { key, value, parent: None, left: None, right: None });
        idx
    }

    fn set_child(&mut self, parent: usize, is_left: bool, child: Option<usize>) {
        if is_left { self.nodes[parent].left = child; } else { self.nodes[parent].right = child; }
        if let Some(c) = child { self.nodes[c].parent = Some(parent); }
    }

    fn rotate(&mut self, idx: usize) {
        let parent = self.nodes[idx].parent.unwrap();
        let grandparent = self.nodes[parent].parent;
        let is_left = self.nodes[parent].left == Some(idx);
        let child = if is_left { self.nodes[idx].right } else { self.nodes[idx].left };
        self.set_child(parent, is_left, child);
        self.set_child(idx, !is_left, Some(parent));
        if let Some(gp) = grandparent {
            let gp_left = self.nodes[gp].left == Some(parent);
            self.set_child(gp, gp_left, Some(idx));
        } else {
            self.nodes[idx].parent = None;
            self.root = Some(idx);
        }
        self.total_rotations += 1;
    }

    fn splay_node(&mut self, mut idx: usize) {
        while let Some(parent) = self.nodes[idx].parent {
            let grandparent = self.nodes[parent].parent;
            match grandparent {
                None => { self.rotate(idx); }
                Some(gp) => {
                    let parent_left = self.nodes[gp].left == Some(parent);
                    let idx_left = self.nodes[parent].left == Some(idx);
                    if parent_left == idx_left {
                        self.rotate(parent);
                        self.rotate(idx);
                    } else {
                        self.rotate(idx);
                        self.rotate(idx);
                    }
                }
            }
        }
        self.root = Some(idx);
    }

    fn find(&self, key: u64) -> Option<usize> {
        let mut cur = self.root;
        while let Some(idx) = cur {
            if key < self.nodes[idx].key { cur = self.nodes[idx].left; }
            else if key > self.nodes[idx].key { cur = self.nodes[idx].right; }
            else { return Some(idx); }
        }
        None
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        if let Some(idx) = self.find(key) {
            self.nodes[idx].value = value;
            self.splay_node(idx);
            return;
        }
        let new_idx = self.alloc(key, value);
        match self.root {
            None => { self.root = Some(new_idx); return; }
            Some(_) => {}
        }
        let mut cur = self.root.unwrap();
        let mut parent: Option<usize> = None;
        let mut went_left = false;
        loop {
            parent = Some(cur);
            if key < self.nodes[cur].key {
                went_left = true;
                match self.nodes[cur].left { Some(l) => cur = l, None => break }
            } else {
                went_left = false;
                match self.nodes[cur].right { Some(r) => cur = r, None => break }
            }
        }
        if let Some(p) = parent { self.set_child(p, went_left, Some(new_idx)); }
        self.splay_node(new_idx);
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.access_count += 1;
        let idx = self.find(key)?;
        let val_ptr = &self.nodes[idx].value as *const Vec<u8>;
        self.splay_node(idx);
        unsafe { Some(&*val_ptr) }
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, SpError> {
        self.total_removes += 1;
        let idx = self.find(key).ok_or(SpError::NotFound { key })?;
        self.splay_node(idx);
        let root = self.root.unwrap();
        let value = std::mem::take(&mut self.nodes[root].value);
        match (self.nodes[root].left, self.nodes[root].right) {
            (None, None) => { self.root = None; }
            (Some(l), None) => { self.nodes[l].parent = None; self.root = Some(l); }
            (None, Some(r)) => { self.nodes[r].parent = None; self.root = Some(r); }
            (Some(l), Some(r)) => {
                self.nodes[l].parent = None;
                self.root = Some(l);
                let mut rightmost = l;
                while let Some(rr) = self.nodes[rightmost].right { rightmost = rr; }
                self.splay_node(rightmost);
                let new_root = self.root.unwrap();
                self.nodes[r].parent = None;
                self.set_child(new_root, false, Some(r));
            }
        }
        self.free.push(root);
        Ok(value)
    }

    pub fn contains(&mut self, key: u64) -> bool { self.find(key).is_some() }
    pub fn len(&self) -> usize { self.nodes.len() - self.free.len() }
    pub fn is_empty(&self) -> bool { self.root.is_none() }
    pub fn access_count(&self) -> u64 { self.access_count }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_rotations(&self) -> u64 { self.total_rotations }
}

impl Default for SplayMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sm() { assert!(SplayMap::new().is_empty()); }

    #[test]
    fn insert_get() {
        let mut sm = SplayMap::new();
        sm.insert(1, b"one".to_vec()); sm.insert(2, b"two".to_vec());
        assert_eq!(sm.get(1), Some(&b"one"[..]));
        assert_eq!(sm.get(2), Some(&b"two"[..]));
    }

    #[test]
    fn overwrite() {
        let mut sm = SplayMap::new();
        sm.insert(1, b"old".to_vec()); sm.insert(1, b"new".to_vec());
        assert_eq!(sm.get(1), Some(&b"new"[..]));
        assert_eq!(sm.len(), 1);
    }

    #[test]
    fn remove() {
        let mut sm = SplayMap::new();
        sm.insert(1, b"a".to_vec()); sm.insert(2, b"b".to_vec());
        let v = sm.remove(1).unwrap();
        assert_eq!(v, b"a".to_vec());
        assert_eq!(sm.get(1), None);
        assert_eq!(sm.len(), 1);
    }

    #[test]
    fn remove_not_found() { assert!(SplayMap::new().remove(1).is_err()); }

    #[test]
    fn many_inserts() {
        let mut sm = SplayMap::new();
        for i in 0..100u64 { sm.insert(i, vec![i as u8]); }
        assert_eq!(sm.len(), 100);
        for i in 0..100u64 { assert_eq!(sm.get(i), Some(&[i as u8][..])); }
    }

    #[test]
    fn rotations() {
        let mut sm = SplayMap::new();
        for i in 0..20u64 { sm.insert(i, vec![]); }
        assert!(sm.total_rotations() > 0);
    }

    #[test]
    fn remove_root() {
        let mut sm = SplayMap::new();
        sm.insert(1, b"a".to_vec()); sm.insert(2, b"b".to_vec()); sm.insert(3, b"c".to_vec());
        sm.remove(3).unwrap();
        assert_eq!(sm.len(), 2);
    }

    #[test]
    fn contains() {
        let mut sm = SplayMap::new();
        sm.insert(42, b"x".to_vec());
        assert!(sm.contains(42));
        assert!(!sm.contains(99));
    }

    #[test]
    fn stats() {
        let mut sm = SplayMap::new();
        sm.insert(1, vec![]); sm.get(1);
        assert_eq!(sm.total_inserts(), 1);
        assert_eq!(sm.access_count(), 1);
    }

    #[test]
    fn error_display() { assert!(SpError::NotFound { key: 1 }.to_string().contains("not found")); }
}

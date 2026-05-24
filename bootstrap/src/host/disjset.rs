use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum DsError {
    ElementNotFound { id: u64 },
}

impl std::fmt::Display for DsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DsError::ElementNotFound { id } => write!(f, "element {id} not found"),
        }
    }
}

impl std::error::Error for DsError {}

struct Elem {
    id: u64,
    parent: u64,
    rank: u8,
    size: u64,
}

pub struct DisjSet {
    elems: BTreeMap<u64, Elem>,
    components: u64,
    total_unions: u64,
    total_finds: u64,
}

impl DisjSet {
    pub fn new() -> Self { Self { elems: BTreeMap::new(), components: 0, total_unions: 0, total_finds: 0 } }

    pub fn make_set(&mut self, id: u64) {
        if self.elems.contains_key(&id) { return; }
        self.elems.insert(id, Elem { id, parent: id, rank: 0, size: 1 });
        self.components += 1;
    }

    pub fn find(&mut self, id: u64) -> Result<u64, DsError> {
        self.total_finds += 1;
        if !self.elems.contains_key(&id) { return Err(DsError::ElementNotFound { id }); }
        let root = self.find_root(id);
        self.compress(id, root);
        Ok(root)
    }

    fn find_root(&mut self, id: u64) -> u64 {
        let mut cur = id;
        loop {
            let parent = self.elems.get(&cur).unwrap().parent;
            if parent == cur { return cur; }
            cur = parent;
        }
    }

    fn compress(&mut self, id: u64, root: u64) {
        let mut cur = id;
        while cur != root {
            let parent = self.elems.get(&cur).unwrap().parent;
            self.elems.get_mut(&cur).unwrap().parent = root;
            cur = parent;
        }
    }

    pub fn union(&mut self, a: u64, b: u64) -> Result<bool, DsError> {
        let ra = self.find(a)?;
        let rb = self.find(b)?;
        if ra == rb { return Ok(false); }
        let (rank_a, size_a) = { let e = &self.elems[&ra]; (e.rank, e.size) };
        let (rank_b, size_b) = { let e = &self.elems[&rb]; (e.rank, e.size) };
        let (root, child) = if rank_a > rank_b { (ra, rb) } else if rank_b > rank_a { (rb, ra) } else { (ra, rb) };
        self.elems.get_mut(&child).unwrap().parent = root;
        self.elems.get_mut(&root).unwrap().size += size_a.max(size_b);
        if rank_a == rank_b { self.elems.get_mut(&root).unwrap().rank += 1; }
        self.components -= 1;
        self.total_unions += 1;
        Ok(true)
    }

    pub fn connected(&mut self, a: u64, b: u64) -> Result<bool, DsError> { Ok(self.find(a)? == self.find(b)?) }

    pub fn component_size(&mut self, id: u64) -> Result<u64, DsError> {
        let root = self.find(id)?;
        Ok(self.elems.get(&root).unwrap().size)
    }

    pub fn contains(&self, id: u64) -> bool { self.elems.contains_key(&id) }
    pub fn len(&self) -> usize { self.elems.len() }
    pub fn components(&self) -> u64 { self.components }
    pub fn total_unions(&self) -> u64 { self.total_unions }
    pub fn total_finds(&self) -> u64 { self.total_finds }
}

impl Default for DisjSet {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ds() { assert_eq!(DisjSet::new().len(), 0); }

    #[test]
    fn make_set_find() {
        let mut ds = DisjSet::new();
        ds.make_set(1);
        assert_eq!(ds.find(1), Ok(1));
    }

    #[test]
    fn union_find() {
        let mut ds = DisjSet::new();
        ds.make_set(1); ds.make_set(2);
        assert!(ds.union(1, 2).unwrap());
        assert!(ds.connected(1, 2).unwrap());
    }

    #[test]
    fn same_set_union() {
        let mut ds = DisjSet::new();
        ds.make_set(1); ds.make_set(2);
        ds.union(1, 2).unwrap();
        assert!(!ds.union(1, 2).unwrap());
    }

    #[test]
    fn transitive() {
        let mut ds = DisjSet::new();
        ds.make_set(1); ds.make_set(2); ds.make_set(3);
        ds.union(1, 2).unwrap(); ds.union(2, 3).unwrap();
        assert!(ds.connected(1, 3).unwrap());
    }

    #[test]
    fn component_count() {
        let mut ds = DisjSet::new();
        for i in 1..=5 { ds.make_set(i); }
        assert_eq!(ds.components(), 5);
        ds.union(1, 2).unwrap(); ds.union(3, 4).unwrap();
        assert_eq!(ds.components(), 3);
    }

    #[test]
    fn component_size() {
        let mut ds = DisjSet::new();
        ds.make_set(1); ds.make_set(2); ds.make_set(3);
        ds.union(1, 2).unwrap();
        assert_eq!(ds.component_size(1), Ok(2));
    }

    #[test]
    fn not_found() {
        let mut ds = DisjSet::new();
        let err = ds.find(99).unwrap_err();
        assert!(matches!(err, DsError::ElementNotFound { .. }));
    }

    #[test]
    fn path_compression() {
        let mut ds = DisjSet::new();
        for i in 1..=10 { ds.make_set(i); }
        for i in 1..10 { ds.union(i, i + 1).unwrap(); }
        let root = ds.find(5).unwrap();
        assert_eq!(ds.elems.get(&5).unwrap().parent, root);
    }

    #[test]
    fn stats() {
        let mut ds = DisjSet::new();
        ds.make_set(1); ds.make_set(2);
        ds.union(1, 2).unwrap();
        assert_eq!(ds.total_unions(), 1);
        assert!(ds.total_finds() > 0);
    }

    #[test]
    fn error_display() { assert!(DsError::ElementNotFound { id: 1 }.to_string().contains("1")); }
}

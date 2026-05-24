use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SnapError {
    SnapshotNotFound { id: u64 },
    BranchNotFound { name: String },
    BranchExists { name: String },
    NoCommits,
}

impl std::fmt::Display for SnapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapError::SnapshotNotFound { id } => write!(f, "snapshot {id} not found"),
            SnapError::BranchNotFound { name } => write!(f, "branch {name} not found"),
            SnapError::BranchExists { name } => write!(f, "branch {name} exists"),
            SnapError::NoCommits => write!(f, "no commits"),
        }
    }
}

impl std::error::Error for SnapError {}

struct Branch {
    name: String,
    head: u64,
}

pub struct SnapshotStore {
    snapshots: BTreeMap<u64, BTreeMap<String, Vec<u8>>>,
    branches: BTreeMap<String, Branch>,
    current_branch: String,
    next_id: u64,
    total_commits: u64,
    total_rollbacks: u64,
}

impl SnapshotStore {
    pub fn new() -> Self {
        let mut branches = BTreeMap::new();
        branches.insert("main".to_string(), Branch { name: "main".to_string(), head: 0 });
        Self { snapshots: BTreeMap::new(), branches, current_branch: "main".to_string(), next_id: 1, total_commits: 0, total_rollbacks: 0 }
    }

    pub fn commit(&mut self, data: BTreeMap<String, Vec<u8>>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.snapshots.insert(id, data);
        if let Some(b) = self.branches.get_mut(&self.current_branch) { b.head = id; }
        self.total_commits += 1;
        id
    }

    pub fn checkout(&self, snap_id: u64) -> Option<&BTreeMap<String, Vec<u8>>> { self.snapshots.get(&snap_id) }

    pub fn rollback(&mut self, snap_id: u64) -> Result<&BTreeMap<String, Vec<u8>>, SnapError> {
        if !self.snapshots.contains_key(&snap_id) { return Err(SnapError::SnapshotNotFound { id: snap_id }); }
        if let Some(b) = self.branches.get_mut(&self.current_branch) { b.head = snap_id; }
        self.total_rollbacks += 1;
        Ok(self.snapshots.get(&snap_id).unwrap())
    }

    pub fn create_branch(&mut self, name: &str, snap_id: u64) -> Result<(), SnapError> {
        if self.branches.contains_key(name) { return Err(SnapError::BranchExists { name: name.to_string() }); }
        self.branches.insert(name.to_string(), Branch { name: name.to_string(), head: snap_id });
        Ok(())
    }

    pub fn switch_branch(&mut self, name: &str) -> Result<u64, SnapError> {
        let b = self.branches.get(name).ok_or_else(|| SnapError::BranchNotFound { name: name.to_string() })?;
        self.current_branch = name.to_string();
        Ok(b.head)
    }

    pub fn current_branch(&self) -> &str { &self.current_branch }
    pub fn current_head(&self) -> u64 { self.branches.get(&self.current_branch).map(|b| b.head).unwrap_or(0) }
    pub fn snapshot_count(&self) -> usize { self.snapshots.len() }
    pub fn branch_count(&self) -> usize { self.branches.len() }
    pub fn total_commits(&self) -> u64 { self.total_commits }
    pub fn total_rollbacks(&self) -> u64 { self.total_rollbacks }

    pub fn gc(&mut self, keep: &[u64]) -> u64 {
        let keep_set: std::collections::BTreeSet<u64> = keep.iter().copied().collect();
        let before = self.snapshots.len();
        self.snapshots.retain(|&id, _| keep_set.contains(&id));
        (before - self.snapshots.len()) as u64
    }
}

impl Default for SnapshotStore {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_data(k: &str, v: &[u8]) -> BTreeMap<String, Vec<u8>> {
        let mut m = BTreeMap::new(); m.insert(k.to_string(), v.to_vec()); m
    }

    #[test]
    fn new_store() {
        let s = SnapshotStore::new();
        assert_eq!(s.current_branch(), "main");
        assert_eq!(s.branch_count(), 1);
    }

    #[test]
    fn commit_checkout() {
        let mut s = SnapshotStore::new();
        let id = s.commit(make_data("k", b"v1"));
        let data = s.checkout(id).unwrap();
        assert_eq!(data.get("k").unwrap(), b"v1");
    }

    #[test]
    fn multiple_commits() {
        let mut s = SnapshotStore::new();
        let id1 = s.commit(make_data("k", b"v1"));
        let id2 = s.commit(make_data("k", b"v2"));
        assert_ne!(id1, id2);
        assert_eq!(s.checkout(id1).unwrap().get("k").unwrap(), b"v1");
        assert_eq!(s.checkout(id2).unwrap().get("k").unwrap(), b"v2");
    }

    #[test]
    fn rollback() {
        let mut s = SnapshotStore::new();
        let id1 = s.commit(make_data("k", b"v1"));
        s.commit(make_data("k", b"v2"));
        let data = s.rollback(id1).unwrap();
        assert_eq!(data.get("k").unwrap(), b"v1");
        assert_eq!(s.total_rollbacks(), 1);
    }

    #[test]
    fn rollback_not_found() {
        let mut s = SnapshotStore::new();
        let err = s.rollback(99).unwrap_err();
        assert!(matches!(err, SnapError::SnapshotNotFound { .. }));
    }

    #[test]
    fn branch() {
        let mut s = SnapshotStore::new();
        let id = s.commit(make_data("k", b"v1"));
        s.create_branch("dev", id).unwrap();
        s.switch_branch("dev").unwrap();
        assert_eq!(s.current_branch(), "dev");
        s.commit(make_data("k", b"dev-v"));
        assert_eq!(s.current_head(), 2);
    }

    #[test]
    fn branch_exists() {
        let mut s = SnapshotStore::new();
        s.create_branch("main", 0).unwrap_err();
    }

    #[test]
    fn branch_not_found() {
        let mut s = SnapshotStore::new();
        let err = s.switch_branch("nope").unwrap_err();
        assert!(matches!(err, SnapError::BranchNotFound { .. }));
    }

    #[test]
    fn gc() {
        let mut s = SnapshotStore::new();
        let id1 = s.commit(make_data("k", b"v1"));
        let id2 = s.commit(make_data("k", b"v2"));
        let id3 = s.commit(make_data("k", b"v3"));
        let removed = s.gc(&[id2, id3]);
        assert_eq!(removed, 1);
        assert!(s.checkout(id1).is_none());
        assert!(s.checkout(id2).is_some());
    }

    #[test]
    fn stats() {
        let mut s = SnapshotStore::new();
        s.commit(make_data("k", b"v1"));
        s.commit(make_data("k", b"v2"));
        assert_eq!(s.total_commits(), 2);
        assert_eq!(s.snapshot_count(), 2);
    }

    #[test]
    fn error_display() {
        assert!(SnapError::NoCommits.to_string().contains("no commits"));
    }
}

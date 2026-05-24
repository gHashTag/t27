use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum VqError {
    VersionNotFound { version: u64 },
    EmptyQueue,
}

impl std::fmt::Display for VqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VqError::VersionNotFound { version } => write!(f, "version {version} not found"),
            VqError::EmptyQueue => write!(f, "empty queue"),
        }
    }
}

impl std::error::Error for VqError {}

#[derive(Debug, Clone)]
pub struct VersionedEntry {
    pub version: u64,
    pub data: Vec<u8>,
}

pub struct VersionQ {
    entries: BTreeMap<u64, VersionedEntry>,
    current_version: u64,
    max_versions: usize,
    total_appends: u64,
    total_reads: u64,
    total_snapshots: u64,
}

impl VersionQ {
    pub fn new(max_versions: usize) -> Self {
        Self { entries: BTreeMap::new(), current_version: 0, max_versions, total_appends: 0, total_reads: 0, total_snapshots: 0 }
    }

    pub fn append(&mut self, data: Vec<u8>) -> u64 {
        self.current_version += 1;
        self.entries.insert(self.current_version, VersionedEntry { version: self.current_version, data });
        self.total_appends += 1;
        while self.entries.len() > self.max_versions {
            if let Some((&k, _)) = self.entries.first_key_value() { self.entries.remove(&k); }
        }
        self.current_version
    }

    pub fn get(&self, version: u64) -> Option<&VersionedEntry> { self.entries.get(&version) }

    pub fn latest(&self) -> Option<&VersionedEntry> { self.entries.last_key_value().map(|(_, e)| e) }

    pub fn snapshot(&mut self, version: u64) -> Result<Vec<&VersionedEntry>, VqError> {
        self.total_snapshots += 1;
        if self.entries.is_empty() { return Err(VqError::EmptyQueue); }
        let result: Vec<&VersionedEntry> = self.entries.range(..=version).map(|(_, e)| e).collect();
        if result.is_empty() { return Err(VqError::VersionNotFound { version }); }
        Ok(result)
    }

    pub fn diff(&mut self, from: u64, to: u64) -> Vec<&VersionedEntry> {
        self.total_reads += 1;
        self.entries.range(from..=to).map(|(_, e)| e).collect()
    }

    pub fn rollback(&mut self, version: u64) -> Result<u64, VqError> {
        let removed: Vec<u64> = self.entries.range((version + 1)..).map(|(&k, _)| k).collect();
        let count = removed.len() as u64;
        for k in &removed { self.entries.remove(k); }
        self.current_version = version;
        Ok(count)
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn current_version(&self) -> u64 { self.current_version }
    pub fn max_versions(&self) -> usize { self.max_versions }
    pub fn total_appends(&self) -> u64 { self.total_appends }
    pub fn total_reads(&self) -> u64 { self.total_reads }
    pub fn total_snapshots(&self) -> u64 { self.total_snapshots }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue() { let q = VersionQ::new(10); assert!(q.is_empty()); }

    #[test]
    fn append_get() {
        let mut q = VersionQ::new(10);
        let v = q.append(b"hello".to_vec());
        let e = q.get(v).unwrap();
        assert_eq!(e.data, b"hello");
        assert_eq!(e.version, 1);
    }

    #[test]
    fn latest() {
        let mut q = VersionQ::new(10);
        q.append(b"v1".to_vec());
        q.append(b"v2".to_vec());
        assert_eq!(q.latest().unwrap().data, b"v2");
    }

    #[test]
    fn snapshot() {
        let mut q = VersionQ::new(10);
        q.append(b"a".to_vec()); q.append(b"b".to_vec()); q.append(b"c".to_vec());
        let snap = q.snapshot(2).unwrap();
        assert_eq!(snap.len(), 2);
    }

    #[test]
    fn diff() {
        let mut q = VersionQ::new(10);
        q.append(b"a".to_vec()); q.append(b"b".to_vec()); q.append(b"c".to_vec());
        let d = q.diff(2, 3);
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn rollback() {
        let mut q = VersionQ::new(10);
        q.append(b"a".to_vec()); q.append(b"b".to_vec()); q.append(b"c".to_vec());
        let removed = q.rollback(1).unwrap();
        assert_eq!(removed, 2);
        assert_eq!(q.current_version(), 1);
    }

    #[test]
    fn version_cap() {
        let mut q = VersionQ::new(3);
        for i in 0..5 { q.append(vec![i as u8]); }
        assert_eq!(q.len(), 3);
        assert!(q.get(1).is_none());
        assert!(q.get(3).is_some());
    }

    #[test]
    fn snapshot_not_found() {
        let mut q = VersionQ::new(10);
        let err = q.snapshot(5).unwrap_err();
        assert!(matches!(err, VqError::EmptyQueue));
    }

    #[test]
    fn stats() {
        let mut q = VersionQ::new(10);
        q.append(b"x".to_vec());
        q.snapshot(1).unwrap();
        assert_eq!(q.total_appends(), 1);
        assert_eq!(q.total_snapshots(), 1);
    }

    #[test]
    fn error_display() { assert!(VqError::EmptyQueue.to_string().contains("empty")); }
}

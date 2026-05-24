use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SbError {
    KeyNotFound { key: u64 },
    NoSnapshot,
}

impl std::fmt::Display for SbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SbError::KeyNotFound { key } => write!(f, "key {key} not found"),
            SbError::NoSnapshot => write!(f, "no active snapshot"),
        }
    }
}

impl std::error::Error for SbError {}

pub struct SnapBuf {
    active: BTreeMap<u64, Vec<u8>>,
    snapshot: Option<BTreeMap<u64, Vec<u8>>>,
    version: u64,
    total_writes: u64,
    total_reads: u64,
    total_swaps: u64,
    total_snapshots: u64,
}

impl SnapBuf {
    pub fn new() -> Self {
        Self { active: BTreeMap::new(), snapshot: None, version: 0, total_writes: 0, total_reads: 0, total_swaps: 0, total_snapshots: 0 }
    }

    pub fn write(&mut self, key: u64, data: Vec<u8>) {
        self.active.insert(key, data);
        self.total_writes += 1;
    }

    pub fn read(&mut self, key: u64) -> Option<&Vec<u8>> {
        self.total_reads += 1;
        self.active.get(&key)
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, SbError> {
        self.active.remove(&key).ok_or(SbError::KeyNotFound { key })
    }

    pub fn snapshot(&mut self) -> u64 {
        self.snapshot = Some(self.active.clone());
        self.total_snapshots += 1;
        self.version
    }

    pub fn read_snapshot(&self, key: u64) -> Option<&Vec<u8>> {
        self.snapshot.as_ref()?.get(&key)
    }

    pub fn swap(&mut self) -> u64 {
        if let Some(snap) = self.snapshot.take() {
            self.active = snap;
        }
        self.version += 1;
        self.total_swaps += 1;
        self.version
    }

    pub fn clear(&mut self) { self.active.clear(); }

    pub fn contains(&self, key: u64) -> bool { self.active.contains_key(&key) }
    pub fn len(&self) -> usize { self.active.len() }
    pub fn is_empty(&self) -> bool { self.active.is_empty() }
    pub fn version(&self) -> u64 { self.version }
    pub fn has_snapshot(&self) -> bool { self.snapshot.is_some() }
    pub fn total_writes(&self) -> u64 { self.total_writes }
    pub fn total_reads(&self) -> u64 { self.total_reads }
    pub fn total_swaps(&self) -> u64 { self.total_swaps }
    pub fn total_snapshots(&self) -> u64 { self.total_snapshots }
}

impl Default for SnapBuf {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buf() { assert!(SnapBuf::new().is_empty()); }

    #[test]
    fn write_read() {
        let mut sb = SnapBuf::new();
        sb.write(1, b"data".to_vec());
        assert_eq!(sb.read(1), Some(&b"data".to_vec()));
    }

    #[test]
    fn snapshot_isolation() {
        let mut sb = SnapBuf::new();
        sb.write(1, b"v1".to_vec());
        sb.snapshot();
        sb.write(1, b"v2".to_vec());
        assert_eq!(sb.read(1), Some(&b"v2".to_vec()));
        assert_eq!(sb.read_snapshot(1), Some(&b"v1".to_vec()));
    }

    #[test]
    fn swap_restores() {
        let mut sb = SnapBuf::new();
        sb.write(1, b"old".to_vec());
        sb.snapshot();
        sb.write(1, b"new".to_vec());
        sb.swap();
        assert_eq!(sb.read(1), Some(&b"old".to_vec()));
        assert!(!sb.has_snapshot());
    }

    #[test]
    fn remove() {
        let mut sb = SnapBuf::new();
        sb.write(1, b"x".to_vec());
        let v = sb.remove(1).unwrap();
        assert_eq!(v, b"x");
        assert!(sb.is_empty());
    }

    #[test]
    fn remove_missing() {
        let mut sb = SnapBuf::new();
        let err = sb.remove(99).unwrap_err();
        assert!(matches!(err, SbError::KeyNotFound { .. }));
    }

    #[test]
    fn clear() {
        let mut sb = SnapBuf::new();
        sb.write(1, b"x".to_vec());
        sb.clear();
        assert!(sb.is_empty());
    }

    #[test]
    fn no_snapshot_read() {
        let sb = SnapBuf::new();
        assert_eq!(sb.read_snapshot(1), None);
    }

    #[test]
    fn version_advances() {
        let mut sb = SnapBuf::new();
        sb.snapshot();
        sb.swap();
        assert_eq!(sb.version(), 1);
        sb.swap();
        assert_eq!(sb.version(), 2);
    }

    #[test]
    fn stats() {
        let mut sb = SnapBuf::new();
        sb.write(1, b"x".to_vec());
        sb.read(1);
        sb.snapshot();
        sb.swap();
        assert_eq!(sb.total_writes(), 1);
        assert_eq!(sb.total_reads(), 1);
        assert_eq!(sb.total_swaps(), 1);
        assert_eq!(sb.total_snapshots(), 1);
    }

    #[test]
    fn error_display() { assert!(SbError::NoSnapshot.to_string().contains("snapshot")); }
}

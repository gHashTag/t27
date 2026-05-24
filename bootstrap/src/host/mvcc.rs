use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TxnId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub enum MvccError {
    KeyNotFound { key: String },
    WriteConflict { key: String, txn: TxnId },
    TxnNotFound { txn: TxnId },
    TxnNotActive { txn: TxnId },
    AlreadyCommitted { txn: TxnId },
}

impl std::fmt::Display for MvccError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MvccError::KeyNotFound { key } => write!(f, "key {key} not found"),
            MvccError::WriteConflict { key, txn } => write!(f, "write conflict on {key} by {:?}", txn),
            MvccError::TxnNotFound { txn } => write!(f, "txn {:?} not found", txn),
            MvccError::TxnNotActive { txn } => write!(f, "txn {:?} not active", txn),
            MvccError::AlreadyCommitted { txn } => write!(f, "txn {:?} already committed", txn),
        }
    }
}

impl std::error::Error for MvccError {}

struct Versioned {
    value: Vec<u8>,
    version: Version,
    created_by: TxnId,
    deleted: bool,
}

struct TxnState {
    id: TxnId,
    start_version: Version,
    write_set: BTreeSet<String>,
    active: bool,
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub version: Version,
    pub data: Vec<(String, Vec<u8>)>,
}

pub struct MvccStore {
    data: BTreeMap<String, Vec<Versioned>>,
    txns: BTreeMap<u64, TxnState>,
    current_version: u64,
    next_txn: u64,
    total_commits: u64,
    total_rollbacks: u64,
}

impl MvccStore {
    pub fn new() -> Self {
        Self { data: BTreeMap::new(), txns: BTreeMap::new(), current_version: 0, next_txn: 1, total_commits: 0, total_rollbacks: 0 }
    }

    pub fn begin(&mut self) -> TxnId {
        let id = TxnId(self.next_txn);
        self.next_txn += 1;
        self.txns.insert(id.0, TxnState { id, start_version: Version(self.current_version), write_set: BTreeSet::new(), active: true });
        id
    }

    pub fn get(&self, txn: TxnId, key: &str) -> Result<Option<Vec<u8>>, MvccError> {
        let _ts = self.txns.get(&txn.0).ok_or(MvccError::TxnNotFound { txn })?;
        match self.data.get(key) {
            None => Ok(None),
            Some(versions) => {
                for v in versions.iter().rev() {
                    if v.version.0 <= _ts.start_version.0 || v.created_by == txn {
                        if v.deleted { return Ok(None); }
                        return Ok(Some(v.value.clone()));
                    }
                }
                Ok(None)
            }
        }
    }

    pub fn put(&mut self, txn: TxnId, key: &str, value: Vec<u8>) -> Result<(), MvccError> {
        let ts = self.txns.get(&txn.0).ok_or(MvccError::TxnNotFound { txn })?;
        if !ts.active { return Err(MvccError::TxnNotActive { txn }); }
        self.txns.get_mut(&txn.0).unwrap().write_set.insert(key.to_string());
        let versioned = Versioned { value, version: Version(self.current_version + 1), created_by: txn, deleted: false };
        self.data.entry(key.to_string()).or_default().push(versioned);
        Ok(())
    }

    pub fn delete(&mut self, txn: TxnId, key: &str) -> Result<(), MvccError> {
        let ts = self.txns.get(&txn.0).ok_or(MvccError::TxnNotFound { txn })?;
        if !ts.active { return Err(MvccError::TxnNotActive { txn }); }
        self.txns.get_mut(&txn.0).unwrap().write_set.insert(key.to_string());
        let versioned = Versioned { value: Vec::new(), version: Version(self.current_version + 1), created_by: txn, deleted: true };
        self.data.entry(key.to_string()).or_default().push(versioned);
        Ok(())
    }

    pub fn commit(&mut self, txn: TxnId) -> Result<Version, MvccError> {
        let ts = self.txns.get(&txn.0).ok_or(MvccError::TxnNotFound { txn })?;
        if !ts.active { return Err(MvccError::AlreadyCommitted { txn }); }
        self.current_version += 1;
        let ver = Version(self.current_version);
        let write_set: BTreeSet<String> = self.txns.get(&txn.0).unwrap().write_set.clone();
        for key in &write_set {
            if let Some(versions) = self.data.get_mut(key) {
                for v in versions.iter_mut().rev() {
                    if v.created_by == txn { v.version = ver; }
                }
            }
        }
        self.txns.get_mut(&txn.0).unwrap().active = false;
        self.total_commits += 1;
        Ok(ver)
    }

    pub fn rollback(&mut self, txn: TxnId) -> Result<(), MvccError> {
        let ts = self.txns.get(&txn.0).ok_or(MvccError::TxnNotFound { txn })?;
        if !ts.active { return Err(MvccError::AlreadyCommitted { txn }); }
        let write_set: BTreeSet<String> = self.txns.get(&txn.0).unwrap().write_set.clone();
        for key in &write_set {
            if let Some(versions) = self.data.get_mut(key) {
                versions.retain(|v| v.created_by != txn);
            }
        }
        self.txns.get_mut(&txn.0).unwrap().active = false;
        self.total_rollbacks += 1;
        Ok(())
    }

    pub fn snapshot(&self, version: Version) -> Snapshot {
        let mut data = Vec::new();
        for (key, versions) in &self.data {
            for v in versions.iter().rev() {
                if v.version.0 <= version.0 && !v.deleted {
                    data.push((key.clone(), v.value.clone()));
                    break;
                }
            }
        }
        Snapshot { version, data }
    }

    pub fn current_version(&self) -> Version { Version(self.current_version) }
    pub fn total_commits(&self) -> u64 { self.total_commits }
    pub fn total_rollbacks(&self) -> u64 { self.total_rollbacks }
    pub fn active_txns(&self) -> usize { self.txns.values().filter(|t| t.active).count() }
    pub fn gc(&mut self, keep_version: Version) {
        for versions in self.data.values_mut() {
            versions.retain(|v| v.version >= keep_version || v.created_by.0 == 0);
        }
    }
}

impl Default for MvccStore {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_store() {
        let s = MvccStore::new();
        assert_eq!(s.current_version(), Version(0));
    }

    #[test]
    fn begin_txn() {
        let mut s = MvccStore::new();
        let t = s.begin();
        assert_eq!(s.active_txns(), 1);
    }

    #[test]
    fn put_get_commit() {
        let mut s = MvccStore::new();
        let t = s.begin();
        s.put(t, "key", vec![1, 2, 3]).unwrap();
        let v = s.get(t, "key").unwrap();
        assert_eq!(v, Some(vec![1, 2, 3]));
        s.commit(t).unwrap();
        let t2 = s.begin();
        let v2 = s.get(t2, "key").unwrap();
        assert_eq!(v2, Some(vec![1, 2, 3]));
    }

    #[test]
    fn delete_key() {
        let mut s = MvccStore::new();
        let t = s.begin();
        s.put(t, "k", vec![1]).unwrap();
        s.commit(t).unwrap();
        let t2 = s.begin();
        s.delete(t2, "k").unwrap();
        assert!(s.get(t2, "k").unwrap().is_none());
    }

    #[test]
    fn rollback() {
        let mut s = MvccStore::new();
        let t = s.begin();
        s.put(t, "k", vec![42]).unwrap();
        s.rollback(t).unwrap();
        let t2 = s.begin();
        assert!(s.get(t2, "k").unwrap().is_none());
        assert_eq!(s.total_rollbacks(), 1);
    }

    #[test]
    fn snapshot() {
        let mut s = MvccStore::new();
        let t1 = s.begin();
        s.put(t1, "a", vec![1]).unwrap();
        s.put(t1, "b", vec![2]).unwrap();
        let v = s.commit(t1).unwrap();
        let snap = s.snapshot(v);
        assert_eq!(snap.data.len(), 2);
    }

    #[test]
    fn txn_not_found() {
        let s = MvccStore::new();
        let err = s.get(TxnId(999), "k").unwrap_err();
        assert!(matches!(err, MvccError::TxnNotFound { .. }));
    }

    #[test]
    fn already_committed() {
        let mut s = MvccStore::new();
        let t = s.begin();
        s.commit(t).unwrap();
        let err = s.commit(t).unwrap_err();
        assert!(matches!(err, MvccError::AlreadyCommitted { .. }));
    }

    #[test]
    fn isolation() {
        let mut s = MvccStore::new();
        let t1 = s.begin();
        let t2 = s.begin();
        s.put(t1, "k", vec![1]).unwrap();
        assert!(s.get(t2, "k").unwrap().is_none());
        s.commit(t1).unwrap();
        let t3 = s.begin();
        assert_eq!(s.get(t3, "k").unwrap(), Some(vec![1]));
    }

    #[test]
    fn gc() {
        let mut s = MvccStore::new();
        let t = s.begin();
        s.put(t, "k", vec![1]).unwrap();
        s.commit(t).unwrap();
        let v = s.current_version();
        s.gc(v);
        let t2 = s.begin();
        assert_eq!(s.get(t2, "k").unwrap(), Some(vec![1]));
    }

    #[test]
    fn stats() {
        let mut s = MvccStore::new();
        let t = s.begin();
        s.commit(t).unwrap();
        assert_eq!(s.total_commits(), 1);
    }

    #[test]
    fn error_display() {
        assert!(MvccError::KeyNotFound { key: "x".into() }.to_string().contains("x"));
    }
}

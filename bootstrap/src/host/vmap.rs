use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum VmapError {
    VersionTooOld { key: String, requested: u64, current: u64 },
    KeyNotFound { key: String },
    TxExpired { tx: u64 },
}

impl std::fmt::Display for VmapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmapError::VersionTooOld { key, requested, current } => write!(f, "{key}: v{requested} > v{current}"),
            VmapError::KeyNotFound { key } => write!(f, "{key}: not found"),
            VmapError::TxExpired { tx } => write!(f, "tx {tx} expired"),
        }
    }
}

impl std::error::Error for VmapError {}

struct Entry {
    versions: Vec<(u64, Vec<u8>)>,
    deleted_at: Vec<u64>,
}

pub struct VersionedMap {
    data: BTreeMap<String, Entry>,
    version: u64,
    tx_counter: u64,
    snapshots: BTreeMap<u64, u64>,
    gc_threshold: u64,
    total_writes: u64,
    total_reads: u64,
}

impl VersionedMap {
    pub fn new(gc_threshold: u64) -> Self { Self { data: BTreeMap::new(), version: 0, tx_counter: 0, snapshots: BTreeMap::new(), gc_threshold, total_writes: 0, total_reads: 0 } }

    pub fn begin_tx(&mut self) -> u64 {
        let tx = self.tx_counter;
        self.tx_counter += 1;
        self.snapshots.insert(tx, self.version);
        tx
    }

    pub fn commit_tx(&mut self, tx: u64) -> bool { self.snapshots.remove(&tx).is_some() }

    pub fn put(&mut self, key: &str, value: Vec<u8>) -> u64 {
        self.version += 1;
        let v = self.version;
        let entry = self.data.entry(key.to_string()).or_insert_with(|| Entry { versions: Vec::new(), deleted_at: Vec::new() });
        entry.versions.push((v, value));
        self.total_writes += 1;
        v
    }

    pub fn put_tx(&mut self, tx: u64, key: &str, value: Vec<u8>) -> Result<u64, VmapError> {
        let snap = *self.snapshots.get(&tx).ok_or(VmapError::TxExpired { tx })?;
        if let Some(entry) = self.data.get(key) {
            if let Some((latest_v, _)) = entry.versions.last() {
                if *latest_v > snap { return Err(VmapError::VersionTooOld { key: key.to_string(), requested: snap, current: *latest_v }); }
            }
        }
        Ok(self.put(key, value))
    }

    pub fn get(&mut self, key: &str) -> Option<(u64, Vec<u8>)> {
        self.total_reads += 1;
        self.data.get(key).and_then(|e| {
            let ver = e.versions.last()?;
            if e.deleted_at.iter().any(|&d| d >= ver.0) { None } else { Some(ver.clone()) }
        })
    }

    pub fn get_at(&mut self, key: &str, version: u64) -> Option<(u64, Vec<u8>)> {
        self.total_reads += 1;
        self.data.get(key).and_then(|e| {
            let mut found = None;
            for &(v, ref val) in &e.versions {
                if v <= version { found = Some((v, val.clone())); } else { break; }
            }
            found.and_then(|(v, val)| {
                if e.deleted_at.iter().any(|&d| d >= v && d <= version) { None } else { Some((v, val)) }
            })
        })
    }

    pub fn delete(&mut self, key: &str) -> bool {
        self.version += 1;
        let v = self.version;
        if let Some(entry) = self.data.get_mut(key) {
            if !entry.versions.is_empty() {
                entry.deleted_at.push(v);
                self.total_writes += 1;
                return true;
            }
        }
        false
    }

    pub fn keys(&self) -> Vec<String> {
        self.data.iter().filter(|(_, e)| {
            if let Some((latest_v, _)) = e.versions.last() {
                !e.deleted_at.iter().any(|&d| d >= *latest_v)
            } else { false }
        }).map(|(k, _)| k.clone()).collect()
    }

    pub fn gc(&mut self) -> usize {
        let min_snap = self.snapshots.values().copied().min().unwrap_or(self.version);
        let threshold = min_snap.saturating_sub(self.gc_threshold);
        let mut removed = 0;
        for entry in self.data.values_mut() {
            let before = entry.versions.len();
            entry.versions.retain(|(v, _)| *v > threshold);
            entry.deleted_at.retain(|d| *d > threshold);
            removed += before - entry.versions.len();
        }
        self.data.retain(|_, e| !e.versions.is_empty() || !e.deleted_at.is_empty());
        removed
    }

    pub fn version(&self) -> u64 { self.version }
    pub fn active_tx(&self) -> usize { self.snapshots.len() }
    pub fn total_writes(&self) -> u64 { self.total_writes }
    pub fn total_reads(&self) -> u64 { self.total_reads }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() { let m = VersionedMap::new(10); assert_eq!(m.version(), 0); }

    #[test]
    fn put_get() {
        let mut m = VersionedMap::new(10);
        let v = m.put("k", vec![1, 2, 3]);
        let (gv, val) = m.get("k").unwrap();
        assert_eq!(v, gv);
        assert_eq!(val, vec![1, 2, 3]);
    }

    #[test]
    fn version_history() {
        let mut m = VersionedMap::new(10);
        let v1 = m.put("k", vec![1]);
        let v2 = m.put("k", vec![2]);
        let (gv, val) = m.get_at("k", v1).unwrap();
        assert_eq!(val, vec![1]);
        assert_eq!(gv, v1);
        let (gv2, val2) = m.get("k").unwrap();
        assert_eq!(val2, vec![2]);
        assert_eq!(gv2, v2);
    }

    #[test]
    fn delete_key() {
        let mut m = VersionedMap::new(10);
        m.put("k", vec![1]);
        assert!(m.delete("k"));
        assert!(m.get("k").is_none());
    }

    #[test]
    fn snapshot_isolation() {
        let mut m = VersionedMap::new(10);
        m.put("k", vec![1]);
        let tx = m.begin_tx();
        m.put("k", vec![2]);
        let (v, val) = m.get_at("k", m.snapshots[&tx]).unwrap();
        assert_eq!(val, vec![1]);
        m.commit_tx(tx);
    }

    #[test]
    fn tx_conflict() {
        let mut m = VersionedMap::new(10);
        let tx = m.begin_tx();
        m.put("k", vec![1]);
        let err = m.put_tx(tx, "k", vec![2]).unwrap_err();
        assert!(matches!(err, VmapError::VersionTooOld { .. }));
    }

    #[test]
    fn gc() {
        let mut m = VersionedMap::new(2);
        for i in 0..10u8 { m.put("k", vec![i]); }
        let removed = m.gc();
        assert!(removed > 0);
    }

    #[test]
    fn keys() {
        let mut m = VersionedMap::new(10);
        m.put("a", vec![1]); m.put("b", vec![2]);
        let keys = m.keys();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn not_found() { let mut m = VersionedMap::new(10); assert!(m.get("x").is_none()); }

    #[test]
    fn stats() {
        let mut m = VersionedMap::new(10);
        m.put("k", vec![1]);
        m.get("k");
        assert_eq!(m.total_writes(), 1);
        assert_eq!(m.total_reads(), 1);
    }

    #[test]
    fn error_display() { assert!(VmapError::KeyNotFound { key: "x".into() }.to_string().contains("x")); }
}

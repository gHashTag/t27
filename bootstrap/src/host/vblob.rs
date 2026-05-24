use std::collections::BTreeMap;

fn hash_blob(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlobError {
    NotFound { hash: u64 },
    RefNotFound { name: String },
}

impl std::fmt::Display for BlobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlobError::NotFound { hash } => write!(f, "blob {hash:016x} not found"),
            BlobError::RefNotFound { name } => write!(f, "ref {name} not found"),
        }
    }
}

impl std::error::Error for BlobError {}

#[derive(Debug, Clone)]
pub struct BlobRef {
    pub name: String,
    pub hash: u64,
    pub version: u64,
}

pub struct VersionedBlob {
    blobs: BTreeMap<u64, Vec<u8>>,
    refs: BTreeMap<String, BlobRef>,
    ref_versions: BTreeMap<String, u64>,
    total_puts: u64,
    total_dedup_hits: u64,
    total_ref_updates: u64,
}

impl VersionedBlob {
    pub fn new() -> Self {
        Self { blobs: BTreeMap::new(), refs: BTreeMap::new(), ref_versions: BTreeMap::new(), total_puts: 0, total_dedup_hits: 0, total_ref_updates: 0 }
    }

    pub fn put(&mut self, data: Vec<u8>) -> u64 {
        let hash = hash_blob(&data);
        self.total_puts += 1;
        if self.blobs.contains_key(&hash) {
            self.total_dedup_hits += 1;
            return hash;
        }
        self.blobs.insert(hash, data);
        hash
    }

    pub fn get(&self, hash: u64) -> Option<&[u8]> { self.blobs.get(&hash).map(|v| v.as_slice()) }

    pub fn update_ref(&mut self, name: &str, hash: u64) -> u64 {
        let ver = self.ref_versions.entry(name.to_string()).or_insert(0);
        *ver += 1;
        self.refs.insert(name.to_string(), BlobRef { name: name.to_string(), hash, version: *ver });
        self.total_ref_updates += 1;
        *ver
    }

    pub fn get_ref(&self, name: &str) -> Option<&BlobRef> { self.refs.get(name) }

    pub fn resolve(&self, name: &str) -> Result<&[u8], BlobError> {
        let r = self.refs.get(name).ok_or_else(|| BlobError::RefNotFound { name: name.to_string() })?;
        self.blobs.get(&r.hash).map(|v| v.as_slice()).ok_or(BlobError::NotFound { hash: r.hash })
    }

    pub fn history(&self, name: &str) -> Option<u64> {
        self.ref_versions.get(name).copied()
    }

    pub fn contains(&self, hash: u64) -> bool { self.blobs.contains_key(&hash) }
    pub fn blob_count(&self) -> usize { self.blobs.len() }
    pub fn ref_count(&self) -> usize { self.refs.len() }
    pub fn total_puts(&self) -> u64 { self.total_puts }
    pub fn total_dedup_hits(&self) -> u64 { self.total_dedup_hits }
    pub fn total_ref_updates(&self) -> u64 { self.total_ref_updates }
    pub fn total_bytes(&self) -> usize { self.blobs.values().map(|v| v.len()).sum() }

    pub fn gc(&mut self, keep_hashes: &[u64]) -> u64 {
        let keep: std::collections::BTreeSet<u64> = keep_hashes.iter().copied().collect();
        let active_refs: std::collections::BTreeSet<u64> = self.refs.values().map(|r| r.hash).collect();
        let before = self.blobs.len();
        self.blobs.retain(|&h, _| keep.contains(&h) || active_refs.contains(&h));
        (before - self.blobs.len()) as u64
    }
}

impl Default for VersionedBlob {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_store() { assert_eq!(VersionedBlob::new().blob_count(), 0); }

    #[test]
    fn put_get() {
        let mut s = VersionedBlob::new();
        let h = s.put(vec![1, 2, 3]);
        assert_eq!(s.get(h), Some(&[1, 2, 3][..]));
    }

    #[test]
    fn dedup() {
        let mut s = VersionedBlob::new();
        let h1 = s.put(vec![1, 2, 3]);
        let h2 = s.put(vec![1, 2, 3]);
        assert_eq!(h1, h2);
        assert_eq!(s.blob_count(), 1);
        assert_eq!(s.total_dedup_hits(), 1);
    }

    #[test]
    fn ref_update_resolve() {
        let mut s = VersionedBlob::new();
        let h = s.put(vec![42]);
        s.update_ref("latest", h);
        assert_eq!(s.resolve("latest").unwrap(), &[42]);
    }

    #[test]
    fn ref_versioning() {
        let mut s = VersionedBlob::new();
        let h1 = s.put(vec![1]); let h2 = s.put(vec![2]);
        s.update_ref("file", h1);
        let v2 = s.update_ref("file", h2);
        assert_eq!(v2, 2);
        assert_eq!(s.resolve("file").unwrap(), &[2]);
        assert_eq!(s.history("file"), Some(2));
    }

    #[test]
    fn ref_not_found() {
        let s = VersionedBlob::new();
        let err = s.resolve("nope").unwrap_err();
        assert!(matches!(err, BlobError::RefNotFound { .. }));
    }

    #[test]
    fn gc() {
        let mut s = VersionedBlob::new();
        let h1 = s.put(vec![1]); let _h2 = s.put(vec![2]);
        s.update_ref("keep", h1);
        let removed = s.gc(&[]);
        assert_eq!(removed, 1);
        assert_eq!(s.blob_count(), 1);
    }

    #[test]
    fn total_bytes() {
        let mut s = VersionedBlob::new();
        s.put(vec![1, 2, 3]); s.put(vec![4, 5]);
        assert_eq!(s.total_bytes(), 5);
    }

    #[test]
    fn contains() {
        let mut s = VersionedBlob::new();
        let h = s.put(vec![1]);
        assert!(s.contains(h));
        assert!(!s.contains(0));
    }

    #[test]
    fn stats() {
        let mut s = VersionedBlob::new();
        s.put(vec![1]); s.put(vec![1]); s.put(vec![2]);
        assert_eq!(s.total_puts(), 3);
        assert_eq!(s.total_dedup_hits(), 1);
    }

    #[test]
    fn get_ref() {
        let mut s = VersionedBlob::new();
        let h = s.put(vec![1]);
        s.update_ref("r", h);
        let r = s.get_ref("r").unwrap();
        assert_eq!(r.hash, h);
        assert_eq!(r.version, 1);
    }

    #[test]
    fn error_display() {
        assert!(BlobError::NotFound { hash: 42 }.to_string().contains("000000000000002a"));
    }
}

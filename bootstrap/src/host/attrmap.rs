use std::collections::BTreeMap;

#[derive(Clone)]
struct Versioned {
    value: Vec<u8>,
    version: u64,
}

pub struct AttrMap {
    attrs: BTreeMap<Vec<u8>, Versioned>,
    version: u64,
    total_sets: u64,
    total_deletes: u64,
    total_snapshots: u64,
}

pub struct Snapshot {
    data: BTreeMap<Vec<u8>, (Vec<u8>, u64)>,
    version: u64,
}

impl AttrMap {
    pub fn new() -> Self { Self { attrs: BTreeMap::new(), version: 0, total_sets: 0, total_deletes: 0, total_snapshots: 0 } }

    pub fn set(&mut self, key: &[u8], value: Vec<u8>) -> u64 {
        self.total_sets += 1;
        self.version += 1;
        self.attrs.insert(key.to_vec(), Versioned { value, version: self.version });
        self.version
    }

    pub fn get(&self, key: &[u8]) -> Option<(&[u8], u64)> {
        self.attrs.get(key).map(|v| (v.value.as_slice(), v.version))
    }

    pub fn delete(&mut self, key: &[u8]) -> Option<(Vec<u8>, u64)> {
        self.total_deletes += 1;
        self.version += 1;
        self.attrs.remove(key).map(|v| (v.value, v.version))
    }

    pub fn contains(&self, key: &[u8]) -> bool { self.attrs.contains_key(key) }

    pub fn snapshot(&mut self) -> Snapshot {
        self.total_snapshots += 1;
        Snapshot {
            data: self.attrs.iter().map(|(k, v)| (k.clone(), (v.value.clone(), v.version))).collect(),
            version: self.version,
        }
    }

    pub fn restore(&mut self, snap: Snapshot) {
        self.attrs = snap.data.into_iter().map(|(k, (value, version))| (k, Versioned { value, version })).collect();
        self.version = snap.version;
    }

    pub fn keys(&self) -> Vec<&[u8]> { self.attrs.keys().map(|k| k.as_slice()).collect() }
    pub fn len(&self) -> usize { self.attrs.len() }
    pub fn is_empty(&self) -> bool { self.attrs.is_empty() }
    pub fn version(&self) -> u64 { self.version }
    pub fn total_sets(&self) -> u64 { self.total_sets }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn total_snapshots(&self) -> u64 { self.total_snapshots }
}

impl Snapshot {
    pub fn get(&self, key: &[u8]) -> Option<(&[u8], u64)> {
        self.data.get(key).map(|(v, ver)| (v.as_slice(), *ver))
    }
    pub fn version(&self) -> u64 { self.version }
    pub fn len(&self) -> usize { self.data.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get() {
        let mut am = AttrMap::new();
        am.set(b"name", b"alice".to_vec());
        let (v, ver) = am.get(b"name").unwrap();
        assert_eq!(v, b"alice");
        assert_eq!(ver, 1);
    }

    #[test]
    fn overwrite_version() {
        let mut am = AttrMap::new();
        am.set(b"k", b"v1".to_vec());
        am.set(b"k", b"v2".to_vec());
        let (v, ver) = am.get(b"k").unwrap();
        assert_eq!(v, b"v2");
        assert_eq!(ver, 2);
    }

    #[test]
    fn delete() {
        let mut am = AttrMap::new();
        am.set(b"k", b"v".to_vec());
        let (v, ver) = am.delete(b"k").unwrap();
        assert_eq!(v, b"v");
        assert!(!am.contains(b"k"));
    }

    #[test]
    fn delete_missing() { assert!(AttrMap::new().delete(b"x").is_none()); }

    #[test]
    fn snapshot_restore() {
        let mut am = AttrMap::new();
        am.set(b"a", b"1".to_vec());
        let snap = am.snapshot();
        am.set(b"a", b"2".to_vec());
        am.restore(snap);
        assert_eq!(am.get(b"a").unwrap().0, b"1");
    }

    #[test]
    fn snapshot_get() {
        let mut am = AttrMap::new();
        am.set(b"x", b"y".to_vec());
        let snap = am.snapshot();
        assert_eq!(snap.get(b"x").unwrap().0, b"y");
    }

    #[test]
    fn keys() {
        let mut am = AttrMap::new();
        am.set(b"a", vec![]); am.set(b"b", vec![]);
        assert_eq!(am.keys().len(), 2);
    }

    #[test]
    fn version_monotone() {
        let mut am = AttrMap::new();
        am.set(b"a", vec![]);
        am.set(b"b", vec![]);
        assert_eq!(am.version(), 2);
    }

    #[test]
    fn stats() {
        let mut am = AttrMap::new();
        am.set(b"a", vec![]);
        am.delete(b"a");
        am.snapshot();
        assert_eq!(am.total_sets(), 1);
        assert_eq!(am.total_deletes(), 1);
        assert_eq!(am.total_snapshots(), 1);
    }
}

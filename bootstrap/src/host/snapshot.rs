use std::collections::BTreeMap;

pub struct Snapshot<T: Clone + PartialEq> {
    versions: BTreeMap<u64, Vec<T>>,
    current: u64,
    total_writes: u64,
    total_reads: u64,
    total_snapshots: u64,
}

impl<T: Clone + PartialEq> Snapshot<T> {
    pub fn new(data: Vec<T>) -> Self { Self { versions: [(0, data)].into_iter().collect(), current: 0, total_writes: 0, total_reads: 0, total_snapshots: 0 } }

    pub fn write(&mut self, index: usize, value: T) {
        self.total_writes += 1;
        self.current += 1;
        let mut data = self.versions.get(&(self.current - 1)).cloned().unwrap();
        if index < data.len() { data[index] = value; }
        self.versions.insert(self.current, data);
    }

    pub fn read(&mut self, version: u64, index: usize) -> Option<&T> {
        self.total_reads += 1;
        self.versions.get(&version).and_then(|v| v.get(index))
    }

    pub fn snapshot(&mut self) -> u64 {
        self.total_snapshots += 1;
        let data = self.versions.get(&self.current).cloned().unwrap();
        self.current += 1;
        self.versions.insert(self.current, data);
        self.current
    }

    pub fn rollback(&mut self, version: u64) -> bool {
        if self.versions.contains_key(&version) { self.current = version; true } else { false }
    }

    pub fn branch(&mut self, from_version: u64) -> Option<u64> {
        let data = self.versions.get(&from_version)?.clone();
        self.current += 1;
        self.versions.insert(self.current, data);
        Some(self.current)
    }

    pub fn diff(&mut self, v1: u64, v2: u64) -> Vec<(usize, Option<&T>, Option<&T>)> {
        self.total_reads += 1;
        let d1 = self.versions.get(&v1);
        let d2 = self.versions.get(&v2);
        match (d1, d2) {
            (Some(a), Some(b)) => {
                let mut result = Vec::new();
                for i in 0..a.len().max(b.len()) {
                    let old = a.get(i);
                    let new = b.get(i);
                    if old != new { result.push((i, old, new)); }
                }
                result
            }
            _ => Vec::new(),
        }
    }

    pub fn current_version(&self) -> u64 { self.current }
    pub fn version_count(&self) -> usize { self.versions.len() }
    pub fn len(&self) -> usize { self.versions.get(&self.current).map(|v| v.len()).unwrap_or(0) }
    pub fn total_writes(&self) -> u64 { self.total_writes }
    pub fn total_reads(&self) -> u64 { self.total_reads }
    pub fn total_snapshots(&self) -> u64 { self.total_snapshots }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write() {
        let mut s = Snapshot::new(vec![1u64, 2, 3]);
        s.write(1, 99);
        assert_eq!(s.read(0, 1), Some(&2));
        assert_eq!(s.read(s.current_version(), 1), Some(&99));
    }

    #[test]
    fn snapshot_rollback() {
        let mut s = Snapshot::new(vec![10u64]);
        s.write(0, 20);
        let snap = s.snapshot();
        s.write(0, 30);
        assert!(s.rollback(snap));
        assert_eq!(s.read(s.current_version(), 0), Some(&20));
    }

    #[test]
    fn branch() {
        let mut s = Snapshot::new(vec![1u64]);
        s.write(0, 2);
        let br = s.branch(0).unwrap();
        assert_eq!(s.read(br, 0), Some(&1));
    }

    #[test]
    fn diff() {
        let mut s = Snapshot::new(vec![1u64, 2, 3]);
        let v1 = s.current_version();
        s.write(1, 99);
        let v2 = s.current_version();
        let d = s.diff(v1, v2);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].0, 1);
    }

    #[test]
    fn rollback_missing() { assert!(!Snapshot::new(vec![1u64]).rollback(999)); }

    #[test]
    fn version_count() {
        let mut s = Snapshot::new(vec![1u64]);
        s.write(0, 2); s.write(0, 3);
        assert_eq!(s.version_count(), 3);
    }

    #[test]
    fn stats() {
        let mut s = Snapshot::new(vec![1u64]);
        s.write(0, 2);
        s.read(0, 0);
        s.snapshot();
        assert_eq!(s.total_writes(), 1);
        assert_eq!(s.total_reads(), 1);
        assert_eq!(s.total_snapshots(), 1);
    }
}

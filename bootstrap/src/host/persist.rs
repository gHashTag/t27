use std::collections::BTreeMap;

pub struct Persist {
    versions: Vec<BTreeMap<usize, u64>>,
    base: Vec<u64>,
    total_writes: u64,
    total_reads: u64,
}

impl Persist {
    pub fn new(data: Vec<u64>) -> Self {
        let v0 = BTreeMap::new();
        Self { versions: vec![v0], base: data, total_writes: 0, total_reads: 0 }
    }

    pub fn snapshot(&mut self) -> usize {
        let v = self.versions.len() - 1;
        let patch = self.versions[v].clone();
        self.versions.push(patch);
        self.versions.len() - 1
    }

    pub fn get(&mut self, ver: usize, idx: usize) -> Option<u64> {
        self.total_reads += 1;
        if ver >= self.versions.len() || idx >= self.base.len() { return None; }
        Some(*self.versions[ver].get(&idx).unwrap_or(&self.base[idx]))
    }

    pub fn set(&mut self, ver: usize, idx: usize, val: u64) -> bool {
        self.total_writes += 1;
        if ver >= self.versions.len() || idx >= self.base.len() { return false; }
        self.versions[ver].insert(idx, val);
        true
    }

    pub fn version_count(&self) -> usize { self.versions.len() }
    pub fn len(&self) -> usize { self.base.len() }
    pub fn total_writes(&self) -> u64 { self.total_writes }
    pub fn total_reads(&self) -> u64 { self.total_reads }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write() {
        let mut p = Persist::new(vec![1, 2, 3]);
        assert_eq!(p.get(0, 1), Some(2));
        p.set(0, 1, 99);
        assert_eq!(p.get(0, 1), Some(99));
    }

    #[test]
    fn snapshot_isolation() {
        let mut p = Persist::new(vec![10, 20, 30]);
        let v1 = p.snapshot();
        p.set(v1, 0, 100);
        assert_eq!(p.get(0, 0), Some(10));
        assert_eq!(p.get(v1, 0), Some(100));
    }

    #[test]
    fn multiple_versions() {
        let mut p = Persist::new(vec![0, 0, 0]);
        let v1 = p.snapshot();
        p.set(v1, 0, 1);
        let v2 = p.snapshot();
        p.set(v2, 0, 2);
        assert_eq!(p.get(0, 0), Some(0));
        assert_eq!(p.get(v1, 0), Some(1));
        assert_eq!(p.get(v2, 0), Some(2));
    }

    #[test]
    fn out_of_bounds() {
        let mut p = Persist::new(vec![1, 2, 3]);
        assert_eq!(p.get(0, 5), None);
        assert!(!p.set(0, 5, 1));
    }

    #[test]
    fn stats() {
        let mut p = Persist::new(vec![1, 2, 3]);
        p.get(0, 0); p.set(0, 0, 10);
        assert_eq!(p.total_reads(), 1);
        assert_eq!(p.total_writes(), 1);
    }
}

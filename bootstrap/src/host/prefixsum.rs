#[derive(Debug, Clone, PartialEq)]
pub enum PsError {
    IndexOutOfRange { idx: usize, len: usize },
}

impl std::fmt::Display for PsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PsError::IndexOutOfRange { idx, len } => write!(f, "index {idx} out of range (len={len})"),
        }
    }
}

impl std::error::Error for PsError {}

pub struct PrefixSum {
    data: Vec<i64>,
    prefix: Vec<i64>,
    dirty: bool,
    total_queries: u64,
    total_updates: u64,
    total_snapshots: u64,
}

impl PrefixSum {
    pub fn new(len: usize) -> Self { Self { data: vec![0; len], prefix: vec![0; len], dirty: false, total_queries: 0, total_updates: 0, total_snapshots: 0 } }

    pub fn from_values(values: &[i64]) -> Self {
        let mut ps = Self { data: values.to_vec(), prefix: vec![0; values.len()], dirty: true, total_queries: 0, total_updates: 0, total_snapshots: 0 };
        ps.rebuild();
        ps.dirty = false;
        ps
    }

    fn rebuild(&mut self) {
        if self.data.is_empty() { return; }
        self.prefix[0] = self.data[0];
        for i in 1..self.data.len() { self.prefix[i] = self.prefix[i - 1] + self.data[i]; }
    }

    pub fn set(&mut self, idx: usize, val: i64) -> Result<(), PsError> {
        if idx >= self.data.len() { return Err(PsError::IndexOutOfRange { idx, len: self.data.len() }); }
        self.total_updates += 1;
        self.data[idx] = val;
        self.dirty = true;
    }

    pub fn add(&mut self, idx: usize, delta: i64) -> Result<(), PsError> {
        if idx >= self.data.len() { return Err(PsError::IndexOutOfRange { idx, len: self.data.len() }); }
        self.total_updates += 1;
        self.data[idx] += delta;
        self.dirty = true;
        Ok(())
    }

    pub fn query(&mut self, idx: usize) -> i64 {
        self.total_queries += 1;
        if self.dirty { self.rebuild(); self.dirty = false; }
        if idx >= self.data.len() { return *self.prefix.last().unwrap_or(&0); }
        self.prefix[idx]
    }

    pub fn range_sum(&mut self, l: usize, r: usize) -> i64 {
        if l == 0 { self.query(r) } else { self.query(r) - self.query(l - 1) }
    }

    pub fn lower_bound(&mut self, target: i64) -> usize {
        self.total_queries += 1;
        if self.dirty { self.rebuild(); self.dirty = false; }
        match self.prefix.binary_search(&target) {
            Ok(idx) => idx,
            Err(idx) => idx.min(self.data.len()),
        }
    }

    pub fn snapshot(&mut self) -> Vec<i64> {
        self.total_snapshots += 1;
        if self.dirty { self.rebuild(); self.dirty = false; }
        self.prefix.clone()
    }

    pub fn get(&self, idx: usize) -> i64 { *self.data.get(idx).unwrap_or(&0) }
    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn is_dirty(&self) -> bool { self.dirty }
    pub fn total_queries(&self) -> u64 { self.total_queries }
    pub fn total_updates(&self) -> u64 { self.total_updates }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ps() { let ps = PrefixSum::new(5); assert_eq!(ps.len(), 5); }

    #[test]
    fn from_values() {
        let mut ps = PrefixSum::from_values(&[1, 2, 3, 4, 5]);
        assert_eq!(ps.query(4), 15);
        assert_eq!(ps.query(2), 6);
    }

    #[test]
    fn range_sum() {
        let mut ps = PrefixSum::from_values(&[3, 1, 4, 1, 5]);
        assert_eq!(ps.range_sum(0, 4), 14);
        assert_eq!(ps.range_sum(1, 3), 6);
    }

    #[test]
    fn update() {
        let mut ps = PrefixSum::from_values(&[1, 2, 3]);
        ps.set(1, 10).unwrap();
        assert_eq!(ps.query(2), 14);
    }

    #[test]
    fn add() {
        let mut ps = PrefixSum::from_values(&[1, 2, 3]);
        ps.add(0, 5).unwrap();
        assert_eq!(ps.query(0), 6);
        assert_eq!(ps.query(2), 11);
    }

    #[test]
    fn lower_bound() {
        let mut ps = PrefixSum::from_values(&[1, 3, 6, 10, 15]);
        assert_eq!(ps.lower_bound(7), 3);
        assert_eq!(ps.lower_bound(1), 0);
    }

    #[test]
    fn snapshot() {
        let mut ps = PrefixSum::from_values(&[1, 2, 3]);
        let snap = ps.snapshot();
        assert_eq!(snap, vec![1, 3, 6]);
    }

    #[test]
    fn dirty_tracking() {
        let mut ps = PrefixSum::from_values(&[1, 2, 3]);
        assert!(!ps.is_dirty());
        ps.add(0, 1).unwrap();
        assert!(ps.is_dirty());
        ps.query(0);
        assert!(!ps.is_dirty());
    }

    #[test]
    fn out_of_range() {
        let mut ps = PrefixSum::new(3);
        assert!(ps.set(5, 1).is_err());
    }

    #[test]
    fn error_display() { assert!(PsError::IndexOutOfRange { idx: 5, len: 3 }.to_string().contains("5")); }
}

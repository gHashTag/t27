use std::collections::BTreeMap;

pub struct SparseVec {
    data: BTreeMap<usize, f64>,
    dim: usize,
    total_ops: u64,
}

impl SparseVec {
    pub fn new(dim: usize) -> Self { Self { data: BTreeMap::new(), dim, total_ops: 0 } }

    pub fn set(&mut self, idx: usize, val: f64) {
        assert!(idx < self.dim);
        self.total_ops += 1;
        if val == 0.0 { self.data.remove(&idx); } else { self.data.insert(idx, val); }
    }

    pub fn get(&mut self, idx: usize) -> f64 {
        assert!(idx < self.dim);
        self.total_ops += 1;
        *self.data.get(&idx).unwrap_or(&0.0)
    }

    pub fn dot(&mut self, other: &SparseVec) -> f64 {
        self.total_ops += 1;
        let mut sum = 0.0;
        let (small, big) = if self.data.len() < other.data.len() { (&self.data, &other.data) } else { (&other.data, &self.data) };
        for (&idx, &v) in small { if let Some(&ov) = big.get(&idx) { sum += v * ov; } }
        sum
    }

    pub fn add(&mut self, other: &SparseVec) -> SparseVec {
        self.total_ops += 1;
        let mut result = SparseVec::new(self.dim.max(other.dim));
        for (&i, &v) in &self.data { result.data.insert(i, v); }
        for (&i, &v) in &other.data { *result.data.entry(i).or_insert(0.0) += v; }
        result
    }

    pub fn scale(&mut self, s: f64) {
        self.total_ops += 1;
        for v in self.data.values_mut() { *v *= s; }
    }

    pub fn nnz(&self) -> usize { self.data.len() }
    pub fn dim(&self) -> usize { self.dim }
    pub fn total_ops(&self) -> u64 { self.total_ops }
    pub fn density(&self) -> f64 { self.data.len() as f64 / self.dim.max(1) as f64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get() {
        let mut sv = SparseVec::new(10);
        sv.set(3, 4.0);
        assert!((sv.get(3) - 4.0).abs() < 1e-9);
        assert!((sv.get(0)).abs() < 1e-9);
    }

    #[test]
    fn dot() {
        let mut a = SparseVec::new(10);
        let mut b = SparseVec::new(10);
        a.set(0, 1.0); a.set(2, 3.0);
        b.set(0, 2.0); b.set(2, 4.0); b.set(5, 1.0);
        let d = a.dot(&b);
        assert!((d - 14.0).abs() < 1e-9);
    }

    #[test]
    fn add() {
        let mut a = SparseVec::new(10);
        let mut b = SparseVec::new(10);
        a.set(1, 2.0); b.set(1, 3.0); b.set(2, 1.0);
        let mut c = a.add(&b);
        assert!((c.get(1) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn scale() {
        let mut sv = SparseVec::new(10);
        sv.set(0, 2.0); sv.scale(3.0);
        assert!((sv.get(0) - 6.0).abs() < 1e-9);
    }

    #[test]
    fn remove_zero() {
        let mut sv = SparseVec::new(10);
        sv.set(1, 5.0); sv.set(1, 0.0);
        assert_eq!(sv.nnz(), 0);
    }

    #[test]
    fn density() {
        let mut sv = SparseVec::new(100);
        sv.set(0, 1.0); sv.set(50, 2.0);
        assert!((sv.density() - 0.02).abs() < 1e-9);
    }

    #[test]
    fn stats() {
        let mut sv = SparseVec::new(10);
        sv.set(0, 1.0); sv.get(0);
        assert_eq!(sv.total_ops(), 2);
    }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum StError {
    IndexOutOfRange { idx: usize, len: usize },
    EmptyTree,
}

impl std::fmt::Display for StError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StError::IndexOutOfRange { idx, len } => write!(f, "index {idx} out of range (len={len})"),
            StError::EmptyTree => write!(f, "empty tree"),
        }
    }
}

impl std::error::Error for StError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AggOp { Sum, Min, Max }

pub struct SegTree {
    data: BTreeMap<usize, i64>,
    n: usize,
    op: AggOp,
    total_updates: u64,
}

impl SegTree {
    pub fn new(values: &[i64], op: AggOp) -> Self {
        let n = values.len();
        let mut st = Self { data: BTreeMap::new(), n, op, total_updates: 0 };
        if n > 0 { st.build(values, 1, 0, n - 1); }
        st
    }

    fn build(&mut self, values: &[i64], node: usize, lo: usize, hi: usize) {
        if lo == hi {
            self.data.insert(node, values[lo]);
        } else {
            let mid = (lo + hi) / 2;
            self.build(values, node * 2, lo, mid);
            self.build(values, node * 2 + 1, mid + 1, hi);
            self.data.insert(node, self.combine(node * 2, node * 2 + 1));
        }
    }

    fn combine(&self, left: usize, right: usize) -> i64 {
        let l = self.data.get(&left).copied().unwrap_or(0);
        let r = self.data.get(&right).copied().unwrap_or(0);
        match self.op {
            AggOp::Sum => l + r,
            AggOp::Min => if self.data.contains_key(&left) && self.data.contains_key(&right) { l.min(r) } else { l + r },
            AggOp::Max => if self.data.contains_key(&left) && self.data.contains_key(&right) { l.max(r) } else { l + r },
        }
    }

    fn identity(&self) -> i64 { match self.op { AggOp::Sum => 0, AggOp::Min => i64::MAX, AggOp::Max => i64::MIN } }

    pub fn update(&mut self, idx: usize, value: i64) -> Result<(), StError> {
        if idx >= self.n { return Err(StError::IndexOutOfRange { idx, len: self.n }); }
        self.total_updates += 1;
        self.update_inner(1, 0, self.n - 1, idx, value);
        Ok(())
    }

    fn update_inner(&mut self, node: usize, lo: usize, hi: usize, idx: usize, value: i64) {
        if lo == hi {
            self.data.insert(node, value);
        } else {
            let mid = (lo + hi) / 2;
            if idx <= mid { self.update_inner(node * 2, lo, mid, idx, value); }
            else { self.update_inner(node * 2 + 1, mid + 1, hi, idx, value); }
            self.data.insert(node, self.combine(node * 2, node * 2 + 1));
        }
    }

    pub fn query(&self, from: usize, to: usize) -> Result<i64, StError> {
        if self.n == 0 { return Err(StError::EmptyTree); }
        if from > to || to >= self.n { return Err(StError::IndexOutOfRange { idx: from, len: self.n }); }
        Ok(self.query_inner(1, 0, self.n - 1, from, to))
    }

    fn query_inner(&self, node: usize, lo: usize, hi: usize, from: usize, to: usize) -> i64 {
        if from > hi || to < lo { return self.identity(); }
        if from <= lo && to >= hi { return self.data.get(&node).copied().unwrap_or(self.identity()); }
        let mid = (lo + hi) / 2;
        let l = self.query_inner(node * 2, lo, mid, from, to);
        let r = self.query_inner(node * 2 + 1, mid + 1, hi, from, to);
        match self.op {
            AggOp::Sum => l + r,
            AggOp::Min => l.min(r),
            AggOp::Max => l.max(r),
        }
    }

    pub fn get(&self, idx: usize) -> Option<i64> {
        if idx >= self.n { return None; }
        let mut node = 1; let mut lo = 0; let mut hi = self.n - 1;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if idx <= mid { node *= 2; hi = mid; } else { node = node * 2 + 1; lo = mid + 1; }
        }
        self.data.get(&node).copied()
    }

    pub fn len(&self) -> usize { self.n }
    pub fn is_empty(&self) -> bool { self.n == 0 }
    pub fn total_updates(&self) -> u64 { self.total_updates }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_tree() {
        let st = SegTree::new(&[1, 2, 3, 4, 5], AggOp::Sum);
        assert_eq!(st.query(0, 4), Ok(15));
        assert_eq!(st.query(1, 3), Ok(9));
    }

    #[test]
    fn min_tree() {
        let st = SegTree::new(&[5, 3, 7, 1, 4], AggOp::Min);
        assert_eq!(st.query(0, 4), Ok(1));
        assert_eq!(st.query(0, 2), Ok(3));
    }

    #[test]
    fn max_tree() {
        let st = SegTree::new(&[1, 5, 3, 8, 2], AggOp::Max);
        assert_eq!(st.query(0, 4), Ok(8));
        assert_eq!(st.query(2, 3), Ok(8));
    }

    #[test]
    fn update() {
        let mut st = SegTree::new(&[1, 2, 3, 4], AggOp::Sum);
        st.update(1, 10).unwrap();
        assert_eq!(st.query(0, 3), Ok(18));
        assert_eq!(st.get(1), Some(10));
    }

    #[test]
    fn point_query() {
        let st = SegTree::new(&[10, 20, 30], AggOp::Sum);
        assert_eq!(st.get(0), Some(10));
        assert_eq!(st.get(2), Some(30));
    }

    #[test]
    fn out_of_range() {
        let mut st = SegTree::new(&[1, 2, 3], AggOp::Sum);
        assert!(st.query(0, 5).is_err());
        assert!(st.update(10, 1).is_err());
    }

    #[test]
    fn empty_tree() {
        let st: SegTree = SegTree::new(&[], AggOp::Sum);
        assert!(st.is_empty());
        assert!(st.query(0, 0).is_err());
    }

    #[test]
    fn single_element() {
        let mut st = SegTree::new(&[42], AggOp::Sum);
        assert_eq!(st.query(0, 0), Ok(42));
        st.update(0, 100).unwrap();
        assert_eq!(st.query(0, 0), Ok(100));
    }

    #[test]
    fn stats() {
        let mut st = SegTree::new(&[1, 2, 3], AggOp::Sum);
        st.update(0, 10).unwrap();
        st.query(0, 2).unwrap();
        assert_eq!(st.total_updates(), 1);
    }

    #[test]
    fn error_display() { assert!(StError::EmptyTree.to_string().contains("empty")); }
}

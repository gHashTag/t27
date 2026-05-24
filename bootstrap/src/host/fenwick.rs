#[derive(Debug, Clone, PartialEq)]
pub enum FwError {
    IndexOutOfRange { idx: usize, len: usize },
}

impl std::fmt::Display for FwError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FwError::IndexOutOfRange { idx, len } => write!(f, "index {idx} out of range (len={len})"),
        }
    }
}

impl std::error::Error for FwError {}

pub struct Fenwick {
    tree: Vec<i64>,
    len: usize,
    total_updates: u64,
    total_queries: u64,
}

impl Fenwick {
    pub fn new(len: usize) -> Self {
        Self { tree: vec![0; len + 1], len, total_updates: 0, total_queries: 0 }
    }

    pub fn from_values(values: &[i64]) -> Self {
        let mut f = Self::new(values.len());
        for (i, &v) in values.iter().enumerate() { f.update(i, v).unwrap(); }
        f
    }

    pub fn update(&mut self, idx: usize, delta: i64) -> Result<(), FwError> {
        if idx >= self.len { return Err(FwError::IndexOutOfRange { idx, len: self.len }); }
        self.total_updates += 1;
        let mut i = idx + 1;
        while i <= self.len {
            self.tree[i] += delta;
            i += i & i.wrapping_neg();
        }
        Ok(())
    }

    pub fn prefix_sum(&mut self, idx: usize) -> i64 {
        self.total_queries += 1;
        let mut sum: i64 = 0;
        let mut i = (idx + 1).min(self.len + 1);
        while i > 0 {
            sum += self.tree[i];
            i -= i & i.wrapping_neg();
        }
        sum
    }

    pub fn range_sum(&mut self, l: usize, r: usize) -> i64 {
        if l == 0 { self.prefix_sum(r) } else { self.prefix_sum(r) - self.prefix_sum(l - 1) }
    }

    pub fn point_query(&mut self, idx: usize) -> i64 {
        if idx == 0 { self.prefix_sum(0) } else { self.prefix_sum(idx) - self.prefix_sum(idx - 1) }
    }

    pub fn find_kth(&mut self, k: i64) -> Option<usize> {
        self.total_queries += 1;
        if k <= 0 { return None; }
        let mut idx = 0usize;
        let mut bit_mask = 1usize << (self.len.next_power_of_two().trailing_zeros());
        let mut remaining = k;
        while bit_mask != 0 {
            let next = idx + bit_mask;
            if next <= self.len && self.tree[next] < remaining {
                remaining -= self.tree[next];
                idx = next;
            }
            bit_mask >>= 1;
        }
        if idx < self.len { Some(idx) } else { None }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn total_updates(&self) -> u64 { self.total_updates }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_fw() { let f = Fenwick::new(10); assert_eq!(f.len(), 10); }

    #[test]
    fn from_values() {
        let mut f = Fenwick::from_values(&[1, 2, 3, 4, 5]);
        assert_eq!(f.prefix_sum(4), 15);
        assert_eq!(f.range_sum(1, 3), 9);
    }

    #[test]
    fn update_query() {
        let mut f = Fenwick::new(5);
        f.update(2, 10).unwrap(); f.update(4, 5).unwrap();
        assert_eq!(f.prefix_sum(4), 15);
        assert_eq!(f.prefix_sum(2), 10);
    }

    #[test]
    fn range_sum() {
        let mut f = Fenwick::from_values(&[3, 1, 4, 1, 5]);
        assert_eq!(f.range_sum(0, 4), 14);
        assert_eq!(f.range_sum(2, 4), 10);
    }

    #[test]
    fn point_query() {
        let mut f = Fenwick::from_values(&[10, 20, 30]);
        assert_eq!(f.point_query(0), 10);
        assert_eq!(f.point_query(1), 20);
        assert_eq!(f.point_query(2), 30);
    }

    #[test]
    fn negative() {
        let mut f = Fenwick::from_values(&[5, -3, 2]);
        assert_eq!(f.range_sum(0, 2), 4);
    }

    #[test]
    fn out_of_range() {
        let mut f = Fenwick::new(3);
        assert!(f.update(5, 1).is_err());
    }

    #[test]
    fn find_kth() {
        let mut f = Fenwick::from_values(&[2, 0, 3, 0, 1]);
        assert_eq!(f.find_kth(1), Some(0));
        assert_eq!(f.find_kth(3), Some(2));
        assert_eq!(f.find_kth(6), Some(4));
    }

    #[test]
    fn find_kth_none() {
        let mut f = Fenwick::from_values(&[1, 1]);
        assert_eq!(f.find_kth(0), None);
    }

    #[test]
    fn stats() {
        let mut f = Fenwick::new(3);
        f.update(0, 1).unwrap(); f.prefix_sum(2);
        assert_eq!(f.total_updates(), 1);
        assert_eq!(f.total_queries(), 1);
    }

    #[test]
    fn error_display() { assert!(FwError::IndexOutOfRange { idx: 5, len: 3 }.to_string().contains("5")); }
}

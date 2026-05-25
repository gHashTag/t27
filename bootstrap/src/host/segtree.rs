pub struct SegTree {
    sum: Vec<i64>,
    mn: Vec<i64>,
    mx: Vec<i64>,
    n: usize,
    total_updates: u64,
    total_queries: u64,
}

impl SegTree {
    pub fn new(n: usize) -> Self {
        let size = n.next_power_of_two() * 2;
        Self { sum: vec![0; size], mn: vec![i64::MAX; size], mx: vec![i64::MIN; size], n, total_updates: 0, total_queries: 0 }
    }

    fn from_slice(data: &[i64]) -> Self {
        let n = data.len();
        let size = n.next_power_of_two() * 2;
        let mut st = Self { sum: vec![0; size], mn: vec![i64::MAX; size], mx: vec![i64::MIN; size], n, total_updates: 0, total_queries: 0 };
        let offset = size / 2;
        for i in 0..n { st.sum[offset + i] = data[i]; st.mn[offset + i] = data[i]; st.mx[offset + i] = data[i]; }
        for i in (1..offset).rev() {
            st.sum[i] = st.sum[2 * i] + st.sum[2 * i + 1];
            st.mn[i] = st.mn[2 * i].min(st.mn[2 * i + 1]);
            st.mx[i] = st.mx[2 * i].max(st.mx[2 * i + 1]);
        }
        st
    }

    pub fn update(&mut self, idx: usize, val: i64) {
        self.total_updates += 1;
        let offset = self.sum.len() / 2;
        let mut i = offset + idx;
        self.sum[i] = val; self.mn[i] = val; self.mx[i] = val;
        while i > 1 {
            i /= 2;
            self.sum[i] = self.sum[2 * i] + self.sum[2 * i + 1];
            self.mn[i] = self.mn[2 * i].min(self.mn[2 * i + 1]);
            self.mx[i] = self.mx[2 * i].max(self.mx[2 * i + 1]);
        }
    }

    pub fn range_sum(&mut self, lo: usize, hi: usize) -> i64 {
        self.total_queries += 1;
        self.query(lo, hi, 1, 0, self.sum.len() / 2).0
    }

    pub fn range_min(&mut self, lo: usize, hi: usize) -> i64 {
        self.total_queries += 1;
        self.query(lo, hi, 1, 0, self.sum.len() / 2).1
    }

    pub fn range_max(&mut self, lo: usize, hi: usize) -> i64 {
        self.total_queries += 1;
        self.query(lo, hi, 1, 0, self.sum.len() / 2).2
    }

    fn query(&self, lo: usize, hi: usize, node: usize, nl: usize, nr: usize) -> (i64, i64, i64) {
        if hi <= nl || nr <= lo { return (0, i64::MAX, i64::MIN); }
        if lo <= nl && nr <= hi { return (self.sum[node], self.mn[node], self.mx[node]); }
        let mid = (nl + nr) / 2;
        let l = self.query(lo, hi, 2 * node, nl, mid);
        let r = self.query(lo, hi, 2 * node + 1, mid, nr);
        (l.0 + r.0, l.1.min(r.1), l.2.max(r.2))
    }

    pub fn len(&self) -> usize { self.n }
    pub fn total_updates(&self) -> u64 { self.total_updates }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_slice_sum() {
        let data = [1i64, 2, 3, 4, 5];
        let mut st = SegTree::from_slice(&data);
        assert_eq!(st.range_sum(0, 5), 15);
        assert_eq!(st.range_sum(2, 4), 7);
    }

    #[test]
    fn range_min_max() {
        let data = [5i64, 3, 8, 1, 4];
        let mut st = SegTree::from_slice(&data);
        assert_eq!(st.range_min(0, 5), 1);
        assert_eq!(st.range_max(0, 5), 8);
        assert_eq!(st.range_min(1, 3), 3);
    }

    #[test]
    fn update() {
        let data = [1i64, 2, 3];
        let mut st = SegTree::from_slice(&data);
        st.update(1, 10);
        assert_eq!(st.range_sum(0, 3), 14);
        assert_eq!(st.range_max(0, 3), 10);
    }

    #[test]
    fn empty_range() {
        let mut st = SegTree::new(5);
        assert_eq!(st.range_sum(0, 0), 0);
    }

    #[test]
    fn single() {
        let mut st = SegTree::new(1);
        st.update(0, 42);
        assert_eq!(st.range_sum(0, 1), 42);
    }

    #[test]
    fn stats() {
        let mut st = SegTree::new(3);
        st.update(0, 1); st.range_sum(0, 3);
        assert_eq!(st.total_updates(), 1);
        assert_eq!(st.total_queries(), 1);
    }
}

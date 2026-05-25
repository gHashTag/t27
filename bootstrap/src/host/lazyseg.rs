pub struct LazySeg {
    sum: Vec<i64>,
    lazy: Vec<i64>,
    n: usize,
    total_updates: u64,
    total_queries: u64,
}

impl LazySeg {
    pub fn new(n: usize) -> Self {
        let size = n.next_power_of_two() * 4;
        Self { sum: vec![0; size], lazy: vec![0; size], n, total_updates: 0, total_queries: 0 }
    }

    fn push(&mut self, node: usize, lo: usize, hi: usize) {
        if self.lazy[node] != 0 {
            let mid = (lo + hi) / 2;
            self.apply(node * 2, lo, mid, self.lazy[node]);
            self.apply(node * 2 + 1, mid, hi, self.lazy[node]);
            self.lazy[node] = 0;
        }
    }

    fn apply(&mut self, node: usize, lo: usize, hi: usize, val: i64) {
        self.sum[node] += val * (hi - lo) as i64;
        self.lazy[node] += val;
    }

    pub fn range_add(&mut self, lo: usize, hi: usize, val: i64) {
        self.total_updates += 1;
        self.update(lo, hi, val, 1, 0, self.n);
    }

    fn update(&mut self, lo: usize, hi: usize, val: i64, node: usize, nl: usize, nh: usize) {
        if lo >= nh || hi <= nl { return; }
        if lo <= nl && nh <= hi { self.apply(node, nl, nh, val); return; }
        self.push(node, nl, nh);
        let mid = (nl + nh) / 2;
        self.update(lo, hi, val, node * 2, nl, mid);
        self.update(lo, hi, val, node * 2 + 1, mid, nh);
        self.sum[node] = self.sum[node * 2] + self.sum[node * 2 + 1];
    }

    pub fn range_sum(&mut self, lo: usize, hi: usize) -> i64 {
        self.total_queries += 1;
        self.query(lo, hi, 1, 0, self.n)
    }

    fn query(&mut self, lo: usize, hi: usize, node: usize, nl: usize, nh: usize) -> i64 {
        if lo >= nh || hi <= nl { return 0; }
        if lo <= nl && nh <= hi { return self.sum[node]; }
        self.push(node, nl, nh);
        let mid = (nl + nh) / 2;
        self.query(lo, hi, node * 2, nl, mid) + self.query(lo, hi, node * 2 + 1, mid, nh)
    }

    pub fn len(&self) -> usize { self.n }
    pub fn total_updates(&self) -> u64 { self.total_updates }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_add_sum() {
        let mut ls = LazySeg::new(10);
        ls.range_add(0, 5, 1);
        assert_eq!(ls.range_sum(0, 5), 5);
        assert_eq!(ls.range_sum(0, 10), 5);
    }

    #[test]
    fn overlapping() {
        let mut ls = LazySeg::new(10);
        ls.range_add(0, 10, 1);
        ls.range_add(3, 7, 2);
        assert_eq!(ls.range_sum(0, 10), 18);
        assert_eq!(ls.range_sum(3, 7), 12);
    }

    #[test]
    fn point() {
        let mut ls = LazySeg::new(10);
        ls.range_add(5, 6, 42);
        assert_eq!(ls.range_sum(5, 6), 42);
        assert_eq!(ls.range_sum(0, 5), 0);
    }

    #[test]
    fn full_range() {
        let mut ls = LazySeg::new(5);
        ls.range_add(0, 5, 3);
        assert_eq!(ls.range_sum(0, 5), 15);
    }

    #[test]
    fn empty_range() {
        let mut ls = LazySeg::new(5);
        assert_eq!(ls.range_sum(0, 0), 0);
    }

    #[test]
    fn stats() {
        let mut ls = LazySeg::new(5);
        ls.range_add(0, 5, 1); ls.range_sum(0, 5);
        assert_eq!(ls.total_updates(), 1);
        assert_eq!(ls.total_queries(), 1);
    }
}

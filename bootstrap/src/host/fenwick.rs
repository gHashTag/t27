pub struct Fenwick {
    tree: Vec<i64>,
    n: usize,
    total_updates: u64,
    total_queries: u64,
}

impl Fenwick {
    pub fn new(n: usize) -> Self { Self { tree: vec![0; n + 1], n, total_updates: 0, total_queries: 0 } }

    pub fn update(&mut self, mut i: usize, delta: i64) {
        self.total_updates += 1;
        i += 1;
        while i <= self.n { self.tree[i] += delta; i += i & i.wrapping_neg(); }
    }

    pub fn prefix_sum(&mut self, mut i: usize) -> i64 {
        self.total_queries += 1;
        let mut sum = 0i64;
        while i > 0 { sum += self.tree[i]; i -= i & i.wrapping_neg(); }
        sum
    }

    pub fn range_sum(&mut self, lo: usize, hi: usize) -> i64 {
        self.prefix_sum(hi) - self.prefix_sum(lo)
    }

    pub fn get(&mut self, i: usize) -> i64 { self.range_sum(i, i + 1) }

    pub fn len(&self) -> usize { self.n }
    pub fn total_updates(&self) -> u64 { self.total_updates }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_sum() {
        let mut f = Fenwick::new(10);
        f.update(0, 1); f.update(1, 2); f.update(2, 3);
        assert_eq!(f.prefix_sum(3), 6);
    }

    #[test]
    fn range_sum() {
        let mut f = Fenwick::new(10);
        for i in 0..10 { f.update(i, (i + 1) as i64); }
        assert_eq!(f.range_sum(2, 5), 3 + 4 + 5);
    }

    #[test]
    fn point_get() {
        let mut f = Fenwick::new(5);
        f.update(2, 42);
        assert_eq!(f.get(2), 42);
        assert_eq!(f.get(0), 0);
    }

    #[test]
    fn negative() {
        let mut f = Fenwick::new(5);
        f.update(0, 10); f.update(0, -5);
        assert_eq!(f.get(0), 5);
    }

    #[test]
    fn empty_range() {
        let mut f = Fenwick::new(5);
        assert_eq!(f.range_sum(0, 0), 0);
    }

    #[test]
    fn stats() {
        let mut f = Fenwick::new(5);
        f.update(0, 1); f.prefix_sum(1);
        assert_eq!(f.total_updates(), 1);
        assert_eq!(f.total_queries(), 1);
    }
}

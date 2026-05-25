pub struct DiffArray {
    diff: Vec<i64>,
    n: usize,
}

impl DiffArray {
    pub fn new(n: usize) -> Self {
        Self { diff: vec![0; n + 1], n }
    }

    pub fn from(values: &[i64]) -> Self {
        let n = values.len();
        let mut diff = vec![0i64; n + 1];
        diff[0] = values[0];
        for i in 1..n { diff[i] = values[i] - values[i - 1]; }
        Self { diff, n }
    }

    pub fn range_add(&mut self, l: usize, r: usize, val: i64) {
        if l >= self.n { return; }
        self.diff[l] += val;
        if r + 1 <= self.n { self.diff[r + 1] -= val; }
    }

    pub fn reconstruct(&self) -> Vec<i64> {
        let mut result = Vec::with_capacity(self.n);
        let mut acc = 0i64;
        for i in 0..self.n {
            acc += self.diff[i];
            result.push(acc);
        }
        result
    }

    pub fn point_query(&self, idx: usize) -> i64 {
        if idx >= self.n { return 0; }
        let mut acc = 0i64;
        for i in 0..=idx { acc += self.diff[i]; }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_reconstruct() {
        let da = DiffArray::from(&[1, 2, 3, 4, 5]);
        assert_eq!(da.reconstruct(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn range_add() {
        let mut da = DiffArray::new(5);
        da.range_add(1, 3, 10);
        assert_eq!(da.reconstruct(), vec![0, 10, 10, 10, 0]);
    }

    #[test]
    fn multiple_adds() {
        let mut da = DiffArray::from(&[1, 1, 1, 1, 1]);
        da.range_add(0, 2, 5);
        da.range_add(2, 4, 3);
        assert_eq!(da.reconstruct(), vec![6, 6, 9, 4, 4]);
    }

    #[test]
    fn point_query() {
        let mut da = DiffArray::new(5);
        da.range_add(1, 3, 7);
        assert_eq!(da.point_query(0), 0);
        assert_eq!(da.point_query(2), 7);
        assert_eq!(da.point_query(4), 0);
    }

    #[test]
    fn full_range() {
        let mut da = DiffArray::new(4);
        da.range_add(0, 3, 1);
        assert_eq!(da.reconstruct(), vec![1, 1, 1, 1]);
    }

    #[test]
    fn empty() {
        let da = DiffArray::new(0);
        assert!(da.reconstruct().is_empty());
    }
}

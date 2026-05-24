#[derive(Debug, Clone, PartialEq)]
pub enum SegOp {
    Sum,
    Min,
    Max,
}

#[derive(Debug, Clone)]
pub struct SegmentTree {
    data: Vec<i64>,
    size: usize,
    op: SegOp,
}

impl SegmentTree {
    pub fn new(size: usize, op: SegOp) -> Self {
        let data = vec![0i64; size * 2];
        Self { data, size, op }
    }

    pub fn from_slice(values: &[i64], op: SegOp) -> Self {
        let n = values.len();
        let mut data = vec![0i64; n * 2];
        data[n..n + n].copy_from_slice(values);
        let mut st = Self { data, size: n, op };
        for i in (1..n).rev() {
            st.data[i] = st.combine(st.data[i * 2], st.data[i * 2 + 1]);
        }
        st
    }

    fn combine(&self, a: i64, b: i64) -> i64 {
        match self.op {
            SegOp::Sum => a + b,
            SegOp::Min => a.min(b),
            SegOp::Max => a.max(b),
        }
    }

    fn identity(&self) -> i64 {
        match self.op {
            SegOp::Sum => 0,
            SegOp::Min => i64::MAX,
            SegOp::Max => i64::MIN,
        }
    }

    pub fn update(&mut self, index: usize, value: i64) {
        assert!(index < self.size);
        let mut i = index + self.size;
        self.data[i] = value;
        while i > 1 {
            i /= 2;
            self.data[i] = self.combine(self.data[i * 2], self.data[i * 2 + 1]);
        }
    }

    pub fn query(&self, left: usize, right: usize) -> i64 {
        assert!(left <= right && right <= self.size);
        let mut l = left + self.size;
        let mut r = right + self.size;
        let mut result = self.identity();
        while l < r {
            if l % 2 == 1 {
                result = self.combine(result, self.data[l]);
                l += 1;
            }
            if r % 2 == 1 {
                r -= 1;
                result = self.combine(result, self.data[r]);
            }
            l /= 2;
            r /= 2;
        }
        result
    }

    pub fn get(&self, index: usize) -> i64 {
        assert!(index < self.size);
        self.data[index + self.size]
    }

    pub fn len(&self) -> usize { self.size }

    pub fn is_empty(&self) -> bool { self.size == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sum_tree() {
        let st = SegmentTree::new(8, SegOp::Sum);
        assert_eq!(st.len(), 8);
        assert_eq!(st.query(0, 8), 0);
    }

    #[test]
    fn from_slice_sum() {
        let st = SegmentTree::from_slice(&[1, 2, 3, 4, 5], SegOp::Sum);
        assert_eq!(st.query(0, 5), 15);
        assert_eq!(st.query(1, 4), 9);
    }

    #[test]
    fn point_update() {
        let mut st = SegmentTree::from_slice(&[1, 2, 3, 4, 5], SegOp::Sum);
        st.update(2, 10);
        assert_eq!(st.get(2), 10);
        assert_eq!(st.query(0, 5), 22);
    }

    #[test]
    fn range_sum() {
        let mut st = SegmentTree::new(4, SegOp::Sum);
        st.update(0, 10); st.update(1, 20); st.update(2, 30); st.update(3, 40);
        assert_eq!(st.query(1, 3), 50);
    }

    #[test]
    fn min_tree() {
        let st = SegmentTree::from_slice(&[5, 3, 7, 1, 4], SegOp::Min);
        assert_eq!(st.query(0, 5), 1);
        assert_eq!(st.query(0, 3), 3);
        assert_eq!(st.query(3, 5), 1);
    }

    #[test]
    fn max_tree() {
        let st = SegmentTree::from_slice(&[5, 3, 7, 1, 4], SegOp::Max);
        assert_eq!(st.query(0, 5), 7);
        assert_eq!(st.query(0, 3), 7);
        assert_eq!(st.query(3, 5), 4);
    }

    #[test]
    fn single_element() {
        let st = SegmentTree::from_slice(&[42], SegOp::Sum);
        assert_eq!(st.query(0, 1), 42);
    }

    #[test]
    fn update_min() {
        let mut st = SegmentTree::from_slice(&[5, 3, 7, 1, 4], SegOp::Min);
        st.update(2, 0);
        assert_eq!(st.query(0, 5), 0);
    }

    #[test]
    fn update_max() {
        let mut st = SegmentTree::from_slice(&[5, 3, 7, 1, 4], SegOp::Max);
        st.update(3, 100);
        assert_eq!(st.query(0, 5), 100);
    }

    #[test]
    fn empty_range() {
        let st = SegmentTree::from_slice(&[1, 2, 3], SegOp::Sum);
        assert_eq!(st.query(1, 1), 0);
    }

    #[test]
    fn is_empty() {
        let st = SegmentTree::new(0, SegOp::Sum);
        assert!(st.is_empty());
    }

    #[test]
    fn full_round_trip() {
        let mut st = SegmentTree::new(4, SegOp::Sum);
        for i in 0..4 { st.update(i, (i + 1) as i64 * 10); }
        assert_eq!(st.query(0, 4), 100);
        assert_eq!(st.get(2), 30);
    }
}

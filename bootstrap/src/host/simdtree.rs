const LANE: usize = 8;

pub struct SimdTree {
    nodes: Vec<[i64; LANE]>,
    total_inserts: u64,
    total_queries: u64,
}

impl SimdTree {
    pub fn new() -> Self { Self { nodes: Vec::new(), total_inserts: 0, total_queries: 0 } }

    pub fn insert(&mut self, values: [i64; LANE]) {
        self.total_inserts += 1;
        self.nodes.push(values);
    }

    pub fn batch_min(&mut self) -> Option<[i64; LANE]> {
        self.total_queries += 1;
        if self.nodes.is_empty() { return None; }
        let mut result = [i64::MAX; LANE];
        for node in &self.nodes {
            for i in 0..LANE { if node[i] < result[i] { result[i] = node[i]; } }
        }
        Some(result)
    }

    pub fn batch_max(&mut self) -> Option<[i64; LANE]> {
        self.total_queries += 1;
        if self.nodes.is_empty() { return None; }
        let mut result = [i64::MIN; LANE];
        for node in &self.nodes {
            for i in 0..LANE { if node[i] > result[i] { result[i] = node[i]; } }
        }
        Some(result)
    }

    pub fn batch_sum(&mut self) -> Option<[i64; LANE]> {
        self.total_queries += 1;
        if self.nodes.is_empty() { return None; }
        let mut result = [0i64; LANE];
        for node in &self.nodes {
            for i in 0..LANE { result[i] += node[i]; }
        }
        Some(result)
    }

    pub fn lane_filter(&mut self, lane: usize, threshold: i64) -> Vec<usize> {
        self.total_queries += 1;
        self.nodes.iter().enumerate().filter(|(_, n)| n[lane] >= threshold).map(|(i, _)| i).collect()
    }

    pub fn lane_sort(&mut self, lane: usize) -> Vec<usize> {
        self.total_queries += 1;
        let mut indices: Vec<usize> = (0..self.nodes.len()).collect();
        indices.sort_by_key(|&i| self.nodes[i][lane]);
        indices
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn lane_count(&self) -> usize { LANE }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_min() {
        let mut st = SimdTree::new();
        st.insert([1, 2, 3, 4, 5, 6, 7, 8]);
        st.insert([8, 7, 6, 5, 4, 3, 2, 1]);
        let min = st.batch_min().unwrap();
        assert_eq!(min[0], 1);
        assert_eq!(min[1], 2);
        assert_eq!(min[7], 1);
    }

    #[test]
    fn batch_max() {
        let mut st = SimdTree::new();
        st.insert([1, 2, 3, 4, 5, 6, 7, 8]);
        st.insert([8, 7, 6, 5, 4, 3, 2, 1]);
        let max = st.batch_max().unwrap();
        assert_eq!(max[0], 8);
        assert_eq!(max[7], 8);
    }

    #[test]
    fn batch_sum() {
        let mut st = SimdTree::new();
        st.insert([1, 0, 0, 0, 0, 0, 0, 0]);
        st.insert([2, 0, 0, 0, 0, 0, 0, 0]);
        let sum = st.batch_sum().unwrap();
        assert_eq!(sum[0], 3);
    }

    #[test]
    fn lane_filter() {
        let mut st = SimdTree::new();
        st.insert([10, 0, 0, 0, 0, 0, 0, 0]);
        st.insert([5, 0, 0, 0, 0, 0, 0, 0]);
        st.insert([15, 0, 0, 0, 0, 0, 0, 0]);
        let f = st.lane_filter(0, 10);
        assert_eq!(f, vec![0, 2]);
    }

    #[test]
    fn lane_sort() {
        let mut st = SimdTree::new();
        st.insert([3, 0, 0, 0, 0, 0, 0, 0]);
        st.insert([1, 0, 0, 0, 0, 0, 0, 0]);
        st.insert([2, 0, 0, 0, 0, 0, 0, 0]);
        let s = st.lane_sort(0);
        assert_eq!(s, vec![1, 2, 0]);
    }

    #[test]
    fn empty_min() { assert!(SimdTree::new().batch_min().is_none()); }

    #[test]
    fn node_count() {
        let mut st = SimdTree::new();
        st.insert([0; 8]); st.insert([0; 8]);
        assert_eq!(st.node_count(), 2);
    }

    #[test]
    fn stats() {
        let mut st = SimdTree::new();
        st.insert([0; 8]);
        st.batch_min();
        assert_eq!(st.total_inserts(), 1);
        assert_eq!(st.total_queries(), 1);
    }
}

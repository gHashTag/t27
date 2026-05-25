pub struct MultiRes {
    levels: Vec<Vec<f64>>,
    total_builds: u64,
    total_queries: u64,
}

impl MultiRes {
    pub fn new(data: Vec<f64>) -> Self {
        let mut levels = vec![data];
        while levels.last().unwrap().len() > 1 {
            let prev = levels.last().unwrap();
            let half: Vec<f64> = prev.chunks(2).map(|c| c.iter().sum::<f64>() / c.len().max(1) as f64).collect();
            levels.push(half);
        }
        Self { levels, total_builds: 1, total_queries: 0 }
    }

    pub fn query(&mut self, level: usize) -> Option<&[f64]> {
        self.total_queries += 1;
        self.levels.get(level).map(|v| v.as_slice())
    }

    pub fn reconstruct(&mut self, level: usize) -> Vec<f64> {
        self.total_queries += 1;
        let mut result = Vec::new();
        if let Some(l) = self.levels.get(level) {
            let factor = 1 << level;
            for &v in l { for _ in 0..factor { result.push(v); } }
        }
        result.truncate(self.levels[0].len());
        result
    }

    pub fn levels(&self) -> usize { self.levels.len() }
    pub fn base_len(&self) -> usize { self.levels.first().map(|v| v.len()).unwrap_or(0) }
    pub fn total_builds(&self) -> u64 { self.total_builds }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_query() {
        let mr = MultiRes::new(vec![1.0, 3.0, 5.0, 7.0]);
        assert!(mr.levels() > 1);
    }

    #[test]
    fn level0() {
        let mut mr = MultiRes::new(vec![1.0, 3.0, 5.0, 7.0]);
        assert_eq!(mr.query(0).unwrap().len(), 4);
    }

    #[test]
    fn level1_avg() {
        let mut mr = MultiRes::new(vec![2.0, 6.0, 4.0, 8.0]);
        let l1 = mr.query(1).unwrap();
        assert!((l1[0] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn reconstruct() {
        let mut mr = MultiRes::new(vec![1.0, 3.0, 5.0, 7.0]);
        let r = mr.reconstruct(1);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn single_element() {
        let mr = MultiRes::new(vec![42.0]);
        assert_eq!(mr.levels(), 1);
    }

    #[test]
    fn stats() {
        let mut mr = MultiRes::new(vec![1.0, 2.0]);
        mr.query(0); mr.reconstruct(1);
        assert_eq!(mr.total_queries(), 2);
    }
}

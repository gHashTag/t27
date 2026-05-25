pub struct CountMin {
    table: Vec<Vec<u64>>,
    width: usize,
    depth: usize,
    total_updates: u64,
    total_queries: u64,
}

impl CountMin {
    pub fn new(width: usize, depth: usize) -> Self {
        Self { table: (0..depth).map(|_| vec![0; width]).collect(), width: width.max(1), depth: depth.max(1), total_updates: 0, total_queries: 0 }
    }

    fn hashes(&self, key: u64) -> Vec<usize> {
        (0..self.depth).map(|d| {
            let h = key.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(d as u64);
            ((h >> 32) as usize) % self.width
        }).collect()
    }

    pub fn update(&mut self, key: u64, count: u64) {
        self.total_updates += 1;
        for (d, idx) in self.hashes(key).iter().enumerate() {
            self.table[d][*idx] += count;
        }
    }

    pub fn estimate(&mut self, key: u64) -> u64 {
        self.total_queries += 1;
        self.hashes(key).iter().enumerate().map(|(d, idx)| self.table[d][*idx]).min().unwrap_or(0)
    }

    pub fn total_updates(&self) -> u64 { self.total_updates }
    pub fn total_queries(&self) -> u64 { self.total_queries }
    pub fn width(&self) -> usize { self.width }
    pub fn depth(&self) -> usize { self.depth }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_key() {
        let mut cm = CountMin::new(64, 4);
        cm.update(42, 10);
        assert!(cm.estimate(42) >= 10);
    }

    #[test]
    fn estimate_upper_bound() {
        let mut cm = CountMin::new(64, 5);
        for i in 0..100u64 { cm.update(i, 1); }
        assert!(cm.estimate(50) <= 5);
    }

    #[test]
    fn heavy_hitter() {
        let mut cm = CountMin::new(128, 5);
        for _ in 0..1000 { cm.update(7, 1); }
        for i in 100..200u64 { cm.update(i, 1); }
        assert!(cm.estimate(7) >= 900);
    }

    #[test]
    fn zero_key() {
        let mut cm = CountMin::new(64, 3);
        assert_eq!(cm.estimate(99), 0);
    }

    #[test]
    fn multi_update() {
        let mut cm = CountMin::new(64, 3);
        cm.update(1, 5); cm.update(1, 3);
        assert!(cm.estimate(1) >= 8);
    }

    #[test]
    fn stats() {
        let mut cm = CountMin::new(64, 3);
        cm.update(1, 1); cm.estimate(1);
        assert_eq!(cm.total_updates(), 1);
        assert_eq!(cm.total_queries(), 1);
    }
}

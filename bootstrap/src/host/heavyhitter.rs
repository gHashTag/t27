use std::collections::BTreeMap;

pub struct HeavyHitter {
    counters: BTreeMap<u64, u64>,
    cap: usize,
    total_updates: u64,
    total_queries: u64,
}

impl HeavyHitter {
    pub fn new(cap: usize) -> Self { Self { counters: BTreeMap::new(), cap: cap.max(1), total_updates: 0, total_queries: 0 } }

    pub fn update(&mut self, item: u64) {
        self.total_updates += 1;
        if let Some(c) = self.counters.get_mut(&item) { *c += 1; return; }
        if self.counters.len() < self.cap { self.counters.insert(item, 1); return; }
        let (min_key, min_val) = self.counters.iter().min_by_key(|(_, &v)| v).map(|(&k, &v)| (k, v)).unwrap();
        self.counters.remove(&min_key);
        self.counters.insert(item, min_val + 1);
    }

    pub fn estimate(&mut self, item: u64) -> u64 {
        self.total_queries += 1;
        *self.counters.get(&item).unwrap_or(&0)
    }

    pub fn top(&mut self, k: usize) -> Vec<(u64, u64)> {
        self.total_queries += 1;
        let mut v: Vec<_> = self.counters.iter().map(|(&i, &c)| (i, c)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v.into_iter().take(k).collect()
    }

    pub fn len(&self) -> usize { self.counters.len() }
    pub fn total_updates(&self) -> u64 { self.total_updates }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heavy() {
        let mut hh = HeavyHitter::new(3);
        for _ in 0..100 { hh.update(1); }
        for i in 2..200u64 { hh.update(i); }
        assert!(hh.estimate(1) > 50);
    }

    #[test]
    fn top_k() {
        let mut hh = HeavyHitter::new(10);
        for _ in 0..50 { hh.update(1); }
        for _ in 0..30 { hh.update(2); }
        for _ in 0..10 { hh.update(3); }
        let t = hh.top(2);
        assert_eq!(t[0].0, 1);
        assert_eq!(t[1].0, 2);
    }

    #[test]
    fn missing() { assert_eq!(HeavyHitter::new(5).estimate(99), 0); }

    #[test]
    fn overflow() {
        let mut hh = HeavyHitter::new(2);
        hh.update(1); hh.update(2); hh.update(3);
        assert_eq!(hh.len(), 2);
    }

    #[test]
    fn space_saving() {
        let mut hh = HeavyHitter::new(3);
        hh.update(1); hh.update(1); hh.update(1);
        hh.update(2);
        hh.update(3);
        assert!(hh.estimate(1) >= 3);
    }

    #[test]
    fn stats() {
        let mut hh = HeavyHitter::new(5);
        hh.update(1); hh.estimate(1);
        assert_eq!(hh.total_updates(), 1);
        assert_eq!(hh.total_queries(), 1);
    }
}

use std::collections::BTreeMap;

pub struct RingSampler<T: Clone> {
    reservoir: Vec<T>,
    capacity: usize,
    count: u64,
    weights: Vec<f64>,
    total_weight: f64,
    total_observed: u64,
    total_sampled: u64,
}

impl<T: Clone> RingSampler<T> {
    pub fn new(capacity: usize) -> Self {
        Self { reservoir: Vec::with_capacity(capacity), capacity, count: 0, weights: Vec::new(), total_weight: 0.0, total_observed: 0, total_sampled: 0 }
    }

    pub fn observe(&mut self, item: T) {
        self.total_observed += 1;
        self.count += 1;
        if self.reservoir.len() < self.capacity {
            self.reservoir.push(item);
            self.weights.push(1.0);
        } else {
            let idx = self.count as usize % self.capacity;
            if idx < self.reservoir.len() {
                self.reservoir[idx] = item;
                self.weights[idx] = 1.0;
            }
        }
    }

    pub fn observe_weighted(&mut self, item: T, weight: f64) {
        self.total_observed += 1;
        self.total_weight += weight;
        self.count += 1;
        if self.reservoir.len() < self.capacity {
            self.reservoir.push(item);
            self.weights.push(weight);
        } else if weight > 0.0 {
            let prob = weight / self.total_weight;
            if prob >= 1.0 / self.count as f64 {
                let idx = self.count as usize % self.capacity;
                if idx < self.reservoir.len() {
                    self.reservoir[idx] = item;
                    self.weights[idx] = weight;
                }
            }
        }
    }

    pub fn sample(&mut self) -> Option<&T> {
        if self.reservoir.is_empty() { return None; }
        self.total_sampled += 1;
        let idx = (self.count as usize) % self.reservoir.len();
        self.reservoir.get(idx)
    }

    pub fn snapshot(&self) -> Vec<T> { self.reservoir.clone() }

    pub fn reset(&mut self) {
        self.reservoir.clear();
        self.weights.clear();
        self.count = 0;
        self.total_weight = 0.0;
    }

    pub fn len(&self) -> usize { self.reservoir.len() }
    pub fn is_empty(&self) -> bool { self.reservoir.is_empty() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn total_observed(&self) -> u64 { self.total_observed }
    pub fn total_sampled(&self) -> u64 { self.total_sampled }
    pub fn observed_count(&self) -> u64 { self.count }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sampler() { let s: RingSampler<i32> = RingSampler::new(5); assert!(s.is_empty()); }

    #[test]
    fn observe_fill() {
        let mut s = RingSampler::new(3);
        s.observe(1); s.observe(2); s.observe(3);
        assert_eq!(s.len(), 3);
        let snap = s.snapshot();
        assert!(snap.contains(&1));
        assert!(snap.contains(&2));
        assert!(snap.contains(&3));
    }

    #[test]
    fn observe_overflow() {
        let mut s = RingSampler::new(2);
        s.observe(1); s.observe(2); s.observe(3); s.observe(4);
        assert_eq!(s.len(), 2);
        assert_eq!(s.total_observed(), 4);
    }

    #[test]
    fn sample() {
        let mut s = RingSampler::new(5);
        s.observe(10); s.observe(20); s.observe(30);
        let v = s.sample();
        assert!(v.is_some());
        assert_eq!(s.total_sampled(), 1);
    }

    #[test]
    fn sample_empty() {
        let mut s: RingSampler<i32> = RingSampler::new(5);
        assert!(s.sample().is_none());
    }

    #[test]
    fn weighted_observe() {
        let mut s = RingSampler::new(3);
        s.observe_weighted(1, 10.0); s.observe_weighted(2, 1.0); s.observe_weighted(3, 1.0);
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn reset() {
        let mut s = RingSampler::new(5);
        s.observe(1); s.observe(2);
        s.reset();
        assert!(s.is_empty());
        assert_eq!(s.observed_count(), 0);
    }

    #[test]
    fn snapshot_preserves() {
        let mut s = RingSampler::new(5);
        s.observe(1); s.observe(2);
        let snap = s.snapshot();
        s.observe(99);
        assert_eq!(snap.len(), 2);
    }

    #[test]
    fn capacity() {
        let s: RingSampler<i32> = RingSampler::new(100);
        assert_eq!(s.capacity(), 100);
    }

    #[test]
    fn stats() {
        let mut s = RingSampler::new(5);
        s.observe(1); s.observe(2);
        s.sample();
        assert_eq!(s.total_observed(), 2);
        assert_eq!(s.total_sampled(), 1);
    }
}

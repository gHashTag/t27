use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub struct TopK {
    k: usize,
    heap: BinaryHeap<Reverse<(u64, u64)>>,
    total_updates: u64,
    total_evictions: u64,
}

impl TopK {
    pub fn new(k: usize) -> Self { Self { k: k.max(1), heap: BinaryHeap::new(), total_updates: 0, total_evictions: 0 } }

    pub fn update(&mut self, item: u64, score: u64) {
        self.total_updates += 1;
        if self.heap.len() < self.k {
            self.heap.push(Reverse((score, item)));
        } else if let Some(Reverse((min_score, _))) = self.heap.peek() {
            if score > *min_score {
                self.heap.pop();
                self.heap.push(Reverse((score, item)));
                self.total_evictions += 1;
            }
        }
    }

    pub fn top(&self) -> Vec<(u64, u64)> {
        let mut v: Vec<_> = self.heap.iter().map(|Reverse((s, i))| (*i, *s)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    }

    pub fn len(&self) -> usize { self.heap.len() }
    pub fn is_empty(&self) -> bool { self.heap.is_empty() }
    pub fn k(&self) -> usize { self.k }
    pub fn total_updates(&self) -> u64 { self.total_updates }
    pub fn total_evictions(&self) -> u64 { self.total_evictions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top3() {
        let mut tk = TopK::new(3);
        for i in 1..=10u64 { tk.update(i, i * 10); }
        let t = tk.top();
        assert_eq!(t.len(), 3);
        assert_eq!(t[0], (10, 100));
    }

    #[test]
    fn fewer_than_k() {
        let mut tk = TopK::new(5);
        tk.update(1, 10); tk.update(2, 20);
        assert_eq!(tk.len(), 2);
    }

    #[test]
    fn eviction() {
        let mut tk = TopK::new(2);
        tk.update(1, 5); tk.update(2, 3);
        tk.update(3, 10);
        assert_eq!(tk.total_evictions(), 1);
        let t = tk.top();
        assert_eq!(t[0], (3, 10));
    }

    #[test]
    fn no_evict_lower() {
        let mut tk = TopK::new(2);
        tk.update(1, 10); tk.update(2, 5);
        tk.update(3, 3);
        assert_eq!(tk.total_evictions(), 0);
    }

    #[test]
    fn sorted_output() {
        let mut tk = TopK::new(3);
        tk.update(1, 30); tk.update(2, 10); tk.update(3, 20);
        let t = tk.top();
        assert_eq!(t[0].1, 30); assert_eq!(t[1].1, 20); assert_eq!(t[2].1, 10);
    }

    #[test]
    fn stats() {
        let mut tk = TopK::new(2);
        tk.update(1, 1); tk.update(2, 2);
        assert_eq!(tk.total_updates(), 2);
    }
}

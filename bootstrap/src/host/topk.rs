use std::collections::BinaryHeap;
use std::cmp::Reverse;

#[derive(Debug, Clone, PartialEq)]
pub struct TopKErr;

impl std::fmt::Display for TopKErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "invalid k") }
}
impl std::error::Error for TopKErr {}

pub struct TopK {
    heap: BinaryHeap<Reverse<(i64, u64)>>,
    k: usize,
    seq: u64,
    total_inserts: u64,
    total_evicts: u64,
    total_queries: u64,
}

impl TopK {
    pub fn new(k: usize) -> Self { Self { heap: BinaryHeap::with_capacity(k + 1), k, seq: 0, total_inserts: 0, total_evicts: 0, total_queries: 0 } }

    pub fn push(&mut self, val: i64) {
        self.total_inserts += 1;
        self.seq += 1;
        self.heap.push(Reverse((val, self.seq)));
        if self.heap.len() > self.k {
            self.heap.pop();
            self.total_evicts += 1;
        }
    }

    pub fn top_k(&mut self) -> Vec<i64> {
        self.total_queries += 1;
        let mut v: Vec<_> = self.heap.iter().map(|Reverse((v, _))| *v).collect();
        v.sort_by(|a, b| b.cmp(a));
        v
    }

    pub fn top_k_with_seq(&mut self) -> Vec<(i64, u64)> {
        self.total_queries += 1;
        let mut v: Vec<_> = self.heap.iter().map(|Reverse(pair)| *pair).collect();
        v.sort_by(|a, b| b.0.cmp(&a.0));
        v
    }

    pub fn min_val(&self) -> Option<i64> { self.heap.peek().map(|Reverse((v, _))| *v) }
    pub fn len(&self) -> usize { self.heap.len() }
    pub fn is_empty(&self) -> bool { self.heap.is_empty() }
    pub fn k(&self) -> usize { self.k }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_evicts(&self) -> u64 { self.total_evicts }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tk() { let tk = TopK::new(3); assert!(tk.is_empty()); assert_eq!(tk.k(), 3); }

    #[test]
    fn basic_topk() {
        let mut tk = TopK::new(3);
        for v in [5, 1, 9, 3, 7] { tk.push(v); }
        assert_eq!(tk.top_k(), vec![9, 7, 5]);
    }

    #[test]
    fn evict_smallest() {
        let mut tk = TopK::new(2);
        tk.push(1); tk.push(5); tk.push(3);
        assert_eq!(tk.top_k(), vec![5, 3]);
        assert_eq!(tk.total_evicts(), 1);
    }

    #[test]
    fn duplicates() {
        let mut tk = TopK::new(3);
        for v in [5, 5, 5, 5] { tk.push(v); }
        assert_eq!(tk.len(), 3);
    }

    #[test]
    fn min_val() {
        let mut tk = TopK::new(3);
        tk.push(10); tk.push(5); tk.push(8);
        assert_eq!(tk.min_val(), Some(5));
    }

    #[test]
    fn negative() {
        let mut tk = TopK::new(2);
        tk.push(-10); tk.push(5); tk.push(-3);
        assert_eq!(tk.top_k(), vec![5, -3]);
    }

    #[test]
    fn single() {
        let mut tk = TopK::new(5);
        tk.push(42);
        assert_eq!(tk.top_k(), vec![42]);
    }

    #[test]
    fn with_seq() {
        let mut tk = TopK::new(2);
        tk.push(1); tk.push(2);
        let items = tk.top_k_with_seq();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, 2);
    }

    #[test]
    fn stats() {
        let mut tk = TopK::new(2);
        tk.push(1); tk.push(2); tk.push(3); tk.top_k();
        assert_eq!(tk.total_inserts(), 3);
        assert_eq!(tk.total_evicts(), 1);
        assert_eq!(tk.total_queries(), 1);
    }
}

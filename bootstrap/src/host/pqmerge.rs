use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub struct PqMerge {
    heap: BinaryHeap<Reverse<(u64, usize, usize)>>,
    streams: Vec<Vec<u64>>,
    total_merged: u64,
}

impl PqMerge {
    pub fn new() -> Self { Self { heap: BinaryHeap::new(), streams: Vec::new(), total_merged: 0 } }

    pub fn add_stream(&mut self, stream: Vec<u64>) {
        let si = self.streams.len();
        self.streams.push(stream);
        if !self.streams[si].is_empty() {
            self.heap.push(Reverse((self.streams[si][0], si, 0)));
        }
    }

    pub fn next(&mut self) -> Option<u64> {
        let Reverse((val, si, ei)) = self.heap.pop()?;
        self.total_merged += 1;
        let next_ei = ei + 1;
        if next_ei < self.streams[si].len() {
            self.heap.push(Reverse((self.streams[si][next_ei], si, next_ei)));
        }
        Some(val)
    }

    pub fn merge_all(&mut self) -> Vec<u64> {
        let mut result = Vec::new();
        while let Some(v) = self.next() { result.push(v); }
        result
    }

    pub fn is_empty(&self) -> bool { self.heap.is_empty() }
    pub fn total_merged(&self) -> u64 { self.total_merged }
    pub fn stream_count(&self) -> usize { self.streams.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_streams() {
        let mut pq = PqMerge::new();
        pq.add_stream(vec![1, 3, 5]);
        pq.add_stream(vec![2, 4, 6]);
        assert_eq!(pq.merge_all(), vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn three_streams() {
        let mut pq = PqMerge::new();
        pq.add_stream(vec![1, 4, 7]);
        pq.add_stream(vec![2, 5, 8]);
        pq.add_stream(vec![3, 6, 9]);
        assert_eq!(pq.merge_all(), (1..=9).collect::<Vec<_>>());
    }

    #[test]
    fn empty_stream() {
        let mut pq = PqMerge::new();
        pq.add_stream(vec![]);
        pq.add_stream(vec![1, 2]);
        assert_eq!(pq.merge_all(), vec![1, 2]);
    }

    #[test]
    fn duplicates() {
        let mut pq = PqMerge::new();
        pq.add_stream(vec![1, 1, 2]);
        pq.add_stream(vec![1, 3]);
        assert_eq!(pq.merge_all(), vec![1, 1, 1, 2, 3]);
    }

    #[test]
    fn single_stream() {
        let mut pq = PqMerge::new();
        pq.add_stream(vec![10, 20, 30]);
        assert_eq!(pq.merge_all(), vec![10, 20, 30]);
    }

    #[test]
    fn next_one_by_one() {
        let mut pq = PqMerge::new();
        pq.add_stream(vec![5, 15]);
        pq.add_stream(vec![10]);
        assert_eq!(pq.next(), Some(5));
        assert_eq!(pq.next(), Some(10));
        assert_eq!(pq.next(), Some(15));
        assert_eq!(pq.next(), None);
    }

    #[test]
    fn stats() {
        let mut pq = PqMerge::new();
        pq.add_stream(vec![1, 2]); pq.add_stream(vec![3]);
        pq.merge_all();
        assert_eq!(pq.total_merged(), 3);
        assert_eq!(pq.stream_count(), 2);
    }
}

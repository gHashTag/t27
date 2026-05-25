use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Interval {
    pub lo: u64,
    pub hi: u64,
    pub tag: u64,
}

pub struct IntervalMap {
    intervals: Vec<Interval>,
    total_inserts: u64,
    total_queries: u64,
}

impl IntervalMap {
    pub fn new() -> Self { Self { intervals: Vec::new(), total_inserts: 0, total_queries: 0 } }

    pub fn insert(&mut self, lo: u64, hi: u64, tag: u64) {
        self.total_inserts += 1;
        self.intervals.push(Interval { lo, hi, tag });
    }

    pub fn query_point(&mut self, point: u64) -> Vec<&Interval> {
        self.total_queries += 1;
        self.intervals.iter().filter(|iv| point >= iv.lo && point <= iv.hi).collect()
    }

    pub fn query_range(&mut self, lo: u64, hi: u64) -> Vec<&Interval> {
        self.total_queries += 1;
        self.intervals.iter().filter(|iv| iv.lo <= hi && iv.hi >= lo).collect()
    }

    pub fn remove(&mut self, tag: u64) -> bool {
        let before = self.intervals.len();
        self.intervals.retain(|iv| iv.tag != tag);
        self.intervals.len() < before
    }

    pub fn len(&self) -> usize { self.intervals.len() }
    pub fn is_empty(&self) -> bool { self.intervals.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_query() {
        let mut im = IntervalMap::new();
        im.insert(10, 20, 1);
        assert_eq!(im.query_point(15).len(), 1);
        assert_eq!(im.query_point(5).len(), 0);
    }

    #[test]
    fn range_query() {
        let mut im = IntervalMap::new();
        im.insert(0, 10, 1); im.insert(5, 15, 2); im.insert(20, 30, 3);
        let r = im.query_range(8, 12);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn overlapping() {
        let mut im = IntervalMap::new();
        im.insert(0, 10, 1); im.insert(5, 15, 2); im.insert(10, 20, 3);
        let r = im.query_point(10);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn remove() {
        let mut im = IntervalMap::new();
        im.insert(0, 10, 1); im.insert(5, 15, 2);
        assert!(im.remove(1));
        assert_eq!(im.len(), 1);
        assert!(!im.remove(99));
    }

    #[test]
    fn boundary() {
        let mut im = IntervalMap::new();
        im.insert(0, 10, 1);
        assert_eq!(im.query_point(0).len(), 1);
        assert_eq!(im.query_point(10).len(), 1);
    }

    #[test]
    fn stats() {
        let mut im = IntervalMap::new();
        im.insert(0, 10, 1); im.query_point(5);
        assert_eq!(im.total_inserts(), 1);
        assert_eq!(im.total_queries(), 1);
    }
}

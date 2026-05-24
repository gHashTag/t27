use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ItError {
    IntervalExists { id: u64 },
    IntervalNotFound { id: u64 },
}

impl std::fmt::Display for ItError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItError::IntervalExists { id } => write!(f, "interval {id} exists"),
            ItError::IntervalNotFound { id } => write!(f, "interval {id} not found"),
        }
    }
}

impl std::error::Error for ItError {}

#[derive(Debug, Clone)]
pub struct Interval {
    pub id: u64,
    pub start: i64,
    pub end: i64,
    pub data: Vec<u8>,
}

pub struct ITree {
    intervals: BTreeMap<u64, Interval>,
    total_inserts: u64,
    total_queries: u64,
    total_overlaps: u64,
}

impl ITree {
    pub fn new() -> Self { Self { intervals: BTreeMap::new(), total_inserts: 0, total_queries: 0, total_overlaps: 0 } }

    pub fn insert(&mut self, id: u64, start: i64, end: i64, data: Vec<u8>) -> Result<(), ItError> {
        if self.intervals.contains_key(&id) { return Err(ItError::IntervalExists { id }); }
        if start > end { return Err(ItError::IntervalExists { id }); }
        self.intervals.insert(id, Interval { id, start, end, data });
        self.total_inserts += 1;
        Ok(())
    }

    pub fn remove(&mut self, id: u64) -> Result<Interval, ItError> {
        self.intervals.remove(&id).ok_or(ItError::IntervalNotFound { id })
    }

    pub fn query_point(&mut self, point: i64) -> Vec<&Interval> {
        self.total_queries += 1;
        self.intervals.values().filter(|i| point >= i.start && point <= i.end).collect()
    }

    pub fn query_range(&mut self, start: i64, end: i64) -> Vec<&Interval> {
        self.total_queries += 1;
        self.intervals.values().filter(|i| i.start <= end && i.end >= start).collect()
    }

    pub fn overlaps(&mut self, id: u64) -> Vec<u64> {
        self.total_overlaps += 1;
        if let Some(a) = self.intervals.get(&id) {
            let (s, e) = (a.start, a.end);
            self.intervals.values()
                .filter(|b| b.id != id && b.start <= e && b.end >= s)
                .map(|b| b.id)
                .collect()
        } else { Vec::new() }
    }

    pub fn get(&self, id: u64) -> Option<&Interval> { self.intervals.get(&id) }
    pub fn len(&self) -> usize { self.intervals.len() }
    pub fn is_empty(&self) -> bool { self.intervals.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_queries(&self) -> u64 { self.total_queries }
    pub fn total_overlaps(&self) -> u64 { self.total_overlaps }
}

impl Default for ITree {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tree() { assert!(ITree::new().is_empty()); }

    #[test]
    fn insert_get() {
        let mut t = ITree::new();
        t.insert(1, 10, 20, b"a".to_vec()).unwrap();
        let iv = t.get(1).unwrap();
        assert_eq!(iv.start, 10);
        assert_eq!(iv.end, 20);
    }

    #[test]
    fn query_point() {
        let mut t = ITree::new();
        t.insert(1, 10, 20, vec![]).unwrap();
        t.insert(2, 15, 25, vec![]).unwrap();
        t.insert(3, 30, 40, vec![]).unwrap();
        let hits = t.query_point(17);
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn query_range() {
        let mut t = ITree::new();
        t.insert(1, 10, 20, vec![]).unwrap();
        t.insert(2, 25, 35, vec![]).unwrap();
        let hits = t.query_range(18, 28);
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn overlaps() {
        let mut t = ITree::new();
        t.insert(1, 10, 20, vec![]).unwrap();
        t.insert(2, 15, 25, vec![]).unwrap();
        t.insert(3, 30, 40, vec![]).unwrap();
        let ov = t.overlaps(1);
        assert_eq!(ov, vec![2]);
    }

    #[test]
    fn no_overlap() {
        let mut t = ITree::new();
        t.insert(1, 10, 20, vec![]).unwrap();
        t.insert(2, 30, 40, vec![]).unwrap();
        assert!(t.overlaps(1).is_empty());
    }

    #[test]
    fn remove() {
        let mut t = ITree::new();
        t.insert(1, 10, 20, vec![]).unwrap();
        let iv = t.remove(1).unwrap();
        assert_eq!(iv.start, 10);
        assert!(t.is_empty());
    }

    #[test]
    fn duplicate() {
        let mut t = ITree::new();
        t.insert(1, 0, 10, vec![]).unwrap();
        let err = t.insert(1, 5, 15, vec![]).unwrap_err();
        assert!(matches!(err, ItError::IntervalExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut t = ITree::new();
        let err = t.remove(99).unwrap_err();
        assert!(matches!(err, ItError::IntervalNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut t = ITree::new();
        t.insert(1, 0, 10, vec![]).unwrap();
        t.query_point(5);
        assert_eq!(t.total_inserts(), 1);
        assert_eq!(t.total_queries(), 1);
    }

    #[test]
    fn error_display() { assert!(ItError::IntervalNotFound { id: 1 }.to_string().contains("1")); }
}

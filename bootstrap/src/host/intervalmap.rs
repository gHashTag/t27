use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntervalMapError {
    Overlap { start: u64, end: u64 },
    EmptyInterval,
    NotFound { start: u64 },
}

impl std::fmt::Display for IntervalMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntervalMapError::Overlap { start, end } => write!(f, "[{start},{end}) overlaps"),
            IntervalMapError::EmptyInterval => write!(f, "empty interval"),
            IntervalMapError::NotFound { start } => write!(f, "interval at {start} not found"),
        }
    }
}

impl std::error::Error for IntervalMapError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interval {
    pub start: u64,
    pub end: u64,
    pub tag: u64,
}

impl Interval {
    pub fn new(start: u64, end: u64, tag: u64) -> Self {
        Self { start, end, tag }
    }

    pub fn len(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr < self.end
    }

    pub fn overlaps(&self, other: &Interval) -> bool {
        self.start < other.end && other.start < self.end
    }
}

#[derive(Debug, Clone)]
pub struct IntervalMap {
    intervals: BTreeMap<u64, Interval>,
    total_inserts: u64,
    total_removes: u64,
    total_merges: u64,
}

impl IntervalMap {
    pub fn new() -> Self {
        Self { intervals: BTreeMap::new(), total_inserts: 0, total_removes: 0, total_merges: 0 }
    }

    pub fn insert(&mut self, start: u64, end: u64, tag: u64) -> Result<(), IntervalMapError> {
        if start >= end { return Err(IntervalMapError::EmptyInterval); }
        let new = Interval::new(start, end, tag);
        for iv in self.intervals.values() {
            if new.overlaps(iv) {
                return Err(IntervalMapError::Overlap { start, end });
            }
        }
        self.intervals.insert(start, new);
        self.total_inserts += 1;
        Ok(())
    }

    pub fn remove(&mut self, start: u64) -> Result<Interval, IntervalMapError> {
        self.intervals.remove(&start)
            .ok_or(IntervalMapError::NotFound { start })
            .map(|iv| { self.total_removes += 1; iv })
    }

    pub fn find(&self, addr: u64) -> Option<&Interval> {
        let mut candidate = None;
        for iv in self.intervals.values().rev() {
            if iv.start <= addr {
                candidate = Some(iv);
                break;
            }
        }
        candidate.filter(|iv| iv.contains(addr))
    }

    pub fn contains(&self, addr: u64) -> bool {
        self.find(addr).is_some()
    }

    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn total_coverage(&self) -> u64 {
        self.intervals.values().map(|iv| iv.len()).sum()
    }

    pub fn intervals(&self) -> Vec<&Interval> {
        self.intervals.values().collect()
    }

    pub fn merge_adjacent(&mut self) -> usize {
        let starts: Vec<u64> = self.intervals.keys().copied().collect();
        let mut count = 0;
        let mut i = 0;
        while i + 1 < starts.len() {
            let a = self.intervals.get(&starts[i]).cloned();
            let b = self.intervals.get(&starts[i + 1]).cloned();
            if let (Some(iv_a), Some(iv_b)) = (a, b) {
                if iv_a.end == iv_b.start && iv_a.tag == iv_b.tag {
                    let merged = Interval::new(iv_a.start, iv_b.end, iv_a.tag);
                    self.intervals.remove(&starts[i]);
                    self.intervals.remove(&starts[i + 1]);
                    self.intervals.insert(merged.start, merged);
                    count += 1;
                    self.total_merges += 1;
                    continue;
                }
            }
            i += 1;
        }
        count
    }

    pub fn split(&mut self, start: u64, at: u64) -> Result<(), IntervalMapError> {
        let iv = self.intervals.remove(&start)
            .ok_or(IntervalMapError::NotFound { start })?;
        if at <= iv.start || at >= iv.end {
            self.intervals.insert(start, iv);
            return Err(IntervalMapError::EmptyInterval);
        }
        let left = Interval::new(iv.start, at, iv.tag);
        let right = Interval::new(at, iv.end, iv.tag);
        self.intervals.insert(left.start, left);
        self.intervals.insert(right.start, right);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.intervals.clear();
    }

    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_merges(&self) -> u64 { self.total_merges }
}

impl Default for IntervalMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() {
        let im = IntervalMap::new();
        assert!(im.is_empty());
    }

    #[test]
    fn insert_and_find() {
        let mut im = IntervalMap::new();
        im.insert(0, 100, 1).unwrap();
        assert!(im.contains(50));
        assert!(!im.contains(100));
        assert_eq!(im.find(50).unwrap().tag, 1);
    }

    #[test]
    fn overlap_rejected() {
        let mut im = IntervalMap::new();
        im.insert(0, 100, 1).unwrap();
        let err = im.insert(50, 150, 2).unwrap_err();
        assert!(matches!(err, IntervalMapError::Overlap { .. }));
    }

    #[test]
    fn empty_interval_rejected() {
        let mut im = IntervalMap::new();
        let err = im.insert(100, 100, 1).unwrap_err();
        assert!(matches!(err, IntervalMapError::EmptyInterval));
    }

    #[test]
    fn remove() {
        let mut im = IntervalMap::new();
        im.insert(0, 100, 1).unwrap();
        let iv = im.remove(0).unwrap();
        assert_eq!(iv.end, 100);
        assert!(im.is_empty());
    }

    #[test]
    fn remove_not_found() {
        let mut im = IntervalMap::new();
        let err = im.remove(99).unwrap_err();
        assert!(matches!(err, IntervalMapError::NotFound { .. }));
    }

    #[test]
    fn multiple_intervals() {
        let mut im = IntervalMap::new();
        im.insert(0, 100, 1).unwrap();
        im.insert(200, 300, 2).unwrap();
        assert_eq!(im.len(), 2);
        assert_eq!(im.total_coverage(), 200);
    }

    #[test]
    fn merge_adjacent() {
        let mut im = IntervalMap::new();
        im.insert(0, 100, 1).unwrap();
        im.insert(100, 200, 1).unwrap();
        assert_eq!(im.merge_adjacent(), 1);
        assert_eq!(im.len(), 1);
        assert_eq!(im.find(150).unwrap().end, 200);
    }

    #[test]
    fn merge_different_tags() {
        let mut im = IntervalMap::new();
        im.insert(0, 100, 1).unwrap();
        im.insert(100, 200, 2).unwrap();
        assert_eq!(im.merge_adjacent(), 0);
    }

    #[test]
    fn split() {
        let mut im = IntervalMap::new();
        im.insert(0, 100, 1).unwrap();
        im.split(0, 50).unwrap();
        assert_eq!(im.len(), 2);
        assert_eq!(im.find(25).unwrap().end, 50);
        assert_eq!(im.find(75).unwrap().start, 50);
    }

    #[test]
    fn interval_overlaps() {
        let a = Interval::new(0, 10, 0);
        let b = Interval::new(5, 15, 0);
        let c = Interval::new(10, 20, 0);
        assert!(a.overlaps(&b));
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn error_display() {
        assert!(IntervalMapError::EmptyInterval.to_string().contains("empty"));
    }
}

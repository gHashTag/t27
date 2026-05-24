use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SnapRingError {
    Empty,
    InvalidIndex { requested: u64, min: u64, max: u64 },
}

impl std::fmt::Display for SnapRingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapRingError::Empty => write!(f, "snapshot ring empty"),
            SnapRingError::InvalidIndex { requested, min, max } => write!(f, "index {requested} out of range [{min},{max}]"),
        }
    }
}

impl std::error::Error for SnapRingError {}

pub struct SnapRing<V: Clone> {
    buffer: Vec<Option<(u64, V)>>,
    capacity: usize,
    head: usize,
    len: usize,
    seq: u64,
    total_appends: u64,
    total_reads: u64,
}

impl<V: Clone> SnapRing<V> {
    pub fn new(capacity: usize) -> Self {
        Self { buffer: vec![None; capacity], capacity, head: 0, len: 0, seq: 0, total_appends: 0, total_reads: 0 }
    }

    pub fn append(&mut self, value: V) -> u64 {
        let seq = self.seq;
        self.buffer[self.head] = Some((seq, value));
        self.head = (self.head + 1) % self.capacity;
        if self.len < self.capacity { self.len += 1; }
        self.seq += 1;
        self.total_appends += 1;
        seq
    }

    pub fn latest(&mut self) -> Option<(u64, V)> {
        self.total_reads += 1;
        if self.len == 0 { return None; }
        let idx = (self.head + self.capacity - 1) % self.capacity;
        self.buffer[idx].as_ref().map(|(s, v)| (*s, v.clone()))
    }

    pub fn get(&mut self, seq: u64) -> Option<(u64, V)> {
        self.total_reads += 1;
        for entry in &self.buffer {
            if let Some((s, v)) = entry {
                if *s == seq { return Some((*s, v.clone())); }
            }
        }
        None
    }

    pub fn range(&mut self, from_seq: u64, to_seq: u64) -> Vec<(u64, V)> {
        self.total_reads += 1;
        let mut result = Vec::new();
        for entry in &self.buffer {
            if let Some((s, v)) = entry {
                if *s >= from_seq && *s <= to_seq {
                    result.push((*s, v.clone()));
                }
            }
        }
        result.sort_by_key(|(s, _)| *s);
        result
    }

    pub fn diff<F>(&mut self, seq_a: u64, seq_b: u64, differ: F) -> Option<String>
    where F: Fn(&V, &V) -> String {
        let a = self.get(seq_a)?;
        let b = self.get(seq_b)?;
        Some(differ(&a.1, &b.1))
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn min_seq(&self) -> Option<u64> {
        for entry in &self.buffer {
            if let Some((s, _)) = entry { return Some(*s); }
        }
        None
    }
    pub fn max_seq(&self) -> Option<u64> { if self.seq > 0 { Some(self.seq - 1) } else { None } }
    pub fn total_appends(&self) -> u64 { self.total_appends }
    pub fn total_reads(&self) -> u64 { self.total_reads }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ring() { let sr: SnapRing<i32> = SnapRing::new(5); assert!(sr.is_empty()); }

    #[test]
    fn append_latest() {
        let mut sr = SnapRing::new(5);
        sr.append(10); sr.append(20); sr.append(30);
        let (seq, val) = sr.latest().unwrap();
        assert_eq!(val, 30);
    }

    #[test]
    fn wrap_around() {
        let mut sr = SnapRing::new(3);
        sr.append(1); sr.append(2); sr.append(3); sr.append(4);
        assert_eq!(sr.len(), 3);
        let (_, val) = sr.latest().unwrap();
        assert_eq!(val, 4);
    }

    #[test]
    fn get_by_seq() {
        let mut sr = SnapRing::new(5);
        let s0 = sr.append(10);
        let s1 = sr.append(20);
        assert_eq!(sr.get(s0), Some((s0, 10)));
        assert_eq!(sr.get(s1), Some((s1, 20)));
    }

    #[test]
    fn range_query() {
        let mut sr = SnapRing::new(10);
        sr.append(1); sr.append(2); sr.append(3); sr.append(4);
        let range = sr.range(1, 2);
        assert_eq!(range.len(), 2);
    }

    #[test]
    fn diff_snapshots() {
        let mut sr = SnapRing::new(5);
        let s0 = sr.append("hello".to_string());
        let s1 = sr.append("world".to_string());
        let d = sr.diff(s0, s1, |a, b| format!("{a}->{b}")).unwrap();
        assert_eq!(d, "hello->world");
    }

    #[test]
    fn get_missing() {
        let mut sr: SnapRing<i32> = SnapRing::new(5);
        assert!(sr.get(99).is_none());
    }

    #[test]
    fn seq_tracking() {
        let mut sr = SnapRing::new(5);
        sr.append(1); sr.append(2);
        assert_eq!(sr.min_seq(), Some(0));
        assert_eq!(sr.max_seq(), Some(1));
    }

    #[test]
    fn stats() {
        let mut sr = SnapRing::new(5);
        sr.append(1); sr.append(2);
        sr.latest(); sr.get(0);
        assert_eq!(sr.total_appends(), 2);
        assert_eq!(sr.total_reads(), 2);
    }

    #[test]
    fn error_display() { assert!(SnapRingError::Empty.to_string().contains("empty")); }
}

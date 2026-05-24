use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum HbError {
    BufferEmpty,
    InvalidRange { from: u64, to: u64 },
    SequenceGap { expected: u64, got: u64 },
}

impl std::fmt::Display for HbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HbError::BufferEmpty => write!(f, "buffer empty"),
            HbError::InvalidRange { from, to } => write!(f, "invalid range {from}..{to}"),
            HbError::SequenceGap { expected, got } => write!(f, "seq gap: expected {expected}, got {got}"),
        }
    }
}

impl std::error::Error for HbError {}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub seq: u64,
    pub timestamp: u64,
    pub data: Vec<u8>,
}

pub struct HistBuf {
    buffer: VecDeque<HistoryEntry>,
    capacity: usize,
    next_seq: u64,
    total_appended: u64,
    total_compacted: u64,
    total_range_queries: u64,
}

impl HistBuf {
    pub fn new(capacity: usize) -> Self {
        Self { buffer: VecDeque::with_capacity(capacity), capacity, next_seq: 1, total_appended: 0, total_compacted: 0, total_range_queries: 0 }
    }

    pub fn append(&mut self, timestamp: u64, data: Vec<u8>) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        if self.buffer.len() >= self.capacity { self.buffer.pop_front(); }
        self.buffer.push_back(HistoryEntry { seq, timestamp, data });
        self.total_appended += 1;
        seq
    }

    pub fn get(&self, seq: u64) -> Option<&HistoryEntry> {
        self.buffer.iter().find(|e| e.seq == seq)
    }

    pub fn range(&mut self, from_seq: u64, to_seq: u64) -> Result<Vec<&HistoryEntry>, HbError> {
        if from_seq > to_seq { return Err(HbError::InvalidRange { from: from_seq, to: to_seq }); }
        self.total_range_queries += 1;
        let result: Vec<&HistoryEntry> = self.buffer.iter().filter(|e| e.seq >= from_seq && e.seq <= to_seq).collect();
        Ok(result)
    }

    pub fn latest(&self) -> Option<&HistoryEntry> { self.buffer.back() }

    pub fn oldest(&self) -> Option<&HistoryEntry> { self.buffer.front() }

    pub fn compact(&mut self, keep_last: usize) -> usize {
        let remove_count = self.buffer.len().saturating_sub(keep_last);
        for _ in 0..remove_count { self.buffer.pop_front(); }
        self.total_compacted += remove_count as u64;
        remove_count
    }

    pub fn replay<F>(&self, mut f: F) -> usize
    where F: FnMut(&HistoryEntry) -> bool {
        let mut count = 0;
        for entry in &self.buffer {
            if !f(entry) { break; }
            count += 1;
        }
        count
    }

    pub fn replay_from<F>(&self, from_seq: u64, mut f: F) -> usize
    where F: FnMut(&HistoryEntry) -> bool {
        let mut count = 0;
        for entry in self.buffer.iter().filter(|e| e.seq >= from_seq) {
            if !f(entry) { break; }
            count += 1;
        }
        count
    }

    pub fn len(&self) -> usize { self.buffer.len() }
    pub fn is_empty(&self) -> bool { self.buffer.is_empty() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn next_seq(&self) -> u64 { self.next_seq }
    pub fn total_appended(&self) -> u64 { self.total_appended }
    pub fn total_compacted(&self) -> u64 { self.total_compacted }
    pub fn total_range_queries(&self) -> u64 { self.total_range_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buf() { let b = HistBuf::new(4); assert_eq!(b.capacity(), 4); assert!(b.is_empty()); }

    #[test]
    fn append_get() {
        let mut b = HistBuf::new(4);
        let seq = b.append(10, b"hello".to_vec());
        assert_eq!(seq, 1);
        let e = b.get(1).unwrap();
        assert_eq!(e.data, b"hello");
        assert_eq!(e.timestamp, 10);
    }

    #[test]
    fn wraparound() {
        let mut b = HistBuf::new(2);
        b.append(1, b"a".to_vec());
        b.append(2, b"b".to_vec());
        b.append(3, b"c".to_vec());
        assert_eq!(b.len(), 2);
        assert_eq!(b.oldest().unwrap().data, b"b");
        assert_eq!(b.latest().unwrap().data, b"c");
    }

    #[test]
    fn range_query() {
        let mut b = HistBuf::new(10);
        for i in 0..5 { b.append(i, vec![i as u8]); }
        let entries = b.range(2, 4).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].seq, 2);
    }

    #[test]
    fn invalid_range() {
        let mut b = HistBuf::new(10);
        let err = b.range(5, 3).unwrap_err();
        assert!(matches!(err, HbError::InvalidRange { .. }));
    }

    #[test]
    fn compact() {
        let mut b = HistBuf::new(10);
        for i in 0..10 { b.append(i, vec![i as u8]); }
        let removed = b.compact(3);
        assert_eq!(removed, 7);
        assert_eq!(b.len(), 3);
        assert_eq!(b.oldest().unwrap().seq, 8);
    }

    #[test]
    fn replay() {
        let mut b = HistBuf::new(10);
        for i in 0..5 { b.append(i, vec![i as u8]); }
        let mut seen = Vec::new();
        let count = b.replay(|e| { seen.push(e.seq); true });
        assert_eq!(count, 5);
        assert_eq!(seen, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn replay_stop() {
        let mut b = HistBuf::new(10);
        for i in 0..5 { b.append(i, vec![i as u8]); }
        let count = b.replay(|e| e.seq < 3);
        assert_eq!(count, 2);
    }

    #[test]
    fn stats() {
        let mut b = HistBuf::new(10);
        b.append(0, b"x".to_vec());
        b.range(1, 1).unwrap();
        assert_eq!(b.total_appended(), 1);
        assert_eq!(b.total_range_queries(), 1);
    }

    #[test]
    fn error_display() { assert!(HbError::BufferEmpty.to_string().contains("empty")); }
}

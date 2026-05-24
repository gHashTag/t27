use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClogError {
    Full { capacity: usize },
    Empty,
    TruncatePosition { pos: usize, len: usize },
}

impl std::fmt::Display for ClogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClogError::Full { capacity } => write!(f, "log full ({capacity})"),
            ClogError::Empty => write!(f, "log empty"),
            ClogError::TruncatePosition { pos, len } => write!(f, "pos {pos} >= len {len}"),
        }
    }
}

impl std::error::Error for ClogError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub seq: u64,
    pub code: u32,
    pub payload: u64,
}

#[derive(Debug, Clone)]
pub struct CompactLog {
    entries: VecDeque<LogEntry>,
    capacity: usize,
    next_seq: u64,
    total_appended: u64,
    total_compacted: u64,
    total_truncated: u64,
    total_replayed: u64,
}

impl CompactLog {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
            next_seq: 1,
            total_appended: 0,
            total_compacted: 0,
            total_truncated: 0,
            total_replayed: 0,
        }
    }

    pub fn capacity(&self) -> usize { self.capacity }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    pub fn append(&mut self, code: u32, payload: u64) -> Result<u64, ClogError> {
        if self.entries.len() >= self.capacity {
            return Err(ClogError::Full { capacity: self.capacity });
        }
        let seq = self.next_seq;
        self.entries.push_back(LogEntry { seq, code, payload });
        self.next_seq += 1;
        self.total_appended += 1;
        Ok(seq)
    }

    pub fn get(&self, seq: u64) -> Option<&LogEntry> {
        self.entries.iter().find(|e| e.seq == seq)
    }

    pub fn first(&self) -> Option<&LogEntry> {
        self.entries.front()
    }

    pub fn last(&self) -> Option<&LogEntry> {
        self.entries.back()
    }

    pub fn replay(&mut self) -> Vec<LogEntry> {
        let entries: Vec<LogEntry> = self.entries.iter().cloned().collect();
        self.total_replayed += 1;
        entries
    }

    pub fn replay_since(&self, seq: u64) -> Vec<LogEntry> {
        self.entries.iter().filter(|e| e.seq >= seq).cloned().collect()
    }

    pub fn compact(&mut self, keep_last: usize) -> usize {
        let remove = self.entries.len().saturating_sub(keep_last);
        for _ in 0..remove { self.entries.pop_front(); }
        self.total_compacted += remove as u64;
        remove
    }

    pub fn truncate(&mut self, pos: usize) -> Result<usize, ClogError> {
        if pos >= self.entries.len() {
            return Err(ClogError::TruncatePosition { pos, len: self.entries.len() });
        }
        let removed = self.entries.len() - pos;
        self.entries.truncate(pos);
        self.total_truncated += removed as u64;
        Ok(removed)
    }

    pub fn compact_by_code(&mut self, code: u32) -> usize {
        let before = self.entries.len();
        self.entries.retain(|e| e.code != code);
        let removed = before - self.entries.len();
        self.total_compacted += removed as u64;
        removed
    }

    pub fn entries_by_code(&self, code: u32) -> Vec<&LogEntry> {
        self.entries.iter().filter(|e| e.code == code).collect()
    }

    pub fn next_seq(&self) -> u64 { self.next_seq }
    pub fn total_appended(&self) -> u64 { self.total_appended }
    pub fn total_compacted(&self) -> u64 { self.total_compacted }
    pub fn total_truncated(&self) -> u64 { self.total_truncated }
    pub fn total_replayed(&self) -> u64 { self.total_replayed }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_log() {
        let cl = CompactLog::new(100);
        assert_eq!(cl.capacity(), 100);
        assert!(cl.is_empty());
    }

    #[test]
    fn append_and_get() {
        let mut cl = CompactLog::new(100);
        let seq = cl.append(1, 42).unwrap();
        assert_eq!(seq, 1);
        assert_eq!(cl.get(seq).unwrap().payload, 42);
        assert_eq!(cl.len(), 1);
    }

    #[test]
    fn full_log() {
        let mut cl = CompactLog::new(2);
        cl.append(1, 1).unwrap();
        cl.append(1, 2).unwrap();
        let err = cl.append(1, 3).unwrap_err();
        assert!(matches!(err, ClogError::Full { .. }));
    }

    #[test]
    fn first_last() {
        let mut cl = CompactLog::new(10);
        cl.append(1, 10).unwrap();
        cl.append(1, 20).unwrap();
        cl.append(1, 30).unwrap();
        assert_eq!(cl.first().unwrap().payload, 10);
        assert_eq!(cl.last().unwrap().payload, 30);
    }

    #[test]
    fn replay() {
        let mut cl = CompactLog::new(10);
        cl.append(1, 10).unwrap();
        cl.append(2, 20).unwrap();
        let entries = cl.replay();
        assert_eq!(entries.len(), 2);
        assert_eq!(cl.total_replayed(), 1);
    }

    #[test]
    fn replay_since() {
        let mut cl = CompactLog::new(10);
        cl.append(1, 10).unwrap();
        cl.append(1, 20).unwrap();
        cl.append(1, 30).unwrap();
        let entries = cl.replay_since(2);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn compact() {
        let mut cl = CompactLog::new(10);
        for i in 0..10 { cl.append(1, i).unwrap(); }
        let removed = cl.compact(5);
        assert_eq!(removed, 5);
        assert_eq!(cl.len(), 5);
        assert_eq!(cl.first().unwrap().payload, 5);
    }

    #[test]
    fn truncate() {
        let mut cl = CompactLog::new(10);
        for i in 0..5 { cl.append(1, i).unwrap(); }
        let removed = cl.truncate(3).unwrap();
        assert_eq!(removed, 2);
        assert_eq!(cl.len(), 3);
    }

    #[test]
    fn truncate_out_of_range() {
        let mut cl = CompactLog::new(10);
        cl.append(1, 0).unwrap();
        let err = cl.truncate(5).unwrap_err();
        assert!(matches!(err, ClogError::TruncatePosition { .. }));
    }

    #[test]
    fn compact_by_code() {
        let mut cl = CompactLog::new(10);
        cl.append(1, 10).unwrap();
        cl.append(2, 20).unwrap();
        cl.append(1, 30).unwrap();
        let removed = cl.compact_by_code(1);
        assert_eq!(removed, 2);
        assert_eq!(cl.len(), 1);
    }

    #[test]
    fn entries_by_code() {
        let mut cl = CompactLog::new(10);
        cl.append(1, 10).unwrap();
        cl.append(2, 20).unwrap();
        cl.append(1, 30).unwrap();
        assert_eq!(cl.entries_by_code(1).len(), 2);
    }

    #[test]
    fn error_display() {
        assert!(ClogError::Full { capacity: 64 }.to_string().contains("64"));
    }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RaftLogError {
    IndexNotFound { index: u64 },
    TermMismatch { expected: u64, got: u64 },
    EmptyLog,
    AlreadyCompacted { index: u64 },
}

impl std::fmt::Display for RaftLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RaftLogError::IndexNotFound { index } => write!(f, "index {index} not found"),
            RaftLogError::TermMismatch { expected, got } =>
                write!(f, "term mismatch: expected {expected}, got {got}"),
            RaftLogError::EmptyLog => write!(f, "log is empty"),
            RaftLogError::AlreadyCompacted { index } => write!(f, "index {index} already compacted"),
        }
    }
}

impl std::error::Error for RaftLogError {}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub index: u64,
    pub term: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct RaftLog {
    entries: BTreeMap<u64, LogEntry>,
    commit_index: u64,
    last_applied: u64,
    compacted_up_to: u64,
    total_appended: u64,
}

impl RaftLog {
    pub fn new() -> Self {
        Self { entries: BTreeMap::new(), commit_index: 0, last_applied: 0, compacted_up_to: 0, total_appended: 0 }
    }

    pub fn append(&mut self, term: u64, data: Vec<u8>) -> u64 {
        let index = self.next_index();
        self.entries.insert(index, LogEntry { index, term, data });
        self.total_appended += 1;
        index
    }

    pub fn append_batch(&mut self, term: u64, entries: Vec<Vec<u8>>) -> Vec<u64> {
        let mut indices = Vec::with_capacity(entries.len());
        for data in entries { indices.push(self.append(term, data)); }
        indices
    }

    fn next_index(&self) -> u64 {
        self.entries.keys().next_back().map(|&k| k + 1).unwrap_or(1)
    }

    pub fn get(&self, index: u64) -> Option<&LogEntry> { self.entries.get(&index) }

    pub fn last_index(&self) -> Option<u64> { self.entries.keys().next_back().copied() }

    pub fn last_term(&self) -> Option<u64> {
        self.entries.values().next_back().map(|e| e.term)
    }

    pub fn commit(&mut self, index: u64) -> Result<u64, RaftLogError> {
        if !self.entries.contains_key(&index) { return Err(RaftLogError::IndexNotFound { index }); }
        let prev = self.commit_index;
        self.commit_index = index;
        Ok(prev)
    }

    pub fn apply(&mut self) -> Vec<LogEntry> {
        let mut applied = Vec::new();
        while self.last_applied < self.commit_index {
            self.last_applied += 1;
            if let Some(e) = self.entries.get(&self.last_applied).cloned() {
                applied.push(e);
            }
        }
        applied
    }

    pub fn compact(&mut self, up_to: u64) -> u64 {
        let removed = self.entries.keys().copied().filter(|&k| k <= up_to).count() as u64;
        self.entries.retain(|&k, _| k > up_to);
        self.compacted_up_to = self.compacted_up_to.max(up_to);
        removed
    }

    pub fn truncate_after(&mut self, after_index: u64) -> u64 {
        let removed = self.entries.keys().copied().filter(|&k| k > after_index).count() as u64;
        self.entries.retain(|&k, _| k <= after_index);
        removed
    }

    pub fn term_at(&self, index: u64) -> Option<u64> { self.entries.get(&index).map(|e| e.term) }

    pub fn range(&self, start: u64, end: u64) -> Vec<LogEntry> {
        self.entries.range(start..=end).map(|(_, e)| e.clone()).collect()
    }

    pub fn commit_index(&self) -> u64 { self.commit_index }
    pub fn last_applied(&self) -> u64 { self.last_applied }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn total_appended(&self) -> u64 { self.total_appended }
    pub fn compacted_up_to(&self) -> u64 { self.compacted_up_to }
}

impl Default for RaftLog {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_log() {
        let rl = RaftLog::new();
        assert!(rl.is_empty());
        assert_eq!(rl.commit_index(), 0);
    }

    #[test]
    fn append() {
        let mut rl = RaftLog::new();
        let idx = rl.append(1, vec![1, 2, 3]);
        assert_eq!(idx, 1);
        assert_eq!(rl.len(), 1);
        assert_eq!(rl.last_index(), Some(1));
    }

    #[test]
    fn append_batch() {
        let mut rl = RaftLog::new();
        let ids = rl.append_batch(1, vec![vec![1], vec![2], vec![3]]);
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn get_entry() {
        let mut rl = RaftLog::new();
        rl.append(2, vec![42]);
        let e = rl.get(1).unwrap();
        assert_eq!(e.term, 2);
        assert_eq!(e.data, vec![42]);
    }

    #[test]
    fn commit_apply() {
        let mut rl = RaftLog::new();
        rl.append(1, vec![1]); rl.append(1, vec![2]); rl.append(1, vec![3]);
        rl.commit(3).unwrap();
        let applied = rl.apply();
        assert_eq!(applied.len(), 3);
        assert_eq!(rl.last_applied(), 3);
    }

    #[test]
    fn partial_apply() {
        let mut rl = RaftLog::new();
        rl.append(1, vec![1]); rl.append(1, vec![2]);
        rl.commit(2).unwrap();
        let a1 = rl.apply();
        assert_eq!(a1.len(), 2);
        rl.append(1, vec![3]);
        rl.commit(3).unwrap();
        let a2 = rl.apply();
        assert_eq!(a2.len(), 1);
    }

    #[test]
    fn compact() {
        let mut rl = RaftLog::new();
        for i in 1..=10u8 { rl.append(1, vec![i]); }
        let removed = rl.compact(5);
        assert_eq!(removed, 5);
        assert_eq!(rl.len(), 5);
        assert!(rl.get(3).is_none());
    }

    #[test]
    fn truncate() {
        let mut rl = RaftLog::new();
        for i in 1..=5u8 { rl.append(1, vec![i]); }
        let removed = rl.truncate_after(3);
        assert_eq!(removed, 2);
        assert_eq!(rl.last_index(), Some(3));
    }

    #[test]
    fn range_query() {
        let mut rl = RaftLog::new();
        for i in 1..=5u8 { rl.append(1, vec![i]); }
        let r = rl.range(2, 4);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn term_at() {
        let mut rl = RaftLog::new();
        rl.append(1, vec![]);
        rl.append(2, vec![]);
        assert_eq!(rl.term_at(1), Some(1));
        assert_eq!(rl.term_at(2), Some(2));
    }

    #[test]
    fn commit_not_found() {
        let mut rl = RaftLog::new();
        let err = rl.commit(99).unwrap_err();
        assert!(matches!(err, RaftLogError::IndexNotFound { .. }));
    }

    #[test]
    fn error_display() {
        assert!(RaftLogError::EmptyLog.to_string().contains("empty"));
    }
}

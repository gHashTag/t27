use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum WlError {
    EntryNotFound { seq: u64 },
    TruncationTarget { target: u64, current: u64 },
}

impl std::fmt::Display for WlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WlError::EntryNotFound { seq } => write!(f, "entry {seq} not found"),
            WlError::TruncationTarget { target, current } => write!(f, "cannot truncate to {target} (current={current})"),
        }
    }
}

impl std::error::Error for WlError {}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub seq: u64,
    pub timestamp: u64,
    pub op: Vec<u8>,
    pub checksum: u64,
}

fn fnv_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

pub struct WriteLog {
    entries: BTreeMap<u64, LogEntry>,
    next_seq: u64,
    total_appended: u64,
    total_replayed: u64,
    total_truncated: u64,
}

impl WriteLog {
    pub fn new() -> Self { Self { entries: BTreeMap::new(), next_seq: 1, total_appended: 0, total_replayed: 0, total_truncated: 0 } }

    pub fn append(&mut self, timestamp: u64, op: Vec<u8>) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        let checksum = fnv_hash(&op);
        self.entries.insert(seq, LogEntry { seq, timestamp, op, checksum });
        self.total_appended += 1;
        seq
    }

    pub fn get(&self, seq: u64) -> Option<&LogEntry> { self.entries.get(&seq) }

    pub fn verify(&self, seq: u64) -> Result<bool, WlError> {
        let e = self.entries.get(&seq).ok_or(WlError::EntryNotFound { seq })?;
        Ok(fnv_hash(&e.op) == e.checksum)
    }

    pub fn replay<F>(&self, mut f: F) -> usize
    where F: FnMut(&LogEntry) -> bool {
        let mut count = 0;
        for (_, entry) in &self.entries {
            if !f(entry) { break; }
            count += 1;
        }
        count
    }

    pub fn replay_from<F>(&self, from_seq: u64, mut f: F) -> usize
    where F: FnMut(&LogEntry) -> bool {
        let mut count = 0;
        for (_, entry) in self.entries.range(from_seq..) {
            if !f(entry) { break; }
            count += 1;
        }
        count
    }

    pub fn truncate(&mut self, before_seq: u64) -> Result<u64, WlError> {
        if before_seq >= self.next_seq { return Err(WlError::TruncationTarget { target: before_seq, current: self.next_seq }); }
        let removed: Vec<u64> = self.entries.keys().filter(|&&s| s < before_seq).copied().collect();
        let count = removed.len() as u64;
        for s in removed { self.entries.remove(&s); }
        self.total_truncated += count;
        Ok(count)
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn next_seq(&self) -> u64 { self.next_seq }
    pub fn total_appended(&self) -> u64 { self.total_appended }
    pub fn total_replayed(&self) -> u64 { self.total_replayed }
    pub fn total_truncated(&self) -> u64 { self.total_truncated }
}

impl Default for WriteLog {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_log() { assert!(WriteLog::new().is_empty()); }

    #[test]
    fn append_get() {
        let mut wl = WriteLog::new();
        let seq = wl.append(10, b"write".to_vec());
        let e = wl.get(seq).unwrap();
        assert_eq!(e.op, b"write");
        assert_eq!(e.timestamp, 10);
    }

    #[test]
    fn verify_ok() {
        let mut wl = WriteLog::new();
        let seq = wl.append(0, b"data".to_vec());
        assert!(wl.verify(seq).unwrap());
    }

    #[test]
    fn verify_tampered() {
        let mut wl = WriteLog::new();
        let seq = wl.append(0, b"data".to_vec());
        wl.entries.get_mut(&seq).unwrap().op = b"tainted".to_vec();
        assert!(!wl.verify(seq).unwrap());
    }

    #[test]
    fn replay() {
        let mut wl = WriteLog::new();
        wl.append(0, b"a".to_vec());
        wl.append(0, b"b".to_vec());
        wl.append(0, b"c".to_vec());
        let mut ops = Vec::new();
        let count = wl.replay(|e| { ops.push(e.op.clone()); true });
        assert_eq!(count, 3);
        assert_eq!(ops.len(), 3);
    }

    #[test]
    fn replay_from() {
        let mut wl = WriteLog::new();
        wl.append(0, b"a".to_vec());
        wl.append(0, b"b".to_vec());
        wl.append(0, b"c".to_vec());
        let count = wl.replay_from(2, |_| true);
        assert_eq!(count, 2);
    }

    #[test]
    fn truncate() {
        let mut wl = WriteLog::new();
        wl.append(0, b"a".to_vec());
        wl.append(0, b"b".to_vec());
        wl.append(0, b"c".to_vec());
        let removed = wl.truncate(3).unwrap();
        assert_eq!(removed, 2);
        assert_eq!(wl.len(), 1);
    }

    #[test]
    fn truncate_error() {
        let mut wl = WriteLog::new();
        wl.append(0, b"a".to_vec());
        let err = wl.truncate(99).unwrap_err();
        assert!(matches!(err, WlError::TruncationTarget { .. }));
    }

    #[test]
    fn verify_missing() {
        let wl = WriteLog::new();
        let err = wl.verify(1).unwrap_err();
        assert!(matches!(err, WlError::EntryNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut wl = WriteLog::new();
        wl.append(0, b"x".to_vec());
        assert_eq!(wl.total_appended(), 1);
        assert_eq!(wl.next_seq(), 2);
    }

    #[test]
    fn error_display() { assert!(WlError::EntryNotFound { seq: 1 }.to_string().contains("1")); }
}

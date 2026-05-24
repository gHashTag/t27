use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum WalError {
    AlreadyClosed,
    SequenceGap { expected: u64, got: u64 },
    NotFound { seq: u64 },
}

impl std::fmt::Display for WalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalError::AlreadyClosed => write!(f, "WAL closed"),
            WalError::SequenceGap { expected, got } =>
                write!(f, "seq gap: expected {expected}, got {got}"),
            WalError::NotFound { seq } => write!(f, "seq {seq} not found"),
        }
    }
}

impl std::error::Error for WalError {}

#[derive(Debug, Clone)]
pub struct WalEntry {
    pub seq: u64,
    pub data: Vec<u8>,
    pub checksum: u64,
}

fn checksum(data: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

pub struct WriteAheadLog {
    entries: BTreeMap<u64, WalEntry>,
    next_seq: u64,
    closed: bool,
    total_appended: u64,
    total_replayed: u64,
    total_truncated: u64,
}

impl WriteAheadLog {
    pub fn new() -> Self {
        Self { entries: BTreeMap::new(), next_seq: 1, closed: false, total_appended: 0, total_replayed: 0, total_truncated: 0 }
    }

    pub fn append(&mut self, data: Vec<u8>) -> Result<u64, WalError> {
        if self.closed { return Err(WalError::AlreadyClosed); }
        let seq = self.next_seq;
        self.next_seq += 1;
        let cs = checksum(&data);
        self.entries.insert(seq, WalEntry { seq, data, checksum: cs });
        self.total_appended += 1;
        Ok(seq)
    }

    pub fn append_batch(&mut self, entries: Vec<Vec<u8>>) -> Result<Vec<u64>, WalError> {
        let mut seqs = Vec::with_capacity(entries.len());
        for data in entries { seqs.push(self.append(data)?); }
        Ok(seqs)
    }

    pub fn get(&self, seq: u64) -> Option<&WalEntry> { self.entries.get(&seq) }

    pub fn replay(&mut self) -> Vec<WalEntry> {
        let entries: Vec<WalEntry> = self.entries.values().cloned().collect();
        for e in &entries {
            if checksum(&e.data) != e.checksum { continue; }
        }
        self.total_replayed += entries.len() as u64;
        entries
    }

    pub fn replay_from(&mut self, start_seq: u64) -> Vec<WalEntry> {
        let entries: Vec<WalEntry> = self.entries.range(start_seq..).map(|(_, e)| e.clone()).collect();
        self.total_replayed += entries.len() as u64;
        entries
    }

    pub fn truncate_before(&mut self, seq: u64) -> u64 {
        let removed = self.entries.keys().copied().filter(|&s| s < seq).count() as u64;
        self.entries.retain(|&s, _| s >= seq);
        self.total_truncated += removed;
        removed
    }

    pub fn verify(&self, seq: u64) -> Result<bool, WalError> {
        let entry = self.entries.get(&seq).ok_or(WalError::NotFound { seq })?;
        Ok(checksum(&entry.data) == entry.checksum)
    }

    pub fn close(&mut self) { self.closed = true; }
    pub fn is_closed(&self) -> bool { self.closed }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn last_seq(&self) -> Option<u64> { self.entries.keys().next_back().copied() }
    pub fn total_appended(&self) -> u64 { self.total_appended }
    pub fn total_replayed(&self) -> u64 { self.total_replayed }
    pub fn total_truncated(&self) -> u64 { self.total_truncated }
}

impl Default for WriteAheadLog {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_wal() {
        let wal = WriteAheadLog::new();
        assert!(wal.is_empty());
        assert!(!wal.is_closed());
    }

    #[test]
    fn append_get() {
        let mut wal = WriteAheadLog::new();
        let seq = wal.append(vec![1, 2, 3]).unwrap();
        assert_eq!(seq, 1);
        let e = wal.get(1).unwrap();
        assert_eq!(e.data, vec![1, 2, 3]);
    }

    #[test]
    fn batch_append() {
        let mut wal = WriteAheadLog::new();
        let seqs = wal.append_batch(vec![vec![1], vec![2], vec![3]]).unwrap();
        assert_eq!(seqs, vec![1, 2, 3]);
    }

    #[test]
    fn replay() {
        let mut wal = WriteAheadLog::new();
        wal.append(vec![1]).unwrap();
        wal.append(vec![2]).unwrap();
        let entries = wal.replay();
        assert_eq!(entries.len(), 2);
        assert_eq!(wal.total_replayed(), 2);
    }

    #[test]
    fn replay_from() {
        let mut wal = WriteAheadLog::new();
        wal.append(vec![1]).unwrap();
        wal.append(vec![2]).unwrap();
        wal.append(vec![3]).unwrap();
        let entries = wal.replay_from(2);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn truncate() {
        let mut wal = WriteAheadLog::new();
        for i in 0..10u8 { wal.append(vec![i]).unwrap(); }
        let removed = wal.truncate_before(5);
        assert_eq!(removed, 4);
        assert_eq!(wal.len(), 6);
    }

    #[test]
    fn verify_integrity() {
        let mut wal = WriteAheadLog::new();
        wal.append(vec![42]).unwrap();
        assert!(wal.verify(1).unwrap());
    }

    #[test]
    fn verify_not_found() {
        let wal = WriteAheadLog::new();
        let err = wal.verify(99).unwrap_err();
        assert!(matches!(err, WalError::NotFound { .. }));
    }

    #[test]
    fn close() {
        let mut wal = WriteAheadLog::new();
        wal.close();
        let err = wal.append(vec![1]).unwrap_err();
        assert!(matches!(err, WalError::AlreadyClosed));
    }

    #[test]
    fn last_seq() {
        let mut wal = WriteAheadLog::new();
        wal.append(vec![1]).unwrap();
        wal.append(vec![2]).unwrap();
        assert_eq!(wal.last_seq(), Some(2));
    }

    #[test]
    fn stats() {
        let mut wal = WriteAheadLog::new();
        wal.append(vec![1]).unwrap();
        wal.append(vec![2]).unwrap();
        assert_eq!(wal.total_appended(), 2);
    }

    #[test]
    fn error_display() {
        assert!(WalError::AlreadyClosed.to_string().contains("closed"));
    }
}

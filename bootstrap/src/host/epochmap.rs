use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum EmError {
    KeyNotFound { key: u64 },
    StaleRead { key: u64, read_epoch: u64, write_epoch: u64 },
}

impl std::fmt::Display for EmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmError::KeyNotFound { key } => write!(f, "key {key} not found"),
            EmError::StaleRead { key, read_epoch, write_epoch } => write!(f, "stale read for {key}: epoch {read_epoch} < {write_epoch}"),
        }
    }
}

impl std::error::Error for EmError {}

struct Entry {
    value: Vec<u8>,
    epoch: u64,
    tombstone: bool,
}

pub struct EpochMap {
    data: BTreeMap<u64, Entry>,
    epoch: u64,
    gc_threshold: u64,
    total_writes: u64,
    total_reads: u64,
    total_gcs: u64,
    total_stale: u64,
}

impl EpochMap {
    pub fn new(gc_threshold: u64) -> Self { Self { data: BTreeMap::new(), epoch: 0, gc_threshold, total_writes: 0, total_reads: 0, total_gcs: 0, total_stale: 0 } }

    pub fn write(&mut self, key: u64, value: Vec<u8>) -> u64 {
        self.epoch += 1;
        self.data.insert(key, Entry { value, epoch: self.epoch, tombstone: false });
        self.total_writes += 1;
        self.epoch
    }

    pub fn read(&self, key: u64) -> Option<(&Vec<u8>, u64)> {
        self.data.get(&key).filter(|e| !e.tombstone).map(|e| (&e.value, e.epoch))
    }

    pub fn read_at_epoch(&mut self, key: u64, epoch: u64) -> Result<&Vec<u8>, EmError> {
        self.total_reads += 1;
        let e = self.data.get(&key).ok_or(EmError::KeyNotFound { key })?;
        if e.tombstone { return Err(EmError::KeyNotFound { key }); }
        if e.epoch > epoch {
            self.total_stale += 1;
            return Err(EmError::StaleRead { key, read_epoch: epoch, write_epoch: e.epoch });
        }
        Ok(&e.value)
    }

    pub fn delete(&mut self, key: u64) -> Result<u64, EmError> {
        let e = self.data.get_mut(&key).ok_or(EmError::KeyNotFound { key })?;
        e.tombstone = true;
        self.epoch += 1;
        e.epoch = self.epoch;
        self.total_writes += 1;
        Ok(self.epoch)
    }

    pub fn gc(&mut self) -> usize {
        let cutoff = self.epoch.saturating_sub(self.gc_threshold);
        let old_len = self.data.len();
        self.data.retain(|_, e| e.epoch >= cutoff || !e.tombstone);
        let removed = old_len - self.data.len();
        self.total_gcs += removed as u64;
        removed
    }

    pub fn epoch(&self) -> u64 { self.epoch }
    pub fn len(&self) -> usize { self.data.values().filter(|e| !e.tombstone).count() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn total_writes(&self) -> u64 { self.total_writes }
    pub fn total_reads(&self) -> u64 { self.total_reads }
    pub fn total_gcs(&self) -> u64 { self.total_gcs }
    pub fn total_stale(&self) -> u64 { self.total_stale }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() { let em = EpochMap::new(10); assert!(em.is_empty()); assert_eq!(em.epoch(), 0); }

    #[test]
    fn write_read() {
        let mut em = EpochMap::new(10);
        em.write(1, b"val".to_vec());
        let (v, ep) = em.read(1).unwrap();
        assert_eq!(v, &b"val".to_vec());
        assert_eq!(ep, 1);
    }

    #[test]
    fn epoch_advances() {
        let mut em = EpochMap::new(10);
        em.write(1, b"a".to_vec());
        em.write(2, b"b".to_vec());
        assert_eq!(em.epoch(), 2);
    }

    #[test]
    fn stale_read() {
        let mut em = EpochMap::new(10);
        em.write(1, b"v1".to_vec());
        let epoch1 = em.epoch();
        em.write(1, b"v2".to_vec());
        let err = em.read_at_epoch(1, epoch1).unwrap_err();
        assert!(matches!(err, EmError::StaleRead { .. }));
    }

    #[test]
    fn valid_epoch_read() {
        let mut em = EpochMap::new(10);
        em.write(1, b"val".to_vec());
        let epoch = em.epoch();
        assert_eq!(em.read_at_epoch(1, epoch), Ok(&b"val".to_vec()));
    }

    #[test]
    fn delete() {
        let mut em = EpochMap::new(10);
        em.write(1, b"val".to_vec());
        em.delete(1).unwrap();
        assert!(em.read(1).is_none());
    }

    #[test]
    fn gc_removes_tombstones() {
        let mut em = EpochMap::new(1);
        em.write(1, b"a".to_vec());
        em.delete(1).unwrap();
        em.write(2, b"b".to_vec());
        em.write(3, b"c".to_vec());
        let removed = em.gc();
        assert!(removed > 0);
    }

    #[test]
    fn not_found() {
        let mut em = EpochMap::new(10);
        let err = em.read_at_epoch(99, 0).unwrap_err();
        assert!(matches!(err, EmError::KeyNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut em = EpochMap::new(10);
        em.write(1, b"x".to_vec());
        em.read_at_epoch(1, 0);
        assert_eq!(em.total_writes(), 1);
        assert_eq!(em.total_reads(), 1);
    }

    #[test]
    fn error_display() { assert!(EmError::KeyNotFound { key: 1 }.to_string().contains("1")); }
}

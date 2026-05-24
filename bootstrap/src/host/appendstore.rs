use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum AsError {
    KeyNotFound { key: Vec<u8> },
    Tombstone { key: Vec<u8> },
}

impl std::fmt::Display for AsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsError::KeyNotFound { key } => write!(f, "{:?}: not found", key),
            AsError::Tombstone { key } => write!(f, "{:?}: tombstoned", key),
        }
    }
}

impl std::error::Error for AsError {}

struct Record {
    key: Vec<u8>,
    value: Option<Vec<u8>>,
    seq: u64,
}

struct Segment {
    id: u32,
    records: Vec<Record>,
    size_bytes: usize,
}

pub struct AppendStore {
    segments: Vec<Segment>,
    active_segment_id: u32,
    segment_capacity: usize,
    index: BTreeMap<Vec<u8>, (u32, usize)>,
    tombstones: BTreeMap<Vec<u8>, u64>,
    seq: u64,
    total_writes: u64,
    total_reads: u64,
    total_deletes: u64,
    total_compactions: u64,
}

impl AppendStore {
    pub fn new(segment_capacity: usize) -> Self {
        Self {
            segments: vec![Segment { id: 0, records: Vec::new(), size_bytes: 0 }],
            active_segment_id: 0,
            segment_capacity,
            index: BTreeMap::new(),
            tombstones: BTreeMap::new(),
            seq: 0,
            total_writes: 0,
            total_reads: 0,
            total_deletes: 0,
            total_compactions: 0,
        }
    }

    fn ensure_capacity(&mut self) {
        let active = self.segments.last_mut().unwrap();
        if active.records.len() >= self.segment_capacity {
            let new_id = self.active_segment_id + 1;
            self.segments.push(Segment { id: new_id, records: Vec::new(), size_bytes: 0 });
            self.active_segment_id = new_id;
        }
    }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> u64 {
        self.ensure_capacity();
        let seq = self.seq;
        self.seq += 1;
        let active = self.segments.last_mut().unwrap();
        let rec_idx = active.records.len();
        active.size_bytes += key.len() + value.len();
        active.records.push(Record { key: key.clone(), value: Some(value), seq });
        self.tombstones.remove(&key);
        self.index.insert(key, (self.active_segment_id, rec_idx));
        self.total_writes += 1;
        seq
    }

    pub fn get(&mut self, key: &[u8]) -> Result<Vec<u8>, AsError> {
        self.total_reads += 1;
        if let Some(ts_seq) = self.tombstones.get(key) {
            if let Some((seg_id, rec_idx)) = self.index.get(key) {
                let seg = &self.segments[*seg_id as usize];
                let rec = &seg.records[*rec_idx];
                if rec.seq <= *ts_seq { return Err(AsError::Tombstone { key: key.to_vec() }); }
            } else { return Err(AsError::Tombstone { key: key.to_vec() }); }
        }
        let (seg_id, rec_idx) = self.index.get(key).ok_or(AsError::KeyNotFound { key: key.to_vec() })?;
        let seg = &self.segments[*seg_id as usize];
        let rec = &seg.records[*rec_idx];
        rec.value.clone().ok_or(AsError::Tombstone { key: key.to_vec() })
    }

    pub fn delete(&mut self, key: &[u8]) -> bool {
        self.total_reads += 1;
        if !self.index.contains_key(key) { return false; }
        self.tombstones.insert(key.to_vec(), self.seq);
        self.seq += 1;
        self.total_deletes += 1;
        true
    }

    pub fn compact(&mut self) -> usize {
        self.total_compactions += 1;
        let mut new_index = BTreeMap::new();
        let mut new_records = Vec::new();
        for seg in &self.segments {
            for (idx, rec) in seg.records.iter().enumerate() {
                if rec.value.is_none() { continue; }
                if let Some(ts_seq) = self.tombstones.get(&rec.key) {
                    if rec.seq <= *ts_seq { continue; }
                }
                if let Some((_, existing_idx)) = new_index.get(&rec.key) {
                    new_records[*existing_idx] = Record { key: rec.key.clone(), value: rec.value.clone(), seq: rec.seq };
                } else {
                    let new_idx = new_records.len();
                    new_records.push(Record { key: rec.key.clone(), value: rec.value.clone(), seq: rec.seq });
                    new_index.insert(rec.key.clone(), (0, new_idx));
                }
            }
        }
        let removed_count = self.segments.iter().map(|s| s.records.len()).sum::<usize>() - new_records.len();
        self.segments.clear();
        let size: usize = new_records.iter().map(|r| r.key.len() + r.value.as_ref().map(|v| v.len()).unwrap_or(0)).sum();
        self.segments.push(Segment { id: 0, records: new_records, size_bytes: size });
        self.active_segment_id = 0;
        self.index = new_index;
        self.tombstones.clear();
        removed_count
    }

    pub fn segment_count(&self) -> usize { self.segments.len() }
    pub fn record_count(&self) -> usize { self.segments.iter().map(|s| s.records.len()).sum() }
    pub fn total_bytes(&self) -> usize { self.segments.iter().map(|s| s.size_bytes).sum() }
    pub fn tombstone_count(&self) -> usize { self.tombstones.len() }
    pub fn total_writes(&self) -> u64 { self.total_writes }
    pub fn total_reads(&self) -> u64 { self.total_reads }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn total_compactions(&self) -> u64 { self.total_compactions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_store() { let s = AppendStore::new(100); assert_eq!(s.record_count(), 0); }

    #[test]
    fn put_get() {
        let mut s = AppendStore::new(100);
        s.put(b"key".to_vec(), b"val".to_vec());
        assert_eq!(s.get(b"key"), Ok(b"val".to_vec()));
    }

    #[test]
    fn overwrite() {
        let mut s = AppendStore::new(100);
        s.put(b"k".to_vec(), b"v1".to_vec());
        s.put(b"k".to_vec(), b"v2".to_vec());
        assert_eq!(s.get(b"k"), Ok(b"v2".to_vec()));
    }

    #[test]
    fn delete() {
        let mut s = AppendStore::new(100);
        s.put(b"k".to_vec(), b"v".to_vec());
        assert!(s.delete(b"k"));
        let err = s.get(b"k").unwrap_err();
        assert!(matches!(err, AsError::Tombstone { .. }));
    }

    #[test]
    fn delete_nonexistent() {
        let mut s = AppendStore::new(100);
        assert!(!s.delete(b"x"));
    }

    #[test]
    fn not_found() {
        let mut s = AppendStore::new(100);
        let err = s.get(b"x").unwrap_err();
        assert!(matches!(err, AsError::KeyNotFound { .. }));
    }

    #[test]
    fn segment_rollover() {
        let mut s = AppendStore::new(3);
        s.put(b"a".to_vec(), b"1".to_vec());
        s.put(b"b".to_vec(), b"2".to_vec());
        s.put(b"c".to_vec(), b"3".to_vec());
        s.put(b"d".to_vec(), b"4".to_vec());
        assert!(s.segment_count() >= 2);
    }

    #[test]
    fn compact() {
        let mut s = AppendStore::new(100);
        s.put(b"a".to_vec(), b"1".to_vec());
        s.put(b"b".to_vec(), b"2".to_vec());
        s.delete(b"a");
        let removed = s.compact();
        assert!(removed > 0);
        assert!(s.get(b"b").is_ok());
        assert!(s.get(b"a").is_err());
        assert_eq!(s.tombstone_count(), 0);
    }

    #[test]
    fn stats() {
        let mut s = AppendStore::new(100);
        s.put(b"k".to_vec(), b"v".to_vec());
        s.get(b"k");
        assert_eq!(s.total_writes(), 1);
        assert_eq!(s.total_reads(), 1);
    }

    #[test]
    fn error_display() { assert!(AsError::KeyNotFound { key: b"k".to_vec() }.to_string().contains("not found")); }
}

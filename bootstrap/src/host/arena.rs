use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ArenaError {
    OutOfMemory { requested: usize, available: usize },
    SnapshotNotFound { id: u64 },
}

impl std::fmt::Display for ArenaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArenaError::OutOfMemory { requested, available } => write!(f, "oom: need {requested}, have {available}"),
            ArenaError::SnapshotNotFound { id } => write!(f, "snapshot {id} not found"),
        }
    }
}

impl std::error::Error for ArenaError {}

struct Snapshot {
    id: u64,
    offset: usize,
}

pub struct MemoryArena {
    buffer: Vec<u8>,
    offset: usize,
    snapshots: BTreeMap<u64, Snapshot>,
    next_snap: u64,
    total_allocations: u64,
    total_bytes_allocated: u64,
    total_rollbacks: u64,
}

impl MemoryArena {
    pub fn new(capacity: usize) -> Self {
        Self { buffer: vec![0; capacity], offset: 0, snapshots: BTreeMap::new(), next_snap: 1, total_allocations: 0, total_bytes_allocated: 0, total_rollbacks: 0 }
    }

    pub fn alloc(&mut self, size: usize) -> Result<usize, ArenaError> {
        let available = self.buffer.len() - self.offset;
        if size > available { return Err(ArenaError::OutOfMemory { requested: size, available }); }
        let ptr = self.offset;
        self.offset += size;
        self.total_allocations += 1;
        self.total_bytes_allocated += size as u64;
        Ok(ptr)
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) -> bool {
        if offset + data.len() > self.offset { return false; }
        self.buffer[offset..offset + data.len()].copy_from_slice(data);
        true
    }

    pub fn read(&self, offset: usize, size: usize) -> Option<&[u8]> {
        if offset + size > self.offset { return None; }
        Some(&self.buffer[offset..offset + size])
    }

    pub fn snapshot(&mut self) -> u64 {
        let id = self.next_snap;
        self.next_snap += 1;
        self.snapshots.insert(id, Snapshot { id, offset: self.offset });
        id
    }

    pub fn rollback(&mut self, snap_id: u64) -> Result<usize, ArenaError> {
        let snap = self.snapshots.get(&snap_id).ok_or(ArenaError::SnapshotNotFound { id: snap_id })?;
        let reclaimed = self.offset - snap.offset;
        self.offset = snap.offset;
        self.snapshots.retain(|&id, _| id <= snap_id);
        self.total_rollbacks += 1;
        Ok(reclaimed)
    }

    pub fn reset(&mut self) {
        self.offset = 0;
        self.snapshots.clear();
    }

    pub fn used(&self) -> usize { self.offset }
    pub fn capacity(&self) -> usize { self.buffer.len() }
    pub fn available(&self) -> usize { self.buffer.len() - self.offset }
    pub fn utilization(&self) -> f64 { if self.buffer.is_empty() { 0.0 } else { self.offset as f64 / self.buffer.len() as f64 } }
    pub fn total_allocations(&self) -> u64 { self.total_allocations }
    pub fn total_bytes_allocated(&self) -> u64 { self.total_bytes_allocated }
    pub fn total_rollbacks(&self) -> u64 { self.total_rollbacks }
    pub fn snapshot_count(&self) -> usize { self.snapshots.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_arena() { let a = MemoryArena::new(1024); assert_eq!(a.capacity(), 1024); assert_eq!(a.used(), 0); }

    #[test]
    fn alloc_write_read() {
        let mut a = MemoryArena::new(1024);
        let off = a.alloc(5).unwrap();
        assert!(a.write(off, b"hello"));
        let data = a.read(off, 5).unwrap();
        assert_eq!(data, b"hello");
    }

    #[test]
    fn oom() {
        let mut a = MemoryArena::new(10);
        let err = a.alloc(20).unwrap_err();
        assert!(matches!(err, ArenaError::OutOfMemory { .. }));
    }

    #[test]
    fn snapshot_rollback() {
        let mut a = MemoryArena::new(1024);
        a.alloc(100).unwrap();
        let snap = a.snapshot();
        a.alloc(200).unwrap();
        assert_eq!(a.used(), 300);
        let reclaimed = a.rollback(snap).unwrap();
        assert_eq!(reclaimed, 200);
        assert_eq!(a.used(), 100);
    }

    #[test]
    fn reset() {
        let mut a = MemoryArena::new(1024);
        a.alloc(500).unwrap();
        a.reset();
        assert_eq!(a.used(), 0);
    }

    #[test]
    fn invalid_snapshot() {
        let mut a = MemoryArena::new(1024);
        let err = a.rollback(99).unwrap_err();
        assert!(matches!(err, ArenaError::SnapshotNotFound { .. }));
    }

    #[test]
    fn out_of_bounds_read() {
        let a = MemoryArena::new(10);
        assert!(a.read(0, 20).is_none());
    }

    #[test]
    fn out_of_bounds_write() {
        let mut a = MemoryArena::new(10);
        assert!(!a.write(0, b"hello world!!!"));
    }

    #[test]
    fn utilization() {
        let mut a = MemoryArena::new(1000);
        a.alloc(250).unwrap();
        assert!(a.utilization() > 0.24 && a.utilization() < 0.26);
    }

    #[test]
    fn stats() {
        let mut a = MemoryArena::new(1024);
        a.alloc(100).unwrap();
        let snap = a.snapshot();
        a.alloc(200).unwrap();
        a.rollback(snap).unwrap();
        assert_eq!(a.total_allocations(), 2);
        assert_eq!(a.total_bytes_allocated(), 300);
        assert_eq!(a.total_rollbacks(), 1);
    }

    #[test]
    fn error_display() { assert!(ArenaError::OutOfMemory { requested: 10, available: 5 }.to_string().contains("10")); }
}

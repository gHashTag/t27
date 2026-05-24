#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlabError {
    Full { capacity: usize },
    InvalidKey { key: usize },
    AlreadyFree { key: usize },
}

impl std::fmt::Display for SlabError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlabError::Full { capacity } => write!(f, "slab full (cap={capacity})"),
            SlabError::InvalidKey { key } => write!(f, "invalid key {key}"),
            SlabError::AlreadyFree { key } => write!(f, "key {key} already free"),
        }
    }
}

impl std::error::Error for SlabError {}

const EMPTY_MARKER: u64 = u64::MAX;

#[derive(Debug, Clone)]
struct SlabEntry {
    data: u64,
    occupied: bool,
    generation: u32,
}

#[derive(Debug, Clone)]
pub struct SlabKey {
    pub index: usize,
    pub generation: u32,
}

impl SlabKey {
    pub fn new(index: usize, generation: u32) -> Self {
        Self { index, generation }
    }
}

#[derive(Debug, Clone)]
pub struct SlabAllocator {
    entries: Vec<SlabEntry>,
    free_head: Option<usize>,
    len: usize,
    capacity: usize,
    total_alloc: u64,
    total_free: u64,
    peak_used: usize,
}

impl SlabAllocator {
    pub fn new(capacity: usize) -> Self {
        let mut entries = Vec::with_capacity(capacity);
        for i in 0..capacity {
            let next = if i + 1 < capacity { Some(i + 1) } else { None };
            entries.push(SlabEntry {
                data: next.map(|n| n as u64).unwrap_or(EMPTY_MARKER),
                occupied: false,
                generation: 0,
            });
        }
        Self {
            entries,
            free_head: if capacity > 0 { Some(0) } else { None },
            len: 0,
            capacity,
            total_alloc: 0,
            total_free: 0,
            peak_used: 0,
        }
    }

    pub fn alloc(&mut self, data: u64) -> Result<SlabKey, SlabError> {
        let idx = self.free_head.ok_or(SlabError::Full { capacity: self.capacity })?;
        let entry = &mut self.entries[idx];
        self.free_head = if entry.data != EMPTY_MARKER { Some(entry.data as usize) } else { None };
        entry.data = data;
        entry.occupied = true;
        let gen = entry.generation;
        self.len += 1;
        self.total_alloc += 1;
        if self.len > self.peak_used { self.peak_used = self.len; }
        Ok(SlabKey::new(idx, gen))
    }

    pub fn get(&self, key: &SlabKey) -> Option<u64> {
        let entry = self.entries.get(key.index)?;
        if !entry.occupied || entry.generation != key.generation { return None; }
        Some(entry.data)
    }

    pub fn get_mut(&mut self, key: &SlabKey) -> Option<&mut u64> {
        let entry = self.entries.get_mut(key.index)?;
        if !entry.occupied || entry.generation != key.generation { return None; }
        Some(&mut entry.data)
    }

    pub fn free(&mut self, key: &SlabKey) -> Result<u64, SlabError> {
        let entry = self.entries.get_mut(key.index)
            .ok_or(SlabError::InvalidKey { key: key.index })?;
        if !entry.occupied || entry.generation != key.generation {
            return Err(SlabError::AlreadyFree { key: key.index });
        }
        let data = entry.data;
        entry.data = self.free_head.map(|h| h as u64).unwrap_or(EMPTY_MARKER);
        self.free_head = Some(key.index);
        entry.occupied = false;
        entry.generation = entry.generation.wrapping_add(1);
        self.len -= 1;
        self.total_free += 1;
        Ok(data)
    }

    pub fn contains(&self, key: &SlabKey) -> bool {
        self.entries.get(key.index)
            .map(|e| e.occupied && e.generation == key.generation)
            .unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn free_count(&self) -> usize {
        self.capacity - self.len
    }

    pub fn peak_used(&self) -> usize {
        self.peak_used
    }

    pub fn total_alloc(&self) -> u64 {
        self.total_alloc
    }

    pub fn total_free(&self) -> u64 {
        self.total_free
    }

    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 { 0.0 } else { self.len as f64 / self.capacity as f64 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_slab() {
        let s = SlabAllocator::new(16);
        assert_eq!(s.capacity(), 16);
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn alloc_and_get() {
        let mut s = SlabAllocator::new(16);
        let key = s.alloc(0xDEAD).unwrap();
        assert_eq!(s.get(&key), Some(0xDEAD));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn alloc_multiple() {
        let mut s = SlabAllocator::new(16);
        let k1 = s.alloc(10).unwrap();
        let k2 = s.alloc(20).unwrap();
        assert_ne!(k1.index, k2.index);
        assert_eq!(s.get(&k1), Some(10));
        assert_eq!(s.get(&k2), Some(20));
    }

    #[test]
    fn full_slab() {
        let mut s = SlabAllocator::new(2);
        s.alloc(1).unwrap();
        s.alloc(2).unwrap();
        let err = s.alloc(3).unwrap_err();
        assert!(matches!(err, SlabError::Full { capacity: 2 }));
    }

    #[test]
    fn free_and_reuse() {
        let mut s = SlabAllocator::new(4);
        let k = s.alloc(42).unwrap();
        let data = s.free(&k).unwrap();
        assert_eq!(data, 42);
        assert_eq!(s.len(), 0);
        let k2 = s.alloc(99).unwrap();
        assert_eq!(k2.index, k.index);
        assert_ne!(k2.generation, k.generation);
    }

    #[test]
    fn stale_key_after_free() {
        let mut s = SlabAllocator::new(4);
        let k = s.alloc(1).unwrap();
        s.free(&k).unwrap();
        assert_eq!(s.get(&k), None);
        assert!(!s.contains(&k));
    }

    #[test]
    fn double_free() {
        let mut s = SlabAllocator::new(4);
        let k = s.alloc(1).unwrap();
        s.free(&k).unwrap();
        let err = s.free(&k).unwrap_err();
        assert!(matches!(err, SlabError::AlreadyFree { .. }));
    }

    #[test]
    fn get_mut() {
        let mut s = SlabAllocator::new(4);
        let k = s.alloc(10).unwrap();
        *s.get_mut(&k).unwrap() = 20;
        assert_eq!(s.get(&k), Some(20));
    }

    #[test]
    fn peak_used() {
        let mut s = SlabAllocator::new(8);
        let k1 = s.alloc(1).unwrap();
        let _k2 = s.alloc(2).unwrap();
        let _k3 = s.alloc(3).unwrap();
        assert_eq!(s.peak_used(), 3);
        s.free(&k1).unwrap();
        assert_eq!(s.peak_used(), 3);
    }

    #[test]
    fn stats() {
        let mut s = SlabAllocator::new(8);
        s.alloc(1).unwrap();
        s.alloc(2).unwrap();
        assert_eq!(s.total_alloc(), 2);
        assert_eq!(s.free_count(), 6);
        assert!((s.utilization() - 0.25).abs() < 0.01);
    }

    #[test]
    fn generation_wraps() {
        let mut s = SlabAllocator::new(4);
        let mut gen = 0u32;
        for _ in 0..10 {
            let k = s.alloc(1).unwrap();
            gen = k.generation;
            s.free(&k).unwrap();
        }
        assert!(gen >= 9);
    }

    #[test]
    fn error_display() {
        assert!(SlabError::Full { capacity: 4 }.to_string().contains("4"));
    }
}

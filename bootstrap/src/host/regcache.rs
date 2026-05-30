use std::collections::BTreeMap;
use super::csr_map;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheError {
    NotCached { offset: u32 },
    Stale { offset: u32, age_us: u64, max_us: u64 },
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::NotCached { offset } => write!(f, "register 0x{offset:X} not cached"),
            CacheError::Stale { offset, age_us, max_us } => {
                write!(f, "register 0x{offset:X} stale: {age_us}us > {max_us}us")
            }
        }
    }
}

impl std::error::Error for CacheError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheEntry {
    pub value: u32,
    pub timestamp_us: u64,
    pub dirty: bool,
}

#[derive(Debug, Clone)]
pub struct RegisterCache {
    entries: BTreeMap<u32, CacheEntry>,
    ttl_us: u64,
    hits: u64,
    misses: u64,
    writes: u64,
    evictions: u64,
}

impl RegisterCache {
    pub fn new(ttl_us: u64) -> Self {
        Self {
            entries: BTreeMap::new(),
            ttl_us,
            hits: 0,
            misses: 0,
            writes: 0,
            evictions: 0,
        }
    }

    pub fn read(&mut self, offset: u32, now_us: u64) -> Option<u32> {
        if let Some(entry) = self.entries.get(&offset) {
            let age = now_us.saturating_sub(entry.timestamp_us);
            if age <= self.ttl_us {
                self.hits += 1;
                return Some(entry.value);
            }
        }
        self.misses += 1;
        None
    }

    pub fn write(&mut self, offset: u32, value: u32, now_us: u64) {
        self.entries.insert(offset, CacheEntry {
            value,
            timestamp_us: now_us,
            dirty: true,
        });
        self.writes += 1;
    }

    pub fn update(&mut self, offset: u32, value: u32, now_us: u64) {
        self.entries.insert(offset, CacheEntry {
            value,
            timestamp_us: now_us,
            dirty: false,
        });
    }

    pub fn invalidate(&mut self, offset: u32) -> bool {
        if self.entries.remove(&offset).is_some() {
            self.evictions += 1;
            true
        } else {
            false
        }
    }

    pub fn invalidate_all(&mut self) {
        let count = self.entries.len() as u64;
        self.entries.clear();
        self.evictions += count;
    }

    pub fn flush_dirty(&mut self) -> Vec<(u32, u32)> {
        let dirty: Vec<(u32, u32)> = self.entries
            .iter()
            .filter(|(_, e)| e.dirty)
            .map(|(&off, e)| (off, e.value))
            .collect();
        for entry in self.entries.values_mut() {
            entry.dirty = false;
        }
        dirty
    }

    pub fn get(&self, offset: u32) -> Option<&CacheEntry> {
        self.entries.get(&offset)
    }

    pub fn contains(&self, offset: u32) -> bool {
        self.entries.contains_key(&offset)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn is_stale(&self, offset: u32, now_us: u64) -> bool {
        match self.entries.get(&offset) {
            Some(e) => now_us.saturating_sub(e.timestamp_us) > self.ttl_us,
            None => true,
        }
    }

    pub fn ttl_us(&self) -> u64 {
        self.ttl_us
    }

    pub fn hits(&self) -> u64 {
        self.hits
    }

    pub fn misses(&self) -> u64 {
        self.misses
    }

    pub fn writes(&self) -> u64 {
        self.writes
    }

    pub fn evictions(&self) -> u64 {
        self.evictions
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }

    pub fn prefill_csrs(&mut self, now_us: u64) {
        for &offset in &csr_map::CSR_OFFSETS {
            self.update(offset, 0, now_us);
        }
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.entries.len(),
            hits: self.hits,
            misses: self.misses,
            writes: self.writes,
            evictions: self.evictions,
            hit_rate: self.hit_rate(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CacheStats {
    pub entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub writes: u64,
    pub evictions: u64,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TTL: u64 = 1000;

    #[test]
    fn new_cache_is_empty() {
        let c = RegisterCache::new(TEST_TTL);
        assert!(c.is_empty());
        assert_eq!(c.ttl_us(), TEST_TTL);
    }

    #[test]
    fn read_miss_then_write_then_hit() {
        let mut c = RegisterCache::new(TEST_TTL);
        assert!(c.read(0x10, 0).is_none());
        assert_eq!(c.misses(), 1);
        c.write(0x10, 42, 0);
        assert_eq!(c.read(0x10, 100).unwrap(), 42);
        assert_eq!(c.hits(), 1);
    }

    #[test]
    fn ttl_expiry() {
        let mut c = RegisterCache::new(100);
        c.write(0x10, 99, 0);
        assert_eq!(c.read(0x10, 50).unwrap(), 99);
        assert!(c.read(0x10, 200).is_none());
    }

    #[test]
    fn invalidate() {
        let mut c = RegisterCache::new(TEST_TTL);
        c.write(0x10, 1, 0);
        assert!(c.invalidate(0x10));
        assert!(!c.contains(0x10));
        assert!(!c.invalidate(0x10));
    }

    #[test]
    fn invalidate_all() {
        let mut c = RegisterCache::new(TEST_TTL);
        c.write(0x10, 1, 0);
        c.write(0x14, 2, 0);
        c.invalidate_all();
        assert!(c.is_empty());
        assert_eq!(c.evictions(), 2);
    }

    #[test]
    fn flush_dirty() {
        let mut c = RegisterCache::new(TEST_TTL);
        c.write(0x10, 0xAA, 0);
        c.write(0x14, 0xBB, 0);
        c.update(0x18, 0xCC, 0);
        let dirty = c.flush_dirty();
        assert_eq!(dirty.len(), 2);
        assert!(dirty.contains(&(0x10, 0xAA)));
        assert!(dirty.contains(&(0x14, 0xBB)));
        assert_eq!(c.get(0x10).unwrap().dirty, false);
    }

    #[test]
    fn update_marks_not_dirty() {
        let mut c = RegisterCache::new(TEST_TTL);
        c.update(0x10, 42, 0);
        assert_eq!(c.get(0x10).unwrap().dirty, false);
    }

    #[test]
    fn is_stale() {
        let mut c = RegisterCache::new(100);
        c.write(0x10, 1, 0);
        assert!(!c.is_stale(0x10, 50));
        assert!(c.is_stale(0x10, 200));
        assert!(c.is_stale(0x20, 0));
    }

    #[test]
    fn hit_rate() {
        let mut c = RegisterCache::new(TEST_TTL);
        assert_eq!(c.hit_rate(), 0.0);
        c.write(0x10, 1, 0);
        c.read(0x10, 0);
        c.read(0x14, 0);
        assert!((c.hit_rate() - 0.5).abs() < 0.001);
    }

    #[test]
    fn prefill_csrs() {
        let mut c = RegisterCache::new(TEST_TTL);
        c.prefill_csrs(0);
        assert_eq!(c.len(), csr_map::CSR_COUNT);
        for &offset in &csr_map::CSR_OFFSETS {
            assert!(c.contains(offset));
        }
    }

    #[test]
    fn stats() {
        let mut c = RegisterCache::new(TEST_TTL);
        c.write(0x10, 1, 0);
        c.read(0x10, 0);
        let s = c.stats();
        assert_eq!(s.entries, 1);
        assert_eq!(s.hits, 1);
        assert_eq!(s.misses, 0);
    }

    #[test]
    fn error_display() {
        let e = CacheError::NotCached { offset: 0x10 };
        assert!(e.to_string().contains("0x10"));
        let e = CacheError::Stale { offset: 0x10, age_us: 200, max_us: 100 };
        assert!(e.to_string().contains("stale"));
    }
}

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub enum IdError {
    Exhausted { prefix: u64 },
    AlreadyAllocated { id: u64 },
    NotAllocated { id: u64 },
    InvalidRange { prefix: u64 },
}

impl std::fmt::Display for IdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdError::Exhausted { prefix } => write!(f, "prefix {prefix} exhausted"),
            IdError::AlreadyAllocated { id } => write!(f, "id {id} already allocated"),
            IdError::NotAllocated { id } => write!(f, "id {id} not allocated"),
            IdError::InvalidRange { prefix } => write!(f, "prefix {prefix} invalid"),
        }
    }
}

impl std::error::Error for IdError {}

#[derive(Debug, Clone)]
pub struct IdRange {
    pub prefix: u64,
    pub base: u64,
    pub count: u64,
    pub allocated: u64,
    pub available: u64,
}

struct PrefixPool {
    prefix: u64,
    base: u64,
    count: u64,
    next: u64,
    free_list: Vec<u64>,
    allocated: BTreeSet<u64>,
}

pub struct IdAllocator {
    pools: BTreeMap<u64, PrefixPool>,
    total_allocated: u64,
    total_freed: u64,
}

impl IdAllocator {
    pub fn new() -> Self { Self { pools: BTreeMap::new(), total_allocated: 0, total_freed: 0 } }

    pub fn register_range(&mut self, prefix: u64, base: u64, count: u64) -> Result<(), IdError> {
        if count == 0 { return Err(IdError::InvalidRange { prefix }); }
        if self.pools.contains_key(&prefix) { return Err(IdError::AlreadyAllocated { id: prefix }); }
        self.pools.insert(prefix, PrefixPool { prefix, base, count, next: 0, free_list: Vec::new(), allocated: BTreeSet::new() });
        Ok(())
    }

    pub fn allocate(&mut self, prefix: u64) -> Result<u64, IdError> {
        let pool = self.pools.get_mut(&prefix).ok_or(IdError::InvalidRange { prefix })?;
        if let Some(id) = pool.free_list.pop() {
            pool.allocated.insert(id);
            self.total_allocated += 1;
            return Ok(id);
        }
        if pool.next >= pool.count { return Err(IdError::Exhausted { prefix }); }
        let id = pool.base + pool.next;
        pool.next += 1;
        pool.allocated.insert(id);
        self.total_allocated += 1;
        Ok(id)
    }

    pub fn allocate_batch(&mut self, prefix: u64, n: usize) -> Result<Vec<u64>, IdError> {
        let mut ids = Vec::with_capacity(n);
        for _ in 0..n { ids.push(self.allocate(prefix)?); }
        Ok(ids)
    }

    pub fn free(&mut self, prefix: u64, id: u64) -> Result<(), IdError> {
        let pool = self.pools.get_mut(&prefix).ok_or(IdError::InvalidRange { prefix })?;
        if !pool.allocated.remove(&id) { return Err(IdError::NotAllocated { id }); }
        pool.free_list.push(id);
        self.total_freed += 1;
        Ok(())
    }

    pub fn is_allocated(&self, id: u64) -> bool {
        self.pools.values().any(|p| p.allocated.contains(&id))
    }

    pub fn range_info(&self, prefix: u64) -> Option<IdRange> {
        self.pools.get(&prefix).map(|p| IdRange {
            prefix: p.prefix, base: p.base, count: p.count,
            allocated: p.allocated.len() as u64,
            available: p.count - p.allocated.len() as u64,
        })
    }

    pub fn prefix_count(&self) -> usize { self.pools.len() }
    pub fn total_allocated(&self) -> u64 { self.total_allocated }
    pub fn total_freed(&self) -> u64 { self.total_freed }
    pub fn total_available(&self) -> u64 {
        self.pools.values().map(|p| p.count - p.allocated.len() as u64).sum()
    }
}

impl Default for IdAllocator {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_allocator() {
        let ia = IdAllocator::new();
        assert_eq!(ia.prefix_count(), 0);
    }

    #[test]
    fn register_and_allocate() {
        let mut ia = IdAllocator::new();
        ia.register_range(1, 100, 10).unwrap();
        let id = ia.allocate(1).unwrap();
        assert_eq!(id, 100);
        assert!(ia.is_allocated(100));
    }

    #[test]
    fn sequential_ids() {
        let mut ia = IdAllocator::new();
        ia.register_range(1, 0, 100).unwrap();
        let ids = ia.allocate_batch(1, 5).unwrap();
        assert_eq!(ids, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn free_and_reuse() {
        let mut ia = IdAllocator::new();
        ia.register_range(1, 0, 10).unwrap();
        let id = ia.allocate(1).unwrap();
        ia.free(1, id).unwrap();
        let id2 = ia.allocate(1).unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn exhausted() {
        let mut ia = IdAllocator::new();
        ia.register_range(1, 0, 2).unwrap();
        ia.allocate(1).unwrap();
        ia.allocate(1).unwrap();
        let err = ia.allocate(1).unwrap_err();
        assert!(matches!(err, IdError::Exhausted { .. }));
    }

    #[test]
    fn double_free() {
        let mut ia = IdAllocator::new();
        ia.register_range(1, 0, 10).unwrap();
        let id = ia.allocate(1).unwrap();
        ia.free(1, id).unwrap();
        let err = ia.free(1, id).unwrap_err();
        assert!(matches!(err, IdError::NotAllocated { .. }));
    }

    #[test]
    fn invalid_prefix() {
        let mut ia = IdAllocator::new();
        let err = ia.allocate(99).unwrap_err();
        assert!(matches!(err, IdError::InvalidRange { .. }));
    }

    #[test]
    fn range_info() {
        let mut ia = IdAllocator::new();
        ia.register_range(1, 100, 50).unwrap();
        ia.allocate(1).unwrap();
        let info = ia.range_info(1).unwrap();
        assert_eq!(info.allocated, 1);
        assert_eq!(info.available, 49);
    }

    #[test]
    fn multiple_prefixes() {
        let mut ia = IdAllocator::new();
        ia.register_range(1, 0, 100).unwrap();
        ia.register_range(2, 1000, 50).unwrap();
        let a = ia.allocate(1).unwrap();
        let b = ia.allocate(2).unwrap();
        assert!(a < 100);
        assert!(b >= 1000);
    }

    #[test]
    fn stats() {
        let mut ia = IdAllocator::new();
        ia.register_range(1, 0, 100).unwrap();
        let id = ia.allocate(1).unwrap();
        ia.free(1, id).unwrap();
        assert_eq!(ia.total_allocated(), 1);
        assert_eq!(ia.total_freed(), 1);
    }

    #[test]
    fn zero_count() {
        let mut ia = IdAllocator::new();
        let err = ia.register_range(1, 0, 0).unwrap_err();
        assert!(matches!(err, IdError::InvalidRange { .. }));
    }

    #[test]
    fn error_display() {
        assert!(IdError::Exhausted { prefix: 1 }.to_string().contains("1"));
    }
}

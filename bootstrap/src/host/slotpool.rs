use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SlotPoolError {
    PoolFull { capacity: usize },
    SlotNotAllocated { id: u64 },
    SlotAlreadyFree { id: u64 },
}

impl std::fmt::Display for SlotPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlotPoolError::PoolFull { capacity } => write!(f, "pool full ({capacity})"),
            SlotPoolError::SlotNotAllocated { id } => write!(f, "slot {id} not allocated"),
            SlotPoolError::SlotAlreadyFree { id } => write!(f, "slot {id} already free"),
        }
    }
}

impl std::error::Error for SlotPoolError {}

struct Slot {
    id: u64,
    allocated: bool,
    owner: Option<u64>,
    alloc_count: u64,
}

pub struct SlotPool {
    slots: BTreeMap<u64, Slot>,
    free_list: Vec<u64>,
    capacity: usize,
    next_id: u64,
    high_water_mark: usize,
    total_allocations: u64,
    total_releases: u64,
    total_reuses: u64,
}

impl SlotPool {
    pub fn new(capacity: usize) -> Self {
        let mut slots = BTreeMap::new();
        let mut free_list = Vec::with_capacity(capacity);
        for i in 0..capacity {
            let id = i as u64;
            slots.insert(id, Slot { id, allocated: false, owner: None, alloc_count: 0 });
            free_list.push(id);
        }
        Self { slots, free_list, capacity, next_id: capacity as u64, high_water_mark: 0, total_allocations: 0, total_releases: 0, total_reuses: 0 }
    }

    pub fn allocate(&mut self, owner: u64) -> Result<u64, SlotPoolError> {
        let id = self.free_list.pop().ok_or(SlotPoolError::PoolFull { capacity: self.capacity })?;
        let slot = self.slots.get_mut(&id).unwrap();
        let was_reused = slot.alloc_count > 0;
        slot.allocated = true;
        slot.owner = Some(owner);
        slot.alloc_count += 1;
        if was_reused { self.total_reuses += 1; }
        self.total_allocations += 1;
        let used = self.capacity - self.free_list.len();
        if used > self.high_water_mark { self.high_water_mark = used; }
        Ok(id)
    }

    pub fn release(&mut self, id: u64) -> Result<(), SlotPoolError> {
        let slot = self.slots.get_mut(&id).ok_or(SlotPoolError::SlotNotAllocated { id })?;
        if !slot.allocated { return Err(SlotPoolError::SlotAlreadyFree { id }); }
        slot.allocated = false;
        slot.owner = None;
        self.free_list.push(id);
        self.total_releases += 1;
        Ok(())
    }

    pub fn is_allocated(&self, id: u64) -> bool { self.slots.get(&id).map(|s| s.allocated).unwrap_or(false) }
    pub fn owner(&self, id: u64) -> Option<u64> { self.slots.get(&id).and_then(|s| s.owner) }
    pub fn alloc_count(&self, id: u64) -> Option<u64> { self.slots.get(&id).map(|s| s.alloc_count) }

    pub fn by_owner(&self, owner: u64) -> Vec<u64> {
        self.slots.values().filter(|s| s.owner == Some(owner)).map(|s| s.id).collect()
    }

    pub fn used_count(&self) -> usize { self.capacity - self.free_list.len() }
    pub fn free_count(&self) -> usize { self.free_list.len() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn high_water_mark(&self) -> usize { self.high_water_mark }
    pub fn total_allocations(&self) -> u64 { self.total_allocations }
    pub fn total_releases(&self) -> u64 { self.total_releases }
    pub fn total_reuses(&self) -> u64 { self.total_reuses }
    pub fn utilization(&self) -> f64 { if self.capacity == 0 { 0.0 } else { self.used_count() as f64 / self.capacity as f64 } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool() { let p = SlotPool::new(10); assert_eq!(p.free_count(), 10); }

    #[test]
    fn allocate() {
        let mut p = SlotPool::new(10);
        let id = p.allocate(1).unwrap();
        assert!(p.is_allocated(id));
        assert_eq!(p.owner(id), Some(1));
        assert_eq!(p.used_count(), 1);
    }

    #[test]
    fn release() {
        let mut p = SlotPool::new(10);
        let id = p.allocate(1).unwrap();
        p.release(id).unwrap();
        assert!(!p.is_allocated(id));
        assert_eq!(p.free_count(), 10);
    }

    #[test]
    fn reuse() {
        let mut p = SlotPool::new(10);
        let id = p.allocate(1).unwrap();
        p.release(id).unwrap();
        let id2 = p.allocate(2).unwrap();
        assert_eq!(id, id2);
        assert!(p.total_reuses() >= 1);
    }

    #[test]
    fn pool_full() {
        let mut p = SlotPool::new(2);
        p.allocate(1).unwrap(); p.allocate(1).unwrap();
        let err = p.allocate(1).unwrap_err();
        assert!(matches!(err, SlotPoolError::PoolFull { .. }));
    }

    #[test]
    fn double_release() {
        let mut p = SlotPool::new(10);
        let id = p.allocate(1).unwrap();
        p.release(id).unwrap();
        let err = p.release(id).unwrap_err();
        assert!(matches!(err, SlotPoolError::SlotAlreadyFree { .. }));
    }

    #[test]
    fn by_owner() {
        let mut p = SlotPool::new(10);
        p.allocate(1).unwrap(); p.allocate(1).unwrap(); p.allocate(2).unwrap();
        assert_eq!(p.by_owner(1).len(), 2);
        assert_eq!(p.by_owner(2).len(), 1);
    }

    #[test]
    fn high_water_mark() {
        let mut p = SlotPool::new(10);
        let a = p.allocate(1).unwrap();
        let b = p.allocate(1).unwrap();
        let c = p.allocate(1).unwrap();
        assert_eq!(p.high_water_mark(), 3);
        p.release(a).unwrap();
        assert_eq!(p.high_water_mark(), 3);
    }

    #[test]
    fn stats() {
        let mut p = SlotPool::new(10);
        let id = p.allocate(1).unwrap();
        p.release(id).unwrap();
        assert_eq!(p.total_allocations(), 1);
        assert_eq!(p.total_releases(), 1);
    }

    #[test]
    fn error_display() { assert!(SlotPoolError::PoolFull { capacity: 5 }.to_string().contains("5")); }
}

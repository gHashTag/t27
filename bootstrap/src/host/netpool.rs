use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum NpError {
    PoolFull { capacity: usize },
    SlotNotFound { id: u64 },
    SlotNotHeld { id: u64 },
    SlotAlreadyFree { id: u64 },
}

impl std::fmt::Display for NpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NpError::PoolFull { capacity } => write!(f, "pool full ({capacity})"),
            NpError::SlotNotFound { id } => write!(f, "slot {id} not found"),
            NpError::SlotNotHeld { id } => write!(f, "slot {id} not held"),
            NpError::SlotAlreadyFree { id } => write!(f, "slot {id} already free"),
        }
    }
}

impl std::error::Error for NpError {}

#[derive(Debug, Clone, PartialEq)]
pub enum SlotState { Free, Held }

struct Slot {
    id: u64,
    state: SlotState,
    data: Vec<u8>,
    last_used: u64,
    hold_count: u64,
}

pub struct NetPool {
    slots: BTreeMap<u64, Slot>,
    capacity: usize,
    next_id: u64,
    idle_timeout: u64,
    total_acquires: u64,
    total_releases: u64,
    total_evictions: u64,
}

impl NetPool {
    pub fn new(capacity: usize, idle_timeout: u64) -> Self {
        Self { slots: BTreeMap::new(), capacity, next_id: 1, idle_timeout, total_acquires: 0, total_releases: 0, total_evictions: 0 }
    }

    pub fn seed(&mut self, data: Vec<u8>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.slots.insert(id, Slot { id, state: SlotState::Free, data, last_used: 1, hold_count: 0 });
        id
    }

    pub fn acquire(&mut self, now: u64) -> Result<(u64, Vec<u8>), NpError> {
        for (&id, slot) in &self.slots {
            if slot.state == SlotState::Free {
                let id = id;
                let slot = self.slots.get_mut(&id).unwrap();
                slot.state = SlotState::Held;
                slot.last_used = now;
                slot.hold_count += 1;
                self.total_acquires += 1;
                return Ok((id, slot.data.clone()));
            }
        }
        Err(NpError::PoolFull { capacity: self.capacity })
    }

    pub fn release(&mut self, id: u64, data: Vec<u8>, now: u64) -> Result<(), NpError> {
        let slot = self.slots.get_mut(&id).ok_or(NpError::SlotNotFound { id })?;
        if slot.state != SlotState::Held { return Err(NpError::SlotNotHeld { id }); }
        slot.state = SlotState::Free;
        slot.data = data;
        slot.last_used = now;
        self.total_releases += 1;
        Ok(())
    }

    pub fn evict_idle(&mut self, now: u64) -> Vec<u64> {
        let to_evict: Vec<u64> = self.slots.iter()
            .filter(|(_, s)| s.state == SlotState::Free && now - s.last_used >= self.idle_timeout && s.last_used > 0)
            .map(|(&id, _)| id)
            .collect();
        for &id in &to_evict {
            self.slots.remove(&id);
            self.total_evictions += 1;
        }
        to_evict
    }

    pub fn get(&self, id: u64) -> Option<&[u8]> { self.slots.get(&id).map(|s| s.data.as_slice()) }
    pub fn slot_state(&self, id: u64) -> Option<&SlotState> { self.slots.get(&id).map(|s| &s.state) }
    pub fn hold_count(&self, id: u64) -> Option<u64> { self.slots.get(&id).map(|s| s.hold_count) }
    pub fn free_count(&self) -> usize { self.slots.values().filter(|s| s.state == SlotState::Free).count() }
    pub fn held_count(&self) -> usize { self.slots.values().filter(|s| s.state == SlotState::Held).count() }
    pub fn slot_count(&self) -> usize { self.slots.len() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn total_acquires(&self) -> u64 { self.total_acquires }
    pub fn total_releases(&self) -> u64 { self.total_releases }
    pub fn total_evictions(&self) -> u64 { self.total_evictions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool() { let p = NetPool::new(4, 100); assert_eq!(p.capacity(), 4); }

    #[test]
    fn seed_acquire() {
        let mut p = NetPool::new(2, 100);
        p.seed(b"obj1".to_vec());
        let (id, data) = p.acquire(0).unwrap();
        assert_eq!(data, b"obj1");
        assert_eq!(p.slot_state(id), Some(&SlotState::Held));
    }

    #[test]
    fn release_recycle() {
        let mut p = NetPool::new(2, 100);
        p.seed(b"obj".to_vec());
        let (id, _) = p.acquire(0).unwrap();
        p.release(id, b"updated".to_vec(), 10).unwrap();
        assert_eq!(p.slot_state(id), Some(&SlotState::Free));
        let (_, data) = p.acquire(20).unwrap();
        assert_eq!(data, b"updated");
    }

    #[test]
    fn pool_full() {
        let mut p = NetPool::new(1, 100);
        p.seed(b"a".to_vec());
        p.acquire(0).unwrap();
        let err = p.acquire(0).unwrap_err();
        assert!(matches!(err, NpError::PoolFull { .. }));
    }

    #[test]
    fn evict_idle() {
        let mut p = NetPool::new(4, 50);
        let id1 = p.seed(b"a".to_vec());
        let id2 = p.seed(b"b".to_vec());
        let _ = p.acquire(0).unwrap();
        p.release(id1, b"a".to_vec(), 10).unwrap();
        let evicted = p.evict_idle(100);
        assert!(evicted.contains(&id1));
        assert!(evicted.contains(&id2));
    }

    #[test]
    fn evict_skips_held() {
        let mut p = NetPool::new(4, 50);
        let id = p.seed(b"a".to_vec());
        p.acquire(0).unwrap();
        let evicted = p.evict_idle(100);
        assert!(evicted.is_empty());
        assert_eq!(p.slot_state(id), Some(&SlotState::Held));
    }

    #[test]
    fn hold_count() {
        let mut p = NetPool::new(2, 100);
        let id = p.seed(b"x".to_vec());
        p.acquire(0).unwrap();
        p.release(id, b"x".to_vec(), 10).unwrap();
        p.acquire(20).unwrap();
        assert_eq!(p.hold_count(id), Some(2));
    }

    #[test]
    fn not_found() {
        let mut p = NetPool::new(2, 100);
        let err = p.release(99, b"x".to_vec(), 0).unwrap_err();
        assert!(matches!(err, NpError::SlotNotFound { .. }));
    }

    #[test]
    fn double_release() {
        let mut p = NetPool::new(2, 100);
        let id = p.seed(b"x".to_vec());
        p.acquire(0).unwrap();
        p.release(id, b"x".to_vec(), 10).unwrap();
        let err = p.release(id, b"x".to_vec(), 20).unwrap_err();
        assert!(matches!(err, NpError::SlotNotHeld { .. }));
    }

    #[test]
    fn stats() {
        let mut p = NetPool::new(2, 100);
        p.seed(b"x".to_vec());
        let (id, _) = p.acquire(0).unwrap();
        p.release(id, b"x".to_vec(), 10).unwrap();
        assert_eq!(p.total_acquires(), 1);
        assert_eq!(p.total_releases(), 1);
    }

    #[test]
    fn error_display() { assert!(NpError::PoolFull { capacity: 3 }.to_string().contains("3")); }
}

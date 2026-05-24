use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum PoolError {
    PoolFull { capacity: usize },
    ResourceNotFound { id: u64 },
    AlreadyCheckedOut { id: u64 },
    AlreadyCheckedIn { id: u64 },
    NotActive { id: u64 },
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolError::PoolFull { capacity } => write!(f, "pool full ({capacity})"),
            PoolError::ResourceNotFound { id } => write!(f, "resource {id} not found"),
            PoolError::AlreadyCheckedOut { id } => write!(f, "resource {id} checked out"),
            PoolError::AlreadyCheckedIn { id } => write!(f, "resource {id} checked in"),
            PoolError::NotActive { id } => write!(f, "resource {id} not active"),
        }
    }
}

impl std::error::Error for PoolError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ResState { Available, CheckedOut, Evicted }

struct Resource {
    id: u64,
    state: ResState,
    checkout_count: u64,
    last_checkout_tick: u64,
}

pub struct ResourcePool {
    resources: BTreeMap<u64, Resource>,
    available: VecDeque<u64>,
    capacity: usize,
    current_tick: u64,
    max_idle_ticks: u64,
    total_checkouts: u64,
    total_checkins: u64,
    total_evictions: u64,
    total_created: u64,
}

impl ResourcePool {
    pub fn new(capacity: usize, max_idle_ticks: u64) -> Self {
        Self { resources: BTreeMap::new(), available: VecDeque::new(), capacity, current_tick: 0, max_idle_ticks, total_checkouts: 0, total_checkins: 0, total_evictions: 0, total_created: 0 }
    }

    pub fn create(&mut self, id: u64) -> Result<(), PoolError> {
        if self.resources.len() >= self.capacity { return Err(PoolError::PoolFull { capacity: self.capacity }); }
        if self.resources.contains_key(&id) { return Err(PoolError::ResourceNotFound { id }); }
        self.resources.insert(id, Resource { id, state: ResState::Available, checkout_count: 0, last_checkout_tick: 0 });
        self.available.push_back(id);
        self.total_created += 1;
        Ok(())
    }

    pub fn checkout(&mut self) -> Option<u64> {
        let id = self.available.pop_front()?;
        let r = self.resources.get_mut(&id)?;
        r.state = ResState::CheckedOut;
        r.checkout_count += 1;
        r.last_checkout_tick = self.current_tick;
        self.total_checkouts += 1;
        Some(id)
    }

    pub fn checkin(&mut self, id: u64) -> Result<(), PoolError> {
        let r = self.resources.get_mut(&id).ok_or(PoolError::ResourceNotFound { id })?;
        if r.state != ResState::CheckedOut { return Err(PoolError::AlreadyCheckedIn { id }); }
        r.state = ResState::Available;
        self.available.push_back(id);
        self.total_checkins += 1;
        Ok(())
    }

    pub fn evict(&mut self, id: u64) -> Result<(), PoolError> {
        let r = self.resources.get_mut(&id).ok_or(PoolError::ResourceNotFound { id })?;
        if r.state == ResState::CheckedOut { return Err(PoolError::AlreadyCheckedOut { id }); }
        r.state = ResState::Evicted;
        self.available.retain(|&x| x != id);
        self.total_evictions += 1;
        Ok(())
    }

    pub fn tick(&mut self) -> Vec<u64> {
        self.current_tick += 1;
        let threshold = self.current_tick.saturating_sub(self.max_idle_ticks);
        let to_evict: Vec<u64> = self.resources.iter()
            .filter(|(_, r)| r.state == ResState::Available && r.last_checkout_tick < threshold)
            .map(|(&id, _)| id)
            .collect();
        for id in &to_evict {
            if let Some(r) = self.resources.get_mut(id) {
                r.state = ResState::Evicted;
                self.available.retain(|&x| x != *id);
                self.total_evictions += 1;
            }
        }
        to_evict
    }

    pub fn state(&self, id: u64) -> Option<&ResState> { self.resources.get(&id).map(|r| &r.state) }
    pub fn available_count(&self) -> usize { self.available.len() }
    pub fn active_count(&self) -> usize { self.resources.values().filter(|r| r.state != ResState::Evicted).count() }
    pub fn total_checkouts(&self) -> u64 { self.total_checkouts }
    pub fn total_checkins(&self) -> u64 { self.total_checkins }
    pub fn total_evictions(&self) -> u64 { self.total_evictions }
    pub fn total_created(&self) -> u64 { self.total_created }
    pub fn current_tick(&self) -> u64 { self.current_tick }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool() { let p = ResourcePool::new(10, 100); assert_eq!(p.available_count(), 0); }

    #[test]
    fn create_checkout_checkin() {
        let mut p = ResourcePool::new(10, 100);
        p.create(1).unwrap();
        let id = p.checkout().unwrap();
        assert_eq!(id, 1);
        assert_eq!(p.state(1), Some(&ResState::CheckedOut));
        p.checkin(1).unwrap();
        assert_eq!(p.state(1), Some(&ResState::Available));
    }

    #[test]
    fn pool_full() {
        let mut p = ResourcePool::new(1, 100);
        p.create(1).unwrap();
        let err = p.create(2).unwrap_err();
        assert!(matches!(err, PoolError::PoolFull { .. }));
    }

    #[test]
    fn double_checkin() {
        let mut p = ResourcePool::new(10, 100);
        p.create(1).unwrap();
        let err = p.checkin(1).unwrap_err();
        assert!(matches!(err, PoolError::AlreadyCheckedIn { .. }));
    }

    #[test]
    fn evict() {
        let mut p = ResourcePool::new(10, 100);
        p.create(1).unwrap();
        p.evict(1).unwrap();
        assert_eq!(p.state(1), Some(&ResState::Evicted));
        assert_eq!(p.available_count(), 0);
    }

    #[test]
    fn evict_checkedout() {
        let mut p = ResourcePool::new(10, 100);
        p.create(1).unwrap();
        p.checkout().unwrap();
        let err = p.evict(1).unwrap_err();
        assert!(matches!(err, PoolError::AlreadyCheckedOut { .. }));
    }

    #[test]
    fn idle_eviction() {
        let mut p = ResourcePool::new(10, 3);
        p.create(1).unwrap();
        p.checkout().unwrap();
        p.checkin(1).unwrap();
        p.tick(); p.tick(); p.tick(); p.tick();
        assert!(p.total_evictions() > 0);
    }

    #[test]
    fn not_found() {
        let mut p = ResourcePool::new(10, 100);
        let err = p.checkin(99).unwrap_err();
        assert!(matches!(err, PoolError::ResourceNotFound { .. }));
    }

    #[test]
    fn checkout_empty() {
        let mut p: ResourcePool = ResourcePool::new(10, 100);
        assert!(p.checkout().is_none());
    }

    #[test]
    fn stats() {
        let mut p = ResourcePool::new(10, 100);
        p.create(1).unwrap();
        p.checkout(); p.checkin(1).unwrap();
        assert_eq!(p.total_checkouts(), 1);
        assert_eq!(p.total_checkins(), 1);
        assert_eq!(p.total_created(), 1);
    }

    #[test]
    fn error_display() { assert!(PoolError::PoolFull { capacity: 5 }.to_string().contains("5")); }
}

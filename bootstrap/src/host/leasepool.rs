use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum LpError {
    PoolFull { capacity: usize },
    LeaseNotFound { id: u64 },
    AlreadyExpired { id: u64 },
    NotHeld { id: u64 },
}

impl std::fmt::Display for LpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LpError::PoolFull { capacity } => write!(f, "pool full ({capacity})"),
            LpError::LeaseNotFound { id } => write!(f, "lease {id} not found"),
            LpError::AlreadyExpired { id } => write!(f, "lease {id} expired"),
            LpError::NotHeld { id } => write!(f, "lease {id} not held"),
        }
    }
}

impl std::error::Error for LpError {}

#[derive(Debug, Clone, PartialEq)]
pub enum LeaseState { Held, Released, Expired }

struct Lease {
    id: u64,
    resource: String,
    holder: u64,
    acquired_at: u64,
    expires_at: u64,
    state: LeaseState,
    renew_count: u64,
}

pub struct LeasePool {
    leases: BTreeMap<u64, Lease>,
    capacity: usize,
    next_id: u64,
    default_ttl: u64,
    total_acquired: u64,
    total_released: u64,
    total_expired: u64,
    total_renewed: u64,
}

impl LeasePool {
    pub fn new(capacity: usize, default_ttl: u64) -> Self { Self { leases: BTreeMap::new(), capacity, next_id: 1, default_ttl, total_acquired: 0, total_released: 0, total_expired: 0, total_renewed: 0 } }

    pub fn acquire(&mut self, holder: u64, resource: &str, ttl: Option<u64>) -> Result<u64, LpError> {
        let active = self.leases.values().filter(|l| l.state == LeaseState::Held).count();
        if active >= self.capacity { return Err(LpError::PoolFull { capacity: self.capacity }); }
        let id = self.next_id;
        self.next_id += 1;
        let now = 0;
        let expires = now + ttl.unwrap_or(self.default_ttl);
        self.leases.insert(id, Lease { id, resource: resource.to_string(), holder, acquired_at: now, expires_at: expires, state: LeaseState::Held, renew_count: 0 });
        self.total_acquired += 1;
        Ok(id)
    }

    pub fn release(&mut self, id: u64) -> Result<(), LpError> {
        let l = self.leases.get_mut(&id).ok_or(LpError::LeaseNotFound { id })?;
        if l.state != LeaseState::Held { return Err(LpError::NotHeld { id }); }
        l.state = LeaseState::Released;
        self.total_released += 1;
        Ok(())
    }

    pub fn renew(&mut self, id: u64, ttl: Option<u64>) -> Result<u64, LpError> {
        let l = self.leases.get_mut(&id).ok_or(LpError::LeaseNotFound { id })?;
        if l.state == LeaseState::Expired { return Err(LpError::AlreadyExpired { id }); }
        if l.state != LeaseState::Held { return Err(LpError::NotHeld { id }); }
        let extension = ttl.unwrap_or(self.default_ttl);
        l.expires_at += extension;
        l.renew_count += 1;
        self.total_renewed += 1;
        Ok(l.expires_at)
    }

    pub fn tick(&mut self, now: u64) -> Vec<u64> {
        let mut expired = Vec::new();
        for (&id, l) in &self.leases {
            if l.state == LeaseState::Held && now >= l.expires_at {
                expired.push(id);
            }
        }
        for id in &expired {
            if let Some(l) = self.leases.get_mut(id) {
                l.state = LeaseState::Expired;
                self.total_expired += 1;
            }
        }
        expired
    }

    pub fn state(&self, id: u64) -> Option<&LeaseState> { self.leases.get(&id).map(|l| &l.state) }
    pub fn holder(&self, id: u64) -> Option<u64> { self.leases.get(&id).map(|l| l.holder) }
    pub fn resource(&self, id: u64) -> Option<&str> { self.leases.get(&id).map(|l| l.resource.as_str()) }
    pub fn expires_at(&self, id: u64) -> Option<u64> { self.leases.get(&id).map(|l| l.expires_at) }
    pub fn renew_count(&self, id: u64) -> Option<u64> { self.leases.get(&id).map(|l| l.renew_count) }
    pub fn active_count(&self) -> usize { self.leases.values().filter(|l| l.state == LeaseState::Held).count() }
    pub fn lease_count(&self) -> usize { self.leases.len() }
    pub fn total_acquired(&self) -> u64 { self.total_acquired }
    pub fn total_released(&self) -> u64 { self.total_released }
    pub fn total_expired(&self) -> u64 { self.total_expired }
    pub fn total_renewed(&self) -> u64 { self.total_renewed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool() { assert_eq!(LeasePool::new(10, 100).active_count(), 0); }

    #[test]
    fn acquire_release() {
        let mut lp = LeasePool::new(10, 100);
        let id = lp.acquire(1, "file", None).unwrap();
        assert_eq!(lp.state(id), Some(&LeaseState::Held));
        lp.release(id).unwrap();
        assert_eq!(lp.state(id), Some(&LeaseState::Released));
    }

    #[test]
    fn expiry() {
        let mut lp = LeasePool::new(10, 5);
        let id = lp.acquire(1, "lock", None).unwrap();
        let expired = lp.tick(10);
        assert!(expired.contains(&id));
        assert_eq!(lp.state(id), Some(&LeaseState::Expired));
    }

    #[test]
    fn renew() {
        let mut lp = LeasePool::new(10, 5);
        let id = lp.acquire(1, "lock", None).unwrap();
        lp.renew(id, None).unwrap();
        let expired = lp.tick(5);
        assert!(!expired.contains(&id));
    }

    #[test]
    fn pool_full() {
        let mut lp = LeasePool::new(1, 100);
        lp.acquire(1, "a", None).unwrap();
        let err = lp.acquire(2, "b", None).unwrap_err();
        assert!(matches!(err, LpError::PoolFull { .. }));
    }

    #[test]
    fn not_found() {
        let mut lp = LeasePool::new(10, 100);
        let err = lp.release(99).unwrap_err();
        assert!(matches!(err, LpError::LeaseNotFound { .. }));
    }

    #[test]
    fn release_expired() {
        let mut lp = LeasePool::new(10, 5);
        let id = lp.acquire(1, "x", None).unwrap();
        lp.tick(10);
        let err = lp.release(id).unwrap_err();
        assert!(matches!(err, LpError::NotHeld { .. }));
    }

    #[test]
    fn renew_expired() {
        let mut lp = LeasePool::new(10, 5);
        let id = lp.acquire(1, "x", None).unwrap();
        lp.tick(10);
        let err = lp.renew(id, None).unwrap_err();
        assert!(matches!(err, LpError::AlreadyExpired { .. }));
    }

    #[test]
    fn holder_resource() {
        let mut lp = LeasePool::new(10, 100);
        let id = lp.acquire(42, "mutex", None).unwrap();
        assert_eq!(lp.holder(id), Some(42));
        assert_eq!(lp.resource(id), Some("mutex"));
    }

    #[test]
    fn stats() {
        let mut lp = LeasePool::new(10, 100);
        let id = lp.acquire(1, "x", None).unwrap();
        lp.renew(id, None).unwrap();
        lp.release(id).unwrap();
        assert_eq!(lp.total_acquired(), 1);
        assert_eq!(lp.total_renewed(), 1);
        assert_eq!(lp.total_released(), 1);
    }

    #[test]
    fn error_display() { assert!(LpError::PoolFull { capacity: 5 }.to_string().contains("5")); }
}

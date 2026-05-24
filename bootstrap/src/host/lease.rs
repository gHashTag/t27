use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaseError {
    AlreadyHeld { resource: u64, holder: u64 },
    NotHeld { resource: u64 },
    WrongHolder { resource: u64, holder: u64 },
    Expired { resource: u64 },
}

impl std::fmt::Display for LeaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeaseError::AlreadyHeld { resource, holder } => write!(f, "res {resource} held by {holder}"),
            LeaseError::NotHeld { resource } => write!(f, "res {resource} not held"),
            LeaseError::WrongHolder { resource, holder } => write!(f, "{holder} does not hold {resource}"),
            LeaseError::Expired { resource } => write!(f, "lease on {resource} expired"),
        }
    }
}

impl std::error::Error for LeaseError {}

#[derive(Debug, Clone)]
struct Lease {
    resource: u64,
    holder: u64,
    acquired_at: u64,
    expires_at: u64,
    renewals: u32,
}

#[derive(Debug, Clone)]
pub struct LeaseInfo {
    pub resource: u64,
    pub holder: u64,
    pub acquired_at: u64,
    pub expires_at: u64,
    pub renewals: u32,
    pub remaining: u64,
}

#[derive(Debug, Clone)]
pub struct LeaseManager {
    leases: BTreeMap<u64, Lease>,
    now: u64,
    total_acquires: u64,
    total_releases: u64,
    total_expirations: u64,
    total_renewals: u64,
}

impl LeaseManager {
    pub fn new() -> Self {
        Self { leases: BTreeMap::new(), now: 0, total_acquires: 0, total_releases: 0, total_expirations: 0, total_renewals: 0 }
    }

    pub fn tick(&mut self) -> u64 {
        self.now += 1;
        self.expire_pass();
        self.now
    }

    pub fn set_now(&mut self, now: u64) {
        self.now = now;
        self.expire_pass();
    }

    pub fn now(&self) -> u64 { self.now }

    pub fn acquire(&mut self, resource: u64, holder: u64, ttl: u64) -> Result<LeaseInfo, LeaseError> {
        if let Some(lease) = self.leases.get(&resource) {
            if lease.expires_at > self.now {
                return Err(LeaseError::AlreadyHeld { resource, holder: lease.holder });
            }
        }
        let lease = Lease { resource, holder, acquired_at: self.now, expires_at: self.now + ttl, renewals: 0 };
        self.leases.insert(resource, lease);
        self.total_acquires += 1;
        Ok(self.lease_info(resource).unwrap())
    }

    pub fn release(&mut self, resource: u64, holder: u64) -> Result<LeaseInfo, LeaseError> {
        let lease = self.leases.get(&resource).ok_or(LeaseError::NotHeld { resource })?;
        if lease.holder != holder {
            return Err(LeaseError::WrongHolder { resource, holder });
        }
        let info = self.lease_info(resource).unwrap();
        self.leases.remove(&resource);
        self.total_releases += 1;
        Ok(info)
    }

    pub fn renew(&mut self, resource: u64, holder: u64, ttl: u64) -> Result<LeaseInfo, LeaseError> {
        let lease = self.leases.get_mut(&resource).ok_or(LeaseError::NotHeld { resource })?;
        if lease.holder != holder {
            return Err(LeaseError::WrongHolder { resource, holder });
        }
        if lease.expires_at <= self.now {
            return Err(LeaseError::Expired { resource });
        }
        lease.expires_at = self.now + ttl;
        lease.renewals += 1;
        self.total_renewals += 1;
        Ok(self.lease_info(resource).unwrap())
    }

    fn expire_pass(&mut self) {
        let expired: Vec<u64> = self.leases.iter()
            .filter(|(_, l)| l.expires_at <= self.now)
            .map(|(&r, _)| r)
            .collect();
        for r in expired {
            self.leases.remove(&r);
            self.total_expirations += 1;
        }
    }

    pub fn is_held(&self, resource: u64) -> bool {
        self.leases.get(&resource).map(|l| l.expires_at > self.now).unwrap_or(false)
    }

    pub fn holder(&self, resource: u64) -> Option<u64> {
        self.leases.get(&resource).filter(|l| l.expires_at > self.now).map(|l| l.holder)
    }

    fn lease_info(&self, resource: u64) -> Option<LeaseInfo> {
        self.leases.get(&resource).map(|l| LeaseInfo {
            resource: l.resource,
            holder: l.holder,
            acquired_at: l.acquired_at,
            expires_at: l.expires_at,
            renewals: l.renewals,
            remaining: l.expires_at.saturating_sub(self.now),
        })
    }

    pub fn active_count(&self) -> usize {
        self.leases.values().filter(|l| l.expires_at > self.now).count()
    }

    pub fn total_acquires(&self) -> u64 { self.total_acquires }
    pub fn total_releases(&self) -> u64 { self.total_releases }
    pub fn total_expirations(&self) -> u64 { self.total_expirations }
    pub fn total_renewals(&self) -> u64 { self.total_renewals }

    pub fn clear(&mut self) {
        self.leases.clear();
    }
}

impl Default for LeaseManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manager() {
        let lm = LeaseManager::new();
        assert_eq!(lm.now(), 0);
        assert_eq!(lm.active_count(), 0);
    }

    #[test]
    fn acquire_and_hold() {
        let mut lm = LeaseManager::new();
        let info = lm.acquire(1, 100, 10).unwrap();
        assert!(lm.is_held(1));
        assert_eq!(lm.holder(1), Some(100));
        assert_eq!(info.remaining, 10);
    }

    #[test]
    fn already_held() {
        let mut lm = LeaseManager::new();
        lm.acquire(1, 100, 10).unwrap();
        let err = lm.acquire(1, 200, 5).unwrap_err();
        assert!(matches!(err, LeaseError::AlreadyHeld { .. }));
    }

    #[test]
    fn release() {
        let mut lm = LeaseManager::new();
        lm.acquire(1, 100, 10).unwrap();
        lm.release(1, 100).unwrap();
        assert!(!lm.is_held(1));
    }

    #[test]
    fn wrong_holder() {
        let mut lm = LeaseManager::new();
        lm.acquire(1, 100, 10).unwrap();
        let err = lm.release(1, 200).unwrap_err();
        assert!(matches!(err, LeaseError::WrongHolder { .. }));
    }

    #[test]
    fn expiry() {
        let mut lm = LeaseManager::new();
        lm.acquire(1, 100, 5).unwrap();
        for _ in 0..5 { lm.tick(); }
        assert!(!lm.is_held(1));
        assert_eq!(lm.total_expirations(), 1);
    }

    #[test]
    fn renew_extends() {
        let mut lm = LeaseManager::new();
        lm.acquire(1, 100, 5).unwrap();
        lm.tick(); lm.tick();
        lm.renew(1, 100, 10).unwrap();
        assert_eq!(lm.lease_info(1).unwrap().remaining, 10);
        assert_eq!(lm.total_renewals(), 1);
    }

    #[test]
    fn renew_expired_fails() {
        let mut lm = LeaseManager::new();
        lm.acquire(1, 100, 2).unwrap();
        lm.set_now(3);
        let err = lm.renew(1, 100, 5).unwrap_err();
        assert!(matches!(err, LeaseError::NotHeld { .. } | LeaseError::Expired { .. }));
    }

    #[test]
    fn multiple_leases() {
        let mut lm = LeaseManager::new();
        lm.acquire(1, 100, 10).unwrap();
        lm.acquire(2, 200, 10).unwrap();
        assert_eq!(lm.active_count(), 2);
    }

    #[test]
    fn stats() {
        let mut lm = LeaseManager::new();
        lm.acquire(1, 100, 5).unwrap();
        lm.release(1, 100).unwrap();
        assert_eq!(lm.total_acquires(), 1);
        assert_eq!(lm.total_releases(), 1);
    }

    #[test]
    fn not_held_release() {
        let mut lm = LeaseManager::new();
        let err = lm.release(99, 1).unwrap_err();
        assert!(matches!(err, LeaseError::NotHeld { .. }));
    }

    #[test]
    fn error_display() {
        assert!(LeaseError::Expired { resource: 5 }.to_string().contains("5"));
    }
}

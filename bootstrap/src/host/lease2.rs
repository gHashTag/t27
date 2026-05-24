use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Lease2Error {
    LeaseExists { id: u64 },
    LeaseNotFound { id: u64 },
    LeaseExpired { id: u64 },
    NotHolder { id: u64, holder: u64 },
    AlreadyRevoked { id: u64 },
}

impl std::fmt::Display for Lease2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lease2Error::LeaseExists { id } => write!(f, "lease {id} exists"),
            Lease2Error::LeaseNotFound { id } => write!(f, "lease {id} not found"),
            Lease2Error::LeaseExpired { id } => write!(f, "lease {id} expired"),
            Lease2Error::NotHolder { id, holder } => write!(f, "lease {id}: not holder {holder}"),
            Lease2Error::AlreadyRevoked { id } => write!(f, "lease {id} revoked"),
        }
    }
}

impl std::error::Error for Lease2Error {}

struct Lease {
    id: u64,
    holder: u64,
    expiry: u64,
    parent: Option<u64>,
    children: Vec<u64>,
    revoked: bool,
    renew_count: u64,
}

pub struct LeaseManager2 {
    leases: BTreeMap<u64, Lease>,
    next_id: u64,
    total_granted: u64,
    total_revoked: u64,
    total_renewed: u64,
    total_expired: u64,
}

impl LeaseManager2 {
    pub fn new() -> Self { Self { leases: BTreeMap::new(), next_id: 1, total_granted: 0, total_revoked: 0, total_renewed: 0, total_expired: 0 } }

    pub fn grant(&mut self, holder: u64, expiry: u64, parent: Option<u64>) -> Result<u64, Lease2Error> {
        if let Some(pid) = parent {
            if !self.leases.contains_key(&pid) { return Err(Lease2Error::LeaseNotFound { id: pid }); }
        }
        let id = self.next_id;
        self.next_id += 1;
        self.leases.insert(id, Lease { id, holder, expiry, parent, children: Vec::new(), revoked: false, renew_count: 0 });
        if let Some(pid) = parent {
            self.leases.get_mut(&pid).unwrap().children.push(id);
        }
        self.total_granted += 1;
        Ok(id)
    }

    pub fn renew(&mut self, id: u64, new_expiry: u64) -> Result<u64, Lease2Error> {
        let l = self.leases.get_mut(&id).ok_or(Lease2Error::LeaseNotFound { id })?;
        if l.revoked { return Err(Lease2Error::AlreadyRevoked { id }); }
        l.expiry = new_expiry;
        l.renew_count += 1;
        self.total_renewed += 1;
        Ok(l.renew_count)
    }

    pub fn revoke(&mut self, id: u64) -> Result<Vec<u64>, Lease2Error> {
        let l = self.leases.get_mut(&id).ok_or(Lease2Error::LeaseNotFound { id })?;
        if l.revoked { return Err(Lease2Error::AlreadyRevoked { id }); }
        l.revoked = true;
        self.total_revoked += 1;
        let children = l.children.clone();
        let mut revoked = vec![id];
        for cid in &children { revoked.extend(self.cascade_revoke(*cid)); }
        Ok(revoked)
    }

    fn cascade_revoke(&mut self, id: u64) -> Vec<u64> {
        let children = self.leases.get(&id).map(|l| l.children.clone()).unwrap_or_default();
        let mut revoked = Vec::new();
        if let Some(l) = self.leases.get_mut(&id) {
            if !l.revoked { l.revoked = true; self.total_revoked += 1; revoked.push(id); }
        }
        for cid in children { revoked.extend(self.cascade_revoke(cid)); }
        revoked
    }

    pub fn tick(&mut self, now: u64) -> Vec<u64> {
        let mut expired = Vec::new();
        for (&id, l) in &self.leases {
            if !l.revoked && l.expiry <= now { expired.push(id); }
        }
        for &id in &expired {
            if let Some(l) = self.leases.get_mut(&id) {
                if !l.revoked { l.revoked = true; self.total_expired += 1; }
            }
        }
        expired
    }

    pub fn is_valid(&self, id: u64) -> bool {
        self.leases.get(&id).map(|l| !l.revoked).unwrap_or(false)
    }

    pub fn holder(&self, id: u64) -> Option<u64> { self.leases.get(&id).map(|l| l.holder) }
    pub fn expiry(&self, id: u64) -> Option<u64> { self.leases.get(&id).map(|l| l.expiry) }
    pub fn renew_count(&self, id: u64) -> Option<u64> { self.leases.get(&id).map(|l| l.renew_count) }
    pub fn active_count(&self) -> usize { self.leases.values().filter(|l| !l.revoked).count() }
    pub fn lease_count(&self) -> usize { self.leases.len() }
    pub fn total_granted(&self) -> u64 { self.total_granted }
    pub fn total_revoked(&self) -> u64 { self.total_revoked }
    pub fn total_renewed(&self) -> u64 { self.total_renewed }
    pub fn total_expired(&self) -> u64 { self.total_expired }
}

impl Default for LeaseManager2 {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_mgr() { assert_eq!(LeaseManager2::new().lease_count(), 0); }

    #[test]
    fn grant_is_valid() {
        let mut m = LeaseManager2::new();
        let id = m.grant(1, 100, None).unwrap();
        assert!(m.is_valid(id));
        assert_eq!(m.holder(id), Some(1));
    }

    #[test]
    fn revoke() {
        let mut m = LeaseManager2::new();
        let id = m.grant(1, 100, None).unwrap();
        let revoked = m.revoke(id).unwrap();
        assert_eq!(revoked, vec![id]);
        assert!(!m.is_valid(id));
    }

    #[test]
    fn renew() {
        let mut m = LeaseManager2::new();
        let id = m.grant(1, 100, None).unwrap();
        let cnt = m.renew(id, 200).unwrap();
        assert_eq!(cnt, 1);
        assert_eq!(m.expiry(id), Some(200));
    }

    #[test]
    fn cascade_revoke() {
        let mut m = LeaseManager2::new();
        let p = m.grant(1, 100, None).unwrap();
        let c1 = m.grant(1, 100, Some(p)).unwrap();
        let c2 = m.grant(1, 100, Some(p)).unwrap();
        let revoked = m.revoke(p).unwrap();
        assert_eq!(revoked.len(), 3);
        assert!(!m.is_valid(c1));
        assert!(!m.is_valid(c2));
    }

    #[test]
    fn tick_expiry() {
        let mut m = LeaseManager2::new();
        m.grant(1, 50, None).unwrap();
        m.grant(2, 100, None).unwrap();
        let expired = m.tick(75);
        assert_eq!(expired.len(), 1);
        assert_eq!(m.total_expired(), 1);
    }

    #[test]
    fn not_found() {
        let m = LeaseManager2::new();
        assert!(!m.is_valid(99));
    }

    #[test]
    fn parent_not_found() {
        let mut m = LeaseManager2::new();
        let err = m.grant(1, 100, Some(99)).unwrap_err();
        assert!(matches!(err, Lease2Error::LeaseNotFound { .. }));
    }

    #[test]
    fn double_revoke() {
        let mut m = LeaseManager2::new();
        let id = m.grant(1, 100, None).unwrap();
        m.revoke(id).unwrap();
        let err = m.revoke(id).unwrap_err();
        assert!(matches!(err, Lease2Error::AlreadyRevoked { .. }));
    }

    #[test]
    fn stats() {
        let mut m = LeaseManager2::new();
        let id = m.grant(1, 100, None).unwrap();
        m.renew(id, 200).unwrap();
        m.revoke(id).unwrap();
        assert_eq!(m.total_granted(), 1);
        assert_eq!(m.total_renewed(), 1);
        assert_eq!(m.total_revoked(), 1);
    }

    #[test]
    fn error_display() { assert!(Lease2Error::LeaseExpired { id: 3 }.to_string().contains("3")); }
}

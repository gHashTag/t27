use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RcError {
    ResourceNotFound { id: u64 },
    NotLeased { id: u64 },
    ResourceExists { id: u64 },
}

impl std::fmt::Display for RcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RcError::ResourceNotFound { id } => write!(f, "resource {id} not found"),
            RcError::NotLeased { id } => write!(f, "resource {id} not leased"),
            RcError::ResourceExists { id } => write!(f, "resource {id} exists"),
        }
    }
}

impl std::error::Error for RcError {}

struct Resource {
    id: u64,
    ref_count: u64,
    data: Vec<u8>,
    dropped: bool,
}

pub struct RefCnt {
    resources: BTreeMap<u64, Resource>,
    dropped: Vec<(u64, Vec<u8>)>,
    total_leases: u64,
    total_releases: u64,
    total_drops: u64,
}

impl RefCnt {
    pub fn new() -> Self { Self { resources: BTreeMap::new(), dropped: Vec::new(), total_leases: 0, total_releases: 0, total_drops: 0 } }

    pub fn create(&mut self, id: u64, data: Vec<u8>) -> Result<(), RcError> {
        if self.resources.contains_key(&id) { return Err(RcError::ResourceExists { id }); }
        self.resources.insert(id, Resource { id, ref_count: 0, data, dropped: false });
        Ok(())
    }

    pub fn lease(&mut self, id: u64) -> Result<u64, RcError> {
        let r = self.resources.get_mut(&id).ok_or(RcError::ResourceNotFound { id })?;
        if r.dropped { return Err(RcError::ResourceNotFound { id }); }
        r.ref_count += 1;
        self.total_leases += 1;
        Ok(r.ref_count)
    }

    pub fn release(&mut self, id: u64) -> Result<u64, RcError> {
        let r = self.resources.get_mut(&id).ok_or(RcError::ResourceNotFound { id })?;
        if r.ref_count == 0 { return Err(RcError::NotLeased { id }); }
        r.ref_count -= 1;
        self.total_releases += 1;
        if r.ref_count == 0 {
            r.dropped = true;
            let data = std::mem::take(&mut r.data);
            self.dropped.push((id, data));
            self.total_drops += 1;
        }
        Ok(r.ref_count)
    }

    pub fn ref_count(&self, id: u64) -> Option<u64> { self.resources.get(&id).map(|r| r.ref_count) }

    pub fn is_dropped(&self, id: u64) -> Option<bool> { self.resources.get(&id).map(|r| r.dropped) }

    pub fn get(&self, id: u64) -> Option<&Vec<u8>> {
        self.resources.get(&id).filter(|r| !r.dropped).map(|r| &r.data)
    }

    pub fn take_drops(&mut self) -> Vec<(u64, Vec<u8>)> { std::mem::take(&mut self.dropped) }

    pub fn active_count(&self) -> usize { self.resources.values().filter(|r| !r.dropped).count() }
    pub fn resource_count(&self) -> usize { self.resources.len() }
    pub fn total_leases(&self) -> u64 { self.total_leases }
    pub fn total_releases(&self) -> u64 { self.total_releases }
    pub fn total_drops(&self) -> u64 { self.total_drops }
}

impl Default for RefCnt {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rc() { assert_eq!(RefCnt::new().resource_count(), 0); }

    #[test]
    fn create_lease_release() {
        let mut rc = RefCnt::new();
        rc.create(1, b"data".to_vec()).unwrap();
        assert_eq!(rc.lease(1), Ok(1));
        assert_eq!(rc.lease(1), Ok(2));
        assert_eq!(rc.release(1), Ok(1));
        assert_eq!(rc.ref_count(1), Some(1));
    }

    #[test]
    fn zero_drop() {
        let mut rc = RefCnt::new();
        rc.create(1, b"data".to_vec()).unwrap();
        rc.lease(1).unwrap();
        rc.release(1).unwrap();
        assert!(rc.is_dropped(1).unwrap());
        assert_eq!(rc.total_drops(), 1);
    }

    #[test]
    fn drop_callback() {
        let mut rc = RefCnt::new();
        rc.create(1, b"val".to_vec()).unwrap();
        rc.lease(1).unwrap();
        rc.release(1).unwrap();
        let drops = rc.take_drops();
        assert_eq!(drops, vec![(1, b"val".to_vec())]);
    }

    #[test]
    fn multi_lease_no_drop() {
        let mut rc = RefCnt::new();
        rc.create(1, b"x".to_vec()).unwrap();
        rc.lease(1).unwrap(); rc.lease(1).unwrap();
        rc.release(1).unwrap();
        assert!(!rc.is_dropped(1).unwrap());
    }

    #[test]
    fn not_found() {
        let mut rc = RefCnt::new();
        let err = rc.lease(99).unwrap_err();
        assert!(matches!(err, RcError::ResourceNotFound { .. }));
    }

    #[test]
    fn not_leased() {
        let mut rc = RefCnt::new();
        rc.create(1, b"x".to_vec()).unwrap();
        let err = rc.release(1).unwrap_err();
        assert!(matches!(err, RcError::NotLeased { .. }));
    }

    #[test]
    fn duplicate_create() {
        let mut rc = RefCnt::new();
        rc.create(1, b"x".to_vec()).unwrap();
        let err = rc.create(1, b"y".to_vec()).unwrap_err();
        assert!(matches!(err, RcError::ResourceExists { .. }));
    }

    #[test]
    fn get_dropped() {
        let mut rc = RefCnt::new();
        rc.create(1, b"x".to_vec()).unwrap();
        rc.lease(1).unwrap(); rc.release(1).unwrap();
        assert_eq!(rc.get(1), None);
    }

    #[test]
    fn stats() {
        let mut rc = RefCnt::new();
        rc.create(1, b"x".to_vec()).unwrap();
        rc.lease(1).unwrap(); rc.release(1).unwrap();
        assert_eq!(rc.total_leases(), 1);
        assert_eq!(rc.total_releases(), 1);
        assert_eq!(rc.total_drops(), 1);
    }

    #[test]
    fn error_display() { assert!(RcError::ResourceNotFound { id: 1 }.to_string().contains("1")); }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Perm(u8);

impl Perm {
    pub const R: Perm = Perm(0x01);
    pub const W: Perm = Perm(0x02);
    pub const X: Perm = Perm(0x04);
    pub const RW: Perm = Perm(0x01 | 0x02);
    pub const RX: Perm = Perm(0x01 | 0x04);
    pub const RWX: Perm = Perm(0x01 | 0x02 | 0x04);
    pub const NONE: Perm = Perm(0x00);

    pub fn contains(&self, other: Perm) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn bits(&self) -> u8 {
        self.0
    }
}

impl std::fmt::Display for Perm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.contains(Perm::R) { s.push('r'); }
        if self.contains(Perm::W) { s.push('w'); }
        if self.contains(Perm::X) { s.push('x'); }
        if s.is_empty() { s.push('-'); }
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
pub struct Region {
    pub name: String,
    pub base: u32,
    pub size: u32,
    pub perm: Perm,
}

impl Region {
    pub fn new(name: &str, base: u32, size: u32, perm: Perm) -> Self {
        Self { name: name.to_string(), base, size, perm }
    }

    pub fn end(&self) -> u32 {
        self.base + self.size
    }

    pub fn contains(&self, addr: u32) -> bool {
        addr >= self.base && addr < self.end()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardError {
    Denied { addr: u32, needed: Perm, got: Perm, region: String },
    NoRegion { addr: u32 },
}

impl std::fmt::Display for GuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuardError::Denied { addr, needed, got, region } => {
                write!(f, "access denied 0x{addr:X}: need {needed} got {got} ({region})")
            }
            GuardError::NoRegion { addr } => {
                write!(f, "no region for 0x{addr:X}")
            }
        }
    }
}

impl std::error::Error for GuardError {}

#[derive(Debug, Clone)]
pub struct AddressGuard {
    regions: BTreeMap<String, Region>,
    total_checks: u64,
    total_denied: u64,
}

impl AddressGuard {
    pub fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
            total_checks: 0,
            total_denied: 0,
        }
    }

    pub fn add(&mut self, region: Region) {
        self.regions.insert(region.name.clone(), region);
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.regions.remove(name).is_some()
    }

    pub fn find_region(&self, addr: u32) -> Option<&Region> {
        self.regions.values().find(|r| r.contains(addr))
    }

    pub fn check(&mut self, addr: u32, needed: Perm) -> Result<Region, GuardError> {
        self.total_checks += 1;
        let region = match self.find_region(addr) {
            Some(r) => r.clone(),
            None => return Err(GuardError::NoRegion { addr }),
        };
        if region.perm.contains(needed) {
            Ok(region)
        } else {
            self.total_denied += 1;
            Err(GuardError::Denied {
                addr,
                needed,
                got: region.perm,
                region: region.name.clone(),
            })
        }
    }

    pub fn check_read(&mut self, addr: u32) -> Result<Region, GuardError> {
        self.check(addr, Perm::R)
    }

    pub fn check_write(&mut self, addr: u32) -> Result<Region, GuardError> {
        self.check(addr, Perm::W)
    }

    pub fn check_exec(&mut self, addr: u32) -> Result<Region, GuardError> {
        self.check(addr, Perm::X)
    }

    pub fn total_checks(&self) -> u64 {
        self.total_checks
    }

    pub fn total_denied(&self) -> u64 {
        self.total_denied
    }

    pub fn deny_rate(&self) -> f64 {
        if self.total_checks == 0 { 0.0 } else { self.total_denied as f64 / self.total_checks as f64 }
    }

    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    pub fn clear(&mut self) {
        self.regions.clear();
    }
}

impl Default for AddressGuard {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perm_display() {
        assert_eq!(Perm::RW.to_string(), "rw");
        assert_eq!(Perm::RWX.to_string(), "rwx");
        assert_eq!(Perm::R.to_string(), "r");
    }

    #[test]
    fn region_contains() {
        let r = Region::new("code", 0x1000, 0x100, Perm::RX);
        assert!(r.contains(0x1000));
        assert!(r.contains(0x10FF));
        assert!(!r.contains(0x1100));
    }

    #[test]
    fn check_allowed() {
        let mut ag = AddressGuard::new();
        ag.add(Region::new("data", 0x2000, 0x100, Perm::RW));
        let r = ag.check_read(0x2050).unwrap();
        assert_eq!(r.name, "data");
    }

    #[test]
    fn check_write_allowed() {
        let mut ag = AddressGuard::new();
        ag.add(Region::new("data", 0x2000, 0x100, Perm::RW));
        ag.check_write(0x2050).unwrap();
    }

    #[test]
    fn check_denied() {
        let mut ag = AddressGuard::new();
        ag.add(Region::new("rom", 0x1000, 0x100, Perm::R));
        let err = ag.check_write(0x1050).unwrap_err();
        assert!(matches!(err, GuardError::Denied { .. }));
        assert_eq!(ag.total_denied(), 1);
    }

    #[test]
    fn check_no_region() {
        let mut ag = AddressGuard::new();
        let err = ag.check_read(0xDEAD).unwrap_err();
        assert!(matches!(err, GuardError::NoRegion { .. }));
    }

    #[test]
    fn check_exec() {
        let mut ag = AddressGuard::new();
        ag.add(Region::new("code", 0x0000, 0x1000, Perm::RX));
        ag.check_exec(0x0100).unwrap();
    }

    #[test]
    fn deny_rate() {
        let mut ag = AddressGuard::new();
        ag.add(Region::new("rom", 0, 0x100, Perm::R));
        ag.check_read(0).unwrap();
        ag.check_write(0).unwrap_err();
        assert!((ag.deny_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn remove_region() {
        let mut ag = AddressGuard::new();
        ag.add(Region::new("x", 0, 1, Perm::RW));
        assert!(ag.remove("x"));
        assert_eq!(ag.region_count(), 0);
    }

    #[test]
    fn clear() {
        let mut ag = AddressGuard::new();
        ag.add(Region::new("x", 0, 1, Perm::RW));
        ag.clear();
        assert_eq!(ag.region_count(), 0);
    }

    #[test]
    fn error_display() {
        assert!(GuardError::Denied { addr: 0, needed: Perm::W, got: Perm::R, region: "rom".into() }.to_string().contains("denied"));
        assert!(GuardError::NoRegion { addr: 0xFF }.to_string().contains("0xFF"));
    }
}

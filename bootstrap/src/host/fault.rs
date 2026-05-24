use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultKind {
    BitFlip,
    StuckAt,
    Drop,
    Delay,
    Corrupt,
}

impl std::fmt::Display for FaultKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FaultKind::BitFlip => write!(f, "bit_flip"),
            FaultKind::StuckAt => write!(f, "stuck_at"),
            FaultKind::Drop => write!(f, "drop"),
            FaultKind::Delay => write!(f, "delay"),
            FaultKind::Corrupt => write!(f, "corrupt"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fault {
    pub name: String,
    pub kind: FaultKind,
    pub address: u32,
    pub mask: u32,
    pub value: u32,
    pub active: bool,
}

impl Fault {
    pub fn bit_flip(name: &str, address: u32, mask: u32) -> Self {
        Self {
            name: name.to_string(),
            kind: FaultKind::BitFlip,
            address,
            mask,
            value: 0,
            active: true,
        }
    }

    pub fn stuck_at(name: &str, address: u32, mask: u32, value: u32) -> Self {
        Self {
            name: name.to_string(),
            kind: FaultKind::StuckAt,
            address,
            mask,
            value,
            active: true,
        }
    }

    pub fn drop(name: &str, address: u32) -> Self {
        Self {
            name: name.to_string(),
            kind: FaultKind::Drop,
            address,
            mask: 0,
            value: 0,
            active: true,
        }
    }

    pub fn corrupt(name: &str, address: u32, value: u32) -> Self {
        Self {
            name: name.to_string(),
            kind: FaultKind::Corrupt,
            address,
            mask: 0xFFFFFFFF,
            value,
            active: true,
        }
    }

    pub fn matches(&self, address: u32) -> bool {
        self.active && self.address == address
    }
}

#[derive(Debug, Clone)]
pub struct InjectResult {
    pub fault_name: String,
    pub original: u32,
    pub modified: u32,
    pub dropped: bool,
}

#[derive(Debug, Clone)]
pub struct FaultInjector {
    faults: BTreeMap<String, Fault>,
    total_injections: u64,
    total_drops: u64,
}

impl FaultInjector {
    pub fn new() -> Self {
        Self {
            faults: BTreeMap::new(),
            total_injections: 0,
            total_drops: 0,
        }
    }

    pub fn add(&mut self, fault: Fault) {
        self.faults.insert(fault.name.clone(), fault);
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.faults.remove(name).is_some()
    }

    pub fn set_active(&mut self, name: &str, active: bool) -> bool {
        if let Some(f) = self.faults.get_mut(name) {
            f.active = active;
            true
        } else {
            false
        }
    }

    pub fn inject_read(&mut self, address: u32, value: u32) -> Result<u32, InjectResult> {
        for fault in self.faults.values() {
            if fault.matches(address) {
                let modified = apply_fault(fault, value);
                let dropped = fault.kind == FaultKind::Drop;
                if dropped {
                    self.total_drops += 1;
                }
                self.total_injections += 1;
                return Err(InjectResult {
                    fault_name: fault.name.clone(),
                    original: value,
                    modified,
                    dropped,
                });
            }
        }
        Ok(value)
    }

    pub fn inject_write(&mut self, address: u32, value: u32) -> Result<u32, InjectResult> {
        for fault in self.faults.values() {
            if fault.matches(address) {
                let modified = apply_fault(fault, value);
                let dropped = fault.kind == FaultKind::Drop;
                if dropped {
                    self.total_drops += 1;
                }
                self.total_injections += 1;
                return Err(InjectResult {
                    fault_name: fault.name.clone(),
                    original: value,
                    modified,
                    dropped,
                });
            }
        }
        Ok(value)
    }

    pub fn fault_count(&self) -> usize {
        self.faults.len()
    }

    pub fn active_faults(&self) -> Vec<&Fault> {
        self.faults.values().filter(|f| f.active).collect()
    }

    pub fn total_injections(&self) -> u64 {
        self.total_injections
    }

    pub fn total_drops(&self) -> u64 {
        self.total_drops
    }

    pub fn clear(&mut self) {
        self.faults.clear();
    }

    pub fn reset_stats(&mut self) {
        self.total_injections = 0;
        self.total_drops = 0;
    }
}

fn apply_fault(fault: &Fault, value: u32) -> u32 {
    match fault.kind {
        FaultKind::BitFlip => value ^ fault.mask,
        FaultKind::StuckAt => (value & !fault.mask) | (fault.value & fault.mask),
        FaultKind::Drop => value,
        FaultKind::Delay => value,
        FaultKind::Corrupt => fault.value,
    }
}

impl Default for FaultInjector {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_display() {
        assert_eq!(FaultKind::BitFlip.to_string(), "bit_flip");
        assert_eq!(FaultKind::Corrupt.to_string(), "corrupt");
    }

    #[test]
    fn bit_flip() {
        let mut fi = FaultInjector::new();
        fi.add(Fault::bit_flip("f1", 0x100, 0xFF));
        let result = fi.inject_read(0x100, 0x00);
        assert!(result.is_err());
        let ir = result.unwrap_err();
        assert_eq!(ir.modified, 0xFF);
        assert!(!ir.dropped);
    }

    #[test]
    fn stuck_at() {
        let mut fi = FaultInjector::new();
        fi.add(Fault::stuck_at("f1", 0x100, 0xFF, 0xAA));
        let result = fi.inject_read(0x100, 0x55);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().modified, 0x55 & !0xFF | (0xAA & 0xFF));
    }

    #[test]
    fn drop_fault() {
        let mut fi = FaultInjector::new();
        fi.add(Fault::drop("f1", 0x100));
        let result = fi.inject_write(0x100, 0x42);
        assert!(result.is_err());
        assert!(result.unwrap_err().dropped);
        assert_eq!(fi.total_drops(), 1);
    }

    #[test]
    fn corrupt_fault() {
        let mut fi = FaultInjector::new();
        fi.add(Fault::corrupt("f1", 0x100, 0xDEAD));
        let result = fi.inject_read(0x100, 0x1234);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().modified, 0xDEAD);
    }

    #[test]
    fn no_fault_passes_through() {
        let mut fi = FaultInjector::new();
        let result = fi.inject_read(0x200, 0x42);
        assert_eq!(result.unwrap(), 0x42);
    }

    #[test]
    fn wrong_address_passes_through() {
        let mut fi = FaultInjector::new();
        fi.add(Fault::bit_flip("f1", 0x100, 0xFF));
        let result = fi.inject_read(0x200, 0x42);
        assert_eq!(result.unwrap(), 0x42);
    }

    #[test]
    fn set_active_disables() {
        let mut fi = FaultInjector::new();
        fi.add(Fault::bit_flip("f1", 0x100, 0xFF));
        fi.set_active("f1", false);
        let result = fi.inject_read(0x100, 0x42);
        assert_eq!(result.unwrap(), 0x42);
    }

    #[test]
    fn remove_fault() {
        let mut fi = FaultInjector::new();
        fi.add(Fault::bit_flip("f1", 0x100, 0xFF));
        assert!(fi.remove("f1"));
        assert_eq!(fi.fault_count(), 0);
    }

    #[test]
    fn stats() {
        let mut fi = FaultInjector::new();
        fi.add(Fault::bit_flip("f1", 0x100, 0xFF));
        fi.inject_read(0x100, 0);
        fi.inject_read(0x100, 0);
        assert_eq!(fi.total_injections(), 2);
        fi.reset_stats();
        assert_eq!(fi.total_injections(), 0);
    }

    #[test]
    fn active_faults_count() {
        let mut fi = FaultInjector::new();
        fi.add(Fault::bit_flip("f1", 0x100, 0xFF));
        fi.add(Fault::drop("f2", 0x200));
        fi.set_active("f2", false);
        assert_eq!(fi.active_faults().len(), 1);
    }
}

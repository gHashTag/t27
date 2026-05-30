#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddrError {
    OutOfRange { addr: u64, region: &'static str },
    Overlap { a: &'static str, b: &'static str },
    ZeroSize { region: &'static str },
}

impl std::fmt::Display for AddrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddrError::OutOfRange { addr, region } => {
                write!(f, "address 0x{addr:X} out of range for {region}")
            }
            AddrError::Overlap { a, b } => {
                write!(f, "regions overlap: {a} and {b}")
            }
            AddrError::ZeroSize { region } => {
                write!(f, "zero-sized region: {region}")
            }
        }
    }
}

impl std::error::Error for AddrError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemRegion {
    pub name: &'static str,
    pub base: u64,
    pub size: u64,
}

impl MemRegion {
    pub const fn new(name: &'static str, base: u64, size: u64) -> Self {
        Self { name, base, size }
    }

    pub const fn end(&self) -> u64 {
        self.base + self.size
    }

    pub const fn contains(&self, addr: u64) -> bool {
        addr >= self.base && addr < self.end()
    }

    pub const fn offset_of(&self, addr: u64) -> Option<u64> {
        if self.contains(addr) {
            Some(addr - self.base)
        } else {
            None
        }
    }

    pub const fn overlaps(&self, other: &MemRegion) -> bool {
        self.base < other.end() && other.base < self.end()
    }

    pub const fn align_up(&self, align: u64) -> u64 {
        (self.base + align - 1) / align * align
    }
}

pub const REGION_CSR: MemRegion = MemRegion::new("csr", 0x4000_0000, 0x1000);
pub const REGION_BRAM: MemRegion = MemRegion::new("bram", 0x0000_0000, 0x0010_0000);
pub const REGION_DDR: MemRegion = MemRegion::new("ddr", 0x8000_0000, 0x1000_0000);
pub const REGION_DMA: MemRegion = MemRegion::new("dma", 0x4000_1000, 0x0100);

#[derive(Debug, Clone)]
pub struct AddrMap {
    regions: Vec<MemRegion>,
}

impl AddrMap {
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    pub fn add(&mut self, region: MemRegion) -> Result<(), AddrError> {
        if region.size == 0 {
            return Err(AddrError::ZeroSize { region: region.name });
        }
        for existing in &self.regions {
            if region.overlaps(existing) {
                return Err(AddrError::Overlap {
                    a: region.name,
                    b: existing.name,
                });
            }
        }
        self.regions.push(region);
        Ok(())
    }

    pub fn find(&self, addr: u64) -> Option<&MemRegion> {
        self.regions.iter().find(|r| r.contains(addr))
    }

    pub fn find_name(&self, name: &str) -> Option<&MemRegion> {
        self.regions.iter().find(|r| r.name == name)
    }

    pub fn regions(&self) -> &[MemRegion] {
        &self.regions
    }

    pub fn len(&self) -> usize {
        self.regions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    pub fn validate_addr(&self, addr: u64) -> Result<&MemRegion, AddrError> {
        self.find(addr)
            .ok_or(AddrError::OutOfRange {
                addr,
                region: "unknown",
            })
    }

    pub fn validate_range(&self, addr: u64, len: u64) -> Result<&MemRegion, AddrError> {
        let region = self.validate_addr(addr)?;
        let end = addr + len;
        if end > region.end() {
            return Err(AddrError::OutOfRange {
                addr: end - 1,
                region: region.name,
            });
        }
        Ok(region)
    }

    pub fn default_map() -> Self {
        let mut m = Self::new();
        m.add(REGION_CSR).unwrap();
        m.add(REGION_BRAM).unwrap();
        m.add(REGION_DDR).unwrap();
        m.add(REGION_DMA).unwrap();
        m
    }
}

impl Default for AddrMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_region_contains() {
        let r = MemRegion::new("test", 0x1000, 0x100);
        assert!(r.contains(0x1000));
        assert!(r.contains(0x10FF));
        assert!(!r.contains(0x1100));
        assert!(!r.contains(0x0FFF));
    }

    #[test]
    fn mem_region_offset() {
        let r = MemRegion::new("test", 0x1000, 0x100);
        assert_eq!(r.offset_of(0x1050), Some(0x50));
        assert_eq!(r.offset_of(0x2000), None);
    }

    #[test]
    fn mem_region_overlaps() {
        let a = MemRegion::new("a", 0x1000, 0x100);
        let b = MemRegion::new("b", 0x1050, 0x100);
        let c = MemRegion::new("c", 0x2000, 0x100);
        assert!(a.overlaps(&b));
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn mem_region_end() {
        let r = MemRegion::new("test", 0x100, 0x50);
        assert_eq!(r.end(), 0x150);
    }

    #[test]
    fn mem_region_align_up() {
        let r = MemRegion::new("test", 0x1003, 0x10);
        assert_eq!(r.align_up(4), 0x1004);
        assert_eq!(r.align_up(0x1000), 0x2000);
    }

    #[test]
    fn addr_map_add_and_find() {
        let mut m = AddrMap::new();
        m.add(MemRegion::new("ram", 0x1000, 0x100)).unwrap();
        m.add(MemRegion::new("rom", 0x2000, 0x100)).unwrap();
        assert_eq!(m.find(0x1050).unwrap().name, "ram");
        assert_eq!(m.find(0x2050).unwrap().name, "rom");
        assert!(m.find(0x3000).is_none());
    }

    #[test]
    fn addr_map_reject_zero_size() {
        let mut m = AddrMap::new();
        let err = m.add(MemRegion::new("zero", 0x1000, 0)).unwrap_err();
        assert!(matches!(err, AddrError::ZeroSize { .. }));
    }

    #[test]
    fn addr_map_reject_overlap() {
        let mut m = AddrMap::new();
        m.add(MemRegion::new("a", 0x1000, 0x100)).unwrap();
        let err = m.add(MemRegion::new("b", 0x1050, 0x100)).unwrap_err();
        assert!(matches!(err, AddrError::Overlap { .. }));
    }

    #[test]
    fn addr_map_validate_addr() {
        let m = AddrMap::default_map();
        assert!(m.validate_addr(0x4000_0050).is_ok());
        assert!(m.validate_addr(0xFFFF_0000).is_err());
    }

    #[test]
    fn addr_map_validate_range() {
        let m = AddrMap::default_map();
        assert!(m.validate_range(0x4000_0000, 0x100).is_ok());
        assert!(m.validate_range(0x4000_0F00, 0x200).is_err());
    }

    #[test]
    fn default_map_has_four_regions() {
        let m = AddrMap::default_map();
        assert_eq!(m.len(), 4);
    }

    #[test]
    fn predefined_regions_no_overlap() {
        let regions = [REGION_CSR, REGION_BRAM, REGION_DDR, REGION_DMA];
        for i in 0..regions.len() {
            for j in (i + 1)..regions.len() {
                assert!(!regions[i].overlaps(&regions[j]),
                    "{} overlaps {}", regions[i].name, regions[j].name);
            }
        }
    }

    #[test]
    fn find_by_name() {
        let m = AddrMap::default_map();
        assert_eq!(m.find_name("csr").unwrap().base, 0x4000_0000);
        assert!(m.find_name("nonexistent").is_none());
    }

    #[test]
    fn error_display() {
        let e = AddrError::OutOfRange { addr: 0xDEAD, region: "csr" };
        assert!(e.to_string().contains("DEAD"));
        let e = AddrError::Overlap { a: "a", b: "b" };
        assert!(e.to_string().contains("overlap"));
    }
}

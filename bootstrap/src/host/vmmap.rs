#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Perm {
    None,
    Read,
    ReadWrite,
    ReadExec,
    All,
}

impl Perm {
    pub fn can_read(self) -> bool { matches!(self, Perm::Read | Perm::ReadWrite | Perm::ReadExec | Perm::All) }
    pub fn can_write(self) -> bool { matches!(self, Perm::ReadWrite | Perm::All) }
    pub fn can_exec(self) -> bool { matches!(self, Perm::ReadExec | Perm::All) }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmErr {
    OutOfSpace { requested: usize, available: usize },
    InvalidRegion { start: usize },
    Overlap { start: usize, len: usize },
}

impl std::fmt::Display for VmErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmErr::OutOfSpace { requested, available } => write!(f, "oom: need {requested} have {available}"),
            VmErr::InvalidRegion { start } => write!(f, "bad region {start}"),
            VmErr::Overlap { start, len } => write!(f, "overlap {start}+{len}"),
        }
    }
}

impl std::error::Error for VmErr {}

#[derive(Clone)]
struct Region {
    start: usize,
    len: usize,
    perm: Perm,
}

pub struct VmMap {
    regions: Vec<Region>,
    total_size: usize,
    page_size: usize,
    total_allocs: u64,
    total_frees: u64,
}

impl VmMap {
    pub fn new(total_size: usize, page_size: usize) -> Self {
        Self { regions: Vec::new(), total_size, page_size, total_allocs: 0, total_frees: 0 }
    }

    fn align_up(&self, size: usize) -> usize { (size + self.page_size - 1) / self.page_size * self.page_size }

    pub fn allocate(&mut self, size: usize, perm: Perm) -> Result<usize, VmErr> {
        self.total_allocs += 1;
        let aligned = self.align_up(size);
        let mut cursor = 0usize;
        for r in &self.regions {
            if r.start >= cursor + aligned {
                return self.insert_region(cursor, aligned, perm);
            }
            cursor = r.start + r.len;
        }
        if cursor + aligned <= self.total_size {
            return self.insert_region(cursor, aligned, perm);
        }
        Err(VmErr::OutOfSpace { requested: aligned, available: self.total_size.saturating_sub(cursor) })
    }

    fn insert_region(&mut self, start: usize, len: usize, perm: Perm) -> Result<usize, VmErr> {
        let idx = self.regions.partition_point(|r| r.start < start);
        self.regions.insert(idx, Region { start, len, perm });
        Ok(start)
    }

    pub fn deallocate(&mut self, start: usize) -> Result<usize, VmErr> {
        self.total_frees += 1;
        let idx = self.regions.iter().position(|r| r.start == start).ok_or(VmErr::InvalidRegion { start })?;
        let len = self.regions[idx].len;
        self.regions.remove(idx);
        Ok(len)
    }

    pub fn protect(&mut self, start: usize, perm: Perm) -> Result<Perm, VmErr> {
        let r = self.regions.iter_mut().find(|r| r.start == start).ok_or(VmErr::InvalidRegion { start })?;
        let old = r.perm;
        r.perm = perm;
        Ok(old)
    }

    pub fn query(&self, addr: usize) -> Option<(usize, usize, Perm)> {
        for r in &self.regions {
            if addr >= r.start && addr < r.start + r.len { return Some((r.start, r.len, r.perm)); }
        }
        None
    }

    pub fn used(&self) -> usize { self.regions.iter().map(|r| r.len).sum() }
    pub fn available(&self) -> usize { self.total_size - self.used() }
    pub fn region_count(&self) -> usize { self.regions.len() }
    pub fn fragmentation(&self) -> usize {
        if self.regions.is_empty() { return 0; }
        let mut gaps = 0usize;
        let mut cursor = 0usize;
        for r in &self.regions {
            if r.start > cursor { gaps += 1; }
            cursor = r.start + r.len;
        }
        gaps
    }
    pub fn total_allocs(&self) -> u64 { self.total_allocs }
    pub fn total_frees(&self) -> u64 { self.total_frees }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_vm() { let vm = VmMap::new(4096, 4096); assert_eq!(vm.available(), 4096); }

    #[test]
    fn alloc_dealloc() {
        let mut vm = VmMap::new(8192, 4096);
        let a = vm.allocate(100, Perm::ReadWrite).unwrap();
        assert_eq!(a, 0);
        assert_eq!(vm.used(), 4096);
        vm.deallocate(a).unwrap();
        assert_eq!(vm.used(), 0);
    }

    #[test]
    fn alloc_two() {
        let mut vm = VmMap::new(12288, 4096);
        let a = vm.allocate(100, Perm::Read).unwrap();
        let b = vm.allocate(100, Perm::ReadWrite).unwrap();
        assert_eq!(b, 4096);
        assert_eq!(vm.region_count(), 2);
    }

    #[test]
    fn alloc_reuse() {
        let mut vm = VmMap::new(8192, 4096);
        let a = vm.allocate(100, Perm::ReadWrite).unwrap();
        let b = vm.allocate(100, Perm::ReadWrite).unwrap();
        vm.deallocate(a).unwrap();
        let c = vm.allocate(100, Perm::ReadExec).unwrap();
        assert_eq!(c, 0);
    }

    #[test]
    fn out_of_space() {
        let mut vm = VmMap::new(4096, 4096);
        vm.allocate(100, Perm::ReadWrite).unwrap();
        assert!(vm.allocate(1, Perm::ReadWrite).is_err());
    }

    #[test]
    fn protect() {
        let mut vm = VmMap::new(4096, 4096);
        let a = vm.allocate(100, Perm::Read).unwrap();
        let old = vm.protect(a, Perm::ReadWrite).unwrap();
        assert_eq!(old, Perm::Read);
        assert_eq!(vm.query(a).unwrap().2, Perm::ReadWrite);
    }

    #[test]
    fn query() {
        let mut vm = VmMap::new(8192, 4096);
        let a = vm.allocate(100, Perm::ReadExec).unwrap();
        assert!(vm.query(50).is_some());
        assert!(vm.query(5000).is_none());
    }

    #[test]
    fn fragmentation() {
        let mut vm = VmMap::new(12288, 4096);
        let a = vm.allocate(100, Perm::ReadWrite).unwrap();
        let b = vm.allocate(100, Perm::ReadWrite).unwrap();
        let c = vm.allocate(100, Perm::ReadWrite).unwrap();
        vm.deallocate(b).unwrap();
        assert_eq!(vm.fragmentation(), 1);
    }

    #[test]
    fn dealloc_invalid() { assert!(VmMap::new(4096, 4096).deallocate(0).is_err()); }

    #[test]
    fn perm_flags() {
        assert!(Perm::ReadWrite.can_read());
        assert!(Perm::ReadWrite.can_write());
        assert!(!Perm::ReadWrite.can_exec());
        assert!(Perm::ReadExec.can_exec());
    }

    #[test]
    fn stats() {
        let mut vm = VmMap::new(8192, 4096);
        let a = vm.allocate(100, Perm::ReadWrite).unwrap();
        vm.deallocate(a).unwrap();
        assert_eq!(vm.total_allocs(), 1);
        assert_eq!(vm.total_frees(), 1);
    }

    #[test]
    fn error_display() { assert!(VmErr::OutOfSpace { requested: 100, available: 0 }.to_string().contains("oom")); }
}

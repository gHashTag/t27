use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum FlError {
    AlreadyFused { bit: u8 },
    BitOutOfRange { bit: u8 },
    NotFused { bit: u8 },
}

impl std::fmt::Display for FlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlError::AlreadyFused { bit } => write!(f, "bit {bit} already fused"),
            FlError::BitOutOfRange { bit } => write!(f, "bit {bit} out of range"),
            FlError::NotFused { bit } => write!(f, "bit {bit} not fused"),
        }
    }
}

impl std::error::Error for FlError {}

pub struct FuseLatch {
    register: u64,
    width: u8,
    total_fuses: u64,
    total_reads: u64,
}

impl FuseLatch {
    pub fn new(width: u8) -> Self { Self { register: 0, width, total_fuses: 0, total_reads: 0 } }

    pub fn fuse(&mut self, bit: u8) -> Result<(), FlError> {
        if bit >= self.width { return Err(FlError::BitOutOfRange { bit }); }
        let mask = 1u64 << bit;
        if self.register & mask != 0 { return Err(FlError::AlreadyFused { bit }); }
        self.register |= mask;
        self.total_fuses += 1;
        Ok(())
    }

    pub fn is_fused(&self, bit: u8) -> Option<bool> {
        if bit >= self.width { return None; }
        Some(self.register & (1u64 << bit) != 0)
    }

    pub fn read(&mut self) -> u64 {
        self.total_reads += 1;
        self.register
    }

    pub fn all_fused(&self) -> bool {
        let mask = (1u64 << self.width) - 1;
        self.register & mask == mask
    }

    pub fn none_fused(&self) -> bool { self.register == 0 }

    pub fn fused_count(&self) -> u8 { self.register.count_ones() as u8 }

    pub fn unfused_count(&self) -> u8 { self.width - self.fused_count() }

    pub fn first_unfused(&self) -> Option<u8> {
        let mask = (1u64 << self.width) - 1;
        let inv = (!self.register) & mask;
        if inv == 0 { None } else { Some(inv.trailing_zeros() as u8) }
    }

    pub fn width(&self) -> u8 { self.width }
    pub fn total_fuses(&self) -> u64 { self.total_fuses }
    pub fn total_reads(&self) -> u64 { self.total_reads }
}

pub struct FuseLatchBank {
    latches: BTreeMap<u64, FuseLatch>,
    total_latches: u64,
}

impl FuseLatchBank {
    pub fn new() -> Self { Self { latches: BTreeMap::new(), total_latches: 0 } }

    pub fn create(&mut self, id: u64, width: u8) -> Result<(), FlError> {
        if self.latches.contains_key(&id) { return Err(FlError::AlreadyFused { bit: 0 }); }
        self.latches.insert(id, FuseLatch::new(width));
        self.total_latches += 1;
        Ok(())
    }

    pub fn fuse(&mut self, id: u64, bit: u8) -> Result<(), FlError> {
        let latch = self.latches.get_mut(&id).ok_or(FlError::BitOutOfRange { bit })?;
        latch.fuse(bit)
    }

    pub fn read(&mut self, id: u64) -> Option<u64> {
        self.latches.get_mut(&id).map(|l| l.read())
    }

    pub fn is_fused(&self, id: u64, bit: u8) -> Option<bool> {
        self.latches.get(&id)?.is_fused(bit)
    }

    pub fn latch_count(&self) -> usize { self.latches.len() }
    pub fn total_latches(&self) -> u64 { self.total_latches }
}

impl Default for FuseLatchBank {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_latch() { let fl = FuseLatch::new(8); assert_eq!(fl.width(), 8); assert!(fl.none_fused()); }

    #[test]
    fn fuse_read() {
        let mut fl = FuseLatch::new(8);
        fl.fuse(0).unwrap(); fl.fuse(3).unwrap();
        assert_eq!(fl.read(), 0b00001001);
    }

    #[test]
    fn already_fused() {
        let mut fl = FuseLatch::new(8);
        fl.fuse(0).unwrap();
        let err = fl.fuse(0).unwrap_err();
        assert!(matches!(err, FlError::AlreadyFused { .. }));
    }

    #[test]
    fn is_fused() {
        let mut fl = FuseLatch::new(8);
        fl.fuse(2).unwrap();
        assert_eq!(fl.is_fused(2), Some(true));
        assert_eq!(fl.is_fused(1), Some(false));
        assert_eq!(fl.is_fused(8), None);
    }

    #[test]
    fn all_fused() {
        let mut fl = FuseLatch::new(4);
        for i in 0..4 { fl.fuse(i).unwrap(); }
        assert!(fl.all_fused());
    }

    #[test]
    fn first_unfused() {
        let mut fl = FuseLatch::new(8);
        fl.fuse(0).unwrap(); fl.fuse(1).unwrap();
        assert_eq!(fl.first_unfused(), Some(2));
        for i in 2..8 { fl.fuse(i).unwrap(); }
        assert_eq!(fl.first_unfused(), None);
    }

    #[test]
    fn bank() {
        let mut bank = FuseLatchBank::new();
        bank.create(1, 8).unwrap();
        bank.fuse(1, 0).unwrap();
        assert_eq!(bank.read(1), Some(1));
        assert!(bank.is_fused(1, 0).unwrap());
    }

    #[test]
    fn bank_missing() {
        let mut bank = FuseLatchBank::new();
        assert!(bank.read(99).is_none());
    }

    #[test]
    fn stats() {
        let mut fl = FuseLatch::new(8);
        fl.fuse(0).unwrap();
        fl.read();
        assert_eq!(fl.total_fuses(), 1);
        assert_eq!(fl.total_reads(), 1);
    }

    #[test]
    fn error_display() { assert!(FlError::AlreadyFused { bit: 3 }.to_string().contains("3")); }
}

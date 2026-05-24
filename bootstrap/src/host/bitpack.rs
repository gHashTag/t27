#[derive(Debug, Clone, PartialEq)]
pub enum BpError {
    IndexOutOfRange { idx: usize, len: usize },
    ValueOverflow { val: u64, bits: u8 },
    InvalidWidth { bits: u8 },
}

impl std::fmt::Display for BpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BpError::IndexOutOfRange { idx, len } => write!(f, "index {idx} out of range (len={len})"),
            BpError::ValueOverflow { val, bits } => write!(f, "value {val} exceeds {bits} bits"),
            BpError::InvalidWidth { bits } => write!(f, "invalid width {bits} (must be 1..=64)"),
        }
    }
}

impl std::error::Error for BpError {}

pub struct BitPack {
    data: Vec<u64>,
    width: u8,
    len: usize,
    total_gets: u64,
    total_sets: u64,
}

impl BitPack {
    pub fn new(width: u8) -> Result<Self, BpError> {
        if width == 0 || width > 64 { return Err(BpError::InvalidWidth { bits: width }); }
        Ok(Self { data: Vec::new(), width, len: 0, total_gets: 0, total_sets: 0 })
    }

    fn mask(&self) -> u64 { (1u64 << self.width) - 1 }

    pub fn append(&mut self, val: u64) -> Result<(), BpError> {
        if val > self.mask() { return Err(BpError::ValueOverflow { val, bits: self.width }); }
        let bit_offset = (self.len as u64) * (self.width as u64);
        let word_idx = (bit_offset / 64) as usize;
        let shift = (bit_offset % 64) as u8;
        while word_idx >= self.data.len() { self.data.push(0); }
        self.data[word_idx] |= val << shift;
        if shift as u16 + self.width as u16 > 64 {
            let spill = word_idx + 1;
            while spill >= self.data.len() { self.data.push(0); }
            let spill_bits = shift as u16 + self.width as u16 - 64;
            self.data[spill] |= val >> (64 - shift);
        }
        self.len += 1;
        Ok(())
    }

    pub fn get(&mut self, idx: usize) -> Result<u64, BpError> {
        self.total_gets += 1;
        if idx >= self.len { return Err(BpError::IndexOutOfRange { idx, len: self.len }); }
        let bit_offset = (idx as u64) * (self.width as u64);
        let word_idx = (bit_offset / 64) as usize;
        let shift = (bit_offset % 64) as u8;
        let mut val = (self.data[word_idx] >> shift) & self.mask();
        if shift as u16 + self.width as u16 > 64 {
            let spill_bits = shift as u16 + self.width as u16 - 64;
            val |= (self.data[word_idx + 1] & ((1u64 << spill_bits) - 1)) << (self.width as u16 - spill_bits) as u8;
        }
        Ok(val)
    }

    pub fn set(&mut self, idx: usize, val: u64) -> Result<(), BpError> {
        self.total_sets += 1;
        if idx >= self.len { return Err(BpError::IndexOutOfRange { idx, len: self.len }); }
        if val > self.mask() { return Err(BpError::ValueOverflow { val, bits: self.width }); }
        let bit_offset = (idx as u64) * (self.width as u64);
        let word_idx = (bit_offset / 64) as usize;
        let shift = (bit_offset % 64) as u8;
        self.data[word_idx] &= !(self.mask() << shift);
        self.data[word_idx] |= val << shift;
        if shift as u16 + self.width as u16 > 64 {
            let spill_bits = shift as u16 + self.width as u16 - 64;
            let spill = word_idx + 1;
            self.data[spill] &= !((1u64 << spill_bits) - 1);
            self.data[spill] |= val >> (64 - shift);
        }
        Ok(())
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn width(&self) -> u8 { self.width }
    pub fn total_gets(&self) -> u64 { self.total_gets }
    pub fn total_sets(&self) -> u64 { self.total_sets }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_basic() { let bp = BitPack::new(8).unwrap(); assert_eq!(bp.width(), 8); assert!(bp.is_empty()); }

    #[test]
    fn append_get_byte() {
        let mut bp = BitPack::new(8).unwrap();
        for v in [0u64, 255, 128, 1] { bp.append(v).unwrap(); }
        assert_eq!(bp.len(), 4);
        assert_eq!(bp.get(0).unwrap(), 0);
        assert_eq!(bp.get(1).unwrap(), 255);
        assert_eq!(bp.get(2).unwrap(), 128);
    }

    #[test]
    fn set_overwrite() {
        let mut bp = BitPack::new(8).unwrap();
        bp.append(42).unwrap();
        bp.set(0, 99).unwrap();
        assert_eq!(bp.get(0).unwrap(), 99);
    }

    #[test]
    fn overflow_err() {
        let mut bp = BitPack::new(4).unwrap();
        let err = bp.append(16).unwrap_err();
        assert!(matches!(err, BpError::ValueOverflow { .. }));
    }

    #[test]
    fn index_err() {
        let mut bp = BitPack::new(8).unwrap();
        bp.append(1).unwrap();
        let err = bp.get(5).unwrap_err();
        assert!(matches!(err, BpError::IndexOutOfRange { .. }));
    }

    #[test]
    fn invalid_width() { assert!(BitPack::new(0).is_err()); assert!(BitPack::new(65).is_err()); }

    #[test]
    fn wide_bits() {
        let mut bp = BitPack::new(13).unwrap();
        for i in 0..100u64 { bp.append(i).unwrap(); }
        for i in 0..100u64 { assert_eq!(bp.get(i as usize).unwrap(), i); }
    }

    #[test]
    fn single_bit() {
        let mut bp = BitPack::new(1).unwrap();
        for b in [true, false, true, true, false] { bp.append(b as u64).unwrap(); }
        assert_eq!(bp.get(0).unwrap(), 1);
        assert_eq!(bp.get(1).unwrap(), 0);
        assert_eq!(bp.get(3).unwrap(), 1);
    }

    #[test]
    fn cross_word_boundary() {
        let mut bp = BitPack::new(7).unwrap();
        for i in 0..20u64 { bp.append(i).unwrap(); }
        for i in 0..20u64 { assert_eq!(bp.get(i as usize).unwrap(), i); }
    }

    #[test]
    fn stats() {
        let mut bp = BitPack::new(8).unwrap();
        bp.append(1).unwrap();
        bp.get(0).unwrap(); bp.set(0, 2).unwrap();
        assert_eq!(bp.total_gets(), 1);
        assert_eq!(bp.total_sets(), 1);
    }

    #[test]
    fn error_display() { assert!(BpError::InvalidWidth { bits: 0 }.to_string().contains("invalid")); }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RlError {
    OutOfRange { idx: u64, len: u64 },
}

impl std::fmt::Display for RlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RlError::OutOfRange { idx, len } => write!(f, "index {idx} out of range (len={len})"),
        }
    }
}

impl std::error::Error for RlError {}

pub struct Reloc {
    words: Vec<u64>,
    bit_len: u64,
    total_sets: u64,
    total_clears: u64,
    total_flips: u64,
}

impl Reloc {
    pub fn new(bit_len: u64) -> Self {
        let word_count = ((bit_len + 63) / 64) as usize;
        Self { words: vec![0u64; word_count], bit_len, total_sets: 0, total_clears: 0, total_flips: 0 }
    }

    pub fn set(&mut self, idx: u64) -> Result<(), RlError> {
        if idx >= self.bit_len { return Err(RlError::OutOfRange { idx, len: self.bit_len }); }
        let (w, b) = ((idx / 64) as usize, idx % 64);
        self.words[w] |= 1 << b;
        self.total_sets += 1;
        Ok(())
    }

    pub fn clear(&mut self, idx: u64) -> Result<(), RlError> {
        if idx >= self.bit_len { return Err(RlError::OutOfRange { idx, len: self.bit_len }); }
        let (w, b) = ((idx / 64) as usize, idx % 64);
        self.words[w] &= !(1 << b);
        self.total_clears += 1;
        Ok(())
    }

    pub fn flip(&mut self, idx: u64) -> Result<(), RlError> {
        if idx >= self.bit_len { return Err(RlError::OutOfRange { idx, len: self.bit_len }); }
        let (w, b) = ((idx / 64) as usize, idx % 64);
        self.words[w] ^= 1 << b;
        self.total_flips += 1;
        Ok(())
    }

    pub fn test(&self, idx: u64) -> Option<bool> {
        if idx >= self.bit_len { return None; }
        let (w, b) = ((idx / 64) as usize, idx % 64);
        Some((self.words[w] >> b) & 1 == 1)
    }

    pub fn set_range(&mut self, from: u64, to: u64) -> Result<(), RlError> {
        if to > self.bit_len || from > to { return Err(RlError::OutOfRange { idx: from, len: self.bit_len }); }
        for i in from..to { self.set(i)?; }
        Ok(())
    }

    pub fn clear_range(&mut self, from: u64, to: u64) -> Result<(), RlError> {
        if to > self.bit_len || from > to { return Err(RlError::OutOfRange { idx: from, len: self.bit_len }); }
        for i in from..to { self.clear(i)?; }
        Ok(())
    }

    pub fn popcount(&self) -> u64 { self.words.iter().map(|w| w.count_ones() as u64).sum() }

    pub fn find_first_set(&self) -> Option<u64> {
        for (wi, &word) in self.words.iter().enumerate() {
            if word != 0 { return Some((wi as u64) * 64 + word.trailing_zeros() as u64); }
        }
        None
    }

    pub fn find_first_clear(&self) -> Option<u64> {
        for (wi, &word) in self.words.iter().enumerate() {
            if word != u64::MAX {
                let bit = (!word).trailing_zeros() as u64;
                let idx = (wi as u64) * 64 + bit;
                if idx < self.bit_len { return Some(idx); }
            }
        }
        None
    }

    pub fn bit_len(&self) -> u64 { self.bit_len }
    pub fn total_sets(&self) -> u64 { self.total_sets }
    pub fn total_clears(&self) -> u64 { self.total_clears }
    pub fn total_flips(&self) -> u64 { self.total_flips }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bm() { let bm = Reloc::new(128); assert_eq!(bm.bit_len(), 128); assert_eq!(bm.popcount(), 0); }

    #[test]
    fn set_test() {
        let mut bm = Reloc::new(128);
        bm.set(5).unwrap();
        assert_eq!(bm.test(5), Some(true));
        assert_eq!(bm.test(4), Some(false));
    }

    #[test]
    fn clear() {
        let mut bm = Reloc::new(64);
        bm.set(0).unwrap(); bm.clear(0).unwrap();
        assert_eq!(bm.test(0), Some(false));
    }

    #[test]
    fn flip() {
        let mut bm = Reloc::new(64);
        bm.flip(10).unwrap();
        assert_eq!(bm.test(10), Some(true));
        bm.flip(10).unwrap();
        assert_eq!(bm.test(10), Some(false));
    }

    #[test]
    fn out_of_range() {
        let mut bm = Reloc::new(64);
        assert!(bm.set(64).is_err());
        assert_eq!(bm.test(64), None);
    }

    #[test]
    fn set_range() {
        let mut bm = Reloc::new(64);
        bm.set_range(10, 15).unwrap();
        for i in 10..15 { assert_eq!(bm.test(i), Some(true)); }
        assert_eq!(bm.popcount(), 5);
    }

    #[test]
    fn find_first() {
        let mut bm = Reloc::new(128);
        bm.set(42).unwrap();
        assert_eq!(bm.find_first_set(), Some(42));
        assert_eq!(bm.find_first_clear(), Some(0));
    }

    #[test]
    fn find_first_clear_all_set() {
        let mut bm = Reloc::new(64);
        bm.set_range(0, 64).unwrap();
        assert_eq!(bm.find_first_clear(), None);
    }

    #[test]
    fn stats() {
        let mut bm = Reloc::new(64);
        bm.set(0).unwrap(); bm.clear(0).unwrap(); bm.flip(1).unwrap();
        assert_eq!(bm.total_sets(), 1);
        assert_eq!(bm.total_clears(), 1);
        assert_eq!(bm.total_flips(), 1);
    }

    #[test]
    fn error_display() { assert!(RlError::OutOfRange { idx: 1, len: 0 }.to_string().contains("1")); }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum St2Error {
    IndexOutOfRange { idx: u64, len: u64 },
    EmptyTree,
}

impl std::fmt::Display for St2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            St2Error::IndexOutOfRange { idx, len } => write!(f, "index {idx} out of range (len={len})"),
            St2Error::EmptyTree => write!(f, "empty tree"),
        }
    }
}

impl std::error::Error for St2Error {}

pub struct SurfTree {
    bits: Vec<u64>,
    bit_len: u64,
    total_ranks: u64,
    total_selects: u64,
}

impl SurfTree {
    pub fn new() -> Self { Self { bits: Vec::new(), bit_len: 0, total_ranks: 0, total_selects: 0 } }

    pub fn set(&mut self, idx: u64) -> Result<(), St2Error> {
        if idx >= self.bit_len { return Err(St2Error::IndexOutOfRange { idx, len: self.bit_len }); }
        let (w, b) = ((idx / 64) as usize, idx % 64);
        self.bits[w] |= 1 << b;
        Ok(())
    }

    pub fn clear(&mut self, idx: u64) -> Result<(), St2Error> {
        if idx >= self.bit_len { return Err(St2Error::IndexOutOfRange { idx, len: self.bit_len }); }
        let (w, b) = ((idx / 64) as usize, idx % 64);
        self.bits[w] &= !(1 << b);
        Ok(())
    }

    pub fn get(&self, idx: u64) -> Option<bool> {
        if idx >= self.bit_len { return None; }
        let (w, b) = ((idx / 64) as usize, idx % 64);
        Some((self.bits[w] >> b) & 1 == 1)
    }

    pub fn push(&mut self, val: bool) {
        let (w, b) = ((self.bit_len / 64) as usize, self.bit_len % 64);
        if w >= self.bits.len() { self.bits.push(0); }
        if val { self.bits[w] |= 1 << b; }
        self.bit_len += 1;
    }

    pub fn rank1(&mut self, idx: u64) -> u64 {
        self.total_ranks += 1;
        if idx == 0 { return 0; }
        let limit = idx.min(self.bit_len);
        let full_words = (limit / 64) as usize;
        let mut count: u64 = self.bits[..full_words].iter().map(|w| w.count_ones() as u64).sum();
        let remaining = limit % 64;
        if remaining > 0 && full_words < self.bits.len() {
            let mask = (1u64 << remaining) - 1;
            count += (self.bits[full_words] & mask).count_ones() as u64;
        }
        count
    }

    pub fn rank0(&mut self, idx: u64) -> u64 { idx - self.rank1(idx) }

    pub fn select1(&mut self, k: u64) -> Option<u64> {
        self.total_selects += 1;
        if k == 0 { return None; }
        let mut remaining = k;
        for i in 0..self.bit_len {
            if self.get(i)? { remaining -= 1; if remaining == 0 { return Some(i); } }
        }
        None
    }

    pub fn select0(&mut self, k: u64) -> Option<u64> {
        self.total_selects += 1;
        if k == 0 { return None; }
        let mut remaining = k;
        for i in 0..self.bit_len {
            if !self.get(i)? { remaining -= 1; if remaining == 0 { return Some(i); } }
        }
        None
    }

    pub fn popcount(&self) -> u64 { self.bits.iter().map(|w| w.count_ones() as u64).sum() }
    pub fn bit_len(&self) -> u64 { self.bit_len }
    pub fn is_empty(&self) -> bool { self.bit_len == 0 }
    pub fn total_ranks(&self) -> u64 { self.total_ranks }
    pub fn total_selects(&self) -> u64 { self.total_selects }
}

impl Default for SurfTree {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tree() { assert!(SurfTree::new().is_empty()); }

    #[test]
    fn push_get() {
        let mut st = SurfTree::new();
        st.push(true); st.push(false); st.push(true);
        assert_eq!(st.get(0), Some(true));
        assert_eq!(st.get(1), Some(false));
        assert_eq!(st.get(2), Some(true));
    }

    #[test]
    fn rank1() {
        let mut st = SurfTree::new();
        for b in [true, false, true, true, false] { st.push(b); }
        assert_eq!(st.rank1(3), 2);
        assert_eq!(st.rank1(5), 3);
    }

    #[test]
    fn rank0() {
        let mut st = SurfTree::new();
        for b in [true, false, true, true, false] { st.push(b); }
        assert_eq!(st.rank0(5), 2);
    }

    #[test]
    fn select1() {
        let mut st = SurfTree::new();
        for b in [true, false, true, true, false] { st.push(b); }
        assert_eq!(st.select1(1), Some(0));
        assert_eq!(st.select1(2), Some(2));
        assert_eq!(st.select1(3), Some(3));
    }

    #[test]
    fn select0() {
        let mut st = SurfTree::new();
        for b in [true, false, true, true, false] { st.push(b); }
        assert_eq!(st.select0(1), Some(1));
        assert_eq!(st.select0(2), Some(4));
    }

    #[test]
    fn select_not_found() {
        let mut st = SurfTree::new();
        st.push(false);
        assert_eq!(st.select1(1), None);
    }

    #[test]
    fn popcount() {
        let mut st = SurfTree::new();
        for b in [true, false, true, true] { st.push(b); }
        assert_eq!(st.popcount(), 3);
    }

    #[test]
    fn large() {
        let mut st = SurfTree::new();
        for i in 0..200 { st.push(i % 3 == 0); }
        assert_eq!(st.bit_len(), 200);
        assert_eq!(st.rank1(200), 67);
    }

    #[test]
    fn stats() {
        let mut st = SurfTree::new();
        st.push(true);
        st.rank1(1); st.select1(1);
        assert_eq!(st.total_ranks(), 1);
        assert_eq!(st.total_selects(), 1);
    }

    #[test]
    fn error_display() { assert!(St2Error::EmptyTree.to_string().contains("empty")); }
}

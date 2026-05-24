const BITS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BmIndexError {
    OutOfRange { bit: usize, size: usize },
}

impl std::fmt::Display for BmIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BmIndexError::OutOfRange { bit, size } => write!(f, "bit {bit} >= {size}"),
        }
    }
}

impl std::error::Error for BmIndexError {}

#[derive(Debug, Clone)]
pub struct BitmapIndex {
    words: Vec<u64>,
    size: usize,
    total_set: u64,
    total_clear: u64,
}

impl BitmapIndex {
    pub fn new(size: usize) -> Self {
        let words = (size + BITS - 1) / BITS;
        Self { words: vec![0u64; words], size, total_set: 0, total_clear: 0 }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn set(&mut self, bit: usize) -> Result<bool, BmIndexError> {
        if bit >= self.size { return Err(BmIndexError::OutOfRange { bit, size: self.size }); }
        let word = bit / BITS;
        let mask = 1u64 << (bit % BITS);
        let was_set = self.words[word] & mask != 0;
        self.words[word] |= mask;
        if !was_set { self.total_set += 1; }
        Ok(!was_set)
    }

    pub fn clear(&mut self, bit: usize) -> Result<bool, BmIndexError> {
        if bit >= self.size { return Err(BmIndexError::OutOfRange { bit, size: self.size }); }
        let word = bit / BITS;
        let mask = 1u64 << (bit % BITS);
        let was_set = self.words[word] & mask != 0;
        self.words[word] &= !mask;
        if was_set { self.total_clear += 1; }
        Ok(was_set)
    }

    pub fn get(&self, bit: usize) -> bool {
        if bit >= self.size { return false; }
        let word = bit / BITS;
        (self.words[word] >> (bit % BITS)) & 1 != 0
    }

    pub fn count_ones(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn count_zeros(&self) -> usize {
        self.size - self.count_ones()
    }

    pub fn and(&self, other: &BitmapIndex) -> BitmapIndex {
        let len = self.words.len().min(other.words.len());
        let mut result = BitmapIndex::new(self.size.min(other.size));
        for i in 0..len {
            result.words[i] = self.words[i] & other.words[i];
        }
        result
    }

    pub fn or(&self, other: &BitmapIndex) -> BitmapIndex {
        let len = self.words.len().min(other.words.len());
        let max_len = self.words.len().max(other.words.len());
        let mut result = BitmapIndex::new(self.size.max(other.size));
        for i in 0..len {
            result.words[i] = self.words[i] | other.words[i];
        }
        if self.words.len() > other.words.len() {
            for i in len..max_len { result.words[i] = self.words[i]; }
        } else if other.words.len() > self.words.len() {
            for i in len..max_len { result.words[i] = other.words[i]; }
        }
        result
    }

    pub fn xor(&self, other: &BitmapIndex) -> BitmapIndex {
        let len = self.words.len().min(other.words.len());
        let max_len = self.words.len().max(other.words.len());
        let mut result = BitmapIndex::new(self.size.max(other.size));
        for i in 0..len {
            result.words[i] = self.words[i] ^ other.words[i];
        }
        if self.words.len() > other.words.len() {
            for i in len..max_len { result.words[i] = self.words[i]; }
        } else if other.words.len() > self.words.len() {
            for i in len..max_len { result.words[i] = other.words[i]; }
        }
        result
    }

    pub fn set_bits(&self) -> Vec<usize> {
        let mut bits = Vec::new();
        for (wi, &word) in self.words.iter().enumerate() {
            let mut w = word;
            while w != 0 {
                let bit = w.trailing_zeros() as usize;
                bits.push(wi * BITS + bit);
                w &= !(1u64 << bit);
            }
        }
        bits
    }

    pub fn is_empty(&self) -> bool {
        self.count_ones() == 0
    }

    pub fn fill_ratio(&self) -> f64 {
        if self.size == 0 { 0.0 } else { self.count_ones() as f64 / self.size as f64 }
    }

    pub fn clear_all(&mut self) {
        self.words.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_index() {
        let bi = BitmapIndex::new(128);
        assert_eq!(bi.size(), 128);
        assert!(bi.is_empty());
    }

    #[test]
    fn set_and_get() {
        let mut bi = BitmapIndex::new(64);
        bi.set(5).unwrap();
        assert!(bi.get(5));
        assert!(!bi.get(6));
    }

    #[test]
    fn clear() {
        let mut bi = BitmapIndex::new(64);
        bi.set(5).unwrap();
        assert!(bi.clear(5).unwrap());
        assert!(!bi.get(5));
    }

    #[test]
    fn out_of_range() {
        let mut bi = BitmapIndex::new(64);
        let err = bi.set(64).unwrap_err();
        assert!(matches!(err, BmIndexError::OutOfRange { .. }));
    }

    #[test]
    fn count_ones() {
        let mut bi = BitmapIndex::new(128);
        bi.set(0).unwrap();
        bi.set(63).unwrap();
        bi.set(100).unwrap();
        assert_eq!(bi.count_ones(), 3);
        assert_eq!(bi.count_zeros(), 125);
    }

    #[test]
    fn and_op() {
        let mut a = BitmapIndex::new(64);
        let mut b = BitmapIndex::new(64);
        a.set(0).unwrap(); a.set(1).unwrap(); a.set(2).unwrap();
        b.set(1).unwrap(); b.set(2).unwrap(); b.set(3).unwrap();
        let c = a.and(&b);
        assert_eq!(c.set_bits(), vec![1, 2]);
    }

    #[test]
    fn or_op() {
        let mut a = BitmapIndex::new(64);
        let mut b = BitmapIndex::new(64);
        a.set(0).unwrap(); a.set(1).unwrap();
        b.set(2).unwrap(); b.set(3).unwrap();
        let c = a.or(&b);
        assert_eq!(c.count_ones(), 4);
    }

    #[test]
    fn xor_op() {
        let mut a = BitmapIndex::new(64);
        let mut b = BitmapIndex::new(64);
        a.set(0).unwrap(); a.set(1).unwrap();
        b.set(1).unwrap(); b.set(2).unwrap();
        let c = a.xor(&b);
        assert_eq!(c.set_bits(), vec![0, 2]);
    }

    #[test]
    fn set_bits_list() {
        let mut bi = BitmapIndex::new(64);
        bi.set(3).unwrap();
        bi.set(7).unwrap();
        bi.set(60).unwrap();
        assert_eq!(bi.set_bits(), vec![3, 7, 60]);
    }

    #[test]
    fn fill_ratio() {
        let mut bi = BitmapIndex::new(100);
        bi.set(0).unwrap();
        bi.set(50).unwrap();
        assert!((bi.fill_ratio() - 0.02).abs() < 0.001);
    }

    #[test]
    fn clear_all() {
        let mut bi = BitmapIndex::new(64);
        bi.set(0).unwrap(); bi.set(1).unwrap();
        bi.clear_all();
        assert!(bi.is_empty());
    }

    #[test]
    fn error_display() {
        assert!(BmIndexError::OutOfRange { bit: 99, size: 64 }.to_string().contains("99"));
    }
}

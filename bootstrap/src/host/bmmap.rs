const BITS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BmmapError {
    Exhausted { regions: usize },
    InvalidRegion { index: usize },
    AlreadyFree { index: usize },
    OutOfRange { index: usize, total: usize },
}

impl std::fmt::Display for BmmapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BmmapError::Exhausted { regions } => write!(f, "all {regions} regions allocated"),
            BmmapError::InvalidRegion { index } => write!(f, "region {index} invalid"),
            BmmapError::AlreadyFree { index } => write!(f, "region {index} already free"),
            BmmapError::OutOfRange { index, total } => write!(f, "region {index} >= {total}"),
        }
    }
}

impl std::error::Error for BmmapError {}

#[derive(Debug, Clone)]
pub struct BmmapStats {
    pub total: usize,
    pub allocated: usize,
    pub free: usize,
    pub first_free: Option<usize>,
    pub contiguity: usize,
}

#[derive(Debug, Clone)]
pub struct BitmapAllocator {
    bitmap: Vec<u64>,
    region_size: usize,
    total_regions: usize,
    total_alloc: u64,
    total_free: u64,
    peak_used: usize,
}

impl BitmapAllocator {
    pub fn new(region_size: usize, total_regions: usize) -> Self {
        let words = (total_regions + BITS - 1) / BITS;
        Self {
            bitmap: vec![0u64; words],
            region_size,
            total_regions,
            total_alloc: 0,
            total_free: 0,
            peak_used: 0,
        }
    }

    pub fn region_size(&self) -> usize {
        self.region_size
    }

    pub fn total_regions(&self) -> usize {
        self.total_regions
    }

    pub fn alloc(&mut self) -> Result<usize, BmmapError> {
        for (word_idx, word) in self.bitmap.iter_mut().enumerate() {
            if *word != !0u64 {
                let bit = word.trailing_ones() as usize;
                let region = word_idx * BITS + bit;
                if region >= self.total_regions {
                    return Err(BmmapError::Exhausted { regions: self.total_regions });
                }
                *word |= 1u64 << bit;
                self.total_alloc += 1;
                let used = self.used();
                if used > self.peak_used { self.peak_used = used; }
                return Ok(region);
            }
        }
        Err(BmmapError::Exhausted { regions: self.total_regions })
    }

    pub fn alloc_at(&mut self, index: usize) -> Result<usize, BmmapError> {
        if index >= self.total_regions {
            return Err(BmmapError::OutOfRange { index, total: self.total_regions });
        }
        let word_idx = index / BITS;
        let bit = index % BITS;
        if self.bitmap[word_idx] & (1u64 << bit) != 0 {
            return Err(BmmapError::InvalidRegion { index });
        }
        self.bitmap[word_idx] |= 1u64 << bit;
        self.total_alloc += 1;
        let used = self.used();
        if used > self.peak_used { self.peak_used = used; }
        Ok(index)
    }

    pub fn free(&mut self, index: usize) -> Result<(), BmmapError> {
        if index >= self.total_regions {
            return Err(BmmapError::OutOfRange { index, total: self.total_regions });
        }
        let word_idx = index / BITS;
        let bit = index % BITS;
        if self.bitmap[word_idx] & (1u64 << bit) == 0 {
            return Err(BmmapError::AlreadyFree { index });
        }
        self.bitmap[word_idx] &= !(1u64 << bit);
        self.total_free += 1;
        Ok(())
    }

    pub fn is_allocated(&self, index: usize) -> bool {
        if index >= self.total_regions { return false; }
        let word_idx = index / BITS;
        let bit = index % BITS;
        self.bitmap[word_idx] & (1u64 << bit) != 0
    }

    pub fn addr_of(&self, index: usize) -> usize {
        index * self.region_size
    }

    pub fn index_of(&self, addr: usize) -> usize {
        addr / self.region_size
    }

    pub fn used(&self) -> usize {
        self.bitmap.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn free_count(&self) -> usize {
        self.total_regions - self.used()
    }

    pub fn first_free(&self) -> Option<usize> {
        for (word_idx, word) in self.bitmap.iter().enumerate() {
            if *word != !0u64 {
                let bit = word.trailing_ones() as usize;
                let region = word_idx * BITS + bit;
                if region < self.total_regions { return Some(region); }
            }
        }
        None
    }

    pub fn max_contiguous(&self) -> usize {
        let mut max_run = 0;
        let mut run = 0;
        for i in 0..self.total_regions {
            if !self.is_allocated(i) {
                run += 1;
                if run > max_run { max_run = run; }
            } else {
                run = 0;
            }
        }
        max_run
    }

    pub fn stats(&self) -> BmmapStats {
        BmmapStats {
            total: self.total_regions,
            allocated: self.used(),
            free: self.free_count(),
            first_free: self.first_free(),
            contiguity: self.max_contiguous(),
        }
    }

    pub fn peak_used(&self) -> usize {
        self.peak_used
    }

    pub fn total_alloc(&self) -> u64 {
        self.total_alloc
    }

    pub fn total_free(&self) -> u64 {
        self.total_free
    }

    pub fn utilization(&self) -> f64 {
        if self.total_regions == 0 { 0.0 } else { self.used() as f64 / self.total_regions as f64 }
    }

    pub fn clear(&mut self) {
        self.bitmap.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_allocator() {
        let ba = BitmapAllocator::new(4096, 16);
        assert_eq!(ba.region_size(), 4096);
        assert_eq!(ba.total_regions(), 16);
        assert_eq!(ba.used(), 0);
    }

    #[test]
    fn alloc_sequential() {
        let mut ba = BitmapAllocator::new(4096, 16);
        let r0 = ba.alloc().unwrap();
        let r1 = ba.alloc().unwrap();
        assert_eq!(r0, 0);
        assert_eq!(r1, 1);
    }

    #[test]
    fn alloc_at_specific() {
        let mut ba = BitmapAllocator::new(4096, 16);
        ba.alloc_at(5).unwrap();
        assert!(ba.is_allocated(5));
        assert_eq!(ba.addr_of(5), 20480);
    }

    #[test]
    fn alloc_at_occupied() {
        let mut ba = BitmapAllocator::new(4096, 16);
        ba.alloc_at(3).unwrap();
        let err = ba.alloc_at(3).unwrap_err();
        assert!(matches!(err, BmmapError::InvalidRegion { .. }));
    }

    #[test]
    fn free_and_reuse() {
        let mut ba = BitmapAllocator::new(4096, 4);
        let r = ba.alloc().unwrap();
        ba.free(r).unwrap();
        let r2 = ba.alloc().unwrap();
        assert_eq!(r, r2);
    }

    #[test]
    fn double_free() {
        let mut ba = BitmapAllocator::new(4096, 4);
        let r = ba.alloc().unwrap();
        ba.free(r).unwrap();
        let err = ba.free(r).unwrap_err();
        assert!(matches!(err, BmmapError::AlreadyFree { .. }));
    }

    #[test]
    fn out_of_range() {
        let mut ba = BitmapAllocator::new(4096, 4);
        let err = ba.free(10).unwrap_err();
        assert!(matches!(err, BmmapError::OutOfRange { .. }));
    }

    #[test]
    fn exhausted() {
        let mut ba = BitmapAllocator::new(4096, 2);
        ba.alloc().unwrap();
        ba.alloc().unwrap();
        let err = ba.alloc().unwrap_err();
        assert!(matches!(err, BmmapError::Exhausted { .. }));
    }

    #[test]
    fn max_contiguous() {
        let mut ba = BitmapAllocator::new(4096, 8);
        ba.alloc_at(2).unwrap();
        ba.alloc_at(3).unwrap();
        ba.alloc_at(4).unwrap();
        assert_eq!(ba.max_contiguous(), 3);
    }

    #[test]
    fn stats_snapshot() {
        let mut ba = BitmapAllocator::new(4096, 16);
        ba.alloc().unwrap();
        ba.alloc().unwrap();
        let s = ba.stats();
        assert_eq!(s.allocated, 2);
        assert_eq!(s.free, 14);
        assert_eq!(s.first_free, Some(2));
    }

    #[test]
    fn addr_index_roundtrip() {
        let ba = BitmapAllocator::new(4096, 16);
        assert_eq!(ba.index_of(ba.addr_of(7)), 7);
    }

    #[test]
    fn peak_and_utilization() {
        let mut ba = BitmapAllocator::new(4096, 16);
        ba.alloc().unwrap();
        ba.alloc().unwrap();
        ba.alloc().unwrap();
        assert_eq!(ba.peak_used(), 3);
        assert!((ba.utilization() - 0.1875).abs() < 0.01);
    }

    #[test]
    fn clear() {
        let mut ba = BitmapAllocator::new(4096, 8);
        ba.alloc().unwrap();
        ba.clear();
        assert_eq!(ba.used(), 0);
    }

    #[test]
    fn error_display() {
        assert!(BmmapError::Exhausted { regions: 4 }.to_string().contains("4"));
        assert!(BmmapError::OutOfRange { index: 5, total: 4 }.to_string().contains("5"));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotError {
    Exhausted { capacity: usize },
    NotAllocated { slot: usize },
    AlreadyFree { slot: usize },
    DoubleFree { slot: usize },
}

impl std::fmt::Display for SlotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlotError::Exhausted { capacity } => write!(f, "all {capacity} slots used"),
            SlotError::NotAllocated { slot } => write!(f, "slot {slot} not allocated"),
            SlotError::AlreadyFree { slot } => write!(f, "slot {slot} already free"),
            SlotError::DoubleFree { slot } => write!(f, "double free: slot {slot}"),
        }
    }
}

impl std::error::Error for SlotError {}

const BITS: usize = 64;

#[derive(Debug, Clone)]
pub struct SlotAllocator {
    bitmap: Vec<u64>,
    capacity: usize,
    total_alloc: u64,
    total_free: u64,
    peak_used: usize,
}

impl SlotAllocator {
    pub fn new(capacity: usize) -> Self {
        let words = (capacity + BITS - 1) / BITS;
        Self {
            bitmap: vec![0u64; words],
            capacity,
            total_alloc: 0,
            total_free: 0,
            peak_used: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn alloc(&mut self) -> Result<usize, SlotError> {
        for (word_idx, word) in self.bitmap.iter_mut().enumerate() {
            if *word != !0u64 {
                let bit = word.trailing_ones() as usize;
                let slot = word_idx * BITS + bit;
                if slot >= self.capacity {
                    return Err(SlotError::Exhausted { capacity: self.capacity });
                }
                *word |= 1u64 << bit;
                self.total_alloc += 1;
                let used = self.used();
                if used > self.peak_used {
                    self.peak_used = used;
                }
                return Ok(slot);
            }
        }
        Err(SlotError::Exhausted { capacity: self.capacity })
    }

    pub fn free(&mut self, slot: usize) -> Result<(), SlotError> {
        if slot >= self.capacity {
            return Err(SlotError::NotAllocated { slot });
        }
        let word_idx = slot / BITS;
        let bit = slot % BITS;
        let mask = 1u64 << bit;
        if self.bitmap[word_idx] & mask == 0 {
            return Err(SlotError::DoubleFree { slot });
        }
        self.bitmap[word_idx] &= !mask;
        self.total_free += 1;
        Ok(())
    }

    pub fn is_allocated(&self, slot: usize) -> bool {
        if slot >= self.capacity { return false; }
        let word_idx = slot / BITS;
        let bit = slot % BITS;
        self.bitmap[word_idx] & (1u64 << bit) != 0
    }

    pub fn used(&self) -> usize {
        self.bitmap.iter().map(|w| w.count_ones() as usize).sum::<usize>()
    }

    pub fn free_count(&self) -> usize {
        self.capacity - self.used()
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
        if self.capacity == 0 { 0.0 } else { self.used() as f64 / self.capacity as f64 }
    }

    pub fn allocated_slots(&self) -> Vec<usize> {
        let mut slots = Vec::new();
        for (word_idx, word) in self.bitmap.iter().enumerate() {
            let mut w = *word;
            while w != 0 {
                let bit = w.trailing_zeros() as usize;
                slots.push(word_idx * BITS + bit);
                w &= !(1u64 << bit);
            }
        }
        slots
    }

    pub fn compact(&mut self) -> usize {
        let slots: Vec<usize> = self.allocated_slots();
        let count = slots.len();
        self.bitmap.fill(0);
        for (i, _) in slots.iter().enumerate() {
            let word_idx = i / BITS;
            let bit = i % BITS;
            self.bitmap[word_idx] |= 1u64 << bit;
        }
        count
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
        let sa = SlotAllocator::new(16);
        assert_eq!(sa.capacity(), 16);
        assert_eq!(sa.used(), 0);
    }

    #[test]
    fn alloc_sequential() {
        let mut sa = SlotAllocator::new(4);
        let s0 = sa.alloc().unwrap();
        let s1 = sa.alloc().unwrap();
        assert_eq!(s0, 0);
        assert_eq!(s1, 1);
        assert_eq!(sa.used(), 2);
    }

    #[test]
    fn alloc_exhausted() {
        let mut sa = SlotAllocator::new(2);
        sa.alloc().unwrap();
        sa.alloc().unwrap();
        let err = sa.alloc().unwrap_err();
        assert!(matches!(err, SlotError::Exhausted { .. }));
    }

    #[test]
    fn free_and_reuse() {
        let mut sa = SlotAllocator::new(4);
        sa.alloc().unwrap();
        let s1 = sa.alloc().unwrap();
        sa.free(s1).unwrap();
        let s2 = sa.alloc().unwrap();
        assert_eq!(s1, s2);
    }

    #[test]
    fn double_free() {
        let mut sa = SlotAllocator::new(4);
        let s = sa.alloc().unwrap();
        sa.free(s).unwrap();
        let err = sa.free(s).unwrap_err();
        assert!(matches!(err, SlotError::DoubleFree { .. }));
    }

    #[test]
    fn free_out_of_range() {
        let mut sa = SlotAllocator::new(4);
        let err = sa.free(99).unwrap_err();
        assert!(matches!(err, SlotError::NotAllocated { .. }));
    }

    #[test]
    fn is_allocated() {
        let mut sa = SlotAllocator::new(4);
        let s = sa.alloc().unwrap();
        assert!(sa.is_allocated(s));
        assert!(!sa.is_allocated(1));
    }

    #[test]
    fn peak_used() {
        let mut sa = SlotAllocator::new(8);
        sa.alloc().unwrap();
        sa.alloc().unwrap();
        sa.alloc().unwrap();
        assert_eq!(sa.peak_used(), 3);
        sa.free(1).unwrap();
        assert_eq!(sa.peak_used(), 3);
    }

    #[test]
    fn allocated_slots() {
        let mut sa = SlotAllocator::new(8);
        sa.alloc().unwrap();
        sa.alloc().unwrap();
        sa.alloc().unwrap();
        sa.free(1).unwrap();
        assert_eq!(sa.allocated_slots(), vec![0, 2]);
    }

    #[test]
    fn compact() {
        let mut sa = SlotAllocator::new(8);
        sa.alloc().unwrap();
        sa.alloc().unwrap();
        sa.alloc().unwrap();
        sa.free(1).unwrap();
        let count = sa.compact();
        assert_eq!(count, 2);
        assert_eq!(sa.allocated_slots(), vec![0, 1]);
    }

    #[test]
    fn stats() {
        let mut sa = SlotAllocator::new(4);
        sa.alloc().unwrap();
        sa.alloc().unwrap();
        sa.free(0).unwrap();
        assert_eq!(sa.total_alloc(), 2);
        assert_eq!(sa.total_free(), 1);
        assert!((sa.utilization() - 0.25).abs() < 0.01);
    }

    #[test]
    fn large_allocator() {
        let mut sa = SlotAllocator::new(200);
        for _ in 0..100 { sa.alloc().unwrap(); }
        assert_eq!(sa.used(), 100);
        assert_eq!(sa.free_count(), 100);
    }

    #[test]
    fn error_display() {
        assert!(SlotError::Exhausted { capacity: 8 }.to_string().contains("8"));
        assert!(SlotError::DoubleFree { slot: 3 }.to_string().contains("3"));
    }
}

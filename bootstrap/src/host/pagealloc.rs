use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageError {
    OutOfMemory { requested: usize },
    DoubleFree { addr: usize },
    NotPageAligned { addr: usize, page_size: usize },
}

impl std::fmt::Display for PageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageError::OutOfMemory { requested } => write!(f, "OOM: need {requested} pages"),
            PageError::DoubleFree { addr } => write!(f, "double free: {addr:#x}"),
            PageError::NotPageAligned { addr, page_size } => write!(f, "{addr:#x} not aligned to {page_size}"),
        }
    }
}

impl std::error::Error for PageError {}

#[derive(Debug, Clone)]
pub struct PageAllocator {
    page_size: usize,
    total_pages: usize,
    bitmap: Vec<u64>,
    next_addr: usize,
    total_alloc: u64,
    total_free: u64,
    peak_used: usize,
    current_used: usize,
}

impl PageAllocator {
    pub fn new(page_size: usize, total_pages: usize, _max_order: usize) -> Self {
        let bitmap_words = (total_pages + 63) / 64;
        Self {
            page_size,
            total_pages,
            bitmap: vec![0u64; bitmap_words],
            next_addr: 0,
            total_alloc: 0,
            total_free: 0,
            peak_used: 0,
            current_used: 0,
        }
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }

    pub fn total_pages(&self) -> usize {
        self.total_pages
    }

    fn find_free(&self, count: usize) -> Option<usize> {
        let mut run = 0;
        let mut start = 0;
        for i in 0..self.total_pages {
            let word = i / 64;
            let bit = i % 64;
            if self.bitmap[word] & (1u64 << bit) == 0 {
                if run == 0 { start = i; }
                run += 1;
                if run >= count { return Some(start); }
            } else {
                run = 0;
            }
        }
        None
    }

    fn mark_allocated(&mut self, start: usize, count: usize) {
        for i in start..start + count {
            let word = i / 64;
            let bit = i % 64;
            self.bitmap[word] |= 1u64 << bit;
        }
    }

    fn mark_free(&mut self, start: usize, count: usize) {
        for i in start..start + count {
            let word = i / 64;
            let bit = i % 64;
            self.bitmap[word] &= !(1u64 << bit);
        }
    }

    pub fn alloc(&mut self, pages: usize) -> Result<usize, PageError> {
        let count = pages.max(1);
        let start = self.find_free(count)
            .ok_or(PageError::OutOfMemory { requested: pages })?;
        self.mark_allocated(start, count);
        let addr = start * self.page_size;
        self.total_alloc += 1;
        self.current_used += count;
        if self.current_used > self.peak_used {
            self.peak_used = self.current_used;
        }
        Ok(addr)
    }

    pub fn free(&mut self, addr: usize) -> Result<(), PageError> {
        if addr % self.page_size != 0 {
            return Err(PageError::NotPageAligned { addr, page_size: self.page_size });
        }
        let start = addr / self.page_size;
        if start >= self.total_pages {
            return Err(PageError::DoubleFree { addr });
        }
        let word = start / 64;
        let bit = start % 64;
        if self.bitmap[word] & (1u64 << bit) == 0 {
            return Err(PageError::DoubleFree { addr });
        }
        self.mark_free(start, 1);
        self.total_free += 1;
        if self.current_used > 0 {
            self.current_used -= 1;
        }
        Ok(())
    }

    pub fn is_allocated(&self, addr: usize) -> bool {
        if addr % self.page_size != 0 { return false; }
        let idx = addr / self.page_size;
        if idx >= self.total_pages { return false; }
        let word = idx / 64;
        let bit = idx % 64;
        self.bitmap[word] & (1u64 << bit) != 0
    }

    pub fn used_pages(&self) -> usize {
        self.current_used
    }

    pub fn free_pages(&self) -> usize {
        self.total_pages - self.current_used
    }

    pub fn allocated_blocks(&self) -> usize {
        (self.total_alloc - self.total_free) as usize
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
        if self.total_pages == 0 { 0.0 } else { self.current_used as f64 / self.total_pages as f64 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_allocator() {
        let pa = PageAllocator::new(4096, 16, 4);
        assert_eq!(pa.page_size(), 4096);
        assert_eq!(pa.total_pages(), 16);
    }

    #[test]
    fn alloc_single_page() {
        let mut pa = PageAllocator::new(4096, 16, 4);
        let addr = pa.alloc(1).unwrap();
        assert_eq!(addr, 0);
        assert!(pa.is_allocated(addr));
    }

    #[test]
    fn alloc_multiple() {
        let mut pa = PageAllocator::new(4096, 16, 4);
        let a = pa.alloc(1).unwrap();
        let b = pa.alloc(1).unwrap();
        assert_ne!(a, b);
        assert!(pa.is_allocated(a));
        assert!(pa.is_allocated(b));
    }

    #[test]
    fn free_and_reuse() {
        let mut pa = PageAllocator::new(4096, 16, 4);
        let a = pa.alloc(1).unwrap();
        pa.free(a).unwrap();
        assert!(!pa.is_allocated(a));
    }

    #[test]
    fn double_free() {
        let mut pa = PageAllocator::new(4096, 16, 4);
        let a = pa.alloc(1).unwrap();
        pa.free(a).unwrap();
        let err = pa.free(a).unwrap_err();
        assert!(matches!(err, PageError::DoubleFree { .. }));
    }

    #[test]
    fn not_aligned() {
        let mut pa = PageAllocator::new(4096, 16, 4);
        let err = pa.free(1).unwrap_err();
        assert!(matches!(err, PageError::NotPageAligned { .. }));
    }

    #[test]
    fn oom() {
        let mut pa = PageAllocator::new(4096, 2, 0);
        pa.alloc(1).unwrap();
        pa.alloc(1).unwrap();
        let err = pa.alloc(1).unwrap_err();
        assert!(matches!(err, PageError::OutOfMemory { .. }));
    }

    #[test]
    fn stats() {
        let mut pa = PageAllocator::new(4096, 16, 4);
        pa.alloc(1).unwrap();
        pa.alloc(1).unwrap();
        assert_eq!(pa.total_alloc(), 2);
        assert_eq!(pa.used_pages(), 2);
        assert_eq!(pa.free_pages(), 14);
    }

    #[test]
    fn peak_used() {
        let mut pa = PageAllocator::new(4096, 16, 4);
        pa.alloc(1).unwrap();
        pa.alloc(1).unwrap();
        pa.alloc(1).unwrap();
        let a = pa.alloc(1).unwrap();
        assert_eq!(pa.peak_used(), 4);
        pa.free(a).unwrap();
        assert_eq!(pa.peak_used(), 4);
    }

    #[test]
    fn utilization() {
        let mut pa = PageAllocator::new(4096, 16, 4);
        pa.alloc(4).unwrap();
        assert!((pa.utilization() - 0.25).abs() < 0.01);
    }

    #[test]
    fn alloc_contiguous() {
        let mut pa = PageAllocator::new(4096, 16, 4);
        let addr = pa.alloc(4).unwrap();
        assert_eq!(addr, 0);
        assert_eq!(pa.used_pages(), 4);
    }

    #[test]
    fn error_display() {
        let e = PageError::OutOfMemory { requested: 42 };
        assert!(e.to_string().contains("42"));
        let e2 = PageError::DoubleFree { addr: 0x1000 };
        assert!(e2.to_string().contains("1000"));
    }
}

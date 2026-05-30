#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingError {
    Full,
    Empty,
    IndexOutOfBounds { index: usize, len: usize },
}

impl std::fmt::Display for RingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RingError::Full => write!(f, "descriptor ring full"),
            RingError::Empty => write!(f, "descriptor ring empty"),
            RingError::IndexOutOfBounds { index, len } => {
                write!(f, "index {index} out of bounds (len={len})")
            }
        }
    }
}

impl std::error::Error for RingError {}

pub const DEFAULT_RING_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescStatus {
    Free,
    Pending,
    Active,
    Completed,
    Error,
}

impl std::fmt::Display for DescStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DescStatus::Free => write!(f, "free"),
            DescStatus::Pending => write!(f, "pending"),
            DescStatus::Active => write!(f, "active"),
            DescStatus::Completed => write!(f, "completed"),
            DescStatus::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Descriptor {
    pub addr: u64,
    pub len: u32,
    pub status: DescStatus,
    pub tag: u8,
}

impl Descriptor {
    pub const fn new(addr: u64, len: u32) -> Self {
        Self {
            addr,
            len,
            status: DescStatus::Free,
            tag: 0,
        }
    }

    pub fn with_tag(mut self, tag: u8) -> Self {
        self.tag = tag;
        self
    }
}

#[derive(Debug, Clone)]
pub struct DescriptorRing {
    ring: Vec<Descriptor>,
    head: usize,
    tail: usize,
    count: usize,
}

impl DescriptorRing {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            ring: vec![Descriptor::new(0, 0); capacity],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn enqueue(&mut self, desc: Descriptor) -> Result<usize, RingError> {
        if self.count >= self.ring.len() {
            return Err(RingError::Full);
        }
        let idx = self.tail;
        let mut d = desc;
        d.status = DescStatus::Pending;
        self.ring[idx] = d;
        self.tail = (self.tail + 1) % self.ring.len();
        self.count += 1;
        Ok(idx)
    }

    pub fn dequeue(&mut self) -> Result<Descriptor, RingError> {
        if self.count == 0 {
            return Err(RingError::Empty);
        }
        let idx = self.head;
        let desc = self.ring[idx];
        self.ring[idx].status = DescStatus::Free;
        self.head = (self.head + 1) % self.ring.len();
        self.count -= 1;
        Ok(desc)
    }

    pub fn peek(&self) -> Result<&Descriptor, RingError> {
        if self.count == 0 {
            return Err(RingError::Empty);
        }
        Ok(&self.ring[self.head])
    }

    pub fn get(&self, index: usize) -> Result<&Descriptor, RingError> {
        if index >= self.ring.len() {
            return Err(RingError::IndexOutOfBounds {
                index,
                len: self.ring.len(),
            });
        }
        Ok(&self.ring[index])
    }

    pub fn get_mut(&mut self, index: usize) -> Result<&mut Descriptor, RingError> {
        if index >= self.ring.len() {
            return Err(RingError::IndexOutOfBounds {
                index,
                len: self.ring.len(),
            });
        }
        Ok(&mut self.ring[index])
    }

    pub fn mark_active(&mut self, index: usize) -> Result<(), RingError> {
        let desc = self.get_mut(index)?;
        desc.status = DescStatus::Active;
        Ok(())
    }

    pub fn mark_completed(&mut self, index: usize) -> Result<(), RingError> {
        let desc = self.get_mut(index)?;
        desc.status = DescStatus::Completed;
        Ok(())
    }

    pub fn mark_error(&mut self, index: usize) -> Result<(), RingError> {
        let desc = self.get_mut(index)?;
        desc.status = DescStatus::Error;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn is_full(&self) -> bool {
        self.count >= self.ring.len()
    }

    pub fn capacity(&self) -> usize {
        self.ring.len()
    }

    pub fn clear(&mut self) {
        for d in &mut self.ring {
            d.status = DescStatus::Free;
        }
        self.head = 0;
        self.tail = 0;
        self.count = 0;
    }

    pub fn count_by_status(&self, status: DescStatus) -> usize {
        self.ring.iter().filter(|d| d.status == status).count()
    }

    pub fn stats(&self) -> RingStats {
        RingStats {
            capacity: self.capacity(),
            used: self.count,
            free: self.capacity() - self.count,
            pending: self.count_by_status(DescStatus::Pending),
            active: self.count_by_status(DescStatus::Active),
            completed: self.count_by_status(DescStatus::Completed),
            errors: self.count_by_status(DescStatus::Error),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RingStats {
    pub capacity: usize,
    pub used: usize,
    pub free: usize,
    pub pending: usize,
    pub active: usize,
    pub completed: usize,
    pub errors: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ring_is_empty() {
        let r = DescriptorRing::new(8);
        assert!(r.is_empty());
        assert_eq!(r.capacity(), 8);
    }

    #[test]
    fn enqueue_dequeue() {
        let mut r = DescriptorRing::new(4);
        let idx = r.enqueue(Descriptor::new(0x1000, 64)).unwrap();
        assert_eq!(r.len(), 1);
        let d = r.dequeue().unwrap();
        assert_eq!(d.addr, 0x1000);
        assert_eq!(d.len, 64);
        assert!(r.is_empty());
    }

    #[test]
    fn enqueue_sets_pending() {
        let mut r = DescriptorRing::new(4);
        r.enqueue(Descriptor::new(0x1000, 64)).unwrap();
        assert_eq!(r.peek().unwrap().status, DescStatus::Pending);
    }

    #[test]
    fn fifo_order() {
        let mut r = DescriptorRing::new(8);
        r.enqueue(Descriptor::new(0x1000, 1)).unwrap();
        r.enqueue(Descriptor::new(0x2000, 2)).unwrap();
        r.enqueue(Descriptor::new(0x3000, 3)).unwrap();
        assert_eq!(r.dequeue().unwrap().addr, 0x1000);
        assert_eq!(r.dequeue().unwrap().addr, 0x2000);
        assert_eq!(r.dequeue().unwrap().addr, 0x3000);
    }

    #[test]
    fn wrap_around() {
        let mut r = DescriptorRing::new(3);
        r.enqueue(Descriptor::new(0xA, 1)).unwrap();
        r.enqueue(Descriptor::new(0xB, 2)).unwrap();
        r.dequeue().unwrap();
        r.enqueue(Descriptor::new(0xC, 3)).unwrap();
        assert_eq!(r.dequeue().unwrap().addr, 0xB);
        assert_eq!(r.dequeue().unwrap().addr, 0xC);
    }

    #[test]
    fn full_errors() {
        let mut r = DescriptorRing::new(2);
        r.enqueue(Descriptor::new(0, 0)).unwrap();
        r.enqueue(Descriptor::new(0, 0)).unwrap();
        assert!(r.is_full());
        assert_eq!(r.enqueue(Descriptor::new(0, 0)).unwrap_err(), RingError::Full);
    }

    #[test]
    fn empty_dequeue_errors() {
        let mut r = DescriptorRing::new(4);
        assert_eq!(r.dequeue().unwrap_err(), RingError::Empty);
    }

    #[test]
    fn mark_status() {
        let mut r = DescriptorRing::new(4);
        let idx = r.enqueue(Descriptor::new(0x1000, 64)).unwrap();
        r.mark_active(idx).unwrap();
        assert_eq!(r.get(idx).unwrap().status, DescStatus::Active);
        r.mark_completed(idx).unwrap();
        assert_eq!(r.get(idx).unwrap().status, DescStatus::Completed);
        r.mark_error(idx).unwrap();
        assert_eq!(r.get(idx).unwrap().status, DescStatus::Error);
    }

    #[test]
    fn get_out_of_bounds() {
        let r = DescriptorRing::new(4);
        assert!(matches!(r.get(5), Err(RingError::IndexOutOfBounds { .. })));
    }

    #[test]
    fn with_tag() {
        let d = Descriptor::new(0x1000, 64).with_tag(7);
        assert_eq!(d.tag, 7);
    }

    #[test]
    fn clear() {
        let mut r = DescriptorRing::new(4);
        r.enqueue(Descriptor::new(0, 0)).unwrap();
        r.clear();
        assert!(r.is_empty());
        assert_eq!(r.count_by_status(DescStatus::Free), 4);
    }

    #[test]
    fn stats() {
        let mut r = DescriptorRing::new(8);
        r.enqueue(Descriptor::new(0, 0)).unwrap();
        r.enqueue(Descriptor::new(0, 0)).unwrap();
        let s = r.stats();
        assert_eq!(s.capacity, 8);
        assert_eq!(s.used, 2);
        assert_eq!(s.free, 6);
        assert_eq!(s.pending, 2);
    }

    #[test]
    fn desc_status_display() {
        assert_eq!(DescStatus::Active.to_string(), "active");
        assert_eq!(DescStatus::Completed.to_string(), "completed");
    }

    #[test]
    fn error_display() {
        assert!(RingError::Full.to_string().contains("full"));
        assert!(RingError::Empty.to_string().contains("empty"));
    }
}

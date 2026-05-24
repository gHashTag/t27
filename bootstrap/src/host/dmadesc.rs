#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescStatus {
    Free,
    Pending,
    Active,
    Complete,
    Error,
}

impl std::fmt::Display for DescStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DescStatus::Free => write!(f, "free"),
            DescStatus::Pending => write!(f, "pending"),
            DescStatus::Active => write!(f, "active"),
            DescStatus::Complete => write!(f, "complete"),
            DescStatus::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDir {
    ToDevice,
    FromDevice,
}

impl std::fmt::Display for DmaDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DmaDir::ToDevice => write!(f, "tx"),
            DmaDir::FromDevice => write!(f, "rx"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DmaDescriptor {
    pub index: usize,
    pub src_addr: u32,
    pub dst_addr: u32,
    pub length: u32,
    pub dir: DmaDir,
    pub status: DescStatus,
    pub tag: u16,
}

impl DmaDescriptor {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            src_addr: 0,
            dst_addr: 0,
            length: 0,
            dir: DmaDir::ToDevice,
            status: DescStatus::Free,
            tag: 0,
        }
    }

    pub fn configure(mut self, src: u32, dst: u32, len: u32, dir: DmaDir) -> Self {
        self.src_addr = src;
        self.dst_addr = dst;
        self.length = len;
        self.dir = dir;
        self
    }

    pub fn with_tag(mut self, tag: u16) -> Self {
        self.tag = tag;
        self
    }

    pub fn submit(&mut self) -> Result<(), &'static str> {
        if self.status != DescStatus::Free {
            return Err("descriptor not free");
        }
        self.status = DescStatus::Pending;
        Ok(())
    }

    pub fn activate(&mut self) -> Result<(), &'static str> {
        if self.status != DescStatus::Pending {
            return Err("descriptor not pending");
        }
        self.status = DescStatus::Active;
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), &'static str> {
        if self.status != DescStatus::Active {
            return Err("descriptor not active");
        }
        self.status = DescStatus::Complete;
        Ok(())
    }

    pub fn mark_error(&mut self) {
        self.status = DescStatus::Error;
    }

    pub fn release(&mut self) {
        self.status = DescStatus::Free;
        self.src_addr = 0;
        self.dst_addr = 0;
        self.length = 0;
        self.tag = 0;
    }

    pub fn is_free(&self) -> bool {
        self.status == DescStatus::Free
    }

    pub fn is_done(&self) -> bool {
        self.status == DescStatus::Complete || self.status == DescStatus::Error
    }
}

#[derive(Debug, Clone)]
pub struct DmaDescriptorRing {
    descs: Vec<DmaDescriptor>,
    head: usize,
    tail: usize,
    total_submitted: u64,
    total_completed: u64,
    total_errors: u64,
}

impl DmaDescriptorRing {
    pub fn new(count: usize) -> Self {
        let descs = (0..count).map(|i| DmaDescriptor::new(i)).collect();
        Self {
            descs,
            head: 0,
            tail: 0,
            total_submitted: 0,
            total_completed: 0,
            total_errors: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.descs.len()
    }

    pub fn alloc(&mut self, src: u32, dst: u32, len: u32, dir: DmaDir) -> Option<usize> {
        let desc = self.descs.iter_mut().find(|d| d.is_free())?;
        desc.src_addr = src;
        desc.dst_addr = dst;
        desc.length = len;
        desc.dir = dir;
        desc.submit().ok()?;
        self.total_submitted += 1;
        Some(desc.index)
    }

    pub fn alloc_tagged(&mut self, src: u32, dst: u32, len: u32, dir: DmaDir, tag: u16) -> Option<usize> {
        let idx = self.alloc(src, dst, len, dir)?;
        self.descs[idx].tag = tag;
        Some(idx)
    }

    pub fn get(&self, index: usize) -> Option<&DmaDescriptor> {
        self.descs.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut DmaDescriptor> {
        self.descs.get_mut(index)
    }

    pub fn advance(&mut self) -> Option<usize> {
        let desc = &self.descs[self.head];
        if desc.status != DescStatus::Pending {
            return None;
        }
        let idx = self.head;
        self.descs[self.head].activate().ok()?;
        self.head = (self.head + 1) % self.descs.len();
        Some(idx)
    }

    pub fn complete_next(&mut self) -> Option<usize> {
        let desc = &self.descs[self.tail];
        if desc.status != DescStatus::Active {
            return None;
        }
        let idx = self.tail;
        self.descs[self.tail].complete().ok()?;
        self.total_completed += 1;
        self.tail = (self.tail + 1) % self.descs.len();
        Some(idx)
    }

    pub fn complete_by_index(&mut self, index: usize) -> bool {
        if let Some(desc) = self.descs.get_mut(index) {
            if desc.complete().is_ok() {
                self.total_completed += 1;
                return true;
            }
        }
        false
    }

    pub fn free_count(&self) -> usize {
        self.descs.iter().filter(|d| d.is_free()).count()
    }

    pub fn pending_count(&self) -> usize {
        self.descs.iter().filter(|d| d.status == DescStatus::Pending || d.status == DescStatus::Active).count()
    }

    pub fn total_submitted(&self) -> u64 {
        self.total_submitted
    }

    pub fn total_completed(&self) -> u64 {
        self.total_completed
    }

    pub fn total_errors(&self) -> u64 {
        self.total_errors
    }

    pub fn release(&mut self, index: usize) {
        if let Some(desc) = self.descs.get_mut(index) {
            if desc.status == DescStatus::Error {
                self.total_errors += 1;
            }
            desc.release();
        }
    }

    pub fn release_all_done(&mut self) -> usize {
        let mut count = 0;
        for desc in &mut self.descs {
            if desc.is_done() {
                if desc.status == DescStatus::Error {
                    self.total_errors += 1;
                }
                desc.release();
                count += 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_display() {
        assert_eq!(DescStatus::Free.to_string(), "free");
        assert_eq!(DescStatus::Error.to_string(), "error");
    }

    #[test]
    fn dir_display() {
        assert_eq!(DmaDir::ToDevice.to_string(), "tx");
        assert_eq!(DmaDir::FromDevice.to_string(), "rx");
    }

    #[test]
    fn descriptor_lifecycle() {
        let mut d = DmaDescriptor::new(0)
            .configure(0x1000, 0x2000, 64, DmaDir::ToDevice)
            .with_tag(42);
        assert!(d.is_free());
        d.submit().unwrap();
        assert_eq!(d.status, DescStatus::Pending);
        d.activate().unwrap();
        assert_eq!(d.status, DescStatus::Active);
        d.complete().unwrap();
        assert!(d.is_done());
        d.release();
        assert!(d.is_free());
    }

    #[test]
    fn submit_not_free_fails() {
        let mut d = DmaDescriptor::new(0);
        d.submit().unwrap();
        assert!(d.submit().is_err());
    }

    #[test]
    fn ring_alloc() {
        let mut ring = DmaDescriptorRing::new(4);
        let idx = ring.alloc(0x1000, 0x2000, 128, DmaDir::FromDevice).unwrap();
        assert_eq!(ring.pending_count(), 1);
        assert_eq!(ring.free_count(), 3);
        assert_eq!(ring.get(idx).unwrap().tag, 0);
    }

    #[test]
    fn ring_alloc_tagged() {
        let mut ring = DmaDescriptorRing::new(4);
        let idx = ring.alloc_tagged(0, 0, 64, DmaDir::ToDevice, 99).unwrap();
        assert_eq!(ring.get(idx).unwrap().tag, 99);
    }

    #[test]
    fn ring_exhaustion() {
        let mut ring = DmaDescriptorRing::new(2);
        ring.alloc(0, 0, 1, DmaDir::ToDevice).unwrap();
        ring.alloc(0, 0, 1, DmaDir::ToDevice).unwrap();
        assert!(ring.alloc(0, 0, 1, DmaDir::ToDevice).is_none());
    }

    #[test]
    fn ring_complete_and_release() {
        let mut ring = DmaDescriptorRing::new(4);
        let idx = ring.alloc(0, 0, 64, DmaDir::ToDevice).unwrap();
        ring.get_mut(idx).unwrap().activate().unwrap();
        ring.complete_by_index(idx);
        ring.release(idx);
        assert_eq!(ring.free_count(), 4);
        assert_eq!(ring.total_completed(), 1);
    }

    #[test]
    fn ring_release_all_done() {
        let mut ring = DmaDescriptorRing::new(4);
        let i1 = ring.alloc(0, 0, 1, DmaDir::ToDevice).unwrap();
        let i2 = ring.alloc(0, 0, 1, DmaDir::ToDevice).unwrap();
        for i in [i1, i2] {
            ring.get_mut(i).unwrap().activate().unwrap();
            ring.get_mut(i).unwrap().complete().unwrap();
        }
        assert_eq!(ring.release_all_done(), 2);
        assert_eq!(ring.free_count(), 4);
    }

    #[test]
    fn ring_stats() {
        let mut ring = DmaDescriptorRing::new(4);
        ring.alloc(0, 0, 1, DmaDir::ToDevice).unwrap();
        assert_eq!(ring.total_submitted(), 1);
    }
}

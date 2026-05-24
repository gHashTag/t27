use std::collections::BTreeSet;

pub type GenId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id(u64, u32);

impl Id {
    pub fn new(index: u64, generation: u32) -> Self {
        Self(index, generation)
    }

    pub fn index(&self) -> u64 {
        self.0
    }

    pub fn generation(&self) -> u32 {
        self.1
    }

    pub fn to_u64(&self) -> u64 {
        (self.1 as u64) << 32 | (self.0 & 0xFFFFFFFF)
    }

    pub fn from_u64(v: u64) -> Self {
        let gen = (v >> 32) as u32;
        let idx = (v & 0xFFFFFFFF) as u64;
        Self(idx, gen)
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdAllocError {
    Exhausted,
    NotAllocated { index: u64 },
    GenerationMismatch { index: u64, expected: u32, got: u32 },
}

impl std::fmt::Display for IdAllocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdAllocError::Exhausted => write!(f, "id space exhausted"),
            IdAllocError::NotAllocated { index } => write!(f, "id {index} not allocated"),
            IdAllocError::GenerationMismatch { index, expected, got } => {
                write!(f, "id {index}: gen {got} != {expected}")
            }
        }
    }
}

impl std::error::Error for IdAllocError {}

#[derive(Debug, Clone)]
struct Slot {
    allocated: bool,
    generation: u32,
}

#[derive(Debug, Clone)]
pub struct IdAllocator {
    slots: Vec<Slot>,
    free_list: Vec<u64>,
    total_allocated: u64,
    total_recycled: u64,
    max_slots: u64,
}

impl IdAllocator {
    pub fn new(max_slots: u64) -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
            total_allocated: 0,
            total_recycled: 0,
            max_slots,
        }
    }

    pub fn alloc(&mut self) -> Result<Id, IdAllocError> {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index as usize];
            slot.allocated = true;
            self.total_allocated += 1;
            return Ok(Id::new(index, slot.generation));
        }
        if self.slots.len() as u64 >= self.max_slots {
            return Err(IdAllocError::Exhausted);
        }
        let index = self.slots.len() as u64;
        self.slots.push(Slot { allocated: true, generation: 0 });
        self.total_allocated += 1;
        Ok(Id::new(index, 0))
    }

    pub fn dealloc(&mut self, id: Id) -> Result<u32, IdAllocError> {
        let idx = id.index() as usize;
        if idx >= self.slots.len() {
            return Err(IdAllocError::NotAllocated { index: id.index() });
        }
        let slot = &mut self.slots[idx];
        if !slot.allocated {
            return Err(IdAllocError::NotAllocated { index: id.index() });
        }
        if slot.generation != id.generation() {
            return Err(IdAllocError::GenerationMismatch {
                index: id.index(),
                expected: slot.generation,
                got: id.generation(),
            });
        }
        slot.allocated = false;
        slot.generation += 1;
        self.free_list.push(id.index());
        self.total_recycled += 1;
        Ok(slot.generation)
    }

    pub fn is_allocated(&self, id: Id) -> bool {
        let idx = id.index() as usize;
        if idx >= self.slots.len() { return false; }
        self.slots[idx].allocated && self.slots[idx].generation == id.generation()
    }

    pub fn allocated_count(&self) -> usize {
        self.slots.iter().filter(|s| s.allocated).count()
    }

    pub fn free_count(&self) -> usize {
        self.free_list.len()
    }

    pub fn capacity(&self) -> u64 {
        self.max_slots
    }

    pub fn total_allocated(&self) -> u64 {
        self.total_allocated
    }

    pub fn total_recycled(&self) -> u64 {
        self.total_recycled
    }

    pub fn recycle_rate(&self) -> f64 {
        if self.total_allocated == 0 { 0.0 } else { self.total_recycled as f64 / self.total_allocated as f64 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_display() {
        assert_eq!(Id::new(5, 2).to_string(), "5:2");
    }

    #[test]
    fn id_roundtrip_u64() {
        let id = Id::new(42, 7);
        let v = id.to_u64();
        let id2 = Id::from_u64(v);
        assert_eq!(id2.index(), 42);
        assert_eq!(id2.generation(), 7);
    }

    #[test]
    fn alloc_first() {
        let mut ia = IdAllocator::new(10);
        let id = ia.alloc().unwrap();
        assert_eq!(id.index(), 0);
        assert_eq!(id.generation(), 0);
        assert!(ia.is_allocated(id));
    }

    #[test]
    fn alloc_sequential() {
        let mut ia = IdAllocator::new(10);
        let id0 = ia.alloc().unwrap();
        let id1 = ia.alloc().unwrap();
        assert_eq!(id0.index(), 0);
        assert_eq!(id1.index(), 1);
        assert_eq!(ia.allocated_count(), 2);
    }

    #[test]
    fn dealloc_and_recycle() {
        let mut ia = IdAllocator::new(10);
        let id = ia.alloc().unwrap();
        let new_gen = ia.dealloc(id).unwrap();
        assert_eq!(new_gen, 1);
        assert!(!ia.is_allocated(id));
        let id2 = ia.alloc().unwrap();
        assert_eq!(id2.index(), 0);
        assert_eq!(id2.generation(), 1);
        assert!(ia.is_allocated(id2));
    }

    #[test]
    fn dealloc_double_fails() {
        let mut ia = IdAllocator::new(10);
        let id = ia.alloc().unwrap();
        ia.dealloc(id).unwrap();
        let err = ia.dealloc(id).unwrap_err();
        assert!(matches!(err, IdAllocError::NotAllocated { .. }));
    }

    #[test]
    fn dealloc_not_allocated() {
        let mut ia = IdAllocator::new(10);
        let err = ia.dealloc(Id::new(99, 0)).unwrap_err();
        assert!(matches!(err, IdAllocError::NotAllocated { .. }));
    }

    #[test]
    fn exhaustion() {
        let mut ia = IdAllocator::new(2);
        ia.alloc().unwrap();
        ia.alloc().unwrap();
        let err = ia.alloc().unwrap_err();
        assert!(matches!(err, IdAllocError::Exhausted));
    }

    #[test]
    fn generation_increments() {
        let mut ia = IdAllocator::new(10);
        let id1 = ia.alloc().unwrap();
        ia.dealloc(id1).unwrap();
        let id2 = ia.alloc().unwrap();
        ia.dealloc(id2).unwrap();
        let id3 = ia.alloc().unwrap();
        assert_eq!(id3.generation(), 2);
    }

    #[test]
    fn stats() {
        let mut ia = IdAllocator::new(10);
        let id = ia.alloc().unwrap();
        ia.dealloc(id).unwrap();
        assert_eq!(ia.total_allocated(), 1);
        assert_eq!(ia.total_recycled(), 1);
        assert!((ia.recycle_rate() - 1.0).abs() < 0.01);
    }

    #[test]
    fn is_allocated_wrong_gen() {
        let mut ia = IdAllocator::new(10);
        let id = ia.alloc().unwrap();
        ia.dealloc(id).unwrap();
        assert!(!ia.is_allocated(id));
    }

    #[test]
    fn error_display() {
        assert!(IdAllocError::Exhausted.to_string().contains("exhausted"));
        assert!(IdAllocError::GenerationMismatch { index: 1, expected: 2, got: 3 }.to_string().contains("1"));
    }
}

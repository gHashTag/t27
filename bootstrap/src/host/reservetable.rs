#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handle {
    pub index: u32,
    pub generation: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RtError {
    InvalidHandle { index: u32, gen: u32, current_gen: u32 },
    TableFull,
}

impl std::fmt::Display for RtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RtError::InvalidHandle { index, gen, current_gen } => write!(f, "stale handle idx={index} gen={gen} (current={current_gen})"),
            RtError::TableFull => write!(f, "table full"),
        }
    }
}

impl std::error::Error for RtError {}

struct Slot<T> {
    data: Option<T>,
    generation: u32,
}

pub struct ReserveTable<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
    cap: usize,
    total_allocs: u64,
    total_deallocs: u64,
}

impl<T> ReserveTable<T> {
    pub fn new(cap: usize) -> Self {
        let slots = (0..cap).map(|_| Slot { data: None, generation: 0 }).collect();
        let free = (0..cap as u32).rev().collect();
        Self { slots, free, cap, total_allocs: 0, total_deallocs: 0 }
    }

    pub fn alloc(&mut self, value: T) -> Result<Handle, RtError> {
        let idx = self.free.pop().ok_or(RtError::TableFull)?;
        self.total_allocs += 1;
        let slot = &mut self.slots[idx as usize];
        slot.data = Some(value);
        Ok(Handle { index: idx, generation: slot.generation })
    }

    pub fn get(&self, handle: Handle) -> Result<&T, RtError> {
        let slot = &self.slots[handle.index as usize];
        if slot.generation != handle.generation || slot.data.is_none() {
            return Err(RtError::InvalidHandle { index: handle.index, gen: handle.generation, current_gen: slot.generation });
        }
        Ok(slot.data.as_ref().unwrap())
    }

    pub fn get_mut(&mut self, handle: Handle) -> Result<&mut T, RtError> {
        let gen = self.slots[handle.index as usize].generation;
        if gen != handle.generation || self.slots[handle.index as usize].data.is_none() {
            return Err(RtError::InvalidHandle { index: handle.index, gen: handle.generation, current_gen: gen });
        }
        Ok(self.slots[handle.index as usize].data.as_mut().unwrap())
    }

    pub fn dealloc(&mut self, handle: Handle) -> Result<T, RtError> {
        let slot = &mut self.slots[handle.index as usize];
        if slot.generation != handle.generation || slot.data.is_none() {
            return Err(RtError::InvalidHandle { index: handle.index, gen: handle.generation, current_gen: slot.generation });
        }
        self.total_deallocs += 1;
        slot.generation += 1;
        let value = slot.data.take().unwrap();
        self.free.push(handle.index);
        Ok(value)
    }

    pub fn is_valid(&self, handle: Handle) -> bool {
        let slot = &self.slots[handle.index as usize];
        slot.generation == handle.generation && slot.data.is_some()
    }

    pub fn len(&self) -> usize { self.cap - self.free.len() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn cap(&self) -> usize { self.cap }
    pub fn free_count(&self) -> usize { self.free.len() }
    pub fn total_allocs(&self) -> u64 { self.total_allocs }
    pub fn total_deallocs(&self) -> u64 { self.total_deallocs }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rt() { let rt: ReserveTable<i32> = ReserveTable::new(8); assert!(rt.is_empty()); assert_eq!(rt.cap(), 8); }

    #[test]
    fn alloc_get() {
        let mut rt = ReserveTable::new(8);
        let h = rt.alloc(42).unwrap();
        assert_eq!(*rt.get(h).unwrap(), 42);
    }

    #[test]
    fn dealloc() {
        let mut rt = ReserveTable::new(8);
        let h = rt.alloc(42).unwrap();
        let v = rt.dealloc(h).unwrap();
        assert_eq!(v, 42);
        assert!(rt.is_empty());
    }

    #[test]
    fn stale_handle() {
        let mut rt = ReserveTable::new(8);
        let h = rt.alloc(1).unwrap();
        rt.dealloc(h).unwrap();
        let h2 = rt.alloc(2).unwrap();
        assert!(rt.get(h).is_err());
        assert_eq!(*rt.get(h2).unwrap(), 2);
        assert_eq!(h.index, h2.index);
        assert_ne!(h.generation, h2.generation);
    }

    #[test]
    fn get_mut() {
        let mut rt = ReserveTable::new(8);
        let h = rt.alloc(1).unwrap();
        *rt.get_mut(h).unwrap() = 99;
        assert_eq!(*rt.get(h).unwrap(), 99);
    }

    #[test]
    fn full() {
        let mut rt = ReserveTable::new(2);
        rt.alloc(1).unwrap(); rt.alloc(2).unwrap();
        assert!(matches!(rt.alloc(3), Err(RtError::TableFull)));
    }

    #[test]
    fn reuse() {
        let mut rt = ReserveTable::new(4);
        let h1 = rt.alloc(1).unwrap();
        rt.dealloc(h1).unwrap();
        let h2 = rt.alloc(2).unwrap();
        assert_eq!(rt.free_count(), 3);
        assert_eq!(rt.len(), 1);
    }

    #[test]
    fn is_valid() {
        let mut rt = ReserveTable::new(4);
        let h = rt.alloc(1).unwrap();
        assert!(rt.is_valid(h));
        rt.dealloc(h).unwrap();
        assert!(!rt.is_valid(h));
    }

    #[test]
    fn stats() {
        let mut rt = ReserveTable::new(4);
        let h = rt.alloc(1).unwrap(); rt.dealloc(h).unwrap();
        assert_eq!(rt.total_allocs(), 1);
        assert_eq!(rt.total_deallocs(), 1);
    }

    #[test]
    fn error_display() { assert!(RtError::TableFull.to_string().contains("full")); }
}

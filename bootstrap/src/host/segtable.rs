#[derive(Debug, Clone, PartialEq)]
pub enum StErr {
    InvalidHandle { gen: u32, expected: u32 },
    SlotEmpty { slot: usize },
    Full { cap: usize },
}

impl std::fmt::Display for StErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StErr::InvalidHandle { gen, expected } => write!(f, "gen {gen} != {expected}"),
            StErr::SlotEmpty { slot } => write!(f, "slot {slot} empty"),
            StErr::Full { cap } => write!(f, "table full {cap}"),
        }
    }
}

impl std::error::Error for StErr {}

#[derive(Clone)]
struct Slot<T> {
    gen: u32,
    value: Option<T>,
}

#[derive(Clone, Copy)]
pub struct Handle {
    pub slot: usize,
    pub gen: u32,
}

pub struct SegTable<T> {
    slots: Vec<Slot<T>>,
    cap: usize,
    free_list: Vec<usize>,
    len: usize,
    total_inserts: u64,
    total_removes: u64,
    total_compacts: u64,
}

impl<T> SegTable<T> {
    pub fn new(cap: usize) -> Self {
        let free_list: Vec<usize> = (0..cap).rev().collect();
        Self { slots: (0..cap).map(|_| Slot { gen: 0, value: None }).collect(), cap, free_list, len: 0, total_inserts: 0, total_removes: 0, total_compacts: 0 }
    }

    pub fn insert(&mut self, value: T) -> Option<Handle> {
        self.total_inserts += 1;
        let slot = self.free_list.pop()?;
        self.slots[slot].value = Some(value);
        self.len += 1;
        Some(Handle { slot, gen: self.slots[slot].gen })
    }

    pub fn get(&self, handle: Handle) -> Result<&T, StErr> {
        let s = &self.slots[handle.slot];
        if s.gen != handle.gen { return Err(StErr::InvalidHandle { gen: handle.gen, expected: s.gen }); }
        s.value.as_ref().ok_or(StErr::SlotEmpty { slot: handle.slot })
    }

    pub fn get_mut(&mut self, handle: Handle) -> Result<&mut T, StErr> {
        let gen = self.slots[handle.slot].gen;
        if gen != handle.gen { return Err(StErr::InvalidHandle { gen: handle.gen, expected: gen }); }
        self.slots[handle.slot].value.as_mut().ok_or(StErr::SlotEmpty { slot: handle.slot })
    }

    pub fn remove(&mut self, handle: Handle) -> Result<T, StErr> {
        let s = &mut self.slots[handle.slot];
        if s.gen != handle.gen { return Err(StErr::InvalidHandle { gen: handle.gen, expected: s.gen }); }
        let val = s.value.take().ok_or(StErr::SlotEmpty { slot: handle.slot })?;
        s.gen += 1;
        self.free_list.push(handle.slot);
        self.len -= 1;
        self.total_removes += 1;
        Ok(val)
    }

    pub fn compact(&mut self) -> usize {
        self.total_compacts += 1;
        let mut moved = 0usize;
        let mut values: Vec<Option<T>> = Vec::new();
        let mut gens: Vec<u32> = Vec::new();
        for i in 0..self.cap {
            if self.slots[i].value.is_some() {
                values.push(self.slots[i].value.take());
                gens.push(self.slots[i].gen);
            }
        }
        moved = values.len();
        for i in 0..self.cap {
            if i < moved {
                self.slots[i].value = values[i].take();
                self.slots[i].gen = gens[i];
            } else {
                self.slots[i].value = None;
                self.slots[i].gen = 0;
            }
        }
        let mut new_free: Vec<usize> = (moved..self.cap).rev().collect();
        self.free_list = new_free;
        moved
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn cap(&self) -> usize { self.cap }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_compacts(&self) -> u64 { self.total_compacts }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut st: SegTable<u64> = SegTable::new(8);
        let h = st.insert(42).unwrap();
        assert_eq!(*st.get(h).unwrap(), 42);
    }

    #[test]
    fn insert_remove() {
        let mut st: SegTable<u64> = SegTable::new(8);
        let h = st.insert(10).unwrap();
        let v = st.remove(h).unwrap();
        assert_eq!(v, 10);
        assert!(st.is_empty());
    }

    #[test]
    fn stale_handle() {
        let mut st: SegTable<u64> = SegTable::new(8);
        let h = st.insert(1).unwrap();
        st.remove(h).unwrap();
        let h2 = st.insert(2).unwrap();
        assert!(st.get(h).is_err());
        assert_eq!(*st.get(h2).unwrap(), 2);
    }

    #[test]
    fn full() {
        let mut st: SegTable<u64> = SegTable::new(2);
        st.insert(1).unwrap();
        st.insert(2).unwrap();
        assert!(st.insert(3).is_none());
    }

    #[test]
    fn get_mut() {
        let mut st: SegTable<u64> = SegTable::new(8);
        let h = st.insert(1).unwrap();
        *st.get_mut(h).unwrap() = 99;
        assert_eq!(*st.get(h).unwrap(), 99);
    }

    #[test]
    fn compact() {
        let mut st: SegTable<u64> = SegTable::new(8);
        let h1 = st.insert(1).unwrap();
        let _h2 = st.insert(2).unwrap();
        st.remove(h1).unwrap();
        let moved = st.compact();
        assert_eq!(moved, 1);
        assert_eq!(st.len(), 1);
    }

    #[test]
    fn len() {
        let mut st: SegTable<u64> = SegTable::new(8);
        st.insert(1).unwrap(); st.insert(2).unwrap();
        assert_eq!(st.len(), 2);
    }

    #[test]
    fn stats() {
        let mut st: SegTable<u64> = SegTable::new(8);
        let h = st.insert(1).unwrap();
        st.remove(h).unwrap();
        st.compact();
        assert_eq!(st.total_inserts(), 1);
        assert_eq!(st.total_removes(), 1);
        assert_eq!(st.total_compacts(), 1);
    }

    #[test]
    fn error_display() { assert!(StErr::Full { cap: 8 }.to_string().contains("full")); }
}

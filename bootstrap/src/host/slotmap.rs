use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlotKey { pub index: u32, pub generation: u32 }

#[derive(Debug, Clone, PartialEq)]
pub enum SlotError {
    InvalidKey { index: u32, expected_gen: u32, found_gen: u32 },
    SlotEmpty { index: u32 },
    Full { capacity: usize },
}

impl std::fmt::Display for SlotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlotError::InvalidKey { index, expected_gen, found_gen } => write!(f, "slot {index}: gen mismatch expected {expected_gen} found {found_gen}"),
            SlotError::SlotEmpty { index } => write!(f, "slot {index} empty"),
            SlotError::Full { capacity } => write!(f, "slot map full ({capacity})"),
        }
    }
}

impl std::error::Error for SlotError {}

struct Slot<T> {
    value: Option<T>,
    generation: u32,
}

pub struct SlotMap<T> {
    slots: Vec<Slot<T>>,
    free_list: Vec<u32>,
    capacity: usize,
    len: usize,
    total_inserts: u64,
    total_removes: u64,
}

impl<T> SlotMap<T> {
    pub fn new(capacity: usize) -> Self {
        let slots = (0..capacity).map(|_| Slot { value: None, generation: 0 }).collect();
        let free_list = (0..capacity as u32).rev().collect();
        Self { slots, free_list, capacity, len: 0, total_inserts: 0, total_removes: 0 }
    }

    pub fn insert(&mut self, value: T) -> Result<SlotKey, SlotError> {
        let idx = self.free_list.pop().ok_or(SlotError::Full { capacity: self.capacity })?;
        let slot = &mut self.slots[idx as usize];
        slot.value = Some(value);
        self.len += 1;
        self.total_inserts += 1;
        Ok(SlotKey { index: idx, generation: slot.generation })
    }

    pub fn remove(&mut self, key: SlotKey) -> Result<T, SlotError> {
        let slot = self.slots.get_mut(key.index as usize).ok_or(SlotError::SlotEmpty { index: key.index })?;
        if slot.generation != key.generation { return Err(SlotError::InvalidKey { index: key.index, expected_gen: slot.generation, found_gen: key.generation }); }
        let val = slot.value.take().ok_or(SlotError::SlotEmpty { index: key.index })?;
        slot.generation = slot.generation.wrapping_add(1);
        self.free_list.push(key.index);
        self.len -= 1;
        self.total_removes += 1;
        Ok(val)
    }

    pub fn get(&self, key: SlotKey) -> Option<&T> {
        let slot = self.slots.get(key.index as usize)?;
        if slot.generation != key.generation { return None; }
        slot.value.as_ref()
    }

    pub fn get_mut(&mut self, key: SlotKey) -> Option<&mut T> {
        let slot = self.slots.get_mut(key.index as usize)?;
        if slot.generation != key.generation { return None; }
        slot.value.as_mut()
    }

    pub fn contains(&self, key: SlotKey) -> bool {
        self.slots.get(key.index as usize)
            .map(|s| s.generation == key.generation && s.value.is_some())
            .unwrap_or(false)
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn generation(&self, index: u32) -> Option<u32> { self.slots.get(index as usize).map(|s| s.generation) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() { let sm: SlotMap<i32> = SlotMap::new(10); assert!(sm.is_empty()); }

    #[test]
    fn insert_get() {
        let mut sm: SlotMap<i32> = SlotMap::new(10);
        let k = sm.insert(42).unwrap();
        assert_eq!(sm.get(k), Some(&42));
    }

    #[test]
    fn remove() {
        let mut sm: SlotMap<i32> = SlotMap::new(10);
        let k = sm.insert(42).unwrap();
        let v = sm.remove(k).unwrap();
        assert_eq!(v, 42);
        assert!(sm.is_empty());
    }

    #[test]
    fn stale_key() {
        let mut sm: SlotMap<i32> = SlotMap::new(10);
        let k = sm.insert(1).unwrap();
        sm.remove(k).unwrap();
        let k2 = sm.insert(2).unwrap();
        assert_eq!(k.index, k2.index);
        assert_ne!(k.generation, k2.generation);
        assert_eq!(sm.get(k), None);
        assert_eq!(sm.get(k2), Some(&2));
    }

    #[test]
    fn full() {
        let mut sm: SlotMap<i32> = SlotMap::new(1);
        sm.insert(1).unwrap();
        let err = sm.insert(2).unwrap_err();
        assert!(matches!(err, SlotError::Full { .. }));
    }

    #[test]
    fn get_mut() {
        let mut sm: SlotMap<i32> = SlotMap::new(10);
        let k = sm.insert(1).unwrap();
        *sm.get_mut(k).unwrap() = 99;
        assert_eq!(sm.get(k), Some(&99));
    }

    #[test]
    fn contains() {
        let mut sm: SlotMap<i32> = SlotMap::new(10);
        let k = sm.insert(1).unwrap();
        assert!(sm.contains(k));
    }

    #[test]
    fn multiple_insert_remove() {
        let mut sm: SlotMap<String> = SlotMap::new(10);
        let k1 = sm.insert("a".to_string()).unwrap();
        let k2 = sm.insert("b".to_string()).unwrap();
        assert_eq!(sm.len(), 2);
        sm.remove(k1).unwrap();
        assert_eq!(sm.len(), 1);
        assert_eq!(sm.get(k2), Some(&"b".to_string()));
    }

    #[test]
    fn stats() {
        let mut sm: SlotMap<i32> = SlotMap::new(10);
        let k = sm.insert(1).unwrap();
        sm.remove(k).unwrap();
        assert_eq!(sm.total_inserts(), 1);
        assert_eq!(sm.total_removes(), 1);
    }

    #[test]
    fn generation_tracking() {
        let mut sm: SlotMap<i32> = SlotMap::new(10);
        let k = sm.insert(1).unwrap();
        let gen0 = sm.generation(k.index).unwrap();
        sm.remove(k).unwrap();
        let gen1 = sm.generation(k.index).unwrap();
        assert_eq!(gen1, gen0 + 1);
    }

    #[test]
    fn error_display() { assert!(SlotError::Full { capacity: 5 }.to_string().contains("5")); }
}

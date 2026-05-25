const BUCKETS: usize = 64;
const SLOTS: usize = 4;
const STASH_SIZE: usize = 4;

#[derive(Clone, Copy)]
struct Slot { key: u64, value: u64, occupied: bool }

pub struct Cuckoo3 {
    table: [[Slot; SLOTS]; BUCKETS],
    stash: [Slot; STASH_SIZE],
    len: usize,
    max_kicks: usize,
    total_inserts: u64,
    total_lookups: u64,
}

fn h1(key: u64) -> usize { ((key.wrapping_mul(0x9e3779b97f4a7c15) >> 32) as usize) % BUCKETS }
fn h2(key: u64) -> usize { ((key.wrapping_mul(0xff51afd7ed558ccd) >> 32) as usize) % BUCKETS }

impl Cuckoo3 {
    pub fn new() -> Self {
        Self {
            table: [[Slot { key: 0, value: 0, occupied: false }; SLOTS]; BUCKETS],
            stash: [Slot { key: 0, value: 0, occupied: false }; STASH_SIZE],
            len: 0, max_kicks: 128, total_inserts: 0, total_lookups: 0,
        }
    }

    pub fn insert(&mut self, key: u64, value: u64) -> bool {
        self.total_inserts += 1;
        let (b1, b2) = (h1(key), h2(key));
        for b in &[b1, b2] {
            for s in 0..SLOTS {
                if !self.table[*b][s].occupied { self.table[*b][s] = Slot { key, value, occupied: true }; self.len += 1; return true; }
            }
        }
        let mut ck = key; let mut cv = value;
        let mut bi = b1;
        for _ in 0..self.max_kicks {
            let si = (ck as usize) % SLOTS;
            let evicted = self.table[bi][si];
            self.table[bi][si] = Slot { key: ck, value: cv, occupied: true };
            ck = evicted.key; cv = evicted.value;
            if !evicted.occupied { self.len += 1; return true; }
            bi = if bi == h1(ck) { h2(ck) } else { h1(ck) };
        }
        for s in 0..STASH_SIZE {
            if !self.stash[s].occupied { self.stash[s] = Slot { key: ck, value: cv, occupied: true }; self.len += 1; return true; }
        }
        false
    }

    pub fn get(&mut self, key: u64) -> Option<u64> {
        self.total_lookups += 1;
        let (b1, b2) = (h1(key), h2(key));
        for b in &[b1, b2] { for s in 0..SLOTS { if self.table[*b][s].occupied && self.table[*b][s].key == key { return Some(self.table[*b][s].value); } } }
        for s in 0..STASH_SIZE { if self.stash[s].occupied && self.stash[s].key == key { return Some(self.stash[s].value); } }
        None
    }

    pub fn remove(&mut self, key: u64) -> bool {
        let (b1, b2) = (h1(key), h2(key));
        for b in &[b1, b2] { for s in 0..SLOTS { if self.table[*b][s].occupied && self.table[*b][s].key == key { self.table[*b][s].occupied = false; self.len -= 1; return true; } } }
        for s in 0..STASH_SIZE { if self.stash[s].occupied && self.stash[s].key == key { self.stash[s].occupied = false; self.len -= 1; return true; } }
        false
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut c = Cuckoo3::new();
        assert!(c.insert(42, 100));
        assert_eq!(c.get(42), Some(100));
    }

    #[test]
    fn missing() { let mut c = Cuckoo3::new(); assert_eq!(c.get(1), None); }

    #[test]
    fn many() {
        let mut c = Cuckoo3::new();
        for i in 0..200u64 { assert!(c.insert(i, i * 10)); }
        for i in 0..200u64 { assert_eq!(c.get(i), Some(i * 10)); }
        assert_eq!(c.len(), 200);
    }

    #[test]
    fn remove() {
        let mut c = Cuckoo3::new();
        c.insert(1, 10); c.insert(2, 20);
        assert!(c.remove(1));
        assert_eq!(c.get(1), None);
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn remove_missing() { assert!(!Cuckoo3::new().remove(1)); }

    #[test]
    fn stats() {
        let mut c = Cuckoo3::new();
        c.insert(1, 1); c.get(1);
        assert_eq!(c.total_inserts(), 1);
        assert_eq!(c.total_lookups(), 1);
    }
}

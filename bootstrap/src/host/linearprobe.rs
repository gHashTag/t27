#[derive(Debug, Clone, PartialEq)]
pub enum LpError {
    NotFound { key: u64 },
    Full,
}

impl std::fmt::Display for LpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LpError::NotFound { key } => write!(f, "key {key} not found"),
            LpError::Full => write!(f, "table full"),
        }
    }
}

impl std::error::Error for LpError {}

#[derive(Clone)]
enum Slot {
    Empty,
    Tombstone,
    Occupied { key: u64, value: Vec<u8> },
}

pub struct LinearProbe {
    slots: Vec<Slot>,
    cap: usize,
    len: usize,
    tombs: usize,
    total_inserts: u64,
    total_lookups: u64,
    total_probes: u64,
}

impl LinearProbe {
    pub fn new(cap: usize) -> Self {
        Self { slots: vec![Slot::Empty; cap], cap, len: 0, tombs: 0, total_inserts: 0, total_lookups: 0, total_probes: 0 }
    }

    fn hash(&self, key: u64) -> usize {
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in key.to_le_bytes() { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
        (h as usize) % self.cap
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) -> Result<(), LpError> {
        self.total_inserts += 1;
        let start = self.hash(key);
        let mut first_tomb: Option<usize> = None;
        for i in 0..self.cap {
            let idx = (start + i) % self.cap;
            self.total_probes += 1;
            match &self.slots[idx] {
                Slot::Empty => {
                    let insert_at = first_tomb.unwrap_or(idx);
                    self.slots[insert_at] = Slot::Occupied { key, value };
                    self.len += 1;
                    if first_tomb.is_some() { self.tombs -= 1; }
                    return Ok(());
                }
                Slot::Tombstone if first_tomb.is_none() => { first_tomb = Some(idx); }
                Slot::Occupied { key: k, .. } if *k == key => {
                    self.slots[idx] = Slot::Occupied { key, value };
                    return Ok(());
                }
                _ => {}
            }
        }
        if let Some(t) = first_tomb {
            self.slots[t] = Slot::Occupied { key, value };
            self.len += 1;
            self.tombs -= 1;
            return Ok(());
        }
        Err(LpError::Full)
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        let start = self.hash(key);
        for i in 0..self.cap {
            let idx = (start + i) % self.cap;
            self.total_probes += 1;
            match &self.slots[idx] {
                Slot::Empty => return None,
                Slot::Occupied { key: k, value } if *k == key => return Some(value),
                _ => {}
            }
        }
        None
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, LpError> {
        let start = self.hash(key);
        for i in 0..self.cap {
            let idx = (start + i) % self.cap;
            match &self.slots[idx] {
                Slot::Empty => return Err(LpError::NotFound { key }),
                Slot::Occupied { key: k, .. } if *k == key => {
                    let old = std::mem::replace(&mut self.slots[idx], Slot::Tombstone);
                    self.len -= 1;
                    self.tombs += 1;
                    return match old { Slot::Occupied { value, .. } => Ok(value), _ => unreachable!() };
                }
                _ => {}
            }
        }
        Err(LpError::NotFound { key })
    }

    pub fn contains(&mut self, key: u64) -> bool { self.get(key).is_some() }
    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn load_factor(&self) -> f64 { (self.len + self.tombs) as f64 / self.cap as f64 }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_probes(&self) -> u64 { self.total_probes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_lp() { let lp = LinearProbe::new(16); assert!(lp.is_empty()); }

    #[test]
    fn insert_get() {
        let mut lp = LinearProbe::new(16);
        lp.insert(1, b"one".to_vec()).unwrap(); lp.insert(2, b"two".to_vec()).unwrap();
        assert_eq!(lp.get(1), Some(&b"one"[..]));
        assert_eq!(lp.get(2), Some(&b"two"[..]));
    }

    #[test]
    fn overwrite() {
        let mut lp = LinearProbe::new(16);
        lp.insert(1, b"old".to_vec()).unwrap(); lp.insert(1, b"new".to_vec()).unwrap();
        assert_eq!(lp.get(1), Some(&b"new"[..]));
        assert_eq!(lp.len(), 1);
    }

    #[test]
    fn remove() {
        let mut lp = LinearProbe::new(16);
        lp.insert(1, b"a".to_vec()).unwrap();
        lp.remove(1).unwrap();
        assert!(!lp.contains(1));
        lp.insert(1, b"b".to_vec()).unwrap();
        assert_eq!(lp.get(1), Some(&b"b"[..]));
    }

    #[test]
    fn remove_not_found() { assert!(LinearProbe::new(16).remove(1).is_err()); }

    #[test]
    fn many() {
        let mut lp = LinearProbe::new(64);
        for i in 0..50u64 { lp.insert(i, vec![i as u8]).unwrap(); }
        assert_eq!(lp.len(), 50);
        for i in 0..50u64 { assert!(lp.contains(i)); }
    }

    #[test]
    fn full() {
        let mut lp = LinearProbe::new(4);
        for i in 0..4u64 { lp.insert(i, vec![]).unwrap(); }
        assert!(lp.insert(99, vec![]).is_err());
    }

    #[test]
    fn load_factor() {
        let mut lp = LinearProbe::new(10);
        for i in 0..5 { lp.insert(i, vec![]).unwrap(); }
        assert!(lp.load_factor() >= 0.5);
    }

    #[test]
    fn probes_tracked() {
        let mut lp = LinearProbe::new(16);
        lp.insert(1, vec![]).unwrap(); lp.get(1);
        assert!(lp.total_probes() > 0);
    }

    #[test]
    fn stats() {
        let mut lp = LinearProbe::new(16);
        lp.insert(1, vec![]).unwrap();
        assert_eq!(lp.total_inserts(), 1);
    }

    #[test]
    fn error_display() { assert!(LpError::Full.to_string().contains("full")); }
}

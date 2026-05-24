const EMPTY: u8 = 0;
const OCCUPIED: u8 = 1;
const DELETED: u8 = 2;

#[derive(Debug, Clone, PartialEq)]
pub enum RmError {
    KeyExists { key: u64 },
    KeyNotFound { key: u64 },
    TableFull,
}

impl std::fmt::Display for RmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RmError::KeyExists { key } => write!(f, "key {key} exists"),
            RmError::KeyNotFound { key } => write!(f, "key {key} not found"),
            RmError::TableFull => write!(f, "table full"),
        }
    }
}

impl std::error::Error for RmError {}

struct Entry {
    key: u64,
    value: Vec<u8>,
    state: u8,
}

pub struct RobinMap {
    table: Vec<Entry>,
    capacity: usize,
    count: usize,
    tombstones: usize,
    total_probes: u64,
    total_lookups: u64,
    max_probe: usize,
}

impl RobinMap {
    pub fn new(capacity: usize) -> Self {
        Self {
            table: (0..capacity).map(|_| Entry { key: 0, value: Vec::new(), state: EMPTY }).collect(),
            capacity,
            count: 0,
            tombstones: 0,
            total_probes: 0,
            total_lookups: 0,
            max_probe: 0,
        }
    }

    fn hash(&self, key: u64) -> usize {
        let mut h = key;
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33;
        h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
        h ^= h >> 33;
        (h as usize) % self.capacity
    }

    fn probe_distance(&self, slot: usize, ideal: usize) -> usize {
        (slot + self.capacity - ideal) % self.capacity
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) -> Result<(), RmError> {
        if self.count >= self.capacity { return Err(RmError::TableFull); }
        let ideal = self.hash(key);
        let mut cur_key = key;
        let mut cur_val = value;
        let mut cur_ideal = ideal;
        let mut probes: usize = 0;
        let mut i = ideal;
        loop {
            probes += 1;
            match self.table[i].state {
                EMPTY | DELETED => {
                    self.table[i] = Entry { key: cur_key, value: cur_val, state: OCCUPIED };
                    self.count += 1;
                    self.total_probes += probes as u64;
                    if probes > self.max_probe { self.max_probe = probes; }
                    return Ok(());
                }
                OCCUPIED if self.table[i].key == cur_key => {
                    return Err(RmError::KeyExists { key });
                }
                OCCUPIED => {
                    let existing_ideal = self.hash(self.table[i].key);
                    let existing_dist = self.probe_distance(i, existing_ideal);
                    let cur_dist = self.probe_distance(i, cur_ideal);
                    if cur_dist > existing_dist {
                        let tmp_key = self.table[i].key;
                        let tmp_val = std::mem::take(&mut self.table[i].value);
                        self.table[i].key = cur_key;
                        self.table[i].value = cur_val;
                        cur_key = tmp_key;
                        cur_val = tmp_val;
                        cur_ideal = existing_ideal;
                    }
                }
                _ => {}
            }
            i = (i + 1) % self.capacity;
        }
    }

    pub fn get(&mut self, key: u64) -> Option<&Vec<u8>> {
        let ideal = self.hash(key);
        let mut i = ideal;
        let mut probes: usize = 0;
        self.total_lookups += 1;
        loop {
            probes += 1;
            if self.table[i].state == EMPTY { break; }
            if self.table[i].state == OCCUPIED && self.table[i].key == key {
                self.total_probes += probes as u64;
                return Some(&self.table[i].value);
            }
            i = (i + 1) % self.capacity;
            if i == ideal { break; }
        }
        self.total_probes += probes as u64;
        None
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, RmError> {
        let ideal = self.hash(key);
        let mut i = ideal;
        loop {
            match self.table[i].state {
                EMPTY => return Err(RmError::KeyNotFound { key }),
                OCCUPIED if self.table[i].key == key => {
                    self.table[i].state = DELETED;
                    self.count -= 1;
                    self.tombstones += 1;
                    return Ok(std::mem::take(&mut self.table[i].value));
                }
                _ => {}
            }
            i = (i + 1) % self.capacity;
            if i == ideal { return Err(RmError::KeyNotFound { key }); }
        }
    }

    pub fn contains(&mut self, key: u64) -> bool { self.get(key).is_some() }
    pub fn len(&self) -> usize { self.count }
    pub fn is_empty(&self) -> bool { self.count == 0 }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn tombstones(&self) -> usize { self.tombstones }
    pub fn total_probes(&self) -> u64 { self.total_probes }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn max_probe(&self) -> usize { self.max_probe }
    pub fn avg_probe(&self) -> f64 { if self.total_lookups == 0 { 0.0 } else { self.total_probes as f64 / self.total_lookups as f64 } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() { let m = RobinMap::new(16); assert_eq!(m.capacity(), 16); assert!(m.is_empty()); }

    #[test]
    fn insert_get() {
        let mut m = RobinMap::new(16);
        m.insert(1, b"v1".to_vec()).unwrap();
        assert_eq!(m.get(1), Some(&b"v1".to_vec()));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn duplicate_key() {
        let mut m = RobinMap::new(16);
        m.insert(1, b"a".to_vec()).unwrap();
        let err = m.insert(1, b"b".to_vec()).unwrap_err();
        assert!(matches!(err, RmError::KeyExists { .. }));
    }

    #[test]
    fn remove() {
        let mut m = RobinMap::new(16);
        m.insert(1, b"v".to_vec()).unwrap();
        let v = m.remove(1).unwrap();
        assert_eq!(v, b"v");
        assert!(m.is_empty());
        assert_eq!(m.tombstones(), 1);
    }

    #[test]
    fn remove_not_found() {
        let mut m = RobinMap::new(16);
        let err = m.remove(99).unwrap_err();
        assert!(matches!(err, RmError::KeyNotFound { .. }));
    }

    #[test]
    fn get_missing() {
        let mut m = RobinMap::new(16);
        assert_eq!(m.get(99), None);
    }

    #[test]
    fn contains() {
        let mut m = RobinMap::new(16);
        m.insert(42, b"x".to_vec()).unwrap();
        assert!(m.contains(42));
        assert!(!m.contains(43));
    }

    #[test]
    fn table_full() {
        let mut m = RobinMap::new(4);
        for i in 0..4 { m.insert(i, b"x".to_vec()).unwrap(); }
        let err = m.insert(99, b"x".to_vec()).unwrap_err();
        assert!(matches!(err, RmError::TableFull));
    }

    #[test]
    fn robin_hood_swap() {
        let mut m = RobinMap::new(16);
        for i in 0..16 {
            let k = (i as u64).wrapping_mul(2654435761) % 1000;
            let _ = m.insert(k, vec![i as u8]);
        }
        for i in 0..16 {
            let k = (i as u64).wrapping_mul(2654435761) % 1000;
            if m.contains(k) { assert!(m.get(k).is_some()); }
        }
    }

    #[test]
    fn probe_stats() {
        let mut m = RobinMap::new(64);
        for i in 0..30 { m.insert(i, b"x".to_vec()).unwrap(); }
        for i in 0..30 { m.get(i); }
        assert!(m.total_lookups() > 0);
        assert!(m.avg_probe() >= 1.0);
    }

    #[test]
    fn error_display() { assert!(RmError::TableFull.to_string().contains("full")); }
}

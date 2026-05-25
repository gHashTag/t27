use std::collections::BTreeMap;

pub struct SkipMap {
    data: BTreeMap<u64, Vec<u8>>,
    max_level: usize,
    total_inserts: u64,
    total_lookups: u64,
}

impl SkipMap {
    pub fn new(max_level: usize) -> Self { Self { data: BTreeMap::new(), max_level: max_level.max(1), total_inserts: 0, total_lookups: 0 } }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        self.data.insert(key, value);
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        self.data.get(&key).map(|v| v.as_slice())
    }

    pub fn remove(&mut self, key: u64) -> Option<Vec<u8>> { self.data.remove(&key) }

    pub fn range(&self, lo: u64, hi: u64) -> Vec<(u64, &[u8])> {
        self.data.range(lo..=hi).map(|(k, v)| (*k, v.as_slice())).collect()
    }

    pub fn first(&self) -> Option<(u64, &[u8])> { self.data.iter().next().map(|(k, v)| (*k, v.as_slice())) }
    pub fn last(&self) -> Option<(u64, &[u8])> { self.data.iter().next_back().map(|(k, v)| (*k, v.as_slice())) }

    pub fn successor(&self, key: u64) -> Option<(u64, &[u8])> {
        self.data.range((key + 1)..).next().map(|(k, v)| (*k, v.as_slice()))
    }

    pub fn predecessor(&self, key: u64) -> Option<(u64, &[u8])> {
        self.data.range(..key).next_back().map(|(k, v)| (*k, v.as_slice()))
    }

    pub fn contains(&mut self, key: u64) -> bool { self.total_lookups += 1; self.data.contains_key(&key) }
    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn max_level(&self) -> usize { self.max_level }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut sm = SkipMap::new(8);
        sm.insert(3, b"three".to_vec());
        assert_eq!(sm.get(3), Some(&b"three"[..]));
    }

    #[test]
    fn missing() { let mut sm = SkipMap::new(4); assert!(sm.get(1).is_none()); }

    #[test]
    fn range() {
        let mut sm = SkipMap::new(8);
        for i in 0..10u64 { sm.insert(i, vec![i as u8]); }
        let r = sm.range(3, 7);
        assert_eq!(r.len(), 5);
        assert_eq!(r[0].0, 3);
    }

    #[test]
    fn first_last() {
        let mut sm = SkipMap::new(4);
        sm.insert(5, vec![]); sm.insert(2, vec![]); sm.insert(8, vec![]);
        assert_eq!(sm.first().unwrap().0, 2);
        assert_eq!(sm.last().unwrap().0, 8);
    }

    #[test]
    fn successor_predecessor() {
        let mut sm = SkipMap::new(4);
        for i in (0..10u64).step_by(2) { sm.insert(i, vec![]); }
        assert_eq!(sm.successor(3).unwrap().0, 4);
        assert_eq!(sm.predecessor(5).unwrap().0, 4);
        assert!(sm.successor(9).is_none());
    }

    #[test]
    fn remove() {
        let mut sm = SkipMap::new(4);
        sm.insert(1, b"one".to_vec());
        assert_eq!(sm.remove(1), Some(b"one".to_vec()));
        assert!(sm.is_empty());
    }

    #[test]
    fn stats() {
        let mut sm = SkipMap::new(4);
        sm.insert(1, vec![]); sm.get(1);
        assert_eq!(sm.total_inserts(), 1);
        assert_eq!(sm.total_lookups(), 1);
    }
}

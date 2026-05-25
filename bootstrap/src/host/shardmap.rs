use std::collections::BTreeMap;

pub struct ShardMap {
    shards: Vec<BTreeMap<u64, Vec<u8>>>,
    num_shards: usize,
    total_inserts: u64,
    total_lookups: u64,
}

impl ShardMap {
    pub fn new(num_shards: usize) -> Self {
        let num_shards = num_shards.max(1);
        Self { shards: (0..num_shards).map(|_| BTreeMap::new()).collect(), num_shards, total_inserts: 0, total_lookups: 0 }
    }

    fn shard(&self, key: u64) -> usize {
        let h = key.wrapping_mul(0x9e3779b97f4a7c15);
        ((h >> 32) as usize) % self.num_shards
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        let s = self.shard(key);
        self.shards[s].insert(key, value);
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        let s = self.shard(key);
        self.shards[s].get(&key).map(|v| v.as_slice())
    }

    pub fn remove(&mut self, key: u64) -> Option<Vec<u8>> { let s = self.shard(key); self.shards[s].remove(&key) }

    pub fn contains(&mut self, key: u64) -> bool { self.total_lookups += 1; let s = self.shard(key); self.shards[s].contains_key(&key) }

    pub fn len(&self) -> usize { self.shards.iter().map(|s| s.len()).sum() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn num_shards(&self) -> usize { self.num_shards }

    pub fn shard_len(&self, idx: usize) -> usize { self.shards.get(idx).map(|s| s.len()).unwrap_or(0) }

    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut sm = ShardMap::new(4);
        sm.insert(1, b"one".to_vec());
        assert_eq!(sm.get(1), Some(&b"one"[..]));
    }

    #[test]
    fn missing() { let mut sm = ShardMap::new(4); assert!(sm.get(99).is_none()); }

    #[test]
    fn distribution() {
        let mut sm = ShardMap::new(4);
        for i in 0..1000u64 { sm.insert(i, vec![]); }
        for s in 0..4 { assert!(sm.shard_len(s) > 100, "shard {s} too small"); }
    }

    #[test]
    fn remove() {
        let mut sm = ShardMap::new(4);
        sm.insert(1, b"v".to_vec());
        assert_eq!(sm.remove(1), Some(b"v".to_vec()));
        assert!(sm.is_empty());
    }

    #[test]
    fn overwrite() {
        let mut sm = ShardMap::new(4);
        sm.insert(1, b"old".to_vec()); sm.insert(1, b"new".to_vec());
        assert_eq!(sm.get(1), Some(&b"new"[..]));
        assert_eq!(sm.len(), 1);
    }

    #[test]
    fn stats() {
        let mut sm = ShardMap::new(4);
        sm.insert(1, vec![]); sm.get(1);
        assert_eq!(sm.total_inserts(), 1);
        assert_eq!(sm.total_lookups(), 1);
    }
}

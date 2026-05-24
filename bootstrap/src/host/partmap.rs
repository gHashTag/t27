use std::collections::BTreeMap;
use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Clone, PartialEq)]
pub enum PartMapError {
    ShardNotFound { shard: u32 },
}

impl std::fmt::Display for PartMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartMapError::ShardNotFound { shard } => write!(f, "shard {shard} not found"),
        }
    }
}

impl std::error::Error for PartMapError {}

fn hash_key(key: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    h.write(key);
    h.finish()
}

struct Shard {
    id: u32,
    data: BTreeMap<Vec<u8>, Vec<u8>>,
}

pub struct PartitionedMap {
    shards: Vec<Shard>,
    shard_count: u32,
    total_inserts: u64,
    total_lookups: u64,
    total_deletes: u64,
    total_rebalances: u64,
}

impl PartitionedMap {
    pub fn new(shard_count: u32) -> Self {
        let shards: Vec<Shard> = (0..shard_count).map(|id| Shard { id, data: BTreeMap::new() }).collect();
        Self { shards, shard_count, total_inserts: 0, total_lookups: 0, total_deletes: 0, total_rebalances: 0 }
    }

    fn shard_for(&self, key: &[u8]) -> u32 {
        (hash_key(key) % self.shard_count as u64) as u32
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) -> u32 {
        let sid = self.shard_for(&key);
        self.shards[sid as usize].data.insert(key, value);
        self.total_inserts += 1;
        sid
    }

    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.total_lookups += 1;
        let sid = self.shard_for(key);
        self.shards[sid as usize].data.get(key).cloned()
    }

    pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.total_deletes += 1;
        let sid = self.shard_for(key);
        self.shards[sid as usize].data.remove(key)
    }

    pub fn contains(&mut self, key: &[u8]) -> bool {
        self.total_lookups += 1;
        let sid = self.shard_for(key);
        self.shards[sid as usize].data.contains_key(key)
    }

    pub fn rebalance(&mut self, new_shard_count: u32) -> u64 {
        let mut all_items: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        for shard in &self.shards {
            for (k, v) in &shard.data { all_items.push((k.clone(), v.clone())); }
        }
        self.shards = (0..new_shard_count).map(|id| Shard { id, data: BTreeMap::new() }).collect();
        self.shard_count = new_shard_count;
        let mut moved = 0u64;
        for (k, v) in all_items {
            let sid = self.shard_for(&k);
            self.shards[sid as usize].data.insert(k, v);
            moved += 1;
        }
        self.total_rebalances += 1;
        moved
    }

    pub fn shard_size(&self, shard: u32) -> Option<usize> { self.shards.get(shard as usize).map(|s| s.data.len()) }
    pub fn total_items(&self) -> usize { self.shards.iter().map(|s| s.data.len()).sum() }
    pub fn shard_count(&self) -> u32 { self.shard_count }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn total_rebalances(&self) -> u64 { self.total_rebalances }
    pub fn distribution(&self) -> Vec<(u32, usize)> { self.shards.iter().map(|s| (s.id, s.data.len())).collect() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() { let pm = PartitionedMap::new(4); assert_eq!(pm.shard_count(), 4); }

    #[test]
    fn insert_get() {
        let mut pm = PartitionedMap::new(4);
        pm.insert(b"key".to_vec(), b"val".to_vec());
        assert_eq!(pm.get(b"key"), Some(b"val".to_vec()));
    }

    #[test]
    fn remove() {
        let mut pm = PartitionedMap::new(4);
        pm.insert(b"key".to_vec(), b"val".to_vec());
        let v = pm.remove(b"key");
        assert_eq!(v, Some(b"val".to_vec()));
        assert!(pm.get(b"key").is_none());
    }

    #[test]
    fn contains() {
        let mut pm = PartitionedMap::new(4);
        pm.insert(b"k".to_vec(), b"v".to_vec());
        assert!(pm.contains(b"k"));
        assert!(!pm.contains(b"x"));
    }

    #[test]
    fn shard_distribution() {
        let mut pm = PartitionedMap::new(4);
        for i in 0..100u8 { pm.insert(vec![i], vec![i]); }
        let dist = pm.distribution();
        assert_eq!(dist.iter().map(|&(_, s)| s).sum::<usize>(), 100);
    }

    #[test]
    fn rebalance() {
        let mut pm = PartitionedMap::new(2);
        for i in 0..50u8 { pm.insert(vec![i], vec![i]); }
        let moved = pm.rebalance(8);
        assert_eq!(moved, 50);
        assert_eq!(pm.shard_count(), 8);
        assert_eq!(pm.total_items(), 50);
    }

    #[test]
    fn data_preserved_after_rebalance() {
        let mut pm = PartitionedMap::new(2);
        pm.insert(b"key".to_vec(), b"val".to_vec());
        pm.rebalance(8);
        assert_eq!(pm.get(b"key"), Some(b"val".to_vec()));
    }

    #[test]
    fn shard_size() {
        let mut pm = PartitionedMap::new(4);
        let sid = pm.insert(b"k".to_vec(), b"v".to_vec());
        assert_eq!(pm.shard_size(sid), Some(1));
    }

    #[test]
    fn total_items() {
        let mut pm = PartitionedMap::new(4);
        pm.insert(b"a".to_vec(), b"1".to_vec());
        pm.insert(b"b".to_vec(), b"2".to_vec());
        assert_eq!(pm.total_items(), 2);
    }

    #[test]
    fn stats() {
        let mut pm = PartitionedMap::new(4);
        pm.insert(b"k".to_vec(), b"v".to_vec());
        pm.get(b"k");
        assert_eq!(pm.total_inserts(), 1);
        assert_eq!(pm.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(PartMapError::ShardNotFound { shard: 3 }.to_string().contains("3")); }
}

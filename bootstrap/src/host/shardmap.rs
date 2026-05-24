use std::collections::BTreeMap;

fn fnv_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

#[derive(Debug, Clone, PartialEq)]
pub enum SmError {
    ShardNotFound { shard: u64 },
    KeyNotFound { key: u64 },
    NoShards,
}

impl std::fmt::Display for SmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmError::ShardNotFound { shard } => write!(f, "shard {shard} not found"),
            SmError::KeyNotFound { key } => write!(f, "key {key} not found"),
            SmError::NoShards => write!(f, "no shards"),
        }
    }
}

impl std::error::Error for SmError {}

struct Shard {
    id: u64,
    data: BTreeMap<u64, Vec<u8>>,
}

pub struct ShardMap {
    shards: BTreeMap<u64, Shard>,
    ring: BTreeMap<u64, u64>,
    virtual_nodes: usize,
    total_inserts: u64,
    total_lookups: u64,
    total_migrations: u64,
}

impl ShardMap {
    pub fn new(virtual_nodes: usize) -> Self {
        Self { shards: BTreeMap::new(), ring: BTreeMap::new(), virtual_nodes, total_inserts: 0, total_lookups: 0, total_migrations: 0 }
    }

    pub fn add_shard(&mut self, shard_id: u64) {
        let mut buf = Vec::new();
        for i in 0..self.virtual_nodes {
            buf.clear();
            buf.extend_from_slice(&shard_id.to_le_bytes());
            buf.extend_from_slice(&(i as u64).to_le_bytes());
            let token = fnv_hash(&buf);
            self.ring.insert(token, shard_id);
        }
        self.shards.insert(shard_id, Shard { id: shard_id, data: BTreeMap::new() });
    }

    pub fn remove_shard(&mut self, shard_id: u64) -> Result<u64, SmError> {
        if !self.shards.contains_key(&shard_id) { return Err(SmError::ShardNotFound { shard: shard_id }); }
        let mut buf = Vec::new();
        for i in 0..self.virtual_nodes {
            buf.clear();
            buf.extend_from_slice(&shard_id.to_le_bytes());
            buf.extend_from_slice(&(i as u64).to_le_bytes());
            let token = fnv_hash(&buf);
            self.ring.remove(&token);
        }
        let shard = self.shards.remove(&shard_id).unwrap();
        let migrated = shard.data.len() as u64;
        for (k, v) in shard.data {
            let new_shard = self.find_shard(k);
            self.shards.get_mut(&new_shard).unwrap().data.insert(k, v);
            self.total_migrations += 1;
        }
        Ok(migrated)
    }

    fn find_shard(&self, key: u64) -> u64 {
        let token = fnv_hash(&key.to_le_bytes());
        if let Some((&t, &s)) = self.ring.range(token..).next() {
            return s;
        }
        if let Some((&_t, &s)) = self.ring.iter().next() { return s; }
        0
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) -> Result<(), SmError> {
        if self.shards.is_empty() { return Err(SmError::NoShards); }
        let shard_id = self.find_shard(key);
        self.shards.get_mut(&shard_id).unwrap().data.insert(key, value);
        self.total_inserts += 1;
        Ok(())
    }

    pub fn get(&mut self, key: u64) -> Option<&Vec<u8>> {
        self.total_lookups += 1;
        let shard_id = self.find_shard(key);
        self.shards.get(&shard_id)?.data.get(&key)
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, SmError> {
        let shard_id = self.find_shard(key);
        let shard = self.shards.get_mut(&shard_id).ok_or(SmError::KeyNotFound { key })?;
        shard.data.remove(&key).ok_or(SmError::KeyNotFound { key })
    }

    pub fn shard_for(&self, key: u64) -> Option<u64> {
        if self.shards.is_empty() { return None; }
        Some(self.find_shard(key))
    }

    pub fn shard_size(&self, shard_id: u64) -> Option<usize> {
        self.shards.get(&shard_id).map(|s| s.data.len())
    }

    pub fn shard_count(&self) -> usize { self.shards.len() }
    pub fn ring_size(&self) -> usize { self.ring.len() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_migrations(&self) -> u64 { self.total_migrations }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() { let sm = ShardMap::new(4); assert_eq!(sm.shard_count(), 0); }

    #[test]
    fn add_shard_insert() {
        let mut sm = ShardMap::new(4);
        sm.add_shard(1);
        sm.insert(42, b"val".to_vec()).unwrap();
        assert_eq!(sm.get(42), Some(&b"val".to_vec()));
    }

    #[test]
    fn consistent_sharding() {
        let mut sm = ShardMap::new(8);
        sm.add_shard(1); sm.add_shard(2);
        let s1 = sm.shard_for(100).unwrap();
        let s2 = sm.shard_for(100).unwrap();
        assert_eq!(s1, s2);
    }

    #[test]
    fn remove_shard_migrates() {
        let mut sm = ShardMap::new(8);
        sm.add_shard(1); sm.add_shard(2);
        sm.insert(42, b"val".to_vec()).unwrap();
        let migrated = sm.remove_shard(sm.shard_for(42).unwrap()).unwrap();
        assert!(migrated > 0);
        assert_eq!(sm.get(42), Some(&b"val".to_vec()));
    }

    #[test]
    fn remove_shard_missing() {
        let mut sm = ShardMap::new(4);
        let err = sm.remove_shard(99).unwrap_err();
        assert!(matches!(err, SmError::ShardNotFound { .. }));
    }

    #[test]
    fn no_shards() {
        let mut sm = ShardMap::new(4);
        let err = sm.insert(1, b"x".to_vec()).unwrap_err();
        assert!(matches!(err, SmError::NoShards));
    }

    #[test]
    fn distribution() {
        let mut sm = ShardMap::new(16);
        sm.add_shard(1); sm.add_shard(2);
        for i in 0..1000 { sm.insert(i, b"x".to_vec()).unwrap(); }
        let s1 = sm.shard_size(1).unwrap();
        let s2 = sm.shard_size(2).unwrap();
        assert!(s1 > 200);
        assert!(s2 > 200);
    }

    #[test]
    fn remove_key() {
        let mut sm = ShardMap::new(4);
        sm.add_shard(1);
        sm.insert(1, b"x".to_vec()).unwrap();
        let v = sm.remove(1).unwrap();
        assert_eq!(v, b"x");
    }

    #[test]
    fn ring_size() {
        let mut sm = ShardMap::new(8);
        sm.add_shard(1);
        assert_eq!(sm.ring_size(), 8);
    }

    #[test]
    fn stats() {
        let mut sm = ShardMap::new(4);
        sm.add_shard(1);
        sm.insert(1, b"x".to_vec()).unwrap();
        sm.get(1);
        assert_eq!(sm.total_inserts(), 1);
        assert_eq!(sm.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(SmError::NoShards.to_string().contains("shards")); }
}

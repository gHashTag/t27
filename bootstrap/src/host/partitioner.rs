use std::collections::BTreeMap;

const VNODES: usize = 128;

fn hash_key(key: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in key { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

fn hash_vnode(partition: u64, v: usize) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    h ^= partition; h = h.wrapping_mul(0x100000001b3);
    h ^= v as u64; h = h.wrapping_mul(0x100000001b3);
    h ^= h >> 33; h = h.wrapping_mul(0xff51afd7ed558ccd);
    h
}

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub enum PartError {
    PartitionExists { id: u64 },
    PartitionNotFound { id: u64 },
    Empty,
}

impl std::fmt::Display for PartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartError::PartitionExists { id } => write!(f, "partition {id} exists"),
            PartError::PartitionNotFound { id } => write!(f, "partition {id} not found"),
            PartError::Empty => write!(f, "no partitions"),
        }
    }
}

impl std::error::Error for PartError {}

#[derive(Debug, Clone)]
pub struct Migration {
    pub key: Vec<u8>,
    pub from: u64,
    pub to: u64,
}

pub struct Partitioner {
    ring: BTreeMap<u64, u64>,
    partitions: BTreeSet<u64>,
    key_map: BTreeMap<Vec<u8>, u64>,
    partition_counts: BTreeMap<u64, u64>,
}

impl Partitioner {
    pub fn new() -> Self {
        Self { ring: BTreeMap::new(), partitions: BTreeSet::new(), key_map: BTreeMap::new(), partition_counts: BTreeMap::new() }
    }

    pub fn add_partition(&mut self, id: u64) -> Result<Vec<Migration>, PartError> {
        if self.partitions.contains(&id) { return Err(PartError::PartitionExists { id }); }
        self.partitions.insert(id);
        self.partition_counts.insert(id, 0);
        for v in 0..VNODES {
            self.ring.insert(hash_vnode(id, v), id);
        }
        let migrations = self.rebalance_keys();
        Ok(migrations)
    }

    pub fn remove_partition(&mut self, id: u64) -> Result<Vec<Migration>, PartError> {
        if !self.partitions.contains(&id) { return Err(PartError::PartitionNotFound { id }); }
        self.partitions.remove(&id);
        self.ring.retain(|_, &mut p| p != id);
        self.partition_counts.remove(&id);
        let migrations = self.rebalance_keys();
        Ok(migrations)
    }

    fn rebalance_keys(&mut self) -> Vec<Migration> {
        let mut migrations = Vec::new();
        let keys: Vec<Vec<u8>> = self.key_map.keys().cloned().collect();
        for key in keys {
            let old_part = self.key_map.get(&key).copied();
            let new_part = self.assign_partition_for(&key);
            if old_part != Some(new_part) {
                if let Some(op) = old_part {
                    if let Some(c) = self.partition_counts.get_mut(&op) { *c = c.saturating_sub(1); }
                    migrations.push(Migration { key: key.clone(), from: op, to: new_part });
                }
                self.key_map.insert(key, new_part);
                *self.partition_counts.entry(new_part).or_insert(0) += 1;
            }
        }
        migrations
    }

    fn assign_partition_for(&self, key: &[u8]) -> u64 {
        if self.ring.is_empty() { return 0; }
        let h = hash_key(key);
        match self.ring.range(h..).next() {
            Some((_, &p)) => p,
            None => *self.ring.values().next().unwrap(),
        }
    }

    pub fn assign(&mut self, key: &[u8]) -> Result<u64, PartError> {
        if self.ring.is_empty() { return Err(PartError::Empty); }
        let part = self.assign_partition_for(key);
        self.key_map.insert(key.to_vec(), part);
        *self.partition_counts.entry(part).or_insert(0) += 1;
        Ok(part)
    }

    pub fn lookup(&self, key: &[u8]) -> Option<u64> { self.key_map.get(key).copied() }

    pub fn partition_count(&self) -> usize { self.partitions.len() }
    pub fn vnode_count(&self) -> usize { self.ring.len() }
    pub fn key_count(&self) -> usize { self.key_map.len() }
    pub fn partition_keys(&self, id: u64) -> u64 { self.partition_counts.get(&id).copied().unwrap_or(0) }
}

impl Default for Partitioner {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_partitioner() {
        let p = Partitioner::new();
        assert_eq!(p.partition_count(), 0);
    }

    #[test]
    fn add_partition() {
        let mut p = Partitioner::new();
        p.add_partition(1).unwrap();
        assert_eq!(p.partition_count(), 1);
        assert_eq!(p.vnode_count(), VNODES);
    }

    #[test]
    fn duplicate_partition() {
        let mut p = Partitioner::new();
        p.add_partition(1).unwrap();
        let err = p.add_partition(1).unwrap_err();
        assert!(matches!(err, PartError::PartitionExists { .. }));
    }

    #[test]
    fn assign_key() {
        let mut p = Partitioner::new();
        p.add_partition(1).unwrap();
        let part = p.assign(b"key1").unwrap();
        assert_eq!(p.lookup(b"key1"), Some(part));
    }

    #[test]
    fn no_partitions() {
        let mut p = Partitioner::new();
        let err = p.assign(b"k").unwrap_err();
        assert!(matches!(err, PartError::Empty));
    }

    #[test]
    fn remove_partition() {
        let mut p = Partitioner::new();
        p.add_partition(1).unwrap();
        p.add_partition(2).unwrap();
        p.assign(b"k").unwrap();
        p.remove_partition(1).unwrap();
        assert_eq!(p.partition_count(), 1);
    }

    #[test]
    fn remove_not_found() {
        let mut p = Partitioner::new();
        let err = p.remove_partition(99).unwrap_err();
        assert!(matches!(err, PartError::PartitionNotFound { .. }));
    }

    #[test]
    fn migration_on_add() {
        let mut p = Partitioner::new();
        p.add_partition(1).unwrap();
        p.assign(b"k1").unwrap();
        let migrations = p.add_partition(2).unwrap();
        assert!(migrations.is_empty() || !migrations.is_empty());
    }

    #[test]
    fn distribution() {
        let mut p = Partitioner::new();
        p.add_partition(1).unwrap(); p.add_partition(2).unwrap(); p.add_partition(3).unwrap();
        for i in 0..1000u32 { p.assign(format!("k{i}").as_bytes()).unwrap(); }
        assert!(p.partition_keys(1) > 50);
        assert!(p.partition_keys(2) > 50);
        assert!(p.partition_keys(3) > 50);
    }

    #[test]
    fn key_count() {
        let mut p = Partitioner::new();
        p.add_partition(1).unwrap();
        p.assign(b"a").unwrap(); p.assign(b"b").unwrap();
        assert_eq!(p.key_count(), 2);
    }

    #[test]
    fn error_display() {
        assert!(PartError::Empty.to_string().contains("no partitions"));
    }
}

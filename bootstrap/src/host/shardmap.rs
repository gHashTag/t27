use std::collections::{BTreeMap, BTreeSet};

const VNODES_PER_SHARD: usize = 150;

fn hash_key(key: &str) -> u64 {
    const FNV: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = FNV;
    for &b in key.as_bytes() { h ^= b as u64; h = h.wrapping_mul(PRIME); }
    h
}

fn hash_vnode(shard_id: u64, vnode: usize) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    h ^= shard_id; h = h.wrapping_mul(0x100000001b3);
    h ^= vnode as u64; h = h.wrapping_mul(0x100000001b3);
    h ^= h >> 33; h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShardError {
    ShardExists { id: u64 },
    ShardNotFound { id: u64 },
    Empty,
}

impl std::fmt::Display for ShardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShardError::ShardExists { id } => write!(f, "shard {id} exists"),
            ShardError::ShardNotFound { id } => write!(f, "shard {id} not found"),
            ShardError::Empty => write!(f, "no shards"),
        }
    }
}

impl std::error::Error for ShardError {}

#[derive(Debug, Clone)]
pub struct RebalancePlan {
    pub additions: Vec<(u64, usize)>,
    pub removals: Vec<(u64, usize)>,
    pub moves: Vec<(u64, u64, usize)>,
}

#[derive(Debug, Clone)]
pub struct ShardInfo {
    pub id: u64,
    pub vnode_count: usize,
    pub keys_assigned: u64,
}

pub struct ShardMap {
    ring: BTreeMap<u64, u64>,
    shards: BTreeSet<u64>,
    key_assignments: BTreeMap<String, u64>,
    shard_key_counts: BTreeMap<u64, u64>,
    total_keys: u64,
}

impl ShardMap {
    pub fn new() -> Self {
        Self { ring: BTreeMap::new(), shards: BTreeSet::new(), key_assignments: BTreeMap::new(), shard_key_counts: BTreeMap::new(), total_keys: 0 }
    }

    pub fn add_shard(&mut self, id: u64) -> Result<usize, ShardError> {
        if self.shards.contains(&id) { return Err(ShardError::ShardExists { id }); }
        self.shards.insert(id);
        let mut added = 0;
        for v in 0..VNODES_PER_SHARD {
            let token = hash_vnode(id, v);
            self.ring.insert(token, id);
            added += 1;
        }
        self.shard_key_counts.insert(id, 0);
        Ok(added)
    }

    pub fn remove_shard(&mut self, id: u64) -> Result<usize, ShardError> {
        if !self.shards.contains(&id) { return Err(ShardError::ShardNotFound { id }); }
        self.shards.remove(&id);
        let mut removed = 0;
        self.ring.retain(|_, &mut sid| {
            if sid == id { removed += 1; false } else { true }
        });
        let keys_to_reassign: Vec<String> = self.key_assignments.iter()
            .filter(|(_, &sid)| sid == id).map(|(k, _)| k.clone()).collect();
        for key in &keys_to_reassign {
            self.assign_key(key);
        }
        self.shard_key_counts.remove(&id);
        Ok(removed)
    }

    fn find_shard(&self, hash: u64) -> Option<u64> {
        if self.ring.is_empty() { return None; }
        match self.ring.range(hash..).next() {
            Some((_, &id)) => Some(id),
            None => Some(*self.ring.values().next().unwrap()),
        }
    }

    pub fn assign_key(&mut self, key: &str) -> Result<u64, ShardError> {
        let hash = hash_key(key);
        let shard_id = self.find_shard(hash).ok_or(ShardError::Empty)?;
        if let Some(old) = self.key_assignments.insert(key.to_string(), shard_id) {
            if let Some(cnt) = self.shard_key_counts.get_mut(&old) { *cnt = cnt.saturating_sub(1); }
        } else {
            self.total_keys += 1;
        }
        *self.shard_key_counts.entry(shard_id).or_insert(0) += 1;
        Ok(shard_id)
    }

    pub fn lookup(&self, key: &str) -> Option<u64> {
        self.key_assignments.get(key).copied()
    }

    pub fn shard_for(&self, key: &str) -> Result<u64, ShardError> {
        let hash = hash_key(key);
        self.find_shard(hash).ok_or(ShardError::Empty)
    }

    pub fn shard_info(&self, id: u64) -> Option<ShardInfo> {
        if !self.shards.contains(&id) { return None; }
        let vnode_count = self.ring.values().filter(|&&sid| sid == id).count();
        Some(ShardInfo { id, vnode_count, keys_assigned: self.shard_key_counts.get(&id).copied().unwrap_or(0) })
    }

    pub fn shard_count(&self) -> usize { self.shards.len() }
    pub fn vnode_count(&self) -> usize { self.ring.len() }
    pub fn total_keys(&self) -> u64 { self.total_keys }

    pub fn rebalance_plan(&self, target_shards: &[u64]) -> RebalancePlan {
        let current: BTreeSet<u64> = self.shards.clone();
        let target: BTreeSet<u64> = target_shards.iter().copied().collect();
        let additions: Vec<(u64, usize)> = target.difference(&current).map(|&id| (id, VNODES_PER_SHARD)).collect();
        let removals: Vec<(u64, usize)> = current.difference(&target).map(|&id| (id, VNODES_PER_SHARD)).collect();
        RebalancePlan { additions, removals, moves: Vec::new() }
    }
}

impl Default for ShardMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() {
        let sm = ShardMap::new();
        assert_eq!(sm.shard_count(), 0);
    }

    #[test]
    fn add_shard() {
        let mut sm = ShardMap::new();
        let vn = sm.add_shard(1).unwrap();
        assert_eq!(vn, VNODES_PER_SHARD);
        assert_eq!(sm.shard_count(), 1);
    }

    #[test]
    fn duplicate_shard() {
        let mut sm = ShardMap::new();
        sm.add_shard(1).unwrap();
        let err = sm.add_shard(1).unwrap_err();
        assert!(matches!(err, ShardError::ShardExists { .. }));
    }

    #[test]
    fn assign_lookup() {
        let mut sm = ShardMap::new();
        sm.add_shard(1).unwrap();
        let sid = sm.assign_key("key1").unwrap();
        assert_eq!(sm.lookup("key1"), Some(sid));
    }

    #[test]
    fn no_shards_assign() {
        let mut sm = ShardMap::new();
        let err = sm.assign_key("k").unwrap_err();
        assert!(matches!(err, ShardError::Empty));
    }

    #[test]
    fn remove_shard_reassigns() {
        let mut sm = ShardMap::new();
        sm.add_shard(1).unwrap();
        sm.add_shard(2).unwrap();
        let sid = sm.assign_key("key1").unwrap();
        sm.remove_shard(sid).unwrap();
        let new_sid = sm.lookup("key1").unwrap();
        assert_ne!(new_sid, sid);
    }

    #[test]
    fn shard_info() {
        let mut sm = ShardMap::new();
        sm.add_shard(1).unwrap();
        sm.assign_key("a").unwrap();
        let info = sm.shard_info(1).unwrap();
        assert_eq!(info.vnode_count, VNODES_PER_SHARD);
    }

    #[test]
    fn distribution() {
        let mut sm = ShardMap::new();
        sm.add_shard(1).unwrap();
        sm.add_shard(2).unwrap();
        sm.add_shard(3).unwrap();
        for i in 0..1000u32 { sm.assign_key(&format!("key{i}")).unwrap(); }
        assert_eq!(sm.total_keys(), 1000);
        let info1 = sm.shard_info(1).unwrap();
        let info2 = sm.shard_info(2).unwrap();
        let info3 = sm.shard_info(3).unwrap();
        assert!(info1.keys_assigned > 20 && info2.keys_assigned > 20 && info3.keys_assigned > 20);
    }

    #[test]
    fn rebalance_plan() {
        let mut sm = ShardMap::new();
        sm.add_shard(1).unwrap();
        let plan = sm.rebalance_plan(&[1, 2]);
        assert_eq!(plan.additions.len(), 1);
        assert_eq!(plan.removals.len(), 0);
    }

    #[test]
    fn shard_for() {
        let mut sm = ShardMap::new();
        sm.add_shard(1).unwrap();
        let sid = sm.shard_for("test").unwrap();
        assert_eq!(sid, 1);
    }

    #[test]
    fn vnode_count() {
        let mut sm = ShardMap::new();
        sm.add_shard(1).unwrap();
        sm.add_shard(2).unwrap();
        assert_eq!(sm.vnode_count(), VNODES_PER_SHARD * 2);
    }

    #[test]
    fn error_display() {
        assert!(ShardError::ShardNotFound { id: 5 }.to_string().contains("5"));
    }
}

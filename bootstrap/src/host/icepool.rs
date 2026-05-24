use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum IpError {
    TierNotFound { tier: u8 },
    ItemNotFound { tier: u8, id: u64 },
    TierFull { tier: u8 },
}

impl std::fmt::Display for IpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpError::TierNotFound { tier } => write!(f, "tier {tier} not found"),
            IpError::ItemNotFound { tier, id } => write!(f, "item {id} not found in tier {tier}"),
            IpError::TierFull { tier } => write!(f, "tier {tier} full"),
        }
    }
}

impl std::error::Error for IpError {}

struct Tier {
    level: u8,
    weight: u64,
    items: BTreeMap<u64, Vec<u8>>,
    capacity: usize,
}

pub struct IcePool {
    tiers: BTreeMap<u8, Tier>,
    next_id: u64,
    total_inserts: u64,
    total_selects: u64,
    total_drains: u64,
}

impl IcePool {
    pub fn new() -> Self { Self { tiers: BTreeMap::new(), next_id: 1, total_inserts: 0, total_selects: 0, total_drains: 0 } }

    pub fn add_tier(&mut self, level: u8, weight: u64, capacity: usize) {
        self.tiers.insert(level, Tier { level, weight, items: BTreeMap::new(), capacity });
    }

    pub fn insert(&mut self, tier: u8, data: Vec<u8>) -> Result<u64, IpError> {
        let t = self.tiers.get_mut(&tier).ok_or(IpError::TierNotFound { tier })?;
        if t.items.len() >= t.capacity { return Err(IpError::TierFull { tier }); }
        let id = self.next_id;
        self.next_id += 1;
        t.items.insert(id, data);
        self.total_inserts += 1;
        Ok(id)
    }

    pub fn select(&mut self, rng: u64) -> Option<(u8, u64, Vec<u8>)> {
        self.total_selects += 1;
        let total_weight: u64 = self.tiers.values().filter(|t| !t.items.is_empty()).map(|t| t.weight).sum();
        if total_weight == 0 { return None; }
        let mut target = rng % total_weight;
        for (&level, tier) in &self.tiers {
            if tier.items.is_empty() { continue; }
            if target < tier.weight {
                let chosen_level = level;
                drop(tier);
                let tier = self.tiers.get_mut(&chosen_level)?;
                let key = *tier.items.keys().next()?;
                let data = tier.items.remove(&key)?;
                return Some((chosen_level, key, data));
            }
            target -= tier.weight;
        }
        let (&level, tier) = self.tiers.iter().find(|(_, t)| !t.items.is_empty())?;
        let key = *tier.items.keys().next()?;
        let data = self.tiers.get_mut(&level)?.items.remove(&key)?;
        Some((level, key, data))
    }

    pub fn remove(&mut self, tier: u8, id: u64) -> Result<Vec<u8>, IpError> {
        let t = self.tiers.get_mut(&tier).ok_or(IpError::TierNotFound { tier })?;
        t.items.remove(&id).ok_or(IpError::ItemNotFound { tier, id })
    }

    pub fn drain_tier(&mut self, tier: u8) -> Result<Vec<(u64, Vec<u8>)>, IpError> {
        let t = self.tiers.get_mut(&tier).ok_or(IpError::TierNotFound { tier })?;
        let keys: Vec<u64> = t.items.keys().copied().collect();
        let mut items = Vec::new();
        for k in keys { items.push((k, t.items.remove(&k).unwrap())); }
        self.total_drains += items.len() as u64;
        Ok(items)
    }

    pub fn tier_size(&self, tier: u8) -> Option<usize> { self.tiers.get(&tier).map(|t| t.items.len()) }
    pub fn tier_count(&self) -> usize { self.tiers.len() }
    pub fn total_items(&self) -> usize { self.tiers.values().map(|t| t.items.len()).sum() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_selects(&self) -> u64 { self.total_selects }
    pub fn total_drains(&self) -> u64 { self.total_drains }
}

impl Default for IcePool {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool() { assert_eq!(IcePool::new().tier_count(), 0); }

    #[test]
    fn insert_select() {
        let mut p = IcePool::new();
        p.add_tier(0, 1, 10);
        p.insert(0, b"item".to_vec()).unwrap();
        let (tier, _, data) = p.select(0).unwrap();
        assert_eq!(tier, 0);
        assert_eq!(data, b"item");
    }

    #[test]
    fn weighted_selection() {
        let mut p = IcePool::new();
        p.add_tier(0, 1, 10);
        p.add_tier(1, 100, 10);
        p.insert(0, b"rare".to_vec()).unwrap();
        p.insert(1, b"common".to_vec()).unwrap();
        let mut tier0 = 0; let mut tier1 = 0;
        for i in 0..100 { let (t, _, _) = p.select(i).unwrap(); if t == 0 { tier0 += 1; } else { tier1 += 1; } p.insert(t, b"x".to_vec()).unwrap(); }
        assert!(tier1 > tier0);
    }

    #[test]
    fn remove() {
        let mut p = IcePool::new();
        p.add_tier(0, 1, 10);
        let id = p.insert(0, b"x".to_vec()).unwrap();
        let data = p.remove(0, id).unwrap();
        assert_eq!(data, b"x");
    }

    #[test]
    fn drain_tier() {
        let mut p = IcePool::new();
        p.add_tier(0, 1, 10);
        p.insert(0, b"a".to_vec()).unwrap();
        p.insert(0, b"b".to_vec()).unwrap();
        let items = p.drain_tier(0).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(p.total_items(), 0);
    }

    #[test]
    fn tier_not_found() {
        let mut p = IcePool::new();
        let err = p.insert(5, b"x".to_vec()).unwrap_err();
        assert!(matches!(err, IpError::TierNotFound { .. }));
    }

    #[test]
    fn tier_full() {
        let mut p = IcePool::new();
        p.add_tier(0, 1, 1);
        p.insert(0, b"a".to_vec()).unwrap();
        let err = p.insert(0, b"b".to_vec()).unwrap_err();
        assert!(matches!(err, IpError::TierFull { .. }));
    }

    #[test]
    fn empty_select() {
        let mut p = IcePool::new();
        p.add_tier(0, 1, 10);
        assert!(p.select(0).is_none());
    }

    #[test]
    fn stats() {
        let mut p = IcePool::new();
        p.add_tier(0, 1, 10);
        p.insert(0, b"x".to_vec()).unwrap();
        p.select(0);
        assert_eq!(p.total_inserts(), 1);
        assert_eq!(p.total_selects(), 1);
    }

    #[test]
    fn error_display() { assert!(IpError::TierNotFound { tier: 1 }.to_string().contains("1")); }
}

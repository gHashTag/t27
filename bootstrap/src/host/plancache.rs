use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PlanError {
    PlanNotFound { id: u64 },
    PlanTooLarge { id: u64, size: usize, max: usize },
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::PlanNotFound { id } => write!(f, "plan {id} not found"),
            PlanError::PlanTooLarge { id, size, max } => write!(f, "plan {id}: {size} > {max}"),
        }
    }
}

impl std::error::Error for PlanError {}

struct Plan {
    id: u64,
    pattern: String,
    data: Vec<u8>,
    use_count: u64,
    last_used: u64,
    size: usize,
}

pub struct PlanCache {
    plans: BTreeMap<u64, Plan>,
    by_pattern: BTreeMap<String, Vec<u64>>,
    lru_order: Vec<u64>,
    capacity: usize,
    max_plan_size: usize,
    current_tick: u64,
    next_id: u64,
    total_inserts: u64,
    total_hits: u64,
    total_misses: u64,
    total_evictions: u64,
    total_invalidations: u64,
}

impl PlanCache {
    pub fn new(capacity: usize, max_plan_size: usize) -> Self {
        Self { plans: BTreeMap::new(), by_pattern: BTreeMap::new(), lru_order: Vec::new(), capacity, max_plan_size, current_tick: 0, next_id: 1, total_inserts: 0, total_hits: 0, total_misses: 0, total_evictions: 0, total_invalidations: 0 }
    }

    pub fn insert(&mut self, pattern: &str, data: Vec<u8>) -> Result<u64, PlanError> {
        if data.len() > self.max_plan_size {
            let id = self.next_id;
            return Err(PlanError::PlanTooLarge { id, size: data.len(), max: self.max_plan_size });
        }
        while self.plans.len() >= self.capacity {
            if let Some(evict_id) = self.lru_order.first().copied() {
                self.evict(evict_id);
            } else { break; }
        }
        let id = self.next_id;
        self.next_id += 1;
        self.current_tick += 1;
        let size = data.len();
        self.plans.insert(id, Plan { id, pattern: pattern.to_string(), data, use_count: 0, last_used: self.current_tick, size });
        self.by_pattern.entry(pattern.to_string()).or_default().push(id);
        self.lru_order.push(id);
        self.total_inserts += 1;
        Ok(id)
    }

    pub fn get(&mut self, id: u64) -> Option<&[u8]> {
        self.current_tick += 1;
        if let Some(plan) = self.plans.get_mut(&id) {
            plan.use_count += 1;
            plan.last_used = self.current_tick;
            self.lru_order.retain(|&x| x != id);
            self.lru_order.push(id);
            self.total_hits += 1;
            Some(plan.data.as_slice())
        } else {
            self.total_misses += 1;
            None
        }
    }

    pub fn lookup_pattern(&mut self, pattern: &str) -> Option<u64> {
        if let Some(ids) = self.by_pattern.get(pattern) {
            if let Some(&id) = ids.first() {
                self.total_hits += 1;
                return Some(id);
            }
        }
        self.total_misses += 1;
        None
    }

    pub fn invalidate_pattern(&mut self, pattern: &str) -> usize {
        let ids = self.by_pattern.remove(pattern).unwrap_or_default();
        let count = ids.len();
        for id in &ids {
            self.plans.remove(id);
            self.lru_order.retain(|&x| x != *id);
        }
        self.total_invalidations += count as u64;
        count
    }

    fn evict(&mut self, id: u64) {
        if let Some(plan) = self.plans.remove(&id) {
            if let Some(ids) = self.by_pattern.get_mut(&plan.pattern) {
                ids.retain(|&x| x != id);
            }
            self.lru_order.retain(|&x| x != id);
            self.total_evictions += 1;
        }
    }

    pub fn plan_count(&self) -> usize { self.plans.len() }
    pub fn use_count(&self, id: u64) -> Option<u64> { self.plans.get(&id).map(|p| p.use_count) }
    pub fn total_size(&self) -> usize { self.plans.values().map(|p| p.size).sum() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_hits(&self) -> u64 { self.total_hits }
    pub fn total_misses(&self) -> u64 { self.total_misses }
    pub fn total_evictions(&self) -> u64 { self.total_evictions }
    pub fn total_invalidations(&self) -> u64 { self.total_invalidations }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cache() { let pc = PlanCache::new(10, 1024); assert_eq!(pc.plan_count(), 0); }

    #[test]
    fn insert_get() {
        let mut pc = PlanCache::new(10, 1024);
        let id = pc.insert("SELECT *", b"plan_data".to_vec()).unwrap();
        let data = pc.get(id).unwrap();
        assert_eq!(data, b"plan_data");
    }

    #[test]
    fn lookup_pattern() {
        let mut pc = PlanCache::new(10, 1024);
        pc.insert("SELECT x", b"data".to_vec()).unwrap();
        let found = pc.lookup_pattern("SELECT x");
        assert_eq!(found, Some(1));
    }

    #[test]
    fn lookup_miss() {
        let mut pc = PlanCache::new(10, 1024);
        assert_eq!(pc.lookup_pattern("MISSING"), None);
    }

    #[test]
    fn lru_eviction() {
        let mut pc = PlanCache::new(2, 1024);
        pc.insert("a", b"1".to_vec()).unwrap();
        pc.insert("b", b"2".to_vec()).unwrap();
        pc.insert("c", b"3".to_vec()).unwrap();
        assert_eq!(pc.plan_count(), 2);
        assert_eq!(pc.total_evictions(), 1);
    }

    #[test]
    fn invalidate() {
        let mut pc = PlanCache::new(10, 1024);
        pc.insert("SELECT x", b"data".to_vec()).unwrap();
        let count = pc.invalidate_pattern("SELECT x");
        assert_eq!(count, 1);
        assert_eq!(pc.plan_count(), 0);
    }

    #[test]
    fn plan_too_large() {
        let mut pc = PlanCache::new(10, 10);
        let err = pc.insert("x", vec![0; 100]).unwrap_err();
        assert!(matches!(err, PlanError::PlanTooLarge { .. }));
    }

    #[test]
    fn use_count() {
        let mut pc = PlanCache::new(10, 1024);
        let id = pc.insert("x", b"d".to_vec()).unwrap();
        pc.get(id); pc.get(id);
        assert_eq!(pc.use_count(id), Some(2));
    }

    #[test]
    fn total_size() {
        let mut pc = PlanCache::new(10, 1024);
        pc.insert("a", vec![0; 10]).unwrap();
        pc.insert("b", vec![0; 20]).unwrap();
        assert_eq!(pc.total_size(), 30);
    }

    #[test]
    fn stats() {
        let mut pc = PlanCache::new(10, 1024);
        let id = pc.insert("x", b"d".to_vec()).unwrap();
        pc.get(id); pc.get(99);
        assert_eq!(pc.total_inserts(), 1);
        assert_eq!(pc.total_hits(), 1);
        assert_eq!(pc.total_misses(), 1);
    }

    #[test]
    fn error_display() { assert!(PlanError::PlanNotFound { id: 3 }.to_string().contains("3")); }
}

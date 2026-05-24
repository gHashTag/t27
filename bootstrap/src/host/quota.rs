use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuotaError {
    OverBudget { resource: u32, requested: u64, remaining: u64 },
    ResourceNotFound { resource: u32 },
    ResourceExists { resource: u32 },
}

impl std::fmt::Display for QuotaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotaError::OverBudget { resource, requested, remaining } => {
                write!(f, "res {resource}: need {requested}, have {remaining}")
            }
            QuotaError::ResourceNotFound { resource } => write!(f, "res {resource} not found"),
            QuotaError::ResourceExists { resource } => write!(f, "res {resource} exists"),
        }
    }
}

impl std::error::Error for QuotaError {}

#[derive(Debug, Clone)]
struct Quota {
    resource: u32,
    budget: u64,
    used: u64,
    peak_used: u64,
    parent: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct QuotaInfo {
    pub resource: u32,
    pub budget: u64,
    pub used: u64,
    pub remaining: u64,
    pub utilization: f64,
}

#[derive(Debug, Clone)]
pub struct QuotaTracker {
    quotas: BTreeMap<u32, Quota>,
    total_allocs: u64,
    total_releases: u64,
    total_overdraws: u64,
}

impl QuotaTracker {
    pub fn new() -> Self {
        Self { quotas: BTreeMap::new(), total_allocs: 0, total_releases: 0, total_overdraws: 0 }
    }

    pub fn create(&mut self, resource: u32, budget: u64, parent: Option<u32>) -> Result<(), QuotaError> {
        if self.quotas.contains_key(&resource) {
            return Err(QuotaError::ResourceExists { resource });
        }
        if let Some(p) = parent {
            if !self.quotas.contains_key(&p) {
                return Err(QuotaError::ResourceNotFound { resource: p });
            }
        }
        self.quotas.insert(resource, Quota { resource, budget, used: 0, peak_used: 0, parent });
        Ok(())
    }

    pub fn allocate(&mut self, resource: u32, amount: u64) -> Result<u64, QuotaError> {
        if !self.quotas.contains_key(&resource) {
            return Err(QuotaError::ResourceNotFound { resource });
        }
        let parent_res = self.quotas.get(&resource).and_then(|q| q.parent);
        {
            let q = self.quotas.get_mut(&resource).unwrap();
            if q.used + amount > q.budget {
                self.total_overdraws += 1;
                return Err(QuotaError::OverBudget {
                    resource,
                    requested: amount,
                    remaining: q.budget - q.used,
                });
            }
            q.used += amount;
            if q.used > q.peak_used { q.peak_used = q.used; }
        }
        self.total_allocs += 1;
        if let Some(parent) = parent_res {
            if let Some(pq) = self.quotas.get_mut(&parent) {
                pq.used += amount;
                if pq.used > pq.peak_used { pq.peak_used = pq.used; }
            }
        }
        Ok(self.quotas.get(&resource).map(|q| q.budget - q.used).unwrap_or(0))
    }

    pub fn release(&mut self, resource: u32, amount: u64) -> Result<u64, QuotaError> {
        if !self.quotas.contains_key(&resource) {
            return Err(QuotaError::ResourceNotFound { resource });
        }
        let parent_res = self.quotas.get(&resource).and_then(|q| q.parent);
        let actual;
        {
            let q = self.quotas.get_mut(&resource).unwrap();
            actual = amount.min(q.used);
            q.used -= actual;
        }
        self.total_releases += 1;
        if let Some(parent) = parent_res {
            if let Some(pq) = self.quotas.get_mut(&parent) {
                pq.used = pq.used.saturating_sub(actual);
            }
        }
        Ok(self.quotas.get(&resource).map(|q| q.budget - q.used).unwrap_or(0))
    }

    pub fn remaining(&self, resource: u32) -> Option<u64> {
        self.quotas.get(&resource).map(|q| q.budget - q.used)
    }

    pub fn used(&self, resource: u32) -> Option<u64> {
        self.quotas.get(&resource).map(|q| q.used)
    }

    pub fn budget(&self, resource: u32) -> Option<u64> {
        self.quotas.get(&resource).map(|q| q.budget)
    }

    pub fn peak_used(&self, resource: u32) -> Option<u64> {
        self.quotas.get(&resource).map(|q| q.peak_used)
    }

    pub fn info(&self, resource: u32) -> Option<QuotaInfo> {
        self.quotas.get(&resource).map(|q| QuotaInfo {
            resource: q.resource,
            budget: q.budget,
            used: q.used,
            remaining: q.budget - q.used,
            utilization: if q.budget == 0 { 0.0 } else { q.used as f64 / q.budget as f64 },
        })
    }

    pub fn resource_count(&self) -> usize {
        self.quotas.len()
    }

    pub fn total_allocs(&self) -> u64 { self.total_allocs }
    pub fn total_releases(&self) -> u64 { self.total_releases }
    pub fn total_overdraws(&self) -> u64 { self.total_overdraws }

    pub fn reset(&mut self) {
        for q in self.quotas.values_mut() {
            q.used = 0;
            q.peak_used = 0;
        }
        self.total_allocs = 0;
        self.total_releases = 0;
        self.total_overdraws = 0;
    }
}

impl Default for QuotaTracker {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tracker() {
        let qt = QuotaTracker::new();
        assert_eq!(qt.resource_count(), 0);
    }

    #[test]
    fn create_and_allocate() {
        let mut qt = QuotaTracker::new();
        qt.create(1, 1000, None).unwrap();
        let rem = qt.allocate(1, 300).unwrap();
        assert_eq!(rem, 700);
        assert_eq!(qt.used(1), Some(300));
    }

    #[test]
    fn over_budget() {
        let mut qt = QuotaTracker::new();
        qt.create(1, 100, None).unwrap();
        qt.allocate(1, 80).unwrap();
        let err = qt.allocate(1, 50).unwrap_err();
        assert!(matches!(err, QuotaError::OverBudget { .. }));
        assert_eq!(qt.total_overdraws(), 1);
    }

    #[test]
    fn release() {
        let mut qt = QuotaTracker::new();
        qt.create(1, 1000, None).unwrap();
        qt.allocate(1, 500).unwrap();
        let rem = qt.release(1, 200).unwrap();
        assert_eq!(rem, 700);
        assert_eq!(qt.used(1), Some(300));
    }

    #[test]
    fn release_saturating() {
        let mut qt = QuotaTracker::new();
        qt.create(1, 1000, None).unwrap();
        qt.allocate(1, 100).unwrap();
        qt.release(1, 500).unwrap();
        assert_eq!(qt.used(1), Some(0));
    }

    #[test]
    fn hierarchical() {
        let mut qt = QuotaTracker::new();
        qt.create(1, 1000, None).unwrap();
        qt.create(2, 500, Some(1)).unwrap();
        qt.allocate(2, 200).unwrap();
        assert_eq!(qt.used(2), Some(200));
        assert_eq!(qt.used(1), Some(200));
    }

    #[test]
    fn peak_used() {
        let mut qt = QuotaTracker::new();
        qt.create(1, 1000, None).unwrap();
        qt.allocate(1, 500).unwrap();
        qt.allocate(1, 200).unwrap();
        qt.release(1, 400).unwrap();
        assert_eq!(qt.peak_used(1), Some(700));
    }

    #[test]
    fn duplicate_resource() {
        let mut qt = QuotaTracker::new();
        qt.create(1, 1000, None).unwrap();
        let err = qt.create(1, 500, None).unwrap_err();
        assert!(matches!(err, QuotaError::ResourceExists { .. }));
    }

    #[test]
    fn info() {
        let mut qt = QuotaTracker::new();
        qt.create(1, 1000, None).unwrap();
        qt.allocate(1, 250).unwrap();
        let info = qt.info(1).unwrap();
        assert_eq!(info.remaining, 750);
        assert!((info.utilization - 0.25).abs() < 0.01);
    }

    #[test]
    fn stats() {
        let mut qt = QuotaTracker::new();
        qt.create(1, 1000, None).unwrap();
        qt.allocate(1, 100).unwrap();
        qt.release(1, 50).unwrap();
        assert_eq!(qt.total_allocs(), 1);
        assert_eq!(qt.total_releases(), 1);
    }

    #[test]
    fn reset() {
        let mut qt = QuotaTracker::new();
        qt.create(1, 1000, None).unwrap();
        qt.allocate(1, 500).unwrap();
        qt.reset();
        assert_eq!(qt.used(1), Some(0));
    }

    #[test]
    fn error_display() {
        assert!(QuotaError::ResourceNotFound { resource: 5 }.to_string().contains("5"));
    }
}

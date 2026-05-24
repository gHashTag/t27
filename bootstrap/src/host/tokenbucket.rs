use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TbError {
    BucketNotFound { id: u64 },
    BucketExists { id: u64 },
    InsufficientTokens { id: u64, requested: u64, available: u64 },
    ParentNotFound { id: u64 },
    OverflowBorrow { id: u64, amount: u64 },
}

impl std::fmt::Display for TbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TbError::BucketNotFound { id } => write!(f, "bucket {id} not found"),
            TbError::BucketExists { id } => write!(f, "bucket {id} exists"),
            TbError::InsufficientTokens { id, requested, available } => write!(f, "bucket {id}: need {requested}, have {available}"),
            TbError::ParentNotFound { id } => write!(f, "parent {id} not found"),
            TbError::OverflowBorrow { id, amount } => write!(f, "bucket {id}: overflow borrow {amount}"),
        }
    }
}

impl std::error::Error for TbError {}

struct Bucket {
    id: u64,
    capacity: u64,
    tokens: u64,
    refill_rate: u64,
    parent: Option<u64>,
    children: Vec<u64>,
    total_consumed: u64,
    total_refilled: u64,
}

pub struct TokenBucket {
    buckets: BTreeMap<u64, Bucket>,
    tick: u64,
    total_ticks: u64,
}

impl TokenBucket {
    pub fn new() -> Self { Self { buckets: BTreeMap::new(), tick: 0, total_ticks: 0 } }

    pub fn create(&mut self, id: u64, capacity: u64, refill_rate: u64, parent: Option<u64>) -> Result<(), TbError> {
        if self.buckets.contains_key(&id) { return Err(TbError::BucketExists { id }); }
        if let Some(pid) = parent {
            if !self.buckets.contains_key(&pid) { return Err(TbError::ParentNotFound { id: pid }); }
        }
        let b = Bucket { id, capacity, tokens: capacity, refill_rate, parent, children: Vec::new(), total_consumed: 0, total_refilled: 0 };
        self.buckets.insert(id, b);
        if let Some(pid) = parent {
            self.buckets.get_mut(&pid).unwrap().children.push(id);
        }
        Ok(())
    }

    pub fn consume(&mut self, id: u64, amount: u64) -> Result<(), TbError> {
        let b = self.buckets.get(&id).ok_or(TbError::BucketNotFound { id })?;
        if b.tokens < amount { return Err(TbError::InsufficientTokens { id, requested: amount, available: b.tokens }); }
        drop(b);
        let b = self.buckets.get_mut(&id).unwrap();
        b.tokens -= amount;
        b.total_consumed += amount;
        Ok(())
    }

    pub fn try_consume(&mut self, id: u64, amount: u64) -> bool {
        if self.buckets.get(&id).map(|b| b.tokens >= amount).unwrap_or(false) {
            let b = self.buckets.get_mut(&id).unwrap();
            b.tokens -= amount;
            b.total_consumed += amount;
            true
        } else { false }
    }

    pub fn borrow_from_parent(&mut self, id: u64, amount: u64) -> Result<(), TbError> {
        let pid = self.buckets.get(&id).and_then(|b| b.parent).ok_or(TbError::BucketNotFound { id })?;
        if self.buckets.get(&pid).map(|p| p.tokens < amount).unwrap_or(true) {
            let avail = self.buckets.get(&pid).map(|p| p.tokens).unwrap_or(0);
            return Err(TbError::InsufficientTokens { id: pid, requested: amount, available: avail });
        }
        let child_cap = self.buckets[&id].capacity;
        let child_tokens = self.buckets[&id].tokens;
        if child_tokens + amount > child_cap { return Err(TbError::OverflowBorrow { id, amount }); }
        self.buckets.get_mut(&pid).unwrap().tokens -= amount;
        self.buckets.get_mut(&pid).unwrap().total_consumed += amount;
        let b = self.buckets.get_mut(&id).unwrap();
        b.tokens += amount;
        Ok(())
    }

    pub fn return_tokens(&mut self, id: u64, amount: u64) -> Result<(), TbError> {
        let b = self.buckets.get_mut(&id).ok_or(TbError::BucketNotFound { id })?;
        b.tokens = (b.tokens + amount).min(b.capacity);
        if let Some(pid) = b.parent {
            let p = self.buckets.get_mut(&pid).unwrap();
            p.tokens = (p.tokens + amount).min(p.capacity);
        }
        Ok(())
    }

    pub fn tick(&mut self) -> u64 {
        self.tick += 1;
        self.total_ticks += 1;
        let ids: Vec<u64> = self.buckets.keys().copied().collect();
        for id in ids {
            let b = self.buckets.get_mut(&id).unwrap();
            let added = b.refill_rate.min(b.capacity.saturating_sub(b.tokens));
            b.tokens += added;
            b.total_refilled += added;
        }
        self.tick
    }

    pub fn tokens(&self, id: u64) -> Option<u64> { self.buckets.get(&id).map(|b| b.tokens) }
    pub fn capacity(&self, id: u64) -> Option<u64> { self.buckets.get(&id).map(|b| b.capacity) }
    pub fn consumed(&self, id: u64) -> Option<u64> { self.buckets.get(&id).map(|b| b.total_consumed) }
    pub fn bucket_count(&self) -> usize { self.buckets.len() }
    pub fn current_tick(&self) -> u64 { self.tick }
}

impl Default for TokenBucket {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tb() { assert_eq!(TokenBucket::new().bucket_count(), 0); }

    #[test]
    fn create_consume() {
        let mut tb = TokenBucket::new();
        tb.create(1, 100, 10, None).unwrap();
        tb.consume(1, 30).unwrap();
        assert_eq!(tb.tokens(1), Some(70));
    }

    #[test]
    fn insufficient() {
        let mut tb = TokenBucket::new();
        tb.create(1, 10, 1, None).unwrap();
        let err = tb.consume(1, 20).unwrap_err();
        assert!(matches!(err, TbError::InsufficientTokens { .. }));
    }

    #[test]
    fn try_consume() {
        let mut tb = TokenBucket::new();
        tb.create(1, 10, 1, None).unwrap();
        assert!(tb.try_consume(1, 5));
        assert!(!tb.try_consume(1, 10));
    }

    #[test]
    fn refill() {
        let mut tb = TokenBucket::new();
        tb.create(1, 100, 10, None).unwrap();
        tb.consume(1, 50).unwrap();
        tb.tick();
        assert_eq!(tb.tokens(1), Some(60));
    }

    #[test]
    fn hierarchical_borrow() {
        let mut tb = TokenBucket::new();
        tb.create(1, 100, 10, None).unwrap();
        tb.create(2, 50, 5, Some(1)).unwrap();
        tb.consume(2, 50).unwrap();
        tb.borrow_from_parent(2, 20).unwrap();
        assert_eq!(tb.tokens(2), Some(20));
        assert_eq!(tb.tokens(1), Some(80));
    }

    #[test]
    fn return_tokens() {
        let mut tb = TokenBucket::new();
        tb.create(1, 100, 10, None).unwrap();
        tb.create(2, 50, 5, Some(1)).unwrap();
        tb.consume(2, 20).unwrap();
        tb.return_tokens(2, 10).unwrap();
        assert_eq!(tb.tokens(2), Some(40));
    }

    #[test]
    fn overflow_borrow() {
        let mut tb = TokenBucket::new();
        tb.create(1, 100, 10, None).unwrap();
        tb.create(2, 50, 5, Some(1)).unwrap();
        let err = tb.borrow_from_parent(2, 60).unwrap_err();
        assert!(matches!(err, TbError::OverflowBorrow { .. }));
    }

    #[test]
    fn duplicate() {
        let mut tb = TokenBucket::new();
        tb.create(1, 100, 10, None).unwrap();
        let err = tb.create(1, 100, 10, None).unwrap_err();
        assert!(matches!(err, TbError::BucketExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut tb = TokenBucket::new();
        let err = tb.consume(99, 1).unwrap_err();
        assert!(matches!(err, TbError::BucketNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut tb = TokenBucket::new();
        tb.create(1, 100, 10, None).unwrap();
        tb.consume(1, 30).unwrap();
        assert_eq!(tb.consumed(1), Some(30));
    }

    #[test]
    fn error_display() { assert!(TbError::BucketNotFound { id: 3 }.to_string().contains("3")); }
}

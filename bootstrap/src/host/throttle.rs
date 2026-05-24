use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ThError {
    KeyNotFound { key: u64 },
}

impl std::fmt::Display for ThError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThError::KeyNotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for ThError {}

struct Bucket {
    key: u64,
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: u64,
    total_allowed: u64,
    total_denied: u64,
}

pub struct Throttle {
    buckets: BTreeMap<u64, Bucket>,
    default_max: f64,
    default_rate: f64,
    total_allowed: u64,
    total_denied: u64,
}

impl Throttle {
    pub fn new(default_max: f64, default_rate: f64) -> Self {
        Self { buckets: BTreeMap::new(), default_max, default_rate, total_allowed: 0, total_denied: 0 }
    }

    pub fn register(&mut self, key: u64, max_tokens: f64, refill_rate: f64, now: u64) {
        self.buckets.insert(key, Bucket { key, tokens: max_tokens, max_tokens, refill_rate, last_refill: now, total_allowed: 0, total_denied: 0 });
    }

    pub fn register_default(&mut self, key: u64, now: u64) {
        self.register(key, self.default_max, self.default_rate, now);
    }

    fn refill(&mut self, key: u64, now: u64) {
        let b = self.buckets.get_mut(&key).unwrap();
        if now > b.last_refill {
            let elapsed = (now - b.last_refill) as f64;
            b.tokens = (b.tokens + elapsed * b.refill_rate).min(b.max_tokens);
            b.last_refill = now;
        }
    }

    pub fn allow(&mut self, key: u64, cost: f64, now: u64) -> bool {
        if !self.buckets.contains_key(&key) { self.register_default(key, now); }
        self.refill(key, now);
        let b = self.buckets.get_mut(&key).unwrap();
        if b.tokens >= cost {
            b.tokens -= cost;
            b.total_allowed += 1;
            self.total_allowed += 1;
            true
        } else {
            b.total_denied += 1;
            self.total_denied += 1;
            false
        }
    }

    pub fn tokens(&mut self, key: u64, now: u64) -> f64 {
        if !self.buckets.contains_key(&key) { return 0.0; }
        self.refill(key, now);
        self.buckets.get(&key).unwrap().tokens
    }

    pub fn reset(&mut self, key: u64) -> Result<(), ThError> {
        let b = self.buckets.get_mut(&key).ok_or(ThError::KeyNotFound { key })?;
        b.tokens = b.max_tokens;
        Ok(())
    }

    pub fn key_stats(&self, key: u64) -> Option<(u64, u64)> {
        self.buckets.get(&key).map(|b| (b.total_allowed, b.total_denied))
    }

    pub fn bucket_count(&self) -> usize { self.buckets.len() }
    pub fn total_allowed(&self) -> u64 { self.total_allowed }
    pub fn total_denied(&self) -> u64 { self.total_denied }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_throttle() { let t = Throttle::new(10.0, 1.0); assert_eq!(t.bucket_count(), 0); }

    #[test]
    fn allow_within_budget() {
        let mut t = Throttle::new(10.0, 1.0);
        assert!(t.allow(1, 5.0, 0));
        assert!(t.allow(1, 5.0, 0));
    }

    #[test]
    fn deny_over_budget() {
        let mut t = Throttle::new(10.0, 1.0);
        assert!(t.allow(1, 10.0, 0));
        assert!(!t.allow(1, 1.0, 0));
    }

    #[test]
    fn refill_over_time() {
        let mut t = Throttle::new(10.0, 1.0);
        t.allow(1, 10.0, 0);
        assert!(t.allow(1, 5.0, 10));
    }

    #[test]
    fn max_cap() {
        let mut t = Throttle::new(10.0, 1.0);
        t.allow(1, 10.0, 0);
        let tokens = t.tokens(1, 1000);
        assert!((tokens - 10.0).abs() < 0.01);
    }

    #[test]
    fn per_key_isolation() {
        let mut t = Throttle::new(5.0, 1.0);
        t.allow(1, 5.0, 0);
        assert!(t.allow(2, 5.0, 0));
    }

    #[test]
    fn reset() {
        let mut t = Throttle::new(5.0, 1.0);
        t.allow(1, 5.0, 0);
        t.reset(1).unwrap();
        assert!(t.allow(1, 5.0, 0));
    }

    #[test]
    fn key_stats() {
        let mut t = Throttle::new(10.0, 1.0);
        t.allow(1, 1.0, 0);
        t.allow(1, 1.0, 0);
        t.allow(1, 20.0, 0);
        let (a, d) = t.key_stats(1).unwrap();
        assert_eq!(a, 2);
        assert_eq!(d, 1);
    }

    #[test]
    fn global_stats() {
        let mut t = Throttle::new(10.0, 1.0);
        t.allow(1, 10.0, 0);
        t.allow(1, 1.0, 0);
        assert_eq!(t.total_allowed(), 1);
        assert_eq!(t.total_denied(), 1);
    }

    #[test]
    fn auto_register() {
        let mut t = Throttle::new(5.0, 1.0);
        assert!(t.allow(99, 3.0, 0));
        assert_eq!(t.bucket_count(), 1);
    }

    #[test]
    fn error_display() { assert!(ThError::KeyNotFound { key: 3 }.to_string().contains("3")); }
}

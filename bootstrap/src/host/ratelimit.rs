use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RlError {
    KeyNotFound { key: u64 },
}

impl std::fmt::Display for RlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RlError::KeyNotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for RlError {}

struct Window {
    key: u64,
    window_size: u64,
    max_requests: u64,
    timestamps: Vec<u64>,
    total_allowed: u64,
    total_denied: u64,
}

pub struct RateLimit {
    windows: BTreeMap<u64, Window>,
    default_window: u64,
    default_max: u64,
    total_allowed: u64,
    total_denied: u64,
}

impl RateLimit {
    pub fn new(default_window: u64, default_max: u64) -> Self {
        Self { windows: BTreeMap::new(), default_window, default_max, total_allowed: 0, total_denied: 0 }
    }

    pub fn register(&mut self, key: u64, window_size: u64, max_requests: u64) {
        self.windows.insert(key, Window { key, window_size, max_requests, timestamps: Vec::new(), total_allowed: 0, total_denied: 0 });
    }

    pub fn register_default(&mut self, key: u64) {
        self.register(key, self.default_window, self.default_max);
    }

    pub fn allow(&mut self, key: u64, now: u64) -> bool {
        if !self.windows.contains_key(&key) { self.register_default(key); }
        let w = self.windows.get_mut(&key).unwrap();
        let cutoff = now.saturating_sub(w.window_size);
        w.timestamps.retain(|&t| now.saturating_sub(t) < w.window_size);
        if (w.timestamps.len() as u64) < w.max_requests {
            w.timestamps.push(now);
            w.total_allowed += 1;
            self.total_allowed += 1;
            true
        } else {
            w.total_denied += 1;
            self.total_denied += 1;
            false
        }
    }

    pub fn remaining(&mut self, key: u64, now: u64) -> Result<u64, RlError> {
        let w = self.windows.get_mut(&key).ok_or(RlError::KeyNotFound { key })?;
        let cutoff = now.saturating_sub(w.window_size);
        w.timestamps.retain(|&t| now.saturating_sub(t) < w.window_size);
        Ok(w.max_requests.saturating_sub(w.timestamps.len() as u64))
    }

    pub fn reset(&mut self, key: u64) -> Result<(), RlError> {
        let w = self.windows.get_mut(&key).ok_or(RlError::KeyNotFound { key })?;
        w.timestamps.clear();
        Ok(())
    }

    pub fn key_stats(&self, key: u64) -> Option<(u64, u64)> {
        self.windows.get(&key).map(|w| (w.total_allowed, w.total_denied))
    }

    pub fn window_count(&self) -> usize { self.windows.len() }
    pub fn total_allowed(&self) -> u64 { self.total_allowed }
    pub fn total_denied(&self) -> u64 { self.total_denied }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_limiter() { let rl = RateLimit::new(100, 5); assert_eq!(rl.window_count(), 0); }

    #[test]
    fn allow_within_limit() {
        let mut rl = RateLimit::new(100, 3);
        assert!(rl.allow(1, 10));
        assert!(rl.allow(1, 20));
        assert!(rl.allow(1, 30));
    }

    #[test]
    fn deny_over_limit() {
        let mut rl = RateLimit::new(100, 2);
        rl.allow(1, 10);
        rl.allow(1, 20);
        assert!(!rl.allow(1, 30));
    }

    #[test]
    fn window_expiry() {
        let mut rl = RateLimit::new(10, 2);
        rl.allow(1, 0);
        rl.allow(1, 5);
        assert!(rl.allow(1, 20));
    }

    #[test]
    fn per_key_isolation() {
        let mut rl = RateLimit::new(100, 1);
        rl.allow(1, 0);
        assert!(rl.allow(2, 0));
    }

    #[test]
    fn remaining() {
        let mut rl = RateLimit::new(100, 5);
        rl.allow(1, 0);
        rl.allow(1, 10);
        assert_eq!(rl.remaining(1, 15).unwrap(), 3);
    }

    #[test]
    fn reset() {
        let mut rl = RateLimit::new(100, 1);
        rl.allow(1, 0);
        rl.reset(1).unwrap();
        assert!(rl.allow(1, 0));
    }

    #[test]
    fn key_stats() {
        let mut rl = RateLimit::new(100, 1);
        rl.allow(1, 0);
        rl.allow(1, 10);
        let (a, d) = rl.key_stats(1).unwrap();
        assert_eq!(a, 1);
        assert_eq!(d, 1);
    }

    #[test]
    fn auto_register() {
        let mut rl = RateLimit::new(100, 5);
        rl.allow(99, 0);
        assert_eq!(rl.window_count(), 1);
    }

    #[test]
    fn stats() {
        let mut rl = RateLimit::new(100, 1);
        rl.allow(1, 0);
        rl.allow(1, 10);
        assert_eq!(rl.total_allowed(), 1);
        assert_eq!(rl.total_denied(), 1);
    }

    #[test]
    fn error_display() { assert!(RlError::KeyNotFound { key: 3 }.to_string().contains("3")); }
}

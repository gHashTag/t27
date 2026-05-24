use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitError {
    LimitExceeded { key: String, limit: u64, window_ms: u64 },
    NotFound { key: String },
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::LimitExceeded { key, limit, window_ms } =>
                write!(f, "{key}: limit {limit} exceeded in {window_ms}ms"),
            RateLimitError::NotFound { key } => write!(f, "rate limit {key} not found"),
        }
    }
}

impl std::error::Error for RateLimitError {}

struct Limiter {
    limit: u64,
    window_ms: u64,
    timestamps: VecDeque<u64>,
    total_allowed: u64,
    total_rejected: u64,
}

pub struct RateLimiter {
    limiters: std::collections::BTreeMap<String, Limiter>,
}

impl RateLimiter {
    pub fn new() -> Self { Self { limiters: std::collections::BTreeMap::new() } }

    pub fn register(&mut self, key: &str, limit: u64, window_ms: u64) {
        self.limiters.insert(key.to_string(), Limiter {
            limit, window_ms, timestamps: VecDeque::new(), total_allowed: 0, total_rejected: 0,
        });
    }

    pub fn check(&mut self, key: &str, now_ms: u64) -> Result<bool, RateLimitError> {
        let limiter = self.limiters.get_mut(key).ok_or_else(|| RateLimitError::NotFound { key: key.to_string() })?;
        let cutoff = now_ms.saturating_sub(limiter.window_ms);
        while let Some(&ts) = limiter.timestamps.front() {
            if ts <= cutoff { limiter.timestamps.pop_front(); } else { break; }
        }
        if limiter.timestamps.len() < limiter.limit as usize {
            limiter.timestamps.push_back(now_ms);
            limiter.total_allowed += 1;
            Ok(true)
        } else {
            limiter.total_rejected += 1;
            Err(RateLimitError::LimitExceeded { key: key.to_string(), limit: limiter.limit, window_ms: limiter.window_ms })
        }
    }

    pub fn remaining(&self, key: &str, now_ms: u64) -> Option<u64> {
        let limiter = self.limiters.get(key)?;
        let cutoff = now_ms.saturating_sub(limiter.window_ms);
        let count = limiter.timestamps.iter().filter(|&&ts| ts > cutoff).count() as u64;
        Some(limiter.limit.saturating_sub(count))
    }

    pub fn reset(&mut self, key: &str) -> Result<(), RateLimitError> {
        let limiter = self.limiters.get_mut(key)
            .ok_or_else(|| RateLimitError::NotFound { key: key.to_string() })?;
        limiter.timestamps.clear();
        Ok(())
    }

    pub fn stats(&self, key: &str) -> Option<(u64, u64)> {
        self.limiters.get(key).map(|l| (l.total_allowed, l.total_rejected))
    }

    pub fn limiter_count(&self) -> usize { self.limiters.len() }
}

impl Default for RateLimiter {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_limiter() {
        let rl = RateLimiter::new();
        assert_eq!(rl.limiter_count(), 0);
    }

    #[test]
    fn register_and_check() {
        let mut rl = RateLimiter::new();
        rl.register("api", 3, 1000);
        assert!(rl.check("api", 100).unwrap());
        assert!(rl.check("api", 200).unwrap());
        assert!(rl.check("api", 300).unwrap());
    }

    #[test]
    fn limit_exceeded() {
        let mut rl = RateLimiter::new();
        rl.register("api", 2, 1000);
        rl.check("api", 100).unwrap();
        rl.check("api", 200).unwrap();
        let err = rl.check("api", 300).unwrap_err();
        assert!(matches!(err, RateLimitError::LimitExceeded { .. }));
    }

    #[test]
    fn window_expiry() {
        let mut rl = RateLimiter::new();
        rl.register("api", 2, 100);
        rl.check("api", 10).unwrap();
        rl.check("api", 20).unwrap();
        assert!(rl.check("api", 200).unwrap());
    }

    #[test]
    fn remaining() {
        let mut rl = RateLimiter::new();
        rl.register("api", 5, 1000);
        rl.check("api", 100).unwrap();
        rl.check("api", 200).unwrap();
        assert_eq!(rl.remaining("api", 300), Some(3));
    }

    #[test]
    fn reset() {
        let mut rl = RateLimiter::new();
        rl.register("api", 2, 1000);
        rl.check("api", 100).unwrap();
        rl.check("api", 200).unwrap();
        rl.reset("api").unwrap();
        assert!(rl.check("api", 300).unwrap());
    }

    #[test]
    fn not_found() {
        let mut rl = RateLimiter::new();
        let err = rl.check("nope", 100).unwrap_err();
        assert!(matches!(err, RateLimitError::NotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut rl = RateLimiter::new();
        rl.register("api", 2, 1000);
        rl.check("api", 100).unwrap();
        rl.check("api", 200).unwrap();
        let _ = rl.check("api", 300);
        let (allowed, rejected) = rl.stats("api").unwrap();
        assert_eq!(allowed, 2);
        assert_eq!(rejected, 1);
    }

    #[test]
    fn multiple_keys() {
        let mut rl = RateLimiter::new();
        rl.register("a", 1, 1000);
        rl.register("b", 10, 1000);
        rl.check("a", 100).unwrap();
        let err = rl.check("a", 200).unwrap_err();
        assert!(matches!(err, RateLimitError::LimitExceeded { .. }));
        assert!(rl.check("b", 100).unwrap());
    }

    #[test]
    fn zero_remaining_no_window() {
        let rl = RateLimiter::new();
        assert_eq!(rl.remaining("nope", 100), None);
    }

    #[test]
    fn error_display() {
        assert!(RateLimitError::NotFound { key: "x".into() }.to_string().contains("x"));
    }

    #[test]
    fn burst_then_wait() {
        let mut rl = RateLimiter::new();
        rl.register("api", 5, 100);
        for t in (0..5).map(|i| 10 + i as u64 * 10) { assert!(rl.check("api", t).unwrap()); }
        assert!(rl.check("api", 90).is_err());
        assert!(rl.check("api", 300).unwrap());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitError {
    WouldBlock { available_in_ms: u64 },
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::WouldBlock { available_in_ms } => {
                write!(f, "rate limited, retry in {available_in_ms}ms")
            }
        }
    }
}

impl std::error::Error for RateLimitError {}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub max_tokens: u64,
    pub refill_rate_per_ms: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_tokens: 100,
            refill_rate_per_ms: 1,
        }
    }
}

impl RateLimitConfig {
    pub fn new(max_tokens: u64, refill_per_second: u64) -> Self {
        Self {
            max_tokens,
            refill_rate_per_ms: refill_per_second / 1000,
        }
    }

    pub fn burst(max: u64) -> Self {
        Self {
            max_tokens: max,
            refill_rate_per_ms: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    tokens: u64,
    last_refill_ms: u64,
    total_allowed: u64,
    total_rejected: u64,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            tokens: config.max_tokens,
            config,
            last_refill_ms: 0,
            total_allowed: 0,
            total_rejected: 0,
        }
    }

    pub fn try_acquire(&mut self, now_ms: u64) -> Result<(), RateLimitError> {
        self.refill(now_ms);
        if self.tokens > 0 {
            self.tokens -= 1;
            self.total_allowed += 1;
            Ok(())
        } else {
            self.total_rejected += 1;
            let available_in_ms = if self.config.refill_rate_per_ms > 0 {
                1
            } else {
                u64::MAX
            };
            Err(RateLimitError::WouldBlock { available_in_ms })
        }
    }

    pub fn acquire(&mut self, now_ms: u64, count: u64) -> Result<(), RateLimitError> {
        self.refill(now_ms);
        if self.tokens >= count {
            self.tokens -= count;
            self.total_allowed += count;
            Ok(())
        } else {
            self.total_rejected += 1;
            let deficit = count - self.tokens;
            let available_in_ms = if self.config.refill_rate_per_ms > 0 {
                deficit / self.config.refill_rate_per_ms + 1
            } else {
                u64::MAX
            };
            Err(RateLimitError::WouldBlock { available_in_ms })
        }
    }

    fn refill(&mut self, now_ms: u64) {
        if now_ms > self.last_refill_ms && self.config.refill_rate_per_ms > 0 {
            let elapsed = now_ms - self.last_refill_ms;
            let added = elapsed * self.config.refill_rate_per_ms;
            self.tokens = (self.tokens + added).min(self.config.max_tokens);
        }
        self.last_refill_ms = now_ms;
    }

    pub fn tokens(&self) -> u64 {
        self.tokens
    }

    pub fn total_allowed(&self) -> u64 {
        self.total_allowed
    }

    pub fn total_rejected(&self) -> u64 {
        self.total_rejected
    }

    pub fn reset(&mut self) {
        self.tokens = self.config.max_tokens;
        self.last_refill_ms = 0;
        self.total_allowed = 0;
        self.total_rejected = 0;
    }

    pub fn stats(&self) -> RateLimitStats {
        RateLimitStats {
            tokens: self.tokens,
            max_tokens: self.config.max_tokens,
            total_allowed: self.total_allowed,
            total_rejected: self.total_rejected,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimitStats {
    pub tokens: u64,
    pub max_tokens: u64,
    pub total_allowed: u64,
    pub total_rejected: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_limiter_has_max_tokens() {
        let r = RateLimiter::new(RateLimitConfig::default());
        assert_eq!(r.tokens(), 100);
    }

    #[test]
    fn try_acquire_decrements() {
        let mut r = RateLimiter::new(RateLimitConfig::default());
        r.try_acquire(0).unwrap();
        assert_eq!(r.tokens(), 99);
        assert_eq!(r.total_allowed(), 1);
    }

    #[test]
    fn try_acquire_exhausted() {
        let mut r = RateLimiter::new(RateLimitConfig::burst(2));
        r.try_acquire(0).unwrap();
        r.try_acquire(0).unwrap();
        let err = r.try_acquire(0).unwrap_err();
        assert!(matches!(err, RateLimitError::WouldBlock { .. }));
        assert_eq!(r.total_rejected(), 1);
    }

    #[test]
    fn refill_over_time() {
        let config = RateLimitConfig {
            max_tokens: 10,
            refill_rate_per_ms: 1,
        };
        let mut r = RateLimiter::new(config);
        for _ in 0..10 {
            r.try_acquire(0).unwrap();
        }
        assert_eq!(r.tokens(), 0);
        r.try_acquire(100).unwrap();
        assert_eq!(r.tokens(), 9);
    }

    #[test]
    fn refill_capped_at_max() {
        let config = RateLimitConfig {
            max_tokens: 5,
            refill_rate_per_ms: 1,
        };
        let mut r = RateLimiter::new(config);
        assert_eq!(r.tokens(), 5);
        r.refill(10000);
        assert_eq!(r.tokens(), 5);
    }

    #[test]
    fn acquire_batch() {
        let mut r = RateLimiter::new(RateLimitConfig::burst(10));
        r.acquire(0, 5).unwrap();
        assert_eq!(r.tokens(), 5);
    }

    #[test]
    fn acquire_batch_too_large() {
        let mut r = RateLimiter::new(RateLimitConfig::burst(3));
        let err = r.acquire(0, 5).unwrap_err();
        assert!(matches!(err, RateLimitError::WouldBlock { .. }));
    }

    #[test]
    fn stats() {
        let mut r = RateLimiter::new(RateLimitConfig::burst(2));
        r.try_acquire(0).unwrap();
        r.try_acquire(0).unwrap();
        r.try_acquire(0).unwrap_err();
        let s = r.stats();
        assert_eq!(s.max_tokens, 2);
        assert_eq!(s.total_allowed, 2);
        assert_eq!(s.total_rejected, 1);
    }

    #[test]
    fn reset() {
        let mut r = RateLimiter::new(RateLimitConfig::burst(5));
        for _ in 0..5 {
            r.try_acquire(0).unwrap();
        }
        r.reset();
        assert_eq!(r.tokens(), 5);
        assert_eq!(r.total_allowed(), 0);
    }

    #[test]
    fn burst_config_no_refill() {
        let mut r = RateLimiter::new(RateLimitConfig::burst(1));
        r.try_acquire(0).unwrap();
        assert!(r.try_acquire(1000).is_err());
    }

    #[test]
    fn error_display() {
        let e = RateLimitError::WouldBlock { available_in_ms: 50 };
        assert!(e.to_string().contains("50ms"));
    }

    #[test]
    fn default_config() {
        let c = RateLimitConfig::default();
        assert_eq!(c.max_tokens, 100);
    }
}

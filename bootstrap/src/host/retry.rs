#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryError {
    MaxAttemptsExceeded { attempts: u32 },
}

impl std::fmt::Display for RetryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryError::MaxAttemptsExceeded { attempts } => {
                write!(f, "max attempts exceeded: {attempts}")
            }
        }
    }
}

impl std::error::Error for RetryError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackoffStrategy {
    Fixed { delay_ms: u64 },
    Linear { base_ms: u64, increment_ms: u64 },
    Exponential { base_ms: u64, multiplier: u64 },
}

impl BackoffStrategy {
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        match self {
            BackoffStrategy::Fixed { delay_ms } => *delay_ms,
            BackoffStrategy::Linear {
                base_ms,
                increment_ms,
            } => base_ms + increment_ms * attempt as u64,
            BackoffStrategy::Exponential {
                base_ms,
                multiplier,
            } => base_ms * multiplier.pow(attempt),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff: BackoffStrategy,
    pub max_delay_ms: u64,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff: BackoffStrategy::Fixed { delay_ms: 100 },
            max_delay_ms: 30_000,
            jitter: false,
        }
    }
}

impl RetryPolicy {
    pub fn new(max_attempts: u32, backoff: BackoffStrategy) -> Self {
        Self {
            max_attempts,
            backoff,
            max_delay_ms: 30_000,
            jitter: false,
        }
    }

    pub fn with_max_delay(mut self, ms: u64) -> Self {
        self.max_delay_ms = ms;
        self
    }

    pub fn with_jitter(mut self, on: bool) -> Self {
        self.jitter = on;
        self
    }

    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        let raw = self.backoff.delay_for_attempt(attempt);
        raw.min(self.max_delay_ms)
    }

    pub fn total_delay_ms(&self) -> u64 {
        (0..self.max_attempts).map(|i| self.delay_for_attempt(i)).sum()
    }

    pub fn should_retry(&self, attempts_so_far: u32) -> bool {
        attempts_so_far < self.max_attempts
    }

    pub fn check(&self, attempts_so_far: u32) -> Result<(), RetryError> {
        if self.should_retry(attempts_so_far) {
            Ok(())
        } else {
            Err(RetryError::MaxAttemptsExceeded {
                attempts: attempts_so_far,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetryState {
    policy: RetryPolicy,
    attempts: u32,
    total_delay_ms: u64,
}

impl RetryState {
    pub fn new(policy: RetryPolicy) -> Self {
        Self {
            policy,
            attempts: 0,
            total_delay_ms: 0,
        }
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    pub fn total_delay_ms(&self) -> u64 {
        self.total_delay_ms
    }

    pub fn policy(&self) -> &RetryPolicy {
        &self.policy
    }

    pub fn next_delay(&mut self) -> Option<u64> {
        if !self.policy.should_retry(self.attempts) {
            return None;
        }
        let delay = self.policy.delay_for_attempt(self.attempts);
        self.total_delay_ms += delay;
        self.attempts += 1;
        Some(delay)
    }

    pub fn reset(&mut self) {
        self.attempts = 0;
        self.total_delay_ms = 0;
    }

    pub fn is_exhausted(&self) -> bool {
        !self.policy.should_retry(self.attempts)
    }

    pub fn remaining(&self) -> u32 {
        self.policy.max_attempts.saturating_sub(self.attempts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_backoff() {
        let b = BackoffStrategy::Fixed { delay_ms: 50 };
        assert_eq!(b.delay_for_attempt(0), 50);
        assert_eq!(b.delay_for_attempt(5), 50);
    }

    #[test]
    fn linear_backoff() {
        let b = BackoffStrategy::Linear {
            base_ms: 100,
            increment_ms: 50,
        };
        assert_eq!(b.delay_for_attempt(0), 100);
        assert_eq!(b.delay_for_attempt(1), 150);
        assert_eq!(b.delay_for_attempt(3), 250);
    }

    #[test]
    fn exponential_backoff() {
        let b = BackoffStrategy::Exponential {
            base_ms: 100,
            multiplier: 2,
        };
        assert_eq!(b.delay_for_attempt(0), 100);
        assert_eq!(b.delay_for_attempt(1), 200);
        assert_eq!(b.delay_for_attempt(2), 400);
        assert_eq!(b.delay_for_attempt(3), 800);
    }

    #[test]
    fn policy_default() {
        let p = RetryPolicy::default();
        assert_eq!(p.max_attempts, 3);
        assert_eq!(p.delay_for_attempt(0), 100);
    }

    #[test]
    fn policy_builder() {
        let p = RetryPolicy::new(5, BackoffStrategy::Exponential {
            base_ms: 50,
            multiplier: 2,
        }).with_max_delay(1000).with_jitter(true);
        assert_eq!(p.max_attempts, 5);
        assert!(p.jitter);
        assert_eq!(p.max_delay_ms, 1000);
    }

    #[test]
    fn max_delay_caps() {
        let p = RetryPolicy::new(10, BackoffStrategy::Exponential {
            base_ms: 100,
            multiplier: 10,
        }).with_max_delay(500);
        assert_eq!(p.delay_for_attempt(2), 500);
    }

    #[test]
    fn total_delay_ms() {
        let p = RetryPolicy::new(3, BackoffStrategy::Fixed { delay_ms: 100 });
        assert_eq!(p.total_delay_ms(), 300);
    }

    #[test]
    fn should_retry() {
        let p = RetryPolicy::new(3, BackoffStrategy::Fixed { delay_ms: 100 });
        assert!(p.should_retry(0));
        assert!(p.should_retry(2));
        assert!(!p.should_retry(3));
    }

    #[test]
    fn check_returns_error() {
        let p = RetryPolicy::new(2, BackoffStrategy::Fixed { delay_ms: 100 });
        p.check(1).unwrap();
        let err = p.check(2).unwrap_err();
        assert!(matches!(err, RetryError::MaxAttemptsExceeded { attempts: 2 }));
    }

    #[test]
    fn state_next_delay() {
        let policy = RetryPolicy::new(3, BackoffStrategy::Fixed { delay_ms: 50 });
        let mut s = RetryState::new(policy);
        assert_eq!(s.next_delay(), Some(50));
        assert_eq!(s.attempts(), 1);
        assert_eq!(s.next_delay(), Some(50));
        assert_eq!(s.next_delay(), Some(50));
        assert_eq!(s.next_delay(), None);
        assert!(s.is_exhausted());
    }

    #[test]
    fn state_total_delay() {
        let policy = RetryPolicy::new(3, BackoffStrategy::Linear {
            base_ms: 100,
            increment_ms: 50,
        });
        let mut s = RetryState::new(policy);
        s.next_delay();
        s.next_delay();
        assert_eq!(s.total_delay_ms(), 100 + 150);
    }

    #[test]
    fn state_remaining() {
        let policy = RetryPolicy::new(5, BackoffStrategy::Fixed { delay_ms: 10 });
        let mut s = RetryState::new(policy);
        assert_eq!(s.remaining(), 5);
        s.next_delay();
        assert_eq!(s.remaining(), 4);
    }

    #[test]
    fn state_reset() {
        let policy = RetryPolicy::new(2, BackoffStrategy::Fixed { delay_ms: 10 });
        let mut s = RetryState::new(policy);
        s.next_delay();
        s.next_delay();
        assert!(s.is_exhausted());
        s.reset();
        assert_eq!(s.attempts(), 0);
        assert!(!s.is_exhausted());
    }

    #[test]
    fn error_display() {
        let e = RetryError::MaxAttemptsExceeded { attempts: 5 };
        assert!(e.to_string().contains("5"));
    }
}

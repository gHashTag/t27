#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackoffMode {
    Fixed,
    Linear,
    Exponential,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackoffError {
    MaxRetriesExceeded { attempts: u32, max: u32 },
    AlreadySucceeded,
}

impl std::fmt::Display for BackoffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackoffError::MaxRetriesExceeded { attempts, max } => {
                write!(f, "{attempts}/{max} retries exceeded")
            }
            BackoffError::AlreadySucceeded => write!(f, "already succeeded"),
        }
    }
}

impl std::error::Error for BackoffError {}

#[derive(Debug, Clone)]
pub struct BackoffConfig {
    pub mode: BackoffMode,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub max_retries: u32,
    pub jitter_pct: u8,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            mode: BackoffMode::Exponential,
            base_delay_ms: 100,
            max_delay_ms: 30_000,
            max_retries: 10,
            jitter_pct: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackoffStrategy {
    config: BackoffConfig,
    attempt: u32,
    total_wait_ms: u64,
    succeeded: bool,
}

impl BackoffStrategy {
    pub fn new(config: BackoffConfig) -> Self {
        Self { config, attempt: 0, total_wait_ms: 0, succeeded: false }
    }

    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    pub fn total_wait_ms(&self) -> u64 {
        self.total_wait_ms
    }

    pub fn succeeded(&self) -> bool {
        self.succeeded
    }

    pub fn remaining(&self) -> u32 {
        self.config.max_retries.saturating_sub(self.attempt)
    }

    pub fn next_delay(&mut self) -> Result<u64, BackoffError> {
        if self.succeeded { return Err(BackoffError::AlreadySucceeded); }
        if self.attempt >= self.config.max_retries {
            return Err(BackoffError::MaxRetriesExceeded {
                attempts: self.attempt,
                max: self.config.max_retries,
            });
        }
        let delay = self.compute_delay(self.attempt);
        self.attempt += 1;
        self.total_wait_ms += delay;
        Ok(delay)
    }

    fn compute_delay(&self, attempt: u32) -> u64 {
        let raw = match self.config.mode {
            BackoffMode::Fixed => self.config.base_delay_ms,
            BackoffMode::Linear => self.config.base_delay_ms * (attempt as u64 + 1),
            BackoffMode::Exponential => {
                let exp = 1u64.checked_shl(attempt).unwrap_or(u64::MAX);
                self.config.base_delay_ms.saturating_mul(exp)
            }
        };
        raw.min(self.config.max_delay_ms)
    }

    pub fn peek_delay(&self) -> u64 {
        if self.attempt >= self.config.max_retries { return 0; }
        self.compute_delay(self.attempt)
    }

    pub fn mark_success(&mut self) {
        self.succeeded = true;
    }

    pub fn reset(&mut self) {
        self.attempt = 0;
        self.total_wait_ms = 0;
        self.succeeded = false;
    }

    pub fn config(&self) -> &BackoffConfig {
        &self.config
    }

    pub fn is_exhausted(&self) -> bool {
        self.attempt >= self.config.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exp_config() -> BackoffConfig {
        BackoffConfig { mode: BackoffMode::Exponential, base_delay_ms: 100, max_delay_ms: 5000, max_retries: 5, jitter_pct: 0 }
    }

    #[test]
    fn exponential_backoff() {
        let mut bs = BackoffStrategy::new(exp_config());
        assert_eq!(bs.next_delay().unwrap(), 100);
        assert_eq!(bs.next_delay().unwrap(), 200);
        assert_eq!(bs.next_delay().unwrap(), 400);
        assert_eq!(bs.next_delay().unwrap(), 800);
    }

    #[test]
    fn max_delay_clamp() {
        let cfg = BackoffConfig { mode: BackoffMode::Exponential, base_delay_ms: 1000, max_delay_ms: 3000, max_retries: 10, jitter_pct: 0 };
        let mut bs = BackoffStrategy::new(cfg);
        assert_eq!(bs.next_delay().unwrap(), 1000);
        assert_eq!(bs.next_delay().unwrap(), 2000);
        assert_eq!(bs.next_delay().unwrap(), 3000);
        assert_eq!(bs.next_delay().unwrap(), 3000);
    }

    #[test]
    fn linear_backoff() {
        let cfg = BackoffConfig { mode: BackoffMode::Linear, base_delay_ms: 50, max_delay_ms: 10000, max_retries: 5, jitter_pct: 0 };
        let mut bs = BackoffStrategy::new(cfg);
        assert_eq!(bs.next_delay().unwrap(), 50);
        assert_eq!(bs.next_delay().unwrap(), 100);
        assert_eq!(bs.next_delay().unwrap(), 150);
    }

    #[test]
    fn fixed_backoff() {
        let cfg = BackoffConfig { mode: BackoffMode::Fixed, base_delay_ms: 200, max_delay_ms: 10000, max_retries: 5, jitter_pct: 0 };
        let mut bs = BackoffStrategy::new(cfg);
        assert_eq!(bs.next_delay().unwrap(), 200);
        assert_eq!(bs.next_delay().unwrap(), 200);
        assert_eq!(bs.next_delay().unwrap(), 200);
    }

    #[test]
    fn max_retries() {
        let cfg = BackoffConfig { mode: BackoffMode::Fixed, base_delay_ms: 10, max_delay_ms: 100, max_retries: 2, jitter_pct: 0 };
        let mut bs = BackoffStrategy::new(cfg);
        bs.next_delay().unwrap();
        bs.next_delay().unwrap();
        let err = bs.next_delay().unwrap_err();
        assert!(matches!(err, BackoffError::MaxRetriesExceeded { max: 2, .. }));
    }

    #[test]
    fn already_succeeded() {
        let mut bs = BackoffStrategy::new(exp_config());
        bs.mark_success();
        let err = bs.next_delay().unwrap_err();
        assert!(matches!(err, BackoffError::AlreadySucceeded));
    }

    #[test]
    fn peek_delay() {
        let bs = BackoffStrategy::new(exp_config());
        assert_eq!(bs.peek_delay(), 100);
    }

    #[test]
    fn remaining() {
        let mut bs = BackoffStrategy::new(exp_config());
        assert_eq!(bs.remaining(), 5);
        bs.next_delay().unwrap();
        assert_eq!(bs.remaining(), 4);
    }

    #[test]
    fn total_wait() {
        let mut bs = BackoffStrategy::new(exp_config());
        bs.next_delay().unwrap();
        bs.next_delay().unwrap();
        assert_eq!(bs.total_wait_ms(), 300);
    }

    #[test]
    fn reset() {
        let mut bs = BackoffStrategy::new(exp_config());
        bs.next_delay().unwrap();
        bs.next_delay().unwrap();
        bs.reset();
        assert_eq!(bs.attempt(), 0);
        assert_eq!(bs.total_wait_ms(), 0);
        assert!(!bs.succeeded());
    }

    #[test]
    fn is_exhausted() {
        let cfg = BackoffConfig { mode: BackoffMode::Fixed, base_delay_ms: 10, max_delay_ms: 100, max_retries: 2, jitter_pct: 0 };
        let mut bs = BackoffStrategy::new(cfg);
        assert!(!bs.is_exhausted());
        bs.next_delay().unwrap();
        bs.next_delay().unwrap();
        assert!(bs.is_exhausted());
    }

    #[test]
    fn error_display() {
        assert!(BackoffError::MaxRetriesExceeded { attempts: 3, max: 3 }.to_string().contains("3"));
    }
}

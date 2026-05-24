#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThrottleError {
    WouldBlock { available: u64, requested: u64 },
    InvalidConfig { reason: String },
}

impl std::fmt::Display for ThrottleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThrottleError::WouldBlock { available, requested } => {
                write!(f, "would block: {available}/{requested}")
            }
            ThrottleError::InvalidConfig { reason } => {
                write!(f, "invalid config: {reason}")
            }
        }
    }
}

impl std::error::Error for ThrottleError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThrottleConfig {
    pub capacity: u64,
    pub refill_rate: u64,
    pub refill_period_us: u64,
}

impl ThrottleConfig {
    pub fn new(capacity: u64, refill_rate: u64, refill_period_us: u64) -> Self {
        Self {
            capacity,
            refill_rate,
            refill_period_us,
        }
    }

    pub fn validate(&self) -> Result<(), ThrottleError> {
        if self.capacity == 0 {
            return Err(ThrottleError::InvalidConfig {
                reason: "capacity must be > 0".into(),
            });
        }
        if self.refill_period_us == 0 {
            return Err(ThrottleError::InvalidConfig {
                reason: "refill_period must be > 0".into(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ThrottleController {
    config: ThrottleConfig,
    tokens: u64,
    total_acquired: u64,
    total_rejected: u64,
    total_refilled: u64,
}

impl ThrottleController {
    pub fn new(config: ThrottleConfig) -> Self {
        Self {
            tokens: config.capacity,
            config,
            total_acquired: 0,
            total_rejected: 0,
            total_refilled: 0,
        }
    }

    pub fn tokens(&self) -> u64 {
        self.tokens
    }

    pub fn capacity(&self) -> u64 {
        self.config.capacity
    }

    pub fn config(&self) -> &ThrottleConfig {
        &self.config
    }

    pub fn try_acquire(&mut self, count: u64) -> Result<(), ThrottleError> {
        if count <= self.tokens {
            self.tokens -= count;
            self.total_acquired += count;
            Ok(())
        } else {
            self.total_rejected += 1;
            Err(ThrottleError::WouldBlock {
                available: self.tokens,
                requested: count,
            })
        }
    }

    pub fn acquire(&mut self, count: u64) -> u64 {
        let actual = count.min(self.tokens);
        self.tokens -= actual;
        self.total_acquired += actual;
        actual
    }

    pub fn refill(&mut self) -> u64 {
        let added = self.config.refill_rate.min(self.config.capacity - self.tokens);
        self.tokens += added;
        self.total_refilled += added;
        added
    }

    pub fn refill_for_duration(&mut self, elapsed_us: u64) -> u64 {
        let periods = elapsed_us / self.config.refill_period_us;
        let mut total_added = 0u64;
        for _ in 0..periods {
            total_added += self.refill();
        }
        total_added
    }

    pub fn force_set(&mut self, tokens: u64) {
        self.tokens = tokens.min(self.config.capacity);
    }

    pub fn reset(&mut self) {
        self.tokens = self.config.capacity;
        self.total_acquired = 0;
        self.total_rejected = 0;
        self.total_refilled = 0;
    }

    pub fn total_acquired(&self) -> u64 {
        self.total_acquired
    }

    pub fn total_rejected(&self) -> u64 {
        self.total_rejected
    }

    pub fn total_refilled(&self) -> u64 {
        self.total_refilled
    }

    pub fn utilization(&self) -> f64 {
        if self.config.capacity == 0 {
            return 0.0;
        }
        1.0 - (self.tokens as f64 / self.config.capacity as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_validate_ok() {
        let c = ThrottleConfig::new(100, 10, 1000);
        assert!(c.validate().is_ok());
    }

    #[test]
    fn config_validate_zero_capacity() {
        let c = ThrottleConfig::new(0, 10, 1000);
        assert!(c.validate().is_err());
    }

    #[test]
    fn config_validate_zero_period() {
        let c = ThrottleConfig::new(100, 10, 0);
        assert!(c.validate().is_err());
    }

    #[test]
    fn new_starts_full() {
        let tc = ThrottleController::new(ThrottleConfig::new(100, 10, 1000));
        assert_eq!(tc.tokens(), 100);
        assert_eq!(tc.capacity(), 100);
    }

    #[test]
    fn try_acquire_ok() {
        let mut tc = ThrottleController::new(ThrottleConfig::new(100, 10, 1000));
        tc.try_acquire(30).unwrap();
        assert_eq!(tc.tokens(), 70);
        assert_eq!(tc.total_acquired(), 30);
    }

    #[test]
    fn try_acquire_blocked() {
        let mut tc = ThrottleController::new(ThrottleConfig::new(10, 1, 1000));
        let err = tc.try_acquire(20).unwrap_err();
        assert!(matches!(err, ThrottleError::WouldBlock { .. }));
        assert_eq!(tc.total_rejected(), 1);
    }

    #[test]
    fn acquire_partial() {
        let mut tc = ThrottleController::new(ThrottleConfig::new(10, 1, 1000));
        let got = tc.acquire(50);
        assert_eq!(got, 10);
        assert_eq!(tc.tokens(), 0);
    }

    #[test]
    fn refill_caps_at_capacity() {
        let mut tc = ThrottleController::new(ThrottleConfig::new(100, 50, 1000));
        tc.try_acquire(80).unwrap();
        let added = tc.refill();
        assert_eq!(added, 50);
        assert_eq!(tc.tokens(), 70);
        let added2 = tc.refill();
        assert_eq!(added2, 30);
        assert_eq!(tc.tokens(), 100);
        let added3 = tc.refill();
        assert_eq!(added3, 0);
        assert_eq!(tc.tokens(), 100);
    }

    #[test]
    fn refill_for_duration() {
        let mut tc = ThrottleController::new(ThrottleConfig::new(100, 10, 1000));
        tc.try_acquire(50).unwrap();
        let added = tc.refill_for_duration(5000);
        assert_eq!(added, 50);
        assert_eq!(tc.tokens(), 100);
    }

    #[test]
    fn force_set() {
        let mut tc = ThrottleController::new(ThrottleConfig::new(100, 10, 1000));
        tc.force_set(999);
        assert_eq!(tc.tokens(), 100);
        tc.force_set(50);
        assert_eq!(tc.tokens(), 50);
    }

    #[test]
    fn reset() {
        let mut tc = ThrottleController::new(ThrottleConfig::new(100, 10, 1000));
        tc.try_acquire(50).unwrap();
        tc.reset();
        assert_eq!(tc.tokens(), 100);
        assert_eq!(tc.total_acquired(), 0);
    }

    #[test]
    fn utilization() {
        let mut tc = ThrottleController::new(ThrottleConfig::new(100, 10, 1000));
        assert_eq!(tc.utilization(), 0.0);
        tc.try_acquire(50).unwrap();
        assert!((tc.utilization() - 0.5).abs() < 0.01);
    }

    #[test]
    fn error_display() {
        let e = ThrottleError::WouldBlock { available: 5, requested: 10 };
        assert!(e.to_string().contains("5/10"));
        let e = ThrottleError::InvalidConfig { reason: "bad".into() };
        assert!(e.to_string().contains("bad"));
    }
}

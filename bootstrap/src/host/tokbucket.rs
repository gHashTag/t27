#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BucketError {
    WouldBlock { needed: u64, available: u64 },
    Overflow,
}

impl std::fmt::Display for BucketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BucketError::WouldBlock { needed, available } => {
                write!(f, "need {needed} tokens, have {available}")
            }
            BucketError::Overflow => write!(f, "token overflow"),
        }
    }
}

impl std::error::Error for BucketError {}

#[derive(Debug, Clone)]
pub struct TokenBucket {
    tokens: u64,
    max_tokens: u64,
    refill_rate: u64,
    refill_interval: u64,
    ticks_since_refill: u64,
    total_consumed: u64,
    total_refilled: u64,
    total_rejected: u64,
    total_burst: u64,
}

impl TokenBucket {
    pub fn new(max_tokens: u64, refill_rate: u64, refill_interval: u64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            refill_interval: refill_interval.max(1),
            ticks_since_refill: 0,
            total_consumed: 0,
            total_refilled: 0,
            total_rejected: 0,
            total_burst: 0,
        }
    }

    pub fn with_initial(mut self, initial: u64) -> Self {
        self.tokens = initial.min(self.max_tokens);
        self
    }

    pub fn tokens(&self) -> u64 {
        self.tokens
    }

    pub fn max_tokens(&self) -> u64 {
        self.max_tokens
    }

    pub fn refill_rate(&self) -> u64 {
        self.refill_rate
    }

    pub fn tick(&mut self) -> u64 {
        self.ticks_since_refill += 1;
        if self.ticks_since_refill >= self.refill_interval {
            self.refill();
            self.ticks_since_refill = 0;
        }
        self.tokens
    }

    pub fn refill(&mut self) -> u64 {
        let added = self.refill_rate.min(self.max_tokens - self.tokens);
        self.tokens += added;
        self.total_refilled += added;
        added
    }

    pub fn try_consume(&mut self, amount: u64) -> Result<u64, BucketError> {
        if amount > self.tokens {
            self.total_rejected += 1;
            return Err(BucketError::WouldBlock { needed: amount, available: self.tokens });
        }
        self.tokens -= amount;
        self.total_consumed += amount;
        if self.tokens == 0 {
            self.total_burst += 1;
        }
        Ok(self.tokens)
    }

    pub fn consume_max(&mut self, amount: u64) -> u64 {
        let actual = amount.min(self.tokens);
        self.tokens -= actual;
        self.total_consumed += actual;
        actual
    }

    pub fn force_consume(&mut self, amount: u64) -> u64 {
        let actual = amount.min(self.tokens);
        self.tokens -= actual;
        self.total_consumed += actual;
        actual
    }

    pub fn available(&self) -> u64 {
        self.tokens
    }

    pub fn is_empty(&self) -> bool {
        self.tokens == 0
    }

    pub fn is_full(&self) -> bool {
        self.tokens == self.max_tokens
    }

    pub fn utilization(&self) -> f64 {
        if self.max_tokens == 0 { 0.0 } else { self.tokens as f64 / self.max_tokens as f64 }
    }

    pub fn total_consumed(&self) -> u64 {
        self.total_consumed
    }

    pub fn total_refilled(&self) -> u64 {
        self.total_refilled
    }

    pub fn total_rejected(&self) -> u64 {
        self.total_rejected
    }

    pub fn total_burst(&self) -> u64 {
        self.total_burst
    }

    pub fn reset(&mut self) {
        self.tokens = self.max_tokens;
        self.ticks_since_refill = 0;
        self.total_consumed = 0;
        self.total_refilled = 0;
        self.total_rejected = 0;
        self.total_burst = 0;
    }

    pub fn set_tokens(&mut self, value: u64) {
        self.tokens = value.min(self.max_tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bucket() {
        let tb = TokenBucket::new(100, 10, 5);
        assert_eq!(tb.tokens(), 100);
        assert_eq!(tb.max_tokens(), 100);
        assert!(tb.is_full());
    }

    #[test]
    fn with_initial() {
        let tb = TokenBucket::new(100, 10, 5).with_initial(50);
        assert_eq!(tb.tokens(), 50);
    }

    #[test]
    fn consume_success() {
        let mut tb = TokenBucket::new(100, 10, 5);
        let remaining = tb.try_consume(30).unwrap();
        assert_eq!(remaining, 70);
        assert_eq!(tb.total_consumed(), 30);
    }

    #[test]
    fn consume_insufficient() {
        let mut tb = TokenBucket::new(100, 10, 5);
        let err = tb.try_consume(150).unwrap_err();
        assert!(matches!(err, BucketError::WouldBlock { needed: 150, available: 100 }));
        assert_eq!(tb.total_rejected(), 1);
    }

    #[test]
    fn consume_max() {
        let mut tb = TokenBucket::new(50, 10, 5);
        let actual = tb.consume_max(100);
        assert_eq!(actual, 50);
        assert_eq!(tb.tokens(), 0);
    }

    #[test]
    fn refill() {
        let mut tb = TokenBucket::new(100, 10, 5).with_initial(80);
        let added = tb.refill();
        assert_eq!(added, 10);
        assert_eq!(tb.tokens(), 90);
    }

    #[test]
    fn refill_clamped() {
        let mut tb = TokenBucket::new(100, 30, 5).with_initial(95);
        let added = tb.refill();
        assert_eq!(added, 5);
        assert_eq!(tb.tokens(), 100);
    }

    #[test]
    fn tick_triggers_refill() {
        let mut tb = TokenBucket::new(100, 10, 3).with_initial(50);
        tb.tick(); tb.tick();
        assert_eq!(tb.tokens(), 50);
        tb.tick();
        assert_eq!(tb.tokens(), 60);
    }

    #[test]
    fn burst_detection() {
        let mut tb = TokenBucket::new(10, 1, 1);
        tb.try_consume(10).unwrap();
        assert_eq!(tb.total_burst(), 1);
        assert!(tb.is_empty());
    }

    #[test]
    fn utilization() {
        let tb = TokenBucket::new(200, 10, 5).with_initial(50);
        assert!((tb.utilization() - 0.25).abs() < 0.01);
    }

    #[test]
    fn reset() {
        let mut tb = TokenBucket::new(100, 10, 5);
        tb.try_consume(50).unwrap();
        tb.reset();
        assert_eq!(tb.tokens(), 100);
        assert_eq!(tb.total_consumed(), 0);
    }

    #[test]
    fn set_tokens() {
        let mut tb = TokenBucket::new(100, 10, 5);
        tb.set_tokens(42);
        assert_eq!(tb.tokens(), 42);
        tb.set_tokens(999);
        assert_eq!(tb.tokens(), 100);
    }

    #[test]
    fn error_display() {
        let e = BucketError::WouldBlock { needed: 10, available: 5 };
        assert!(e.to_string().contains("10"));
    }
}

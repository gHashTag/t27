use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OpId(pub u64);

impl std::fmt::Display for OpId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "op{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct RetryRecord {
    pub op: OpId,
    pub attempts: u32,
    pub max_attempts: u32,
    pub succeeded: bool,
    pub last_backoff_us: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetError {
    BudgetExhausted { remaining: u32 },
    MaxAttemptsReached { op: OpId, attempts: u32 },
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetError::BudgetExhausted { remaining } => {
                write!(f, "budget exhausted: {remaining} left")
            }
            BudgetError::MaxAttemptsReached { op, attempts } => {
                write!(f, "{op}: max attempts {attempts}")
            }
        }
    }
}

impl std::error::Error for BudgetError {}

#[derive(Debug, Clone)]
pub struct RetryBudget {
    global_budget: u32,
    initial_budget: u32,
    default_max: u32,
    base_backoff_us: u64,
    records: BTreeMap<u64, RetryRecord>,
    next_id: u64,
    total_attempts: u64,
    total_successes: u64,
    total_failures: u64,
}

impl RetryBudget {
    pub fn new(global_budget: u32, default_max: u32, base_backoff_us: u64) -> Self {
        Self {
            global_budget,
            initial_budget: global_budget,
            default_max,
            base_backoff_us,
            records: BTreeMap::new(),
            next_id: 1,
            total_attempts: 0,
            total_successes: 0,
            total_failures: 0,
        }
    }

    pub fn register(&mut self) -> OpId {
        let id = OpId(self.next_id);
        self.next_id += 1;
        self.records.insert(id.0, RetryRecord {
            op: id,
            attempts: 0,
            max_attempts: self.default_max,
            succeeded: false,
            last_backoff_us: 0,
        });
        id
    }

    pub fn register_with_max(&mut self, max_attempts: u32) -> OpId {
        let id = OpId(self.next_id);
        self.next_id += 1;
        self.records.insert(id.0, RetryRecord {
            op: id,
            attempts: 0,
            max_attempts,
            succeeded: false,
            last_backoff_us: 0,
        });
        id
    }

    pub fn try_retry(&mut self, op: OpId) -> Result<u64, BudgetError> {
        let rec = self.records.get_mut(&op.0)
            .ok_or(BudgetError::MaxAttemptsReached { op, attempts: 0 })?;
        if rec.succeeded {
            return Ok(0);
        }
        if rec.attempts >= rec.max_attempts {
            self.total_failures += 1;
            return Err(BudgetError::MaxAttemptsReached { op, attempts: rec.attempts });
        }
        if self.global_budget == 0 {
            return Err(BudgetError::BudgetExhausted { remaining: 0 });
        }
        rec.attempts += 1;
        self.global_budget -= 1;
        self.total_attempts += 1;
        let backoff = self.base_backoff_us * (1 << rec.attempts.min(10));
        rec.last_backoff_us = backoff;
        Ok(backoff)
    }

    pub fn succeed(&mut self, op: OpId) -> bool {
        if let Some(rec) = self.records.get_mut(&op.0) {
            rec.succeeded = true;
            self.total_successes += 1;
            true
        } else {
            false
        }
    }

    pub fn get(&self, op: OpId) -> Option<&RetryRecord> {
        self.records.get(&op.0)
    }

    pub fn budget_remaining(&self) -> u32 {
        self.global_budget
    }

    pub fn budget_used(&self) -> u32 {
        self.initial_budget - self.global_budget
    }

    pub fn active_count(&self) -> usize {
        self.records.values().filter(|r| !r.succeeded && r.attempts < r.max_attempts).count()
    }

    pub fn total_attempts(&self) -> u64 {
        self.total_attempts
    }

    pub fn total_successes(&self) -> u64 {
        self.total_successes
    }

    pub fn total_failures(&self) -> u64 {
        self.total_failures
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_successes + self.total_failures;
        if total == 0 { 0.0 } else { self.total_successes as f64 / total as f64 }
    }

    pub fn reset(&mut self) {
        self.global_budget = self.initial_budget;
        self.records.clear();
        self.total_attempts = 0;
        self.total_successes = 0;
        self.total_failures = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_display() {
        assert_eq!(OpId(42).to_string(), "op42");
    }

    #[test]
    fn register_and_get() {
        let mut rb = RetryBudget::new(10, 3, 100);
        let op = rb.register();
        let rec = rb.get(op).unwrap();
        assert_eq!(rec.attempts, 0);
        assert!(!rec.succeeded);
    }

    #[test]
    fn try_retry_increments() {
        let mut rb = RetryBudget::new(10, 3, 100);
        let op = rb.register();
        let backoff = rb.try_retry(op).unwrap();
        assert_eq!(rb.get(op).unwrap().attempts, 1);
        assert!(backoff > 0);
        assert_eq!(rb.budget_remaining(), 9);
    }

    #[test]
    fn try_retry_exponential_backoff() {
        let mut rb = RetryBudget::new(10, 5, 100);
        let op = rb.register();
        let b1 = rb.try_retry(op).unwrap();
        let b2 = rb.try_retry(op).unwrap();
        assert!(b2 > b1);
    }

    #[test]
    fn max_attempts_reached() {
        let mut rb = RetryBudget::new(10, 2, 100);
        let op = rb.register();
        rb.try_retry(op).unwrap();
        rb.try_retry(op).unwrap();
        let err = rb.try_retry(op).unwrap_err();
        assert!(matches!(err, BudgetError::MaxAttemptsReached { .. }));
        assert_eq!(rb.total_failures(), 1);
    }

    #[test]
    fn budget_exhausted() {
        let mut rb = RetryBudget::new(1, 5, 100);
        let op1 = rb.register();
        let op2 = rb.register();
        rb.try_retry(op1).unwrap();
        let err = rb.try_retry(op2).unwrap_err();
        assert!(matches!(err, BudgetError::BudgetExhausted { .. }));
    }

    #[test]
    fn succeed() {
        let mut rb = RetryBudget::new(10, 3, 100);
        let op = rb.register();
        rb.try_retry(op).unwrap();
        assert!(rb.succeed(op));
        assert!(rb.get(op).unwrap().succeeded);
        assert_eq!(rb.total_successes(), 1);
    }

    #[test]
    fn succeed_unknown() {
        let mut rb = RetryBudget::new(10, 3, 100);
        assert!(!rb.succeed(OpId(999)));
    }

    #[test]
    fn active_count() {
        let mut rb = RetryBudget::new(10, 3, 100);
        let op1 = rb.register();
        let op2 = rb.register();
        rb.try_retry(op1).unwrap();
        rb.succeed(op2);
        assert_eq!(rb.active_count(), 1);
    }

    #[test]
    fn success_rate() {
        let mut rb = RetryBudget::new(10, 3, 100);
        let op1 = rb.register();
        let op2 = rb.register();
        rb.succeed(op1);
        rb.try_retry(op2).unwrap();
        rb.try_retry(op2).unwrap();
        rb.try_retry(op2).unwrap();
        let _ = rb.try_retry(op2);
        assert!((rb.success_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn reset() {
        let mut rb = RetryBudget::new(10, 3, 100);
        let op = rb.register();
        rb.try_retry(op).unwrap();
        rb.reset();
        assert_eq!(rb.budget_remaining(), 10);
        assert_eq!(rb.total_attempts(), 0);
    }

    #[test]
    fn error_display() {
        assert!(BudgetError::BudgetExhausted { remaining: 0 }.to_string().contains("exhausted"));
        assert!(BudgetError::MaxAttemptsReached { op: OpId(1), attempts: 3 }.to_string().contains("op1"));
    }
}

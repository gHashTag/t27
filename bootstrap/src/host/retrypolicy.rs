use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RetryError {
    BudgetExhausted { max: u32 },
    MaxAttempts { attempts: u32, max: u32 },
    NotRetryable { attempt: u32 },
}

impl std::fmt::Display for RetryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryError::BudgetExhausted { max } => write!(f, "budget exhausted ({max})"),
            RetryError::MaxAttempts { attempts, max } => write!(f, "{attempts}/{max} attempts"),
            RetryError::NotRetryable { attempt } => write!(f, "attempt {attempt} not retryable"),
        }
    }
}

impl std::error::Error for RetryError {}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: u64,
    pub jitter_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self { max_attempts: 3, base_delay_ms: 100, max_delay_ms: 30_000, multiplier: 2, jitter_ms: 50 }
    }
}

struct RetryTracker {
    id: u64,
    config: RetryConfig,
    attempts: u32,
    total_delay_ms: u64,
    succeeded: bool,
}

pub struct RetryPolicy {
    trackers: BTreeMap<u64, RetryTracker>,
    budget: u32,
    budget_used: u32,
    next_id: u64,
    total_started: u64,
    total_succeeded: u64,
    total_failed: u64,
}

impl RetryPolicy {
    pub fn new(budget: u32) -> Self {
        Self { trackers: BTreeMap::new(), budget, budget_used: 0, next_id: 1, total_started: 0, total_succeeded: 0, total_failed: 0 }
    }

    pub fn start(&mut self, config: RetryConfig) -> Result<u64, RetryError> {
        if self.budget_used >= self.budget { return Err(RetryError::BudgetExhausted { max: self.budget }); }
        let id = self.next_id;
        self.next_id += 1;
        self.trackers.insert(id, RetryTracker { id, config, attempts: 0, total_delay_ms: 0, succeeded: false });
        self.total_started += 1;
        Ok(id)
    }

    pub fn next_delay(&mut self, id: u64) -> Result<u64, RetryError> {
        let t = self.trackers.get_mut(&id).ok_or(RetryError::NotRetryable { attempt: 0 })?;
        if t.succeeded { return Err(RetryError::NotRetryable { attempt: t.attempts }); }
        t.attempts += 1;
        if t.attempts > t.config.max_attempts { return Err(RetryError::MaxAttempts { attempts: t.attempts, max: t.config.max_attempts }); }
        self.budget_used += 1;
        let exp = (t.config.multiplier).pow(t.attempts - 1);
        let base = t.config.base_delay_ms * exp;
        let delay = base.min(t.config.max_delay_ms) + t.config.jitter_ms;
        t.total_delay_ms += delay;
        Ok(delay)
    }

    pub fn mark_success(&mut self, id: u64) -> Result<u32, RetryError> {
        let t = self.trackers.get_mut(&id).ok_or(RetryError::NotRetryable { attempt: 0 })?;
        t.succeeded = true;
        self.total_succeeded += 1;
        Ok(t.attempts)
    }

    pub fn mark_fail(&mut self, id: u64) -> u32 {
        if let Some(t) = self.trackers.get_mut(&id) {
            self.total_failed += 1;
            t.attempts
        } else { 0 }
    }

    pub fn attempts(&self, id: u64) -> Option<u32> { self.trackers.get(&id).map(|t| t.attempts) }
    pub fn total_delay(&self, id: u64) -> Option<u64> { self.trackers.get(&id).map(|t| t.total_delay_ms) }
    pub fn succeeded(&self, id: u64) -> bool { self.trackers.get(&id).map(|t| t.succeeded).unwrap_or(false) }
    pub fn budget_remaining(&self) -> u32 { self.budget.saturating_sub(self.budget_used) }
    pub fn budget_used(&self) -> u32 { self.budget_used }
    pub fn total_started(&self) -> u64 { self.total_started }
    pub fn total_succeeded(&self) -> u64 { self.total_succeeded }
    pub fn total_failed(&self) -> u64 { self.total_failed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_policy() { let p = RetryPolicy::new(100); assert_eq!(p.budget_remaining(), 100); }

    #[test]
    fn start_next_delay() {
        let mut p = RetryPolicy::new(100);
        let id = p.start(RetryConfig::default()).unwrap();
        let d = p.next_delay(id).unwrap();
        assert!(d >= 100);
    }

    #[test]
    fn exponential_backoff() {
        let mut p = RetryPolicy::new(100);
        let id = p.start(RetryConfig { multiplier: 2, ..Default::default() }).unwrap();
        let d1 = p.next_delay(id).unwrap();
        let d2 = p.next_delay(id).unwrap();
        assert!(d2 > d1, "d2={d2} d1={d1}");
    }

    #[test]
    fn max_delay_cap() {
        let mut p = RetryPolicy::new(100);
        let cfg = RetryConfig { max_delay_ms: 200, multiplier: 10, ..Default::default() };
        let id = p.start(cfg).unwrap();
        let d = p.next_delay(id).unwrap();
        assert!(d <= 250);
    }

    #[test]
    fn mark_success() {
        let mut p = RetryPolicy::new(100);
        let id = p.start(RetryConfig::default()).unwrap();
        p.next_delay(id).unwrap();
        let attempts = p.mark_success(id).unwrap();
        assert_eq!(attempts, 1);
        assert!(p.succeeded(id));
    }

    #[test]
    fn max_attempts() {
        let mut p = RetryPolicy::new(100);
        let id = p.start(RetryConfig { max_attempts: 2, ..Default::default() }).unwrap();
        p.next_delay(id).unwrap();
        p.next_delay(id).unwrap();
        let err = p.next_delay(id).unwrap_err();
        assert!(matches!(err, RetryError::MaxAttempts { .. }));
    }

    #[test]
    fn budget_exhausted() {
        let mut p = RetryPolicy::new(1);
        let id = p.start(RetryConfig::default()).unwrap();
        p.next_delay(id).unwrap();
        let err = p.start(RetryConfig::default()).unwrap_err();
        assert!(matches!(err, RetryError::BudgetExhausted { .. }));
    }

    #[test]
    fn total_delay() {
        let mut p = RetryPolicy::new(100);
        let id = p.start(RetryConfig { jitter_ms: 0, ..Default::default() }).unwrap();
        p.next_delay(id).unwrap();
        p.next_delay(id).unwrap();
        let d = p.total_delay(id).unwrap();
        assert!(d > 0);
    }

    #[test]
    fn stats() {
        let mut p = RetryPolicy::new(100);
        let id = p.start(RetryConfig::default()).unwrap();
        p.next_delay(id).unwrap();
        p.mark_success(id).unwrap();
        assert_eq!(p.total_started(), 1);
        assert_eq!(p.total_succeeded(), 1);
    }

    #[test]
    fn mark_fail() {
        let mut p = RetryPolicy::new(100);
        let id = p.start(RetryConfig::default()).unwrap();
        p.next_delay(id).unwrap();
        let a = p.mark_fail(id);
        assert_eq!(a, 1);
        assert_eq!(p.total_failed(), 1);
    }

    #[test]
    fn error_display() { assert!(RetryError::BudgetExhausted { max: 5 }.to_string().contains("5")); }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CbState { Closed, Open, HalfOpen }

#[derive(Debug, Clone, PartialEq)]
pub enum HxError {
    CircuitOpen { key: u64 },
    KeyNotFound { key: u64 },
}

impl std::fmt::Display for HxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HxError::CircuitOpen { key } => write!(f, "circuit open for key {key}"),
            HxError::KeyNotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for HxError {}

struct Circuit {
    key: u64,
    state: CbState,
    failure_threshold: u64,
    success_threshold: u64,
    timeout: u64,
    failures: u64,
    successes: u64,
    opened_at: Option<u64>,
    total_successes: u64,
    total_failures: u64,
    total_rejections: u64,
    total_state_changes: u64,
}

pub struct Hystrix {
    circuits: BTreeMap<u64, Circuit>,
    total_successes: u64,
    total_failures: u64,
    total_rejections: u64,
}

impl Hystrix {
    pub fn new() -> Self { Self { circuits: BTreeMap::new(), total_successes: 0, total_failures: 0, total_rejections: 0 } }

    pub fn register(&mut self, key: u64, failure_threshold: u64, success_threshold: u64, timeout: u64) {
        self.circuits.insert(key, Circuit {
            key, state: CbState::Closed, failure_threshold, success_threshold, timeout,
            failures: 0, successes: 0, opened_at: None,
            total_successes: 0, total_failures: 0, total_rejections: 0, total_state_changes: 0,
        });
    }

    pub fn allow(&mut self, key: u64, now: u64) -> Result<(), HxError> {
        let c = self.circuits.get_mut(&key).ok_or(HxError::KeyNotFound { key })?;
        match c.state {
            CbState::Closed => Ok(()),
            CbState::Open => {
                if let Some(opened) = c.opened_at {
                    if now >= opened + c.timeout {
                        c.state = CbState::HalfOpen;
                        c.successes = 0;
                        c.total_state_changes += 1;
                        Ok(())
                    } else {
                        c.total_rejections += 1;
                        self.total_rejections += 1;
                        Err(HxError::CircuitOpen { key })
                    }
                } else {
                    c.total_rejections += 1;
                    self.total_rejections += 1;
                    Err(HxError::CircuitOpen { key })
                }
            }
            CbState::HalfOpen => Ok(()),
        }
    }

    pub fn record_success(&mut self, key: u64) {
        let c = self.circuits.get_mut(&key).unwrap();
        c.total_successes += 1;
        self.total_successes += 1;
        match c.state {
            CbState::HalfOpen => {
                c.successes += 1;
                if c.successes >= c.success_threshold {
                    c.state = CbState::Closed;
                    c.failures = 0;
                    c.successes = 0;
                    c.opened_at = None;
                    c.total_state_changes += 1;
                }
            }
            CbState::Closed => { c.failures = c.failures.saturating_sub(1); }
            _ => {}
        }
    }

    pub fn record_failure(&mut self, key: u64, now: u64) {
        let c = self.circuits.get_mut(&key).unwrap();
        c.total_failures += 1;
        self.total_failures += 1;
        match c.state {
            CbState::Closed => {
                c.failures += 1;
                if c.failures >= c.failure_threshold {
                    c.state = CbState::Open;
                    c.opened_at = Some(now);
                    c.total_state_changes += 1;
                }
            }
            CbState::HalfOpen => {
                c.state = CbState::Open;
                c.opened_at = Some(now);
                c.successes = 0;
                c.total_state_changes += 1;
            }
            _ => {}
        }
    }

    pub fn state(&self, key: u64) -> Option<&CbState> { self.circuits.get(&key).map(|c| &c.state) }
    pub fn circuit_count(&self) -> usize { self.circuits.len() }
    pub fn total_successes(&self) -> u64 { self.total_successes }
    pub fn total_failures(&self) -> u64 { self.total_failures }
    pub fn total_rejections(&self) -> u64 { self.total_rejections }
}

impl Default for Hystrix {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cb() { let h = Hystrix::new(); assert_eq!(h.circuit_count(), 0); }

    #[test]
    fn closed_allows() {
        let mut h = Hystrix::new();
        h.register(1, 3, 2, 100);
        assert!(h.allow(1, 0).is_ok());
    }

    #[test]
    fn trips_on_failures() {
        let mut h = Hystrix::new();
        h.register(1, 3, 2, 100);
        for _ in 0..3 { h.record_failure(1, 0); }
        assert_eq!(h.state(1), Some(&CbState::Open));
        assert!(h.allow(1, 0).is_err());
    }

    #[test]
    fn half_open_after_timeout() {
        let mut h = Hystrix::new();
        h.register(1, 2, 2, 50);
        h.record_failure(1, 0); h.record_failure(1, 0);
        assert!(h.allow(1, 49).is_err());
        assert!(h.allow(1, 50).is_ok());
        assert_eq!(h.state(1), Some(&CbState::HalfOpen));
    }

    #[test]
    fn recovery() {
        let mut h = Hystrix::new();
        h.register(1, 2, 2, 50);
        h.record_failure(1, 0); h.record_failure(1, 0);
        h.allow(1, 50).unwrap();
        h.record_success(1); h.record_success(1);
        assert_eq!(h.state(1), Some(&CbState::Closed));
    }

    #[test]
    fn half_open_failure_reopens() {
        let mut h = Hystrix::new();
        h.register(1, 2, 2, 50);
        h.record_failure(1, 0); h.record_failure(1, 0);
        h.allow(1, 50).unwrap();
        h.record_failure(1, 60);
        assert_eq!(h.state(1), Some(&CbState::Open));
    }

    #[test]
    fn success_decrements() {
        let mut h = Hystrix::new();
        h.register(1, 3, 2, 100);
        h.record_failure(1, 0); h.record_failure(1, 0);
        h.record_success(1);
        h.record_failure(1, 0);
        assert_eq!(h.state(1), Some(&CbState::Closed));
    }

    #[test]
    fn not_found() {
        let mut h = Hystrix::new();
        let err = h.allow(99, 0).unwrap_err();
        assert!(matches!(err, HxError::KeyNotFound { .. }));
    }

    #[test]
    fn rejection_stats() {
        let mut h = Hystrix::new();
        h.register(1, 1, 1, 100);
        h.record_failure(1, 0);
        h.allow(1, 10).unwrap_err();
        assert_eq!(h.total_rejections(), 1);
    }

    #[test]
    fn global_stats() {
        let mut h = Hystrix::new();
        h.register(1, 5, 2, 100);
        h.record_success(1); h.record_failure(1, 0);
        assert_eq!(h.total_successes(), 1);
        assert_eq!(h.total_failures(), 1);
    }

    #[test]
    fn error_display() { assert!(HxError::CircuitOpen { key: 1 }.to_string().contains("open")); }
}

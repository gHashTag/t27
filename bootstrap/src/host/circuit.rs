use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitError {
    Open { failures: u64 },
    NotFound { id: u64 },
    Exists { id: u64 },
}

impl std::fmt::Display for CircuitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitError::Open { failures } => write!(f, "circuit open ({failures} failures)"),
            CircuitError::NotFound { id } => write!(f, "circuit {id} not found"),
            CircuitError::Exists { id } => write!(f, "circuit {id} exists"),
        }
    }
}

impl std::error::Error for CircuitError {}

struct Circuit {
    id: u64,
    state: CircuitState,
    failure_threshold: u64,
    failure_count: u64,
    success_count: u64,
    total_successes: u64,
    total_failures: u64,
    half_open_max: u64,
    half_open_successes: u64,
    opened_at: Option<Instant>,
    cooldown: Duration,
}

impl Circuit {
    fn new(id: u64, failure_threshold: u64, cooldown: Duration, half_open_max: u64) -> Self {
        Self { id, state: CircuitState::Closed, failure_threshold, failure_count: 0, success_count: 0, total_successes: 0, total_failures: 0, half_open_max, half_open_successes: 0, opened_at: None, cooldown }
    }
}

#[derive(Debug, Clone)]
pub struct CircuitInfo {
    pub id: u64,
    pub state: CircuitState,
    pub failure_count: u64,
    pub success_count: u64,
    pub total_successes: u64,
    pub total_failures: u64,
}

pub struct CircuitBreaker {
    circuits: Vec<Circuit>,
}

impl CircuitBreaker {
    pub fn new() -> Self { Self { circuits: Vec::new() } }

    pub fn register(&mut self, id: u64, failure_threshold: u64, cooldown: Duration, half_open_max: u64) -> Result<(), CircuitError> {
        if self.circuits.iter().any(|c| c.id == id) { return Err(CircuitError::Exists { id }); }
        self.circuits.push(Circuit::new(id, failure_threshold, cooldown, half_open_max));
        Ok(())
    }

    fn find(&self, id: u64) -> Result<usize, CircuitError> {
        self.circuits.iter().position(|c| c.id == id).ok_or(CircuitError::NotFound { id })
    }

    pub fn allow(&mut self, id: u64) -> Result<CircuitState, CircuitError> {
        let idx = self.circuits.iter().position(|c| c.id == id).ok_or(CircuitError::NotFound { id })?;
        let c = &mut self.circuits[idx];
        match c.state {
            CircuitState::Closed => Ok(CircuitState::Closed),
            CircuitState::Open => {
                if let Some(opened) = c.opened_at {
                    if opened.elapsed() >= c.cooldown {
                        c.state = CircuitState::HalfOpen;
                        c.half_open_successes = 0;
                        return Ok(CircuitState::HalfOpen);
                    }
                }
                Err(CircuitError::Open { failures: c.failure_count })
            }
            CircuitState::HalfOpen => Ok(CircuitState::HalfOpen),
        }
    }

    pub fn record_success(&mut self, id: u64) -> Result<CircuitState, CircuitError> {
        let idx = self.circuits.iter().position(|c| c.id == id).ok_or(CircuitError::NotFound { id })?;
        let c = &mut self.circuits[idx];
        c.success_count += 1;
        c.total_successes += 1;
        match c.state {
            CircuitState::HalfOpen => {
                c.half_open_successes += 1;
                if c.half_open_successes >= c.half_open_max {
                    c.state = CircuitState::Closed;
                    c.failure_count = 0;
                    c.opened_at = None;
                }
            }
            CircuitState::Closed => { c.failure_count = 0; }
            _ => {}
        }
        Ok(c.state)
    }

    pub fn record_failure(&mut self, id: u64) -> Result<CircuitState, CircuitError> {
        let idx = self.circuits.iter().position(|c| c.id == id).ok_or(CircuitError::NotFound { id })?;
        let c = &mut self.circuits[idx];
        c.failure_count += 1;
        c.total_failures += 1;
        match c.state {
            CircuitState::Closed => {
                if c.failure_count >= c.failure_threshold {
                    c.state = CircuitState::Open;
                    c.opened_at = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                c.state = CircuitState::Open;
                c.opened_at = Some(Instant::now());
                c.half_open_successes = 0;
            }
            _ => {}
        }
        Ok(c.state)
    }

    pub fn state(&self, id: u64) -> Result<CircuitState, CircuitError> {
        Ok(self.circuits[self.find(id)?].state)
    }

    pub fn info(&self, id: u64) -> Result<CircuitInfo, CircuitError> {
        let c = &self.circuits[self.find(id)?];
        Ok(CircuitInfo { id: c.id, state: c.state, failure_count: c.failure_count, success_count: c.success_count, total_successes: c.total_successes, total_failures: c.total_failures })
    }

    pub fn reset(&mut self, id: u64) -> Result<(), CircuitError> {
        let idx = self.circuits.iter().position(|c| c.id == id).ok_or(CircuitError::NotFound { id })?;
        let c = &mut self.circuits[idx];
        c.state = CircuitState::Closed;
        c.failure_count = 0;
        c.success_count = 0;
        c.opened_at = None;
        Ok(())
    }

    pub fn circuit_count(&self) -> usize { self.circuits.len() }
}

impl Default for CircuitBreaker {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn new_breaker() {
        let cb = CircuitBreaker::new();
        assert_eq!(cb.circuit_count(), 0);
    }

    #[test]
    fn register() {
        let mut cb = CircuitBreaker::new();
        cb.register(1, 3, Duration::from_secs(5), 2).unwrap();
        assert_eq!(cb.circuit_count(), 1);
        assert_eq!(cb.state(1).unwrap(), CircuitState::Closed);
    }

    #[test]
    fn duplicate() {
        let mut cb = CircuitBreaker::new();
        cb.register(1, 3, Duration::from_secs(5), 2).unwrap();
        let err = cb.register(1, 3, Duration::from_secs(5), 2).unwrap_err();
        assert!(matches!(err, CircuitError::Exists { .. }));
    }

    #[test]
    fn success_stays_closed() {
        let mut cb = CircuitBreaker::new();
        cb.register(1, 3, Duration::from_secs(5), 2).unwrap();
        cb.record_success(1).unwrap();
        assert_eq!(cb.state(1).unwrap(), CircuitState::Closed);
    }

    #[test]
    fn failures_open() {
        let mut cb = CircuitBreaker::new();
        cb.register(1, 3, Duration::from_secs(60), 2).unwrap();
        cb.record_failure(1).unwrap();
        cb.record_failure(1).unwrap();
        cb.record_failure(1).unwrap();
        assert_eq!(cb.state(1).unwrap(), CircuitState::Open);
    }

    #[test]
    fn open_blocks() {
        let mut cb = CircuitBreaker::new();
        cb.register(1, 1, Duration::from_secs(60), 2).unwrap();
        cb.record_failure(1).unwrap();
        let err = cb.allow(1).unwrap_err();
        assert!(matches!(err, CircuitError::Open { .. }));
    }

    #[test]
    fn reset() {
        let mut cb = CircuitBreaker::new();
        cb.register(1, 1, Duration::from_secs(60), 2).unwrap();
        cb.record_failure(1).unwrap();
        cb.reset(1).unwrap();
        assert_eq!(cb.state(1).unwrap(), CircuitState::Closed);
    }

    #[test]
    fn info() {
        let mut cb = CircuitBreaker::new();
        cb.register(1, 3, Duration::from_secs(5), 2).unwrap();
        cb.record_success(1).unwrap();
        cb.record_failure(1).unwrap();
        let info = cb.info(1).unwrap();
        assert_eq!(info.total_successes, 1);
        assert_eq!(info.total_failures, 1);
    }

    #[test]
    fn not_found() {
        let cb = CircuitBreaker::new();
        let err = cb.state(99).unwrap_err();
        assert!(matches!(err, CircuitError::NotFound { .. }));
    }

    #[test]
    fn half_open_after_cooldown() {
        let mut cb = CircuitBreaker::new();
        cb.register(1, 1, Duration::from_millis(1), 1).unwrap();
        cb.record_failure(1).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        let state = cb.allow(1).unwrap();
        assert_eq!(state, CircuitState::HalfOpen);
    }

    #[test]
    fn half_open_success_closes() {
        let mut cb = CircuitBreaker::new();
        cb.register(1, 1, Duration::from_millis(1), 1).unwrap();
        cb.record_failure(1).unwrap();
        std::thread::sleep(Duration::from_millis(5));
        cb.allow(1).unwrap();
        cb.record_success(1).unwrap();
        assert_eq!(cb.state(1).unwrap(), CircuitState::Closed);
    }

    #[test]
    fn error_display() {
        assert!(CircuitError::NotFound { id: 5 }.to_string().contains("5"));
    }
}

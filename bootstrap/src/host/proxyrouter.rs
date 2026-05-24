use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ProxyError {
    BackendExists { id: u64 },
    BackendNotFound { id: u64 },
    NoHealthyBackends,
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::BackendExists { id } => write!(f, "backend {id} exists"),
            ProxyError::BackendNotFound { id } => write!(f, "backend {id} not found"),
            ProxyError::NoHealthyBackends => write!(f, "no healthy backends"),
        }
    }
}

impl std::error::Error for ProxyError {}

#[derive(Debug, Clone, PartialEq)]
pub enum BackendState { Healthy, Unhealthy, CircuitOpen }

struct Backend {
    id: u64,
    weight: u32,
    state: BackendState,
    successes: u64,
    failures: u64,
    consecutive_failures: u32,
    circuit_open_until: u64,
}

pub struct ProxyRouter {
    backends: BTreeMap<u64, Backend>,
    order: Vec<u64>,
    current_tick: u64,
    failure_threshold: u32,
    recovery_ticks: u64,
    total_routed: u64,
    total_failovers: u64,
    total_circuit_opens: u64,
}

impl ProxyRouter {
    pub fn new(failure_threshold: u32, recovery_ticks: u64) -> Self {
        Self { backends: BTreeMap::new(), order: Vec::new(), current_tick: 0, failure_threshold, recovery_ticks, total_routed: 0, total_failovers: 0, total_circuit_opens: 0 }
    }

    pub fn add(&mut self, id: u64, weight: u32) -> Result<(), ProxyError> {
        if self.backends.contains_key(&id) { return Err(ProxyError::BackendExists { id }); }
        self.backends.insert(id, Backend { id, weight, state: BackendState::Healthy, successes: 0, failures: 0, consecutive_failures: 0, circuit_open_until: 0 });
        self.order.push(id);
        Ok(())
    }

    pub fn remove(&mut self, id: u64) -> Result<(), ProxyError> {
        if self.backends.remove(&id).is_none() { return Err(ProxyError::BackendNotFound { id }); }
        self.order.retain(|&x| x != id);
        Ok(())
    }

    pub fn route(&mut self) -> Result<u64, ProxyError> {
        self.current_tick += 1;
        self.try_recover();
        let healthy: Vec<u64> = self.order.iter().copied().filter(|&id| {
            matches!(self.backends.get(&id).map(|b| &b.state), Some(BackendState::Healthy))
        }).collect();
        if healthy.is_empty() { return Err(ProxyError::NoHealthyBackends); }
        let mut best = healthy[0];
        let mut best_weight = self.backends[&healthy[0]].weight;
        for &id in &healthy[1..] {
            let w = self.backends[&id].weight;
            if w > best_weight { best = id; best_weight = w; }
        }
        self.total_routed += 1;
        Ok(best)
    }

    pub fn report_success(&mut self, id: u64) -> Result<(), ProxyError> {
        let b = self.backends.get_mut(&id).ok_or(ProxyError::BackendNotFound { id })?;
        b.successes += 1;
        b.consecutive_failures = 0;
        b.state = BackendState::Healthy;
        Ok(())
    }

    pub fn report_failure(&mut self, id: u64) -> Result<bool, ProxyError> {
        let b = self.backends.get_mut(&id).ok_or(ProxyError::BackendNotFound { id })?;
        b.failures += 1;
        b.consecutive_failures += 1;
        if b.consecutive_failures >= self.failure_threshold {
            b.state = BackendState::CircuitOpen;
            b.circuit_open_until = self.current_tick + self.recovery_ticks;
            self.total_circuit_opens += 1;
            self.total_failovers += 1;
            return Ok(true);
        }
        b.state = BackendState::Unhealthy;
        Ok(false)
    }

    fn try_recover(&mut self) {
        let ids: Vec<u64> = self.backends.keys().copied().collect();
        for id in ids {
            let b = self.backends.get_mut(&id).unwrap();
            if b.state == BackendState::CircuitOpen && self.current_tick >= b.circuit_open_until {
                b.state = BackendState::Healthy;
                b.consecutive_failures = 0;
            }
        }
    }

    pub fn backend_state(&self, id: u64) -> Option<&BackendState> { self.backends.get(&id).map(|b| &b.state) }
    pub fn healthy_count(&self) -> usize { self.backends.values().filter(|b| b.state == BackendState::Healthy).count() }
    pub fn backend_count(&self) -> usize { self.backends.len() }
    pub fn total_routed(&self) -> u64 { self.total_routed }
    pub fn total_failovers(&self) -> u64 { self.total_failovers }
    pub fn total_circuit_opens(&self) -> u64 { self.total_circuit_opens }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_router() { assert_eq!(ProxyRouter::new(3, 10).backend_count(), 0); }

    #[test]
    fn add_route() {
        let mut pr = ProxyRouter::new(3, 10);
        pr.add(1, 10).unwrap();
        let id = pr.route().unwrap();
        assert_eq!(id, 1);
    }

    #[test]
    fn weighted_routing() {
        let mut pr = ProxyRouter::new(3, 10);
        pr.add(1, 10).unwrap(); pr.add(2, 5).unwrap();
        let id = pr.route().unwrap();
        assert_eq!(id, 1);
    }

    #[test]
    fn failover() {
        let mut pr = ProxyRouter::new(2, 10);
        pr.add(1, 10).unwrap(); pr.add(2, 5).unwrap();
        pr.report_failure(1).unwrap();
        pr.report_failure(1).unwrap();
        assert_eq!(pr.backend_state(1), Some(&BackendState::CircuitOpen));
        let id = pr.route().unwrap();
        assert_eq!(id, 2);
        assert!(pr.total_failovers() >= 1);
    }

    #[test]
    fn circuit_recovery() {
        let mut pr = ProxyRouter::new(1, 2);
        pr.add(1, 10).unwrap();
        pr.report_failure(1).unwrap();
        assert_eq!(pr.backend_state(1), Some(&BackendState::CircuitOpen));
        pr.route().unwrap_err();
        pr.route().unwrap();
        pr.route().unwrap();
        assert_eq!(pr.backend_state(1), Some(&BackendState::Healthy));
    }

    #[test]
    fn success_resets() {
        let mut pr = ProxyRouter::new(2, 10);
        pr.add(1, 10).unwrap();
        pr.report_failure(1).unwrap();
        pr.report_success(1).unwrap();
        assert_eq!(pr.backend_state(1), Some(&BackendState::Healthy));
    }

    #[test]
    fn no_healthy() {
        let mut pr = ProxyRouter::new(1, 100);
        pr.add(1, 10).unwrap();
        pr.report_failure(1).unwrap();
        let err = pr.route().unwrap_err();
        assert!(matches!(err, ProxyError::NoHealthyBackends));
    }

    #[test]
    fn duplicate() {
        let mut pr = ProxyRouter::new(3, 10);
        pr.add(1, 10).unwrap();
        let err = pr.add(1, 10).unwrap_err();
        assert!(matches!(err, ProxyError::BackendExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut pr = ProxyRouter::new(3, 10);
        let err = pr.report_success(99).unwrap_err();
        assert!(matches!(err, ProxyError::BackendNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut pr = ProxyRouter::new(3, 10);
        pr.add(1, 10).unwrap();
        pr.route().unwrap();
        assert_eq!(pr.total_routed(), 1);
    }

    #[test]
    fn error_display() { assert!(ProxyError::NoHealthyBackends.to_string().contains("no healthy")); }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Health {
    Healthy = 3,
    Degraded = 2,
    Unhealthy = 1,
    Critical = 0,
}

impl std::fmt::Display for Health {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Health::Healthy => write!(f, "healthy"),
            Health::Degraded => write!(f, "degraded"),
            Health::Unhealthy => write!(f, "unhealthy"),
            Health::Critical => write!(f, "critical"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subsystem {
    Transport,
    Session,
    Pipeline,
    Watchdog,
    Memory,
    Firmware,
}

impl std::fmt::Display for Subsystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subsystem::Transport => write!(f, "transport"),
            Subsystem::Session => write!(f, "session"),
            Subsystem::Pipeline => write!(f, "pipeline"),
            Subsystem::Watchdog => write!(f, "watchdog"),
            Subsystem::Memory => write!(f, "memory"),
            Subsystem::Firmware => write!(f, "firmware"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubsystemHealth {
    pub subsystem: Subsystem,
    pub health: Health,
    pub message: String,
    pub timestamp_us: u64,
}

impl SubsystemHealth {
    pub fn new(subsystem: Subsystem, health: Health, message: &str, timestamp_us: u64) -> Self {
        Self {
            subsystem,
            health,
            message: message.to_string(),
            timestamp_us,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthMonitor {
    subsystems: Vec<SubsystemHealth>,
    checks: u64,
    last_check_us: u64,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            subsystems: Vec::new(),
            checks: 0,
            last_check_us: 0,
        }
    }

    pub fn report(&mut self, report: SubsystemHealth) {
        if let Some(existing) = self.subsystems.iter_mut().find(|s| s.subsystem == report.subsystem) {
            *existing = report;
        } else {
            self.subsystems.push(report);
        }
    }

    pub fn get(&self, subsystem: Subsystem) -> Option<&SubsystemHealth> {
        self.subsystems.iter().find(|s| s.subsystem == subsystem)
    }

    pub fn overall(&self) -> Health {
        self.subsystems
            .iter()
            .map(|s| s.health)
            .min()
            .unwrap_or(Health::Healthy)
    }

    pub fn score(&self) -> u8 {
        let total = self.subsystems.len() as u32;
        if total == 0 {
            return 100;
        }
        let sum: u32 = self.subsystems.iter().map(|s| s.health as u32).sum();
        ((sum * 100) / (total * (Health::Healthy as u32))) as u8
    }

    pub fn degraded_subsystems(&self) -> Vec<&SubsystemHealth> {
        self.subsystems
            .iter()
            .filter(|s| s.health < Health::Healthy)
            .collect()
    }

    pub fn check(&mut self, now_us: u64) -> Health {
        self.checks += 1;
        self.last_check_us = now_us;
        self.overall()
    }

    pub fn checks(&self) -> u64 {
        self.checks
    }

    pub fn last_check_us(&self) -> u64 {
        self.last_check_us
    }

    pub fn subsystem_count(&self) -> usize {
        self.subsystems.len()
    }

    pub fn clear(&mut self) {
        self.subsystems.clear();
        self.checks = 0;
        self.last_check_us = 0;
    }

    pub fn summary(&self) -> HealthSummary {
        HealthSummary {
            overall: self.overall(),
            score: self.score(),
            subsystem_count: self.subsystems_count(),
            degraded: self.degraded_subsystems().len(),
        }
    }

    fn subsystems_count(&self) -> usize {
        self.subsystems.len()
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthSummary {
    pub overall: Health,
    pub score: u8,
    pub subsystem_count: usize,
    pub degraded: usize,
}

impl std::fmt::Display for HealthSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (score={}, {} subsystems, {} degraded)",
            self.overall, self.score, self.subsystem_count, self.degraded
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_monitor_healthy() {
        let m = HealthMonitor::new();
        assert_eq!(m.overall(), Health::Healthy);
        assert_eq!(m.score(), 100);
    }

    #[test]
    fn report_and_get() {
        let mut m = HealthMonitor::new();
        m.report(SubsystemHealth::new(Subsystem::Transport, Health::Healthy, "ok", 0));
        assert!(m.get(Subsystem::Transport).is_some());
        assert!(m.get(Subsystem::Pipeline).is_none());
    }

    #[test]
    fn report_updates_existing() {
        let mut m = HealthMonitor::new();
        m.report(SubsystemHealth::new(Subsystem::Pipeline, Health::Healthy, "ok", 0));
        m.report(SubsystemHealth::new(Subsystem::Pipeline, Health::Degraded, "slow", 100));
        assert_eq!(m.get(Subsystem::Pipeline).unwrap().health, Health::Degraded);
        assert_eq!(m.subsystem_count(), 1);
    }

    #[test]
    fn overall_takes_minimum() {
        let mut m = HealthMonitor::new();
        m.report(SubsystemHealth::new(Subsystem::Transport, Health::Healthy, "ok", 0));
        m.report(SubsystemHealth::new(Subsystem::Pipeline, Health::Degraded, "slow", 0));
        m.report(SubsystemHealth::new(Subsystem::Memory, Health::Healthy, "ok", 0));
        assert_eq!(m.overall(), Health::Degraded);
    }

    #[test]
    fn score_calculation() {
        let mut m = HealthMonitor::new();
        m.report(SubsystemHealth::new(Subsystem::Transport, Health::Healthy, "ok", 0));
        m.report(SubsystemHealth::new(Subsystem::Pipeline, Health::Degraded, "slow", 0));
        assert_eq!(m.score(), 83);
    }

    #[test]
    fn degraded_subsystems() {
        let mut m = HealthMonitor::new();
        m.report(SubsystemHealth::new(Subsystem::Transport, Health::Healthy, "ok", 0));
        m.report(SubsystemHealth::new(Subsystem::Pipeline, Health::Unhealthy, "err", 0));
        let deg = m.degraded_subsystems();
        assert_eq!(deg.len(), 1);
        assert_eq!(deg[0].subsystem, Subsystem::Pipeline);
    }

    #[test]
    fn check_increments() {
        let mut m = HealthMonitor::new();
        m.check(100);
        m.check(200);
        assert_eq!(m.checks(), 2);
        assert_eq!(m.last_check_us(), 200);
    }

    #[test]
    fn clear() {
        let mut m = HealthMonitor::new();
        m.report(SubsystemHealth::new(Subsystem::Transport, Health::Critical, "dead", 0));
        m.check(100);
        m.clear();
        assert_eq!(m.overall(), Health::Healthy);
        assert_eq!(m.checks(), 0);
    }

    #[test]
    fn summary_display() {
        let mut m = HealthMonitor::new();
        m.report(SubsystemHealth::new(Subsystem::Transport, Health::Healthy, "ok", 0));
        let s = m.summary();
        assert!(s.to_string().contains("healthy"));
        assert!(s.to_string().contains("100"));
    }

    #[test]
    fn health_ordering() {
        assert!(Health::Critical < Health::Unhealthy);
        assert!(Health::Unhealthy < Health::Degraded);
        assert!(Health::Degraded < Health::Healthy);
    }

    #[test]
    fn health_display() {
        assert_eq!(Health::Healthy.to_string(), "healthy");
        assert_eq!(Health::Critical.to_string(), "critical");
    }

    #[test]
    fn subsystem_display() {
        assert_eq!(Subsystem::Pipeline.to_string(), "pipeline");
    }

    #[test]
    fn default_is_new() {
        let m = HealthMonitor::default();
        assert_eq!(m.subsystem_count(), 0);
    }
}

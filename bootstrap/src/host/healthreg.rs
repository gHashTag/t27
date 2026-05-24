use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HealthStatus { Healthy, Degraded, Unhealthy }

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthError {
    ServiceExists { name: String },
    ServiceNotFound { name: String },
    DependencyCycle { name: String },
}

impl std::fmt::Display for HealthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthError::ServiceExists { name } => write!(f, "service {name} exists"),
            HealthError::ServiceNotFound { name } => write!(f, "service {name} not found"),
            HealthError::DependencyCycle { name } => write!(f, "dependency cycle: {name}"),
        }
    }
}

impl std::error::Error for HealthError {}

struct Service {
    name: String,
    status: HealthStatus,
    deps: BTreeSet<String>,
    dependents: BTreeSet<String>,
    checks_passed: u64,
    checks_failed: u64,
}

pub struct HealthRegistry {
    services: BTreeMap<String, Service>,
    total_checks: u64,
    total_propagations: u64,
}

impl HealthRegistry {
    pub fn new() -> Self { Self { services: BTreeMap::new(), total_checks: 0, total_propagations: 0 } }

    pub fn register(&mut self, name: &str) -> Result<(), HealthError> {
        if self.services.contains_key(name) { return Err(HealthError::ServiceExists { name: name.to_string() }); }
        self.services.insert(name.to_string(), Service { name: name.to_string(), status: HealthStatus::Healthy, deps: BTreeSet::new(), dependents: BTreeSet::new(), checks_passed: 0, checks_failed: 0 });
        Ok(())
    }

    pub fn add_dep(&mut self, service: &str, dep: &str) -> Result<(), HealthError> {
        if !self.services.contains_key(service) { return Err(HealthError::ServiceNotFound { name: service.to_string() }); }
        if !self.services.contains_key(dep) { return Err(HealthError::ServiceNotFound { name: dep.to_string() }); }
        if self.has_cycle(service, dep) { return Err(HealthError::DependencyCycle { name: service.to_string() }); }
        self.services.get_mut(dep).unwrap().dependents.insert(service.to_string());
        self.services.get_mut(service).unwrap().deps.insert(dep.to_string());
        Ok(())
    }

    fn has_cycle(&self, from: &str, target: &str) -> bool {
        let mut visited = BTreeSet::new();
        let mut stack = vec![target];
        while let Some(n) = stack.pop() {
            if n == from { return true; }
            if visited.insert(n.to_string()) {
                if let Some(s) = self.services.get(n) {
                    for d in &s.deps { stack.push(d); }
                }
            }
        }
        false
    }

    pub fn set_status(&mut self, name: &str, status: HealthStatus) -> Result<Vec<String>, HealthError> {
        let s = self.services.get_mut(name).ok_or_else(|| HealthError::ServiceNotFound { name: name.to_string() })?;
        s.status = status;
        match status {
            HealthStatus::Healthy => s.checks_passed += 1,
            HealthStatus::Degraded | HealthStatus::Unhealthy => s.checks_failed += 1,
        }
        self.total_checks += 1;
        let mut affected = vec![name.to_string()];
        let dependents = self.services[name].dependents.clone();
        for dep in &dependents {
            self.propagate(dep, &mut affected);
        }
        Ok(affected)
    }

    fn propagate(&mut self, name: &str, affected: &mut Vec<String>) {
        let deps = self.services[name].deps.clone();
        let worst = deps.iter().map(|d| self.services[d].status).fold(HealthStatus::Healthy, |acc, s| {
            match (acc, s) {
                (HealthStatus::Unhealthy, _) | (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
                (HealthStatus::Degraded, _) | (_, HealthStatus::Degraded) => HealthStatus::Degraded,
                _ => HealthStatus::Healthy,
            }
        });
        let old = self.services[name].status;
        if worst != old || matches!(worst, HealthStatus::Unhealthy) {
            self.services.get_mut(name).unwrap().status = worst;
            self.total_propagations += 1;
            if !affected.contains(&name.to_string()) { affected.push(name.to_string()); }
        }
        let dependents = self.services[name].dependents.clone();
        for dep in &dependents { self.propagate(dep, affected); }
    }

    pub fn status(&self, name: &str) -> Option<HealthStatus> { self.services.get(name).map(|s| s.status) }
    pub fn deps(&self, name: &str) -> Option<Vec<String>> { self.services.get(name).map(|s| s.deps.iter().cloned().collect()) }
    pub fn service_count(&self) -> usize { self.services.len() }
    pub fn healthy_count(&self) -> usize { self.services.values().filter(|s| s.status == HealthStatus::Healthy).count() }
    pub fn checks(&self, name: &str) -> Option<(u64, u64)> { self.services.get(name).map(|s| (s.checks_passed, s.checks_failed)) }
    pub fn total_checks(&self) -> u64 { self.total_checks }
    pub fn total_propagations(&self) -> u64 { self.total_propagations }
}

impl Default for HealthRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_reg() { assert_eq!(HealthRegistry::new().service_count(), 0); }

    #[test]
    fn register_status() {
        let mut hr = HealthRegistry::new();
        hr.register("db").unwrap();
        assert_eq!(hr.status("db"), Some(HealthStatus::Healthy));
    }

    #[test]
    fn set_unhealthy() {
        let mut hr = HealthRegistry::new();
        hr.register("db").unwrap();
        let affected = hr.set_status("db", HealthStatus::Unhealthy).unwrap();
        assert_eq!(hr.status("db"), Some(HealthStatus::Unhealthy));
        assert!(affected.contains(&"db".to_string()));
    }

    #[test]
    fn propagation() {
        let mut hr = HealthRegistry::new();
        hr.register("db").unwrap();
        hr.register("api").unwrap();
        hr.add_dep("api", "db").unwrap();
        hr.set_status("db", HealthStatus::Unhealthy).unwrap();
        assert_eq!(hr.status("api"), Some(HealthStatus::Unhealthy));
    }

    #[test]
    fn degraded_propagation() {
        let mut hr = HealthRegistry::new();
        hr.register("cache").unwrap();
        hr.register("api").unwrap();
        hr.add_dep("api", "cache").unwrap();
        hr.set_status("cache", HealthStatus::Degraded).unwrap();
        assert_eq!(hr.status("api"), Some(HealthStatus::Degraded));
    }

    #[test]
    fn cycle_detection() {
        let mut hr = HealthRegistry::new();
        hr.register("a").unwrap();
        hr.register("b").unwrap();
        hr.add_dep("a", "b").unwrap();
        let err = hr.add_dep("b", "a").unwrap_err();
        assert!(matches!(err, HealthError::DependencyCycle { .. }));
    }

    #[test]
    fn duplicate() {
        let mut hr = HealthRegistry::new();
        hr.register("a").unwrap();
        let err = hr.register("a").unwrap_err();
        assert!(matches!(err, HealthError::ServiceExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut hr = HealthRegistry::new();
        let err = hr.set_status("x", HealthStatus::Healthy).unwrap_err();
        assert!(matches!(err, HealthError::ServiceNotFound { .. }));
    }

    #[test]
    fn checks_count() {
        let mut hr = HealthRegistry::new();
        hr.register("db").unwrap();
        hr.set_status("db", HealthStatus::Healthy).unwrap();
        hr.set_status("db", HealthStatus::Unhealthy).unwrap();
        let (p, f) = hr.checks("db").unwrap();
        assert_eq!(p, 1);
        assert_eq!(f, 1);
    }

    #[test]
    fn healthy_count() {
        let mut hr = HealthRegistry::new();
        hr.register("a").unwrap(); hr.register("b").unwrap();
        hr.set_status("b", HealthStatus::Unhealthy).unwrap();
        assert_eq!(hr.healthy_count(), 1);
    }

    #[test]
    fn error_display() { assert!(HealthError::ServiceNotFound { name: "x".into() }.to_string().contains("x")); }
}

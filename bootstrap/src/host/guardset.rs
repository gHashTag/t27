use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum GuardError {
    GuardExists { id: u64 },
    GuardNotFound { id: u64 },
    AlreadyReleased { id: u64 },
    ScopeClosed { scope: u64 },
}

impl std::fmt::Display for GuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuardError::GuardExists { id } => write!(f, "guard {id} exists"),
            GuardError::GuardNotFound { id } => write!(f, "guard {id} not found"),
            GuardError::AlreadyReleased { id } => write!(f, "guard {id} released"),
            GuardError::ScopeClosed { scope } => write!(f, "scope {scope} closed"),
        }
    }
}

impl std::error::Error for GuardError {}

#[derive(Debug, Clone, PartialEq)]
pub enum GuardState { Active, Released, Leaked }

struct Guard {
    id: u64,
    scope: u64,
    resource: String,
    state: GuardState,
}

struct Scope {
    id: u64,
    closed: bool,
    guards: Vec<u64>,
}

pub struct GuardSet {
    guards: BTreeMap<u64, Guard>,
    scopes: BTreeMap<u64, Scope>,
    next_guard: u64,
    next_scope: u64,
    total_acquired: u64,
    total_released: u64,
    total_leaked: u64,
}

impl GuardSet {
    pub fn new() -> Self { Self { guards: BTreeMap::new(), scopes: BTreeMap::new(), next_guard: 1, next_scope: 1, total_acquired: 0, total_released: 0, total_leaked: 0 } }

    pub fn create_scope(&mut self) -> u64 {
        let id = self.next_scope;
        self.next_scope += 1;
        self.scopes.insert(id, Scope { id, closed: false, guards: Vec::new() });
        id
    }

    pub fn close_scope(&mut self, scope: u64) -> Result<Vec<u64>, GuardError> {
        let s = self.scopes.get_mut(&scope).ok_or(GuardError::ScopeClosed { scope })?;
        s.closed = true;
        let guard_ids: Vec<u64> = s.guards.clone();
        let mut leaked = Vec::new();
        for gid in &guard_ids {
            if let Some(g) = self.guards.get_mut(gid) {
                if g.state == GuardState::Active {
                    g.state = GuardState::Leaked;
                    self.total_leaked += 1;
                    leaked.push(*gid);
                }
            }
        }
        Ok(leaked)
    }

    pub fn acquire(&mut self, scope: u64, resource: &str) -> Result<u64, GuardError> {
        let s = self.scopes.get_mut(&scope).ok_or(GuardError::ScopeClosed { scope })?;
        if s.closed { return Err(GuardError::ScopeClosed { scope }); }
        let id = self.next_guard;
        self.next_guard += 1;
        self.guards.insert(id, Guard { id, scope, resource: resource.to_string(), state: GuardState::Active });
        s.guards.push(id);
        self.total_acquired += 1;
        Ok(id)
    }

    pub fn release(&mut self, id: u64) -> Result<(), GuardError> {
        let g = self.guards.get_mut(&id).ok_or(GuardError::GuardNotFound { id })?;
        if g.state == GuardState::Released { return Err(GuardError::AlreadyReleased { id }); }
        g.state = GuardState::Released;
        self.total_released += 1;
        Ok(())
    }

    pub fn guard_state(&self, id: u64) -> Option<&GuardState> { self.guards.get(&id).map(|g| &g.state) }
    pub fn guard_resource(&self, id: u64) -> Option<&str> { self.guards.get(&id).map(|g| g.resource.as_str()) }
    pub fn active_guards(&self) -> usize { self.guards.values().filter(|g| g.state == GuardState::Active).count() }
    pub fn scope_guards(&self, scope: u64) -> Vec<u64> { self.scopes.get(&scope).map(|s| s.guards.clone()).unwrap_or_default() }
    pub fn scope_count(&self) -> usize { self.scopes.len() }
    pub fn guard_count(&self) -> usize { self.guards.len() }
    pub fn total_acquired(&self) -> u64 { self.total_acquired }
    pub fn total_released(&self) -> u64 { self.total_released }
    pub fn total_leaked(&self) -> u64 { self.total_leaked }
}

impl Default for GuardSet {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_gs() { assert_eq!(GuardSet::new().guard_count(), 0); }

    #[test]
    fn acquire_release() {
        let mut gs = GuardSet::new();
        let scope = gs.create_scope();
        let g = gs.acquire(scope, "file").unwrap();
        gs.release(g).unwrap();
        assert_eq!(gs.guard_state(g), Some(&GuardState::Released));
    }

    #[test]
    fn leak_detection() {
        let mut gs = GuardSet::new();
        let scope = gs.create_scope();
        gs.acquire(scope, "lock").unwrap();
        gs.acquire(scope, "conn").unwrap();
        let leaked = gs.close_scope(scope).unwrap();
        assert_eq!(leaked.len(), 2);
        assert_eq!(gs.total_leaked(), 2);
    }

    #[test]
    fn no_leak_if_released() {
        let mut gs = GuardSet::new();
        let scope = gs.create_scope();
        let g = gs.acquire(scope, "file").unwrap();
        gs.release(g).unwrap();
        let leaked = gs.close_scope(scope).unwrap();
        assert!(leaked.is_empty());
    }

    #[test]
    fn scope_closed_acquire() {
        let mut gs = GuardSet::new();
        let scope = gs.create_scope();
        gs.close_scope(scope).unwrap();
        let err = gs.acquire(scope, "x").unwrap_err();
        assert!(matches!(err, GuardError::ScopeClosed { .. }));
    }

    #[test]
    fn double_release() {
        let mut gs = GuardSet::new();
        let scope = gs.create_scope();
        let g = gs.acquire(scope, "x").unwrap();
        gs.release(g).unwrap();
        let err = gs.release(g).unwrap_err();
        assert!(matches!(err, GuardError::AlreadyReleased { .. }));
    }

    #[test]
    fn not_found() {
        let mut gs = GuardSet::new();
        let err = gs.release(99).unwrap_err();
        assert!(matches!(err, GuardError::GuardNotFound { .. }));
    }

    #[test]
    fn scope_guards() {
        let mut gs = GuardSet::new();
        let scope = gs.create_scope();
        gs.acquire(scope, "a").unwrap();
        gs.acquire(scope, "b").unwrap();
        assert_eq!(gs.scope_guards(scope).len(), 2);
    }

    #[test]
    fn resource_tracking() {
        let mut gs = GuardSet::new();
        let scope = gs.create_scope();
        let g = gs.acquire(scope, "mutex").unwrap();
        assert_eq!(gs.guard_resource(g), Some("mutex"));
    }

    #[test]
    fn stats() {
        let mut gs = GuardSet::new();
        let scope = gs.create_scope();
        gs.acquire(scope, "x").unwrap();
        gs.release(1).unwrap();
        assert_eq!(gs.total_acquired(), 1);
        assert_eq!(gs.total_released(), 1);
    }

    #[test]
    fn error_display() { assert!(GuardError::GuardNotFound { id: 3 }.to_string().contains("3")); }
}

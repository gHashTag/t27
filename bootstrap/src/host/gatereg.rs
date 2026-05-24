use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateState {
    Open,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    GateExists { id: u64 },
    GateNotFound { id: u64 },
    AlreadyOpen { id: u64 },
    AlreadyClosed { id: u64 },
    DependencyClosed { id: u64, dep: u64 },
    CircularDep { id: u64 },
}

impl std::fmt::Display for GateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateError::GateExists { id } => write!(f, "gate {id} exists"),
            GateError::GateNotFound { id } => write!(f, "gate {id} not found"),
            GateError::AlreadyOpen { id } => write!(f, "gate {id} already open"),
            GateError::AlreadyClosed { id } => write!(f, "gate {id} already closed"),
            GateError::DependencyClosed { id, dep } => write!(f, "gate {id} blocked by closed dep {dep}"),
            GateError::CircularDep { id } => write!(f, "circular dep on gate {id}"),
        }
    }
}

impl std::error::Error for GateError {}

#[derive(Debug, Clone)]
struct Gate {
    id: u64,
    state: GateState,
    deps: BTreeSet<u64>,
}

#[derive(Debug, Clone)]
pub struct GateInfo {
    pub id: u64,
    pub state: GateState,
    pub deps: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct GateRegistry {
    gates: BTreeMap<u64, Gate>,
    total_opens: u64,
    total_closes: u64,
}

impl GateRegistry {
    pub fn new() -> Self {
        Self { gates: BTreeMap::new(), total_opens: 0, total_closes: 0 }
    }

    pub fn register(&mut self, id: u64) -> Result<(), GateError> {
        if self.gates.contains_key(&id) {
            return Err(GateError::GateExists { id });
        }
        self.gates.insert(id, Gate { id, state: GateState::Closed, deps: BTreeSet::new() });
        Ok(())
    }

    pub fn add_dep(&mut self, id: u64, dep: u64) -> Result<(), GateError> {
        if id == dep { return Err(GateError::CircularDep { id }); }
        if !self.gates.contains_key(&id) { return Err(GateError::GateNotFound { id }); }
        if !self.gates.contains_key(&dep) { return Err(GateError::GateNotFound { id: dep }); }
        if self.would_cycle(id, dep) {
            return Err(GateError::CircularDep { id });
        }
        self.gates.get_mut(&id).unwrap().deps.insert(dep);
        Ok(())
    }

    fn would_cycle(&self, from: u64, to: u64) -> bool {
        let mut visited = BTreeSet::new();
        let mut stack = vec![to];
        while let Some(n) = stack.pop() {
            if n == from { return true; }
            if visited.insert(n) {
                if let Some(g) = self.gates.get(&n) {
                    for &d in &g.deps { stack.push(d); }
                }
            }
        }
        false
    }

    pub fn open(&mut self, id: u64) -> Result<(), GateError> {
        let gate = self.gates.get(&id).ok_or(GateError::GateNotFound { id })?;
        if gate.state == GateState::Open { return Err(GateError::AlreadyOpen { id }); }
        for &dep in &gate.deps {
            if self.gates[&dep].state == GateState::Closed {
                return Err(GateError::DependencyClosed { id, dep });
            }
        }
        self.gates.get_mut(&id).unwrap().state = GateState::Open;
        self.total_opens += 1;
        Ok(())
    }

    pub fn close(&mut self, id: u64) -> Result<(), GateError> {
        if !self.gates.contains_key(&id) { return Err(GateError::GateNotFound { id }); }
        let gate = &self.gates[&id];
        if gate.state == GateState::Closed { return Err(GateError::AlreadyClosed { id }); }
        self.gates.get_mut(&id).unwrap().state = GateState::Closed;
        self.total_closes += 1;
        Ok(())
    }

    pub fn state(&self, id: u64) -> Option<GateState> {
        self.gates.get(&id).map(|g| g.state)
    }

    pub fn is_open(&self, id: u64) -> bool {
        self.state(id) == Some(GateState::Open)
    }

    pub fn info(&self, id: u64) -> Option<GateInfo> {
        self.gates.get(&id).map(|g| GateInfo {
            id: g.id,
            state: g.state,
            deps: g.deps.iter().copied().collect(),
        })
    }

    pub fn gate_count(&self) -> usize { self.gates.len() }

    pub fn open_count(&self) -> usize {
        self.gates.values().filter(|g| g.state == GateState::Open).count()
    }

    pub fn closed_count(&self) -> usize {
        self.gates.values().filter(|g| g.state == GateState::Closed).count()
    }

    pub fn can_open(&self, id: u64) -> bool {
        self.gates.get(&id).map(|g| {
            g.state == GateState::Closed && g.deps.iter().all(|&d| self.gates[&d].state == GateState::Open)
        }).unwrap_or(false)
    }

    pub fn total_opens(&self) -> u64 { self.total_opens }
    pub fn total_closes(&self) -> u64 { self.total_closes }

    pub fn clear(&mut self) { self.gates.clear(); }
}

impl Default for GateRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_registry() {
        let gr = GateRegistry::new();
        assert_eq!(gr.gate_count(), 0);
    }

    #[test]
    fn register_and_count() {
        let mut gr = GateRegistry::new();
        gr.register(1).unwrap();
        gr.register(2).unwrap();
        assert_eq!(gr.gate_count(), 2);
        assert_eq!(gr.state(1), Some(GateState::Closed));
    }

    #[test]
    fn duplicate_gate() {
        let mut gr = GateRegistry::new();
        gr.register(1).unwrap();
        let err = gr.register(1).unwrap_err();
        assert!(matches!(err, GateError::GateExists { .. }));
    }

    #[test]
    fn open_and_close() {
        let mut gr = GateRegistry::new();
        gr.register(1).unwrap();
        gr.open(1).unwrap();
        assert!(gr.is_open(1));
        gr.close(1).unwrap();
        assert!(!gr.is_open(1));
    }

    #[test]
    fn already_open() {
        let mut gr = GateRegistry::new();
        gr.register(1).unwrap();
        gr.open(1).unwrap();
        let err = gr.open(1).unwrap_err();
        assert!(matches!(err, GateError::AlreadyOpen { .. }));
    }

    #[test]
    fn dependency_blocks_open() {
        let mut gr = GateRegistry::new();
        gr.register(1).unwrap();
        gr.register(2).unwrap();
        gr.add_dep(2, 1).unwrap();
        let err = gr.open(2).unwrap_err();
        assert!(matches!(err, GateError::DependencyClosed { .. }));
    }

    #[test]
    fn dependency_allows_open() {
        let mut gr = GateRegistry::new();
        gr.register(1).unwrap();
        gr.register(2).unwrap();
        gr.add_dep(2, 1).unwrap();
        gr.open(1).unwrap();
        gr.open(2).unwrap();
        assert!(gr.is_open(2));
    }

    #[test]
    fn can_open_check() {
        let mut gr = GateRegistry::new();
        gr.register(1).unwrap();
        gr.register(2).unwrap();
        gr.add_dep(2, 1).unwrap();
        assert!(!gr.can_open(2));
        gr.open(1).unwrap();
        assert!(gr.can_open(2));
    }

    #[test]
    fn circular_dep_rejected() {
        let mut gr = GateRegistry::new();
        gr.register(1).unwrap(); gr.register(2).unwrap();
        gr.add_dep(1, 2).unwrap();
        let err = gr.add_dep(2, 1).unwrap_err();
        assert!(matches!(err, GateError::CircularDep { .. }));
    }

    #[test]
    fn stats() {
        let mut gr = GateRegistry::new();
        gr.register(1).unwrap();
        gr.open(1).unwrap();
        gr.close(1).unwrap();
        assert_eq!(gr.total_opens(), 1);
        assert_eq!(gr.total_closes(), 1);
    }

    #[test]
    fn info() {
        let mut gr = GateRegistry::new();
        gr.register(1).unwrap(); gr.register(2).unwrap();
        gr.add_dep(2, 1).unwrap();
        let info = gr.info(2).unwrap();
        assert_eq!(info.deps, vec![1]);
    }

    #[test]
    fn error_display() {
        assert!(GateError::GateNotFound { id: 3 }.to_string().contains("3"));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    Drain = 0,
    Quiesce = 1,
    Cleanup = 2,
    Final = 3,
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phase::Drain => write!(f, "drain"),
            Phase::Quiesce => write!(f, "quiesce"),
            Phase::Cleanup => write!(f, "cleanup"),
            Phase::Final => write!(f, "final"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownError {
    AlreadyShuttingDown,
    NotRegistered { name: &'static str },
}

impl std::fmt::Display for ShutdownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShutdownError::AlreadyShuttingDown => write!(f, "already shutting down"),
            ShutdownError::NotRegistered { name } => write!(f, "not registered: {name}"),
        }
    }
}

impl std::error::Error for ShutdownError {}

pub type ShutdownFn = fn(Phase) -> Result<(), String>;

#[derive(Debug, Clone)]
struct ShutdownEntry {
    name: &'static str,
    phase: Phase,
    handler: ShutdownFn,
    priority: u8,
    completed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownState {
    Running,
    ShuttingDown { phase: Phase, step: usize },
    Complete,
    Failed { phase: Phase, step: usize },
}

#[derive(Debug, Clone)]
pub struct ShutdownCoordinator {
    entries: Vec<ShutdownEntry>,
    state: ShutdownState,
    timeout_ms: u64,
    total_completed: u64,
    total_failures: u64,
}

impl ShutdownCoordinator {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            entries: Vec::new(),
            state: ShutdownState::Running,
            timeout_ms,
            total_completed: 0,
            total_failures: 0,
        }
    }

    pub fn register(&mut self, name: &'static str, phase: Phase, priority: u8, handler: ShutdownFn) {
        self.entries.push(ShutdownEntry {
            name,
            phase,
            handler,
            priority,
            completed: false,
        });
    }

    pub fn begin(&mut self) -> Result<(), ShutdownError> {
        if self.state != ShutdownState::Running {
            return Err(ShutdownError::AlreadyShuttingDown);
        }
        self.state = ShutdownState::ShuttingDown {
            phase: Phase::Drain,
            step: 0,
        };
        Ok(())
    }

    pub fn step(&mut self) -> ShutdownState {
        match self.state {
            ShutdownState::Running | ShutdownState::Complete | ShutdownState::Failed { .. } => {
                return self.state;
            }
            ShutdownState::ShuttingDown { phase, .. } => {
                let next_entry = self
                    .entries
                    .iter()
                    .enumerate()
                    .find(|(_, e)| e.phase == phase && !e.completed);
                match next_entry {
                    Some((idx, _)) => {
                        let handler = self.entries[idx].handler;
                        match handler(phase) {
                            Ok(()) => {
                                self.entries[idx].completed = true;
                                self.total_completed += 1;
                                self.state = ShutdownState::ShuttingDown { phase, step: 0 };
                            }
                            Err(_) => {
                                self.total_failures += 1;
                                self.state = ShutdownState::Failed { phase, step: idx };
                            }
                        }
                    }
                    None => {
                        let next = match phase {
                            Phase::Drain => Phase::Quiesce,
                            Phase::Quiesce => Phase::Cleanup,
                            Phase::Cleanup => Phase::Final,
                            Phase::Final => {
                                self.state = ShutdownState::Complete;
                                return self.state;
                            }
                        };
                        self.state = ShutdownState::ShuttingDown {
                            phase: next,
                            step: 0,
                        };
                    }
                }
                self.state
            }
        }
    }

    pub fn run_all(&mut self) -> ShutdownState {
        loop {
            match self.step() {
                ShutdownState::Complete | ShutdownState::Failed { .. } => return self.state,
                _ => continue,
            }
        }
    }

    pub fn state(&self) -> ShutdownState {
        self.state
    }

    pub fn is_running(&self) -> bool {
        self.state == ShutdownState::Running
    }

    pub fn is_complete(&self) -> bool {
        self.state == ShutdownState::Complete
    }

    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn completed_count(&self) -> usize {
        self.entries.iter().filter(|e| e.completed).count()
    }

    pub fn total_completed(&self) -> u64 {
        self.total_completed
    }

    pub fn total_failures(&self) -> u64 {
        self.total_failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    static DRAIN_COUNT: AtomicU32 = AtomicU32::new(0);
    static CLEANUP_COUNT: AtomicU32 = AtomicU32::new(0);

    fn drain_handler(_phase: Phase) -> Result<(), String> {
        DRAIN_COUNT.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn cleanup_handler(_phase: Phase) -> Result<(), String> {
        CLEANUP_COUNT.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn fail_handler(_phase: Phase) -> Result<(), String> {
        Err("boom".into())
    }

    fn noop_handler(_phase: Phase) -> Result<(), String> {
        Ok(())
    }

    fn setup() {
        DRAIN_COUNT.store(0, Ordering::SeqCst);
        CLEANUP_COUNT.store(0, Ordering::SeqCst);
    }

    #[test]
    fn new_is_running() {
        let c = ShutdownCoordinator::new(5000);
        assert!(c.is_running());
        assert_eq!(c.timeout_ms(), 5000);
    }

    #[test]
    fn begin_starts_shutdown() {
        let mut c = ShutdownCoordinator::new(1000);
        c.begin().unwrap();
        assert!(!c.is_running());
    }

    #[test]
    fn begin_twice_errors() {
        let mut c = ShutdownCoordinator::new(1000);
        c.begin().unwrap();
        assert!(matches!(c.begin(), Err(ShutdownError::AlreadyShuttingDown)));
    }

    #[test]
    fn run_all_completes() {
        setup();
        let mut c = ShutdownCoordinator::new(1000);
        c.register("drain", Phase::Drain, 0, drain_handler);
        c.register("cleanup", Phase::Cleanup, 0, cleanup_handler);
        c.begin().unwrap();
        let state = c.run_all();
        assert_eq!(state, ShutdownState::Complete);
        assert!(c.is_complete());
        assert_eq!(DRAIN_COUNT.load(Ordering::SeqCst), 1);
        assert_eq!(CLEANUP_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn failure_stops() {
        let mut c = ShutdownCoordinator::new(1000);
        c.register("fail", Phase::Drain, 0, fail_handler);
        c.begin().unwrap();
        let state = c.run_all();
        assert!(matches!(state, ShutdownState::Failed { .. }));
        assert_eq!(c.total_failures(), 1);
    }

    #[test]
    fn phases_advance_in_order() {
        let mut c = ShutdownCoordinator::new(1000);
        c.register("a", Phase::Drain, 0, noop_handler);
        c.register("b", Phase::Quiesce, 0, noop_handler);
        c.begin().unwrap();
        c.step();
        let state = c.step();
        assert!(matches!(state, ShutdownState::ShuttingDown { phase: Phase::Quiesce, .. }));
    }

    #[test]
    fn empty_shutdown_completes() {
        let mut c = ShutdownCoordinator::new(1000);
        c.begin().unwrap();
        let state = c.run_all();
        assert_eq!(state, ShutdownState::Complete);
    }

    #[test]
    fn entry_count() {
        let mut c = ShutdownCoordinator::new(1000);
        c.register("a", Phase::Drain, 0, noop_handler);
        c.register("b", Phase::Drain, 0, noop_handler);
        assert_eq!(c.entry_count(), 2);
    }

    #[test]
    fn completed_count() {
        setup();
        let mut c = ShutdownCoordinator::new(1000);
        c.register("a", Phase::Drain, 0, drain_handler);
        c.register("b", Phase::Drain, 0, noop_handler);
        c.begin().unwrap();
        c.run_all();
        assert_eq!(c.completed_count(), 2);
    }

    #[test]
    fn phase_display() {
        assert_eq!(Phase::Drain.to_string(), "drain");
        assert_eq!(Phase::Final.to_string(), "final");
    }

    #[test]
    fn error_display() {
        assert!(ShutdownError::AlreadyShuttingDown.to_string().contains("shutting down"));
        let e = ShutdownError::NotRegistered { name: "x" };
        assert!(e.to_string().contains("x"));
    }
}

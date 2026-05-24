#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StateId(pub u8);

impl std::fmt::Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "S{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventId(pub u8);

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{}", self.0)
    }
}

pub type GuardFn = fn() -> bool;

#[derive(Debug, Clone)]
pub struct Transition {
    pub from: StateId,
    pub event: EventId,
    pub to: StateId,
    pub guard: Option<GuardFn>,
    pub action: Option<String>,
}

impl Transition {
    pub fn new(from: StateId, event: EventId, to: StateId) -> Self {
        Self {
            from,
            event,
            to,
            guard: None,
            action: None,
        }
    }

    pub fn with_guard(mut self, guard: GuardFn) -> Self {
        self.guard = Some(guard);
        self
    }

    pub fn with_action(mut self, action: &str) -> Self {
        self.action = Some(action.to_string());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmError {
    NoTransition { state: StateId, event: EventId },
    GuardBlocked { state: StateId, event: EventId },
}

impl std::fmt::Display for SmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmError::NoTransition { state, event } => {
                write!(f, "no transition: {state} + {event}")
            }
            SmError::GuardBlocked { state, event } => {
                write!(f, "guard blocked: {state} + {event}")
            }
        }
    }
}

impl std::error::Error for SmError {}

#[derive(Debug, Clone)]
pub struct SmEngine {
    transitions: Vec<Transition>,
    current: StateId,
    history: Vec<(StateId, EventId, StateId)>,
    total_transitions: u64,
    total_blocked: u64,
}

impl SmEngine {
    pub fn new(initial: StateId) -> Self {
        Self {
            transitions: Vec::new(),
            current: initial,
            history: Vec::new(),
            total_transitions: 0,
            total_blocked: 0,
        }
    }

    pub fn add_transition(&mut self, t: Transition) {
        self.transitions.push(t);
    }

    pub fn current(&self) -> StateId {
        self.current
    }

    pub fn process(&mut self, event: EventId) -> Result<StateId, SmError> {
        let candidate = self
            .transitions
            .iter()
            .find(|t| t.from == self.current && t.event == event);
        let t = match candidate {
            Some(t) => t,
            None => {
                self.total_blocked += 1;
                return Err(SmError::NoTransition {
                    state: self.current,
                    event,
                });
            }
        };
        if let Some(guard) = t.guard {
            if !guard() {
                self.total_blocked += 1;
                return Err(SmError::GuardBlocked {
                    state: self.current,
                    event,
                });
            }
        }
        let from = self.current;
        self.current = t.to;
        self.history.push((from, event, t.to));
        self.total_transitions += 1;
        Ok(self.current)
    }

    pub fn force_state(&mut self, state: StateId) {
        self.current = state;
    }

    pub fn history(&self) -> &[(StateId, EventId, StateId)] {
        &self.history
    }

    pub fn total_transitions(&self) -> u64 {
        self.total_transitions
    }

    pub fn total_blocked(&self) -> u64 {
        self.total_blocked
    }

    pub fn available_events(&self) -> Vec<EventId> {
        self.transitions
            .iter()
            .filter(|t| t.from == self.current)
            .map(|t| t.event)
            .collect()
    }

    pub fn reset(&mut self, initial: StateId) {
        self.current = initial;
        self.history.clear();
        self.total_transitions = 0;
        self.total_blocked = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const S_IDLE: StateId = StateId(0);
    const S_ACTIVE: StateId = StateId(1);
    const S_DONE: StateId = StateId(2);
    const S_ERROR: StateId = StateId(3);

    const E_START: EventId = EventId(0);
    const E_FINISH: EventId = EventId(1);
    const E_FAIL: EventId = EventId(2);
    const E_RESET: EventId = EventId(3);

    fn always_true() -> bool {
        true
    }

    fn always_false() -> bool {
        false
    }

    #[test]
    fn state_event_display() {
        assert_eq!(S_IDLE.to_string(), "S0");
        assert_eq!(E_START.to_string(), "E0");
    }

    #[test]
    fn initial_state() {
        let sm = SmEngine::new(S_IDLE);
        assert_eq!(sm.current(), S_IDLE);
    }

    #[test]
    fn simple_transition() {
        let mut sm = SmEngine::new(S_IDLE);
        sm.add_transition(Transition::new(S_IDLE, E_START, S_ACTIVE));
        let next = sm.process(E_START).unwrap();
        assert_eq!(next, S_ACTIVE);
        assert_eq!(sm.total_transitions(), 1);
    }

    #[test]
    fn no_transition() {
        let mut sm = SmEngine::new(S_IDLE);
        let err = sm.process(E_FINISH).unwrap_err();
        assert!(matches!(err, SmError::NoTransition { .. }));
        assert_eq!(sm.total_blocked(), 1);
    }

    #[test]
    fn guard_passes() {
        let mut sm = SmEngine::new(S_IDLE);
        sm.add_transition(Transition::new(S_IDLE, E_START, S_ACTIVE).with_guard(always_true));
        assert_eq!(sm.process(E_START).unwrap(), S_ACTIVE);
    }

    #[test]
    fn guard_blocks() {
        let mut sm = SmEngine::new(S_IDLE);
        sm.add_transition(Transition::new(S_IDLE, E_START, S_ACTIVE).with_guard(always_false));
        let err = sm.process(E_START).unwrap_err();
        assert!(matches!(err, SmError::GuardBlocked { .. }));
    }

    #[test]
    fn multi_step() {
        let mut sm = SmEngine::new(S_IDLE);
        sm.add_transition(Transition::new(S_IDLE, E_START, S_ACTIVE));
        sm.add_transition(Transition::new(S_ACTIVE, E_FINISH, S_DONE));
        sm.add_transition(Transition::new(S_DONE, E_RESET, S_IDLE));
        sm.process(E_START).unwrap();
        sm.process(E_FINISH).unwrap();
        sm.process(E_RESET).unwrap();
        assert_eq!(sm.current(), S_IDLE);
        assert_eq!(sm.history().len(), 3);
    }

    #[test]
    fn history_records() {
        let mut sm = SmEngine::new(S_IDLE);
        sm.add_transition(Transition::new(S_IDLE, E_START, S_ACTIVE));
        sm.process(E_START).unwrap();
        assert_eq!(sm.history()[0], (S_IDLE, E_START, S_ACTIVE));
    }

    #[test]
    fn available_events() {
        let mut sm = SmEngine::new(S_IDLE);
        sm.add_transition(Transition::new(S_IDLE, E_START, S_ACTIVE));
        sm.add_transition(Transition::new(S_IDLE, E_FAIL, S_ERROR));
        let events = sm.available_events();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn force_state() {
        let mut sm = SmEngine::new(S_IDLE);
        sm.force_state(S_DONE);
        assert_eq!(sm.current(), S_DONE);
    }

    #[test]
    fn reset() {
        let mut sm = SmEngine::new(S_IDLE);
        sm.add_transition(Transition::new(S_IDLE, E_START, S_ACTIVE));
        sm.process(E_START).unwrap();
        sm.reset(S_IDLE);
        assert_eq!(sm.current(), S_IDLE);
        assert_eq!(sm.history().len(), 0);
    }

    #[test]
    fn error_display() {
        assert!(SmError::NoTransition { state: S_IDLE, event: E_START }.to_string().contains("S0"));
        assert!(SmError::GuardBlocked { state: S_IDLE, event: E_START }.to_string().contains("blocked"));
    }

    #[test]
    fn transition_with_action() {
        let t = Transition::new(S_IDLE, E_START, S_ACTIVE).with_action("activate");
        assert_eq!(t.action.as_deref(), Some("activate"));
    }
}

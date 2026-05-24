#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PowerState {
    Active,
    Idle,
    Sleep,
    Off,
}

impl std::fmt::Display for PowerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerState::Active => write!(f, "active"),
            PowerState::Idle => write!(f, "idle"),
            PowerState::Sleep => write!(f, "sleep"),
            PowerState::Off => write!(f, "off"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerError {
    InvalidTransition { from: PowerState, to: PowerState },
    AlreadyInState { state: PowerState },
}

impl std::fmt::Display for PowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerError::InvalidTransition { from, to } => {
                write!(f, "invalid transition: {from} -> {to}")
            }
            PowerError::AlreadyInState { state } => {
                write!(f, "already in {state}")
            }
        }
    }
}

impl std::error::Error for PowerError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    pub from: PowerState,
    pub to: PowerState,
    pub latency_us: u64,
}

impl Transition {
    pub const fn new(from: PowerState, to: PowerState, latency_us: u64) -> Self {
        Self { from, to, latency_us }
    }
}

const VALID_TRANSITIONS: &[Transition] = &[
    Transition::new(PowerState::Active, PowerState::Idle, 10),
    Transition::new(PowerState::Idle, PowerState::Active, 5),
    Transition::new(PowerState::Idle, PowerState::Sleep, 100),
    Transition::new(PowerState::Sleep, PowerState::Idle, 200),
    Transition::new(PowerState::Active, PowerState::Off, 1000),
    Transition::new(PowerState::Idle, PowerState::Off, 500),
    Transition::new(PowerState::Sleep, PowerState::Off, 100),
    Transition::new(PowerState::Off, PowerState::Active, 5000),
];

pub fn is_valid_transition(from: PowerState, to: PowerState) -> bool {
    VALID_TRANSITIONS.iter().any(|t| t.from == from && t.to == to)
}

pub fn transition_latency(from: PowerState, to: PowerState) -> Option<u64> {
    VALID_TRANSITIONS
        .iter()
        .find(|t| t.from == from && t.to == to)
        .map(|t| t.latency_us)
}

#[derive(Debug, Clone)]
pub struct PowerManager {
    state: PowerState,
    total_transitions: u64,
    total_time_us: u64,
    history: Vec<(PowerState, PowerState, u64)>,
}

impl PowerManager {
    pub fn new(initial: PowerState) -> Self {
        Self {
            state: initial,
            total_transitions: 0,
            total_time_us: 0,
            history: Vec::new(),
        }
    }

    pub fn state(&self) -> PowerState {
        self.state
    }

    pub fn transition(&mut self, to: PowerState) -> Result<u64, PowerError> {
        if self.state == to {
            return Err(PowerError::AlreadyInState { state: to });
        }
        if !is_valid_transition(self.state, to) {
            return Err(PowerError::InvalidTransition {
                from: self.state,
                to,
            });
        }
        let latency = transition_latency(self.state, to).unwrap_or(0);
        let from = self.state;
        self.history.push((from, to, latency));
        self.state = to;
        self.total_transitions += 1;
        self.total_time_us += latency;
        Ok(latency)
    }

    pub fn wake(&mut self) -> Result<u64, PowerError> {
        match self.state {
            PowerState::Idle => self.transition(PowerState::Active),
            PowerState::Sleep => {
                let l1 = self.transition(PowerState::Idle)?;
                let l2 = self.transition(PowerState::Active)?;
                Ok(l1 + l2)
            }
            PowerState::Off => self.transition(PowerState::Active),
            PowerState::Active => Err(PowerError::AlreadyInState { state: PowerState::Active }),
        }
    }

    pub fn suspend(&mut self) -> Result<u64, PowerError> {
        match self.state {
            PowerState::Active => {
                let l1 = self.transition(PowerState::Idle)?;
                let l2 = self.transition(PowerState::Sleep)?;
                Ok(l1 + l2)
            }
            PowerState::Idle => self.transition(PowerState::Sleep),
            PowerState::Sleep => Err(PowerError::AlreadyInState { state: PowerState::Sleep }),
            PowerState::Off => Err(PowerError::InvalidTransition {
                from: PowerState::Off,
                to: PowerState::Sleep,
            }),
        }
    }

    pub fn shutdown(&mut self) -> Result<u64, PowerError> {
        self.transition(PowerState::Off)
    }

    pub fn total_transitions(&self) -> u64 {
        self.total_transitions
    }

    pub fn total_time_us(&self) -> u64 {
        self.total_time_us
    }

    pub fn history(&self) -> &[(PowerState, PowerState, u64)] {
        &self.history
    }

    pub fn reset(&mut self) {
        self.state = PowerState::Active;
        self.total_transitions = 0;
        self.total_time_us = 0;
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state() {
        let pm = PowerManager::new(PowerState::Active);
        assert_eq!(pm.state(), PowerState::Active);
    }

    #[test]
    fn valid_transitions() {
        assert!(is_valid_transition(PowerState::Active, PowerState::Idle));
        assert!(is_valid_transition(PowerState::Idle, PowerState::Sleep));
        assert!(is_valid_transition(PowerState::Off, PowerState::Active));
        assert!(!is_valid_transition(PowerState::Active, PowerState::Sleep));
        assert!(!is_valid_transition(PowerState::Sleep, PowerState::Active));
    }

    #[test]
    fn transition_latency_values() {
        assert_eq!(transition_latency(PowerState::Active, PowerState::Idle), Some(10));
        assert_eq!(transition_latency(PowerState::Off, PowerState::Active), Some(5000));
        assert_eq!(transition_latency(PowerState::Active, PowerState::Sleep), None);
    }

    #[test]
    fn transition_ok() {
        let mut pm = PowerManager::new(PowerState::Active);
        let latency = pm.transition(PowerState::Idle).unwrap();
        assert_eq!(latency, 10);
        assert_eq!(pm.state(), PowerState::Idle);
        assert_eq!(pm.total_transitions(), 1);
        assert_eq!(pm.total_time_us(), 10);
    }

    #[test]
    fn transition_same_state() {
        let mut pm = PowerManager::new(PowerState::Active);
        let err = pm.transition(PowerState::Active).unwrap_err();
        assert!(matches!(err, PowerError::AlreadyInState { .. }));
    }

    #[test]
    fn transition_invalid() {
        let mut pm = PowerManager::new(PowerState::Active);
        let err = pm.transition(PowerState::Sleep).unwrap_err();
        assert!(matches!(err, PowerError::InvalidTransition { .. }));
    }

    #[test]
    fn wake_from_idle() {
        let mut pm = PowerManager::new(PowerState::Idle);
        let latency = pm.wake().unwrap();
        assert_eq!(pm.state(), PowerState::Active);
        assert_eq!(latency, 5);
    }

    #[test]
    fn wake_from_sleep() {
        let mut pm = PowerManager::new(PowerState::Sleep);
        let latency = pm.wake().unwrap();
        assert_eq!(pm.state(), PowerState::Active);
        assert_eq!(latency, 200 + 5);
    }

    #[test]
    fn wake_from_off() {
        let mut pm = PowerManager::new(PowerState::Off);
        let latency = pm.wake().unwrap();
        assert_eq!(pm.state(), PowerState::Active);
        assert_eq!(latency, 5000);
    }

    #[test]
    fn suspend_from_active() {
        let mut pm = PowerManager::new(PowerState::Active);
        let latency = pm.suspend().unwrap();
        assert_eq!(pm.state(), PowerState::Sleep);
        assert_eq!(latency, 10 + 100);
    }

    #[test]
    fn shutdown() {
        let mut pm = PowerManager::new(PowerState::Active);
        pm.shutdown().unwrap();
        assert_eq!(pm.state(), PowerState::Off);
    }

    #[test]
    fn history() {
        let mut pm = PowerManager::new(PowerState::Active);
        pm.transition(PowerState::Idle).unwrap();
        pm.transition(PowerState::Sleep).unwrap();
        assert_eq!(pm.history().len(), 2);
        assert_eq!(pm.history()[0], (PowerState::Active, PowerState::Idle, 10));
    }

    #[test]
    fn reset() {
        let mut pm = PowerManager::new(PowerState::Active);
        pm.transition(PowerState::Off).unwrap();
        pm.reset();
        assert_eq!(pm.state(), PowerState::Active);
        assert_eq!(pm.total_transitions(), 0);
    }

    #[test]
    fn state_display() {
        assert_eq!(PowerState::Active.to_string(), "active");
        assert_eq!(PowerState::Off.to_string(), "off");
    }

    #[test]
    fn error_display() {
        let e = PowerError::InvalidTransition { from: PowerState::Off, to: PowerState::Sleep };
        assert!(e.to_string().contains("invalid"));
        let e = PowerError::AlreadyInState { state: PowerState::Active };
        assert!(e.to_string().contains("active"));
    }
}

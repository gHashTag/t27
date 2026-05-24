use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SagaState { Running, Completed, Compensating, Compensated, Failed }

#[derive(Debug, Clone, PartialEq)]
pub enum StepState { Pending, Running, Completed, Compensating, Compensated, Failed }

#[derive(Debug, Clone, PartialEq)]
pub enum SagaError {
    SagaNotFound { id: u64 },
    StepNotFound { saga: u64, step: u32 },
    InvalidTransition { saga: u64, step: u32, from: StepState, to: StepState },
    NotRunning { saga: u64, state: SagaState },
    DuplicateStep { saga: u64, name: String },
}

impl std::fmt::Display for SagaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SagaError::SagaNotFound { id } => write!(f, "saga {id} not found"),
            SagaError::StepNotFound { saga, step } => write!(f, "saga {saga} step {step} not found"),
            SagaError::InvalidTransition { saga, step, from, to } => write!(f, "saga {saga} step {step}: {:?}->{from:?} invalid", to),
            SagaError::NotRunning { saga, state } => write!(f, "saga {saga}: {:?} not running", state),
            SagaError::DuplicateStep { saga, name } => write!(f, "saga {saga}: step {name} duplicate"),
        }
    }
}

impl std::error::Error for SagaError {}

struct Step {
    index: u32,
    name: String,
    state: StepState,
    has_compensate: bool,
}

struct Saga {
    id: u64,
    state: SagaState,
    steps: Vec<Step>,
    current_step: u32,
    completed_steps: u32,
}

pub struct SagaOrchestrator {
    sagas: BTreeMap<u64, Saga>,
    next_id: u64,
    total_started: u64,
    total_completed: u64,
    total_compensated: u64,
    total_failed: u64,
}

impl SagaOrchestrator {
    pub fn new() -> Self { Self { sagas: BTreeMap::new(), next_id: 1, total_started: 0, total_completed: 0, total_compensated: 0, total_failed: 0 } }

    pub fn begin(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.sagas.insert(id, Saga { id, state: SagaState::Running, steps: Vec::new(), current_step: 0, completed_steps: 0 });
        self.total_started += 1;
        id
    }

    pub fn add_step(&mut self, saga: u64, name: &str, has_compensate: bool) -> Result<u32, SagaError> {
        let s = self.sagas.get_mut(&saga).ok_or(SagaError::SagaNotFound { id: saga })?;
        if s.state != SagaState::Running { return Err(SagaError::NotRunning { saga, state: s.state.clone() }); }
        if s.steps.iter().any(|st| st.name == name) { return Err(SagaError::DuplicateStep { saga, name: name.to_string() }); }
        let idx = s.steps.len() as u32;
        s.steps.push(Step { index: idx, name: name.to_string(), state: StepState::Pending, has_compensate });
        Ok(idx)
    }

    pub fn start_step(&mut self, saga: u64, step: u32) -> Result<(), SagaError> {
        let s = self.sagas.get_mut(&saga).ok_or(SagaError::SagaNotFound { id: saga })?;
        if s.state != SagaState::Running { return Err(SagaError::NotRunning { saga, state: s.state.clone() }); }
        let st = s.steps.get_mut(step as usize).ok_or(SagaError::StepNotFound { saga, step })?;
        if st.state != StepState::Pending { return Err(SagaError::InvalidTransition { saga, step, from: st.state.clone(), to: StepState::Running }); }
        st.state = StepState::Running;
        s.current_step = step;
        Ok(())
    }

    pub fn complete_step(&mut self, saga: u64, step: u32) -> Result<(), SagaError> {
        let s = self.sagas.get_mut(&saga).ok_or(SagaError::SagaNotFound { id: saga })?;
        let st = s.steps.get_mut(step as usize).ok_or(SagaError::StepNotFound { saga, step })?;
        if st.state != StepState::Running { return Err(SagaError::InvalidTransition { saga, step, from: st.state.clone(), to: StepState::Completed }); }
        st.state = StepState::Completed;
        s.completed_steps += 1;
        if s.completed_steps == s.steps.len() as u32 {
            s.state = SagaState::Completed;
            self.total_completed += 1;
        }
        Ok(())
    }

    pub fn fail_step(&mut self, saga: u64, step: u32) -> Result<Vec<u32>, SagaError> {
        let s = self.sagas.get_mut(&saga).ok_or(SagaError::SagaNotFound { id: saga })?;
        let st = s.steps.get_mut(step as usize).ok_or(SagaError::StepNotFound { saga, step })?;
        st.state = StepState::Failed;
        s.state = SagaState::Compensating;
        self.total_failed += 1;
        let compensatable: Vec<u32> = s.steps.iter()
            .take(step as usize)
            .filter(|st| st.has_compensate && st.state == StepState::Completed)
            .map(|st| st.index)
            .collect();
        let comp_len = compensatable.len();
        for &idx in &compensatable {
            s.steps[idx as usize].state = StepState::Compensating;
        }
        if comp_len == 0 { s.state = SagaState::Compensated; self.total_compensated += 1; }
        drop(s);
        Ok(compensatable)
    }

    pub fn compensate_done(&mut self, saga: u64, step: u32) -> Result<bool, SagaError> {
        let s = self.sagas.get_mut(&saga).ok_or(SagaError::SagaNotFound { id: saga })?;
        let st = s.steps.get_mut(step as usize).ok_or(SagaError::StepNotFound { saga, step })?;
        st.state = StepState::Compensated;
        let all_done = s.steps.iter().all(|st| !matches!(st.state, StepState::Compensating));
        if all_done && s.state == SagaState::Compensating {
            s.state = SagaState::Compensated;
            self.total_compensated += 1;
        }
        Ok(all_done)
    }

    pub fn saga_state(&self, saga: u64) -> Option<&SagaState> { self.sagas.get(&saga).map(|s| &s.state) }
    pub fn step_state(&self, saga: u64, step: u32) -> Option<&StepState> { self.sagas.get(&saga).and_then(|s| s.steps.get(step as usize)).map(|st| &st.state) }
    pub fn step_count(&self, saga: u64) -> Option<usize> { self.sagas.get(&saga).map(|s| s.steps.len()) }
    pub fn active_sagas(&self) -> usize { self.sagas.values().filter(|s| matches!(s.state, SagaState::Running | SagaState::Compensating)).count() }
    pub fn total_started(&self) -> u64 { self.total_started }
    pub fn total_completed(&self) -> u64 { self.total_completed }
    pub fn total_compensated(&self) -> u64 { self.total_compensated }
    pub fn total_failed(&self) -> u64 { self.total_failed }
}

impl Default for SagaOrchestrator {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_orch() { assert_eq!(SagaOrchestrator::new().total_started(), 0); }

    #[test]
    fn begin_add_steps() {
        let mut o = SagaOrchestrator::new();
        let s = o.begin();
        o.add_step(s, "reserve", true).unwrap();
        o.add_step(s, "charge", true).unwrap();
        assert_eq!(o.step_count(s), Some(2));
    }

    #[test]
    fn complete_saga() {
        let mut o = SagaOrchestrator::new();
        let s = o.begin();
        o.add_step(s, "a", false).unwrap();
        o.start_step(s, 0).unwrap();
        o.complete_step(s, 0).unwrap();
        assert_eq!(o.saga_state(s), Some(&SagaState::Completed));
    }

    #[test]
    fn fail_compensate() {
        let mut o = SagaOrchestrator::new();
        let s = o.begin();
        o.add_step(s, "reserve", true).unwrap();
        o.add_step(s, "charge", true).unwrap();
        o.start_step(s, 0).unwrap();
        o.complete_step(s, 0).unwrap();
        let comp = o.fail_step(s, 1).unwrap();
        assert_eq!(comp, vec![0]);
        assert_eq!(o.step_state(s, 0), Some(&StepState::Compensating));
    }

    #[test]
    fn full_compensate_flow() {
        let mut o = SagaOrchestrator::new();
        let s = o.begin();
        o.add_step(s, "a", true).unwrap();
        o.add_step(s, "b", true).unwrap();
        o.start_step(s, 0).unwrap();
        o.complete_step(s, 0).unwrap();
        o.fail_step(s, 1).unwrap();
        o.compensate_done(s, 0).unwrap();
        assert_eq!(o.saga_state(s), Some(&SagaState::Compensated));
    }

    #[test]
    fn not_found() {
        let mut o = SagaOrchestrator::new();
        let err = o.start_step(99, 0).unwrap_err();
        assert!(matches!(err, SagaError::SagaNotFound { .. }));
    }

    #[test]
    fn duplicate_step() {
        let mut o = SagaOrchestrator::new();
        let s = o.begin();
        o.add_step(s, "x", false).unwrap();
        let err = o.add_step(s, "x", false).unwrap_err();
        assert!(matches!(err, SagaError::DuplicateStep { .. }));
    }

    #[test]
    fn invalid_transition() {
        let mut o = SagaOrchestrator::new();
        let s = o.begin();
        o.add_step(s, "a", false).unwrap();
        let err = o.complete_step(s, 0).unwrap_err();
        assert!(matches!(err, SagaError::InvalidTransition { .. }));
    }

    #[test]
    fn stats() {
        let mut o = SagaOrchestrator::new();
        let s = o.begin();
        o.add_step(s, "a", false).unwrap();
        o.start_step(s, 0).unwrap();
        o.complete_step(s, 0).unwrap();
        assert_eq!(o.total_started(), 1);
        assert_eq!(o.total_completed(), 1);
    }

    #[test]
    fn active_sagas() {
        let mut o = SagaOrchestrator::new();
        o.begin(); o.begin();
        assert_eq!(o.active_sagas(), 2);
    }

    #[test]
    fn error_display() { assert!(SagaError::SagaNotFound { id: 3 }.to_string().contains("3")); }
}

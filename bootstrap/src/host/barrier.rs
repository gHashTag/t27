use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhaseId(pub u8);

impl std::fmt::Display for PhaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "phase{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticipantId(pub u8);

impl std::fmt::Display for ParticipantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "p{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BarrierError {
    AlreadyArrived { phase: PhaseId, participant: ParticipantId },
    UnknownPhase { phase: PhaseId },
    UnknownParticipant { participant: ParticipantId },
    NotRegistered { phase: PhaseId, participant: ParticipantId },
}

impl std::fmt::Display for BarrierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BarrierError::AlreadyArrived { phase, participant } => {
                write!(f, "{participant} already arrived at {phase}")
            }
            BarrierError::UnknownPhase { phase } => write!(f, "unknown {phase}"),
            BarrierError::UnknownParticipant { participant } => {
                write!(f, "unknown {participant}")
            }
            BarrierError::NotRegistered { phase, participant } => {
                write!(f, "{participant} not registered for {phase}")
            }
        }
    }
}

impl std::error::Error for BarrierError {}

#[derive(Debug, Clone)]
pub struct PhaseBarrier {
    pub phase: PhaseId,
    arrived: std::collections::HashSet<ParticipantId>,
    expected: usize,
    timeout_us: u64,
}

impl PhaseBarrier {
    pub fn new(phase: PhaseId, expected: usize, timeout_us: u64) -> Self {
        Self {
            phase,
            arrived: std::collections::HashSet::new(),
            expected,
            timeout_us,
        }
    }

    pub fn arrive(&mut self, pid: ParticipantId) -> Result<bool, BarrierError> {
        if self.arrived.contains(&pid) {
            return Err(BarrierError::AlreadyArrived { phase: self.phase, participant: pid });
        }
        self.arrived.insert(pid);
        Ok(self.arrived.len() >= self.expected)
    }

    pub fn is_complete(&self) -> bool {
        self.arrived.len() >= self.expected
    }

    pub fn arrived_count(&self) -> usize {
        self.arrived.len()
    }

    pub fn expected(&self) -> usize {
        self.expected
    }

    pub fn timeout_us(&self) -> u64 {
        self.timeout_us
    }

    pub fn missing(&self) -> usize {
        self.expected.saturating_sub(self.arrived.len())
    }
}

#[derive(Debug, Clone)]
pub struct BarrierSync {
    barriers: BTreeMap<PhaseId, PhaseBarrier>,
    participants: std::collections::HashSet<ParticipantId>,
    total_barriers_released: u64,
}

impl BarrierSync {
    pub fn new() -> Self {
        Self {
            barriers: BTreeMap::new(),
            participants: std::collections::HashSet::new(),
            total_barriers_released: 0,
        }
    }

    pub fn register(&mut self, pid: ParticipantId) {
        self.participants.insert(pid);
    }

    pub fn unregister(&mut self, pid: ParticipantId) -> bool {
        self.participants.remove(&pid)
    }

    pub fn add_barrier(&mut self, phase: PhaseId, expected: usize, timeout_us: u64) {
        self.barriers.insert(phase, PhaseBarrier::new(phase, expected, timeout_us));
    }

    pub fn arrive(&mut self, phase: PhaseId, pid: ParticipantId) -> Result<bool, BarrierError> {
        if !self.participants.contains(&pid) {
            return Err(BarrierError::UnknownParticipant { participant: pid });
        }
        let barrier = self.barriers.get_mut(&phase)
            .ok_or(BarrierError::UnknownPhase { phase })?;
        let complete = barrier.arrive(pid)?;
        if complete {
            self.total_barriers_released += 1;
        }
        Ok(complete)
    }

    pub fn is_complete(&self, phase: PhaseId) -> bool {
        self.barriers.get(&phase).map_or(false, |b| b.is_complete())
    }

    pub fn barrier(&self, phase: PhaseId) -> Option<&PhaseBarrier> {
        self.barriers.get(&phase)
    }

    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    pub fn barrier_count(&self) -> usize {
        self.barriers.len()
    }

    pub fn total_released(&self) -> u64 {
        self.total_barriers_released
    }

    pub fn completed_barriers(&self) -> Vec<PhaseId> {
        self.barriers.values()
            .filter(|b| b.is_complete())
            .map(|b| b.phase)
            .collect()
    }

    pub fn pending_barriers(&self) -> Vec<&PhaseBarrier> {
        self.barriers.values().filter(|b| !b.is_complete()).collect()
    }

    pub fn reset_barrier(&mut self, phase: PhaseId) {
        if let Some(b) = self.barriers.get_mut(&phase) {
            let expected = b.expected;
            let timeout = b.timeout_us;
            *b = PhaseBarrier::new(phase, expected, timeout);
        }
    }

    pub fn clear(&mut self) {
        self.barriers.clear();
        self.participants.clear();
    }
}

impl Default for BarrierSync {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_display() {
        assert_eq!(PhaseId(3).to_string(), "phase3");
    }

    #[test]
    fn participant_display() {
        assert_eq!(ParticipantId(5).to_string(), "p5");
    }

    #[test]
    fn barrier_arrive_completes() {
        let mut bs = BarrierSync::new();
        bs.register(ParticipantId(1));
        bs.register(ParticipantId(2));
        bs.add_barrier(PhaseId(0), 2, 1000);
        assert!(!bs.arrive(PhaseId(0), ParticipantId(1)).unwrap());
        let complete = bs.arrive(PhaseId(0), ParticipantId(2)).unwrap();
        assert!(complete);
        assert!(bs.is_complete(PhaseId(0)));
        assert_eq!(bs.total_released(), 1);
    }

    #[test]
    fn barrier_already_arrived() {
        let mut bs = BarrierSync::new();
        bs.register(ParticipantId(1));
        bs.add_barrier(PhaseId(0), 2, 1000);
        bs.arrive(PhaseId(0), ParticipantId(1)).unwrap();
        let err = bs.arrive(PhaseId(0), ParticipantId(1)).unwrap_err();
        assert!(matches!(err, BarrierError::AlreadyArrived { .. }));
    }

    #[test]
    fn barrier_unknown_phase() {
        let mut bs = BarrierSync::new();
        bs.register(ParticipantId(1));
        let err = bs.arrive(PhaseId(99), ParticipantId(1)).unwrap_err();
        assert!(matches!(err, BarrierError::UnknownPhase { .. }));
    }

    #[test]
    fn barrier_unknown_participant() {
        let mut bs = BarrierSync::new();
        bs.add_barrier(PhaseId(0), 1, 1000);
        let err = bs.arrive(PhaseId(0), ParticipantId(99)).unwrap_err();
        assert!(matches!(err, BarrierError::UnknownParticipant { .. }));
    }

    #[test]
    fn barrier_missing_count() {
        let mut bs = BarrierSync::new();
        bs.register(ParticipantId(1));
        bs.register(ParticipantId(2));
        bs.register(ParticipantId(3));
        bs.add_barrier(PhaseId(0), 3, 1000);
        bs.arrive(PhaseId(0), ParticipantId(1)).unwrap();
        assert_eq!(bs.barrier(PhaseId(0)).unwrap().missing(), 2);
    }

    #[test]
    fn completed_and_pending() {
        let mut bs = BarrierSync::new();
        bs.register(ParticipantId(1));
        bs.add_barrier(PhaseId(0), 1, 1000);
        bs.add_barrier(PhaseId(1), 2, 1000);
        bs.arrive(PhaseId(0), ParticipantId(1)).unwrap();
        assert_eq!(bs.completed_barriers(), vec![PhaseId(0)]);
        assert_eq!(bs.pending_barriers().len(), 1);
    }

    #[test]
    fn reset_barrier() {
        let mut bs = BarrierSync::new();
        bs.register(ParticipantId(1));
        bs.add_barrier(PhaseId(0), 1, 1000);
        bs.arrive(PhaseId(0), ParticipantId(1)).unwrap();
        bs.reset_barrier(PhaseId(0));
        assert!(!bs.is_complete(PhaseId(0)));
        assert_eq!(bs.barrier(PhaseId(0)).unwrap().arrived_count(), 0);
    }

    #[test]
    fn unregister() {
        let mut bs = BarrierSync::new();
        bs.register(ParticipantId(1));
        assert!(bs.unregister(ParticipantId(1)));
        assert!(!bs.unregister(ParticipantId(1)));
    }

    #[test]
    fn clear() {
        let mut bs = BarrierSync::new();
        bs.register(ParticipantId(1));
        bs.add_barrier(PhaseId(0), 1, 1000);
        bs.clear();
        assert_eq!(bs.participant_count(), 0);
        assert_eq!(bs.barrier_count(), 0);
    }

    #[test]
    fn error_display() {
        assert!(BarrierError::AlreadyArrived { phase: PhaseId(0), participant: ParticipantId(1) }.to_string().contains("p1"));
        assert!(BarrierError::UnknownPhase { phase: PhaseId(5) }.to_string().contains("phase5"));
    }
}

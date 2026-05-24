use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarrierPhase {
    Waiting,
    Reached,
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BarrierError {
    AlreadyArrived { party: u64 },
    UnknownParty { party: u64 },
    AlreadyComplete,
    NotAllArrived { arrived: usize, expected: usize },
}

impl std::fmt::Display for BarrierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BarrierError::AlreadyArrived { party } => write!(f, "party {party} already arrived"),
            BarrierError::UnknownParty { party } => write!(f, "unknown party {party}"),
            BarrierError::AlreadyComplete => write!(f, "barrier already complete"),
            BarrierError::NotAllArrived { arrived, expected } => {
                write!(f, "{arrived}/{expected} arrived")
            }
        }
    }
}

impl std::error::Error for BarrierError {}

#[derive(Debug, Clone)]
struct Party {
    id: u64,
    arrived: bool,
    arrive_tick: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CoordBarrier {
    parties: BTreeMap<u64, Party>,
    party_count: usize,
    arrived_count: usize,
    phase: BarrierPhase,
    current_round: u64,
    timeout_ticks: u64,
    created_tick: u64,
    total_rounds: u64,
    total_timeouts: u64,
}

impl CoordBarrier {
    pub fn new(party_count: usize, timeout_ticks: u64) -> Self {
        Self {
            parties: BTreeMap::new(),
            party_count,
            arrived_count: 0,
            phase: BarrierPhase::Waiting,
            current_round: 0,
            timeout_ticks,
            created_tick: 0,
            total_rounds: 0,
            total_timeouts: 0,
        }
    }

    pub fn register(&mut self, party_id: u64) {
        self.parties.insert(party_id, Party { id: party_id, arrived: false, arrive_tick: None });
    }

    pub fn arrive(&mut self, party_id: u64, tick: u64) -> Result<BarrierPhase, BarrierError> {
        if self.phase == BarrierPhase::Reached { return Err(BarrierError::AlreadyComplete); }
        let party = self.parties.get_mut(&party_id)
            .ok_or(BarrierError::UnknownParty { party: party_id })?;
        if party.arrived { return Err(BarrierError::AlreadyArrived { party: party_id }); }
        party.arrived = true;
        party.arrive_tick = Some(tick);
        self.arrived_count += 1;
        if self.arrived_count >= self.party_count {
            self.phase = BarrierPhase::Reached;
        }
        Ok(self.phase)
    }

    pub fn check_timeout(&mut self, tick: u64) -> bool {
        if self.phase != BarrierPhase::Waiting { return false; }
        let earliest = self.parties.values()
            .filter(|p| p.arrived)
            .filter_map(|p| p.arrive_tick)
            .min();
        if let Some(earliest_tick) = earliest {
            if tick >= earliest_tick + self.timeout_ticks {
                self.phase = BarrierPhase::TimedOut;
                self.total_timeouts += 1;
                return true;
            }
        }
        false
    }

    pub fn reset(&mut self) {
        for party in self.parties.values_mut() {
            party.arrived = false;
            party.arrive_tick = None;
        }
        self.arrived_count = 0;
        self.phase = BarrierPhase::Waiting;
        self.current_round += 1;
        self.total_rounds += 1;
    }

    pub fn phase(&self) -> BarrierPhase {
        self.phase
    }

    pub fn current_round(&self) -> u64 {
        self.current_round
    }

    pub fn arrived_count(&self) -> usize {
        self.arrived_count
    }

    pub fn party_count(&self) -> usize {
        self.party_count
    }

    pub fn is_complete(&self) -> bool {
        self.phase == BarrierPhase::Reached
    }

    pub fn is_waiting(&self) -> bool {
        self.phase == BarrierPhase::Waiting
    }

    pub fn missing(&self) -> Vec<u64> {
        self.parties.values()
            .filter(|p| !p.arrived)
            .map(|p| p.id)
            .collect()
    }

    pub fn total_rounds(&self) -> u64 {
        self.total_rounds
    }

    pub fn total_timeouts(&self) -> u64 {
        self.total_timeouts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_barrier() {
        let cb = CoordBarrier::new(3, 100);
        assert_eq!(cb.party_count(), 3);
        assert!(cb.is_waiting());
    }

    #[test]
    fn all_arrive() {
        let mut cb = CoordBarrier::new(3, 100);
        cb.register(1); cb.register(2); cb.register(3);
        cb.arrive(1, 0).unwrap();
        cb.arrive(2, 0).unwrap();
        assert!(!cb.is_complete());
        cb.arrive(3, 0).unwrap();
        assert!(cb.is_complete());
        assert_eq!(cb.arrived_count(), 3);
    }

    #[test]
    fn duplicate_arrive() {
        let mut cb = CoordBarrier::new(2, 100);
        cb.register(1); cb.register(2);
        cb.arrive(1, 0).unwrap();
        let err = cb.arrive(1, 0).unwrap_err();
        assert!(matches!(err, BarrierError::AlreadyArrived { .. }));
    }

    #[test]
    fn unknown_party() {
        let mut cb = CoordBarrier::new(2, 100);
        let err = cb.arrive(99, 0).unwrap_err();
        assert!(matches!(err, BarrierError::UnknownParty { .. }));
    }

    #[test]
    fn timeout() {
        let mut cb = CoordBarrier::new(3, 10);
        cb.register(1); cb.register(2); cb.register(3);
        cb.arrive(1, 5).unwrap();
        assert!(cb.check_timeout(20));
        assert_eq!(cb.phase(), BarrierPhase::TimedOut);
    }

    #[test]
    fn no_timeout_before_limit() {
        let mut cb = CoordBarrier::new(3, 100);
        cb.register(1); cb.register(2); cb.register(3);
        cb.arrive(1, 0).unwrap();
        assert!(!cb.check_timeout(50));
    }

    #[test]
    fn missing_parties() {
        let mut cb = CoordBarrier::new(3, 100);
        cb.register(1); cb.register(2); cb.register(3);
        cb.arrive(1, 0).unwrap();
        assert_eq!(cb.missing(), vec![2, 3]);
    }

    #[test]
    fn reset_for_next_round() {
        let mut cb = CoordBarrier::new(2, 100);
        cb.register(1); cb.register(2);
        cb.arrive(1, 0).unwrap();
        cb.arrive(2, 0).unwrap();
        cb.reset();
        assert!(cb.is_waiting());
        assert_eq!(cb.arrived_count(), 0);
        assert_eq!(cb.current_round(), 1);
    }

    #[test]
    fn multiple_rounds() {
        let mut cb = CoordBarrier::new(2, 100);
        cb.register(1); cb.register(2);
        cb.arrive(1, 0).unwrap(); cb.arrive(2, 0).unwrap();
        cb.reset();
        cb.arrive(1, 10).unwrap(); cb.arrive(2, 10).unwrap();
        cb.reset();
        assert_eq!(cb.total_rounds(), 2);
    }

    #[test]
    fn already_complete() {
        let mut cb = CoordBarrier::new(1, 100);
        cb.register(1);
        cb.arrive(1, 0).unwrap();
        let err = cb.arrive(1, 0).unwrap_err();
        assert!(matches!(err, BarrierError::AlreadyComplete));
    }

    #[test]
    fn stats() {
        let mut cb = CoordBarrier::new(2, 100);
        cb.register(1); cb.register(2);
        cb.arrive(1, 0).unwrap(); cb.arrive(2, 0).unwrap();
        assert_eq!(cb.total_timeouts(), 0);
    }

    #[test]
    fn error_display() {
        assert!(BarrierError::NotAllArrived { arrived: 1, expected: 3 }.to_string().contains("1/3"));
    }
}

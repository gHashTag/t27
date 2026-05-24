use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum BarrierError {
    PartyExists { id: u64 },
    PartyNotFound { id: u64 },
    AlreadyWaiting { id: u64 },
    NotWaiting { id: u64 },
    Timeout { id: u64, phase: u64 },
}

impl std::fmt::Display for BarrierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BarrierError::PartyExists { id } => write!(f, "party {id} exists"),
            BarrierError::PartyNotFound { id } => write!(f, "party {id} not found"),
            BarrierError::AlreadyWaiting { id } => write!(f, "party {id} already waiting"),
            BarrierError::NotWaiting { id } => write!(f, "party {id} not waiting"),
            BarrierError::Timeout { id, phase } => write!(f, "party {id} timeout at phase {phase}"),
        }
    }
}

impl std::error::Error for BarrierError {}

#[derive(Debug, Clone, PartialEq)]
pub enum PartyState { Ready, Waiting, Passed }

struct Party {
    id: u64,
    state: PartyState,
    phases_passed: u64,
    wait_since: Option<u64>,
}

pub struct BarrierSync {
    parties: BTreeMap<u64, Party>,
    party_count: usize,
    phase: u64,
    total_arrivals: u64,
    total_syncs: u64,
    total_timeouts: u64,
}

impl BarrierSync {
    pub fn new(party_count: usize) -> Self { Self { parties: BTreeMap::new(), party_count, phase: 0, total_arrivals: 0, total_syncs: 0, total_timeouts: 0 } }

    pub fn register(&mut self, id: u64) -> Result<(), BarrierError> {
        if self.parties.contains_key(&id) { return Err(BarrierError::PartyExists { id }); }
        self.parties.insert(id, Party { id, state: PartyState::Ready, phases_passed: 0, wait_since: None });
        Ok(())
    }

    pub fn arrive(&mut self, id: u64, now: u64) -> Result<bool, BarrierError> {
        let p = self.parties.get_mut(&id).ok_or(BarrierError::PartyNotFound { id })?;
        if p.state == PartyState::Waiting { return Err(BarrierError::AlreadyWaiting { id }); }
        p.state = PartyState::Waiting;
        p.wait_since = Some(now);
        self.total_arrivals += 1;
        let waiting = self.parties.values().filter(|p| p.state == PartyState::Waiting).count();
        if waiting >= self.party_count {
            self.phase += 1;
            self.total_syncs += 1;
            for p in self.parties.values_mut() {
                if p.state == PartyState::Waiting {
                    p.state = PartyState::Passed;
                    p.phases_passed += 1;
                    p.wait_since = None;
                }
            }
            Ok(true)
        } else { Ok(false) }
    }

    pub fn check_timeout(&mut self, id: u64, now: u64, timeout: u64) -> Result<bool, BarrierError> {
        let p = self.parties.get(&id).ok_or(BarrierError::PartyNotFound { id })?;
        if p.state != PartyState::Waiting { return Ok(false); }
        if let Some(since) = p.wait_since {
            if now - since >= timeout {
                drop(p);
                let p = self.parties.get_mut(&id).unwrap();
                p.state = PartyState::Ready;
                p.wait_since = None;
                self.total_timeouts += 1;
                return Err(BarrierError::Timeout { id, phase: self.phase });
            }
        }
        Ok(false)
    }

    pub fn reset(&mut self, id: u64) -> Result<(), BarrierError> {
        let p = self.parties.get_mut(&id).ok_or(BarrierError::PartyNotFound { id })?;
        p.state = PartyState::Ready;
        p.wait_since = None;
        Ok(())
    }

    pub fn party_state(&self, id: u64) -> Option<&PartyState> { self.parties.get(&id).map(|p| &p.state) }
    pub fn phase(&self) -> u64 { self.phase }
    pub fn waiting_count(&self) -> usize { self.parties.values().filter(|p| p.state == PartyState::Waiting).count() }
    pub fn phases_passed(&self, id: u64) -> Option<u64> { self.parties.get(&id).map(|p| p.phases_passed) }
    pub fn party_count(&self) -> usize { self.party_count }
    pub fn total_arrivals(&self) -> u64 { self.total_arrivals }
    pub fn total_syncs(&self) -> u64 { self.total_syncs }
    pub fn total_timeouts(&self) -> u64 { self.total_timeouts }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_barrier() { let b = BarrierSync::new(3); assert_eq!(b.party_count(), 3); }

    #[test]
    fn full_sync() {
        let mut b = BarrierSync::new(2);
        b.register(1).unwrap(); b.register(2).unwrap();
        assert!(!b.arrive(1, 0).unwrap());
        assert!(b.arrive(2, 0).unwrap());
        assert_eq!(b.phase(), 1);
        assert_eq!(b.party_state(1), Some(&PartyState::Passed));
        assert_eq!(b.party_state(2), Some(&PartyState::Passed));
    }

    #[test]
    fn multi_phase() {
        let mut b = BarrierSync::new(2);
        b.register(1).unwrap(); b.register(2).unwrap();
        b.arrive(1, 0).unwrap(); b.arrive(2, 0).unwrap();
        b.reset(1).unwrap(); b.reset(2).unwrap();
        b.arrive(1, 10).unwrap(); b.arrive(2, 10).unwrap();
        assert_eq!(b.phase(), 2);
        assert_eq!(b.phases_passed(1), Some(2));
    }

    #[test]
    fn partial_arrival() {
        let mut b = BarrierSync::new(3);
        b.register(1).unwrap(); b.register(2).unwrap(); b.register(3).unwrap();
        b.arrive(1, 0).unwrap();
        assert_eq!(b.waiting_count(), 1);
        assert_eq!(b.phase(), 0);
    }

    #[test]
    fn timeout() {
        let mut b = BarrierSync::new(3);
        b.register(1).unwrap();
        b.arrive(1, 0).unwrap();
        let err = b.check_timeout(1, 100, 50).unwrap_err();
        assert!(matches!(err, BarrierError::Timeout { .. }));
        assert_eq!(b.total_timeouts(), 1);
    }

    #[test]
    fn no_timeout() {
        let mut b = BarrierSync::new(3);
        b.register(1).unwrap();
        b.arrive(1, 0).unwrap();
        let result = b.check_timeout(1, 10, 50).unwrap();
        assert!(!result);
    }

    #[test]
    fn duplicate_register() {
        let mut b = BarrierSync::new(3);
        b.register(1).unwrap();
        let err = b.register(1).unwrap_err();
        assert!(matches!(err, BarrierError::PartyExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut b = BarrierSync::new(3);
        let err = b.arrive(99, 0).unwrap_err();
        assert!(matches!(err, BarrierError::PartyNotFound { .. }));
    }

    #[test]
    fn double_arrive() {
        let mut b = BarrierSync::new(3);
        b.register(1).unwrap();
        b.arrive(1, 0).unwrap();
        let err = b.arrive(1, 0).unwrap_err();
        assert!(matches!(err, BarrierError::AlreadyWaiting { .. }));
    }

    #[test]
    fn stats() {
        let mut b = BarrierSync::new(2);
        b.register(1).unwrap(); b.register(2).unwrap();
        b.arrive(1, 0).unwrap(); b.arrive(2, 0).unwrap();
        assert_eq!(b.total_arrivals(), 2);
        assert_eq!(b.total_syncs(), 1);
    }

    #[test]
    fn error_display() { assert!(BarrierError::Timeout { id: 3, phase: 1 }.to_string().contains("3")); }
}

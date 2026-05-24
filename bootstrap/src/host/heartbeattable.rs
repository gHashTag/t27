use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum MemberState { Alive, Suspect, Dead }

impl std::fmt::Display for MemberState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemberState::Alive => write!(f, "alive"),
            MemberState::Suspect => write!(f, "suspect"),
            MemberState::Dead => write!(f, "dead"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HbError {
    MemberExists { id: u64 },
    MemberNotFound { id: u64 },
}

impl std::fmt::Display for HbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HbError::MemberExists { id } => write!(f, "member {id} exists"),
            HbError::MemberNotFound { id } => write!(f, "member {id} not found"),
        }
    }
}

impl std::error::Error for HbError {}

struct Member {
    id: u64,
    state: MemberState,
    last_heartbeat: u64,
    heartbeat_count: u64,
    intervals: Vec<u64>,
    suspect_since: Option<u64>,
    max_intervals: usize,
}

impl Member {
    fn phi(&self) -> f64 {
        if self.intervals.len() < 2 { return 0.0; }
        let mean = self.intervals.iter().sum::<u64>() as f64 / self.intervals.len() as f64;
        if mean == 0.0 { return 0.0; }
        let variance: f64 = self.intervals.iter().map(|&i| { let d = i as f64 - mean; d * d }).sum::<f64>() / self.intervals.len() as f64;
        let std_dev = variance.sqrt().max(1.0);
        let elapsed = 0.0;
        (-(elapsed + mean) / std_dev).exp()
    }
}

pub struct HeartbeatTable {
    members: BTreeMap<u64, Member>,
    current_tick: u64,
    suspect_threshold: u64,
    dead_threshold: u64,
    max_intervals: usize,
    total_heartbeats: u64,
    total_suspects: u64,
    total_deaths: u64,
}

impl HeartbeatTable {
    pub fn new(suspect_threshold: u64, dead_threshold: u64, max_intervals: usize) -> Self {
        Self { members: BTreeMap::new(), current_tick: 0, suspect_threshold, dead_threshold, max_intervals, total_heartbeats: 0, total_suspects: 0, total_deaths: 0 }
    }

    pub fn register(&mut self, id: u64) -> Result<(), HbError> {
        if self.members.contains_key(&id) { return Err(HbError::MemberExists { id }); }
        self.members.insert(id, Member { id, state: MemberState::Alive, last_heartbeat: self.current_tick, heartbeat_count: 0, intervals: Vec::new(), suspect_since: None, max_intervals: self.max_intervals });
        Ok(())
    }

    pub fn heartbeat(&mut self, id: u64) -> Result<(), HbError> {
        let m = self.members.get_mut(&id).ok_or(HbError::MemberNotFound { id })?;
        let elapsed = self.current_tick - m.last_heartbeat;
        if m.heartbeat_count > 0 && elapsed > 0 {
            if m.intervals.len() >= m.max_intervals { m.intervals.remove(0); }
            m.intervals.push(elapsed);
        }
        m.last_heartbeat = self.current_tick;
        m.heartbeat_count += 1;
        if m.state != MemberState::Alive {
            m.state = MemberState::Alive;
            m.suspect_since = None;
        }
        self.total_heartbeats += 1;
        Ok(())
    }

    pub fn tick(&mut self) -> Vec<(u64, MemberState)> {
        self.current_tick += 1;
        let mut transitions = Vec::new();
        let ids: Vec<u64> = self.members.keys().copied().collect();
        for id in ids {
            let m = self.members.get_mut(&id).unwrap();
            let elapsed = self.current_tick - m.last_heartbeat;
            match m.state {
                MemberState::Alive if elapsed > self.suspect_threshold => {
                    m.state = MemberState::Suspect;
                    m.suspect_since = Some(self.current_tick);
                    self.total_suspects += 1;
                    transitions.push((id, MemberState::Suspect));
                }
                MemberState::Suspect => {
                    if let Some(since) = m.suspect_since {
                        if self.current_tick - since > self.dead_threshold {
                            m.state = MemberState::Dead;
                            self.total_deaths += 1;
                            transitions.push((id, MemberState::Dead));
                        }
                    }
                }
                _ => {}
            }
        }
        transitions
    }

    pub fn state(&self, id: u64) -> Option<&MemberState> { self.members.get(&id).map(|m| &m.state) }
    pub fn alive_count(&self) -> usize { self.members.values().filter(|m| m.state == MemberState::Alive).count() }
    pub fn suspect_count(&self) -> usize { self.members.values().filter(|m| m.state == MemberState::Suspect).count() }
    pub fn dead_count(&self) -> usize { self.members.values().filter(|m| m.state == MemberState::Dead).count() }
    pub fn member_count(&self) -> usize { self.members.len() }
    pub fn current_tick(&self) -> u64 { self.current_tick }
    pub fn total_heartbeats(&self) -> u64 { self.total_heartbeats }
    pub fn total_suspects(&self) -> u64 { self.total_suspects }
    pub fn total_deaths(&self) -> u64 { self.total_deaths }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_table() { let t = HeartbeatTable::new(5, 10, 10); assert_eq!(t.member_count(), 0); }

    #[test]
    fn register_heartbeat() {
        let mut t = HeartbeatTable::new(5, 10, 10);
        t.register(1).unwrap();
        t.heartbeat(1).unwrap();
        assert_eq!(t.state(1), Some(&MemberState::Alive));
    }

    #[test]
    fn suspect_transition() {
        let mut t = HeartbeatTable::new(3, 10, 10);
        t.register(1).unwrap();
        t.heartbeat(1).unwrap();
        let mut all = Vec::new();
        for _ in 0..6 { all.extend(t.tick()); }
        assert!(all.iter().any(|(_, s)| *s == MemberState::Suspect));
    }

    #[test]
    fn dead_transition() {
        let mut t = HeartbeatTable::new(2, 3, 10);
        t.register(1).unwrap();
        t.heartbeat(1).unwrap();
        for _ in 0..10 { t.tick(); }
        assert_eq!(t.state(1), Some(&MemberState::Dead));
    }

    #[test]
    fn recovery() {
        let mut t = HeartbeatTable::new(3, 10, 10);
        t.register(1).unwrap();
        t.heartbeat(1).unwrap();
        for _ in 0..5 { t.tick(); }
        t.heartbeat(1).unwrap();
        assert_eq!(t.state(1), Some(&MemberState::Alive));
    }

    #[test]
    fn duplicate() {
        let mut t = HeartbeatTable::new(5, 10, 10);
        t.register(1).unwrap();
        let err = t.register(1).unwrap_err();
        assert!(matches!(err, HbError::MemberExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut t = HeartbeatTable::new(5, 10, 10);
        let err = t.heartbeat(99).unwrap_err();
        assert!(matches!(err, HbError::MemberNotFound { .. }));
    }

    #[test]
    fn counts() {
        let mut t = HeartbeatTable::new(5, 10, 10);
        t.register(1).unwrap(); t.register(2).unwrap(); t.register(3).unwrap();
        assert_eq!(t.alive_count(), 3);
    }

    #[test]
    fn stats() {
        let mut t = HeartbeatTable::new(5, 10, 10);
        t.register(1).unwrap();
        t.heartbeat(1).unwrap();
        t.heartbeat(1).unwrap();
        assert_eq!(t.total_heartbeats(), 2);
    }

    #[test]
    fn error_display() { assert!(HbError::MemberNotFound { id: 3 }.to_string().contains("3")); }
}

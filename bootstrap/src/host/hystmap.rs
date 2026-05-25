use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HystState {
    Low,
    High,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HmErr {
    NotFound { key: u64 },
    InvalidThreshold { lo: f64, hi: f64 },
}

impl std::fmt::Display for HmErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HmErr::NotFound { key } => write!(f, "key {key} not found"),
            HmErr::InvalidThreshold { lo, hi } => write!(f, "lo {lo} >= hi {hi}"),
        }
    }
}

impl std::error::Error for HmErr {}

struct Entry {
    state: HystState,
    value: f64,
    lo: f64,
    hi: f64,
    transitions: u64,
}

pub struct HystMap {
    entries: BTreeMap<u64, Entry>,
    total_updates: u64,
    total_transitions: u64,
}

impl HystMap {
    pub fn new() -> Self { Self { entries: BTreeMap::new(), total_updates: 0, total_transitions: 0 } }

    pub fn insert(&mut self, key: u64, lo: f64, hi: f64, initial: f64) -> Result<HystState, HmErr> {
        if lo >= hi { return Err(HmErr::InvalidThreshold { lo, hi }); }
        self.total_updates += 1;
        let state = if initial >= hi { HystState::High } else { HystState::Low };
        self.entries.insert(key, Entry { state, value: initial, lo, hi, transitions: 0 });
        Ok(state)
    }

    pub fn update(&mut self, key: u64, value: f64) -> Result<HystState, HmErr> {
        self.total_updates += 1;
        let e = self.entries.get_mut(&key).ok_or(HmErr::NotFound { key })?;
        let old_state = e.state;
        e.value = value;
        match e.state {
            HystState::Low => { if value >= e.hi { e.state = HystState::High; } }
            HystState::High => { if value < e.lo { e.state = HystState::Low; } }
        }
        if e.state != old_state { e.transitions += 1; self.total_transitions += 1; }
        Ok(e.state)
    }

    pub fn get(&self, key: u64) -> Option<(HystState, f64)> {
        self.entries.get(&key).map(|e| (e.state, e.value))
    }

    pub fn state(&self, key: u64) -> Option<HystState> { self.entries.get(&key).map(|e| e.state) }
    pub fn value(&self, key: u64) -> Option<f64> { self.entries.get(&key).map(|e| e.value) }
    pub fn transitions(&self, key: u64) -> Option<u64> { self.entries.get(&key).map(|e| e.transitions) }
    pub fn remove(&mut self, key: u64) -> Option<(HystState, f64)> { self.entries.remove(&key).map(|e| (e.state, e.value)) }
    pub fn contains(&self, key: u64) -> bool { self.entries.contains_key(&key) }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn total_updates(&self) -> u64 { self.total_updates }
    pub fn total_transitions(&self) -> u64 { self.total_transitions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_initial_low() {
        let mut hm = HystMap::new();
        assert_eq!(hm.insert(1, 20.0, 80.0, 50.0).unwrap(), HystState::Low);
    }

    #[test]
    fn insert_initial_high() {
        let mut hm = HystMap::new();
        assert_eq!(hm.insert(1, 20.0, 80.0, 90.0).unwrap(), HystState::High);
    }

    #[test]
    fn transition_low_to_high() {
        let mut hm = HystMap::new();
        hm.insert(1, 20.0, 80.0, 50.0).unwrap();
        assert_eq!(hm.update(1, 85.0).unwrap(), HystState::High);
    }

    #[test]
    fn hysteresis_stay_high() {
        let mut hm = HystMap::new();
        hm.insert(1, 20.0, 80.0, 85.0).unwrap();
        assert_eq!(hm.update(1, 50.0).unwrap(), HystState::High);
        assert_eq!(hm.update(1, 19.0).unwrap(), HystState::Low);
    }

    #[test]
    fn no_transition_in_dead_band() {
        let mut hm = HystMap::new();
        hm.insert(1, 20.0, 80.0, 50.0).unwrap();
        assert_eq!(hm.update(1, 60.0).unwrap(), HystState::Low);
    }

    #[test]
    fn transition_count() {
        let mut hm = HystMap::new();
        hm.insert(1, 20.0, 80.0, 50.0).unwrap();
        hm.update(1, 85.0).unwrap();
        hm.update(1, 10.0).unwrap();
        assert_eq!(hm.transitions(1).unwrap(), 2);
    }

    #[test]
    fn remove() {
        let mut hm = HystMap::new();
        hm.insert(1, 20.0, 80.0, 50.0).unwrap();
        let (s, v) = hm.remove(1).unwrap();
        assert_eq!(s, HystState::Low);
        assert!(!hm.contains(1));
    }

    #[test]
    fn invalid_threshold() { assert!(HystMap::new().insert(1, 80.0, 20.0, 50.0).is_err()); }

    #[test]
    fn stats() {
        let mut hm = HystMap::new();
        hm.insert(1, 20.0, 80.0, 50.0).unwrap();
        hm.update(1, 85.0).unwrap();
        assert_eq!(hm.total_transitions(), 1);
    }

    #[test]
    fn error_display() { assert!(HmErr::NotFound { key: 1 }.to_string().contains("not found")); }
}

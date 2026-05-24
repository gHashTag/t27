use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ClockOrder {
    Before,
    After,
    Concurrent,
    Equal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VClockError {
    UnknownProcess { id: u64 },
}

impl std::fmt::Display for VClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VClockError::UnknownProcess { id } => write!(f, "process {id} unknown"),
        }
    }
}

impl std::error::Error for VClockError {}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorClock {
    clock: BTreeMap<u64, u64>,
}

impl VectorClock {
    pub fn new() -> Self { Self { clock: BTreeMap::new() } }

    pub fn from_map(map: BTreeMap<u64, u64>) -> Self { Self { clock: map } }

    pub fn increment(&mut self, process: u64) -> u64 {
        let entry = self.clock.entry(process).or_insert(0);
        *entry += 1;
        *entry
    }

    pub fn get(&self, process: u64) -> u64 { *self.clock.get(&process).unwrap_or(&0) }

    pub fn merge(&self, other: &VectorClock) -> VectorClock {
        let mut result = self.clock.clone();
        for (&proc, &seq) in &other.clock {
            let entry = result.entry(proc).or_insert(0);
            *entry = (*entry).max(seq);
        }
        VectorClock { clock: result }
    }

    pub fn compare(&self, other: &VectorClock) -> ClockOrder {
        let all_keys: std::collections::BTreeSet<u64> =
            self.clock.keys().chain(other.clock.keys()).copied().collect();
        let mut self_less = false;
        let mut other_less = false;
        for k in &all_keys {
            let a = self.get(*k);
            let b = other.get(*k);
            if a < b { self_less = true; }
            if a > b { other_less = true; }
        }
        if !self_less && !other_less { return ClockOrder::Equal; }
        if self_less && !other_less { return ClockOrder::Before; }
        if other_less && !self_less { return ClockOrder::After; }
        ClockOrder::Concurrent
    }

    pub fn happens_before(&self, other: &VectorClock) -> bool {
        matches!(self.compare(other), ClockOrder::Before)
    }

    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        matches!(self.compare(other), ClockOrder::Concurrent)
    }

    pub fn processes(&self) -> Vec<u64> { self.clock.keys().copied().collect() }
    pub fn len(&self) -> usize { self.clock.len() }
    pub fn is_empty(&self) -> bool { self.clock.is_empty() }
}

impl Default for VectorClock {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clock() { assert!(VectorClock::new().is_empty()); }

    #[test]
    fn increment() {
        let mut vc = VectorClock::new();
        assert_eq!(vc.increment(1), 1);
        assert_eq!(vc.increment(1), 2);
        assert_eq!(vc.get(1), 2);
    }

    #[test]
    fn multi_process() {
        let mut vc = VectorClock::new();
        vc.increment(1); vc.increment(2); vc.increment(1);
        assert_eq!(vc.get(1), 2);
        assert_eq!(vc.get(2), 1);
        assert_eq!(vc.len(), 2);
    }

    #[test]
    fn merge() {
        let mut v1 = VectorClock::new();
        v1.increment(1); v1.increment(1);
        let mut v2 = VectorClock::new();
        v2.increment(2); v2.increment(2); v2.increment(2);
        let merged = v1.merge(&v2);
        assert_eq!(merged.get(1), 2);
        assert_eq!(merged.get(2), 3);
    }

    #[test]
    fn before() {
        let mut v1 = VectorClock::new();
        v1.increment(1);
        let mut v2 = v1.clone();
        v2.increment(1);
        assert!(v1.happens_before(&v2));
        assert_eq!(v1.compare(&v2), ClockOrder::Before);
    }

    #[test]
    fn after() {
        let mut v1 = VectorClock::new();
        v1.increment(1); v1.increment(1);
        let mut v2 = VectorClock::new();
        v2.increment(1);
        assert_eq!(v1.compare(&v2), ClockOrder::After);
    }

    #[test]
    fn concurrent() {
        let mut v1 = VectorClock::new();
        v1.increment(1);
        let mut v2 = VectorClock::new();
        v2.increment(2);
        assert!(v1.is_concurrent(&v2));
    }

    #[test]
    fn equal() {
        let mut v1 = VectorClock::new();
        v1.increment(1);
        let v2 = v1.clone();
        assert_eq!(v1.compare(&v2), ClockOrder::Equal);
    }

    #[test]
    fn processes() {
        let mut vc = VectorClock::new();
        vc.increment(3); vc.increment(1); vc.increment(2);
        assert_eq!(vc.processes(), vec![1, 2, 3]);
    }

    #[test]
    fn get_missing() {
        let vc = VectorClock::new();
        assert_eq!(vc.get(99), 0);
    }

    #[test]
    fn from_map() {
        let mut m = BTreeMap::new();
        m.insert(1u64, 5u64); m.insert(2, 10);
        let vc = VectorClock::from_map(m);
        assert_eq!(vc.get(1), 5);
    }

    #[test]
    fn error_display() {
        assert!(VClockError::UnknownProcess { id: 1 }.to_string().contains("1"));
    }
}

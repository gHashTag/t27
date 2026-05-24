use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum BcError {
    NodeExists { id: u64 },
    NodeNotFound { id: u64 },
}

impl std::fmt::Display for BcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BcError::NodeExists { id } => write!(f, "node {id} exists"),
            BcError::NodeNotFound { id } => write!(f, "node {id} not found"),
        }
    }
}

impl std::error::Error for BcError {}

#[derive(Debug, Clone)]
pub struct ClockEntry {
    pub node: u64,
    pub count: u32,
}

pub struct BloomClock {
    clocks: BTreeMap<u64, Vec<ClockEntry>>,
    num_slots: usize,
    total_ticks: u64,
    total_merges: u64,
}

impl BloomClock {
    pub fn new(num_slots: usize) -> Self { Self { clocks: BTreeMap::new(), num_slots, total_ticks: 0, total_merges: 0 } }

    pub fn register(&mut self, id: u64) -> Result<(), BcError> {
        if self.clocks.contains_key(&id) { return Err(BcError::NodeExists { id }); }
        self.clocks.insert(id, vec![ClockEntry { node: id, count: 0 }; self.num_slots]);
        Ok(())
    }

    pub fn tick(&mut self, id: u64) -> Result<u32, BcError> {
        let clock = self.clocks.get_mut(&id).ok_or(BcError::NodeNotFound { id })?;
        let slot = (id as usize) % self.num_slots;
        clock[slot].count += 1;
        self.total_ticks += 1;
        Ok(clock[slot].count)
    }

    pub fn merge(&mut self, target: u64, source: u64) -> Result<(), BcError> {
        if !self.clocks.contains_key(&target) { return Err(BcError::NodeNotFound { id: target }); }
        if !self.clocks.contains_key(&source) { return Err(BcError::NodeNotFound { id: source }); }
        let src = self.clocks.get(&source).cloned().unwrap();
        let dst = self.clocks.get_mut(&target).unwrap();
        for i in 0..self.num_slots {
            if src[i].count > dst[i].count {
                dst[i] = src[i].clone();
            }
        }
        self.total_merges += 1;
        Ok(())
    }

    pub fn happens_before(&self, a: u64, b: u64) -> Option<bool> {
        let ca = self.clocks.get(&a)?;
        let cb = self.clocks.get(&b)?;
        let mut all_leq = true;
        let mut any_lt = false;
        for i in 0..self.num_slots {
            if ca[i].count > cb[i].count { all_leq = false; break; }
            if ca[i].count < cb[i].count { any_lt = true; }
        }
        Some(all_leq && any_lt)
    }

    pub fn concurrent(&self, a: u64, b: u64) -> Option<bool> {
        let hb_ab = self.happens_before(a, b);
        let hb_ba = self.happens_before(b, a);
        match (hb_ab, hb_ba) {
            (Some(false), Some(false)) => Some(true),
            _ => Some(false),
        }
    }

    pub fn get(&self, id: u64) -> Option<&[ClockEntry]> { self.clocks.get(&id).map(|c| c.as_slice()) }
    pub fn node_count(&self) -> usize { self.clocks.len() }
    pub fn total_ticks(&self) -> u64 { self.total_ticks }
    pub fn total_merges(&self) -> u64 { self.total_merges }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clock() { assert_eq!(BloomClock::new(8).node_count(), 0); }

    #[test]
    fn register_tick() {
        let mut bc = BloomClock::new(8);
        bc.register(1).unwrap();
        let count = bc.tick(1).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn multiple_ticks() {
        let mut bc = BloomClock::new(8);
        bc.register(1).unwrap();
        bc.tick(1).unwrap(); bc.tick(1).unwrap(); bc.tick(1).unwrap();
        let clock = bc.get(1).unwrap();
        let slot = (1usize) % 8;
        assert_eq!(clock[slot].count, 3);
    }

    #[test]
    fn happens_before() {
        let mut bc = BloomClock::new(16);
        bc.register(1).unwrap(); bc.register(2).unwrap();
        bc.tick(1).unwrap();
        bc.merge(2, 1).unwrap();
        bc.tick(2).unwrap();
        assert_eq!(bc.happens_before(1, 2), Some(true));
        assert_eq!(bc.happens_before(2, 1), Some(false));
    }

    #[test]
    fn concurrent() {
        let mut bc = BloomClock::new(16);
        bc.register(1).unwrap(); bc.register(2).unwrap();
        bc.tick(1).unwrap(); bc.tick(2).unwrap();
        assert_eq!(bc.concurrent(1, 2), Some(true));
    }

    #[test]
    fn merge() {
        let mut bc = BloomClock::new(8);
        bc.register(1).unwrap(); bc.register(2).unwrap();
        bc.tick(1).unwrap(); bc.tick(1).unwrap();
        bc.merge(2, 1).unwrap();
        let clock = bc.get(2).unwrap();
        let slot = (1usize) % 8;
        assert_eq!(clock[slot].count, 2);
    }

    #[test]
    fn duplicate_register() {
        let mut bc = BloomClock::new(8);
        bc.register(1).unwrap();
        let err = bc.register(1).unwrap_err();
        assert!(matches!(err, BcError::NodeExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut bc = BloomClock::new(8);
        let err = bc.tick(99).unwrap_err();
        assert!(matches!(err, BcError::NodeNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut bc = BloomClock::new(8);
        bc.register(1).unwrap(); bc.register(2).unwrap();
        bc.tick(1).unwrap();
        bc.merge(2, 1).unwrap();
        assert_eq!(bc.total_ticks(), 1);
        assert_eq!(bc.total_merges(), 1);
    }

    #[test]
    fn error_display() { assert!(BcError::NodeNotFound { id: 3 }.to_string().contains("3")); }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PnErr {
    PlaceNotFound(u64),
    TransitionNotFound(u64),
    NotEnabled(u64),
}

impl std::fmt::Display for PnErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PnErr::PlaceNotFound(id) => write!(f, "place {id} not found"),
            PnErr::TransitionNotFound(id) => write!(f, "transition {id} not found"),
            PnErr::NotEnabled(id) => write!(f, "transition {id} not enabled"),
        }
    }
}

impl std::error::Error for PnErr {}

struct Transition {
    id: u64,
    inputs: Vec<(u64, usize)>,
    outputs: Vec<(u64, usize)>,
}

pub struct PetriNet {
    places: BTreeMap<u64, usize>,
    transitions: BTreeMap<u64, Transition>,
    total_fires: u64,
    total_checks: u64,
}

impl PetriNet {
    pub fn new() -> Self { Self { places: BTreeMap::new(), transitions: BTreeMap::new(), total_fires: 0, total_checks: 0 } }

    pub fn add_place(&mut self, id: u64, tokens: usize) { self.places.insert(id, tokens); }

    pub fn add_transition(&mut self, id: u64, inputs: Vec<(u64, usize)>, outputs: Vec<(u64, usize)>) {
        self.transitions.insert(id, Transition { id, inputs, outputs });
    }

    pub fn tokens(&self, place: u64) -> Option<usize> { self.places.get(&place).copied() }

    pub fn is_enabled(&mut self, tid: u64) -> Result<bool, PnErr> {
        self.total_checks += 1;
        let t = self.transitions.get(&tid).ok_or(PnErr::TransitionNotFound(tid))?;
        for &(pid, need) in &t.inputs {
            let have = self.places.get(&pid).copied().unwrap_or(0);
            if have < need { return Ok(false); }
        }
        Ok(true)
    }

    pub fn fire(&mut self, tid: u64) -> Result<(), PnErr> {
        if !self.is_enabled(tid)? { return Err(PnErr::NotEnabled(tid)); }
        self.total_fires += 1;
        let t = self.transitions.get(&tid).unwrap();
        let inputs = t.inputs.clone();
        let outputs = t.outputs.clone();
        for (pid, n) in &inputs {
            let p = self.places.get_mut(pid).ok_or(PnErr::PlaceNotFound(*pid))?;
            *p -= n;
        }
        for (pid, n) in &outputs {
            let p = self.places.get_mut(pid).ok_or(PnErr::PlaceNotFound(*pid))?;
            *p += n;
        }
        Ok(())
    }

    pub fn fire_any(&mut self) -> Option<u64> {
        let tids: Vec<u64> = self.transitions.keys().copied().collect();
        for tid in tids {
            if self.is_enabled(tid).unwrap_or(false) {
                self.fire(tid).ok();
                return Some(tid);
            }
        }
        None
    }

    pub fn total_tokens(&self) -> usize { self.places.values().sum() }
    pub fn place_count(&self) -> usize { self.places.len() }
    pub fn transition_count(&self) -> usize { self.transitions.len() }
    pub fn total_fires(&self) -> u64 { self.total_fires }
    pub fn total_checks(&self) -> u64 { self.total_checks }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_place() {
        let mut pn = PetriNet::new();
        pn.add_place(1, 5);
        assert_eq!(pn.tokens(1), Some(5));
    }

    #[test]
    fn fire_basic() {
        let mut pn = PetriNet::new();
        pn.add_place(1, 3);
        pn.add_place(2, 0);
        pn.add_transition(10, vec![(1, 1)], vec![(2, 1)]);
        assert!(pn.is_enabled(10).unwrap());
        pn.fire(10).unwrap();
        assert_eq!(pn.tokens(1), Some(2));
        assert_eq!(pn.tokens(2), Some(1));
    }

    #[test]
    fn not_enabled() {
        let mut pn = PetriNet::new();
        pn.add_place(1, 0);
        pn.add_transition(10, vec![(1, 1)], vec![]);
        assert!(!pn.is_enabled(10).unwrap());
        assert!(pn.fire(10).is_err());
    }

    #[test]
    fn fire_any() {
        let mut pn = PetriNet::new();
        pn.add_place(1, 1); pn.add_place(2, 0);
        pn.add_transition(10, vec![(1, 1)], vec![(2, 1)]);
        assert_eq!(pn.fire_any(), Some(10));
        assert_eq!(pn.fire_any(), None);
    }

    #[test]
    fn multi_input() {
        let mut pn = PetriNet::new();
        pn.add_place(1, 2); pn.add_place(2, 3); pn.add_place(3, 0);
        pn.add_transition(10, vec![(1, 1), (2, 2)], vec![(3, 3)]);
        pn.fire(10).unwrap();
        assert_eq!(pn.tokens(3), Some(3));
    }

    #[test]
    fn total_tokens() {
        let mut pn = PetriNet::new();
        pn.add_place(1, 3); pn.add_place(2, 2);
        assert_eq!(pn.total_tokens(), 5);
    }

    #[test]
    fn stats() {
        let mut pn = PetriNet::new();
        pn.add_place(1, 1); pn.add_place(2, 0);
        pn.add_transition(10, vec![(1, 1)], vec![(2, 1)]);
        pn.is_enabled(10).unwrap(); pn.fire(10).unwrap();
        assert_eq!(pn.total_checks(), 2);
        assert_eq!(pn.total_fires(), 1);
    }

    #[test]
    fn error_display() { assert!(PnErr::NotEnabled(5).to_string().contains("enabled")); }
}

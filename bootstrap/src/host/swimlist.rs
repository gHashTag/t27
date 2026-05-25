use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NodeState { Alive, Suspect, Dead }

#[derive(Clone)]
struct Member {
    id: u64,
    state: NodeState,
    incarnation: u64,
    suspicion_start: u64,
}

pub struct SwimList {
    members: BTreeMap<u64, Member>,
    now: u64,
    suspect_timeout: u64,
    total_pings: u64,
    total_acks: u64,
    total_suspects: u64,
    total_deaths: u64,
}

impl SwimList {
    pub fn new(suspect_timeout: u64) -> Self { Self { members: BTreeMap::new(), now: 0, suspect_timeout, total_pings: 0, total_acks: 0, total_suspects: 0, total_deaths: 0 } }

    pub fn add(&mut self, id: u64) { self.members.insert(id, Member { id, state: NodeState::Alive, incarnation: 0, suspicion_start: 0 }); }

    pub fn ping(&mut self, id: u64) -> bool {
        self.total_pings += 1;
        self.members.contains_key(&id)
    }

    pub fn ack(&mut self, id: u64) {
        self.total_acks += 1;
        if let Some(m) = self.members.get_mut(&id) {
            if m.state == NodeState::Suspect {
                m.incarnation += 1;
                m.state = NodeState::Alive;
            }
        }
    }

    pub fn suspect(&mut self, id: u64) {
        self.total_suspects += 1;
        if let Some(m) = self.members.get_mut(&id) {
            if m.state == NodeState::Alive {
                m.state = NodeState::Suspect;
                m.suspicion_start = self.now;
            }
        }
    }

    pub fn advance(&mut self, delta: u64) -> usize {
        self.now += delta;
        let mut dead = Vec::new();
        for (&id, m) in &self.members {
            if m.state == NodeState::Suspect && self.now - m.suspicion_start >= self.suspect_timeout {
                dead.push(id);
            }
        }
        let count = dead.len();
        for id in dead {
            self.total_deaths += 1;
            self.members.get_mut(&id).unwrap().state = NodeState::Dead;
        }
        count
    }

    pub fn state(&self, id: u64) -> Option<NodeState> { self.members.get(&id).map(|m| m.state) }
    pub fn incarnation(&self, id: u64) -> Option<u64> { self.members.get(&id).map(|m| m.incarnation) }
    pub fn alive_count(&self) -> usize { self.members.values().filter(|m| m.state == NodeState::Alive).count() }
    pub fn suspect_count(&self) -> usize { self.members.values().filter(|m| m.state == NodeState::Suspect).count() }
    pub fn dead_count(&self) -> usize { self.members.values().filter(|m| m.state == NodeState::Dead).count() }
    pub fn len(&self) -> usize { self.members.len() }
    pub fn now(&self) -> u64 { self.now }
    pub fn total_pings(&self) -> u64 { self.total_pings }
    pub fn total_acks(&self) -> u64 { self.total_acks }
    pub fn total_suspects(&self) -> u64 { self.total_suspects }
    pub fn total_deaths(&self) -> u64 { self.total_deaths }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_state() {
        let mut sl = SwimList::new(10);
        sl.add(1);
        assert_eq!(sl.state(1), Some(NodeState::Alive));
    }

    #[test]
    fn suspect_ack() {
        let mut sl = SwimList::new(10);
        sl.add(1); sl.suspect(1);
        assert_eq!(sl.state(1), Some(NodeState::Suspect));
        sl.ack(1);
        assert_eq!(sl.state(1), Some(NodeState::Alive));
        assert_eq!(sl.incarnation(1), Some(1));
    }

    #[test]
    fn suspect_timeout() {
        let mut sl = SwimList::new(5);
        sl.add(1); sl.suspect(1);
        sl.advance(10);
        assert_eq!(sl.state(1), Some(NodeState::Dead));
    }

    #[test]
    fn alive_stays() {
        let mut sl = SwimList::new(5);
        sl.add(1);
        sl.advance(100);
        assert_eq!(sl.state(1), Some(NodeState::Alive));
    }

    #[test]
    fn counts() {
        let mut sl = SwimList::new(10);
        sl.add(1); sl.add(2); sl.add(3);
        sl.suspect(2);
        assert_eq!(sl.alive_count(), 2);
        assert_eq!(sl.suspect_count(), 1);
    }

    #[test]
    fn stats() {
        let mut sl = SwimList::new(10);
        sl.add(1); sl.ping(1); sl.ack(1); sl.suspect(1);
        assert_eq!(sl.total_pings(), 1);
        assert_eq!(sl.total_acks(), 1);
        assert_eq!(sl.total_suspects(), 1);
    }

    #[test]
    fn missing_state() { assert_eq!(SwimList::new(10).state(99), None); }
}

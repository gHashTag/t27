use std::collections::BTreeMap;

pub type PeerId = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerHealth {
    Healthy,
    Suspect,
    Dead,
}

impl std::fmt::Display for PeerHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerHealth::Healthy => write!(f, "healthy"),
            PeerHealth::Suspect => write!(f, "suspect"),
            PeerHealth::Dead => write!(f, "dead"),
        }
    }
}

#[derive(Debug, Clone)]
struct PeerEntry {
    id: PeerId,
    name: String,
    last_beat_us: u64,
    timeout_us: u64,
    suspect_threshold_us: u64,
    total_beats: u64,
    missed: u64,
    health: PeerHealth,
}

#[derive(Debug, Clone)]
pub struct PeerStatus {
    pub id: PeerId,
    pub name: String,
    pub health: PeerHealth,
    pub last_beat_us: u64,
    pub total_beats: u64,
    pub missed: u64,
    pub latency_us: u64,
}

#[derive(Debug, Clone)]
pub struct HeartbeatMonitor {
    peers: BTreeMap<PeerId, PeerEntry>,
    total_beats: u64,
    total_missed: u64,
}

impl HeartbeatMonitor {
    pub fn new() -> Self {
        Self {
            peers: BTreeMap::new(),
            total_beats: 0,
            total_missed: 0,
        }
    }

    pub fn register(&mut self, id: PeerId, name: &str, timeout_us: u64, suspect_threshold_us: u64) {
        self.peers.insert(id, PeerEntry {
            id,
            name: name.to_string(),
            last_beat_us: 0,
            timeout_us,
            suspect_threshold_us,
            total_beats: 0,
            missed: 0,
            health: PeerHealth::Suspect,
        });
    }

    pub fn unregister(&mut self, id: PeerId) -> bool {
        self.peers.remove(&id).is_some()
    }

    pub fn beat(&mut self, id: PeerId, now_us: u64) -> Result<(), String> {
        let peer = self.peers.get_mut(&id).ok_or_else(|| format!("peer {id} not found"))?;
        peer.last_beat_us = now_us;
        peer.total_beats += 1;
        peer.health = PeerHealth::Healthy;
        self.total_beats += 1;
        Ok(())
    }

    pub fn check(&mut self, now_us: u64) -> Vec<PeerId> {
        let mut changed = Vec::new();
        for peer in self.peers.values_mut() {
            if peer.last_beat_us == 0 {
                continue;
            }
            let elapsed = now_us.saturating_sub(peer.last_beat_us);
            let old_health = peer.health;
            if elapsed > peer.timeout_us {
                peer.health = PeerHealth::Dead;
                peer.missed += 1;
                self.total_missed += 1;
            } else if elapsed > peer.suspect_threshold_us {
                peer.health = PeerHealth::Suspect;
            }
            if peer.health != old_health {
                changed.push(peer.id);
            }
        }
        changed
    }

    pub fn health(&self, id: PeerId) -> Option<PeerHealth> {
        self.peers.get(&id).map(|p| p.health)
    }

    pub fn peer_status(&self, id: PeerId) -> Option<PeerStatus> {
        self.peers.get(&id).map(|p| {
            let latency = if p.total_beats == 0 { 0 } else { p.timeout_us };
            PeerStatus {
                id: p.id,
                name: p.name.clone(),
                health: p.health,
                last_beat_us: p.last_beat_us,
                total_beats: p.total_beats,
                missed: p.missed,
                latency_us: latency,
            }
        })
    }

    pub fn all_statuses(&self) -> Vec<PeerStatus> {
        self.peers.keys().filter_map(|&id| self.peer_status(id)).collect()
    }

    pub fn healthy_count(&self) -> usize {
        self.peers.values().filter(|p| p.health == PeerHealth::Healthy).count()
    }

    pub fn suspect_count(&self) -> usize {
        self.peers.values().filter(|p| p.health == PeerHealth::Suspect).count()
    }

    pub fn dead_count(&self) -> usize {
        self.peers.values().filter(|p| p.health == PeerHealth::Dead).count()
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn total_beats(&self) -> u64 {
        self.total_beats
    }

    pub fn total_missed(&self) -> u64 {
        self.total_missed
    }

    pub fn is_all_healthy(&self) -> bool {
        self.peers.values().all(|p| p.health == PeerHealth::Healthy)
    }

    pub fn clear(&mut self) {
        self.peers.clear();
    }
}

impl Default for HeartbeatMonitor {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_display() {
        assert_eq!(PeerHealth::Healthy.to_string(), "healthy");
        assert_eq!(PeerHealth::Dead.to_string(), "dead");
    }

    #[test]
    fn register_and_beat() {
        let mut hm = HeartbeatMonitor::new();
        hm.register(1, "fpga", 1000, 500);
        hm.beat(1, 100).unwrap();
        assert_eq!(hm.health(1), Some(PeerHealth::Healthy));
        assert_eq!(hm.total_beats(), 1);
    }

    #[test]
    fn beat_unknown_peer() {
        let mut hm = HeartbeatMonitor::new();
        assert!(hm.beat(99, 0).is_err());
    }

    #[test]
    fn check_suspect() {
        let mut hm = HeartbeatMonitor::new();
        hm.register(1, "fpga", 1000, 500);
        hm.beat(1, 100).unwrap();
        let changed = hm.check(700);
        assert_eq!(hm.health(1), Some(PeerHealth::Suspect));
        assert!(changed.contains(&1));
    }

    #[test]
    fn check_dead() {
        let mut hm = HeartbeatMonitor::new();
        hm.register(1, "fpga", 1000, 500);
        hm.beat(1, 100).unwrap();
        hm.check(1200);
        assert_eq!(hm.health(1), Some(PeerHealth::Dead));
        assert_eq!(hm.total_missed(), 1);
    }

    #[test]
    fn check_healthy_no_change() {
        let mut hm = HeartbeatMonitor::new();
        hm.register(1, "fpga", 1000, 500);
        hm.beat(1, 100).unwrap();
        let changed = hm.check(200);
        assert!(changed.is_empty());
        assert_eq!(hm.health(1), Some(PeerHealth::Healthy));
    }

    #[test]
    fn peer_status() {
        let mut hm = HeartbeatMonitor::new();
        hm.register(1, "fpga", 1000, 500);
        hm.beat(1, 100).unwrap();
        let s = hm.peer_status(1).unwrap();
        assert_eq!(s.name, "fpga");
        assert_eq!(s.total_beats, 1);
    }

    #[test]
    fn counts() {
        let mut hm = HeartbeatMonitor::new();
        hm.register(1, "a", 100, 50);
        hm.register(2, "b", 100, 50);
        hm.beat(1, 0).unwrap();
        assert_eq!(hm.healthy_count(), 1);
        assert_eq!(hm.suspect_count(), 1);
    }

    #[test]
    fn is_all_healthy() {
        let mut hm = HeartbeatMonitor::new();
        hm.register(1, "a", 100, 50);
        hm.register(2, "b", 100, 50);
        assert!(!hm.is_all_healthy());
        hm.beat(1, 0).unwrap();
        assert!(!hm.is_all_healthy());
        hm.beat(2, 0).unwrap();
        assert!(hm.is_all_healthy());
    }

    #[test]
    fn unregister() {
        let mut hm = HeartbeatMonitor::new();
        hm.register(1, "a", 100, 50);
        assert!(hm.unregister(1));
        assert_eq!(hm.peer_count(), 0);
    }

    #[test]
    fn all_statuses() {
        let mut hm = HeartbeatMonitor::new();
        hm.register(1, "a", 100, 50);
        hm.register(2, "b", 100, 50);
        assert_eq!(hm.all_statuses().len(), 2);
    }
}

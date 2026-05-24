use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub enum GossipError {
    NodeExists { id: u64 },
    NodeNotFound { id: u64 },
    AlreadyInfected { msg_id: u64, node: u64 },
}

impl std::fmt::Display for GossipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GossipError::NodeExists { id } => write!(f, "node {id} exists"),
            GossipError::NodeNotFound { id } => write!(f, "node {id} not found"),
            GossipError::AlreadyInfected { msg_id, node } =>
                write!(f, "msg {msg_id} already at node {node}"),
        }
    }
}

impl std::error::Error for GossipError {}

#[derive(Debug, Clone)]
pub struct GossipMessage {
    pub id: u64,
    pub payload: Vec<u8>,
    pub origin: u64,
    pub round: u64,
}

struct Node {
    id: u64,
    peers: Vec<u64>,
    infected: BTreeSet<u64>,
}

#[derive(Debug, Clone)]
pub struct GossipRound {
    pub sender: u64,
    pub targets: Vec<u64>,
    pub messages: Vec<u64>,
}

pub struct GossipProtocol {
    nodes: BTreeMap<u64, Node>,
    messages: BTreeMap<u64, GossipMessage>,
    fanout: usize,
    next_msg_id: u64,
    total_rounds: u64,
    total_deliveries: u64,
}

impl GossipProtocol {
    pub fn new(fanout: usize) -> Self {
        Self { nodes: BTreeMap::new(), messages: BTreeMap::new(), fanout, next_msg_id: 1, total_rounds: 0, total_deliveries: 0 }
    }

    pub fn add_node(&mut self, id: u64) -> Result<(), GossipError> {
        if self.nodes.contains_key(&id) { return Err(GossipError::NodeExists { id }); }
        self.nodes.insert(id, Node { id, peers: Vec::new(), infected: BTreeSet::new() });
        Ok(())
    }

    pub fn add_peer(&mut self, node: u64, peer: u64) -> Result<(), GossipError> {
        if !self.nodes.contains_key(&node) { return Err(GossipError::NodeNotFound { id: node }); }
        if !self.nodes.contains_key(&peer) { return Err(GossipError::NodeNotFound { id: peer }); }
        let n = self.nodes.get_mut(&node).unwrap();
        if !n.peers.contains(&peer) { n.peers.push(peer); }
        Ok(())
    }

    pub fn inject(&mut self, origin: u64, payload: Vec<u8>) -> Result<u64, GossipError> {
        if !self.nodes.contains_key(&origin) { return Err(GossipError::NodeNotFound { id: origin }); }
        let msg_id = self.next_msg_id;
        self.next_msg_id += 1;
        self.messages.insert(msg_id, GossipMessage { id: msg_id, payload, origin, round: 0 });
        self.nodes.get_mut(&origin).unwrap().infected.insert(msg_id);
        Ok(msg_id)
    }

    pub fn round(&mut self) -> Vec<GossipRound> {
        let mut rounds = Vec::new();
        let node_ids: Vec<u64> = self.nodes.keys().copied().collect();
        for nid in node_ids {
            let node = self.nodes.get(&nid).unwrap();
            let new_msgs: Vec<u64> = node.infected.iter().copied().collect();
            if new_msgs.is_empty() || node.peers.is_empty() { continue; }
            let targets: Vec<u64> = {
                let start = (self.total_rounds as usize) % node.peers.len();
                node.peers.iter().cycle().skip(start).take(self.fanout.min(node.peers.len())).copied().collect()
            };
            for &tid in &targets {
                let target = self.nodes.get_mut(&tid).unwrap();
                for &mid in &new_msgs {
                    if target.infected.insert(mid) { self.total_deliveries += 1; }
                }
            }
            rounds.push(GossipRound { sender: nid, targets, messages: new_msgs });
        }
        for msg in self.messages.values_mut() { msg.round += 1; }
        self.total_rounds += 1;
        rounds
    }

    pub fn is_infected(&self, node: u64, msg_id: u64) -> bool {
        self.nodes.get(&node).map(|n| n.infected.contains(&msg_id)).unwrap_or(false)
    }

    pub fn infected_count(&self, msg_id: u64) -> usize {
        self.nodes.values().filter(|n| n.infected.contains(&msg_id)).count()
    }

    pub fn convergence(&self, msg_id: u64) -> f64 {
        if self.nodes.is_empty() { return 0.0; }
        self.infected_count(msg_id) as f64 / self.nodes.len() as f64
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn msg_count(&self) -> usize { self.messages.len() }
    pub fn total_rounds(&self) -> u64 { self.total_rounds }
    pub fn total_deliveries(&self) -> u64 { self.total_deliveries }
    pub fn fanout(&self) -> usize { self.fanout }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_protocol() {
        let gp = GossipProtocol::new(3);
        assert_eq!(gp.fanout(), 3);
        assert_eq!(gp.node_count(), 0);
    }

    #[test]
    fn add_nodes() {
        let mut gp = GossipProtocol::new(2);
        gp.add_node(1).unwrap(); gp.add_node(2).unwrap(); gp.add_node(3).unwrap();
        assert_eq!(gp.node_count(), 3);
    }

    #[test]
    fn duplicate_node() {
        let mut gp = GossipProtocol::new(2);
        gp.add_node(1).unwrap();
        let err = gp.add_node(1).unwrap_err();
        assert!(matches!(err, GossipError::NodeExists { .. }));
    }

    #[test]
    fn inject_and_spread() {
        let mut gp = GossipProtocol::new(3);
        for i in 1..=5 { gp.add_node(i).unwrap(); }
        for i in 1..=5 { for j in 1..=5 { if i != j { gp.add_peer(i, j).unwrap(); } } }
        let mid = gp.inject(1, vec![42]).unwrap();
        for _ in 0..5 { gp.round(); }
        for i in 1..=5 { assert!(gp.is_infected(i, mid)); }
        assert_eq!(gp.convergence(mid), 1.0);
    }

    #[test]
    fn convergence_partial() {
        let mut gp = GossipProtocol::new(1);
        gp.add_node(1).unwrap(); gp.add_node(2).unwrap(); gp.add_node(3).unwrap(); gp.add_node(4).unwrap();
        gp.add_peer(1, 2).unwrap();
        let mid = gp.inject(1, vec![1]).unwrap();
        gp.round();
        assert!(gp.infected_count(mid) < 4);
        assert!(gp.convergence(mid) < 1.0);
    }

    #[test]
    fn not_found() {
        let mut gp = GossipProtocol::new(2);
        let err = gp.inject(99, vec![]).unwrap_err();
        assert!(matches!(err, GossipError::NodeNotFound { .. }));
    }

    #[test]
    fn node_not_found_peer() {
        let mut gp = GossipProtocol::new(2);
        gp.add_node(1).unwrap();
        let err = gp.add_peer(1, 99).unwrap_err();
        assert!(matches!(err, GossipError::NodeNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut gp = GossipProtocol::new(3);
        for i in 1..=4 { gp.add_node(i).unwrap(); }
        for i in 1..=4 { for j in 1..=4 { if i != j { gp.add_peer(i, j).unwrap(); } } }
        gp.inject(1, vec![1]).unwrap();
        gp.round();
        assert!(gp.total_deliveries() > 0);
        assert!(gp.total_rounds() > 0);
    }

    #[test]
    fn multiple_messages() {
        let mut gp = GossipProtocol::new(3);
        gp.add_node(1).unwrap(); gp.add_node(2).unwrap();
        gp.add_peer(1, 2).unwrap(); gp.add_peer(2, 1).unwrap();
        let m1 = gp.inject(1, vec![1]).unwrap();
        let m2 = gp.inject(2, vec![2]).unwrap();
        for _ in 0..3 { gp.round(); }
        assert!(gp.is_infected(1, m2));
        assert!(gp.is_infected(2, m1));
    }

    #[test]
    fn empty_round() {
        let mut gp = GossipProtocol::new(2);
        gp.add_node(1).unwrap();
        let rounds = gp.round();
        assert!(rounds.is_empty());
    }

    #[test]
    fn msg_count() {
        let mut gp = GossipProtocol::new(2);
        gp.add_node(1).unwrap();
        gp.inject(1, vec![]).unwrap();
        gp.inject(1, vec![]).unwrap();
        assert_eq!(gp.msg_count(), 2);
    }

    #[test]
    fn error_display() {
        assert!(GossipError::NodeNotFound { id: 5 }.to_string().contains("5"));
    }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState { Follower, Candidate, Leader }

#[derive(Debug, Clone, PartialEq)]
pub enum ElectError {
    NodeExists { id: u64 },
    NodeNotFound { id: u64 },
    AlreadyVoted { term: u64, voter: u64 },
    StaleTerm { term: u64, current: u64 },
}

impl std::fmt::Display for ElectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElectError::NodeExists { id } => write!(f, "node {id} exists"),
            ElectError::NodeNotFound { id } => write!(f, "node {id} not found"),
            ElectError::AlreadyVoted { term, voter } => write!(f, "term {term}: voter {voter} already voted"),
            ElectError::StaleTerm { term, current } => write!(f, "stale term {term} (current {current})"),
        }
    }
}

impl std::error::Error for ElectError {}

struct Node {
    id: u64,
    state: NodeState,
    current_term: u64,
    voted_for: Option<u64>,
}

pub struct Election {
    nodes: BTreeMap<u64, Node>,
    leader: Option<u64>,
    cluster_size: usize,
    total_elections: u64,
    total_term_advances: u64,
}

impl Election {
    pub fn new(cluster_size: usize) -> Self {
        Self { nodes: BTreeMap::new(), leader: None, cluster_size, total_elections: 0, total_term_advances: 0 }
    }

    pub fn add_node(&mut self, id: u64) -> Result<(), ElectError> {
        if self.nodes.contains_key(&id) { return Err(ElectError::NodeExists { id }); }
        self.nodes.insert(id, Node { id, state: NodeState::Follower, current_term: 0, voted_for: None });
        Ok(())
    }

    pub fn start_election(&mut self, candidate: u64) -> Result<(u64, NodeState), ElectError> {
        let node = self.nodes.get_mut(&candidate).ok_or(ElectError::NodeNotFound { id: candidate })?;
        node.current_term += 1;
        node.state = NodeState::Candidate;
        node.voted_for = Some(candidate);
        self.total_elections += 1;
        self.total_term_advances += 1;
        let term = node.current_term;
        Ok((term, NodeState::Candidate))
    }

    pub fn request_vote(&mut self, candidate: u64, voter: u64, term: u64) -> Result<bool, ElectError> {
        if !self.nodes.contains_key(&candidate) { return Err(ElectError::NodeNotFound { id: candidate }); }
        let voter_node = self.nodes.get_mut(&voter).ok_or(ElectError::NodeNotFound { id: voter })?;
        if term < voter_node.current_term { return Err(ElectError::StaleTerm { term, current: voter_node.current_term }); }
        if term > voter_node.current_term {
            voter_node.current_term = term;
            voter_node.voted_for = None;
            voter_node.state = NodeState::Follower;
            self.total_term_advances += 1;
        }
        if voter_node.voted_for.is_some() && voter_node.voted_for != Some(candidate) {
            return Err(ElectError::AlreadyVoted { term, voter });
        }
        voter_node.voted_for = Some(candidate);
        Ok(true)
    }

    pub fn count_votes(&mut self, candidate: u64, term: u64) -> Option<NodeState> {
        let votes = self.nodes.values().filter(|n| n.voted_for == Some(candidate) && n.current_term == term).count();
        let majority = self.nodes.len() / 2 + 1;
        if votes >= majority {
            if let Some(n) = self.nodes.get_mut(&candidate) {
                n.state = NodeState::Leader;
                self.leader = Some(candidate);
            }
            Some(NodeState::Leader)
        } else { Some(NodeState::Candidate) }
    }

    pub fn leader(&self) -> Option<u64> { self.leader }
    pub fn state(&self, id: u64) -> Option<NodeState> { self.nodes.get(&id).map(|n| n.state) }
    pub fn term(&self, id: u64) -> Option<u64> { self.nodes.get(&id).map(|n| n.current_term) }
    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn total_elections(&self) -> u64 { self.total_elections }
    pub fn total_term_advances(&self) -> u64 { self.total_term_advances }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_election() {
        let e = Election::new(3);
        assert_eq!(e.node_count(), 0);
        assert_eq!(e.leader(), None);
    }

    #[test]
    fn add_nodes() {
        let mut e = Election::new(3);
        e.add_node(1).unwrap(); e.add_node(2).unwrap(); e.add_node(3).unwrap();
        assert_eq!(e.node_count(), 3);
    }

    #[test]
    fn start_election() {
        let mut e = Election::new(3);
        e.add_node(1).unwrap(); e.add_node(2).unwrap(); e.add_node(3).unwrap();
        let (term, state) = e.start_election(1).unwrap();
        assert_eq!(term, 1);
        assert_eq!(state, NodeState::Candidate);
    }

    #[test]
    fn grant_vote() {
        let mut e = Election::new(3);
        e.add_node(1).unwrap(); e.add_node(2).unwrap(); e.add_node(3).unwrap();
        e.start_election(1).unwrap();
        assert!(e.request_vote(1, 2, 1).unwrap());
    }

    #[test]
    fn double_vote_rejected() {
        let mut e = Election::new(3);
        e.add_node(1).unwrap(); e.add_node(2).unwrap(); e.add_node(3).unwrap();
        e.start_election(1).unwrap();
        e.request_vote(1, 2, 1).unwrap();
        let err = e.request_vote(3, 2, 1).unwrap_err();
        assert!(matches!(err, ElectError::AlreadyVoted { .. }));
    }

    #[test]
    fn win_election() {
        let mut e = Election::new(3);
        e.add_node(1).unwrap(); e.add_node(2).unwrap(); e.add_node(3).unwrap();
        e.start_election(1).unwrap();
        e.request_vote(1, 2, 1).unwrap();
        let state = e.count_votes(1, 1).unwrap();
        assert_eq!(state, NodeState::Leader);
        assert_eq!(e.leader(), Some(1));
    }

    #[test]
    fn not_enough_votes() {
        let mut e = Election::new(5);
        for i in 1..=5 { e.add_node(i).unwrap(); }
        e.start_election(1).unwrap();
        e.request_vote(1, 2, 1).unwrap();
        let state = e.count_votes(1, 1).unwrap();
        assert_eq!(state, NodeState::Candidate);
    }

    #[test]
    fn duplicate_node() {
        let mut e = Election::new(3);
        e.add_node(1).unwrap();
        let err = e.add_node(1).unwrap_err();
        assert!(matches!(err, ElectError::NodeExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut e = Election::new(3);
        let err = e.start_election(99).unwrap_err();
        assert!(matches!(err, ElectError::NodeNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut e = Election::new(3);
        e.add_node(1).unwrap(); e.add_node(2).unwrap(); e.add_node(3).unwrap();
        e.start_election(1).unwrap();
        assert_eq!(e.total_elections(), 1);
        assert_eq!(e.total_term_advances(), 1);
    }

    #[test]
    fn error_display() {
        assert!(ElectError::NodeNotFound { id: 5 }.to_string().contains("5"));
    }
}

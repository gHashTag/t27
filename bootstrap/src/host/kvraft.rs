use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum KvErr {
    NotLeader { id: u64 },
    StaleTerm { current: u64, proposed: u64 },
    NotFound { key: Vec<u8> },
}

impl std::fmt::Display for KvErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KvErr::NotLeader { id } => write!(f, "not leader (node {id})"),
            KvErr::StaleTerm { current, proposed } => write!(f, "stale term {proposed} < {current}"),
            KvErr::NotFound { key } => write!(f, "key {:?} not found", key),
        }
    }
}

impl std::error::Error for KvErr {}

#[derive(Clone)]
struct LogEntry {
    term: u64,
    key: Vec<u8>,
    value: Option<Vec<u8>>,
}

struct Node {
    id: u64,
    term: u64,
    voted_for: Option<u64>,
    log: Vec<LogEntry>,
    data: BTreeMap<Vec<u8>, Vec<u8>>,
    is_leader: bool,
}

pub struct KvRaft {
    nodes: BTreeMap<u64, Node>,
    leader: Option<u64>,
    total_proposals: u64,
    total_reads: u64,
    total_elections: u64,
}

impl KvRaft {
    pub fn new(node_ids: &[u64]) -> Self {
        let mut nodes = BTreeMap::new();
        for &id in node_ids { nodes.insert(id, Node { id, term: 0, voted_for: None, log: Vec::new(), data: BTreeMap::new(), is_leader: false }); }
        Self { nodes, leader: None, total_proposals: 0, total_reads: 0, total_elections: 0 }
    }

    pub fn elect(&mut self, candidate: u64) -> Result<u64, KvErr> {
        self.total_elections += 1;
        let n = self.nodes.get_mut(&candidate).ok_or(KvErr::NotLeader { id: candidate })?;
        n.term += 1;
        n.voted_for = Some(candidate);
        let term = n.term;
        let mut votes = 1usize;
        let total = self.nodes.len();
        for (&nid, nn) in self.nodes.iter_mut() {
            if nid != candidate && nn.term <= term {
                nn.term = term;
                nn.voted_for = Some(candidate);
                nn.is_leader = false;
                votes += 1;
            }
        }
        if votes * 2 > total {
            if let Some(old) = self.leader { if let Some(on) = self.nodes.get_mut(&old) { on.is_leader = false; } }
            self.nodes.get_mut(&candidate).unwrap().is_leader = true;
            self.leader = Some(candidate);
            Ok(term)
        } else {
            Err(KvErr::NotLeader { id: candidate })
        }
    }

    pub fn propose(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<u64, KvErr> {
        let leader_id = self.leader.ok_or(KvErr::NotLeader { id: 0 })?;
        self.total_proposals += 1;
        let term = self.nodes.get(&leader_id).unwrap().term;
        for n in self.nodes.values_mut() {
            n.log.push(LogEntry { term, key: key.clone(), value: Some(value.clone()) });
            n.data.insert(key.clone(), value.clone());
        }
        Ok(term)
    }

    pub fn read(&mut self, node: u64, key: &[u8]) -> Result<Vec<u8>, KvErr> {
        self.total_reads += 1;
        let n = self.nodes.get(&node).ok_or(KvErr::NotLeader { id: node })?;
        n.data.get(key).cloned().ok_or(KvErr::NotFound { key: key.to_vec() })
    }

    pub fn delete(&mut self, key: &[u8]) -> Result<u64, KvErr> {
        let leader_id = self.leader.ok_or(KvErr::NotLeader { id: 0 })?;
        self.total_proposals += 1;
        let term = self.nodes.get(&leader_id).unwrap().term;
        for n in self.nodes.values_mut() {
            n.log.push(LogEntry { term, key: key.to_vec(), value: None });
            n.data.remove(key);
        }
        Ok(term)
    }

    pub fn leader(&self) -> Option<u64> { self.leader }
    pub fn term(&self, node: u64) -> Option<u64> { self.nodes.get(&node).map(|n| n.term) }
    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn log_len(&self, node: u64) -> Option<usize> { self.nodes.get(&node).map(|n| n.log.len()) }
    pub fn total_proposals(&self) -> u64 { self.total_proposals }
    pub fn total_reads(&self) -> u64 { self.total_reads }
    pub fn total_elections(&self) -> u64 { self.total_elections }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elect_leader() {
        let mut kr = KvRaft::new(&[1, 2, 3]);
        let term = kr.elect(1).unwrap();
        assert_eq!(kr.leader(), Some(1));
        assert!(term > 0);
    }

    #[test]
    fn propose_read() {
        let mut kr = KvRaft::new(&[1, 2, 3]);
        kr.elect(1).unwrap();
        kr.propose(b"k".to_vec(), b"v".to_vec()).unwrap();
        assert_eq!(kr.read(2, b"k").unwrap(), b"v");
    }

    #[test]
    fn read_missing() {
        let mut kr = KvRaft::new(&[1, 2, 3]);
        kr.elect(1).unwrap();
        assert!(kr.read(1, b"x").is_err());
    }

    #[test]
    fn no_leader_propose() { assert!(KvRaft::new(&[1, 2, 3]).propose(b"k".to_vec(), b"v".to_vec()).is_err()); }

    #[test]
    fn re_elect() {
        let mut kr = KvRaft::new(&[1, 2, 3]);
        kr.elect(1).unwrap();
        kr.elect(2).unwrap();
        assert_eq!(kr.leader(), Some(2));
    }

    #[test]
    fn delete() {
        let mut kr = KvRaft::new(&[1, 2, 3]);
        kr.elect(1).unwrap();
        kr.propose(b"k".to_vec(), b"v".to_vec()).unwrap();
        kr.delete(b"k").unwrap();
        assert!(kr.read(1, b"k").is_err());
    }

    #[test]
    fn log_replicated() {
        let mut kr = KvRaft::new(&[1, 2, 3]);
        kr.elect(1).unwrap();
        kr.propose(b"a".to_vec(), b"1".to_vec()).unwrap();
        kr.propose(b"b".to_vec(), b"2".to_vec()).unwrap();
        assert_eq!(kr.log_len(1), Some(2));
        assert_eq!(kr.log_len(2), Some(2));
    }

    #[test]
    fn stats() {
        let mut kr = KvRaft::new(&[1, 2, 3]);
        kr.elect(1).unwrap();
        kr.propose(b"k".to_vec(), b"v".to_vec()).unwrap();
        kr.read(1, b"k").unwrap();
        assert_eq!(kr.total_elections(), 1);
        assert_eq!(kr.total_proposals(), 1);
        assert_eq!(kr.total_reads(), 1);
    }

    #[test]
    fn error_display() { assert!(KvErr::NotLeader { id: 5 }.to_string().contains("leader")); }
}

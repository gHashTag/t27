use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vote {
    Yes,
    No,
    Veto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BallotState {
    Open,
    Accepted,
    Rejected,
    Vetoed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BallotError {
    BallotExists { id: u64 },
    BallotNotFound { id: u64 },
    BallotClosed { id: u64 },
    AlreadyVoted { ballot: u64, voter: u64 },
    UnknownVoter { voter: u64 },
}

impl std::fmt::Display for BallotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BallotError::BallotExists { id } => write!(f, "ballot {id} exists"),
            BallotError::BallotNotFound { id } => write!(f, "ballot {id} not found"),
            BallotError::BallotClosed { id } => write!(f, "ballot {id} closed"),
            BallotError::AlreadyVoted { ballot, voter } =>
                write!(f, "voter {voter} already voted on {ballot}"),
            BallotError::UnknownVoter { voter } => write!(f, "voter {voter} unknown"),
        }
    }
}

impl std::error::Error for BallotError {}

struct Ballot {
    id: u64,
    term: u64,
    quorum: usize,
    votes: BTreeMap<u64, Vote>,
    state: BallotState,
}

#[derive(Debug, Clone)]
pub struct BallotResult {
    pub id: u64,
    pub term: u64,
    pub state: BallotState,
    pub yes: usize,
    pub no: usize,
    pub veto: usize,
    pub total_voters: usize,
}

pub struct ConsensusBallot {
    ballots: BTreeMap<u64, Ballot>,
    voters: BTreeSet<u64>,
    next_id: u64,
    total_ballots: u64,
}

impl ConsensusBallot {
    pub fn new() -> Self { Self { ballots: BTreeMap::new(), voters: BTreeSet::new(), next_id: 1, total_ballots: 0 } }

    pub fn register_voter(&mut self, id: u64) { self.voters.insert(id); }

    pub fn create_ballot(&mut self, term: u64, quorum: usize) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.ballots.insert(id, Ballot { id, term, quorum, votes: BTreeMap::new(), state: BallotState::Open });
        self.total_ballots += 1;
        id
    }

    pub fn vote(&mut self, ballot_id: u64, voter: u64, vote: Vote) -> Result<BallotState, BallotError> {
        if !self.voters.contains(&voter) { return Err(BallotError::UnknownVoter { voter }); }
        let b = self.ballots.get_mut(&ballot_id).ok_or(BallotError::BallotNotFound { id: ballot_id })?;
        if b.state != BallotState::Open { return Err(BallotError::BallotClosed { id: ballot_id }); }
        if b.votes.contains_key(&voter) { return Err(BallotError::AlreadyVoted { ballot: ballot_id, voter }); }
        b.votes.insert(voter, vote);
        if vote == Vote::Veto { b.state = BallotState::Vetoed; return Ok(b.state); }
        let yes_count = b.votes.values().filter(|v| **v == Vote::Yes).count();
        let no_count = b.votes.values().filter(|v| **v == Vote::No).count();
        let total = b.votes.len();
        if yes_count >= b.quorum { b.state = BallotState::Accepted; }
        else if self.voters.len() - no_count < b.quorum { b.state = BallotState::Rejected; }
        Ok(b.state)
    }

    pub fn result(&self, ballot_id: u64) -> Option<BallotResult> {
        self.ballots.get(&ballot_id).map(|b| BallotResult {
            id: b.id, term: b.term, state: b.state,
            yes: b.votes.values().filter(|v| **v == Vote::Yes).count(),
            no: b.votes.values().filter(|v| **v == Vote::No).count(),
            veto: b.votes.values().filter(|v| **v == Vote::Veto).count(),
            total_voters: self.voters.len(),
        })
    }

    pub fn voter_count(&self) -> usize { self.voters.len() }
    pub fn ballot_count(&self) -> usize { self.ballots.len() }
    pub fn total_ballots(&self) -> u64 { self.total_ballots }
}

impl Default for ConsensusBallot {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_consensus() {
        let cb = ConsensusBallot::new();
        assert_eq!(cb.voter_count(), 0);
    }

    #[test]
    fn register_voters() {
        let mut cb = ConsensusBallot::new();
        cb.register_voter(1); cb.register_voter(2); cb.register_voter(3);
        assert_eq!(cb.voter_count(), 3);
    }

    #[test]
    fn create_ballot() {
        let mut cb = ConsensusBallot::new();
        let id = cb.create_ballot(1, 2);
        assert_eq!(cb.ballot_count(), 1);
        let r = cb.result(id).unwrap();
        assert_eq!(r.state, BallotState::Open);
    }

    #[test]
    fn quorum_accept() {
        let mut cb = ConsensusBallot::new();
        cb.register_voter(1); cb.register_voter(2); cb.register_voter(3);
        let id = cb.create_ballot(1, 2);
        let s = cb.vote(id, 1, Vote::Yes).unwrap();
        assert_eq!(s, BallotState::Open);
        let s = cb.vote(id, 2, Vote::Yes).unwrap();
        assert_eq!(s, BallotState::Accepted);
    }

    #[test]
    fn veto() {
        let mut cb = ConsensusBallot::new();
        cb.register_voter(1); cb.register_voter(2); cb.register_voter(3);
        let id = cb.create_ballot(1, 3);
        cb.vote(id, 1, Vote::Yes).unwrap();
        let s = cb.vote(id, 2, Vote::Veto).unwrap();
        assert_eq!(s, BallotState::Vetoed);
    }

    #[test]
    fn already_voted() {
        let mut cb = ConsensusBallot::new();
        cb.register_voter(1); cb.register_voter(2);
        let id = cb.create_ballot(1, 2);
        cb.vote(id, 1, Vote::Yes).unwrap();
        let err = cb.vote(id, 1, Vote::No).unwrap_err();
        assert!(matches!(err, BallotError::AlreadyVoted { .. }));
    }

    #[test]
    fn unknown_voter() {
        let mut cb = ConsensusBallot::new();
        let id = cb.create_ballot(1, 1);
        let err = cb.vote(id, 99, Vote::Yes).unwrap_err();
        assert!(matches!(err, BallotError::UnknownVoter { .. }));
    }

    #[test]
    fn ballot_not_found() {
        let cb = ConsensusBallot::new();
        assert!(cb.result(999).is_none());
    }

    #[test]
    fn result_stats() {
        let mut cb = ConsensusBallot::new();
        cb.register_voter(1); cb.register_voter(2); cb.register_voter(3);
        let id = cb.create_ballot(1, 2);
        cb.vote(id, 1, Vote::Yes).unwrap();
        cb.vote(id, 2, Vote::No).unwrap();
        let r = cb.result(id).unwrap();
        assert_eq!(r.yes, 1);
        assert_eq!(r.no, 1);
    }

    #[test]
    fn closed_ballot() {
        let mut cb = ConsensusBallot::new();
        cb.register_voter(1); cb.register_voter(2);
        let id = cb.create_ballot(1, 1);
        cb.vote(id, 1, Vote::Yes).unwrap();
        let err = cb.vote(id, 2, Vote::No).unwrap_err();
        assert!(matches!(err, BallotError::BallotClosed { .. }));
    }

    #[test]
    fn total_ballots() {
        let mut cb = ConsensusBallot::new();
        cb.create_ballot(1, 1);
        cb.create_ballot(2, 2);
        assert_eq!(cb.total_ballots(), 2);
    }

    #[test]
    fn error_display() {
        assert!(BallotError::UnknownVoter { voter: 5 }.to_string().contains("5"));
    }
}

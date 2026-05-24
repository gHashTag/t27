use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxnState { Active, Prepared, Committed, Aborted }

#[derive(Debug, Clone, PartialEq)]
pub enum TpcError {
    TxnNotFound { id: u64 },
    ParticipantNotFound { txn: u64, participant: u64 },
    AlreadyResolved { id: u64, state: TxnState },
    NotAllPrepared { id: u64, prepared: usize, total: usize },
}

impl std::fmt::Display for TpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TpcError::TxnNotFound { id } => write!(f, "txn {id} not found"),
            TpcError::ParticipantNotFound { txn, participant } =>
                write!(f, "txn {txn}: participant {participant} not found"),
            TpcError::AlreadyResolved { id, state } =>
                write!(f, "txn {id} already {state:?}"),
            TpcError::NotAllPrepared { id, prepared, total } =>
                write!(f, "txn {id}: {prepared}/{total} prepared"),
        }
    }
}

impl std::error::Error for TpcError {}

struct Participant {
    id: u64,
    prepared: bool,
    committed: bool,
}

struct Txn {
    id: u64,
    state: TxnState,
    participants: Vec<Participant>,
}

pub struct TwoPhaseCommit {
    txns: BTreeMap<u64, Txn>,
    next_id: u64,
    total_committed: u64,
    total_aborted: u64,
}

impl TwoPhaseCommit {
    pub fn new() -> Self { Self { txns: BTreeMap::new(), next_id: 1, total_committed: 0, total_aborted: 0 } }

    pub fn begin(&mut self, participant_ids: Vec<u64>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.txns.insert(id, Txn {
            id, state: TxnState::Active,
            participants: participant_ids.into_iter().map(|pid| Participant { id: pid, prepared: false, committed: false }).collect(),
        });
        id
    }

    pub fn prepare(&mut self, txn_id: u64, participant: u64) -> Result<TxnState, TpcError> {
        let txn = self.txns.get_mut(&txn_id).ok_or(TpcError::TxnNotFound { id: txn_id })?;
        if txn.state != TxnState::Active { return Err(TpcError::AlreadyResolved { id: txn_id, state: txn.state }); }
        let p = txn.participants.iter_mut().find(|p| p.id == participant)
            .ok_or(TpcError::ParticipantNotFound { txn: txn_id, participant })?;
        p.prepared = true;
        let all_prepared = txn.participants.iter().all(|p| p.prepared);
        if all_prepared { txn.state = TxnState::Prepared; }
        Ok(txn.state)
    }

    pub fn commit(&mut self, txn_id: u64) -> Result<TxnState, TpcError> {
        let txn = self.txns.get_mut(&txn_id).ok_or(TpcError::TxnNotFound { id: txn_id })?;
        if txn.state == TxnState::Committed || txn.state == TxnState::Aborted {
            return Err(TpcError::AlreadyResolved { id: txn_id, state: txn.state });
        }
        if txn.state != TxnState::Prepared {
            let prepared = txn.participants.iter().filter(|p| p.prepared).count();
            return Err(TpcError::NotAllPrepared { id: txn_id, prepared, total: txn.participants.len() });
        }
        for p in &mut txn.participants { p.committed = true; }
        txn.state = TxnState::Committed;
        self.total_committed += 1;
        Ok(TxnState::Committed)
    }

    pub fn abort(&mut self, txn_id: u64) -> Result<TxnState, TpcError> {
        let txn = self.txns.get_mut(&txn_id).ok_or(TpcError::TxnNotFound { id: txn_id })?;
        if txn.state == TxnState::Committed || txn.state == TxnState::Aborted {
            return Err(TpcError::AlreadyResolved { id: txn_id, state: txn.state });
        }
        txn.state = TxnState::Aborted;
        self.total_aborted += 1;
        Ok(TxnState::Aborted)
    }

    pub fn state(&self, txn_id: u64) -> Option<TxnState> { self.txns.get(&txn_id).map(|t| t.state) }
    pub fn txn_count(&self) -> usize { self.txns.len() }
    pub fn total_committed(&self) -> u64 { self.total_committed }
    pub fn total_aborted(&self) -> u64 { self.total_aborted }
}

impl Default for TwoPhaseCommit {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_2pc() { assert_eq!(TwoPhaseCommit::new().txn_count(), 0); }

    #[test]
    fn begin_txn() {
        let mut t = TwoPhaseCommit::new();
        let id = t.begin(vec![1, 2, 3]);
        assert_eq!(t.state(id), Some(TxnState::Active));
    }

    #[test]
    fn prepare_all() {
        let mut t = TwoPhaseCommit::new();
        let id = t.begin(vec![1, 2]);
        let s = t.prepare(id, 1).unwrap();
        assert_eq!(s, TxnState::Active);
        let s = t.prepare(id, 2).unwrap();
        assert_eq!(s, TxnState::Prepared);
    }

    #[test]
    fn commit() {
        let mut t = TwoPhaseCommit::new();
        let id = t.begin(vec![1, 2]);
        t.prepare(id, 1).unwrap(); t.prepare(id, 2).unwrap();
        let s = t.commit(id).unwrap();
        assert_eq!(s, TxnState::Committed);
        assert_eq!(t.total_committed(), 1);
    }

    #[test]
    fn abort() {
        let mut t = TwoPhaseCommit::new();
        let id = t.begin(vec![1, 2]);
        t.prepare(id, 1).unwrap();
        let s = t.abort(id).unwrap();
        assert_eq!(s, TxnState::Aborted);
        assert_eq!(t.total_aborted(), 1);
    }

    #[test]
    fn commit_without_prepare() {
        let mut t = TwoPhaseCommit::new();
        let id = t.begin(vec![1, 2]);
        let err = t.commit(id).unwrap_err();
        assert!(matches!(err, TpcError::NotAllPrepared { .. }));
    }

    #[test]
    fn already_committed() {
        let mut t = TwoPhaseCommit::new();
        let id = t.begin(vec![1]);
        t.prepare(id, 1).unwrap(); t.commit(id).unwrap();
        let err = t.commit(id).unwrap_err();
        assert!(matches!(err, TpcError::AlreadyResolved { .. }));
    }

    #[test]
    fn txn_not_found() {
        let t = TwoPhaseCommit::new();
        assert_eq!(t.state(99), None);
    }

    #[test]
    fn participant_not_found() {
        let mut t = TwoPhaseCommit::new();
        let id = t.begin(vec![1, 2]);
        let err = t.prepare(id, 99).unwrap_err();
        assert!(matches!(err, TpcError::ParticipantNotFound { .. }));
    }

    #[test]
    fn abort_after_commit() {
        let mut t = TwoPhaseCommit::new();
        let id = t.begin(vec![1]);
        t.prepare(id, 1).unwrap(); t.commit(id).unwrap();
        let err = t.abort(id).unwrap_err();
        assert!(matches!(err, TpcError::AlreadyResolved { .. }));
    }

    #[test]
    fn stats() {
        let mut t = TwoPhaseCommit::new();
        let id1 = t.begin(vec![1]); t.prepare(id1, 1).unwrap(); t.commit(id1).unwrap();
        let id2 = t.begin(vec![1]); t.abort(id2).unwrap();
        assert_eq!(t.total_committed(), 1);
        assert_eq!(t.total_aborted(), 1);
    }

    #[test]
    fn error_display() {
        assert!(TpcError::TxnNotFound { id: 5 }.to_string().contains("5"));
    }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum IntentState { Proposed, Committed, Aborted }

#[derive(Debug, Clone, PartialEq)]
pub enum IntentError {
    IntentNotFound { id: u64 },
    AlreadyResolved { id: u64, state: IntentState },
    Conflict { id: u64, conflicting: u64, key: Vec<u8> },
}

impl std::fmt::Display for IntentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentError::IntentNotFound { id } => write!(f, "intent {id} not found"),
            IntentError::AlreadyResolved { id, state } => write!(f, "intent {id}: {:?}", state),
            IntentError::Conflict { id, conflicting, key } => write!(f, "intent {id} conflicts with {conflicting} on {:?}", key),
        }
    }
}

impl std::error::Error for IntentError {}

struct Intent {
    id: u64,
    keys: Vec<Vec<u8>>,
    state: IntentState,
    proposer: u64,
}

pub struct IntentLog {
    intents: BTreeMap<u64, Intent>,
    key_locks: BTreeMap<Vec<u8>, u64>,
    next_id: u64,
    total_proposed: u64,
    total_committed: u64,
    total_aborted: u64,
    total_conflicts: u64,
}

impl IntentLog {
    pub fn new() -> Self { Self { intents: BTreeMap::new(), key_locks: BTreeMap::new(), next_id: 1, total_proposed: 0, total_committed: 0, total_aborted: 0, total_conflicts: 0 } }

    pub fn propose(&mut self, proposer: u64, keys: Vec<Vec<u8>>) -> Result<u64, IntentError> {
        for key in &keys {
            if let Some(&holder) = self.key_locks.get(key) {
                self.total_conflicts += 1;
                return Err(IntentError::Conflict { id: self.next_id, conflicting: holder, key: key.clone() });
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        for key in &keys { self.key_locks.insert(key.clone(), id); }
        self.intents.insert(id, Intent { id, keys, state: IntentState::Proposed, proposer });
        self.total_proposed += 1;
        Ok(id)
    }

    pub fn commit(&mut self, id: u64) -> Result<(), IntentError> {
        let intent = self.intents.get_mut(&id).ok_or(IntentError::IntentNotFound { id })?;
        if intent.state != IntentState::Proposed {
            return Err(IntentError::AlreadyResolved { id, state: intent.state.clone() });
        }
        intent.state = IntentState::Committed;
        for key in &intent.keys { self.key_locks.remove(key); }
        self.total_committed += 1;
        Ok(())
    }

    pub fn abort(&mut self, id: u64) -> Result<(), IntentError> {
        let intent = self.intents.get_mut(&id).ok_or(IntentError::IntentNotFound { id })?;
        if intent.state != IntentState::Proposed {
            return Err(IntentError::AlreadyResolved { id, state: intent.state.clone() });
        }
        intent.state = IntentState::Aborted;
        for key in &intent.keys { self.key_locks.remove(key); }
        self.total_aborted += 1;
        Ok(())
    }

    pub fn state(&self, id: u64) -> Option<&IntentState> { self.intents.get(&id).map(|i| &i.state) }
    pub fn proposer(&self, id: u64) -> Option<u64> { self.intents.get(&id).map(|i| i.proposer) }
    pub fn locked_keys(&self) -> usize { self.key_locks.len() }
    pub fn intent_count(&self) -> usize { self.intents.len() }
    pub fn pending_count(&self) -> usize { self.intents.values().filter(|i| i.state == IntentState::Proposed).count() }
    pub fn total_proposed(&self) -> u64 { self.total_proposed }
    pub fn total_committed(&self) -> u64 { self.total_committed }
    pub fn total_aborted(&self) -> u64 { self.total_aborted }
    pub fn total_conflicts(&self) -> u64 { self.total_conflicts }
}

impl Default for IntentLog {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_log() { assert_eq!(IntentLog::new().intent_count(), 0); }

    #[test]
    fn propose_commit() {
        let mut il = IntentLog::new();
        let id = il.propose(1, vec![b"key1".to_vec()]).unwrap();
        il.commit(id).unwrap();
        assert_eq!(il.state(id), Some(&IntentState::Committed));
        assert_eq!(il.locked_keys(), 0);
    }

    #[test]
    fn propose_abort() {
        let mut il = IntentLog::new();
        let id = il.propose(1, vec![b"key1".to_vec()]).unwrap();
        il.abort(id).unwrap();
        assert_eq!(il.state(id), Some(&IntentState::Aborted));
        assert_eq!(il.locked_keys(), 0);
    }

    #[test]
    fn conflict_detection() {
        let mut il = IntentLog::new();
        il.propose(1, vec![b"key1".to_vec()]).unwrap();
        let err = il.propose(2, vec![b"key1".to_vec()]).unwrap_err();
        assert!(matches!(err, IntentError::Conflict { .. }));
        assert_eq!(il.total_conflicts(), 1);
    }

    #[test]
    fn no_conflict_after_commit() {
        let mut il = IntentLog::new();
        let id = il.propose(1, vec![b"key1".to_vec()]).unwrap();
        il.commit(id).unwrap();
        let id2 = il.propose(2, vec![b"key1".to_vec()]);
        assert!(id2.is_ok());
    }

    #[test]
    fn already_resolved() {
        let mut il = IntentLog::new();
        let id = il.propose(1, vec![b"k".to_vec()]).unwrap();
        il.commit(id).unwrap();
        let err = il.commit(id).unwrap_err();
        assert!(matches!(err, IntentError::AlreadyResolved { .. }));
    }

    #[test]
    fn not_found() {
        let mut il = IntentLog::new();
        let err = il.commit(99).unwrap_err();
        assert!(matches!(err, IntentError::IntentNotFound { .. }));
    }

    #[test]
    fn multi_key() {
        let mut il = IntentLog::new();
        let id = il.propose(1, vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]).unwrap();
        assert_eq!(il.locked_keys(), 3);
        il.commit(id).unwrap();
        assert_eq!(il.locked_keys(), 0);
    }

    #[test]
    fn proposer() {
        let mut il = IntentLog::new();
        let id = il.propose(42, vec![b"k".to_vec()]).unwrap();
        assert_eq!(il.proposer(id), Some(42));
    }

    #[test]
    fn stats() {
        let mut il = IntentLog::new();
        il.propose(1, vec![b"k".to_vec()]).unwrap();
        il.commit(1).unwrap();
        assert_eq!(il.total_proposed(), 1);
        assert_eq!(il.total_committed(), 1);
    }

    #[test]
    fn error_display() { assert!(IntentError::IntentNotFound { id: 3 }.to_string().contains("3")); }
}

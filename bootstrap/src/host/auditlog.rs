use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum AuditError {
    EntryNotFound { seq: u64 },
    AlreadySealed,
    ChainBroken { seq: u64, expected: u64, found: u64 },
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditError::EntryNotFound { seq } => write!(f, "entry {seq} not found"),
            AuditError::AlreadySealed => write!(f, "audit log sealed"),
            AuditError::ChainBroken { seq, expected, found } => write!(f, "chain broken at {seq}: expected {expected}, found {found}"),
        }
    }
}

impl std::error::Error for AuditError {}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub seq: u64,
    pub action: String,
    pub actor: u64,
    pub target: String,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub prev_hash: u64,
    pub hash: u64,
}

fn simple_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub struct AuditLog {
    entries: Vec<AuditEntry>,
    sealed: bool,
    filter_actor: Option<u64>,
    filter_action: Option<String>,
    total_appended: u64,
    total_verified: u64,
}

impl AuditLog {
    pub fn new() -> Self { Self { entries: Vec::new(), sealed: false, filter_actor: None, filter_action: None, total_appended: 0, total_verified: 0 } }

    pub fn append(&mut self, action: &str, actor: u64, target: &str, payload: Vec<u8>, timestamp: u64) -> Result<u64, AuditError> {
        if self.sealed { return Err(AuditError::AlreadySealed); }
        let seq = self.entries.len() as u64;
        let prev_hash = self.entries.last().map(|e| e.hash).unwrap_or(0);
        let mut data = Vec::new();
        data.extend_from_slice(&seq.to_le_bytes());
        data.extend_from_slice(action.as_bytes());
        data.extend_from_slice(&actor.to_le_bytes());
        data.extend_from_slice(target.as_bytes());
        data.extend_from_slice(&payload);
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&prev_hash.to_le_bytes());
        let hash = simple_hash(&data);
        self.entries.push(AuditEntry { seq, action: action.to_string(), actor, target: target.to_string(), payload, timestamp, prev_hash, hash });
        self.total_appended += 1;
        Ok(seq)
    }

    pub fn verify(&mut self) -> Result<u64, AuditError> {
        let mut prev: u64 = 0;
        for e in &self.entries {
            if e.prev_hash != prev {
                return Err(AuditError::ChainBroken { seq: e.seq, expected: prev, found: e.prev_hash });
            }
            prev = e.hash;
        }
        self.total_verified += 1;
        Ok(self.entries.len() as u64)
    }

    pub fn get(&self, seq: u64) -> Option<&AuditEntry> { self.entries.get(seq as usize) }

    pub fn seal(&mut self) { self.sealed = true; }

    pub fn by_actor(&self, actor: u64) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.actor == actor).collect()
    }

    pub fn by_action(&self, action: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.action == action).collect()
    }

    pub fn by_target(&self, target: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.target == target).collect()
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn is_sealed(&self) -> bool { self.sealed }
    pub fn total_appended(&self) -> u64 { self.total_appended }
    pub fn total_verified(&self) -> u64 { self.total_verified }
    pub fn last_hash(&self) -> Option<u64> { self.entries.last().map(|e| e.hash) }
}

impl Default for AuditLog {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_log() { assert!(AuditLog::new().is_empty()); }

    #[test]
    fn append_get() {
        let mut al = AuditLog::new();
        let seq = al.append("login", 1, "server", vec![], 100).unwrap();
        let e = al.get(seq).unwrap();
        assert_eq!(e.action, "login");
        assert_eq!(e.actor, 1);
    }

    #[test]
    fn chain_verify() {
        let mut al = AuditLog::new();
        al.append("a", 1, "x", vec![], 100).unwrap();
        al.append("b", 2, "y", vec![], 200).unwrap();
        let count = al.verify().unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn seal_blocks() {
        let mut al = AuditLog::new();
        al.seal();
        let err = al.append("a", 1, "x", vec![], 100).unwrap_err();
        assert!(matches!(err, AuditError::AlreadySealed));
    }

    #[test]
    fn by_actor() {
        let mut al = AuditLog::new();
        al.append("a", 1, "x", vec![], 100).unwrap();
        al.append("b", 2, "y", vec![], 200).unwrap();
        al.append("c", 1, "z", vec![], 300).unwrap();
        assert_eq!(al.by_actor(1).len(), 2);
    }

    #[test]
    fn by_action() {
        let mut al = AuditLog::new();
        al.append("read", 1, "x", vec![], 100).unwrap();
        al.append("write", 1, "x", vec![], 200).unwrap();
        assert_eq!(al.by_action("read").len(), 1);
    }

    #[test]
    fn by_target() {
        let mut al = AuditLog::new();
        al.append("a", 1, "db1", vec![], 100).unwrap();
        al.append("b", 2, "db2", vec![], 200).unwrap();
        assert_eq!(al.by_target("db1").len(), 1);
    }

    #[test]
    fn prev_hash_chain() {
        let mut al = AuditLog::new();
        al.append("a", 1, "x", vec![], 100).unwrap();
        let h1 = al.last_hash().unwrap();
        al.append("b", 2, "y", vec![], 200).unwrap();
        let e1 = al.get(1).unwrap();
        assert_eq!(e1.prev_hash, h1);
    }

    #[test]
    fn stats() {
        let mut al = AuditLog::new();
        al.append("a", 1, "x", vec![], 100).unwrap();
        al.verify().unwrap();
        assert_eq!(al.total_appended(), 1);
        assert_eq!(al.total_verified(), 1);
    }

    #[test]
    fn entry_not_found() { assert!(AuditLog::new().get(0).is_none()); }

    #[test]
    fn error_display() { assert!(AuditError::AlreadySealed.to_string().contains("sealed")); }
}

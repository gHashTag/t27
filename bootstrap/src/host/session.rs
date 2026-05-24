use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SessionState {
    Idle,
    Active,
    Suspended,
    Closed,
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionState::Idle => write!(f, "idle"),
            SessionState::Active => write!(f, "active"),
            SessionState::Suspended => write!(f, "suspended"),
            SessionState::Closed => write!(f, "closed"),
        }
    }
}

pub type SessionId = u64;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub state: SessionState,
    pub created_us: u64,
    pub last_active_us: u64,
    pub seq: u16,
    pub peer: String,
}

impl Session {
    pub fn new(id: SessionId, created_us: u64, peer: &str) -> Self {
        Self {
            id,
            state: SessionState::Active,
            created_us,
            last_active_us: created_us,
            seq: 0,
            peer: peer.to_string(),
        }
    }

    pub fn touch(&mut self, now_us: u64) {
        self.last_active_us = now_us;
        self.seq = self.seq.wrapping_add(1);
    }

    pub fn age_us(&self, now_us: u64) -> u64 {
        now_us.saturating_sub(self.created_us)
    }

    pub fn idle_us(&self, now_us: u64) -> u64 {
        now_us.saturating_sub(self.last_active_us)
    }

    pub fn is_expired(&self, now_us: u64, timeout_us: u64) -> bool {
        self.idle_us(now_us) > timeout_us
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    Duplicate { id: SessionId },
    NotFound { id: SessionId },
    AlreadyClosed { id: SessionId },
    InvalidState { id: SessionId, state: SessionState },
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::Duplicate { id } => write!(f, "duplicate session: {id}"),
            SessionError::NotFound { id } => write!(f, "session not found: {id}"),
            SessionError::AlreadyClosed { id } => write!(f, "session already closed: {id}"),
            SessionError::InvalidState { id, state } => {
                write!(f, "session {id}: invalid state {state}")
            }
        }
    }
}

impl std::error::Error for SessionError {}

#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions: BTreeMap<SessionId, Session>,
    next_id: SessionId,
    timeout_us: u64,
    total_created: u64,
    total_closed: u64,
}

impl SessionManager {
    pub fn new(timeout_us: u64) -> Self {
        Self {
            sessions: BTreeMap::new(),
            next_id: 1,
            timeout_us,
            total_created: 0,
            total_closed: 0,
        }
    }

    pub fn create(&mut self, now_us: u64, peer: &str) -> SessionId {
        let id = self.next_id;
        self.next_id += 1;
        let session = Session::new(id, now_us, peer);
        self.sessions.insert(id, session);
        self.total_created += 1;
        id
    }

    pub fn get(&self, id: SessionId) -> Option<&Session> {
        self.sessions.get(&id)
    }

    pub fn get_mut(&mut self, id: SessionId) -> Option<&mut Session> {
        self.sessions.get_mut(&id)
    }

    pub fn close(&mut self, id: SessionId) -> Result<(), SessionError> {
        let session = self
            .sessions
            .get_mut(&id)
            .ok_or(SessionError::NotFound { id })?;
        if session.state == SessionState::Closed {
            return Err(SessionError::AlreadyClosed { id });
        }
        session.state = SessionState::Closed;
        self.total_closed += 1;
        Ok(())
    }

    pub fn suspend(&mut self, id: SessionId) -> Result<(), SessionError> {
        let session = self
            .sessions
            .get_mut(&id)
            .ok_or(SessionError::NotFound { id })?;
        if session.state != SessionState::Active {
            return Err(SessionError::InvalidState {
                id,
                state: session.state,
            });
        }
        session.state = SessionState::Suspended;
        Ok(())
    }

    pub fn resume(&mut self, id: SessionId) -> Result<(), SessionError> {
        let session = self
            .sessions
            .get_mut(&id)
            .ok_or(SessionError::NotFound { id })?;
        if session.state != SessionState::Suspended {
            return Err(SessionError::InvalidState {
                id,
                state: session.state,
            });
        }
        session.state = SessionState::Active;
        Ok(())
    }

    pub fn expire(&mut self, now_us: u64) -> Vec<SessionId> {
        let timeout = self.timeout_us;
        let expired: Vec<SessionId> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.state == SessionState::Active && s.is_expired(now_us, timeout))
            .map(|(&id, _)| id)
            .collect();
        for &id in &expired {
            self.close(id).ok();
        }
        expired
    }

    pub fn active_sessions(&self) -> Vec<&Session> {
        self.sessions
            .values()
            .filter(|s| s.state == SessionState::Active)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn total_created(&self) -> u64 {
        self.total_created
    }

    pub fn total_closed(&self) -> u64 {
        self.total_closed
    }

    pub fn remove_closed(&mut self) -> usize {
        let before = self.sessions.len();
        self.sessions.retain(|_, s| s.state != SessionState::Closed);
        before - self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_display() {
        assert_eq!(SessionState::Active.to_string(), "active");
    }

    #[test]
    fn create_and_get() {
        let mut sm = SessionManager::new(1000);
        let id = sm.create(100, "peer1");
        let s = sm.get(id).unwrap();
        assert_eq!(s.state, SessionState::Active);
        assert_eq!(s.peer, "peer1");
        assert_eq!(sm.total_created(), 1);
    }

    #[test]
    fn touch_updates() {
        let mut sm = SessionManager::new(1000);
        let id = sm.create(100, "p");
        let s = sm.get_mut(id).unwrap();
        s.touch(200);
        assert_eq!(s.last_active_us, 200);
        assert_eq!(s.seq, 1);
    }

    #[test]
    fn age_and_idle() {
        let mut sm = SessionManager::new(1000);
        let id = sm.create(100, "p");
        let s = sm.get(id).unwrap();
        assert_eq!(s.age_us(200), 100);
        assert_eq!(s.idle_us(200), 100);
    }

    #[test]
    fn close_session() {
        let mut sm = SessionManager::new(1000);
        let id = sm.create(100, "p");
        sm.close(id).unwrap();
        assert_eq!(sm.get(id).unwrap().state, SessionState::Closed);
        assert_eq!(sm.total_closed(), 1);
    }

    #[test]
    fn close_not_found() {
        let mut sm = SessionManager::new(1000);
        let err = sm.close(999).unwrap_err();
        assert!(matches!(err, SessionError::NotFound { .. }));
    }

    #[test]
    fn close_already_closed() {
        let mut sm = SessionManager::new(1000);
        let id = sm.create(100, "p");
        sm.close(id).unwrap();
        let err = sm.close(id).unwrap_err();
        assert!(matches!(err, SessionError::AlreadyClosed { .. }));
    }

    #[test]
    fn suspend_resume() {
        let mut sm = SessionManager::new(1000);
        let id = sm.create(100, "p");
        sm.suspend(id).unwrap();
        assert_eq!(sm.get(id).unwrap().state, SessionState::Suspended);
        sm.resume(id).unwrap();
        assert_eq!(sm.get(id).unwrap().state, SessionState::Active);
    }

    #[test]
    fn suspend_wrong_state() {
        let mut sm = SessionManager::new(1000);
        let id = sm.create(100, "p");
        sm.close(id).unwrap();
        let err = sm.suspend(id).unwrap_err();
        assert!(matches!(err, SessionError::InvalidState { .. }));
    }

    #[test]
    fn expire_sessions() {
        let mut sm = SessionManager::new(100);
        let id1 = sm.create(0, "a");
        let _id2 = sm.create(150, "b");
        let expired = sm.expire(200);
        assert_eq!(expired, vec![id1]);
        assert_eq!(sm.get(id1).unwrap().state, SessionState::Closed);
        assert_eq!(sm.active_sessions().len(), 1);
    }

    #[test]
    fn remove_closed() {
        let mut sm = SessionManager::new(100);
        let id = sm.create(0, "p");
        sm.close(id).unwrap();
        assert_eq!(sm.remove_closed(), 1);
        assert_eq!(sm.len(), 0);
    }

    #[test]
    fn error_display() {
        assert!(SessionError::NotFound { id: 42 }.to_string().contains("42"));
        assert!(SessionError::AlreadyClosed { id: 1 }.to_string().contains("closed"));
    }
}

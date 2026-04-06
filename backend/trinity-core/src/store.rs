//! Trinity Core - In-Memory Store
//!
//! Thread-safe in-memory storage for sessions, messages, and parts.

use crate::models::{Message, Part, Session, SessionState};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Thread-safe storage for all data
#[derive(Debug, Default)]
pub struct Store {
    sessions: RwLock<HashMap<String, Session>>,
    messages: RwLock<HashMap<String, Vec<Message>>>,
    parts: RwLock<HashMap<String, Vec<Part>>>,
}

impl Store {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    // ─── Session Operations ──────────────────────────────────────────────────────

    pub fn create_session(&self, session: Session) -> Session {
        let mut sessions = self.sessions.write();
        sessions.insert(session.id.clone(), session.clone());
        session
    }

    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.read().get(session_id).cloned()
    }

    pub fn list_sessions(&self, directory: Option<&str>) -> Vec<Session> {
        let sessions = self.sessions.read();
        match directory {
            Some(dir) => sessions
                .values()
                .filter(|s| s.directory == dir)
                .cloned()
                .collect(),
            None => sessions.values().cloned().collect(),
        }
    }

    pub fn update_session(&self, session_id: &str, update: SessionUpdate) -> Option<Session> {
        let mut sessions = self.sessions.write();
        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(status) = update.status {
                session.state = status;
            }
            if let Some(title) = update.title {
                session.title = Some(title);
            }
            if let Some(summary) = update.summary {
                session.summary = Some(summary);
            }
            if let Some(archived) = update.archived {
                session.time.archived = Some(archived);
            }
            session.time.updated = chrono::Utc::now();
            return Some(session.clone());
        }
        None
    }

    pub fn delete_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.write();
        if sessions.remove(session_id).is_some() {
            // Also delete all messages and parts for this session
            self.messages.write().remove(session_id);
            // Clear parts for all messages in this session
            let mut parts = self.parts.write();
            parts.retain(|msg_id, _| !msg_id.starts_with(session_id));
            true
        } else {
            false
        }
    }

    // ─── Message Operations ────────────────────────────────────────────────────

    pub fn create_message(&self, session_id: &str, message: Message) -> Option<Message> {
        let mut messages = self.messages.write();
        let session_messages = messages.entry(session_id.to_string()).or_default();
        
        // Insert in order (binary search would be better for large lists)
        let msg_id = message.id();
        let insert_pos = session_messages
            .iter()
            .position(|m| m.id() > msg_id)
            .unwrap_or(session_messages.len());
        
        session_messages.insert(insert_pos, message.clone());
        Some(message)
    }

    pub fn get_message(&self, session_id: &str, message_id: &str) -> Option<Message> {
        self.messages
            .read()
            .get(session_id)
            .and_then(|msgs| msgs.iter().find(|m| m.id() == message_id).cloned())
    }

    pub fn list_messages(&self, session_id: &str) -> Vec<Message> {
        self.messages
            .read()
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn update_message(
        &self,
        session_id: &str,
        message_id: &str,
        update: MessageUpdate,
    ) -> Option<Message> {
        let mut messages = self.messages.write();
        if let Some(session_messages) = messages.get_mut(session_id) {
            for msg in session_messages {
                if msg.id() == message_id {
                    if let Message::Assistant(asm) = msg {
                        if let Some(completed) = update.completed {
                            if completed {
                                asm.time.completed = Some(chrono::Utc::now());
                            }
                        }
                        if let Some(cost) = update.cost {
                            asm.cost = cost;
                        }
                        if let Some(tokens) = &update.tokens {
                            asm.tokens = tokens.clone();
                        }
                        if let Some(error) = &update.error {
                            asm.error = Some(error.clone());
                        }
                        return Some(msg.clone());
                    }
                }
            }
        }
        None
    }

    pub fn delete_message(&self, session_id: &str, message_id: &str) -> bool {
        let mut messages = self.messages.write();
        if let Some(session_messages) = messages.get_mut(session_id) {
            let len_before = session_messages.len();
            session_messages.retain(|m| m.id() != message_id);
            if session_messages.len() < len_before {
                // Also delete parts for this message
                self.parts.write().remove(message_id);
                return true;
            }
        }
        false
    }

    // ─── Part Operations ───────────────────────────────────────────────────────

    pub fn add_part(&self, message_id: &str, part: Part) -> Option<Part> {
        let mut parts = self.parts.write();
        let message_parts = parts.entry(message_id.to_string()).or_default();
        message_parts.push(part.clone());
        Some(part)
    }

    pub fn list_parts(&self, message_id: &str) -> Vec<Part> {
        self.parts.read().get(message_id).cloned().unwrap_or_default()
    }

    pub fn get_part(&self, message_id: &str, part_id: &str) -> Option<Part> {
        self.parts
            .read()
            .get(message_id)
            .and_then(|parts| parts.iter().find(|p| p.id() == part_id).cloned())
    }

    #[allow(dead_code)]
    pub fn update_part_text(&self, message_id: &str, part_id: &str, text: &str) -> Option<()> {
        let mut parts = self.parts.write();
        if let Some(message_parts) = parts.get_mut(message_id) {
            for part in message_parts {
                if let Part::Text { id, text: t, .. } = part {
                    if id == part_id {
                        *t = text.to_string();
                        return Some(());
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Default)]
pub struct SessionUpdate {
    pub status: Option<SessionState>,
    pub title: Option<String>,
    pub summary: Option<crate::models::SessionSummary>,
    pub archived: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Default)]
pub struct MessageUpdate {
    pub completed: Option<bool>,
    pub cost: Option<f64>,
    pub tokens: Option<crate::models::MessageTokens>,
    pub error: Option<crate::models::MessageError>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AssistantMessage, ContentBlock, SessionState, UserMessage,
    };

    fn make_session(dir: &str) -> Session {
        Session::new(dir.to_string())
    }

    fn make_user_msg(session_id: &str, parent_id: &str) -> Message {
        Message::User(UserMessage::new(
            session_id.to_string(),
            parent_id.to_string(),
            vec![ContentBlock::text("b1".to_string(), "hello".to_string())],
        ))
    }

    fn make_assistant_msg(session_id: &str, parent_id: &str) -> Message {
        Message::Assistant(AssistantMessage::new(
            session_id.to_string(),
            parent_id.to_string(),
            "claude-sonnet-4-5".to_string(),
            "anthropic".to_string(),
        ))
    }

    #[test]
    fn session_crud() {
        let store = Store::new();
        let s = make_session("/tmp/test");
        let id = s.id.clone();

        let created = store.create_session(s);
        assert_eq!(created.id, id);

        let got = store.get_session(&id).unwrap();
        assert_eq!(got.id, id);
        assert_eq!(got.directory, "/tmp/test");

        let upd = SessionUpdate {
            status: Some(SessionState::Busy),
            title: Some("updated".to_string()),
            ..Default::default()
        };
        let updated = store.update_session(&id, upd).unwrap();
        assert_eq!(updated.state, SessionState::Busy);
        assert_eq!(updated.title.unwrap(), "updated");

        assert!(store.delete_session(&id));
        assert!(store.get_session(&id).is_none());
    }

    #[test]
    fn list_sessions_by_directory() {
        let store = Store::new();
        let s1 = make_session("/a");
        let s2 = make_session("/b");
        store.create_session(s1);
        store.create_session(s2);

        assert_eq!(store.list_sessions(None).len(), 2);
        assert_eq!(store.list_sessions(Some("/a")).len(), 1);
        assert_eq!(store.list_sessions(Some("/c")).len(), 0);
    }

    #[test]
    fn message_crud() {
        let store = Store::new();
        let s = make_session("/tmp/m");
        let sid = s.id.clone();
        store.create_session(s);

        let msg = make_user_msg(&sid, "root");
        let mid = msg.id().to_string();
        let created = store.create_message(&sid, msg).unwrap();
        assert_eq!(created.id(), mid);

        let got = store.get_message(&sid, &mid).unwrap();
        assert_eq!(got.id(), mid);

        let msgs = store.list_messages(&sid);
        assert_eq!(msgs.len(), 1);

        assert!(store.delete_message(&sid, &mid));
        assert!(store.get_message(&sid, &mid).is_none());
    }

    #[test]
    fn update_assistant_message() {
        let store = Store::new();
        let s = make_session("/tmp/um");
        let sid = s.id.clone();
        store.create_session(s);

        let msg = make_assistant_msg(&sid, "root");
        let mid = msg.id().to_string();
        store.create_message(&sid, msg);

        let upd = MessageUpdate {
            completed: Some(true),
            cost: Some(0.05),
            ..Default::default()
        };
        let updated = store.update_message(&sid, &mid, upd).unwrap();
        assert_eq!(updated.id(), mid);
    }

    #[test]
    fn parts_crud() {
        let store = Store::new();
        let part = Part::Text {
            id: "p1".to_string(),
            message_id: "m1".to_string(),
            text: "hello".to_string(),
            time: None,
        };

        let added = store.add_part("m1", part).unwrap();
        assert_eq!(added.id(), "p1");

        let parts = store.list_parts("m1");
        assert_eq!(parts.len(), 1);

        let got = store.get_part("m1", "p1").unwrap();
        assert_eq!(got.id(), "p1");

        assert!(store.get_part("m1", "p999").is_none());
    }

    #[test]
    fn delete_session_cascades() {
        let store = Store::new();
        let s = make_session("/tmp/dc");
        let sid = s.id.clone();
        store.create_session(s);

        let msg = make_user_msg(&sid, "root");
        let mid = msg.id().to_string();
        store.create_message(&sid, msg);

        assert!(store.delete_session(&sid));
        assert!(store.list_messages(&sid).is_empty());
    }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum FeedError {
    SubscriberExists { id: u64 },
    SubscriberNotFound { id: u64 },
    AlreadyCaughtUp { id: u64 },
}

impl std::fmt::Display for FeedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedError::SubscriberExists { id } => write!(f, "subscriber {id} exists"),
            FeedError::SubscriberNotFound { id } => write!(f, "subscriber {id} not found"),
            FeedError::AlreadyCaughtUp { id } => write!(f, "subscriber {id} caught up"),
        }
    }
}

impl std::error::Error for FeedError {}

#[derive(Debug, Clone)]
pub struct FeedEvent {
    pub seq: u64,
    pub topic: String,
    pub payload: Vec<u8>,
}

struct Subscriber {
    id: u64,
    cursor: u64,
    topic_filter: Option<String>,
    pending: Vec<FeedEvent>,
}

pub struct FeedLog {
    events: Vec<FeedEvent>,
    subscribers: BTreeMap<u64, Subscriber>,
    next_sub: u64,
    next_seq: u64,
    total_appended: u64,
    total_delivered: u64,
    total_replayed: u64,
}

impl FeedLog {
    pub fn new() -> Self { Self { events: Vec::new(), subscribers: BTreeMap::new(), next_sub: 1, next_seq: 0, total_appended: 0, total_delivered: 0, total_replayed: 0 } }

    pub fn subscribe(&mut self, topic_filter: Option<String>) -> u64 {
        let id = self.next_sub;
        self.next_sub += 1;
        let cursor = self.next_seq;
        self.subscribers.insert(id, Subscriber { id, cursor, topic_filter, pending: Vec::new() });
        id
    }

    pub fn unsubscribe(&mut self, id: u64) -> Result<(), FeedError> {
        if self.subscribers.remove(&id).is_none() { return Err(FeedError::SubscriberNotFound { id }); }
        Ok(())
    }

    pub fn append(&mut self, topic: &str, payload: Vec<u8>) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        let event = FeedEvent { seq, topic: topic.to_string(), payload };
        for sub in self.subscribers.values_mut() {
            let matches_topic = match &sub.topic_filter {
                Some(f) => f == topic,
                None => true,
            };
            if matches_topic && sub.cursor <= seq {
                sub.pending.push(event.clone());
                sub.cursor = seq + 1;
                self.total_delivered += 1;
            }
        }
        self.events.push(event);
        self.total_appended += 1;
        seq
    }

    pub fn poll(&mut self, id: u64) -> Result<Vec<FeedEvent>, FeedError> {
        let sub = self.subscribers.get_mut(&id).ok_or(FeedError::SubscriberNotFound { id })?;
        let events: Vec<FeedEvent> = sub.pending.drain(..).collect();
        Ok(events)
    }

    pub fn replay(&mut self, id: u64, from_seq: u64) -> Result<Vec<FeedEvent>, FeedError> {
        let sub = self.subscribers.get_mut(&id).ok_or(FeedError::SubscriberNotFound { id })?;
        self.total_replayed += 1;
        let events: Vec<FeedEvent> = self.events.iter()
            .filter(|e| e.seq >= from_seq)
            .filter(|e| match &sub.topic_filter { Some(f) => f == &e.topic, None => true })
            .cloned()
            .collect();
        sub.cursor = self.next_seq;
        Ok(events)
    }

    pub fn cursor(&self, id: u64) -> Option<u64> { self.subscribers.get(&id).map(|s| s.cursor) }
    pub fn pending(&self, id: u64) -> Option<usize> { self.subscribers.get(&id).map(|s| s.pending.len()) }
    pub fn event_count(&self) -> usize { self.events.len() }
    pub fn subscriber_count(&self) -> usize { self.subscribers.len() }
    pub fn total_appended(&self) -> u64 { self.total_appended }
    pub fn total_delivered(&self) -> u64 { self.total_delivered }
    pub fn total_replayed(&self) -> u64 { self.total_replayed }
}

impl Default for FeedLog {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_feed() { assert_eq!(FeedLog::new().event_count(), 0); }

    #[test]
    fn subscribe_append_poll() {
        let mut fl = FeedLog::new();
        let sub = fl.subscribe(None);
        fl.append("trade", b"buy 100".to_vec());
        fl.append("trade", b"sell 50".to_vec());
        let events = fl.poll(sub).unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn topic_filter() {
        let mut fl = FeedLog::new();
        let sub = fl.subscribe(Some("trade".to_string()));
        fl.append("trade", b"buy".to_vec());
        fl.append("price", b"100".to_vec());
        let events = fl.poll(sub).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].topic, "trade");
    }

    #[test]
    fn unsubscribe() {
        let mut fl = FeedLog::new();
        let sub = fl.subscribe(None);
        fl.unsubscribe(sub).unwrap();
        assert_eq!(fl.subscriber_count(), 0);
    }

    #[test]
    fn not_found() {
        let mut fl = FeedLog::new();
        let err = fl.poll(99).unwrap_err();
        assert!(matches!(err, FeedError::SubscriberNotFound { .. }));
    }

    #[test]
    fn replay() {
        let mut fl = FeedLog::new();
        fl.append("a", b"1".to_vec());
        fl.append("b", b"2".to_vec());
        fl.append("a", b"3".to_vec());
        let sub = fl.subscribe(None);
        let events = fl.replay(sub, 1).unwrap();
        assert!(events.len() >= 2);
    }

    #[test]
    fn cursor_tracking() {
        let mut fl = FeedLog::new();
        let sub = fl.subscribe(None);
        fl.append("x", b"1".to_vec());
        assert_eq!(fl.cursor(sub), Some(1));
    }

    #[test]
    fn pending() {
        let mut fl = FeedLog::new();
        let sub = fl.subscribe(None);
        fl.append("x", b"1".to_vec());
        fl.append("x", b"2".to_vec());
        assert_eq!(fl.pending(sub), Some(2));
        fl.poll(sub).unwrap();
        assert_eq!(fl.pending(sub), Some(0));
    }

    #[test]
    fn late_subscriber() {
        let mut fl = FeedLog::new();
        fl.append("x", b"1".to_vec());
        let sub = fl.subscribe(None);
        let events = fl.poll(sub).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn stats() {
        let mut fl = FeedLog::new();
        let sub = fl.subscribe(None);
        fl.append("x", b"1".to_vec());
        fl.poll(sub).unwrap();
        assert_eq!(fl.total_appended(), 1);
        assert_eq!(fl.total_delivered(), 1);
    }

    #[test]
    fn error_display() { assert!(FeedError::SubscriberNotFound { id: 3 }.to_string().contains("3")); }
}

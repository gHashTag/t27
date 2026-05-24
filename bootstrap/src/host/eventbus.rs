use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum EbError {
    SubscriberNotFound { id: u64 },
    TopicNotFound { topic: String },
}

impl std::fmt::Display for EbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EbError::SubscriberNotFound { id } => write!(f, "subscriber {id} not found"),
            EbError::TopicNotFound { topic } => write!(f, "topic {topic} not found"),
        }
    }
}

impl std::error::Error for EbError {}

#[derive(Debug, Clone)]
pub struct Event {
    pub topic: String,
    pub payload: Vec<u8>,
    pub seq: u64,
}

struct Subscriber {
    id: u64,
    topics: Vec<String>,
    pending: Vec<Event>,
    filter: Option<Box<dyn Fn(&Event) -> bool>>,
}

pub struct EventBus {
    subscribers: BTreeMap<u64, Subscriber>,
    next_sub: u64,
    seq: u64,
    total_published: u64,
    total_delivered: u64,
    total_dropped: u64,
    max_pending: usize,
}

impl EventBus {
    pub fn new(max_pending: usize) -> Self { Self { subscribers: BTreeMap::new(), next_sub: 1, seq: 0, total_published: 0, total_delivered: 0, total_dropped: 0, max_pending } }

    pub fn subscribe(&mut self, topics: Vec<String>) -> u64 {
        let id = self.next_sub;
        self.next_sub += 1;
        self.subscribers.insert(id, Subscriber { id, topics, pending: Vec::new(), filter: None });
        id
    }

    pub fn subscribe_filtered(&mut self, topics: Vec<String>, filter: Box<dyn Fn(&Event) -> bool>) -> u64 {
        let id = self.next_sub;
        self.next_sub += 1;
        self.subscribers.insert(id, Subscriber { id, topics, pending: Vec::new(), filter: Some(filter) });
        id
    }

    pub fn unsubscribe(&mut self, id: u64) -> Result<(), EbError> {
        if self.subscribers.remove(&id).is_none() { return Err(EbError::SubscriberNotFound { id }); }
        Ok(())
    }

    pub fn publish(&mut self, topic: &str, payload: Vec<u8>) -> u64 {
        let seq = self.seq;
        self.seq += 1;
        let event = Event { topic: topic.to_string(), payload, seq };
        for sub in self.subscribers.values_mut() {
            let matches_topic = sub.topics.is_empty() || sub.topics.iter().any(|t| t == topic);
            if !matches_topic { continue; }
            if let Some(ref filter) = sub.filter {
                if !filter(&event) { continue; }
            }
            if sub.pending.len() >= self.max_pending {
                self.total_dropped += 1;
                sub.pending.remove(0);
            }
            sub.pending.push(event.clone());
            self.total_delivered += 1;
        }
        self.total_published += 1;
        seq
    }

    pub fn poll(&mut self, id: u64) -> Result<Vec<Event>, EbError> {
        let sub = self.subscribers.get_mut(&id).ok_or(EbError::SubscriberNotFound { id })?;
        Ok(sub.pending.drain(..).collect())
    }

    pub fn pending(&self, id: u64) -> Option<usize> { self.subscribers.get(&id).map(|s| s.pending.len()) }
    pub fn subscriber_count(&self) -> usize { self.subscribers.len() }
    pub fn total_published(&self) -> u64 { self.total_published }
    pub fn total_delivered(&self) -> u64 { self.total_delivered }
    pub fn total_dropped(&self) -> u64 { self.total_dropped }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bus() { assert_eq!(EventBus::new(100).subscriber_count(), 0); }

    #[test]
    fn subscribe_publish_poll() {
        let mut eb = EventBus::new(100);
        let sub = eb.subscribe(vec!["trade".to_string()]);
        eb.publish("trade", b"buy 100".to_vec());
        let events = eb.poll(sub).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].topic, "trade");
    }

    #[test]
    fn topic_filter() {
        let mut eb = EventBus::new(100);
        let sub = eb.subscribe(vec!["trade".to_string()]);
        eb.publish("price", b"100".to_vec());
        let events = eb.poll(sub).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn wildcard_subscribe() {
        let mut eb = EventBus::new(100);
        let sub = eb.subscribe(vec![]);
        eb.publish("a", b"1".to_vec());
        eb.publish("b", b"2".to_vec());
        let events = eb.poll(sub).unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn custom_filter() {
        let mut eb = EventBus::new(100);
        let sub = eb.subscribe_filtered(vec![], Box::new(|e| e.payload.len() > 2));
        eb.publish("x", b"ab".to_vec());
        eb.publish("x", b"abc".to_vec());
        let events = eb.poll(sub).unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn unsubscribe() {
        let mut eb = EventBus::new(100);
        let sub = eb.subscribe(vec![]);
        eb.unsubscribe(sub).unwrap();
        assert_eq!(eb.subscriber_count(), 0);
    }

    #[test]
    fn backpressure() {
        let mut eb = EventBus::new(2);
        let sub = eb.subscribe(vec![]);
        eb.publish("x", b"1".to_vec());
        eb.publish("x", b"2".to_vec());
        eb.publish("x", b"3".to_vec());
        assert_eq!(eb.pending(sub), Some(2));
        assert!(eb.total_dropped() > 0);
    }

    #[test]
    fn not_found() {
        let mut eb = EventBus::new(100);
        let err = eb.poll(99).unwrap_err();
        assert!(matches!(err, EbError::SubscriberNotFound { .. }));
    }

    #[test]
    fn multiple_subscribers() {
        let mut eb = EventBus::new(100);
        let s1 = eb.subscribe(vec!["a".to_string()]);
        let s2 = eb.subscribe(vec!["a".to_string()]);
        eb.publish("a", b"x".to_vec());
        assert_eq!(eb.pending(s1), Some(1));
        assert_eq!(eb.pending(s2), Some(1));
    }

    #[test]
    fn stats() {
        let mut eb = EventBus::new(100);
        let sub = eb.subscribe(vec![]);
        eb.publish("x", b"1".to_vec());
        eb.poll(sub).unwrap();
        assert_eq!(eb.total_published(), 1);
        assert_eq!(eb.total_delivered(), 1);
    }

    #[test]
    fn error_display() { assert!(EbError::SubscriberNotFound { id: 3 }.to_string().contains("3")); }
}

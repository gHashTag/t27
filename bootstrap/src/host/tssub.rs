use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub enum TsError {
    AlreadySubscribed { sub_id: u64, topic: u64 },
    NotSubscribed { sub_id: u64, topic: u64 },
    SubscriberNotFound { sub_id: u64 },
}

impl std::fmt::Display for TsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TsError::AlreadySubscribed { sub_id, topic } => write!(f, "{sub_id} already subscribed to {topic}"),
            TsError::NotSubscribed { sub_id, topic } => write!(f, "{sub_id} not subscribed to {topic}"),
            TsError::SubscriberNotFound { sub_id } => write!(f, "subscriber {sub_id} not found"),
        }
    }
}

impl std::error::Error for TsError {}

struct Subscriber {
    id: u64,
    topics: BTreeSet<u64>,
    inbox: Vec<(u64, Vec<u8>)>,
    max_inbox: usize,
    total_received: u64,
    total_dropped: u64,
}

pub struct TopicSub {
    subscribers: BTreeMap<u64, Subscriber>,
    topic_subs: BTreeMap<u64, BTreeSet<u64>>,
    next_id: u64,
    default_inbox: usize,
    total_published: u64,
    total_dispatched: u64,
}

impl TopicSub {
    pub fn new(default_inbox: usize) -> Self {
        Self { subscribers: BTreeMap::new(), topic_subs: BTreeMap::new(), next_id: 1, default_inbox, total_published: 0, total_dispatched: 0 }
    }

    pub fn create(&mut self, max_inbox: Option<usize>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.subscribers.insert(id, Subscriber { id, topics: BTreeSet::new(), inbox: Vec::new(), max_inbox: max_inbox.unwrap_or(self.default_inbox), total_received: 0, total_dropped: 0 });
        id
    }

    pub fn subscribe(&mut self, sub_id: u64, topic: u64) -> Result<(), TsError> {
        let sub = self.subscribers.get_mut(&sub_id).ok_or(TsError::SubscriberNotFound { sub_id })?;
        if sub.topics.contains(&topic) { return Err(TsError::AlreadySubscribed { sub_id, topic }); }
        sub.topics.insert(topic);
        self.topic_subs.entry(topic).or_default().insert(sub_id);
        Ok(())
    }

    pub fn unsubscribe(&mut self, sub_id: u64, topic: u64) -> Result<(), TsError> {
        let sub = self.subscribers.get_mut(&sub_id).ok_or(TsError::SubscriberNotFound { sub_id })?;
        if !sub.topics.remove(&topic) { return Err(TsError::NotSubscribed { sub_id, topic }); }
        if let Some(subs) = self.topic_subs.get_mut(&topic) {
            subs.remove(&sub_id);
            if subs.is_empty() { self.topic_subs.remove(&topic); }
        }
        Ok(())
    }

    pub fn publish(&mut self, topic: u64, data: Vec<u8>) -> usize {
        self.total_published += 1;
        let mut dispatched = 0;
        if let Some(sub_ids) = self.topic_subs.get(&topic).cloned() {
            for sid in sub_ids {
                if let Some(sub) = self.subscribers.get_mut(&sid) {
                    if sub.inbox.len() < sub.max_inbox {
                        sub.inbox.push((topic, data.clone()));
                        sub.total_received += 1;
                        dispatched += 1;
                        self.total_dispatched += 1;
                    } else {
                        sub.total_dropped += 1;
                    }
                }
            }
        }
        dispatched
    }

    pub fn recv(&mut self, sub_id: u64) -> Option<(u64, Vec<u8>)> {
        self.subscribers.get_mut(&sub_id)?.inbox.pop()
    }

    pub fn inbox_len(&self, sub_id: u64) -> Option<usize> {
        self.subscribers.get(&sub_id).map(|s| s.inbox.len())
    }

    pub fn topics(&self, sub_id: u64) -> Option<Vec<u64>> {
        self.subscribers.get(&sub_id).map(|s| s.topics.iter().copied().collect())
    }

    pub fn subscriber_count(&self) -> usize { self.subscribers.len() }
    pub fn topic_count(&self) -> usize { self.topic_subs.len() }
    pub fn topic_sub_count(&self, topic: u64) -> usize { self.topic_subs.get(&topic).map(|s| s.len()).unwrap_or(0) }
    pub fn total_published(&self) -> u64 { self.total_published }
    pub fn total_dispatched(&self) -> u64 { self.total_dispatched }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sub() { let ts = TopicSub::new(10); assert_eq!(ts.subscriber_count(), 0); }

    #[test]
    fn create_subscribe() {
        let mut ts = TopicSub::new(10);
        let id = ts.create(None);
        ts.subscribe(id, 1).unwrap();
        assert_eq!(ts.topic_sub_count(1), 1);
    }

    #[test]
    fn publish_recv() {
        let mut ts = TopicSub::new(10);
        let id = ts.create(None);
        ts.subscribe(id, 1).unwrap();
        let dispatched = ts.publish(1, b"msg".to_vec());
        assert_eq!(dispatched, 1);
        let (topic, data) = ts.recv(id).unwrap();
        assert_eq!(topic, 1);
        assert_eq!(data, b"msg");
    }

    #[test]
    fn no_subscribers() {
        let mut ts = TopicSub::new(10);
        assert_eq!(ts.publish(1, b"msg".to_vec()), 0);
    }

    #[test]
    fn unsubscribe() {
        let mut ts = TopicSub::new(10);
        let id = ts.create(None);
        ts.subscribe(id, 1).unwrap();
        ts.unsubscribe(id, 1).unwrap();
        assert_eq!(ts.topic_sub_count(1), 0);
    }

    #[test]
    fn inbox_overflow() {
        let mut ts = TopicSub::new(2);
        let id = ts.create(Some(1));
        ts.subscribe(id, 1).unwrap();
        ts.publish(1, b"a".to_vec());
        ts.publish(1, b"b".to_vec());
        assert_eq!(ts.publish(1, b"c".to_vec()), 0);
    }

    #[test]
    fn fanout() {
        let mut ts = TopicSub::new(10);
        let s1 = ts.create(None);
        let s2 = ts.create(None);
        ts.subscribe(s1, 1).unwrap();
        ts.subscribe(s2, 1).unwrap();
        assert_eq!(ts.publish(1, b"msg".to_vec()), 2);
    }

    #[test]
    fn duplicate_subscribe() {
        let mut ts = TopicSub::new(10);
        let id = ts.create(None);
        ts.subscribe(id, 1).unwrap();
        let err = ts.subscribe(id, 1).unwrap_err();
        assert!(matches!(err, TsError::AlreadySubscribed { .. }));
    }

    #[test]
    fn not_found() {
        let mut ts = TopicSub::new(10);
        let err = ts.subscribe(99, 1).unwrap_err();
        assert!(matches!(err, TsError::SubscriberNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut ts = TopicSub::new(10);
        let id = ts.create(None);
        ts.subscribe(id, 1).unwrap();
        ts.publish(1, b"x".to_vec());
        assert_eq!(ts.total_published(), 1);
        assert_eq!(ts.total_dispatched(), 1);
    }

    #[test]
    fn error_display() { assert!(TsError::SubscriberNotFound { sub_id: 1 }.to_string().contains("1")); }
}

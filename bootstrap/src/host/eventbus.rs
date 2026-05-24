use std::collections::BTreeMap;

pub type SubscriberId = u64;
pub type Topic = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Message { topic: Topic, payload: Vec<u8> },
    Signal { topic: Topic, code: u32 },
}

impl Event {
    pub fn topic(&self) -> Topic {
        match self {
            Event::Message { topic, .. } => *topic,
            Event::Signal { topic, .. } => *topic,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusError {
    SubscriberNotFound { id: SubscriberId },
    AlreadySubscribed { id: SubscriberId, topic: Topic },
    QueueFull { id: SubscriberId },
}

impl std::fmt::Display for BusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BusError::SubscriberNotFound { id } => write!(f, "subscriber {id} not found"),
            BusError::AlreadySubscribed { id, topic } => write!(f, "{id} already on topic {topic}"),
            BusError::QueueFull { id } => write!(f, "subscriber {id} queue full"),
        }
    }
}

impl std::error::Error for BusError {}

#[derive(Debug, Clone)]
struct Subscriber {
    id: SubscriberId,
    topics: Vec<Topic>,
    queue: Vec<Event>,
    depth: usize,
    total_received: u64,
    total_dropped: u64,
}

#[derive(Debug, Clone)]
pub struct SubscriberInfo {
    pub id: SubscriberId,
    pub topics: Vec<Topic>,
    pub queued: usize,
    pub total_received: u64,
    pub total_dropped: u64,
}

#[derive(Debug, Clone)]
pub struct EventBus {
    subscribers: BTreeMap<SubscriberId, Subscriber>,
    next_id: SubscriberId,
    total_published: u64,
    total_dispatched: u64,
}

impl EventBus {
    pub fn new() -> Self {
        Self { subscribers: BTreeMap::new(), next_id: 1, total_published: 0, total_dispatched: 0 }
    }

    pub fn subscribe(&mut self, topics: &[Topic], queue_depth: usize) -> SubscriberId {
        let id = self.next_id;
        self.next_id += 1;
        self.subscribers.insert(id, Subscriber {
            id,
            topics: topics.to_vec(),
            queue: Vec::with_capacity(queue_depth),
            depth: queue_depth,
            total_received: 0,
            total_dropped: 0,
        });
        id
    }

    pub fn unsubscribe(&mut self, id: SubscriberId) -> Result<(), BusError> {
        self.subscribers.remove(&id)
            .ok_or(BusError::SubscriberNotFound { id })?;
        Ok(())
    }

    pub fn add_topic(&mut self, id: SubscriberId, topic: Topic) -> Result<(), BusError> {
        let sub = self.subscribers.get_mut(&id)
            .ok_or(BusError::SubscriberNotFound { id })?;
        if sub.topics.contains(&topic) {
            return Err(BusError::AlreadySubscribed { id, topic });
        }
        sub.topics.push(topic);
        Ok(())
    }

    pub fn remove_topic(&mut self, id: SubscriberId, topic: Topic) -> Result<bool, BusError> {
        let sub = self.subscribers.get_mut(&id)
            .ok_or(BusError::SubscriberNotFound { id })?;
        if let Some(pos) = sub.topics.iter().position(|&t| t == topic) {
            sub.topics.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn publish(&mut self, event: Event) -> usize {
        let topic = event.topic();
        self.total_published += 1;
        let mut count = 0;
        for sub in self.subscribers.values_mut() {
            if sub.topics.contains(&topic) {
                if sub.queue.len() < sub.depth {
                    sub.queue.push(event.clone());
                    sub.total_received += 1;
                    self.total_dispatched += 1;
                    count += 1;
                } else {
                    sub.total_dropped += 1;
                }
            }
        }
        count
    }

    pub fn poll(&mut self, id: SubscriberId) -> Option<Event> {
        let sub = self.subscribers.get_mut(&id)?;
        if sub.queue.is_empty() { return None; }
        Some(sub.queue.remove(0))
    }

    pub fn poll_all(&mut self, id: SubscriberId) -> Vec<Event> {
        if let Some(sub) = self.subscribers.get_mut(&id) {
            std::mem::take(&mut sub.queue)
        } else {
            Vec::new()
        }
    }

    pub fn subscriber_info(&self, id: SubscriberId) -> Option<SubscriberInfo> {
        self.subscribers.get(&id).map(|s| SubscriberInfo {
            id: s.id,
            topics: s.topics.clone(),
            queued: s.queue.len(),
            total_received: s.total_received,
            total_dropped: s.total_dropped,
        })
    }

    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    pub fn total_published(&self) -> u64 {
        self.total_published
    }

    pub fn total_dispatched(&self) -> u64 {
        self.total_dispatched
    }

    pub fn topic_subscribers(&self, topic: Topic) -> Vec<SubscriberId> {
        self.subscribers.values()
            .filter(|s| s.topics.contains(&topic))
            .map(|s| s.id)
            .collect()
    }
}

impl Default for EventBus {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bus() {
        let eb = EventBus::new();
        assert_eq!(eb.subscriber_count(), 0);
    }

    #[test]
    fn subscribe_and_count() {
        let mut eb = EventBus::new();
        let s1 = eb.subscribe(&[1, 2], 8);
        let s2 = eb.subscribe(&[2, 3], 8);
        assert_eq!(eb.subscriber_count(), 2);
        assert_ne!(s1, s2);
    }

    #[test]
    fn publish_and_poll() {
        let mut eb = EventBus::new();
        let s = eb.subscribe(&[10], 8);
        eb.publish(Event::Signal { topic: 10, code: 42 });
        let evt = eb.poll(s).unwrap();
        assert_eq!(evt, Event::Signal { topic: 10, code: 42 });
    }

    #[test]
    fn topic_filtering() {
        let mut eb = EventBus::new();
        let s1 = eb.subscribe(&[1], 8);
        let s2 = eb.subscribe(&[2], 8);
        eb.publish(Event::Signal { topic: 1, code: 1 });
        assert!(eb.poll(s1).is_some());
        assert!(eb.poll(s2).is_none());
    }

    #[test]
    fn multi_topic_subscriber() {
        let mut eb = EventBus::new();
        let s = eb.subscribe(&[1, 2, 3], 8);
        eb.publish(Event::Signal { topic: 2, code: 99 });
        let evt = eb.poll(s).unwrap();
        assert_eq!(evt.topic(), 2);
    }

    #[test]
    fn queue_overflow() {
        let mut eb = EventBus::new();
        let s = eb.subscribe(&[1], 2);
        eb.publish(Event::Signal { topic: 1, code: 1 });
        eb.publish(Event::Signal { topic: 1, code: 2 });
        eb.publish(Event::Signal { topic: 1, code: 3 });
        let info = eb.subscriber_info(s).unwrap();
        assert_eq!(info.queued, 2);
        assert_eq!(info.total_dropped, 1);
    }

    #[test]
    fn poll_all() {
        let mut eb = EventBus::new();
        let s = eb.subscribe(&[1], 8);
        eb.publish(Event::Signal { topic: 1, code: 1 });
        eb.publish(Event::Signal { topic: 1, code: 2 });
        let events = eb.poll_all(s);
        assert_eq!(events.len(), 2);
        assert!(eb.poll(s).is_none());
    }

    #[test]
    fn unsubscribe() {
        let mut eb = EventBus::new();
        let s = eb.subscribe(&[1], 8);
        eb.unsubscribe(s).unwrap();
        assert_eq!(eb.subscriber_count(), 0);
        let err = eb.unsubscribe(s).unwrap_err();
        assert!(matches!(err, BusError::SubscriberNotFound { .. }));
    }

    #[test]
    fn add_remove_topic() {
        let mut eb = EventBus::new();
        let s = eb.subscribe(&[1], 8);
        eb.add_topic(s, 5).unwrap();
        eb.publish(Event::Signal { topic: 5, code: 1 });
        assert!(eb.poll(s).is_some());
        eb.remove_topic(s, 5).unwrap();
        eb.publish(Event::Signal { topic: 5, code: 2 });
        assert!(eb.poll(s).is_none());
    }

    #[test]
    fn topic_subscribers_list() {
        let mut eb = EventBus::new();
        eb.subscribe(&[1, 2], 8);
        eb.subscribe(&[2, 3], 8);
        let subs = eb.topic_subscribers(2);
        assert_eq!(subs.len(), 2);
    }

    #[test]
    fn stats() {
        let mut eb = EventBus::new();
        eb.subscribe(&[1], 8);
        eb.subscribe(&[1], 8);
        let dispatched = eb.publish(Event::Signal { topic: 1, code: 0 });
        assert_eq!(dispatched, 2);
        assert_eq!(eb.total_published(), 1);
        assert_eq!(eb.total_dispatched(), 2);
    }

    #[test]
    fn error_display() {
        assert!(BusError::SubscriberNotFound { id: 3 }.to_string().contains("3"));
    }
}

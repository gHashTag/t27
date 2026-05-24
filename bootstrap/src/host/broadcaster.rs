pub type SubscriberId = u64;
pub type DeliverFn = fn(&[u8]) -> bool;

#[derive(Debug, Clone)]
pub struct Subscriber {
    pub id: SubscriberId,
    pub name: String,
    pub deliver: DeliverFn,
    pub active: bool,
}

impl Subscriber {
    pub fn new(name: &str, deliver: DeliverFn) -> Self {
        static mut NEXT_ID: SubscriberId = 1;
        let id = unsafe {
            let id = NEXT_ID;
            NEXT_ID += 1;
            id
        };
        Self {
            id,
            name: name.to_string(),
            deliver,
            active: true,
        }
    }

    pub fn deactivate(mut self) -> Self {
        self.active = false;
        self
    }
}

#[derive(Debug, Clone)]
pub struct BroadcastReport {
    pub message_id: u64,
    pub delivered: usize,
    pub nacked: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone)]
pub struct RingBroadcaster {
    subscribers: Vec<Subscriber>,
    next_msg_id: u64,
    total_delivered: u64,
    total_nacked: u64,
    total_messages: u64,
}

impl RingBroadcaster {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
            next_msg_id: 1,
            total_delivered: 0,
            total_nacked: 0,
            total_messages: 0,
        }
    }

    pub fn subscribe(&mut self, sub: Subscriber) -> SubscriberId {
        let id = sub.id;
        self.subscribers.push(sub);
        id
    }

    pub fn unsubscribe(&mut self, id: SubscriberId) -> bool {
        let len_before = self.subscribers.len();
        self.subscribers.retain(|s| s.id != id);
        self.subscribers.len() != len_before
    }

    pub fn broadcast(&mut self, message: &[u8]) -> BroadcastReport {
        let msg_id = self.next_msg_id;
        self.next_msg_id += 1;
        self.total_messages += 1;
        let mut delivered = 0usize;
        let mut nacked = 0usize;
        let mut skipped = 0usize;
        for sub in &self.subscribers {
            if !sub.active {
                skipped += 1;
                continue;
            }
            if (sub.deliver)(message) {
                delivered += 1;
            } else {
                nacked += 1;
            }
        }
        self.total_delivered += delivered as u64;
        self.total_nacked += nacked as u64;
        BroadcastReport {
            message_id: msg_id,
            delivered,
            nacked,
            skipped,
        }
    }

    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    pub fn active_count(&self) -> usize {
        self.subscribers.iter().filter(|s| s.active).count()
    }

    pub fn total_messages(&self) -> u64 {
        self.total_messages
    }

    pub fn total_delivered(&self) -> u64 {
        self.total_delivered
    }

    pub fn total_nacked(&self) -> u64 {
        self.total_nacked
    }

    pub fn nack_rate(&self) -> f64 {
        let total = self.total_delivered + self.total_nacked;
        if total == 0 {
            0.0
        } else {
            self.total_nacked as f64 / total as f64
        }
    }

    pub fn clear(&mut self) {
        self.subscribers.clear();
    }

    pub fn reset_stats(&mut self) {
        self.total_delivered = 0;
        self.total_nacked = 0;
        self.total_messages = 0;
    }
}

impl Default for RingBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DELIVERED: AtomicUsize = AtomicUsize::new(0);

    fn accept(_msg: &[u8]) -> bool {
        DELIVERED.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn reject(_msg: &[u8]) -> bool {
        false
    }

    fn accept_if_short(msg: &[u8]) -> bool {
        msg.len() < 5
    }

    fn setup() {
        DELIVERED.store(0, Ordering::SeqCst);
    }

    #[test]
    fn subscribe_and_count() {
        let mut rb = RingBroadcaster::new();
        rb.subscribe(Subscriber::new("a", accept));
        rb.subscribe(Subscriber::new("b", accept));
        assert_eq!(rb.subscriber_count(), 2);
        assert_eq!(rb.active_count(), 2);
    }

    #[test]
    fn unsubscribe() {
        let mut rb = RingBroadcaster::new();
        let sub = Subscriber::new("a", accept);
        let id = sub.id;
        rb.subscribe(sub);
        assert!(rb.unsubscribe(id));
        assert!(!rb.unsubscribe(id));
    }

    #[test]
    fn broadcast_all_accept() {
        setup();
        let mut rb = RingBroadcaster::new();
        rb.subscribe(Subscriber::new("a", accept));
        rb.subscribe(Subscriber::new("b", accept));
        let report = rb.broadcast(b"hello");
        assert_eq!(report.delivered, 2);
        assert_eq!(report.nacked, 0);
    }

    #[test]
    fn broadcast_mixed() {
        let mut rb = RingBroadcaster::new();
        rb.subscribe(Subscriber::new("acceptor", accept));
        rb.subscribe(Subscriber::new("rejector", reject));
        let report = rb.broadcast(b"data");
        assert_eq!(report.delivered, 1);
        assert_eq!(report.nacked, 1);
    }

    #[test]
    fn broadcast_skips_inactive() {
        let mut rb = RingBroadcaster::new();
        rb.subscribe(Subscriber::new("active", accept));
        rb.subscribe(Subscriber::new("inactive", reject).deactivate());
        let report = rb.broadcast(b"test");
        assert_eq!(report.delivered, 1);
        assert_eq!(report.skipped, 1);
    }

    #[test]
    fn broadcast_empty() {
        let mut rb = RingBroadcaster::new();
        let report = rb.broadcast(b"hello");
        assert_eq!(report.delivered, 0);
        assert_eq!(report.message_id, 1);
    }

    #[test]
    fn message_ids_increment() {
        let mut rb = RingBroadcaster::new();
        let r1 = rb.broadcast(b"a");
        let r2 = rb.broadcast(b"b");
        assert!(r2.message_id > r1.message_id);
        assert_eq!(rb.total_messages(), 2);
    }

    #[test]
    fn conditional_accept() {
        let mut rb = RingBroadcaster::new();
        rb.subscribe(Subscriber::new("short", accept_if_short));
        let r1 = rb.broadcast(b"abc");
        assert_eq!(r1.delivered, 1);
        let r2 = rb.broadcast(b"abcdefgh");
        assert_eq!(r2.nacked, 1);
    }

    #[test]
    fn nack_rate() {
        let mut rb = RingBroadcaster::new();
        rb.subscribe(Subscriber::new("a", accept));
        rb.subscribe(Subscriber::new("r", reject));
        rb.broadcast(b"x");
        assert!((rb.nack_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn clear() {
        let mut rb = RingBroadcaster::new();
        rb.subscribe(Subscriber::new("a", accept));
        rb.clear();
        assert_eq!(rb.subscriber_count(), 0);
    }

    #[test]
    fn reset_stats() {
        let mut rb = RingBroadcaster::new();
        rb.subscribe(Subscriber::new("a", accept));
        rb.broadcast(b"x");
        rb.reset_stats();
        assert_eq!(rb.total_delivered(), 0);
        assert_eq!(rb.total_nacked(), 0);
    }
}

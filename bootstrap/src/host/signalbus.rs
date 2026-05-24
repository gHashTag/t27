use std::collections::BTreeMap;

pub type SignalId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    pub id: SignalId,
    pub channel: String,
    pub payload: Vec<u8>,
}

impl Signal {
    pub fn new(channel: &str, payload: Vec<u8>) -> Self {
        static mut COUNTER: SignalId = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        Self {
            id,
            channel: channel.to_string(),
            payload,
        }
    }

    pub fn empty(channel: &str) -> Self {
        Self::new(channel, Vec::new())
    }
}

pub type Handler = fn(&Signal);

#[derive(Debug, Clone)]
pub struct Subscription {
    pub channel: String,
    pub handler: Handler,
}

#[derive(Debug)]
pub struct SignalBus {
    subscriptions: BTreeMap<String, Vec<Handler>>,
    pending: Vec<Signal>,
    delivered: u64,
    dropped: u64,
}

impl SignalBus {
    pub fn new() -> Self {
        Self {
            subscriptions: BTreeMap::new(),
            pending: Vec::new(),
            delivered: 0,
            dropped: 0,
        }
    }

    pub fn subscribe(&mut self, channel: &str, handler: Handler) {
        self.subscriptions
            .entry(channel.to_string())
            .or_default()
            .push(handler);
    }

    pub fn unsubscribe(&mut self, channel: &str) -> bool {
        self.subscriptions.remove(channel).is_some()
    }

    pub fn publish(&mut self, signal: Signal) {
        if self.subscriptions.contains_key(&signal.channel) {
            self.pending.push(signal);
        } else {
            self.dropped += 1;
        }
    }

    pub fn dispatch(&mut self) -> u64 {
        let signals: Vec<Signal> = self.pending.drain(..).collect();
        let count = signals.len() as u64;
        for signal in &signals {
            if let Some(handlers) = self.subscriptions.get(&signal.channel) {
                for handler in handlers {
                    handler(signal);
                }
            }
        }
        self.delivered += count;
        count
    }

    pub fn publish_and_dispatch(&mut self, signal: Signal) -> u64 {
        self.publish(signal);
        self.dispatch()
    }

    pub fn subscriber_count(&self, channel: &str) -> usize {
        self.subscriptions.get(channel).map_or(0, |v| v.len())
    }

    pub fn channel_count(&self) -> usize {
        self.subscriptions.len()
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn delivered(&self) -> u64 {
        self.delivered
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn channels(&self) -> Vec<&str> {
        self.subscriptions.keys().map(|s| s.as_str()).collect()
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }

    pub fn reset(&mut self) {
        self.subscriptions.clear();
        self.pending.clear();
        self.delivered = 0;
        self.dropped = 0;
    }
}

impl Default for SignalBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn signal_new() {
        let s = Signal::new("irq", vec![1, 2, 3]);
        assert_eq!(s.channel, "irq");
        assert_eq!(s.payload, vec![1, 2, 3]);
    }

    #[test]
    fn signal_empty() {
        let s = Signal::empty("test");
        assert!(s.payload.is_empty());
    }

    #[test]
    fn subscribe_and_count() {
        let mut bus = SignalBus::new();
        fn handler(_: &Signal) {}
        bus.subscribe("irq", handler);
        assert_eq!(bus.subscriber_count("irq"), 1);
        assert_eq!(bus.channel_count(), 1);
    }

    #[test]
    fn unsubscribe() {
        let mut bus = SignalBus::new();
        fn handler(_: &Signal) {}
        bus.subscribe("irq", handler);
        assert!(bus.unsubscribe("irq"));
        assert!(!bus.unsubscribe("irq"));
        assert_eq!(bus.subscriber_count("irq"), 0);
    }

    #[test]
    fn publish_no_subscriber_drops() {
        let mut bus = SignalBus::new();
        bus.publish(Signal::empty("noone"));
        assert_eq!(bus.dropped(), 1);
        assert_eq!(bus.pending_count(), 0);
    }

    #[test]
    fn publish_with_subscriber_queues() {
        let mut bus = SignalBus::new();
        fn handler(_: &Signal) {}
        bus.subscribe("irq", handler);
        bus.publish(Signal::empty("irq"));
        assert_eq!(bus.pending_count(), 1);
        assert_eq!(bus.dropped(), 0);
    }

    #[test]
    fn dispatch_calls_handlers() {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        COUNT.store(0, Ordering::SeqCst);
        fn handler(_: &Signal) {
            COUNT.fetch_add(1, Ordering::SeqCst);
        }
        let mut bus = SignalBus::new();
        bus.subscribe("irq", handler);
        bus.publish(Signal::empty("irq"));
        bus.publish(Signal::empty("irq"));
        let dispatched = bus.dispatch();
        assert_eq!(dispatched, 2);
        assert_eq!(COUNT.load(Ordering::SeqCst), 2);
        assert_eq!(bus.delivered(), 2);
        assert_eq!(bus.pending_count(), 0);
    }

    #[test]
    fn multiple_handlers() {
        static SUM: AtomicUsize = AtomicUsize::new(0);
        SUM.store(0, Ordering::SeqCst);
        fn h1(_: &Signal) { SUM.fetch_add(1, Ordering::SeqCst); }
        fn h2(_: &Signal) { SUM.fetch_add(10, Ordering::SeqCst); }
        let mut bus = SignalBus::new();
        bus.subscribe("x", h1);
        bus.subscribe("x", h2);
        bus.publish_and_dispatch(Signal::empty("x"));
        assert_eq!(SUM.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn channels_list() {
        let mut bus = SignalBus::new();
        fn handler(_: &Signal) {}
        bus.subscribe("b", handler);
        bus.subscribe("a", handler);
        assert_eq!(bus.channels(), vec!["a", "b"]);
    }

    #[test]
    fn clear_pending() {
        let mut bus = SignalBus::new();
        fn handler(_: &Signal) {}
        bus.subscribe("x", handler);
        bus.publish(Signal::empty("x"));
        bus.clear();
        assert_eq!(bus.pending_count(), 0);
    }

    #[test]
    fn reset() {
        let mut bus = SignalBus::new();
        fn handler(_: &Signal) {}
        bus.subscribe("x", handler);
        bus.publish_and_dispatch(Signal::empty("x"));
        bus.reset();
        assert_eq!(bus.channel_count(), 0);
        assert_eq!(bus.delivered(), 0);
        assert_eq!(bus.dropped(), 0);
    }

    #[test]
    fn publish_and_dispatch_convenience() {
        static CALLED: AtomicUsize = AtomicUsize::new(0);
        CALLED.store(0, Ordering::SeqCst);
        fn handler(_: &Signal) { CALLED.fetch_add(1, Ordering::SeqCst); }
        let mut bus = SignalBus::new();
        bus.subscribe("x", handler);
        let n = bus.publish_and_dispatch(Signal::empty("x"));
        assert_eq!(n, 1);
        assert_eq!(CALLED.load(Ordering::SeqCst), 1);
    }
}

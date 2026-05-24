#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    ConfigureStart,
    ConfigureDone,
    WeightLoadStart,
    WeightLoadDone,
    InferenceStart,
    InferenceDone,
    InferenceError,
    DmaTransfer,
    DmaComplete,
    WatchdogFeed,
    WatchdogExpire,
    CommandEnqueue,
    CommandDequeue,
    SessionStart,
    SessionClose,
    Custom(u8),
}

impl EventKind {
    pub fn name(self) -> &'static str {
        match self {
            EventKind::ConfigureStart => "configure_start",
            EventKind::ConfigureDone => "configure_done",
            EventKind::WeightLoadStart => "weight_load_start",
            EventKind::WeightLoadDone => "weight_load_done",
            EventKind::InferenceStart => "inference_start",
            EventKind::InferenceDone => "inference_done",
            EventKind::InferenceError => "inference_error",
            EventKind::DmaTransfer => "dma_transfer",
            EventKind::DmaComplete => "dma_complete",
            EventKind::WatchdogFeed => "watchdog_feed",
            EventKind::WatchdogExpire => "watchdog_expire",
            EventKind::CommandEnqueue => "command_enqueue",
            EventKind::CommandDequeue => "command_dequeue",
            EventKind::SessionStart => "session_start",
            EventKind::SessionClose => "session_close",
            EventKind::Custom(_) => "custom",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub kind: EventKind,
    pub timestamp_us: u64,
    pub data: u32,
}

impl Event {
    pub fn new(kind: EventKind, timestamp_us: u64) -> Self {
        Self {
            kind,
            timestamp_us,
            data: 0,
        }
    }

    pub fn with_data(kind: EventKind, timestamp_us: u64, data: u32) -> Self {
        Self {
            kind,
            timestamp_us,
            data,
        }
    }

    pub fn duration_since(&self, earlier: &Event) -> Option<u64> {
        if self.timestamp_us >= earlier.timestamp_us {
            Some(self.timestamp_us - earlier.timestamp_us)
        } else {
            None
        }
    }
}

pub const DEFAULT_CAPACITY: usize = 256;

#[derive(Debug, Clone)]
pub struct EventLog {
    events: Vec<Event>,
    capacity: usize,
    dropped: u64,
}

impl EventLog {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            capacity: capacity.max(1),
            dropped: 0,
        }
    }

    pub fn record(&mut self, kind: EventKind, timestamp_us: u64) {
        self.push(Event::new(kind, timestamp_us));
    }

    pub fn record_with_data(&mut self, kind: EventKind, timestamp_us: u64, data: u32) {
        self.push(Event::with_data(kind, timestamp_us, data));
    }

    fn push(&mut self, event: Event) {
        if self.events.len() >= self.capacity {
            self.events.remove(0);
            self.dropped += 1;
        }
        self.events.push(event);
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn last(&self) -> Option<&Event> {
        self.events.last()
    }

    pub fn first(&self) -> Option<&Event> {
        self.events.first()
    }

    pub fn find(&self, kind: EventKind) -> Option<&Event> {
        self.events.iter().find(|e| e.kind == kind)
    }

    pub fn find_all(&self, kind: EventKind) -> Vec<&Event> {
        self.events.iter().filter(|e| e.kind == kind).collect()
    }

    pub fn count(&self, kind: EventKind) -> usize {
        self.events.iter().filter(|e| e.kind == kind).count()
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.dropped = 0;
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn duration(&self) -> Option<u64> {
        let first = self.first()?;
        let last = self.last()?;
        last.duration_since(first)
    }

    pub fn stats(&self) -> EventLogStats {
        EventLogStats {
            len: self.len(),
            capacity: self.capacity,
            dropped: self.dropped,
            first_ts: self.first().map(|e| e.timestamp_us),
            last_ts: self.last().map(|e| e.timestamp_us),
        }
    }

    pub fn filter_by_range(&self, start_us: u64, end_us: u64) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.timestamp_us >= start_us && e.timestamp_us <= end_us)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventLogStats {
    pub len: usize,
    pub capacity: usize,
    pub dropped: u64,
    pub first_ts: Option<u64>,
    pub last_ts: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_log_is_empty() {
        let log = EventLog::new(16);
        assert!(log.is_empty());
        assert_eq!(log.capacity(), 16);
    }

    #[test]
    fn record_and_retrieve() {
        let mut log = EventLog::new(16);
        log.record(EventKind::InferenceStart, 100);
        log.record(EventKind::InferenceDone, 250);
        assert_eq!(log.len(), 2);
        assert_eq!(log.first().unwrap().kind, EventKind::InferenceStart);
        assert_eq!(log.last().unwrap().kind, EventKind::InferenceDone);
    }

    #[test]
    fn record_with_data() {
        let mut log = EventLog::new(16);
        log.record_with_data(EventKind::DmaTransfer, 50, 1024);
        let e = log.last().unwrap();
        assert_eq!(e.data, 1024);
    }

    #[test]
    fn capacity_overflow_drops_oldest() {
        let mut log = EventLog::new(3);
        log.record(EventKind::ConfigureStart, 1);
        log.record(EventKind::ConfigureDone, 2);
        log.record(EventKind::WeightLoadStart, 3);
        assert_eq!(log.dropped(), 0);
        log.record(EventKind::WeightLoadDone, 4);
        assert_eq!(log.dropped(), 1);
        assert_eq!(log.len(), 3);
        assert_eq!(log.first().unwrap().kind, EventKind::ConfigureDone);
    }

    #[test]
    fn find() {
        let mut log = EventLog::new(16);
        log.record(EventKind::InferenceStart, 100);
        log.record(EventKind::InferenceDone, 200);
        log.record(EventKind::InferenceStart, 300);
        assert_eq!(log.find(EventKind::InferenceDone).unwrap().timestamp_us, 200);
        assert!(log.find(EventKind::WatchdogExpire).is_none());
    }

    #[test]
    fn find_all() {
        let mut log = EventLog::new(16);
        log.record(EventKind::InferenceStart, 100);
        log.record(EventKind::InferenceDone, 200);
        log.record(EventKind::InferenceStart, 300);
        let starts = log.find_all(EventKind::InferenceStart);
        assert_eq!(starts.len(), 2);
    }

    #[test]
    fn count_by_kind() {
        let mut log = EventLog::new(16);
        log.record(EventKind::WatchdogFeed, 10);
        log.record(EventKind::WatchdogFeed, 20);
        log.record(EventKind::WatchdogFeed, 30);
        log.record(EventKind::WatchdogExpire, 40);
        assert_eq!(log.count(EventKind::WatchdogFeed), 3);
        assert_eq!(log.count(EventKind::WatchdogExpire), 1);
    }

    #[test]
    fn duration() {
        let mut log = EventLog::new(16);
        log.record(EventKind::SessionStart, 100);
        log.record(EventKind::SessionClose, 500);
        assert_eq!(log.duration(), Some(400));
    }

    #[test]
    fn duration_empty_is_none() {
        let log = EventLog::new(16);
        assert!(log.duration().is_none());
    }

    #[test]
    fn event_duration_since() {
        let a = Event::new(EventKind::InferenceStart, 100);
        let b = Event::new(EventKind::InferenceDone, 350);
        assert_eq!(b.duration_since(&a), Some(250));
        assert_eq!(a.duration_since(&b), None);
    }

    #[test]
    fn clear() {
        let mut log = EventLog::new(16);
        log.record(EventKind::SessionStart, 0);
        log.record(EventKind::SessionClose, 0);
        log.clear();
        assert!(log.is_empty());
        assert_eq!(log.dropped(), 0);
    }

    #[test]
    fn filter_by_range() {
        let mut log = EventLog::new(16);
        log.record(EventKind::Custom(1), 100);
        log.record(EventKind::Custom(2), 200);
        log.record(EventKind::Custom(3), 300);
        log.record(EventKind::Custom(4), 400);
        let filtered = log.filter_by_range(150, 350);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn stats() {
        let mut log = EventLog::new(8);
        log.record(EventKind::SessionStart, 10);
        log.record(EventKind::SessionClose, 90);
        let stats = log.stats();
        assert_eq!(stats.len, 2);
        assert_eq!(stats.capacity, 8);
        assert_eq!(stats.dropped, 0);
        assert_eq!(stats.first_ts, Some(10));
        assert_eq!(stats.last_ts, Some(90));
    }

    #[test]
    fn event_kind_names() {
        assert_eq!(EventKind::InferenceStart.name(), "inference_start");
        assert_eq!(EventKind::DmaComplete.name(), "dma_complete");
        assert_eq!(EventKind::Custom(42).name(), "custom");
    }

    #[test]
    fn min_capacity_is_one() {
        let log = EventLog::new(0);
        assert_eq!(log.capacity(), 1);
    }

    #[test]
    fn events_slice() {
        let mut log = EventLog::new(16);
        log.record(EventKind::ConfigureStart, 1);
        log.record(EventKind::ConfigureDone, 2);
        assert_eq!(log.events().len(), 2);
    }
}

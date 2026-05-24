use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IrqKind {
    DmaComplete,
    RxReady,
    TxEmpty,
    Error,
    Watchdog,
}

impl std::fmt::Display for IrqKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrqKind::DmaComplete => write!(f, "dma_complete"),
            IrqKind::RxReady => write!(f, "rx_ready"),
            IrqKind::TxEmpty => write!(f, "tx_empty"),
            IrqKind::Error => write!(f, "error"),
            IrqKind::Watchdog => write!(f, "watchdog"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CoalescedEvent {
    pub kind: IrqKind,
    pub count: u32,
    pub first_us: u64,
    pub last_us: u64,
}

impl CoalescedEvent {
    pub fn new(kind: IrqKind, timestamp_us: u64) -> Self {
        Self {
            kind,
            count: 1,
            first_us: timestamp_us,
            last_us: timestamp_us,
        }
    }

    pub fn merge(&mut self, timestamp_us: u64) {
        self.count += 1;
        self.last_us = timestamp_us;
    }

    pub fn span_us(&self) -> u64 {
        self.last_us - self.first_us
    }
}

#[derive(Debug, Clone)]
pub struct IrqCoalescer {
    window_us: u64,
    pending: BTreeMap<IrqKind, CoalescedEvent>,
    total_irqs: u64,
    total_events: u64,
    flushed: u64,
}

impl IrqCoalescer {
    pub fn new(window_us: u64) -> Self {
        Self {
            window_us,
            pending: BTreeMap::new(),
            total_irqs: 0,
            total_events: 0,
            flushed: 0,
        }
    }

    pub fn inject(&mut self, kind: IrqKind, timestamp_us: u64) {
        self.total_irqs += 1;
        if let Some(event) = self.pending.get_mut(&kind) {
            let elapsed = timestamp_us.saturating_sub(event.first_us);
            if elapsed <= self.window_us {
                event.merge(timestamp_us);
                return;
            }
        }
        self.pending.insert(kind, CoalescedEvent::new(kind, timestamp_us));
        self.total_events += 1;
    }

    pub fn flush(&mut self) -> Vec<CoalescedEvent> {
        let events: Vec<CoalescedEvent> = self.pending.values().cloned().collect();
        self.flushed += events.len() as u64;
        self.pending.clear();
        events
    }

    pub fn flush_expired(&mut self, now_us: u64) -> Vec<CoalescedEvent> {
        let mut expired = Vec::new();
        let mut remaining = BTreeMap::new();
        let old = std::mem::take(&mut self.pending);
        for (kind, event) in old {
            if now_us.saturating_sub(event.last_us) > self.window_us {
                expired.push(event);
            } else {
                remaining.insert(kind, event);
            }
        }
        self.pending = remaining;
        self.flushed += expired.len() as u64;
        expired
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn pending_kinds(&self) -> Vec<IrqKind> {
        self.pending.keys().copied().collect()
    }

    pub fn total_irqs(&self) -> u64 {
        self.total_irqs
    }

    pub fn total_events(&self) -> u64 {
        self.total_events
    }

    pub fn total_flushed(&self) -> u64 {
        self.flushed
    }

    pub fn coalesce_ratio(&self) -> f64 {
        if self.total_irqs == 0 {
            0.0
        } else {
            1.0 - (self.total_events as f64 / self.total_irqs as f64)
        }
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }

    pub fn reset(&mut self) {
        self.pending.clear();
        self.total_irqs = 0;
        self.total_events = 0;
        self.flushed = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_display() {
        assert_eq!(IrqKind::DmaComplete.to_string(), "dma_complete");
    }

    #[test]
    fn coalesced_event_merge() {
        let mut e = CoalescedEvent::new(IrqKind::RxReady, 100);
        e.merge(200);
        assert_eq!(e.count, 2);
        assert_eq!(e.span_us(), 100);
    }

    #[test]
    fn inject_single() {
        let mut c = IrqCoalescer::new(100);
        c.inject(IrqKind::RxReady, 10);
        assert_eq!(c.total_irqs(), 1);
        assert_eq!(c.total_events(), 1);
        assert_eq!(c.pending_count(), 1);
    }

    #[test]
    fn inject_coalesces_within_window() {
        let mut c = IrqCoalescer::new(100);
        c.inject(IrqKind::RxReady, 10);
        c.inject(IrqKind::RxReady, 50);
        c.inject(IrqKind::RxReady, 80);
        assert_eq!(c.total_irqs(), 3);
        assert_eq!(c.total_events(), 1);
        assert_eq!(c.pending_count(), 1);
        let event = &c.pending[&IrqKind::RxReady];
        assert_eq!(event.count, 3);
    }

    #[test]
    fn inject_new_window_after_expiry() {
        let mut c = IrqCoalescer::new(100);
        c.inject(IrqKind::RxReady, 10);
        c.inject(IrqKind::RxReady, 200);
        assert_eq!(c.total_events(), 2);
    }

    #[test]
    fn different_kinds_independent() {
        let mut c = IrqCoalescer::new(100);
        c.inject(IrqKind::RxReady, 10);
        c.inject(IrqKind::TxEmpty, 10);
        assert_eq!(c.pending_count(), 2);
    }

    #[test]
    fn flush_all() {
        let mut c = IrqCoalescer::new(100);
        c.inject(IrqKind::RxReady, 10);
        c.inject(IrqKind::Error, 20);
        let events = c.flush();
        assert_eq!(events.len(), 2);
        assert_eq!(c.pending_count(), 0);
        assert_eq!(c.total_flushed(), 2);
    }

    #[test]
    fn flush_expired() {
        let mut c = IrqCoalescer::new(100);
        c.inject(IrqKind::RxReady, 10);
        c.inject(IrqKind::TxEmpty, 1100);
        let expired = c.flush_expired(1200);
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].kind, IrqKind::RxReady);
        assert_eq!(c.pending_count(), 1);
    }

    #[test]
    fn coalesce_ratio() {
        let mut c = IrqCoalescer::new(100);
        c.inject(IrqKind::RxReady, 10);
        c.inject(IrqKind::RxReady, 20);
        assert!((c.coalesce_ratio() - 0.5).abs() < 0.01);
    }

    #[test]
    fn clear_and_reset() {
        let mut c = IrqCoalescer::new(100);
        c.inject(IrqKind::RxReady, 10);
        c.reset();
        assert_eq!(c.total_irqs(), 0);
        assert_eq!(c.pending_count(), 0);
    }

    #[test]
    fn pending_kinds() {
        let mut c = IrqCoalescer::new(100);
        c.inject(IrqKind::Error, 10);
        c.inject(IrqKind::DmaComplete, 10);
        assert_eq!(c.pending_kinds(), vec![IrqKind::DmaComplete, IrqKind::Error]);
    }
}

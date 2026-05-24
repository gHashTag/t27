use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum BpError {
    ChannelNotFound { id: u64 },
}

impl std::fmt::Display for BpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BpError::ChannelNotFound { id } => write!(f, "channel {id} not found"),
        }
    }
}

impl std::error::Error for BpError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ValveState { Open, Pressure, Closed }

struct Channel {
    id: u64,
    capacity: usize,
    current: usize,
    low_watermark: usize,
    high_watermark: usize,
    state: ValveState,
    total_accepted: u64,
    total_rejected: u64,
    total_probes: u64,
}

pub struct Backpressure {
    channels: BTreeMap<u64, Channel>,
    total_accepted: u64,
    total_rejected: u64,
    total_probes: u64,
}

impl Backpressure {
    pub fn new() -> Self { Self { channels: BTreeMap::new(), total_accepted: 0, total_rejected: 0, total_probes: 0 } }

    pub fn register(&mut self, id: u64, capacity: usize, low_wm: usize, high_wm: usize) {
        self.channels.insert(id, Channel { id, capacity, current: 0, low_watermark: low_wm, high_watermark: high_wm, state: ValveState::Open, total_accepted: 0, total_rejected: 0, total_probes: 0 });
    }

    pub fn send(&mut self, id: u64, amount: usize) -> Result<bool, BpError> {
        let ch = self.channels.get_mut(&id).ok_or(BpError::ChannelNotFound { id })?;
        if ch.state == ValveState::Closed {
            ch.total_rejected += 1;
            self.total_rejected += 1;
            return Ok(false);
        }
        if ch.current + amount > ch.capacity {
            ch.total_rejected += 1;
            self.total_rejected += 1;
            return Ok(false);
        }
        ch.current += amount;
        ch.total_accepted += 1;
        self.total_accepted += 1;
        if ch.current >= ch.high_watermark { ch.state = ValveState::Pressure; }
        if ch.current >= ch.capacity { ch.state = ValveState::Closed; }
        Ok(true)
    }

    pub fn ack(&mut self, id: u64, amount: usize) -> Result<(), BpError> {
        let ch = self.channels.get_mut(&id).ok_or(BpError::ChannelNotFound { id })?;
        ch.current = ch.current.saturating_sub(amount);
        if ch.current <= ch.low_watermark { ch.state = ValveState::Open; }
        else if ch.current < ch.high_watermark { ch.state = ValveState::Open; }
        Ok(())
    }

    pub fn probe(&mut self, id: u64) -> Result<(ValveState, usize, usize), BpError> {
        let ch = self.channels.get_mut(&id).ok_or(BpError::ChannelNotFound { id })?;
        ch.total_probes += 1;
        self.total_probes += 1;
        Ok((ch.state.clone(), ch.current, ch.capacity - ch.current))
    }

    pub fn state(&self, id: u64) -> Option<&ValveState> { self.channels.get(&id).map(|c| &c.state) }
    pub fn channel_count(&self) -> usize { self.channels.len() }
    pub fn total_accepted(&self) -> u64 { self.total_accepted }
    pub fn total_rejected(&self) -> u64 { self.total_rejected }
    pub fn total_probes(&self) -> u64 { self.total_probes }
}

impl Default for Backpressure {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bp() { assert_eq!(Backpressure::new().channel_count(), 0); }

    #[test]
    fn send_ack() {
        let mut bp = Backpressure::new();
        bp.register(1, 100, 20, 80);
        assert!(bp.send(1, 50).unwrap());
        bp.ack(1, 50).unwrap();
        assert_eq!(bp.state(1), Some(&ValveState::Open));
    }

    #[test]
    fn high_watermark() {
        let mut bp = Backpressure::new();
        bp.register(1, 100, 20, 80);
        bp.send(1, 80).unwrap();
        assert_eq!(bp.state(1), Some(&ValveState::Pressure));
    }

    #[test]
    fn full_closed() {
        let mut bp = Backpressure::new();
        bp.register(1, 100, 20, 80);
        bp.send(1, 100).unwrap();
        assert_eq!(bp.state(1), Some(&ValveState::Closed));
        assert!(!bp.send(1, 1).unwrap());
    }

    #[test]
    fn recover() {
        let mut bp = Backpressure::new();
        bp.register(1, 100, 20, 80);
        bp.send(1, 100).unwrap();
        bp.ack(1, 50).unwrap();
        assert_eq!(bp.state(1), Some(&ValveState::Open));
    }

    #[test]
    fn probe() {
        let mut bp = Backpressure::new();
        bp.register(1, 100, 20, 80);
        bp.send(1, 30).unwrap();
        let (state, used, avail) = bp.probe(1).unwrap();
        assert_eq!(state, ValveState::Open);
        assert_eq!(used, 30);
        assert_eq!(avail, 70);
    }

    #[test]
    fn channel_not_found() {
        let mut bp = Backpressure::new();
        let err = bp.send(99, 1).unwrap_err();
        assert!(matches!(err, BpError::ChannelNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut bp = Backpressure::new();
        bp.register(1, 100, 20, 80);
        bp.send(1, 50).unwrap();
        bp.probe(1).unwrap();
        assert_eq!(bp.total_accepted(), 1);
        assert_eq!(bp.total_probes(), 1);
    }

    #[test]
    fn reject_stats() {
        let mut bp = Backpressure::new();
        bp.register(1, 10, 5, 8);
        bp.send(1, 10).unwrap();
        bp.send(1, 1).unwrap();
        assert_eq!(bp.total_rejected(), 1);
    }

    #[test]
    fn error_display() { assert!(BpError::ChannelNotFound { id: 1 }.to_string().contains("1")); }
}

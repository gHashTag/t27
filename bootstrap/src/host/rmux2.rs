use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum MuxError {
    ChannelExists { id: u64 },
    ChannelNotFound { id: u64 },
    NoHealthyChannels,
    AllDown,
}

impl std::fmt::Display for MuxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MuxError::ChannelExists { id } => write!(f, "channel {id} exists"),
            MuxError::ChannelNotFound { id } => write!(f, "channel {id} not found"),
            MuxError::NoHealthyChannels => write!(f, "no healthy channels"),
            MuxError::AllDown => write!(f, "all channels down"),
        }
    }
}

impl std::error::Error for MuxError {}

struct Channel {
    id: u64,
    weight: u32,
    healthy: bool,
    sent: u64,
    errors: u64,
}

pub struct RingMux2 {
    channels: BTreeMap<u64, Channel>,
    order: Vec<u64>,
    cursor: usize,
    weight_counters: BTreeMap<u64, u32>,
    total_dispatched: u64,
    total_errors: u64,
}

impl RingMux2 {
    pub fn new() -> Self { Self { channels: BTreeMap::new(), order: Vec::new(), cursor: 0, weight_counters: BTreeMap::new(), total_dispatched: 0, total_errors: 0 } }

    pub fn add(&mut self, id: u64, weight: u32) -> Result<(), MuxError> {
        if self.channels.contains_key(&id) { return Err(MuxError::ChannelExists { id }); }
        self.channels.insert(id, Channel { id, weight, healthy: true, sent: 0, errors: 0 });
        self.order.push(id);
        self.weight_counters.insert(id, weight);
        Ok(())
    }

    pub fn remove(&mut self, id: u64) -> Result<(), MuxError> {
        if !self.channels.contains_key(&id) { return Err(MuxError::ChannelNotFound { id }); }
        self.channels.remove(&id);
        self.order.retain(|&x| x != id);
        self.weight_counters.remove(&id);
        if self.cursor >= self.order.len() { self.cursor = 0; }
        Ok(())
    }

    pub fn set_healthy(&mut self, id: u64, healthy: bool) -> Result<(), MuxError> {
        let ch = self.channels.get_mut(&id).ok_or(MuxError::ChannelNotFound { id })?;
        ch.healthy = healthy;
        if !healthy { self.weight_counters.insert(id, 0); }
        else { self.weight_counters.insert(id, self.channels[&id].weight); }
        Ok(())
    }

    pub fn next(&mut self) -> Result<u64, MuxError> {
        let healthy: Vec<u64> = self.order.iter().copied().filter(|&id| self.channels[&id].healthy).collect();
        if healthy.is_empty() { return Err(MuxError::NoHealthyChannels); }
        let max_w = self.channels.values().map(|c| c.weight).max().unwrap_or(1);
        for _ in 0..(self.order.len() * (max_w as usize + 1)) {
            let id = self.order[self.cursor % self.order.len()];
            self.cursor = (self.cursor + 1) % self.order.len();
            if !self.channels[&id].healthy { continue; }
            let counter = self.weight_counters.get_mut(&id).unwrap();
            if *counter > 0 {
                *counter -= 1;
                self.channels.get_mut(&id).unwrap().sent += 1;
                self.total_dispatched += 1;
                return Ok(id);
            }
        }
        for &id in &healthy { self.weight_counters.insert(id, self.channels[&id].weight); }
        let id = healthy[0];
        self.channels.get_mut(&id).unwrap().sent += 1;
        self.total_dispatched += 1;
        Ok(id)
    }

    pub fn record_error(&mut self, id: u64) -> Result<(), MuxError> {
        let ch = self.channels.get_mut(&id).ok_or(MuxError::ChannelNotFound { id })?;
        ch.errors += 1;
        self.total_errors += 1;
        Ok(())
    }

    pub fn channel_count(&self) -> usize { self.channels.len() }
    pub fn healthy_count(&self) -> usize { self.channels.values().filter(|c| c.healthy).count() }
    pub fn sent(&self, id: u64) -> Option<u64> { self.channels.get(&id).map(|c| c.sent) }
    pub fn errors(&self, id: u64) -> Option<u64> { self.channels.get(&id).map(|c| c.errors) }
    pub fn total_dispatched(&self) -> u64 { self.total_dispatched }
    pub fn total_errors(&self) -> u64 { self.total_errors }
}

impl Default for RingMux2 {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_mux() { assert_eq!(RingMux2::new().channel_count(), 0); }

    #[test]
    fn add_next() {
        let mut m = RingMux2::new();
        m.add(1, 1).unwrap(); m.add(2, 1).unwrap();
        let id = m.next().unwrap();
        assert!(id == 1 || id == 2);
    }

    #[test]
    fn weighted_dispatches_more_to_heavier() {
        let mut m = RingMux2::new();
        m.add(1, 3).unwrap(); m.add(2, 1).unwrap();
        let mut c1 = 0u64; let mut c2 = 0u64;
        for _ in 0..100 {
            let id = m.next().unwrap();
            if id == 1 { c1 += 1; } else { c2 += 1; }
        }
        assert!(c1 > c2, "c1={c1} c2={c2}");
    }

    #[test]
    fn skip_unhealthy() {
        let mut m = RingMux2::new();
        m.add(1, 1).unwrap(); m.add(2, 1).unwrap();
        m.set_healthy(1, false).unwrap();
        for _ in 0..5 { assert_eq!(m.next().unwrap(), 2); }
    }

    #[test]
    fn no_healthy() {
        let mut m = RingMux2::new();
        m.add(1, 1).unwrap();
        m.set_healthy(1, false).unwrap();
        let err = m.next().unwrap_err();
        assert!(matches!(err, MuxError::NoHealthyChannels));
    }

    #[test]
    fn remove() {
        let mut m = RingMux2::new();
        m.add(1, 1).unwrap(); m.add(2, 1).unwrap();
        m.remove(1).unwrap();
        assert_eq!(m.channel_count(), 1);
    }

    #[test]
    fn duplicate() {
        let mut m = RingMux2::new();
        m.add(1, 1).unwrap();
        let err = m.add(1, 1).unwrap_err();
        assert!(matches!(err, MuxError::ChannelExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut m = RingMux2::new();
        let err = m.remove(99).unwrap_err();
        assert!(matches!(err, MuxError::ChannelNotFound { .. }));
    }

    #[test]
    fn error_tracking() {
        let mut m = RingMux2::new();
        m.add(1, 1).unwrap();
        m.record_error(1).unwrap();
        assert_eq!(m.errors(1), Some(1));
        assert_eq!(m.total_errors(), 1);
    }

    #[test]
    fn sent_tracking() {
        let mut m = RingMux2::new();
        m.add(1, 1).unwrap();
        m.next().unwrap(); m.next().unwrap();
        assert_eq!(m.sent(1), Some(2));
    }

    #[test]
    fn health_restore() {
        let mut m = RingMux2::new();
        m.add(1, 1).unwrap(); m.add(2, 1).unwrap();
        m.set_healthy(1, false).unwrap();
        assert_eq!(m.healthy_count(), 1);
        m.set_healthy(1, true).unwrap();
        assert_eq!(m.healthy_count(), 2);
    }

    #[test]
    fn error_display() { assert!(MuxError::AllDown.to_string().contains("all")); }
}

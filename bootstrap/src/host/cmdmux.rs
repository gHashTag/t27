use std::collections::VecDeque;

pub type ChannelId = u8;

#[derive(Debug, Clone)]
pub struct MuxEntry {
    pub channel: ChannelId,
    pub data: Vec<u8>,
}

impl MuxEntry {
    pub fn new(channel: ChannelId, data: Vec<u8>) -> Self {
        Self { channel, data }
    }
}

#[derive(Debug, Clone)]
pub struct ChannelStats {
    pub id: ChannelId,
    pub sent: u64,
    pub received: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MuxError {
    ChannelFull { channel: ChannelId },
    NoChannels,
}

impl std::fmt::Display for MuxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MuxError::ChannelFull { channel } => write!(f, "channel {channel} full"),
            MuxError::NoChannels => write!(f, "no channels registered"),
        }
    }
}

impl std::error::Error for MuxError {}

#[derive(Debug, Clone)]
struct Channel {
    id: ChannelId,
    queue: VecDeque<Vec<u8>>,
    depth: usize,
    sent: u64,
    received: u64,
    bytes: u64,
}

#[derive(Debug, Clone)]
pub struct CommandMux {
    channels: Vec<Channel>,
    rr_index: usize,
    output: VecDeque<MuxEntry>,
}

impl CommandMux {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            rr_index: 0,
            output: VecDeque::new(),
        }
    }

    pub fn add_channel(&mut self, id: ChannelId, depth: usize) {
        self.channels.push(Channel {
            id,
            queue: VecDeque::with_capacity(depth),
            depth,
            sent: 0,
            received: 0,
            bytes: 0,
        });
    }

    pub fn remove_channel(&mut self, id: ChannelId) -> bool {
        let len_before = self.channels.len();
        self.channels.retain(|c| c.id != id);
        if self.rr_index >= self.channels.len() && !self.channels.is_empty() {
            self.rr_index = 0;
        }
        self.channels.len() != len_before
    }

    pub fn submit(&mut self, channel: ChannelId, data: Vec<u8>) -> Result<(), MuxError> {
        let ch = self.channels.iter_mut().find(|c| c.id == channel)
            .ok_or(MuxError::NoChannels)?;
        if ch.queue.len() >= ch.depth {
            return Err(MuxError::ChannelFull { channel });
        }
        ch.bytes += data.len() as u64;
        ch.sent += 1;
        ch.queue.push_back(data);
        Ok(())
    }

    pub fn drain_round(&mut self) -> Vec<MuxEntry> {
        let mut results = Vec::new();
        if self.channels.is_empty() {
            return results;
        }
        let n = self.channels.len();
        for _ in 0..n {
            let ch = &mut self.channels[self.rr_index];
            if let Some(data) = ch.queue.pop_front() {
                ch.received += 1;
                results.push(MuxEntry::new(ch.id, data));
            }
            self.rr_index = (self.rr_index + 1) % n;
        }
        results
    }

    pub fn drain_all(&mut self) -> Vec<MuxEntry> {
        let mut all = Vec::new();
        loop {
            let round = self.drain_round();
            if round.is_empty() {
                break;
            }
            all.extend(round);
        }
        all
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn pending_count(&self) -> usize {
        self.channels.iter().map(|c| c.queue.len()).sum()
    }

    pub fn channel_stats(&self, id: ChannelId) -> Option<ChannelStats> {
        self.channels.iter().find(|c| c.id == id).map(|c| ChannelStats {
            id: c.id,
            sent: c.sent,
            received: c.received,
            bytes: c.bytes,
        })
    }

    pub fn all_stats(&self) -> Vec<ChannelStats> {
        self.channels.iter().map(|c| ChannelStats {
            id: c.id,
            sent: c.sent,
            received: c.received,
            bytes: c.bytes,
        }).collect()
    }

    pub fn total_sent(&self) -> u64 {
        self.channels.iter().map(|c| c.sent).sum()
    }

    pub fn total_received(&self) -> u64 {
        self.channels.iter().map(|c| c.received).sum()
    }

    pub fn clear(&mut self) {
        for ch in &mut self.channels {
            ch.queue.clear();
        }
        self.output.clear();
    }
}

impl Default for CommandMux {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_remove_channel() {
        let mut mx = CommandMux::new();
        mx.add_channel(0, 4);
        mx.add_channel(1, 4);
        assert_eq!(mx.channel_count(), 2);
        assert!(mx.remove_channel(1));
        assert_eq!(mx.channel_count(), 1);
    }

    #[test]
    fn submit_and_drain_round() {
        let mut mx = CommandMux::new();
        mx.add_channel(0, 4);
        mx.add_channel(1, 4);
        mx.submit(0, vec![1]).unwrap();
        mx.submit(1, vec![2]).unwrap();
        let entries = mx.drain_round();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].channel, 0);
        assert_eq!(entries[1].channel, 1);
    }

    #[test]
    fn round_robin_order() {
        let mut mx = CommandMux::new();
        mx.add_channel(0, 4);
        mx.add_channel(1, 4);
        mx.submit(0, vec![0xA]).unwrap();
        mx.submit(0, vec![0xB]).unwrap();
        mx.submit(1, vec![0xC]).unwrap();
        let entries = mx.drain_round();
        assert_eq!(entries[0].data, vec![0xA]);
        assert_eq!(entries[1].data, vec![0xC]);
        assert_eq!(entries[0].channel, 0);
        assert_eq!(entries[1].channel, 1);
    }

    #[test]
    fn channel_full() {
        let mut mx = CommandMux::new();
        mx.add_channel(0, 2);
        mx.submit(0, vec![1]).unwrap();
        mx.submit(0, vec![2]).unwrap();
        let err = mx.submit(0, vec![3]).unwrap_err();
        assert!(matches!(err, MuxError::ChannelFull { .. }));
    }

    #[test]
    fn submit_unknown_channel() {
        let mut mx = CommandMux::new();
        let err = mx.submit(99, vec![1]).unwrap_err();
        assert!(matches!(err, MuxError::NoChannels));
    }

    #[test]
    fn drain_all() {
        let mut mx = CommandMux::new();
        mx.add_channel(0, 4);
        mx.submit(0, vec![1]).unwrap();
        mx.submit(0, vec![2]).unwrap();
        mx.submit(0, vec![3]).unwrap();
        let entries = mx.drain_all();
        assert_eq!(entries.len(), 3);
        assert_eq!(mx.pending_count(), 0);
    }

    #[test]
    fn channel_stats() {
        let mut mx = CommandMux::new();
        mx.add_channel(0, 4);
        mx.submit(0, vec![0xAA, 0xBB]).unwrap();
        mx.drain_round();
        let s = mx.channel_stats(0).unwrap();
        assert_eq!(s.sent, 1);
        assert_eq!(s.received, 1);
        assert_eq!(s.bytes, 2);
    }

    #[test]
    fn total_stats() {
        let mut mx = CommandMux::new();
        mx.add_channel(0, 4);
        mx.add_channel(1, 4);
        mx.submit(0, vec![1]).unwrap();
        mx.submit(1, vec![2]).unwrap();
        mx.drain_round();
        assert_eq!(mx.total_sent(), 2);
        assert_eq!(mx.total_received(), 2);
    }

    #[test]
    fn pending_count() {
        let mut mx = CommandMux::new();
        mx.add_channel(0, 4);
        mx.submit(0, vec![1]).unwrap();
        mx.submit(0, vec![2]).unwrap();
        assert_eq!(mx.pending_count(), 2);
    }

    #[test]
    fn clear() {
        let mut mx = CommandMux::new();
        mx.add_channel(0, 4);
        mx.submit(0, vec![1]).unwrap();
        mx.clear();
        assert_eq!(mx.pending_count(), 0);
    }

    #[test]
    fn error_display() {
        assert!(MuxError::ChannelFull { channel: 5 }.to_string().contains("5"));
        assert!(MuxError::NoChannels.to_string().contains("no channels"));
    }
}

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChanState {
    Open,
    Paused,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChanMuxError {
    ChannelFull { chan: usize },
    ChannelClosed { chan: usize },
    NoSuchChannel { chan: usize },
    NoReadyChannel,
}

impl std::fmt::Display for ChanMuxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChanMuxError::ChannelFull { chan } => write!(f, "channel {chan} full"),
            ChanMuxError::ChannelClosed { chan } => write!(f, "channel {chan} closed"),
            ChanMuxError::NoSuchChannel { chan } => write!(f, "no channel {chan}"),
            ChanMuxError::NoReadyChannel => write!(f, "no ready channel"),
        }
    }
}

impl std::error::Error for ChanMuxError {}

struct Channel<T> {
    id: usize,
    state: ChanState,
    queue: Vec<T>,
    depth: usize,
    total_sent: u64,
    total_recv: u64,
    total_dropped: u64,
}

impl<T> Channel<T> {
    fn new(id: usize, depth: usize) -> Self {
        Self { id, state: ChanState::Open, queue: Vec::new(), depth, total_sent: 0, total_recv: 0, total_dropped: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct ChanStats {
    pub chan: usize,
    pub state: ChanState,
    pub queued: usize,
    pub depth: usize,
    pub total_sent: u64,
    pub total_recv: u64,
}

pub struct ChanMux<T> {
    channels: BTreeMap<usize, Channel<T>>,
    rr_index: usize,
    channel_order: Vec<usize>,
}

impl<T> ChanMux<T> {
    pub fn new() -> Self {
        Self { channels: BTreeMap::new(), rr_index: 0, channel_order: Vec::new() }
    }

    pub fn add_channel(&mut self, id: usize, depth: usize) {
        self.channels.insert(id, Channel::new(id, depth));
        self.channel_order.push(id);
        self.channel_order.sort_unstable();
    }

    pub fn send(&mut self, chan: usize, item: T) -> Result<(), ChanMuxError> {
        let ch = self.channels.get_mut(&chan).ok_or(ChanMuxError::NoSuchChannel { chan })?;
        if ch.state == ChanState::Closed {
            return Err(ChanMuxError::ChannelClosed { chan });
        }
        if ch.queue.len() >= ch.depth {
            ch.total_dropped += 1;
            return Err(ChanMuxError::ChannelFull { chan });
        }
        ch.queue.push(item);
        ch.total_sent += 1;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<(usize, T), ChanMuxError> {
        let start = self.rr_index % self.channel_order.len().max(1);
        for _ in 0..self.channel_order.len() {
            let chan_id = self.channel_order[self.rr_index % self.channel_order.len()];
            self.rr_index = (self.rr_index + 1) % self.channel_order.len().max(1);
            if let Some(ch) = self.channels.get_mut(&chan_id) {
                if ch.state == ChanState::Open && !ch.queue.is_empty() {
                    let item = ch.queue.remove(0);
                    ch.total_recv += 1;
                    return Ok((chan_id, item));
                }
            }
        }
        Err(ChanMuxError::NoReadyChannel)
    }

    pub fn recv_from(&mut self, chan: usize) -> Option<T> {
        let ch = self.channels.get_mut(&chan)?;
        if ch.state != ChanState::Open || ch.queue.is_empty() { return None; }
        let item = ch.queue.remove(0);
        ch.total_recv += 1;
        Some(item)
    }

    pub fn set_state(&mut self, chan: usize, state: ChanState) -> Result<(), ChanMuxError> {
        let ch = self.channels.get_mut(&chan).ok_or(ChanMuxError::NoSuchChannel { chan })?;
        ch.state = state;
        Ok(())
    }

    pub fn state(&self, chan: usize) -> Option<ChanState> {
        self.channels.get(&chan).map(|ch| ch.state)
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn total_queued(&self) -> usize {
        self.channels.values().map(|ch| ch.queue.len()).sum()
    }

    pub fn stats(&self, chan: usize) -> Option<ChanStats> {
        self.channels.get(&chan).map(|ch| ChanStats {
            chan: ch.id,
            state: ch.state,
            queued: ch.queue.len(),
            depth: ch.depth,
            total_sent: ch.total_sent,
            total_recv: ch.total_recv,
        })
    }

    pub fn flush(&mut self, chan: usize) -> usize {
        if let Some(ch) = self.channels.get_mut(&chan) {
            let count = ch.queue.len();
            ch.queue.clear();
            count
        } else {
            0
        }
    }

    pub fn close_all(&mut self) {
        for ch in self.channels.values_mut() {
            ch.state = ChanState::Closed;
        }
    }
}

impl<T> Default for ChanMux<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_mux() {
        let mx: ChanMux<i32> = ChanMux::new();
        assert_eq!(mx.channel_count(), 0);
    }

    #[test]
    fn add_channel() {
        let mut mx: ChanMux<i32> = ChanMux::new();
        mx.add_channel(0, 4);
        mx.add_channel(1, 4);
        assert_eq!(mx.channel_count(), 2);
        assert_eq!(mx.state(0), Some(ChanState::Open));
    }

    #[test]
    fn send_recv_roundtrip() {
        let mut mx = ChanMux::new();
        mx.add_channel(0, 4);
        mx.send(0, 42).unwrap();
        let (ch, val) = mx.recv().unwrap();
        assert_eq!(ch, 0);
        assert_eq!(val, 42);
    }

    #[test]
    fn round_robin() {
        let mut mx = ChanMux::new();
        mx.add_channel(0, 4);
        mx.add_channel(1, 4);
        mx.send(0, 10).unwrap();
        mx.send(1, 20).unwrap();
        let (c0, v0) = mx.recv().unwrap();
        let (c1, v1) = mx.recv().unwrap();
        assert_eq!(c0, 0);
        assert_eq!(v0, 10);
        assert_eq!(c1, 1);
        assert_eq!(v1, 20);
    }

    #[test]
    fn channel_full() {
        let mut mx = ChanMux::new();
        mx.add_channel(0, 2);
        mx.send(0, 1).unwrap();
        mx.send(0, 2).unwrap();
        let err = mx.send(0, 3).unwrap_err();
        assert!(matches!(err, ChanMuxError::ChannelFull { chan: 0 }));
    }

    #[test]
    fn closed_channel() {
        let mut mx = ChanMux::new();
        mx.add_channel(0, 4);
        mx.set_state(0, ChanState::Closed).unwrap();
        let err = mx.send(0, 1).unwrap_err();
        assert!(matches!(err, ChanMuxError::ChannelClosed { .. }));
    }

    #[test]
    fn no_such_channel() {
        let mut mx: ChanMux<u8> = ChanMux::new();
        let err = mx.send(99, 1).unwrap_err();
        assert!(matches!(err, ChanMuxError::NoSuchChannel { .. }));
    }

    #[test]
    fn paused_skipped() {
        let mut mx = ChanMux::new();
        mx.add_channel(0, 4);
        mx.add_channel(1, 4);
        mx.send(0, 10).unwrap();
        mx.send(1, 20).unwrap();
        mx.set_state(0, ChanState::Paused).unwrap();
        let (ch, val) = mx.recv().unwrap();
        assert_eq!(ch, 1);
        assert_eq!(val, 20);
    }

    #[test]
    fn recv_from_specific() {
        let mut mx = ChanMux::new();
        mx.add_channel(0, 4);
        mx.add_channel(1, 4);
        mx.send(0, 10).unwrap();
        mx.send(1, 20).unwrap();
        let val = mx.recv_from(1).unwrap();
        assert_eq!(val, 20);
        assert_eq!(mx.total_queued(), 1);
    }

    #[test]
    fn flush() {
        let mut mx = ChanMux::new();
        mx.add_channel(0, 8);
        mx.send(0, 1).unwrap();
        mx.send(0, 2).unwrap();
        mx.send(0, 3).unwrap();
        assert_eq!(mx.flush(0), 3);
        assert_eq!(mx.total_queued(), 0);
    }

    #[test]
    fn stats() {
        let mut mx = ChanMux::new();
        mx.add_channel(0, 4);
        mx.send(0, 1).unwrap();
        mx.send(0, 2).unwrap();
        mx.recv().unwrap();
        let s = mx.stats(0).unwrap();
        assert_eq!(s.total_sent, 2);
        assert_eq!(s.total_recv, 1);
        assert_eq!(s.queued, 1);
    }

    #[test]
    fn close_all() {
        let mut mx: ChanMux<u8> = ChanMux::new();
        mx.add_channel(0, 4);
        mx.add_channel(1, 4);
        mx.close_all();
        assert_eq!(mx.state(0), Some(ChanState::Closed));
        assert_eq!(mx.state(1), Some(ChanState::Closed));
    }

    #[test]
    fn error_display() {
        assert!(ChanMuxError::NoReadyChannel.to_string().contains("ready"));
        assert!(ChanMuxError::ChannelFull { chan: 3 }.to_string().contains("3"));
    }
}

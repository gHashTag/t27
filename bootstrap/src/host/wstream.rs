#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    Idle,
    Streaming,
    Paused,
    Complete,
    Error,
}

impl std::fmt::Display for StreamState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamState::Idle => write!(f, "idle"),
            StreamState::Streaming => write!(f, "streaming"),
            StreamState::Paused => write!(f, "paused"),
            StreamState::Complete => write!(f, "complete"),
            StreamState::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamError {
    NotIdle,
    NotStreaming,
    AlreadyComplete,
    NoData,
    AddressOverflow { base: u32, offset: u32 },
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamError::NotIdle => write!(f, "not idle"),
            StreamError::NotStreaming => write!(f, "not streaming"),
            StreamError::AlreadyComplete => write!(f, "already complete"),
            StreamError::NoData => write!(f, "no data"),
            StreamError::AddressOverflow { base, offset } => {
                write!(f, "address overflow: 0x{base:X}+0x{offset:X}")
            }
        }
    }
}

impl std::error::Error for StreamError {}

#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub base_addr: u32,
    pub chunk_size: usize,
    pub total_weights: usize,
}

#[derive(Debug, Clone)]
pub struct StreamProgress {
    pub total: usize,
    pub transferred: usize,
    pub remaining: usize,
    pub chunks: usize,
    pub current_addr: u32,
}

impl StreamProgress {
    pub fn percent(&self) -> f64 {
        if self.total == 0 { 0.0 } else { self.transferred as f64 / self.total as f64 * 100.0 }
    }
}

#[derive(Debug, Clone)]
pub struct WeightStreamer {
    config: StreamConfig,
    state: StreamState,
    transferred: usize,
    current_addr: u32,
    total_chunks: u64,
    total_bytes: u64,
}

impl WeightStreamer {
    pub fn new(base_addr: u32, chunk_size: usize, total_weights: usize) -> Self {
        Self {
            config: StreamConfig { base_addr, chunk_size, total_weights },
            state: StreamState::Idle,
            transferred: 0,
            current_addr: base_addr,
            total_chunks: 0,
            total_bytes: 0,
        }
    }

    pub fn state(&self) -> StreamState {
        self.state
    }

    pub fn start(&mut self) -> Result<(), StreamError> {
        if self.state != StreamState::Idle {
            return Err(StreamError::NotIdle);
        }
        if self.config.total_weights == 0 {
            return Err(StreamError::NoData);
        }
        self.state = StreamState::Streaming;
        self.current_addr = self.config.base_addr;
        self.transferred = 0;
        Ok(())
    }

    pub fn next_chunk(&mut self) -> Option<(u32, usize)> {
        if self.state != StreamState::Streaming {
            return None;
        }
        let remaining = self.config.total_weights - self.transferred;
        if remaining == 0 {
            self.state = StreamState::Complete;
            return None;
        }
        let size = remaining.min(self.config.chunk_size);
        let addr = self.current_addr;
        self.transferred += size;
        self.total_chunks += 1;
        self.total_bytes += size as u64;
        self.current_addr = self.config.base_addr + (self.transferred * 4) as u32;
        Some((addr, size))
    }

    pub fn pause(&mut self) -> Result<(), StreamError> {
        if self.state != StreamState::Streaming {
            return Err(StreamError::NotStreaming);
        }
        self.state = StreamState::Paused;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), StreamError> {
        if self.state != StreamState::Paused {
            return Err(StreamError::NotStreaming);
        }
        self.state = StreamState::Streaming;
        Ok(())
    }

    pub fn abort(&mut self) {
        self.state = StreamState::Error;
    }

    pub fn progress(&self) -> StreamProgress {
        let remaining = self.config.total_weights.saturating_sub(self.transferred);
        StreamProgress {
            total: self.config.total_weights,
            transferred: self.transferred,
            remaining,
            chunks: self.total_chunks as usize,
            current_addr: self.current_addr,
        }
    }

    pub fn total_chunks(&self) -> u64 {
        self.total_chunks
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    pub fn is_complete(&self) -> bool {
        self.state == StreamState::Complete
    }

    pub fn reset(&mut self) {
        self.state = StreamState::Idle;
        self.transferred = 0;
        self.current_addr = self.config.base_addr;
        self.total_chunks = 0;
        self.total_bytes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_display() {
        assert_eq!(StreamState::Streaming.to_string(), "streaming");
    }

    #[test]
    fn new_streamer() {
        let ws = WeightStreamer::new(0x1000, 32, 100);
        assert_eq!(ws.state(), StreamState::Idle);
        assert!(!ws.is_complete());
    }

    #[test]
    fn start_and_stream() {
        let mut ws = WeightStreamer::new(0x1000, 32, 100);
        ws.start().unwrap();
        assert_eq!(ws.state(), StreamState::Streaming);
        let (addr, size) = ws.next_chunk().unwrap();
        assert_eq!(addr, 0x1000);
        assert_eq!(size, 32);
        assert_eq!(ws.progress().transferred, 32);
    }

    #[test]
    fn stream_to_completion() {
        let mut ws = WeightStreamer::new(0x1000, 50, 100);
        ws.start().unwrap();
        ws.next_chunk().unwrap();
        ws.next_chunk().unwrap();
        assert!(ws.next_chunk().is_none());
        assert!(ws.is_complete());
        assert_eq!(ws.progress().percent(), 100.0);
    }

    #[test]
    fn address_increments() {
        let mut ws = WeightStreamer::new(0x1000, 10, 30);
        ws.start().unwrap();
        let (a1, _) = ws.next_chunk().unwrap();
        let (a2, _) = ws.next_chunk().unwrap();
        let (a3, _) = ws.next_chunk().unwrap();
        assert_eq!(a1, 0x1000);
        assert_eq!(a2, 0x1028);
        assert_eq!(a3, 0x1050);
    }

    #[test]
    fn last_chunk_partial() {
        let mut ws = WeightStreamer::new(0, 10, 25);
        ws.start().unwrap();
        ws.next_chunk().unwrap();
        ws.next_chunk().unwrap();
        let (_, size) = ws.next_chunk().unwrap();
        assert_eq!(size, 5);
    }

    #[test]
    fn pause_resume() {
        let mut ws = WeightStreamer::new(0, 10, 100);
        ws.start().unwrap();
        ws.pause().unwrap();
        assert_eq!(ws.state(), StreamState::Paused);
        assert!(ws.next_chunk().is_none());
        ws.resume().unwrap();
        assert_eq!(ws.state(), StreamState::Streaming);
        ws.next_chunk().unwrap();
    }

    #[test]
    fn start_not_idle() {
        let mut ws = WeightStreamer::new(0, 10, 100);
        ws.start().unwrap();
        let err = ws.start().unwrap_err();
        assert!(matches!(err, StreamError::NotIdle));
    }

    #[test]
    fn start_no_data() {
        let mut ws = WeightStreamer::new(0, 10, 0);
        let err = ws.start().unwrap_err();
        assert!(matches!(err, StreamError::NoData));
    }

    #[test]
    fn pause_not_streaming() {
        let mut ws = WeightStreamer::new(0, 10, 100);
        let err = ws.pause().unwrap_err();
        assert!(matches!(err, StreamError::NotStreaming));
    }

    #[test]
    fn abort() {
        let mut ws = WeightStreamer::new(0, 10, 100);
        ws.start().unwrap();
        ws.abort();
        assert_eq!(ws.state(), StreamState::Error);
    }

    #[test]
    fn reset() {
        let mut ws = WeightStreamer::new(0, 10, 100);
        ws.start().unwrap();
        ws.next_chunk().unwrap();
        ws.reset();
        assert_eq!(ws.state(), StreamState::Idle);
        assert_eq!(ws.progress().transferred, 0);
    }

    #[test]
    fn stats() {
        let mut ws = WeightStreamer::new(0, 10, 30);
        ws.start().unwrap();
        ws.next_chunk().unwrap();
        ws.next_chunk().unwrap();
        assert_eq!(ws.total_chunks(), 2);
        assert_eq!(ws.total_bytes(), 20);
    }

    #[test]
    fn error_display() {
        assert!(StreamError::NotIdle.to_string().contains("idle"));
        assert!(StreamError::AddressOverflow { base: 1, offset: 2 }.to_string().contains("overflow"));
    }
}

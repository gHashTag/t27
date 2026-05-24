#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferState {
    Free,
    Acquired,
    Presented,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwapError {
    NoFreeBuffer { chain_len: usize },
    NotAcquired { slot: usize },
    AlreadyAcquired { slot: usize },
    InvalidSlot { slot: usize, chain_len: usize },
}

impl std::fmt::Display for SwapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwapError::NoFreeBuffer { chain_len } => write!(f, "no free buffer (chain={chain_len})"),
            SwapError::NotAcquired { slot } => write!(f, "slot {slot} not acquired"),
            SwapError::AlreadyAcquired { slot } => write!(f, "slot {slot} already acquired"),
            SwapError::InvalidSlot { slot, chain_len } => write!(f, "slot {slot} out of range (chain={chain_len})"),
        }
    }
}

impl std::error::Error for SwapError {}

#[derive(Debug, Clone)]
struct SwapBuffer {
    slot: usize,
    state: BufferState,
    frame: u64,
}

#[derive(Debug, Clone)]
pub struct SwapChain {
    buffers: Vec<SwapBuffer>,
    current_frame: u64,
    total_acquires: u64,
    total_presents: u64,
    total_drops: u64,
    acquired_slot: Option<usize>,
    presented_slot: Option<usize>,
}

impl SwapChain {
    pub fn new(chain_len: usize) -> Self {
        let buffers = (0..chain_len)
            .map(|slot| SwapBuffer { slot, state: BufferState::Free, frame: 0 })
            .collect();
        Self {
            buffers,
            current_frame: 0,
            total_acquires: 0,
            total_presents: 0,
            total_drops: 0,
            acquired_slot: None,
            presented_slot: None,
        }
    }

    pub fn chain_len(&self) -> usize {
        self.buffers.len()
    }

    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }

    pub fn acquire(&mut self) -> Result<usize, SwapError> {
        for buf in &mut self.buffers {
            if buf.state == BufferState::Free {
                buf.state = BufferState::Acquired;
                buf.frame = self.current_frame;
                self.total_acquires += 1;
                self.acquired_slot = Some(buf.slot);
                return Ok(buf.slot);
            }
        }
        Err(SwapError::NoFreeBuffer { chain_len: self.buffers.len() })
    }

    pub fn present(&mut self, slot: usize) -> Result<u64, SwapError> {
        if slot >= self.buffers.len() {
            return Err(SwapError::InvalidSlot { slot, chain_len: self.buffers.len() });
        }
        if self.buffers[slot].state != BufferState::Acquired {
            return Err(SwapError::NotAcquired { slot });
        }
        if let Some(prev) = self.presented_slot {
            if prev < self.buffers.len() && self.buffers[prev].state == BufferState::Presented {
                self.buffers[prev].state = BufferState::Free;
            }
        }
        self.buffers[slot].state = BufferState::Presented;
        self.total_presents += 1;
        self.current_frame += 1;
        self.presented_slot = Some(slot);
        Ok(self.current_frame)
    }

    pub fn release(&mut self, slot: usize) -> Result<(), SwapError> {
        if slot >= self.buffers.len() {
            return Err(SwapError::InvalidSlot { slot, chain_len: self.buffers.len() });
        }
        if self.buffers[slot].state == BufferState::Free {
            return Err(SwapError::NotAcquired { slot });
        }
        if self.buffers[slot].state == BufferState::Presented {
            self.total_drops += 1;
        }
        self.buffers[slot].state = BufferState::Free;
        if self.acquired_slot == Some(slot) { self.acquired_slot = None; }
        if self.presented_slot == Some(slot) { self.presented_slot = None; }
        Ok(())
    }

    pub fn state(&self, slot: usize) -> Option<BufferState> {
        self.buffers.get(slot).map(|b| b.state)
    }

    pub fn acquired_slot(&self) -> Option<usize> {
        self.acquired_slot
    }

    pub fn presented_slot(&self) -> Option<usize> {
        self.presented_slot
    }

    pub fn free_count(&self) -> usize {
        self.buffers.iter().filter(|b| b.state == BufferState::Free).count()
    }

    pub fn total_acquires(&self) -> u64 {
        self.total_acquires
    }

    pub fn total_presents(&self) -> u64 {
        self.total_presents
    }

    pub fn total_drops(&self) -> u64 {
        self.total_drops
    }

    pub fn reset(&mut self) {
        for buf in &mut self.buffers {
            buf.state = BufferState::Free;
            buf.frame = 0;
        }
        self.current_frame = 0;
        self.acquired_slot = None;
        self.presented_slot = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_swapchain() {
        let sc = SwapChain::new(3);
        assert_eq!(sc.chain_len(), 3);
        assert_eq!(sc.current_frame(), 0);
        assert_eq!(sc.free_count(), 3);
    }

    #[test]
    fn acquire_returns_slot() {
        let mut sc = SwapChain::new(3);
        let slot = sc.acquire().unwrap();
        assert_eq!(slot, 0);
        assert_eq!(sc.state(0), Some(BufferState::Acquired));
        assert_eq!(sc.acquired_slot(), Some(0));
    }

    #[test]
    fn present_advances_frame() {
        let mut sc = SwapChain::new(3);
        let slot = sc.acquire().unwrap();
        let frame = sc.present(slot).unwrap();
        assert_eq!(frame, 1);
        assert_eq!(sc.current_frame(), 1);
    }

    #[test]
    fn full_acquire_cycle() {
        let mut sc = SwapChain::new(2);
        let s0 = sc.acquire().unwrap();
        sc.present(s0).unwrap();
        let s1 = sc.acquire().unwrap();
        assert_eq!(s1, 1);
        sc.present(s1).unwrap();
        let s2 = sc.acquire().unwrap();
        assert_eq!(s2, 0);
    }

    #[test]
    fn no_free_buffer() {
        let mut sc = SwapChain::new(2);
        sc.acquire().unwrap();
        sc.acquire().unwrap();
        let err = sc.acquire().unwrap_err();
        assert!(matches!(err, SwapError::NoFreeBuffer { .. }));
    }

    #[test]
    fn present_not_acquired() {
        let mut sc = SwapChain::new(2);
        let err = sc.present(0).unwrap_err();
        assert!(matches!(err, SwapError::NotAcquired { .. }));
    }

    #[test]
    fn invalid_slot() {
        let mut sc = SwapChain::new(2);
        let err = sc.present(5).unwrap_err();
        assert!(matches!(err, SwapError::InvalidSlot { .. }));
    }

    #[test]
    fn release_frees_buffer() {
        let mut sc = SwapChain::new(2);
        let s = sc.acquire().unwrap();
        sc.release(s).unwrap();
        assert_eq!(sc.state(s), Some(BufferState::Free));
        assert_eq!(sc.free_count(), 2);
    }

    #[test]
    fn stats() {
        let mut sc = SwapChain::new(3);
        let s = sc.acquire().unwrap();
        sc.present(s).unwrap();
        sc.release(s).unwrap();
        assert_eq!(sc.total_acquires(), 1);
        assert_eq!(sc.total_presents(), 1);
        assert_eq!(sc.total_drops(), 1);
    }

    #[test]
    fn reset() {
        let mut sc = SwapChain::new(2);
        sc.acquire().unwrap();
        sc.reset();
        assert_eq!(sc.free_count(), 2);
        assert_eq!(sc.current_frame(), 0);
        assert_eq!(sc.acquired_slot(), None);
    }

    #[test]
    fn present_releases_old() {
        let mut sc = SwapChain::new(2);
        let s0 = sc.acquire().unwrap();
        sc.present(s0).unwrap();
        let s1 = sc.acquire().unwrap();
        sc.present(s1).unwrap();
        assert_eq!(sc.state(0), Some(BufferState::Free));
    }

    #[test]
    fn error_display() {
        assert!(SwapError::NoFreeBuffer { chain_len: 3 }.to_string().contains("3"));
        assert!(SwapError::InvalidSlot { slot: 5, chain_len: 2 }.to_string().contains("5"));
    }
}

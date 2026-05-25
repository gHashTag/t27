#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowKind {
    Tumbling { size: u64 },
    Sliding { size: u64, slide: u64 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamErr {
    InvalidWindow { size: u64 },
    SlideLargerThanSize { slide: u64, size: u64 },
}

impl std::fmt::Display for StreamErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamErr::InvalidWindow { size } => write!(f, "invalid window size {size}"),
            StreamErr::SlideLargerThanSize { slide, size } => write!(f, "slide {slide} > size {size}"),
        }
    }
}

impl std::error::Error for StreamErr {}

#[derive(Clone)]
struct Event {
    ts: u64,
    value: f64,
}

#[derive(Clone)]
struct Window {
    start: u64,
    end: u64,
    events: Vec<Event>,
    sum: f64,
    count: usize,
}

pub struct Streaming {
    kind: WindowKind,
    windows: Vec<Window>,
    buffer: Vec<Event>,
    total_events: u64,
    total_windows: u64,
}

impl Streaming {
    pub fn new(kind: WindowKind) -> Result<Self, StreamErr> {
        match kind {
            WindowKind::Tumbling { size } if size == 0 => return Err(StreamErr::InvalidWindow { size }),
            WindowKind::Sliding { size, slide } => {
                if size == 0 { return Err(StreamErr::InvalidWindow { size }); }
                if slide > size { return Err(StreamErr::SlideLargerThanSize { slide, size }); }
            }
            _ => {}
        }
        Ok(Self { kind, windows: Vec::new(), buffer: Vec::new(), total_events: 0, total_windows: 0 })
    }

    pub fn push(&mut self, ts: u64, value: f64) {
        self.total_events += 1;
        self.buffer.push(Event { ts, value });
        self.try_flush();
    }

    fn try_flush(&mut self) {
        match self.kind {
            WindowKind::Tumbling { size } => self.flush_tumbling(size),
            WindowKind::Sliding { size, slide } => self.flush_sliding(size, slide),
        }
    }

    fn flush_tumbling(&mut self, size: u64) {
        if self.buffer.is_empty() { return; }
        let max_ts = self.buffer.iter().map(|e| e.ts).max().unwrap();
        let min_ts = self.buffer.iter().map(|e| e.ts).min().unwrap();
        let window_end = ((min_ts / size) + 1) * size;
        if max_ts < window_end { return; }
        let mut remaining = Vec::new();
        let mut win_events = Vec::new();
        for e in self.buffer.drain(..) {
            if e.ts < window_end { win_events.push(e); } else { remaining.push(e); }
        }
        self.buffer = remaining;
        if !win_events.is_empty() {
            let sum: f64 = win_events.iter().map(|e| e.value).sum();
            let count = win_events.len();
            self.windows.push(Window { start: window_end - size, end: window_end, events: win_events, sum, count });
            self.total_windows += 1;
        }
    }

    fn flush_sliding(&mut self, size: u64, slide: u64) {
        if self.buffer.is_empty() { return; }
        let max_ts = self.buffer.iter().map(|e| e.ts).max().unwrap();
        let min_start = if max_ts >= size { max_ts - size + 1 } else { 0 };
        let first_window_start = (min_start / slide) * slide;
        let mut ws = first_window_start;
        while ws + size <= max_ts + 1 {
            let win_start = ws;
            let win_end = ws + size;
            let win_events: Vec<Event> = self.buffer.iter().filter(|e| e.ts >= win_start && e.ts < win_end).cloned().collect();
            if !win_events.is_empty() {
                let sum: f64 = win_events.iter().map(|e| e.value).sum();
                let count = win_events.len();
                self.windows.push(Window { start: win_start, end: win_end, events: win_events, sum, count });
                self.total_windows += 1;
            }
            ws += slide;
        }
        let cutoff = if max_ts >= size { max_ts - size + 1 } else { 0 };
        self.buffer.retain(|e| e.ts >= cutoff);
    }

    pub fn window_count(&self) -> usize { self.windows.len() }
    pub fn window_sum(&self, idx: usize) -> Option<f64> { self.windows.get(idx).map(|w| w.sum) }
    pub fn window_avg(&self, idx: usize) -> Option<f64> { self.windows.get(idx).map(|w| if w.count > 0 { w.sum / w.count as f64 } else { 0.0 }) }
    pub fn window_range(&self, idx: usize) -> Option<(u64, u64)> { self.windows.get(idx).map(|w| (w.start, w.end)) }
    pub fn window_count_events(&self, idx: usize) -> Option<usize> { self.windows.get(idx).map(|w| w.count) }
    pub fn total_events(&self) -> u64 { self.total_events }
    pub fn total_windows(&self) -> u64 { self.total_windows }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tumbling() {
        let mut s = Streaming::new(WindowKind::Tumbling { size: 5 }).unwrap();
        for i in 0..6u64 { s.push(i, 1.0); }
        assert!(s.window_count() >= 1);
        assert!((s.window_sum(0).unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn tumbling_avg() {
        let mut s = Streaming::new(WindowKind::Tumbling { size: 3 }).unwrap();
        for i in 0..4u64 { s.push(i, i as f64); }
        assert!((s.window_avg(0).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn sliding() {
        let mut s = Streaming::new(WindowKind::Sliding { size: 5, slide: 2 }).unwrap();
        for i in 0..10u64 { s.push(i, 1.0); }
        assert!(s.window_count() >= 2);
    }

    #[test]
    fn sliding_range() {
        let mut s = Streaming::new(WindowKind::Sliding { size: 5, slide: 5 }).unwrap();
        for i in 0..10u64 { s.push(i, 1.0); }
        let (start, end) = s.window_range(0).unwrap();
        assert_eq!(end - start, 5);
    }

    #[test]
    fn zero_size() { assert!(Streaming::new(WindowKind::Tumbling { size: 0 }).is_err()); }

    #[test]
    fn slide_too_big() { assert!(Streaming::new(WindowKind::Sliding { size: 5, slide: 10 }).is_err()); }

    #[test]
    fn stats() {
        let mut s = Streaming::new(WindowKind::Tumbling { size: 3 }).unwrap();
        s.push(0, 1.0); s.push(1, 2.0); s.push(2, 3.0);
        assert_eq!(s.total_events(), 3);
    }

    #[test]
    fn empty_window_count() { let s = Streaming::new(WindowKind::Tumbling { size: 10 }).unwrap(); assert_eq!(s.window_count(), 0); }

    #[test]
    fn error_display() { assert!(StreamErr::InvalidWindow { size: 0 }.to_string().contains("invalid")); }
}

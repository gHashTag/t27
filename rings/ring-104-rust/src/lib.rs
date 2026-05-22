//! ring-104 — **Telemetry Bus**
//!
//! Wave 12 / Track C scaffolding. A bounded, *lossy* ring buffer collecting
//! key-value telemetry samples from on-chip counters. When the buffer is full
//! the **oldest** sample is dropped (FIFO eviction). Designed for fixed-memory
//! environments — no allocation past `new()`.
//!
//! ## Status (honest)
//! * Compilation **not** yet verified in CI.
//! * Not part of the workspace `members` — opt-in until Wave 12 Track D.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// A single telemetry sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Monotonic timestamp (chip-local clock, arbitrary units).
    pub ts: u64,
    /// 4-byte ASCII tag — e.g. `b"TEMP"`, `b"VDD0"`, `b"HOPS"`.
    pub tag: [u8; 4],
    /// Scalar value.
    pub value: f32,
}

/// Bounded ring buffer for telemetry samples.
#[derive(Debug, Clone)]
pub struct TelemetryBus {
    buf: alloc_vec_polyfill::Vec<Sample>,
    head: usize,
    len: usize,
    cap: usize,
    dropped: u64,
}

impl TelemetryBus {
    /// Build a ring buffer with capacity `cap`. Panics if `cap == 0`.
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0, "TelemetryBus capacity must be > 0");
        let placeholder = Sample { ts: 0, tag: [0; 4], value: 0.0 };
        let buf = (0..cap).map(|_| placeholder).collect::<alloc_vec_polyfill::Vec<_>>();
        Self { buf, head: 0, len: 0, cap, dropped: 0 }
    }

    /// Push a sample. If the buffer is full, the oldest sample is evicted
    /// and `dropped()` is incremented.
    pub fn push(&mut self, s: Sample) {
        if self.len == self.cap {
            // overwrite oldest
            self.buf[self.head] = s;
            self.head = (self.head + 1) % self.cap;
            self.dropped += 1;
        } else {
            let slot = (self.head + self.len) % self.cap;
            self.buf[slot] = s;
            self.len += 1;
        }
    }

    /// Number of samples currently buffered.
    pub fn len(&self) -> usize {
        self.len
    }

    /// `true` iff the buffer holds no samples.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Capacity.
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Number of samples dropped due to overflow since construction.
    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    /// Drain *all* samples in FIFO order. Buffer becomes empty afterwards.
    pub fn drain(&mut self) -> alloc_vec_polyfill::Vec<Sample> {
        let mut out = alloc_vec_polyfill::Vec::new();
        for i in 0..self.len {
            let idx = (self.head + i) % self.cap;
            out.push(self.buf[idx]);
        }
        self.head = 0;
        self.len = 0;
        out
    }

    /// Mean value of samples whose tag matches `tag`. Returns `None` if no
    /// matching samples are present.
    pub fn mean_by_tag(&self, tag: [u8; 4]) -> Option<f32> {
        let mut sum = 0.0f32;
        let mut n = 0u32;
        for i in 0..self.len {
            let idx = (self.head + i) % self.cap;
            let s = self.buf[idx];
            if s.tag == tag {
                sum += s.value;
                n += 1;
            }
        }
        if n == 0 { None } else { Some(sum / n as f32) }
    }
}

/// Identity witness — see ring-100.
pub fn identity_witness() -> bool {
    let phi = (1.0_f64 + 5.0_f64.sqrt()) / 2.0;
    ((phi * phi + 1.0 / (phi * phi)) - 3.0).abs() < 1e-15
}

mod alloc_vec_polyfill {
    extern crate alloc;
    pub use alloc::vec::Vec;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(ts: u64, tag: &[u8; 4], v: f32) -> Sample {
        Sample { ts, tag: *tag, value: v }
    }

    #[test]
    fn identity_witness_holds() {
        assert!(identity_witness());
    }

    #[test]
    fn push_and_drain_in_fifo_order() {
        let mut bus = TelemetryBus::new(4);
        bus.push(s(1, b"TEMP", 30.0));
        bus.push(s(2, b"TEMP", 31.0));
        bus.push(s(3, b"VDD0", 0.9));
        assert_eq!(bus.len(), 3);
        let out = bus.drain();
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].ts, 1);
        assert_eq!(out[1].ts, 2);
        assert_eq!(out[2].ts, 3);
        assert!(bus.is_empty());
    }

    #[test]
    fn overflow_evicts_oldest_and_counts_drops() {
        let mut bus = TelemetryBus::new(2);
        bus.push(s(1, b"HOPS", 1.0));
        bus.push(s(2, b"HOPS", 2.0));
        bus.push(s(3, b"HOPS", 3.0)); // evicts ts=1
        bus.push(s(4, b"HOPS", 4.0)); // evicts ts=2
        assert_eq!(bus.dropped(), 2);
        let out = bus.drain();
        assert_eq!(out[0].ts, 3);
        assert_eq!(out[1].ts, 4);
    }

    #[test]
    fn mean_by_tag_filters_correctly() {
        let mut bus = TelemetryBus::new(8);
        bus.push(s(1, b"TEMP", 10.0));
        bus.push(s(2, b"TEMP", 20.0));
        bus.push(s(3, b"VDD0", 0.9));
        assert!((bus.mean_by_tag(*b"TEMP").unwrap() - 15.0).abs() < 1e-6);
        assert!((bus.mean_by_tag(*b"VDD0").unwrap() - 0.9).abs() < 1e-6);
        assert!(bus.mean_by_tag(*b"NONE").is_none());
    }

    #[test]
    fn capacity_is_preserved() {
        let bus = TelemetryBus::new(16);
        assert_eq!(bus.capacity(), 16);
        assert_eq!(bus.len(), 0);
        assert!(bus.is_empty());
    }

    #[test]
    #[should_panic(expected = "capacity must be > 0")]
    fn zero_capacity_panics() {
        let _ = TelemetryBus::new(0);
    }
}

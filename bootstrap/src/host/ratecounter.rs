use std::collections::VecDeque;

pub struct RateCounter {
    events: VecDeque<(u64, u64)>,
    window_ms: u64,
    total_events: u64,
    total_bytes: u64,
    latencies: Vec<u64>,
    max_latency_samples: usize,
}

impl RateCounter {
    pub fn new(window_ms: u64, max_latency_samples: usize) -> Self {
        Self { events: VecDeque::new(), window_ms, total_events: 0, total_bytes: 0, latencies: Vec::with_capacity(max_latency_samples), max_latency_samples }
    }

    pub fn record(&mut self, timestamp: u64, bytes: u64) {
        self.events.push_back((timestamp, bytes));
        self.total_events += 1;
        self.total_bytes += bytes;
        self.evict(timestamp);
    }

    pub fn record_latency(&mut self, latency_us: u64) {
        if self.latencies.len() >= self.max_latency_samples {
            let _ = self.latencies.remove(0);
        }
        self.latencies.push(latency_us);
    }

    fn evict(&mut self, now: u64) {
        let cutoff = now.saturating_sub(self.window_ms);
        while let Some(&(ts, _)) = self.events.front() {
            if ts < cutoff { self.events.pop_front(); } else { break; }
        }
    }

    pub fn rate_per_second(&mut self, now: u64) -> f64 {
        self.evict(now);
        if self.events.is_empty() { return 0.0; }
        let first_ts = self.events.front().map(|(ts, _)| *ts).unwrap_or(now);
        let duration_ms = (now - first_ts).max(1);
        self.events.len() as f64 / (duration_ms as f64 / 1000.0)
    }

    pub fn throughput(&mut self, now: u64) -> u64 {
        self.evict(now);
        self.events.iter().map(|&(_, b)| b).sum()
    }

    pub fn event_count(&mut self, now: u64) -> usize {
        self.evict(now);
        self.events.len()
    }

    pub fn percentile(&self, p: f64) -> u64 {
        if self.latencies.is_empty() { return 0; }
        let mut sorted = self.latencies.clone();
        sorted.sort();
        let idx = ((p / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    pub fn min_latency(&self) -> u64 { self.latencies.iter().copied().min().unwrap_or(0) }
    pub fn max_latency(&self) -> u64 { self.latencies.iter().copied().max().unwrap_or(0) }
    pub fn avg_latency(&self) -> u64 {
        if self.latencies.is_empty() { return 0; }
        self.latencies.iter().sum::<u64>() / self.latencies.len() as u64
    }
    pub fn total_events(&self) -> u64 { self.total_events }
    pub fn total_bytes(&self) -> u64 { self.total_bytes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_counter() { let rc = RateCounter::new(1000, 100); assert_eq!(rc.total_events(), 0); }

    #[test]
    fn record_count() {
        let mut rc = RateCounter::new(1000, 100);
        rc.record(100, 10); rc.record(200, 20); rc.record(300, 30);
        assert_eq!(rc.event_count(300), 3);
    }

    #[test]
    fn window_eviction() {
        let mut rc = RateCounter::new(100, 100);
        rc.record(0, 10); rc.record(50, 20); rc.record(150, 30);
        assert_eq!(rc.event_count(150), 2);
    }

    #[test]
    fn throughput() {
        let mut rc = RateCounter::new(1000, 100);
        rc.record(100, 10); rc.record(200, 20);
        assert_eq!(rc.throughput(200), 30);
    }

    #[test]
    fn rate_per_second() {
        let mut rc = RateCounter::new(1000, 100);
        for i in 0..10 { rc.record(i * 100, 1); }
        let rate = rc.rate_per_second(900);
        assert!(rate > 0.0);
    }

    #[test]
    fn percentile_latency() {
        let mut rc = RateCounter::new(1000, 100);
        for i in 1..=100 { rc.record_latency(i); }
        let p50 = rc.percentile(50.0);
        assert!(p50 >= 45 && p50 <= 55);
        let p99 = rc.percentile(99.0);
        assert!(p99 >= 90);
    }

    #[test]
    fn min_max_avg() {
        let mut rc = RateCounter::new(1000, 100);
        rc.record_latency(10); rc.record_latency(50); rc.record_latency(100);
        assert_eq!(rc.min_latency(), 10);
        assert_eq!(rc.max_latency(), 100);
        assert_eq!(rc.avg_latency(), 53);
    }

    #[test]
    fn empty_percentile() {
        let rc = RateCounter::new(1000, 100);
        assert_eq!(rc.percentile(50.0), 0);
    }

    #[test]
    fn total_stats() {
        let mut rc = RateCounter::new(1000, 100);
        rc.record(100, 50); rc.record(200, 100);
        assert_eq!(rc.total_events(), 2);
        assert_eq!(rc.total_bytes(), 150);
    }

    #[test]
    fn latency_sample_cap() {
        let mut rc = RateCounter::new(1000, 5);
        for i in 0..10 { rc.record_latency(i); }
        assert_eq!(rc.latencies.len(), 5);
    }
}

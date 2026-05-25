pub struct RateCounter {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_tick: u64,
}

impl RateCounter {
    pub fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self { tokens: max_tokens, max_tokens, refill_rate, last_tick: 0 }
    }

    pub fn try_consume(&mut self, now: u64, cost: f64) -> bool {
        self.refill(now);
        if self.tokens >= cost {
            self.tokens -= cost;
            true
        } else { false }
    }

    fn refill(&mut self, now: u64) {
        if now > self.last_tick {
            let elapsed = (now - self.last_tick) as f64;
            self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
            self.last_tick = now;
        }
    }

    pub fn available(&mut self, now: u64) -> f64 { self.refill(now); self.tokens }

    pub fn sliding_window(events: &[u64], window: u64, now: u64) -> usize {
        let cutoff = now.saturating_sub(window);
        events.iter().filter(|&&t| t >= cutoff).count()
    }

    pub fn rate_per_second(events: &[u64], window: u64, now: u64) -> f64 {
        let count = Self::sliding_window(events, window, now) as f64;
        if window == 0 { return 0.0; }
        count / window as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_consume() {
        let mut rc = RateCounter::new(10.0, 1.0);
        assert!(rc.try_consume(0, 5.0));
        assert!(rc.try_consume(0, 5.0));
        assert!(!rc.try_consume(0, 1.0));
    }

    #[test]
    fn refill() {
        let mut rc = RateCounter::new(10.0, 1.0);
        rc.try_consume(0, 10.0);
        assert!(!rc.try_consume(0, 1.0));
        assert!(rc.try_consume(5, 5.0));
    }

    #[test]
    fn max_cap() {
        let mut rc = RateCounter::new(10.0, 100.0);
        assert!(rc.available(100) <= 10.0);
    }

    #[test]
    fn sliding_window() {
        let events = vec![1, 3, 5, 7, 9, 15, 20];
        assert_eq!(RateCounter::sliding_window(&events, 5, 10), 5);
        assert_eq!(RateCounter::sliding_window(&events, 100, 20), 7);
    }

    #[test]
    fn rate() {
        let events = vec![1, 2, 3, 4, 5];
        let r = RateCounter::rate_per_second(&events, 10, 10);
        assert!((r - 0.5).abs() < 1e-9);
    }
}

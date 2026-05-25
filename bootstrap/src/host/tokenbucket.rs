pub struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_time: u64,
    total_requests: u64,
    total_accepted: u64,
    total_rejected: u64,
}

impl TokenBucket {
    pub fn new(max_tokens: f64, refill_per_sec: f64) -> Self { Self { tokens: max_tokens, max_tokens, refill_rate: refill_per_sec, last_time: 0, total_requests: 0, total_accepted: 0, total_rejected: 0 } }

    pub fn try_consume(&mut self, now_ms: u64, count: f64) -> bool {
        self.total_requests += 1;
        self.refill(now_ms);
        if self.tokens >= count { self.tokens -= count; self.total_accepted += 1; true } else { self.total_rejected += 1; false }
    }

    pub fn consume_or_wait(&mut self, now_ms: u64, count: f64) -> u64 {
        self.refill(now_ms);
        if self.tokens >= count { self.tokens -= count; self.total_accepted += 1; self.total_requests += 1; 0 }
        else {
            let deficit = count - self.tokens;
            let wait_ms = ((deficit / self.refill_rate) * 1000.0).ceil() as u64;
            self.total_requests += 1;
            self.total_rejected += 1;
            wait_ms
        }
    }

    fn refill(&mut self, now_ms: u64) {
        if now_ms > self.last_time {
            let elapsed = (now_ms - self.last_time) as f64 / 1000.0;
            self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
            self.last_time = now_ms;
        }
    }

    pub fn set_time(&mut self, t: u64) { self.last_time = t; }
    pub fn tokens(&self) -> f64 { self.tokens }
    pub fn max_tokens(&self) -> f64 { self.max_tokens }
    pub fn refill_rate(&self) -> f64 { self.refill_rate }
    pub fn total_requests(&self) -> u64 { self.total_requests }
    pub fn total_accepted(&self) -> u64 { self.total_accepted }
    pub fn total_rejected(&self) -> u64 { self.total_rejected }
    pub fn accept_rate(&self) -> f64 { if self.total_requests == 0 { 1.0 } else { self.total_accepted as f64 / self.total_requests as f64 } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_accept() {
        let mut tb = TokenBucket::new(10.0, 1.0);
        tb.set_time(0);
        assert!(tb.try_consume(0, 5.0));
    }

    #[test]
    fn reject_on_empty() {
        let mut tb = TokenBucket::new(5.0, 1.0);
        tb.set_time(0);
        tb.try_consume(0, 5.0);
        assert!(!tb.try_consume(0, 1.0));
    }

    #[test]
    fn refill() {
        let mut tb = TokenBucket::new(10.0, 10.0);
        tb.set_time(0);
        tb.try_consume(0, 10.0);
        assert!(tb.try_consume(1000, 5.0));
    }

    #[test]
    fn max_cap() {
        let mut tb = TokenBucket::new(10.0, 100.0);
        tb.set_time(0);
        tb.refill(100000);
        assert!((tb.tokens() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn consume_or_wait() {
        let mut tb = TokenBucket::new(5.0, 10.0);
        tb.set_time(0);
        tb.try_consume(0, 5.0);
        let wait = tb.consume_or_wait(0, 5.0);
        assert!(wait > 0);
    }

    #[test]
    fn accept_rate() {
        let mut tb = TokenBucket::new(10.0, 1.0);
        tb.set_time(0);
        tb.try_consume(0, 5.0); tb.try_consume(0, 5.0); tb.try_consume(0, 1.0);
        assert!((tb.accept_rate() - (2.0 / 3.0)).abs() < 1e-10);
    }

    #[test]
    fn stats() {
        let mut tb = TokenBucket::new(10.0, 1.0);
        tb.set_time(0);
        tb.try_consume(0, 1.0);
        assert_eq!(tb.total_requests(), 1);
        assert_eq!(tb.total_accepted(), 1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogError {
    AlreadyRunning,
    NotRunning,
    FeedExpired { deadline_ms: u64, now_ms: u64 },
}

impl std::fmt::Display for WatchdogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WatchdogError::AlreadyRunning => write!(f, "watchdog already running"),
            WatchdogError::NotRunning => write!(f, "watchdog not running"),
            WatchdogError::FeedExpired { deadline_ms, now_ms } => {
                write!(f, "feed expired: deadline {deadline_ms}ms, now {now_ms}ms")
            }
        }
    }
}

impl std::error::Error for WatchdogError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogState {
    Idle,
    Running,
    Expired,
    Stopped,
}

#[derive(Debug, Clone, Copy)]
pub struct WatchdogConfig {
    pub timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub max_missed_heartbeats: u8,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            heartbeat_interval_ms: 100,
            max_missed_heartbeats: 10,
        }
    }
}

impl WatchdogConfig {
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    pub fn with_heartbeat_interval(mut self, ms: u64) -> Self {
        self.heartbeat_interval_ms = ms;
        self
    }

    pub fn with_max_missed(mut self, count: u8) -> Self {
        self.max_missed_heartbeats = count;
        self
    }
}

#[derive(Debug, Clone)]
pub struct WatchdogTimer {
    config: WatchdogConfig,
    state: WatchdogState,
    start_ms: u64,
    last_feed_ms: u64,
    missed_heartbeats: u8,
    total_feeds: u64,
    total_expirations: u64,
}

impl WatchdogTimer {
    pub fn new(config: WatchdogConfig) -> Self {
        Self {
            config,
            state: WatchdogState::Idle,
            start_ms: 0,
            last_feed_ms: 0,
            missed_heartbeats: 0,
            total_feeds: 0,
            total_expirations: 0,
        }
    }

    pub fn state(&self) -> WatchdogState {
        self.state
    }

    pub fn start(&mut self, now_ms: u64) -> Result<(), WatchdogError> {
        if self.state == WatchdogState::Running {
            return Err(WatchdogError::AlreadyRunning);
        }
        self.state = WatchdogState::Running;
        self.start_ms = now_ms;
        self.last_feed_ms = now_ms;
        self.missed_heartbeats = 0;
        Ok(())
    }

    pub fn feed(&mut self, now_ms: u64) -> Result<(), WatchdogError> {
        if self.state != WatchdogState::Running {
            return Err(WatchdogError::NotRunning);
        }
        if now_ms > self.last_feed_ms + self.config.timeout_ms {
            self.state = WatchdogState::Expired;
            self.total_expirations += 1;
            return Err(WatchdogError::FeedExpired {
                deadline_ms: self.last_feed_ms + self.config.timeout_ms,
                now_ms,
            });
        }
        self.last_feed_ms = now_ms;
        self.missed_heartbeats = 0;
        self.total_feeds += 1;
        Ok(())
    }

    pub fn check(&mut self, now_ms: u64) -> WatchdogState {
        if self.state != WatchdogState::Running {
            return self.state;
        }
        let elapsed_since_feed = now_ms.saturating_sub(self.last_feed_ms);
        if elapsed_since_feed > self.config.timeout_ms {
            self.state = WatchdogState::Expired;
            self.total_expirations += 1;
        }
        self.state
    }

    pub fn record_heartbeat(&mut self) -> bool {
        if self.state != WatchdogState::Running {
            return false;
        }
        self.missed_heartbeats = 0;
        true
    }

    pub fn record_missed_heartbeat(&mut self) -> bool {
        if self.state != WatchdogState::Running {
            return false;
        }
        self.missed_heartbeats += 1;
        if self.missed_heartbeats >= self.config.max_missed_heartbeats {
            self.state = WatchdogState::Expired;
            self.total_expirations += 1;
            return false;
        }
        true
    }

    pub fn missed_heartbeats(&self) -> u8 {
        self.missed_heartbeats
    }

    pub fn elapsed_ms(&self, now_ms: u64) -> u64 {
        now_ms.saturating_sub(self.start_ms)
    }

    pub fn remaining_ms(&self, now_ms: u64) -> u64 {
        if self.state != WatchdogState::Running {
            return 0;
        }
        let deadline = self.last_feed_ms + self.config.timeout_ms;
        deadline.saturating_sub(now_ms)
    }

    pub fn stop(&mut self) -> Result<(), WatchdogError> {
        if self.state != WatchdogState::Running {
            return Err(WatchdogError::NotRunning);
        }
        self.state = WatchdogState::Stopped;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.state = WatchdogState::Idle;
        self.start_ms = 0;
        self.last_feed_ms = 0;
        self.missed_heartbeats = 0;
    }

    pub fn stats(&self) -> WatchdogStats {
        WatchdogStats {
            state: self.state,
            total_feeds: self.total_feeds,
            total_expirations: self.total_expirations,
            missed_heartbeats: self.missed_heartbeats,
            timeout_ms: self.config.timeout_ms,
            max_missed: self.config.max_missed_heartbeats,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatchdogStats {
    pub state: WatchdogState,
    pub total_feeds: u64,
    pub total_expirations: u64,
    pub missed_heartbeats: u8,
    pub timeout_ms: u64,
    pub max_missed: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_watchdog_is_idle() {
        let w = WatchdogTimer::new(WatchdogConfig::default());
        assert_eq!(w.state(), WatchdogState::Idle);
    }

    #[test]
    fn start_transitions_to_running() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default());
        w.start(1000).unwrap();
        assert_eq!(w.state(), WatchdogState::Running);
    }

    #[test]
    fn start_twice_errors() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default());
        w.start(0).unwrap();
        assert_eq!(w.start(100).unwrap_err(), WatchdogError::AlreadyRunning);
    }

    #[test]
    fn feed_resets_timer() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default().with_timeout(1000));
        w.start(0).unwrap();
        w.feed(500).unwrap();
        assert_eq!(w.remaining_ms(500), 1000);
        assert_eq!(w.stats().total_feeds, 1);
    }

    #[test]
    fn feed_not_running_errors() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default());
        assert_eq!(w.feed(0).unwrap_err(), WatchdogError::NotRunning);
    }

    #[test]
    fn feed_expired() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default().with_timeout(100));
        w.start(0).unwrap();
        let err = w.feed(200).unwrap_err();
        assert!(matches!(err, WatchdogError::FeedExpired { .. }));
        assert_eq!(w.state(), WatchdogState::Expired);
    }

    #[test]
    fn check_detects_timeout() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default().with_timeout(100));
        w.start(0).unwrap();
        assert_eq!(w.check(50), WatchdogState::Running);
        assert_eq!(w.check(200), WatchdogState::Expired);
    }

    #[test]
    fn check_non_running_returns_state() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default());
        assert_eq!(w.check(0), WatchdogState::Idle);
    }

    #[test]
    fn remaining_ms_running() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default().with_timeout(500));
        w.start(100).unwrap();
        assert_eq!(w.remaining_ms(200), 400);
    }

    #[test]
    fn remaining_ms_not_running_is_zero() {
        let w = WatchdogTimer::new(WatchdogConfig::default());
        assert_eq!(w.remaining_ms(0), 0);
    }

    #[test]
    fn elapsed_ms() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default());
        w.start(100).unwrap();
        assert_eq!(w.elapsed_ms(350), 250);
    }

    #[test]
    fn stop_running() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default());
        w.start(0).unwrap();
        w.stop().unwrap();
        assert_eq!(w.state(), WatchdogState::Stopped);
    }

    #[test]
    fn stop_not_running_errors() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default());
        assert_eq!(w.stop().unwrap_err(), WatchdogError::NotRunning);
    }

    #[test]
    fn reset_returns_to_idle() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default());
        w.start(0).unwrap();
        w.stop().unwrap();
        w.reset();
        assert_eq!(w.state(), WatchdogState::Idle);
    }

    #[test]
    fn missed_heartbeats_triggers_expiry() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default().with_max_missed(4));
        w.start(0).unwrap();
        assert!(w.record_missed_heartbeat());
        assert_eq!(w.missed_heartbeats(), 1);
        assert!(w.record_missed_heartbeat());
        assert!(w.record_missed_heartbeat());
        assert!(!w.record_missed_heartbeat());
        assert_eq!(w.state(), WatchdogState::Expired);
        assert_eq!(w.stats().total_expirations, 1);
    }

    #[test]
    fn record_heartbeat_resets_missed() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default().with_max_missed(5));
        w.start(0).unwrap();
        w.record_missed_heartbeat();
        w.record_missed_heartbeat();
        assert_eq!(w.missed_heartbeats(), 2);
        w.record_heartbeat();
        assert_eq!(w.missed_heartbeats(), 0);
    }

    #[test]
    fn stats_reflect_state() {
        let mut w = WatchdogTimer::new(WatchdogConfig::default().with_timeout(200));
        w.start(0).unwrap();
        w.feed(50).unwrap();
        w.feed(100).unwrap();
        let stats = w.stats();
        assert_eq!(stats.total_feeds, 2);
        assert_eq!(stats.state, WatchdogState::Running);
        assert_eq!(stats.timeout_ms, 200);
    }

    #[test]
    fn config_builder() {
        let c = WatchdogConfig::default()
            .with_timeout(1000)
            .with_heartbeat_interval(50)
            .with_max_missed(5);
        assert_eq!(c.timeout_ms, 1000);
        assert_eq!(c.heartbeat_interval_ms, 50);
        assert_eq!(c.max_missed_heartbeats, 5);
    }

    #[test]
    fn error_display() {
        assert!(WatchdogError::AlreadyRunning.to_string().contains("running"));
        assert!(WatchdogError::NotRunning.to_string().contains("not"));
        let e = WatchdogError::FeedExpired { deadline_ms: 100, now_ms: 200 };
        assert!(e.to_string().contains("expired"));
    }
}

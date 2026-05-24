use std::collections::BTreeMap;

pub type TimerId = u64;
pub type TimerCallback = fn(TimerId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Pending,
    Fired,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct TimerEntry {
    pub id: TimerId,
    pub fire_at: u64,
    pub callback: TimerCallback,
    pub state: TimerState,
}

impl TimerEntry {
    pub fn new(id: TimerId, fire_at: u64, callback: TimerCallback) -> Self {
        Self {
            id,
            fire_at,
            callback,
            state: TimerState::Pending,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimerWheel {
    entries: BTreeMap<TimerId, TimerEntry>,
    current_tick: u64,
    next_id: TimerId,
    granularity: u64,
    total_fired: u64,
    total_cancelled: u64,
}

impl TimerWheel {
    pub fn new(granularity: u64) -> Self {
        Self {
            entries: BTreeMap::new(),
            current_tick: 0,
            next_id: 1,
            granularity,
            total_fired: 0,
            total_cancelled: 0,
        }
    }

    pub fn schedule(&mut self, delay_ticks: u64, callback: TimerCallback) -> TimerId {
        let id = self.next_id;
        self.next_id += 1;
        let fire_at = self.current_tick + delay_ticks;
        self.entries.insert(id, TimerEntry::new(id, fire_at, callback));
        id
    }

    pub fn schedule_abs(&mut self, fire_at: u64, callback: TimerCallback) -> TimerId {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.insert(id, TimerEntry::new(id, fire_at, callback));
        id
    }

    pub fn cancel(&mut self, id: TimerId) -> bool {
        if let Some(entry) = self.entries.get_mut(&id) {
            if entry.state == TimerState::Pending {
                entry.state = TimerState::Cancelled;
                self.total_cancelled += 1;
                return true;
            }
        }
        false
    }

    pub fn tick(&mut self) -> Vec<TimerId> {
        self.current_tick += self.granularity;
        self.advance_to(self.current_tick)
    }

    pub fn advance_to(&mut self, target: u64) -> Vec<TimerId> {
        self.current_tick = target;
        let mut fired = Vec::new();
        for entry in self.entries.values_mut() {
            if entry.state == TimerState::Pending && entry.fire_at <= self.current_tick {
                entry.state = TimerState::Fired;
                fired.push(entry.id);
                self.total_fired += 1;
            }
        }
        for &id in &fired {
            if let Some(entry) = self.entries.get(&id) {
                (entry.callback)(id);
            }
        }
        fired
    }

    pub fn get(&self, id: TimerId) -> Option<&TimerEntry> {
        self.entries.get(&id)
    }

    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    pub fn pending_count(&self) -> usize {
        self.entries.values().filter(|e| e.state == TimerState::Pending).count()
    }

    pub fn total_fired(&self) -> u64 {
        self.total_fired
    }

    pub fn total_cancelled(&self) -> u64 {
        self.total_cancelled
    }

    pub fn total_scheduled(&self) -> u64 {
        self.next_id - 1
    }

    pub fn expire_rate(&self) -> f64 {
        let total = self.total_fired + self.total_cancelled;
        if total == 0 { 0.0 } else { self.total_fired as f64 / total as f64 }
    }

    pub fn prune(&mut self) -> usize {
        let before = self.entries.len();
        self.entries.retain(|_, e| e.state == TimerState::Pending);
        before - self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

static mut LAST_FIRED: Option<TimerId> = None;

fn test_cb(id: TimerId) {
    unsafe { LAST_FIRED = Some(id); }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        unsafe { LAST_FIRED = None; }
    }

    #[test]
    fn schedule_and_fire() {
        setup();
        let mut tw = TimerWheel::new(1);
        let id = tw.schedule(5, test_cb);
        assert_eq!(tw.pending_count(), 1);
        tw.advance_to(5);
        assert_eq!(tw.total_fired(), 1);
        assert_eq!(unsafe { LAST_FIRED }, Some(id));
        assert_eq!(tw.get(id).unwrap().state, TimerState::Fired);
    }

    #[test]
    fn schedule_abs() {
        setup();
        let mut tw = TimerWheel::new(1);
        let id = tw.schedule_abs(100, test_cb);
        tw.advance_to(99);
        assert_eq!(tw.total_fired(), 0);
        tw.advance_to(100);
        assert_eq!(tw.total_fired(), 1);
    }

    #[test]
    fn cancel_prevents_fire() {
        let mut tw = TimerWheel::new(1);
        let id = tw.schedule(10, test_cb);
        assert!(tw.cancel(id));
        tw.advance_to(20);
        assert_eq!(tw.total_fired(), 0);
        assert_eq!(tw.total_cancelled(), 1);
    }

    #[test]
    fn cancel_already_fired() {
        setup();
        let mut tw = TimerWheel::new(1);
        let id = tw.schedule(1, test_cb);
        tw.advance_to(1);
        assert!(!tw.cancel(id));
    }

    #[test]
    fn cancel_missing() {
        let mut tw = TimerWheel::new(1);
        assert!(!tw.cancel(999));
    }

    #[test]
    fn tick_advances_granularity() {
        setup();
        let mut tw = TimerWheel::new(10);
        tw.schedule(10, test_cb);
        tw.tick();
        assert_eq!(tw.current_tick(), 10);
        assert_eq!(tw.total_fired(), 1);
    }

    #[test]
    fn multiple_timers() {
        setup();
        let mut tw = TimerWheel::new(1);
        tw.schedule(5, test_cb);
        tw.schedule(10, test_cb);
        tw.schedule(15, test_cb);
        let fired = tw.advance_to(10);
        assert_eq!(fired.len(), 2);
        assert_eq!(tw.pending_count(), 1);
    }

    #[test]
    fn expire_rate() {
        let mut tw = TimerWheel::new(1);
        tw.schedule(1, test_cb);
        tw.schedule(2, test_cb);
        let id = tw.schedule(3, test_cb);
        tw.cancel(id);
        tw.advance_to(10);
        assert!((tw.expire_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn prune() {
        setup();
        let mut tw = TimerWheel::new(1);
        tw.schedule(1, test_cb);
        tw.advance_to(1);
        assert_eq!(tw.prune(), 1);
        assert_eq!(tw.entries.len(), 0);
    }

    #[test]
    fn total_scheduled() {
        let mut tw = TimerWheel::new(1);
        tw.schedule(1, test_cb);
        tw.schedule(2, test_cb);
        assert_eq!(tw.total_scheduled(), 2);
    }

    #[test]
    fn clear() {
        let mut tw = TimerWheel::new(1);
        tw.schedule(1, test_cb);
        tw.clear();
        assert_eq!(tw.pending_count(), 0);
    }
}

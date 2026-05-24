use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerId(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub enum TimerError {
    Duplicate { id: u64 },
    NotFound { id: u64 },
    Expired { id: u64 },
}

impl std::fmt::Display for TimerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimerError::Duplicate { id } => write!(f, "timer {id} duplicate"),
            TimerError::NotFound { id } => write!(f, "timer {id} not found"),
            TimerError::Expired { id } => write!(f, "timer {id} expired"),
        }
    }
}

impl std::error::Error for TimerError {}

#[derive(Debug, Clone)]
struct TimerEntry {
    id: TimerId,
    deadline: u64,
    period: Option<u64>,
    cancelled: bool,
}

#[derive(Debug, Clone)]
pub struct ExpiredTimer {
    pub id: TimerId,
    pub fired_at: u64,
    pub rescheduled: bool,
}

#[derive(Debug, Clone)]
pub struct TimerWheel {
    entries: BTreeMap<u64, TimerEntry>,
    next_id: u64,
    total_fired: u64,
    total_cancelled: u64,
}

impl TimerWheel {
    pub fn new() -> Self {
        Self { entries: BTreeMap::new(), next_id: 1, total_fired: 0, total_cancelled: 0 }
    }

    pub fn schedule(&mut self, deadline: u64, period: Option<u64>) -> TimerId {
        let id = TimerId(self.next_id);
        self.next_id += 1;
        self.entries.insert(id.0, TimerEntry { id, deadline, period, cancelled: false });
        id
    }

    pub fn cancel(&mut self, id: TimerId) -> Result<(), TimerError> {
        let entry = self.entries.get_mut(&id.0).ok_or(TimerError::NotFound { id: id.0 })?;
        if entry.cancelled { return Err(TimerError::Expired { id: id.0 }); }
        entry.cancelled = true;
        self.total_cancelled += 1;
        Ok(())
    }

    pub fn advance(&mut self, now: u64) -> Vec<ExpiredTimer> {
        let mut fired = Vec::new();
        let to_fire: Vec<(u64, TimerId, Option<u64>)> = self.entries.iter()
            .filter(|(_, e)| !e.cancelled && e.deadline <= now)
            .map(|(k, e)| (*k, e.id, e.period))
            .collect();
        for (kid, id, period) in to_fire {
            self.entries.remove(&kid);
            self.total_fired += 1;
            let rescheduled = period.is_some();
            if let Some(p) = period {
                let new_deadline = now + p;
                self.entries.insert(id.0, TimerEntry { id, deadline: new_deadline, period: Some(p), cancelled: false });
            }
            fired.push(ExpiredTimer { id, fired_at: now, rescheduled });
        }
        fired
    }

    pub fn reschedule(&mut self, id: TimerId, new_deadline: u64) -> Result<(), TimerError> {
        let entry = self.entries.get_mut(&id.0).ok_or(TimerError::NotFound { id: id.0 })?;
        if entry.cancelled { return Err(TimerError::Expired { id: id.0 }); }
        entry.deadline = new_deadline;
        Ok(())
    }

    pub fn active_count(&self) -> usize {
        self.entries.values().filter(|e| !e.cancelled).count()
    }

    pub fn total_fired(&self) -> u64 { self.total_fired }
    pub fn total_cancelled(&self) -> u64 { self.total_cancelled }

    pub fn next_deadline(&self) -> Option<u64> {
        self.entries.values().filter(|e| !e.cancelled).map(|e| e.deadline).min()
    }

    pub fn has_timer(&self, id: TimerId) -> bool {
        self.entries.contains_key(&id.0)
    }
}

impl Default for TimerWheel {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_wheel() {
        let tw = TimerWheel::new();
        assert_eq!(tw.active_count(), 0);
        assert_eq!(tw.next_deadline(), None);
    }

    #[test]
    fn schedule_one() {
        let mut tw = TimerWheel::new();
        let id = tw.schedule(100, None);
        assert!(tw.has_timer(id));
        assert_eq!(tw.active_count(), 1);
        assert_eq!(tw.next_deadline(), Some(100));
    }

    #[test]
    fn fire_one() {
        let mut tw = TimerWheel::new();
        let id = tw.schedule(50, None);
        let expired = tw.advance(60);
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].id, id);
        assert!(!expired[0].rescheduled);
        assert_eq!(tw.active_count(), 0);
        assert_eq!(tw.total_fired(), 1);
    }

    #[test]
    fn not_yet_expired() {
        let mut tw = TimerWheel::new();
        tw.schedule(100, None);
        let expired = tw.advance(50);
        assert!(expired.is_empty());
        assert_eq!(tw.active_count(), 1);
    }

    #[test]
    fn cancel() {
        let mut tw = TimerWheel::new();
        let id = tw.schedule(100, None);
        tw.cancel(id).unwrap();
        assert_eq!(tw.active_count(), 0);
        let expired = tw.advance(200);
        assert!(expired.is_empty());
        assert_eq!(tw.total_cancelled(), 1);
    }

    #[test]
    fn cancel_twice() {
        let mut tw = TimerWheel::new();
        let id = tw.schedule(100, None);
        tw.cancel(id).unwrap();
        let err = tw.cancel(id).unwrap_err();
        assert!(matches!(err, TimerError::Expired { .. }));
    }

    #[test]
    fn periodic() {
        let mut tw = TimerWheel::new();
        let id = tw.schedule(10, Some(10));
        let e1 = tw.advance(10);
        assert_eq!(e1.len(), 1);
        assert!(e1[0].rescheduled);
        assert_eq!(tw.active_count(), 1);
        let e2 = tw.advance(20);
        assert_eq!(e2.len(), 1);
        assert_eq!(tw.total_fired(), 2);
    }

    #[test]
    fn multiple_timers() {
        let mut tw = TimerWheel::new();
        tw.schedule(30, None);
        tw.schedule(10, None);
        tw.schedule(20, None);
        let expired = tw.advance(25);
        assert_eq!(expired.len(), 2);
    }

    #[test]
    fn reschedule() {
        let mut tw = TimerWheel::new();
        let id = tw.schedule(50, None);
        tw.reschedule(id, 200).unwrap();
        let e1 = tw.advance(100);
        assert!(e1.is_empty());
        let e2 = tw.advance(200);
        assert_eq!(e2.len(), 1);
    }

    #[test]
    fn not_found() {
        let mut tw = TimerWheel::new();
        let err = tw.cancel(TimerId(99)).unwrap_err();
        assert!(matches!(err, TimerError::NotFound { .. }));
    }

    #[test]
    fn error_display() {
        assert!(TimerError::NotFound { id: 5 }.to_string().contains("5"));
    }

    #[test]
    fn advance_empty() {
        let mut tw = TimerWheel::new();
        assert!(tw.advance(1000).is_empty());
    }
}

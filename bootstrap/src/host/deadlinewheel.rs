use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum DwError {
    TimerExists { id: u64 },
    TimerNotFound { id: u64 },
    AlreadyCancelled { id: u64 },
    AlreadyFired { id: u64 },
}

impl std::fmt::Display for DwError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DwError::TimerExists { id } => write!(f, "timer {id} exists"),
            DwError::TimerNotFound { id } => write!(f, "timer {id} not found"),
            DwError::AlreadyCancelled { id } => write!(f, "timer {id} cancelled"),
            DwError::AlreadyFired { id } => write!(f, "timer {id} fired"),
        }
    }
}

impl std::error::Error for DwError {}

#[derive(Debug, Clone, PartialEq)]
pub enum TimerState { Pending, Fired, Cancelled }

struct Timer {
    id: u64,
    deadline_tick: u64,
    state: TimerState,
    label: String,
}

pub struct DeadlineWheel {
    slots: Vec<Vec<u64>>,
    ticks_per_wheel: u64,
    current_tick: u64,
    timers: BTreeMap<u64, Timer>,
    next_id: u64,
    total_scheduled: u64,
    total_fired: u64,
    total_cancelled: u64,
}

impl DeadlineWheel {
    pub fn new(ticks_per_wheel: u64) -> Self {
        Self {
            slots: vec![Vec::new(); ticks_per_wheel as usize],
            ticks_per_wheel,
            current_tick: 0,
            timers: BTreeMap::new(),
            next_id: 1,
            total_scheduled: 0,
            total_fired: 0,
            total_cancelled: 0,
        }
    }

    pub fn schedule(&mut self, deadline_tick: u64, label: &str) -> Result<u64, DwError> {
        let id = self.next_id;
        self.next_id += 1;
        let slot = (deadline_tick % self.ticks_per_wheel) as usize;
        self.slots[slot].push(id);
        self.timers.insert(id, Timer { id, deadline_tick, state: TimerState::Pending, label: label.to_string() });
        self.total_scheduled += 1;
        Ok(id)
    }

    pub fn cancel(&mut self, id: u64) -> Result<(), DwError> {
        let t = self.timers.get_mut(&id).ok_or(DwError::TimerNotFound { id })?;
        match t.state {
            TimerState::Pending => { t.state = TimerState::Cancelled; self.total_cancelled += 1; Ok(()) }
            TimerState::Cancelled => Err(DwError::AlreadyCancelled { id }),
            TimerState::Fired => Err(DwError::AlreadyFired { id }),
        }
    }

    pub fn tick(&mut self) -> Vec<u64> {
        self.current_tick += 1;
        let slot = (self.current_tick % self.ticks_per_wheel) as usize;
        let timer_ids: Vec<u64> = self.slots[slot].drain(..).collect();
        let mut fired = Vec::new();
        for id in timer_ids {
            if let Some(t) = self.timers.get_mut(&id) {
                if t.state == TimerState::Pending && t.deadline_tick <= self.current_tick {
                    t.state = TimerState::Fired;
                    self.total_fired += 1;
                    fired.push(id);
                } else if t.state == TimerState::Pending {
                    let new_slot = (t.deadline_tick % self.ticks_per_wheel) as usize;
                    self.slots[new_slot].push(id);
                }
            }
        }
        fired
    }

    pub fn advance(&mut self, ticks: u64) -> Vec<u64> {
        let mut all_fired = Vec::new();
        for _ in 0..ticks {
            all_fired.extend(self.tick());
        }
        all_fired
    }

    pub fn timer_state(&self, id: u64) -> Option<&TimerState> { self.timers.get(&id).map(|t| &t.state) }
    pub fn timer_label(&self, id: u64) -> Option<&str> { self.timers.get(&id).map(|t| t.label.as_str()) }
    pub fn timer_deadline(&self, id: u64) -> Option<u64> { self.timers.get(&id).map(|t| t.deadline_tick) }
    pub fn current_tick(&self) -> u64 { self.current_tick }
    pub fn pending_count(&self) -> usize { self.timers.values().filter(|t| t.state == TimerState::Pending).count() }
    pub fn total_scheduled(&self) -> u64 { self.total_scheduled }
    pub fn total_fired(&self) -> u64 { self.total_fired }
    pub fn total_cancelled(&self) -> u64 { self.total_cancelled }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_wheel() { let dw = DeadlineWheel::new(64); assert_eq!(dw.current_tick(), 0); }

    #[test]
    fn schedule_fire() {
        let mut dw = DeadlineWheel::new(64);
        let id = dw.schedule(3, "test").unwrap();
        dw.tick(); dw.tick();
        let fired = dw.tick();
        assert!(fired.contains(&id));
        assert_eq!(dw.timer_state(id), Some(&TimerState::Fired));
    }

    #[test]
    fn cancel_timer() {
        let mut dw = DeadlineWheel::new(64);
        let id = dw.schedule(5, "x").unwrap();
        dw.cancel(id).unwrap();
        assert_eq!(dw.timer_state(id), Some(&TimerState::Cancelled));
        dw.advance(10);
        assert_eq!(dw.timer_state(id), Some(&TimerState::Cancelled));
    }

    #[test]
    fn double_cancel() {
        let mut dw = DeadlineWheel::new(64);
        let id = dw.schedule(5, "x").unwrap();
        dw.cancel(id).unwrap();
        let err = dw.cancel(id).unwrap_err();
        assert!(matches!(err, DwError::AlreadyCancelled { .. }));
    }

    #[test]
    fn not_found() {
        let mut dw = DeadlineWheel::new(64);
        let err = dw.cancel(99).unwrap_err();
        assert!(matches!(err, DwError::TimerNotFound { .. }));
    }

    #[test]
    fn advance() {
        let mut dw = DeadlineWheel::new(64);
        dw.schedule(2, "a").unwrap();
        dw.schedule(4, "b").unwrap();
        let fired = dw.advance(5);
        assert_eq!(fired.len(), 2);
    }

    #[test]
    fn wrap_around() {
        let mut dw = DeadlineWheel::new(8);
        dw.schedule(10, "wrap").unwrap();
        let fired = dw.advance(10);
        assert!(fired.len() >= 1);
    }

    #[test]
    fn label_tracking() {
        let mut dw = DeadlineWheel::new(64);
        dw.schedule(5, "heartbeat").unwrap();
        assert_eq!(dw.timer_label(1), Some("heartbeat"));
    }

    #[test]
    fn pending_count() {
        let mut dw = DeadlineWheel::new(64);
        dw.schedule(5, "a").unwrap();
        dw.schedule(10, "b").unwrap();
        assert_eq!(dw.pending_count(), 2);
    }

    #[test]
    fn stats() {
        let mut dw = DeadlineWheel::new(64);
        dw.schedule(1, "a").unwrap();
        dw.advance(2);
        assert_eq!(dw.total_scheduled(), 1);
        assert_eq!(dw.total_fired(), 1);
    }

    #[test]
    fn error_display() { assert!(DwError::TimerNotFound { id: 3 }.to_string().contains("3")); }
}

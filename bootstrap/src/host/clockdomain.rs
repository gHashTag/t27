use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Deadline {
    pub name: String,
    pub target_tick: u64,
    pub callback: fn(&str),
    pub fired: bool,
}

impl Deadline {
    pub fn new(name: &str, target_tick: u64, callback: fn(&str)) -> Self {
        Self {
            name: name.to_string(),
            target_tick,
            callback,
            fired: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClockDomain {
    name: String,
    freq_hz: u64,
    tick: u64,
    origin_us: u64,
    deadlines: BTreeMap<String, Deadline>,
    total_ticks: u64,
    total_deadlines_fired: u64,
}

impl ClockDomain {
    pub fn new(name: &str, freq_hz: u64) -> Self {
        Self {
            name: name.to_string(),
            freq_hz,
            tick: 0,
            origin_us: 0,
            deadlines: BTreeMap::new(),
            total_ticks: 0,
            total_deadlines_fired: 0,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn freq_hz(&self) -> u64 {
        self.freq_hz
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn period_ns(&self) -> u64 {
        if self.freq_hz == 0 {
            return 0;
        }
        1_000_000_000 / self.freq_hz
    }

    pub fn tick_to_ns(&self, tick: u64) -> u64 {
        tick * self.period_ns()
    }

    pub fn ns_to_tick(&self, ns: u64) -> u64 {
        let period = self.period_ns();
        if period == 0 {
            return 0;
        }
        ns / period
    }

    pub fn advance(&mut self, ticks: u64) {
        self.tick += ticks;
        self.total_ticks += ticks;
        self.check_deadlines();
    }

    pub fn advance_to(&mut self, target: u64) {
        if target > self.tick {
            let delta = target - self.tick;
            self.advance(delta);
        }
    }

    pub fn set_origin(&mut self, us: u64) {
        self.origin_us = us;
    }

    pub fn origin_us(&self) -> u64 {
        self.origin_us
    }

    pub fn elapsed_ns(&self) -> u64 {
        self.tick_to_ns(self.tick)
    }

    pub fn add_deadline(&mut self, dl: Deadline) {
        self.deadlines.insert(dl.name.clone(), dl);
    }

    pub fn remove_deadline(&mut self, name: &str) -> bool {
        self.deadlines.remove(name).is_some()
    }

    fn check_deadlines(&mut self) {
        let current = self.tick;
        let mut to_fire: Vec<Deadline> = Vec::new();
        for dl in self.deadlines.values_mut() {
            if !dl.fired && current >= dl.target_tick {
                dl.fired = true;
                to_fire.push(dl.clone());
            }
        }
        for dl in &to_fire {
            (dl.callback)(&dl.name);
            self.total_deadlines_fired += 1;
        }
    }

    pub fn pending_deadlines(&self) -> Vec<&Deadline> {
        self.deadlines.values().filter(|d| !d.fired).collect()
    }

    pub fn deadline_count(&self) -> usize {
        self.deadlines.len()
    }

    pub fn total_ticks(&self) -> u64 {
        self.total_ticks
    }

    pub fn total_deadlines_fired(&self) -> u64 {
        self.total_deadlines_fired
    }

    pub fn reset(&mut self) {
        self.tick = 0;
        self.deadlines.clear();
        self.total_ticks = 0;
        self.total_deadlines_fired = 0;
    }
}

static mut FIRED_COUNT: u64 = 0;

fn test_callback(_name: &str) {
    unsafe {
        FIRED_COUNT += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        unsafe { FIRED_COUNT = 0; }
    }

    #[test]
    fn new_clock() {
        let c = ClockDomain::new("fclk", 66_000_000);
        assert_eq!(c.freq_hz(), 66_000_000);
        assert_eq!(c.tick(), 0);
    }

    #[test]
    fn period_ns() {
        let c = ClockDomain::new("fclk", 1_000_000);
        assert_eq!(c.period_ns(), 1000);
    }

    #[test]
    fn tick_to_ns() {
        let c = ClockDomain::new("fclk", 1_000_000);
        assert_eq!(c.tick_to_ns(1000), 1_000_000);
    }

    #[test]
    fn ns_to_tick() {
        let c = ClockDomain::new("fclk", 1_000_000);
        assert_eq!(c.ns_to_tick(1_000_000), 1000);
    }

    #[test]
    fn advance() {
        let mut c = ClockDomain::new("fclk", 1_000_000);
        c.advance(100);
        assert_eq!(c.tick(), 100);
        assert_eq!(c.total_ticks(), 100);
    }

    #[test]
    fn advance_to() {
        let mut c = ClockDomain::new("fclk", 1_000_000);
        c.advance(50);
        c.advance_to(200);
        assert_eq!(c.tick(), 200);
    }

    #[test]
    fn advance_to_no_backward() {
        let mut c = ClockDomain::new("fclk", 1_000_000);
        c.advance(200);
        c.advance_to(100);
        assert_eq!(c.tick(), 200);
    }

    #[test]
    fn origin() {
        let mut c = ClockDomain::new("fclk", 1_000_000);
        c.set_origin(42);
        assert_eq!(c.origin_us(), 42);
    }

    #[test]
    fn deadline_fires() {
        setup();
        let mut c = ClockDomain::new("fclk", 1_000_000);
        c.add_deadline(Deadline::new("d1", 100, test_callback));
        c.advance(50);
        assert_eq!(unsafe { FIRED_COUNT }, 0);
        c.advance(60);
        assert_eq!(unsafe { FIRED_COUNT }, 1);
        assert_eq!(c.total_deadlines_fired(), 1);
    }

    #[test]
    fn deadline_pending() {
        let mut c = ClockDomain::new("fclk", 1_000_000);
        c.add_deadline(Deadline::new("d1", 100, test_callback));
        c.add_deadline(Deadline::new("d2", 200, test_callback));
        c.advance(50);
        assert_eq!(c.pending_deadlines().len(), 2);
    }

    #[test]
    fn remove_deadline() {
        let mut c = ClockDomain::new("fclk", 1_000_000);
        c.add_deadline(Deadline::new("d1", 100, test_callback));
        assert!(c.remove_deadline("d1"));
        assert!(!c.remove_deadline("d1"));
    }

    #[test]
    fn elapsed_ns() {
        let mut c = ClockDomain::new("fclk", 1_000_000);
        c.advance(500);
        assert_eq!(c.elapsed_ns(), 500_000);
    }

    #[test]
    fn reset() {
        let mut c = ClockDomain::new("fclk", 1_000_000);
        c.advance(100);
        c.reset();
        assert_eq!(c.tick(), 0);
        assert_eq!(c.total_ticks(), 0);
    }
}

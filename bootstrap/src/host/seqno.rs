use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeqnoError {
    Stale { seq: u64, expected: u64 },
    Duplicate { seq: u64 },
    WindowExceeded { seq: u64, window: u64 },
}

impl std::fmt::Display for SeqnoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeqnoError::Stale { seq, expected } => write!(f, "stale {seq}, expected {expected}"),
            SeqnoError::Duplicate { seq } => write!(f, "duplicate {seq}"),
            SeqnoError::WindowExceeded { seq, window } => write!(f, "{seq} outside window {window}"),
        }
    }
}

impl std::error::Error for SeqnoError {}

#[derive(Debug, Clone)]
pub struct SeqnoStats {
    pub current: u64,
    pub total_accepted: u64,
    pub total_rejected: u64,
    pub total_gaps: u64,
    pub gap_count: usize,
}

#[derive(Debug, Clone)]
pub struct SeqnoTracker {
    next_expected: u64,
    window: u64,
    seen: BTreeSet<u64>,
    gaps: BTreeSet<u64>,
    total_accepted: u64,
    total_rejected: u64,
    total_gaps: u64,
    max_seen: u64,
}

impl SeqnoTracker {
    pub fn new(initial: u64, window: u64) -> Self {
        Self {
            next_expected: initial,
            window: window.max(1),
            seen: BTreeSet::new(),
            gaps: BTreeSet::new(),
            total_accepted: 0,
            total_rejected: 0,
            total_gaps: 0,
            max_seen: initial,
        }
    }

    pub fn next_expected(&self) -> u64 {
        self.next_expected
    }

    pub fn window(&self) -> u64 {
        self.window
    }

    pub fn record(&mut self, seq: u64) -> Result<(), SeqnoError> {
        if seq < self.next_expected {
            if self.seen.contains(&seq) {
                self.total_rejected += 1;
                return Err(SeqnoError::Duplicate { seq });
            }
            self.total_rejected += 1;
            return Err(SeqnoError::Stale { seq, expected: self.next_expected });
        }
        if seq > self.next_expected + self.window {
            self.total_rejected += 1;
            return Err(SeqnoError::WindowExceeded { seq, window: self.window });
        }
        if self.seen.contains(&seq) {
            self.total_rejected += 1;
            return Err(SeqnoError::Duplicate { seq });
        }
        if seq > self.next_expected {
            for gap in self.next_expected..seq {
                self.gaps.insert(gap);
                self.total_gaps += 1;
            }
        }
        self.seen.insert(seq);
        if seq > self.max_seen { self.max_seen = seq; }
        self.total_accepted += 1;
        while self.seen.contains(&self.next_expected) {
            self.gaps.remove(&self.next_expected);
            self.next_expected += 1;
        }
        Ok(())
    }

    pub fn is_gap(&self, seq: u64) -> bool {
        self.gaps.contains(&seq)
    }

    pub fn gap_list(&self) -> Vec<u64> {
        self.gaps.iter().copied().collect()
    }

    pub fn gap_count(&self) -> usize {
        self.gaps.len()
    }

    pub fn has_seen(&self, seq: u64) -> bool {
        self.seen.contains(&seq)
    }

    pub fn max_seen(&self) -> u64 {
        self.max_seen
    }

    pub fn total_accepted(&self) -> u64 {
        self.total_accepted
    }

    pub fn total_rejected(&self) -> u64 {
        self.total_rejected
    }

    pub fn total_gaps(&self) -> u64 {
        self.total_gaps
    }

    pub fn stats(&self) -> SeqnoStats {
        SeqnoStats {
            current: self.next_expected,
            total_accepted: self.total_accepted,
            total_rejected: self.total_rejected,
            total_gaps: self.total_gaps,
            gap_count: self.gaps.len(),
        }
    }

    pub fn reset(&mut self, initial: u64) {
        self.next_expected = initial;
        self.seen.clear();
        self.gaps.clear();
        self.total_accepted = 0;
        self.total_rejected = 0;
        self.total_gaps = 0;
        self.max_seen = initial;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tracker() {
        let st = SeqnoTracker::new(1, 100);
        assert_eq!(st.next_expected(), 1);
        assert_eq!(st.window(), 100);
    }

    #[test]
    fn sequential() {
        let mut st = SeqnoTracker::new(1, 100);
        st.record(1).unwrap();
        st.record(2).unwrap();
        st.record(3).unwrap();
        assert_eq!(st.next_expected(), 4);
        assert_eq!(st.total_accepted(), 3);
    }

    #[test]
    fn out_of_order_fill_gap() {
        let mut st = SeqnoTracker::new(1, 100);
        st.record(1).unwrap();
        st.record(3).unwrap();
        assert_eq!(st.gap_count(), 1);
        assert!(st.is_gap(2));
        st.record(2).unwrap();
        assert_eq!(st.next_expected(), 4);
        assert_eq!(st.gap_count(), 0);
    }

    #[test]
    fn duplicate() {
        let mut st = SeqnoTracker::new(1, 100);
        st.record(1).unwrap();
        let err = st.record(1).unwrap_err();
        assert!(matches!(err, SeqnoError::Duplicate { seq: 1 }));
    }

    #[test]
    fn stale() {
        let mut st = SeqnoTracker::new(1, 100);
        st.record(1).unwrap();
        st.record(2).unwrap();
        st.record(3).unwrap();
        let err = st.record(1).unwrap_err();
        assert!(matches!(err, SeqnoError::Stale { seq: 1, .. } | SeqnoError::Duplicate { seq: 1 }));
    }

    #[test]
    fn window_exceeded() {
        let mut st = SeqnoTracker::new(1, 10);
        let err = st.record(20).unwrap_err();
        assert!(matches!(err, SeqnoError::WindowExceeded { .. }));
    }

    #[test]
    fn gap_tracking() {
        let mut st = SeqnoTracker::new(1, 100);
        st.record(1).unwrap();
        st.record(5).unwrap();
        assert_eq!(st.gap_list(), vec![2, 3, 4]);
        assert_eq!(st.total_gaps(), 3);
    }

    #[test]
    fn max_seen() {
        let mut st = SeqnoTracker::new(1, 100);
        st.record(1).unwrap();
        st.record(10).unwrap();
        assert_eq!(st.max_seen(), 10);
    }

    #[test]
    fn has_seen() {
        let mut st = SeqnoTracker::new(1, 100);
        st.record(5).unwrap();
        assert!(st.has_seen(5));
        assert!(!st.has_seen(6));
    }

    #[test]
    fn stats_snapshot() {
        let mut st = SeqnoTracker::new(1, 100);
        st.record(1).unwrap();
        st.record(3).unwrap();
        let s = st.stats();
        assert_eq!(s.total_accepted, 2);
        assert_eq!(s.gap_count, 1);
    }

    #[test]
    fn reset() {
        let mut st = SeqnoTracker::new(1, 100);
        st.record(1).unwrap();
        st.record(2).unwrap();
        st.reset(10);
        assert_eq!(st.next_expected(), 10);
        assert_eq!(st.total_accepted(), 0);
    }

    #[test]
    fn error_display() {
        assert!(SeqnoError::Duplicate { seq: 5 }.to_string().contains("5"));
    }
}

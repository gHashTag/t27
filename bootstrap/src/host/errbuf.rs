#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Warn,
    Error,
    Fatal,
}

#[derive(Clone)]
pub struct ErrEntry {
    pub code: u32,
    pub severity: Severity,
    pub msg: Vec<u8>,
}

pub struct ErrBuf {
    entries: Vec<ErrEntry>,
    cap: usize,
    head: usize,
    len: usize,
    counts: [u64; 3],
    total_pushed: u64,
    total_drained: u64,
}

impl ErrBuf {
    pub fn new(cap: usize) -> Self {
        Self { entries: (0..cap).map(|_| ErrEntry { code: 0, severity: Severity::Warn, msg: Vec::new() }).collect(), cap, head: 0, len: 0, counts: [0; 3], total_pushed: 0, total_drained: 0 }
    }

    fn sev_idx(s: Severity) -> usize { match s { Severity::Warn => 0, Severity::Error => 1, Severity::Fatal => 2 } }

    pub fn push(&mut self, code: u32, severity: Severity, msg: Vec<u8>) {
        self.total_pushed += 1;
        self.counts[Self::sev_idx(severity)] += 1;
        if self.len < self.cap {
            let idx = (self.head + self.len) % self.cap;
            self.entries[idx] = ErrEntry { code, severity, msg };
            self.len += 1;
        } else {
            self.entries[self.head] = ErrEntry { code, severity, msg };
            self.head = (self.head + 1) % self.cap;
        }
    }

    pub fn get(&self, idx: usize) -> Option<&ErrEntry> {
        if idx >= self.len { return None; }
        Some(&self.entries[(self.head + idx) % self.cap])
    }

    pub fn drain(&mut self) -> Vec<ErrEntry> {
        self.total_drained += 1;
        let mut out = Vec::with_capacity(self.len);
        for i in 0..self.len {
            let idx = (self.head + i) % self.cap;
            out.push(self.entries[idx].clone());
        }
        self.head = 0;
        self.len = 0;
        out
    }

    pub fn drain_severity(&mut self, severity: Severity) -> Vec<ErrEntry> {
        let si = Self::sev_idx(severity);
        let mut out = Vec::new();
        let mut kept = Vec::new();
        for i in 0..self.len {
            let idx = (self.head + i) % self.cap;
            let e = self.entries[idx].clone();
            if Self::sev_idx(e.severity) == si { out.push(e); } else { kept.push(e); }
        }
        self.head = 0;
        self.len = 0;
        for e in kept { self.push(e.code, e.severity, e.msg); }
        out
    }

    pub fn histogram(&self) -> [u64; 3] { self.counts }
    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn cap(&self) -> usize { self.cap }
    pub fn total_pushed(&self) -> u64 { self.total_pushed }
    pub fn total_drained(&self) -> u64 { self.total_drained }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_get() {
        let mut eb = ErrBuf::new(8);
        eb.push(1, Severity::Warn, b"w1".to_vec());
        eb.push(2, Severity::Error, b"e1".to_vec());
        assert_eq!(eb.get(0).unwrap().code, 1);
        assert_eq!(eb.get(1).unwrap().severity, Severity::Error);
    }

    #[test]
    fn overflow_wraps() {
        let mut eb = ErrBuf::new(3);
        for i in 0..5u32 { eb.push(i, Severity::Warn, vec![]); }
        assert_eq!(eb.len(), 3);
        assert_eq!(eb.get(0).unwrap().code, 2);
    }

    #[test]
    fn drain() {
        let mut eb = ErrBuf::new(8);
        eb.push(1, Severity::Warn, vec![]);
        eb.push(2, Severity::Error, vec![]);
        let d = eb.drain();
        assert_eq!(d.len(), 2);
        assert!(eb.is_empty());
    }

    #[test]
    fn drain_severity() {
        let mut eb = ErrBuf::new(8);
        eb.push(1, Severity::Warn, vec![]);
        eb.push(2, Severity::Error, vec![]);
        eb.push(3, Severity::Warn, vec![]);
        let errs = eb.drain_severity(Severity::Error);
        assert_eq!(errs.len(), 1);
        assert_eq!(eb.len(), 2);
    }

    #[test]
    fn histogram() {
        let mut eb = ErrBuf::new(8);
        eb.push(1, Severity::Warn, vec![]);
        eb.push(2, Severity::Error, vec![]);
        eb.push(3, Severity::Fatal, vec![]);
        let h = eb.histogram();
        assert_eq!(h[0], 1);
        assert_eq!(h[1], 1);
        assert_eq!(h[2], 1);
    }

    #[test]
    fn get_out_of_range() { assert!(ErrBuf::new(4).get(0).is_none()); }

    #[test]
    fn stats() {
        let mut eb = ErrBuf::new(8);
        eb.push(1, Severity::Warn, vec![]);
        eb.drain();
        assert_eq!(eb.total_pushed(), 1);
        assert_eq!(eb.total_drained(), 1);
    }

    #[test]
    fn severity_ord() { assert!(Severity::Fatal > Severity::Error); }
}

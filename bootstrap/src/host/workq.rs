pub struct WorkQ {
    buf: Vec<u64>,
    top: usize,
    bottom: usize,
    total_push: u64,
    total_pop: u64,
    total_steal: u64,
}

impl WorkQ {
    pub fn new(cap: usize) -> Self {
        Self { buf: vec![0; cap.max(1)], top: 0, bottom: 0,
               total_push: 0, total_pop: 0, total_steal: 0 }
    }

    pub fn push(&mut self, val: u64) -> bool {
        if self.len() >= self.buf.len() { return false; }
        self.total_push += 1;
        let idx = self.bottom % self.buf.len();
        self.buf[idx] = val;
        self.bottom += 1;
        true
    }

    pub fn pop(&mut self) -> Option<u64> {
        if self.bottom <= self.top { return None; }
        self.total_pop += 1;
        self.bottom -= 1;
        let idx = self.bottom % self.buf.len();
        Some(self.buf[idx])
    }

    pub fn steal(&mut self) -> Option<u64> {
        if self.bottom <= self.top { return None; }
        self.total_steal += 1;
        let idx = self.top % self.buf.len();
        let val = self.buf[idx];
        self.top += 1;
        Some(val)
    }

    pub fn len(&self) -> usize { self.bottom.saturating_sub(self.top) }
    pub fn is_empty(&self) -> bool { self.bottom <= self.top }
    pub fn cap(&self) -> usize { self.buf.len() }
    pub fn total_push(&self) -> u64 { self.total_push }
    pub fn total_pop(&self) -> u64 { self.total_pop }
    pub fn total_steal(&self) -> u64 { self.total_steal }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut q = WorkQ::new(4);
        q.push(1); q.push(2); q.push(3);
        assert_eq!(q.pop(), Some(3));
        assert_eq!(q.pop(), Some(2));
    }

    #[test]
    fn steal() {
        let mut q = WorkQ::new(4);
        q.push(10); q.push(20);
        assert_eq!(q.steal(), Some(10));
        assert_eq!(q.steal(), Some(20));
    }

    #[test]
    fn mixed() {
        let mut q = WorkQ::new(8);
        q.push(1); q.push(2); q.push(3);
        assert_eq!(q.steal(), Some(1));
        assert_eq!(q.pop(), Some(3));
        assert_eq!(q.steal(), Some(2));
        assert!(q.is_empty());
    }

    #[test]
    fn full() {
        let mut q = WorkQ::new(2);
        assert!(q.push(1)); assert!(q.push(2));
        assert!(!q.push(3));
    }

    #[test]
    fn empty_ops() {
        let mut q = WorkQ::new(4);
        assert!(q.pop().is_none());
        assert!(q.steal().is_none());
    }

    #[test]
    fn stats() {
        let mut q = WorkQ::new(4);
        q.push(1); q.push(2); q.pop(); q.steal();
        assert_eq!(q.total_push(), 2);
        assert_eq!(q.total_pop(), 1);
        assert_eq!(q.total_steal(), 1);
    }
}

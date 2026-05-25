pub struct CircQ {
    buf: Vec<u64>,
    head: usize,
    len: usize,
    total_enq: u64,
    total_deq: u64,
}

impl CircQ {
    pub fn new(cap: usize) -> Self {
        Self { buf: vec![0; cap.max(1)], head: 0, len: 0, total_enq: 0, total_deq: 0 }
    }

    pub fn enqueue(&mut self, val: u64) -> bool {
        if self.len >= self.buf.len() { return false; }
        self.total_enq += 1;
        let idx = (self.head + self.len) % self.buf.len();
        self.buf[idx] = val;
        self.len += 1;
        true
    }

    pub fn dequeue(&mut self) -> Option<u64> {
        if self.len == 0 { return None; }
        self.total_deq += 1;
        let val = self.buf[self.head];
        self.head = (self.head + 1) % self.buf.len();
        self.len -= 1;
        Some(val)
    }

    pub fn front(&self) -> Option<u64> { if self.len == 0 { None } else { Some(self.buf[self.head]) } }
    pub fn back(&self) -> Option<u64> { if self.len == 0 { None } else { let i = (self.head + self.len - 1) % self.buf.len(); Some(self.buf[i]) } }
    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn cap(&self) -> usize { self.buf.len() }
    pub fn total_enq(&self) -> u64 { self.total_enq }
    pub fn total_deq(&self) -> u64 { self.total_deq }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enq_deq() {
        let mut q = CircQ::new(3);
        q.enqueue(1); q.enqueue(2);
        assert_eq!(q.dequeue(), Some(1));
        assert_eq!(q.dequeue(), Some(2));
    }

    #[test]
    fn wrap() {
        let mut q = CircQ::new(3);
        q.enqueue(1); q.enqueue(2); q.dequeue();
        q.enqueue(3); q.enqueue(4);
        assert_eq!(q.len(), 3);
    }

    #[test]
    fn full() {
        let mut q = CircQ::new(2);
        assert!(q.enqueue(1)); assert!(q.enqueue(2));
        assert!(!q.enqueue(3));
    }

    #[test]
    fn front_back() {
        let mut q = CircQ::new(4);
        q.enqueue(10); q.enqueue(20);
        assert_eq!(q.front(), Some(10));
        assert_eq!(q.back(), Some(20));
    }

    #[test]
    fn empty_deq() { assert!(CircQ::new(4).dequeue().is_none()); }

    #[test]
    fn stats() {
        let mut q = CircQ::new(4);
        q.enqueue(1); q.enqueue(2); q.dequeue();
        assert_eq!(q.total_enq(), 2);
        assert_eq!(q.total_deq(), 1);
    }
}

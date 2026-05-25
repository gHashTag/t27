pub struct DequeS {
    buf: Vec<Option<u64>>,
    head: usize,
    tail: usize,
    len: usize,
    total_push_front: u64,
    total_push_back: u64,
    total_pop_front: u64,
    total_pop_back: u64,
}

impl DequeS {
    pub fn new(cap: usize) -> Self {
        let cap = cap.max(1);
        Self { buf: (0..cap).map(|_| None).collect(), head: 0, tail: 0, len: 0,
               total_push_front: 0, total_push_back: 0, total_pop_front: 0, total_pop_back: 0 }
    }

    pub fn push_back(&mut self, val: u64) -> bool {
        if self.len == self.buf.len() { return false; }
        self.total_push_back += 1;
        self.buf[self.tail] = Some(val);
        self.tail = (self.tail + 1) % self.buf.len();
        self.len += 1;
        true
    }

    pub fn push_front(&mut self, val: u64) -> bool {
        if self.len == self.buf.len() { return false; }
        self.total_push_front += 1;
        self.head = (self.head + self.buf.len() - 1) % self.buf.len();
        self.buf[self.head] = Some(val);
        self.len += 1;
        true
    }

    pub fn pop_front(&mut self) -> Option<u64> {
        if self.len == 0 { return None; }
        self.total_pop_front += 1;
        let val = self.buf[self.head].take();
        self.head = (self.head + 1) % self.buf.len();
        self.len -= 1;
        val
    }

    pub fn pop_back(&mut self) -> Option<u64> {
        if self.len == 0 { return None; }
        self.total_pop_back += 1;
        self.tail = (self.tail + self.buf.len() - 1) % self.buf.len();
        let val = self.buf[self.tail].take();
        self.len -= 1;
        val
    }

    pub fn front(&self) -> Option<u64> { if self.len == 0 { None } else { self.buf[self.head] } }
    pub fn back(&self) -> Option<u64> { if self.len == 0 { None } else { self.buf[(self.tail + self.buf.len() - 1) % self.buf.len()] } }
    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn cap(&self) -> usize { self.buf.len() }
    pub fn total_push_front(&self) -> u64 { self.total_push_front }
    pub fn total_push_back(&self) -> u64 { self.total_push_back }
    pub fn total_pop_front(&self) -> u64 { self.total_pop_front }
    pub fn total_pop_back(&self) -> u64 { self.total_pop_back }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_back_pop_front() {
        let mut d = DequeS::new(4);
        d.push_back(1); d.push_back(2); d.push_back(3);
        assert_eq!(d.pop_front(), Some(1));
        assert_eq!(d.pop_front(), Some(2));
        assert_eq!(d.pop_front(), Some(3));
    }

    #[test]
    fn push_front_pop_back() {
        let mut d = DequeS::new(4);
        d.push_front(1); d.push_front(2);
        assert_eq!(d.pop_back(), Some(1));
        assert_eq!(d.pop_back(), Some(2));
    }

    #[test]
    fn full() {
        let mut d = DequeS::new(2);
        assert!(d.push_back(1)); assert!(d.push_back(2));
        assert!(!d.push_back(3));
    }

    #[test]
    fn wrap_around() {
        let mut d = DequeS::new(3);
        d.push_back(1); d.push_back(2); d.pop_front();
        assert!(d.push_back(3)); assert!(d.push_back(4));
        assert_eq!(d.len(), 3);
    }

    #[test]
    fn front_back() {
        let mut d = DequeS::new(4);
        d.push_back(10); d.push_back(20);
        assert_eq!(d.front(), Some(10));
        assert_eq!(d.back(), Some(20));
    }

    #[test]
    fn empty_pop() {
        let mut d = DequeS::new(4);
        assert!(d.pop_front().is_none());
        assert!(d.pop_back().is_none());
    }

    #[test]
    fn stats() {
        let mut d = DequeS::new(4);
        d.push_back(1); d.push_front(2); d.pop_front(); d.pop_back();
        assert_eq!(d.total_push_back(), 1);
        assert_eq!(d.total_push_front(), 1);
        assert_eq!(d.total_pop_front(), 1);
        assert_eq!(d.total_pop_back(), 1);
    }
}

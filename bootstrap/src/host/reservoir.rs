pub struct Reservoir {
    buf: Vec<u64>,
    cap: usize,
    seen: u64,
    state: u64,
}

impl Reservoir {
    pub fn new(cap: usize, seed: u64) -> Self { Self { buf: Vec::with_capacity(cap), cap: cap.max(1), seen: 0, state: if seed == 0 { 1 } else { seed } } }

    fn next_rand(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn add(&mut self, val: u64) {
        self.seen += 1;
        if self.buf.len() < self.cap { self.buf.push(val); }
        else {
            let r = (self.next_rand() as usize) % (self.seen as usize);
            if r < self.cap { self.buf[r] = val; }
        }
    }

    pub fn sample(&self) -> &[u64] { &self.buf }
    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
    pub fn seen(&self) -> u64 { self.seen }
    pub fn cap(&self) -> usize { self.cap }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill() {
        let mut r = Reservoir::new(5, 42);
        for i in 0..5u64 { r.add(i); }
        assert_eq!(r.sample().len(), 5);
    }

    #[test]
    fn overflow() {
        let mut r = Reservoir::new(3, 42);
        for i in 0..100u64 { r.add(i); }
        assert_eq!(r.sample().len(), 3);
        assert_eq!(r.seen(), 100);
    }

    #[test]
    fn replacement() {
        let mut r = Reservoir::new(10, 123);
        for i in 0..1000u64 { r.add(i); }
        let max = *r.sample().iter().max().unwrap();
        assert!(max >= 10, "at least one item > 10 should be sampled, got max={max}");
    }

    #[test]
    fn empty() { assert!(Reservoir::new(5, 42).is_empty()); }

    #[test]
    fn single_cap() {
        let mut r = Reservoir::new(1, 42);
        for i in 0..100u64 { r.add(i); }
        assert_eq!(r.sample().len(), 1);
    }

    #[test]
    fn stats() {
        let mut r = Reservoir::new(5, 42);
        for i in 0..10u64 { r.add(i); }
        assert_eq!(r.seen(), 10);
        assert_eq!(r.cap(), 5);
    }
}

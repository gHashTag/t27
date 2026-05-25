use std::collections::VecDeque;

pub struct BlockQ {
    q: VecDeque<Vec<u8>>,
    cap: usize,
    closed: bool,
    total_enqueued: u64,
    total_dequeued: u64,
    total_dropped: u64,
    bytes_enqueued: u64,
    bytes_dequeued: u64,
}

impl BlockQ {
    pub fn new(cap: usize) -> Self { Self { q: VecDeque::with_capacity(cap), cap, closed: false, total_enqueued: 0, total_dequeued: 0, total_dropped: 0, bytes_enqueued: 0, bytes_dequeued: 0 } }

    pub fn try_push(&mut self, data: Vec<u8>) -> Result<(), Vec<u8>> {
        if self.closed { return Err(data); }
        if self.q.len() >= self.cap { self.total_dropped += 1; return Err(data); }
        self.total_enqueued += 1;
        self.bytes_enqueued += data.len() as u64;
        self.q.push_back(data);
        Ok(())
    }

    pub fn force_push(&mut self, data: Vec<u8>) {
        if self.closed { return; }
        self.total_enqueued += 1;
        self.bytes_enqueued += data.len() as u64;
        if self.q.len() >= self.cap { let _ = self.q.pop_front(); self.total_dropped += 1; }
        self.q.push_back(data);
    }

    pub fn pop(&mut self) -> Option<Vec<u8>> {
        let data = self.q.pop_front()?;
        self.total_dequeued += 1;
        self.bytes_dequeued += data.len() as u64;
        Some(data)
    }

    pub fn peek(&self) -> Option<&Vec<u8>> { self.q.front() }

    pub fn close(&mut self) { self.closed = true; }

    pub fn drain(&mut self) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        while let Some(d) = self.pop() { out.push(d); }
        out
    }

    pub fn len(&self) -> usize { self.q.len() }
    pub fn is_empty(&self) -> bool { self.q.is_empty() }
    pub fn is_full(&self) -> bool { self.q.len() >= self.cap }
    pub fn is_closed(&self) -> bool { self.closed }
    pub fn cap(&self) -> usize { self.cap }
    pub fn total_enqueued(&self) -> u64 { self.total_enqueued }
    pub fn total_dequeued(&self) -> u64 { self.total_dequeued }
    pub fn total_dropped(&self) -> u64 { self.total_dropped }
    pub fn drop_rate(&self) -> f64 { if self.total_enqueued == 0 { 0.0 } else { self.total_dropped as f64 / self.total_enqueued as f64 } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut bq = BlockQ::new(4);
        bq.try_push(b"hello".to_vec()).unwrap();
        assert_eq!(bq.pop().unwrap(), b"hello");
    }

    #[test]
    fn full_rejects() {
        let mut bq = BlockQ::new(2);
        bq.try_push(b"a".to_vec()).unwrap();
        bq.try_push(b"b".to_vec()).unwrap();
        assert!(bq.try_push(b"c".to_vec()).is_err());
    }

    #[test]
    fn force_push_evicts() {
        let mut bq = BlockQ::new(2);
        bq.force_push(b"a".to_vec());
        bq.force_push(b"b".to_vec());
        bq.force_push(b"c".to_vec());
        assert_eq!(bq.len(), 2);
        assert_eq!(bq.total_dropped(), 1);
    }

    #[test]
    fn close_rejects() {
        let mut bq = BlockQ::new(4);
        bq.close();
        assert!(bq.try_push(b"x".to_vec()).is_err());
    }

    #[test]
    fn drain() {
        let mut bq = BlockQ::new(4);
        bq.try_push(b"a".to_vec()).unwrap();
        bq.try_push(b"b".to_vec()).unwrap();
        let all = bq.drain();
        assert_eq!(all.len(), 2);
        assert!(bq.is_empty());
    }

    #[test]
    fn peek() {
        let mut bq = BlockQ::new(4);
        bq.try_push(b"front".to_vec()).unwrap();
        assert_eq!(bq.peek().unwrap(), b"front");
    }

    #[test]
    fn drop_rate() {
        let mut bq = BlockQ::new(1);
        bq.try_push(b"a".to_vec()).unwrap();
        let _ = bq.try_push(b"b".to_vec());
        assert!(bq.drop_rate() > 0.0);
    }

    #[test]
    fn stats() {
        let mut bq = BlockQ::new(4);
        bq.try_push(b"x".to_vec()).unwrap(); bq.pop();
        assert_eq!(bq.total_enqueued(), 1);
        assert_eq!(bq.total_dequeued(), 1);
    }
}

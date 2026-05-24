use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MailboxError {
    Full { capacity: usize },
    Empty,
}

impl std::fmt::Display for MailboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MailboxError::Full { capacity } => write!(f, "mailbox full (cap {capacity})"),
            MailboxError::Empty => write!(f, "mailbox empty"),
        }
    }
}

impl std::error::Error for MailboxError {}

#[derive(Debug, Clone)]
pub struct Envelope {
    pub id: u64,
    pub sender: u64,
    pub priority: Priority,
    pub payload: Vec<u8>,
}

pub struct Mailbox {
    lanes: [VecDeque<Envelope>; 3],
    capacity: usize,
    next_id: u64,
    total_sent: u64,
    total_received: u64,
    total_dropped: u64,
}

impl Mailbox {
    pub fn new(capacity: usize) -> Self {
        Self {
            lanes: [VecDeque::new(), VecDeque::new(), VecDeque::new()],
            capacity, next_id: 1, total_sent: 0, total_received: 0, total_dropped: 0,
        }
    }

    fn total_len(&self) -> usize {
        self.lanes.iter().map(|q| q.len()).sum()
    }

    pub fn send(&mut self, sender: u64, priority: Priority, payload: Vec<u8>) -> Result<u64, MailboxError> {
        if self.total_len() >= self.capacity {
            self.total_dropped += 1;
            return Err(MailboxError::Full { capacity: self.capacity });
        }
        let id = self.next_id;
        self.next_id += 1;
        let lane = match priority {
            Priority::High => &mut self.lanes[2],
            Priority::Normal => &mut self.lanes[1],
            Priority::Low => &mut self.lanes[0],
        };
        lane.push_back(Envelope { id, sender, priority, payload });
        self.total_sent += 1;
        Ok(id)
    }

    pub fn receive(&mut self) -> Option<Envelope> {
        for lane in self.lanes.iter_mut().rev() {
            if let Some(env) = lane.pop_front() {
                self.total_received += 1;
                return Some(env);
            }
        }
        None
    }

    pub fn receive_batch(&mut self, max: usize) -> Vec<Envelope> {
        let mut batch = Vec::with_capacity(max);
        while batch.len() < max {
            match self.receive() {
                Some(env) => batch.push(env),
                None => break,
            }
        }
        batch
    }

    pub fn peek(&self) -> Option<&Envelope> {
        for lane in self.lanes.iter().rev() {
            if let Some(env) = lane.front() { return Some(env); }
        }
        None
    }

    pub fn len(&self) -> usize { self.total_len() }
    pub fn is_empty(&self) -> bool { self.total_len() == 0 }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn total_sent(&self) -> u64 { self.total_sent }
    pub fn total_received(&self) -> u64 { self.total_received }
    pub fn total_dropped(&self) -> u64 { self.total_dropped }

    pub fn lane_len(&self, p: Priority) -> usize {
        match p { Priority::Low => self.lanes[0].len(), Priority::Normal => self.lanes[1].len(), Priority::High => self.lanes[2].len() }
    }

    pub fn clear(&mut self) {
        for lane in &mut self.lanes { lane.clear(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_mailbox() {
        let mb = Mailbox::new(100);
        assert!(mb.is_empty());
        assert_eq!(mb.capacity(), 100);
    }

    #[test]
    fn send_receive() {
        let mut mb = Mailbox::new(10);
        let id = mb.send(1, Priority::Normal, vec![1, 2, 3]).unwrap();
        assert_eq!(mb.len(), 1);
        let env = mb.receive().unwrap();
        assert_eq!(env.id, id);
        assert_eq!(env.sender, 1);
        assert_eq!(env.payload, vec![1, 2, 3]);
    }

    #[test]
    fn priority_order() {
        let mut mb = Mailbox::new(10);
        mb.send(1, Priority::Low, vec![1]).unwrap();
        mb.send(1, Priority::High, vec![2]).unwrap();
        mb.send(1, Priority::Normal, vec![3]).unwrap();
        let e1 = mb.receive().unwrap();
        assert_eq!(e1.priority, Priority::High);
        let e2 = mb.receive().unwrap();
        assert_eq!(e2.priority, Priority::Normal);
        let e3 = mb.receive().unwrap();
        assert_eq!(e3.priority, Priority::Low);
    }

    #[test]
    fn full() {
        let mut mb = Mailbox::new(2);
        mb.send(1, Priority::Normal, vec![]).unwrap();
        mb.send(1, Priority::Normal, vec![]).unwrap();
        let err = mb.send(1, Priority::Normal, vec![]).unwrap_err();
        assert!(matches!(err, MailboxError::Full { .. }));
        assert_eq!(mb.total_dropped(), 1);
    }

    #[test]
    fn empty_receive() {
        let mut mb = Mailbox::new(10);
        assert!(mb.receive().is_none());
    }

    #[test]
    fn batch_receive() {
        let mut mb = Mailbox::new(10);
        for i in 0..5 { mb.send(1, Priority::Normal, vec![i]).unwrap(); }
        let batch = mb.receive_batch(3);
        assert_eq!(batch.len(), 3);
        assert_eq!(mb.len(), 2);
    }

    #[test]
    fn peek() {
        let mut mb = Mailbox::new(10);
        mb.send(1, Priority::High, vec![42]).unwrap();
        let env = mb.peek().unwrap();
        assert_eq!(env.payload, vec![42]);
        assert_eq!(mb.len(), 1);
    }

    #[test]
    fn lane_lengths() {
        let mut mb = Mailbox::new(20);
        mb.send(1, Priority::Low, vec![]).unwrap();
        mb.send(1, Priority::High, vec![]).unwrap();
        mb.send(1, Priority::Normal, vec![]).unwrap();
        assert_eq!(mb.lane_len(Priority::High), 1);
        assert_eq!(mb.lane_len(Priority::Normal), 1);
        assert_eq!(mb.lane_len(Priority::Low), 1);
    }

    #[test]
    fn stats() {
        let mut mb = Mailbox::new(10);
        mb.send(1, Priority::Normal, vec![]).unwrap();
        mb.send(1, Priority::Normal, vec![]).unwrap();
        mb.receive().unwrap();
        assert_eq!(mb.total_sent(), 2);
        assert_eq!(mb.total_received(), 1);
    }

    #[test]
    fn clear() {
        let mut mb = Mailbox::new(10);
        mb.send(1, Priority::Normal, vec![]).unwrap();
        mb.clear();
        assert!(mb.is_empty());
    }

    #[test]
    fn fifo_within_lane() {
        let mut mb = Mailbox::new(10);
        mb.send(1, Priority::Normal, vec![1]).unwrap();
        mb.send(1, Priority::Normal, vec![2]).unwrap();
        assert_eq!(mb.receive().unwrap().payload, vec![1]);
        assert_eq!(mb.receive().unwrap().payload, vec![2]);
    }

    #[test]
    fn error_display() {
        assert!(MailboxError::Empty.to_string().contains("empty"));
    }
}

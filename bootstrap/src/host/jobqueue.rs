use std::collections::BinaryHeap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    pub id: usize,
    pub priority: i32,
    pub data: Vec<u8>,
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.priority.cmp(&other.priority).then(other.id.cmp(&self.id)) }
}
impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}

pub struct JobQueue {
    heap: BinaryHeap<Job>,
    next_id: usize,
    completed: usize,
    max_priority: i32,
}

impl JobQueue {
    pub fn new() -> Self { Self { heap: BinaryHeap::new(), next_id: 0, completed: 0, max_priority: 0 } }

    pub fn submit(&mut self, priority: i32, data: Vec<u8>) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.max_priority = self.max_priority.max(priority);
        self.heap.push(Job { id, priority, data });
        id
    }

    pub fn poll(&mut self) -> Option<Job> {
        let job = self.heap.pop();
        if job.is_some() { self.completed += 1; }
        job
    }

    pub fn peek(&self) -> Option<&Job> { self.heap.peek() }

    pub fn len(&self) -> usize { self.heap.len() }
    pub fn is_empty(&self) -> bool { self.heap.is_empty() }
    pub fn completed(&self) -> usize { self.completed }
    pub fn total_submitted(&self) -> usize { self.next_id }
    pub fn max_priority(&self) -> i32 { self.max_priority }

    pub fn drain(&mut self) -> Vec<Job> {
        let mut jobs = Vec::new();
        while let Some(j) = self.poll() { jobs.push(j); }
        jobs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_order() {
        let mut q = JobQueue::new();
        q.submit(1, vec![1]); q.submit(3, vec![3]); q.submit(2, vec![2]);
        assert_eq!(q.poll().unwrap().priority, 3);
        assert_eq!(q.poll().unwrap().priority, 2);
        assert_eq!(q.poll().unwrap().priority, 1);
    }

    #[test]
    fn fifo_same_priority() {
        let mut q = JobQueue::new();
        q.submit(1, vec![1]); q.submit(1, vec![2]);
        assert_eq!(q.poll().unwrap().data, vec![1]);
        assert_eq!(q.poll().unwrap().data, vec![2]);
    }

    #[test]
    fn stats() {
        let mut q = JobQueue::new();
        q.submit(5, vec![]); q.submit(3, vec![]);
        assert_eq!(q.total_submitted(), 2);
        q.poll();
        assert_eq!(q.completed(), 1);
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn drain() {
        let mut q = JobQueue::new();
        q.submit(1, vec![]); q.submit(2, vec![]); q.submit(3, vec![]);
        let jobs = q.drain();
        assert_eq!(jobs.len(), 3);
        assert!(q.is_empty());
    }

    #[test]
    fn empty_poll() {
        let mut q = JobQueue::new();
        assert!(q.poll().is_none());
        assert_eq!(q.completed(), 0);
    }
}

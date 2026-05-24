#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PriError {
    Empty,
    CapacityExceeded { capacity: usize },
}

impl std::fmt::Display for PriError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PriError::Empty => write!(f, "queue empty"),
            PriError::CapacityExceeded { capacity } => write!(f, "cap {capacity} exceeded"),
        }
    }
}

impl std::error::Error for PriError {}

#[derive(Debug, Clone)]
pub struct PriEntry<T> {
    pub priority: u32,
    pub data: T,
}

impl<T> Ord for PriEntry<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl<T> PartialOrd for PriEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for PriEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl<T> Eq for PriEntry<T> {}

#[derive(Debug, Clone)]
pub struct PriorityQueue<T> {
    heap: Vec<PriEntry<T>>,
    capacity: Option<usize>,
    total_push: u64,
    total_pop: u64,
    max_depth: usize,
}

impl<T> PriorityQueue<T> {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            capacity: None,
            total_push: 0,
            total_pop: 0,
            max_depth: 0,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            heap: Vec::with_capacity(cap),
            capacity: Some(cap),
            total_push: 0,
            total_pop: 0,
            max_depth: 0,
        }
    }

    pub fn push(&mut self, priority: u32, data: T) -> Result<(), PriError> {
        if let Some(cap) = self.capacity {
            if self.heap.len() >= cap {
                return Err(PriError::CapacityExceeded { capacity: cap });
            }
        }
        self.heap.push(PriEntry { priority, data });
        let mut i = self.heap.len() - 1;
        while i > 0 {
            let parent = (i - 1) / 2;
            if self.heap[i].priority > self.heap[parent].priority {
                self.heap.swap(i, parent);
                i = parent;
            } else {
                break;
            }
        }
        self.total_push += 1;
        if self.heap.len() > self.max_depth {
            self.max_depth = self.heap.len();
        }
        Ok(())
    }

    pub fn pop(&mut self) -> Option<PriEntry<T>> {
        if self.heap.is_empty() { return None; }
        let last = self.heap.len() - 1;
        self.heap.swap(0, last);
        let result = self.heap.pop();
        self.sink_down(0);
        self.total_pop += 1;
        result
    }

    fn sink_down(&mut self, mut i: usize) {
        let len = self.heap.len();
        loop {
            let left = 2 * i + 1;
            let right = 2 * i + 2;
            let mut largest = i;
            if left < len && self.heap[left].priority > self.heap[largest].priority {
                largest = left;
            }
            if right < len && self.heap[right].priority > self.heap[largest].priority {
                largest = right;
            }
            if largest == i { break; }
            self.heap.swap(i, largest);
            i = largest;
        }
    }

    pub fn peek(&self) -> Option<&PriEntry<T>> {
        self.heap.first()
    }

    pub fn peek_priority(&self) -> Option<u32> {
        self.heap.first().map(|e| e.priority)
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn clear(&mut self) {
        self.heap.clear();
    }

    pub fn total_push(&self) -> u64 {
        self.total_push
    }

    pub fn total_pop(&self) -> u64 {
        self.total_pop
    }

    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    pub fn contains_priority<F>(&self, priority: u32, eq: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        self.heap.iter().any(|e| e.priority == priority && eq(&e.data))
    }

    pub fn retain<F>(&mut self, mut pred: F)
    where
        F: FnMut(&PriEntry<T>) -> bool,
    {
        let old_len = self.heap.len();
        self.heap.retain(|e| pred(e));
        if self.heap.len() < old_len {
            for i in (0..self.heap.len() / 2).rev() {
                self.sink_down(i);
            }
        }
    }
}

impl<T> Default for PriorityQueue<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue() {
        let q: PriorityQueue<i32> = PriorityQueue::new();
        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn push_pop_order() {
        let mut q = PriorityQueue::new();
        q.push(1, "low").unwrap();
        q.push(5, "high").unwrap();
        q.push(3, "mid").unwrap();
        assert_eq!(q.pop().unwrap().data, "high");
        assert_eq!(q.pop().unwrap().data, "mid");
        assert_eq!(q.pop().unwrap().data, "low");
    }

    #[test]
    fn peek_does_not_remove() {
        let mut q = PriorityQueue::new();
        q.push(10, "a").unwrap();
        assert_eq!(q.peek().unwrap().data, "a");
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn peek_priority() {
        let mut q = PriorityQueue::new();
        q.push(5, "a").unwrap();
        q.push(10, "b").unwrap();
        assert_eq!(q.peek_priority(), Some(10));
    }

    #[test]
    fn pop_empty() {
        let mut q: PriorityQueue<i32> = PriorityQueue::new();
        assert!(q.pop().is_none());
    }

    #[test]
    fn capacity_exceeded() {
        let mut q = PriorityQueue::with_capacity(2);
        q.push(1, "a").unwrap();
        q.push(2, "b").unwrap();
        let err = q.push(3, "c").unwrap_err();
        assert!(matches!(err, PriError::CapacityExceeded { capacity: 2 }));
    }

    #[test]
    fn stats() {
        let mut q = PriorityQueue::new();
        q.push(1, "a").unwrap();
        q.push(2, "b").unwrap();
        q.push(3, "c").unwrap();
        assert_eq!(q.total_push(), 3);
        assert_eq!(q.max_depth(), 3);
        q.pop();
        assert_eq!(q.total_pop(), 1);
    }

    #[test]
    fn clear() {
        let mut q = PriorityQueue::new();
        q.push(1, "a").unwrap();
        q.push(2, "b").unwrap();
        q.clear();
        assert!(q.is_empty());
    }

    #[test]
    fn retain() {
        let mut q = PriorityQueue::new();
        q.push(1, "keep").unwrap();
        q.push(2, "drop").unwrap();
        q.push(3, "keep").unwrap();
        q.retain(|e| e.data != "drop");
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn contains_priority() {
        let mut q = PriorityQueue::new();
        q.push(5, "a").unwrap();
        assert!(q.contains_priority(5, |d| *d == "a"));
        assert!(!q.contains_priority(5, |d| *d == "b"));
        assert!(!q.contains_priority(1, |d| *d == "a"));
    }

    #[test]
    fn many_elements() {
        let mut q = PriorityQueue::new();
        for i in 0..100 {
            q.push(i, i).unwrap();
        }
        assert_eq!(q.pop().unwrap().priority, 99);
        assert_eq!(q.pop().unwrap().priority, 98);
    }

    #[test]
    fn error_display() {
        assert!(PriError::Empty.to_string().contains("empty"));
        assert!(PriError::CapacityExceeded { capacity: 8 }.to_string().contains("8"));
    }
}

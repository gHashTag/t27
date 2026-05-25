use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub struct KahnPq {
    heap: BinaryHeap<Reverse<u64>>,
    total_push: u64,
    total_pop: u64,
}

impl KahnPq {
    pub fn new() -> Self { Self { heap: BinaryHeap::new(), total_push: 0, total_pop: 0 } }
    pub fn push(&mut self, val: u64) { self.total_push += 1; self.heap.push(Reverse(val)); }
    pub fn pop(&mut self) -> Option<u64> { self.total_pop += 1; self.heap.pop().map(|Reverse(v)| v) }
    pub fn peek(&self) -> Option<u64> { self.heap.peek().map(|Reverse(v)| *v) }
    pub fn len(&self) -> usize { self.heap.len() }
    pub fn is_empty(&self) -> bool { self.heap.is_empty() }
    pub fn total_push(&self) -> u64 { self.total_push }
    pub fn total_pop(&self) -> u64 { self.total_pop }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered() { let mut q = KahnPq::new(); q.push(3); q.push(1); q.push(2); assert_eq!(q.pop(), Some(1)); assert_eq!(q.pop(), Some(2)); assert_eq!(q.pop(), Some(3)); }
    #[test]
    fn peek() { let mut q = KahnPq::new(); q.push(5); q.push(1); assert_eq!(q.peek(), Some(1)); }
    #[test]
    fn empty() { assert!(KahnPq::new().pop().is_none()); }
    #[test]
    fn dup() { let mut q = KahnPq::new(); q.push(1); q.push(1); assert_eq!(q.pop(), Some(1)); assert_eq!(q.pop(), Some(1)); }
    #[test]
    fn stats() { let mut q = KahnPq::new(); q.push(1); q.pop(); assert_eq!(q.total_push(), 1); assert_eq!(q.total_pop(), 1); }
}

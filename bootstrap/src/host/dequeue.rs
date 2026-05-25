pub struct Dequeue<T> {
    front: Vec<T>,
    back: Vec<T>,
}

impl<T> Dequeue<T> {
    pub fn new() -> Self { Self { front: Vec::new(), back: Vec::new() } }

    pub fn push_back(&mut self, val: T) { self.back.push(val); }

    pub fn push_front(&mut self, val: T) { self.front.push(val); }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.front.is_empty() {
            self.back.reverse();
            std::mem::swap(&mut self.front, &mut self.back);
        }
        self.front.pop()
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.back.is_empty() {
            self.front.reverse();
            std::mem::swap(&mut self.front, &mut self.back);
        }
        self.back.pop()
    }

    pub fn front(&self) -> Option<&T> {
        if let Some(v) = self.front.last() { Some(v) }
        else { self.back.first() }
    }

    pub fn back(&self) -> Option<&T> {
        if let Some(v) = self.back.last() { Some(v) }
        else { self.front.first() }
    }

    pub fn len(&self) -> usize { self.front.len() + self.back.len() }
    pub fn is_empty(&self) -> bool { self.front.is_empty() && self.back.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_back() {
        let mut d: Dequeue<i32> = Dequeue::new();
        d.push_back(1); d.push_back(2); d.push_back(3);
        assert_eq!(d.pop_back(), Some(3));
        assert_eq!(d.pop_front(), Some(1));
    }

    #[test]
    fn push_pop_front() {
        let mut d: Dequeue<i32> = Dequeue::new();
        d.push_front(1); d.push_front(2);
        assert_eq!(d.pop_front(), Some(2));
        assert_eq!(d.pop_back(), Some(1));
    }

    #[test]
    fn front_back() {
        let mut d = Dequeue::new();
        d.push_back(10); d.push_back(20); d.push_back(30);
        assert_eq!(d.front(), Some(&10));
        assert_eq!(d.back(), Some(&30));
    }

    #[test]
    fn empty_ops() {
        let mut d: Dequeue<i32> = Dequeue::new();
        assert_eq!(d.pop_front(), None);
        assert_eq!(d.pop_back(), None);
        assert!(d.is_empty());
    }

    #[test]
    fn mixed_ops() {
        let mut d = Dequeue::new();
        d.push_back(1); d.push_front(2); d.push_back(3);
        assert_eq!(d.pop_front(), Some(2));
        assert_eq!(d.pop_back(), Some(3));
        assert_eq!(d.len(), 1);
    }
}

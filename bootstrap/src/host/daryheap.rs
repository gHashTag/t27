pub struct DAryHeap {
    data: Vec<(u64, i64)>,
    d: usize,
    total_pushes: u64,
    total_pops: u64,
}

impl DAryHeap {
    pub fn new(d: usize) -> Self { Self { data: Vec::new(), d: d.max(2), total_pushes: 0, total_pops: 0 } }

    fn parent(&self, i: usize) -> usize { (i - 1) / self.d }
    fn child(&self, i: usize, k: usize) -> usize { self.d * i + k + 1 }

    fn sift_up(&mut self, mut i: usize) {
        while i > 0 {
            let p = self.parent(i);
            if self.data[i].1 < self.data[p].1 { self.data.swap(i, p); i = p; } else { break; }
        }
    }

    fn sift_down(&mut self, mut i: usize) {
        loop {
            let mut smallest = i;
            for k in 0..self.d {
                let c = self.child(i, k);
                if c < self.data.len() && self.data[c].1 < self.data[smallest].1 { smallest = c; }
            }
            if smallest == i { break; }
            self.data.swap(i, smallest);
            i = smallest;
        }
    }

    pub fn push(&mut self, id: u64, key: i64) {
        self.total_pushes += 1;
        self.data.push((id, key));
        self.sift_up(self.data.len() - 1);
    }

    pub fn pop(&mut self) -> Option<(u64, i64)> {
        self.total_pops += 1;
        if self.data.is_empty() { return None; }
        let root = self.data[0];
        let last = self.data.pop()?;
        if !self.data.is_empty() { self.data[0] = last; self.sift_down(0); }
        Some(root)
    }

    pub fn peek(&self) -> Option<(u64, i64)> { self.data.first().copied() }
    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn branching(&self) -> usize { self.d }
    pub fn total_pushes(&self) -> u64 { self.total_pushes }
    pub fn total_pops(&self) -> u64 { self.total_pops }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_push_pop() {
        let mut h = DAryHeap::new(2);
        h.push(1, 30); h.push(2, 10); h.push(3, 20);
        assert_eq!(h.pop(), Some((2, 10)));
        assert_eq!(h.pop(), Some((3, 20)));
        assert_eq!(h.pop(), Some((1, 30)));
    }

    #[test]
    fn quad_heap() {
        let mut h = DAryHeap::new(4);
        for i in (1..=20u64).rev() { h.push(i, i as i64); }
        let mut prev = 0i64;
        while let Some((_, k)) = h.pop() { assert!(k >= prev); prev = k; }
    }

    #[test]
    fn peek() {
        let mut h = DAryHeap::new(3);
        h.push(1, 5); h.push(2, 1);
        assert_eq!(h.peek(), Some((2, 1)));
    }

    #[test]
    fn empty_pop() { assert!(DAryHeap::new(2).pop().is_none()); }

    #[test]
    fn single() {
        let mut h = DAryHeap::new(2);
        h.push(1, 42);
        assert_eq!(h.pop(), Some((1, 42)));
    }

    #[test]
    fn decrease_key_via_push() {
        let mut h = DAryHeap::new(2);
        h.push(1, 100); h.push(2, 50);
        assert_eq!(h.pop(), Some((2, 50)));
    }

    #[test]
    fn stats() {
        let mut h = DAryHeap::new(2);
        h.push(1, 1); h.pop();
        assert_eq!(h.total_pushes(), 1);
        assert_eq!(h.total_pops(), 1);
    }
}

use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum RpError {
    PoolEmpty,
}

impl std::fmt::Display for RpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpError::PoolEmpty => write!(f, "pool empty"),
        }
    }
}

impl std::error::Error for RpError {}

pub struct RefPool<T> {
    available: VecDeque<T>,
    in_use: usize,
    total_acquires: u64,
    total_releases: u64,
    total_created: u64,
    max_in_use: usize,
}

impl<T> RefPool<T> {
    pub fn new() -> Self { Self { available: VecDeque::new(), in_use: 0, total_acquires: 0, total_releases: 0, total_created: 0, max_in_use: 0 } }

    pub fn with_capacity(cap: usize) -> Self { Self { available: VecDeque::with_capacity(cap), in_use: 0, total_acquires: 0, total_releases: 0, total_created: 0, max_in_use: 0 } }

    pub fn seed(&mut self, item: T) {
        self.total_created += 1;
        self.available.push_back(item);
    }

    pub fn acquire(&mut self) -> Result<T, RpError> {
        self.total_acquires += 1;
        match self.available.pop_front() {
            Some(item) => {
                self.in_use += 1;
                if self.in_use > self.max_in_use { self.max_in_use = self.in_use; }
                Ok(item)
            }
            None => Err(RpError::PoolEmpty),
        }
    }

    pub fn release(&mut self, item: T) {
        self.total_releases += 1;
        self.in_use -= 1;
        self.available.push_back(item);
    }

    pub fn acquire_or_create<F>(&mut self, factory: F) -> T
    where F: FnOnce() -> T {
        self.total_acquires += 1;
        if let Some(item) = self.available.pop_front() {
            self.in_use += 1;
            if self.in_use > self.max_in_use { self.max_in_use = self.in_use; }
            item
        } else {
            self.total_created += 1;
            self.in_use += 1;
            if self.in_use > self.max_in_use { self.max_in_use = self.in_use; }
            factory()
        }
    }

    pub fn drain(&mut self) -> Vec<T> { self.available.drain(..).collect() }

    pub fn available(&self) -> usize { self.available.len() }
    pub fn in_use(&self) -> usize { self.in_use }
    pub fn total_acquires(&self) -> u64 { self.total_acquires }
    pub fn total_releases(&self) -> u64 { self.total_releases }
    pub fn total_created(&self) -> u64 { self.total_created }
    pub fn max_in_use(&self) -> usize { self.max_in_use }
}

impl<T> Default for RefPool<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool() { let p: RefPool<i32> = RefPool::new(); assert_eq!(p.available(), 0); assert_eq!(p.in_use(), 0); }

    #[test]
    fn seed_acquire_release() {
        let mut p = RefPool::new();
        p.seed(42); p.seed(99);
        let a = p.acquire().unwrap();
        assert_eq!(p.in_use(), 1);
        p.release(a);
        assert_eq!(p.available(), 2);
        assert_eq!(p.in_use(), 0);
    }

    #[test]
    fn empty_acquire() { assert!(RefPool::<i32>::new().acquire().is_err()); }

    #[test]
    fn acquire_or_create() {
        let mut p: RefPool<i32> = RefPool::new();
        let v = p.acquire_or_create(|| 100);
        assert_eq!(v, 100);
        assert_eq!(p.total_created(), 1);
    }

    #[test]
    fn reuse() {
        let mut p = RefPool::new();
        p.seed(1); p.seed(2);
        let a = p.acquire().unwrap(); let b = p.acquire().unwrap();
        p.release(a); p.release(b);
        let c = p.acquire().unwrap();
        assert!(c == 1 || c == 2);
        assert_eq!(p.total_created(), 2);
    }

    #[test]
    fn max_in_use() {
        let mut p = RefPool::new();
        for i in 0..5 { p.seed(i); }
        let mut held = Vec::new();
        for _ in 0..5 { held.push(p.acquire().unwrap()); }
        assert_eq!(p.max_in_use(), 5);
        for h in held { p.release(h); }
        assert_eq!(p.max_in_use(), 5);
    }

    #[test]
    fn drain() {
        let mut p = RefPool::new();
        for i in 0..3 { p.seed(i); }
        let drained = p.drain();
        assert_eq!(drained.len(), 3);
        assert_eq!(p.available(), 0);
    }

    #[test]
    fn stats() {
        let mut p = RefPool::new();
        p.seed(1);
        let v = p.acquire().unwrap();
        p.release(v);
        assert_eq!(p.total_acquires(), 1);
        assert_eq!(p.total_releases(), 1);
        assert_eq!(p.total_created(), 1);
    }

    #[test]
    fn error_display() { assert!(RpError::PoolEmpty.to_string().contains("empty")); }
}

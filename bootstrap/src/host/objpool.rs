use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum PoolError {
    Exhausted { capacity: usize },
    NotInPool { id: u64 },
    DoubleReturn { id: u64 },
    Closed,
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolError::Exhausted { capacity } => write!(f, "pool exhausted (cap {capacity})"),
            PoolError::NotInPool { id } => write!(f, "obj {id} not in pool"),
            PoolError::DoubleReturn { id } => write!(f, "obj {id} double return"),
            PoolError::Closed => write!(f, "pool closed"),
        }
    }
}

impl std::error::Error for PoolError {}

struct PoolSlot<T> {
    id: u64,
    value: Option<T>,
    generation: u64,
}

#[derive(Debug, Clone)]
pub struct PoolHandle {
    pub id: u64,
    pub generation: u64,
}

pub struct ObjectPool<T> {
    slots: Vec<PoolSlot<T>>,
    free_list: VecDeque<usize>,
    capacity: usize,
    next_id: u64,
    total_acquires: u64,
    total_releases: u64,
    closed: bool,
}

impl<T> ObjectPool<T> {
    pub fn new(capacity: usize) -> Self {
        let mut slots = Vec::with_capacity(capacity);
        let mut free_list = VecDeque::with_capacity(capacity);
        for i in 0..capacity {
            slots.push(PoolSlot { id: i as u64, value: None, generation: 0 });
            free_list.push_back(i);
        }
        Self { slots, free_list, capacity, next_id: capacity as u64, total_acquires: 0, total_releases: 0, closed: false }
    }

    pub fn acquire(&mut self, value: T) -> Result<PoolHandle, PoolError> {
        if self.closed { return Err(PoolError::Closed); }
        let idx = self.free_list.pop_front().ok_or(PoolError::Exhausted { capacity: self.capacity })?;
        let slot = &mut self.slots[idx];
        slot.value = Some(value);
        let handle = PoolHandle { id: slot.id, generation: slot.generation };
        self.total_acquires += 1;
        Ok(handle)
    }

    pub fn release(&mut self, handle: PoolHandle) -> Result<T, PoolError> {
        if self.closed { return Err(PoolError::Closed); }
        let idx = self.slots.iter().position(|s| s.id == handle.id)
            .ok_or(PoolError::NotInPool { id: handle.id })?;
        let slot = &mut self.slots[idx];
        if slot.value.is_none() { return Err(PoolError::DoubleReturn { id: handle.id }); }
        if slot.generation != handle.generation { return Err(PoolError::NotInPool { id: handle.id }); }
        slot.generation += 1;
        let value = slot.value.take().unwrap();
        self.free_list.push_back(idx);
        self.total_releases += 1;
        Ok(value)
    }

    pub fn get(&self, handle: &PoolHandle) -> Option<&T> {
        let slot = self.slots.iter().find(|s| s.id == handle.id)?;
        if slot.generation != handle.generation { return None; }
        slot.value.as_ref()
    }

    pub fn get_mut(&mut self, handle: &PoolHandle) -> Option<&mut T> {
        let slot = self.slots.iter_mut().find(|s| s.id == handle.id)?;
        if slot.generation != handle.generation { return None; }
        slot.value.as_mut()
    }

    pub fn available(&self) -> usize { self.free_list.len() }
    pub fn in_use(&self) -> usize { self.capacity - self.free_list.len() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn total_acquires(&self) -> u64 { self.total_acquires }
    pub fn total_releases(&self) -> u64 { self.total_releases }

    pub fn close(&mut self) -> Vec<T> {
        self.closed = true;
        let mut drained = Vec::new();
        for slot in &mut self.slots {
            if let Some(v) = slot.value.take() { drained.push(v); }
        }
        self.free_list.clear();
        drained
    }

    pub fn is_closed(&self) -> bool { self.closed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool() {
        let pool: ObjectPool<i32> = ObjectPool::new(4);
        assert_eq!(pool.available(), 4);
        assert_eq!(pool.in_use(), 0);
    }

    #[test]
    fn acquire_release() {
        let mut pool: ObjectPool<String> = ObjectPool::new(2);
        let h = pool.acquire("hello".to_string()).unwrap();
        assert_eq!(pool.available(), 1);
        assert_eq!(pool.in_use(), 1);
        let v = pool.release(h).unwrap();
        assert_eq!(v, "hello");
        assert_eq!(pool.available(), 2);
    }

    #[test]
    fn exhausted() {
        let mut pool: ObjectPool<i32> = ObjectPool::new(1);
        pool.acquire(1).unwrap();
        let err = pool.acquire(2).unwrap_err();
        assert!(matches!(err, PoolError::Exhausted { .. }));
    }

    #[test]
    fn get_value() {
        let mut pool: ObjectPool<i32> = ObjectPool::new(4);
        let h = pool.acquire(42).unwrap();
        assert_eq!(*pool.get(&h).unwrap(), 42);
    }

    #[test]
    fn get_mut() {
        let mut pool: ObjectPool<i32> = ObjectPool::new(4);
        let h = pool.acquire(10).unwrap();
        *pool.get_mut(&h).unwrap() = 20;
        assert_eq!(*pool.get(&h).unwrap(), 20);
    }

    #[test]
    fn double_return() {
        let mut pool: ObjectPool<i32> = ObjectPool::new(4);
        let h = pool.acquire(1).unwrap();
        pool.release(h.clone()).unwrap();
        let err = pool.release(h).unwrap_err();
        assert!(matches!(err, PoolError::DoubleReturn { .. }));
    }

    #[test]
    fn generation_mismatch() {
        let mut pool: ObjectPool<i32> = ObjectPool::new(4);
        let h = pool.acquire(1).unwrap();
        pool.release(h.clone()).unwrap();
        let h2 = pool.acquire(2).unwrap();
        assert!(pool.get(&h).is_none());
        assert_eq!(*pool.get(&h2).unwrap(), 2);
    }

    #[test]
    fn stats() {
        let mut pool: ObjectPool<i32> = ObjectPool::new(4);
        let h1 = pool.acquire(1).unwrap();
        let h2 = pool.acquire(2).unwrap();
        pool.release(h1).unwrap();
        assert_eq!(pool.total_acquires(), 2);
        assert_eq!(pool.total_releases(), 1);
    }

    #[test]
    fn close_drains() {
        let mut pool: ObjectPool<i32> = ObjectPool::new(4);
        pool.acquire(1).unwrap();
        pool.acquire(2).unwrap();
        let drained = pool.close();
        assert_eq!(drained.len(), 2);
        assert!(pool.is_closed());
        assert_eq!(pool.available(), 0);
    }

    #[test]
    fn acquire_after_close() {
        let mut pool: ObjectPool<i32> = ObjectPool::new(4);
        pool.close();
        let err = pool.acquire(1).unwrap_err();
        assert!(matches!(err, PoolError::Closed));
    }

    #[test]
    fn not_in_pool() {
        let pool: ObjectPool<i32> = ObjectPool::new(4);
        let fake = PoolHandle { id: 999, generation: 0 };
        assert!(pool.get(&fake).is_none());
    }

    #[test]
    fn error_display() {
        assert!(PoolError::Exhausted { capacity: 4 }.to_string().contains("4"));
    }
}

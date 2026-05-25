#[derive(Debug, Clone, PartialEq)]
pub enum MvErr {
    Empty,
    Full,
}

impl std::fmt::Display for MvErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MvErr::Empty => write!(f, "mvar empty"),
            MvErr::Full => write!(f, "mvar full"),
        }
    }
}

impl std::error::Error for MvErr {}

pub struct MVar<T> {
    value: Option<T>,
    total_puts: u64,
    total_takes: u64,
    total_reads: u64,
}

impl<T> MVar<T> {
    pub fn new(value: T) -> Self { Self { value: Some(value), total_puts: 0, total_takes: 0, total_reads: 0 } }

    pub fn empty() -> Self { Self { value: None, total_puts: 0, total_takes: 0, total_reads: 0 } }

    pub fn put(&mut self, value: T) -> Result<(), MvErr> {
        self.total_puts += 1;
        if self.value.is_some() { return Err(MvErr::Full); }
        self.value = Some(value);
        Ok(())
    }

    pub fn take(&mut self) -> Result<T, MvErr> {
        self.total_takes += 1;
        self.value.take().ok_or(MvErr::Empty)
    }

    pub fn read(&mut self) -> Result<&T, MvErr> {
        self.total_reads += 1;
        self.value.as_ref().ok_or(MvErr::Empty)
    }

    pub fn try_put(&mut self, value: T) -> Result<Option<T>, MvErr> {
        self.total_puts += 1;
        let old = self.value.replace(value);
        Ok(old)
    }

    pub fn swap(&mut self, value: T) -> Result<T, MvErr> {
        self.total_puts += 1;
        self.total_reads += 1;
        self.value.replace(value).ok_or(MvErr::Empty)
    }

    pub fn is_empty(&self) -> bool { self.value.is_none() }
    pub fn is_full(&self) -> bool { self.value.is_some() }
    pub fn total_puts(&self) -> u64 { self.total_puts }
    pub fn total_takes(&self) -> u64 { self.total_takes }
    pub fn total_reads(&self) -> u64 { self.total_reads }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_read() {
        let mut mv = MVar::new(42u64);
        assert_eq!(*mv.read().unwrap(), 42);
    }

    #[test]
    fn put_take() {
        let mut mv: MVar<u64> = MVar::empty();
        mv.put(10).unwrap();
        let v = mv.take().unwrap();
        assert_eq!(v, 10);
        assert!(mv.is_empty());
    }

    #[test]
    fn put_full() {
        let mut mv = MVar::new(1u64);
        assert!(mv.put(2).is_err());
    }

    #[test]
    fn take_empty() { assert!(MVar::<u64>::empty().take().is_err()); }

    #[test]
    fn read_empty() { assert!(MVar::<u64>::empty().read().is_err()); }

    #[test]
    fn swap() {
        let mut mv = MVar::new(1u64);
        let old = mv.swap(2).unwrap();
        assert_eq!(old, 1);
        assert_eq!(*mv.read().unwrap(), 2);
    }

    #[test]
    fn try_put() {
        let mut mv = MVar::new(1u64);
        let old = mv.try_put(2).unwrap();
        assert_eq!(old, Some(1));
    }

    #[test]
    fn is_full() {
        let mv = MVar::new(42u64);
        assert!(mv.is_full());
    }

    #[test]
    fn stats() {
        let mut mv: MVar<u64> = MVar::empty();
        mv.put(1).unwrap();
        mv.read().unwrap();
        mv.take().unwrap();
        assert_eq!(mv.total_puts(), 1);
        assert_eq!(mv.total_takes(), 1);
        assert_eq!(mv.total_reads(), 1);
    }

    #[test]
    fn error_display() { assert!(MvErr::Empty.to_string().contains("empty")); }
}

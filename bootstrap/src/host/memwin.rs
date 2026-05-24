#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowError {
    OutOfBounds { offset: u32, size: u32, region: u32 },
    WindowEmpty,
    InvalidRange { start: u32, end: u32 },
}

impl std::fmt::Display for WindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowError::OutOfBounds { offset, size, region } => {
                write!(f, "0x{offset:X}+0x{size:X} exceeds 0x{region:X}")
            }
            WindowError::WindowEmpty => write!(f, "window empty"),
            WindowError::InvalidRange { start, end } => {
                write!(f, "invalid range: 0x{start:X}..0x{end:X}")
            }
        }
    }
}

impl std::error::Error for WindowError {}

#[derive(Debug, Clone)]
pub struct MemoryWindow {
    region_size: u32,
    base: u32,
    size: u32,
}

impl MemoryWindow {
    pub fn new(region_size: u32) -> Self {
        Self {
            region_size,
            base: 0,
            size: region_size,
        }
    }

    pub fn region_size(&self) -> u32 {
        self.region_size
    }

    pub fn base(&self) -> u32 {
        self.base
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn end(&self) -> u32 {
        self.base + self.size
    }

    pub fn set_window(&mut self, base: u32, size: u32) -> Result<(), WindowError> {
        if size == 0 {
            return Err(WindowError::WindowEmpty);
        }
        if base.checked_add(size).map_or(true, |e| e > self.region_size) {
            return Err(WindowError::OutOfBounds {
                offset: base,
                size,
                region: self.region_size,
            });
        }
        self.base = base;
        self.size = size;
        Ok(())
    }

    pub fn contains(&self, addr: u32) -> bool {
        addr >= self.base && addr < self.end()
    }

    pub fn translate(&self, addr: u32) -> Option<u32> {
        if self.contains(addr) {
            Some(addr - self.base)
        } else {
            None
        }
    }

    pub fn translate_in(&self, local_offset: u32) -> Option<u32> {
        if local_offset < self.size {
            Some(self.base + local_offset)
        } else {
            None
        }
    }

    pub fn contains_range(&self, start: u32, len: u32) -> bool {
        start >= self.base && start.checked_add(len).map_or(false, |e| e <= self.end())
    }

    pub fn clamp(&mut self, base: u32, size: u32) {
        let end = (base + size).min(self.region_size);
        let actual_base = base.min(self.region_size);
        self.base = actual_base;
        self.size = end.saturating_sub(actual_base);
    }

    pub fn slide(&mut self, delta: i32) -> Result<(), WindowError> {
        let new_base = if delta >= 0 {
            self.base.checked_add(delta as u32)
        } else {
            self.base.checked_sub((-delta) as u32)
        };
        match new_base {
            Some(b) => {
                let old_base = self.base;
                let old_size = self.size;
                self.base = b;
                self.size = old_size;
                if self.end() > self.region_size {
                    self.base = old_base;
                    self.size = old_size;
                    return Err(WindowError::OutOfBounds {
                        offset: b,
                        size: old_size,
                        region: self.region_size,
                    });
                }
                Ok(())
            }
            None => Err(WindowError::OutOfBounds {
                offset: 0,
                size: self.size,
                region: self.region_size,
            }),
        }
    }

    pub fn reset(&mut self) {
        self.base = 0;
        self.size = self.region_size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_full_region() {
        let w = MemoryWindow::new(0x1000);
        assert_eq!(w.base(), 0);
        assert_eq!(w.size(), 0x1000);
    }

    #[test]
    fn set_window_ok() {
        let mut w = MemoryWindow::new(0x1000);
        w.set_window(0x100, 0x200).unwrap();
        assert_eq!(w.base(), 0x100);
        assert_eq!(w.size(), 0x200);
        assert_eq!(w.end(), 0x300);
    }

    #[test]
    fn set_window_overflow() {
        let mut w = MemoryWindow::new(0x100);
        let err = w.set_window(0x80, 0x100).unwrap_err();
        assert!(matches!(err, WindowError::OutOfBounds { .. }));
    }

    #[test]
    fn set_window_empty() {
        let mut w = MemoryWindow::new(0x100);
        let err = w.set_window(0, 0).unwrap_err();
        assert!(matches!(err, WindowError::WindowEmpty));
    }

    #[test]
    fn contains() {
        let mut w = MemoryWindow::new(0x1000);
        w.set_window(0x100, 0x10).unwrap();
        assert!(w.contains(0x100));
        assert!(w.contains(0x10F));
        assert!(!w.contains(0x110));
    }

    #[test]
    fn translate() {
        let mut w = MemoryWindow::new(0x1000);
        w.set_window(0x200, 0x100).unwrap();
        assert_eq!(w.translate(0x250), Some(0x50));
        assert_eq!(w.translate(0x100), None);
    }

    #[test]
    fn translate_in() {
        let mut w = MemoryWindow::new(0x1000);
        w.set_window(0x200, 0x100).unwrap();
        assert_eq!(w.translate_in(0x50), Some(0x250));
        assert_eq!(w.translate_in(0x100), None);
    }

    #[test]
    fn contains_range() {
        let mut w = MemoryWindow::new(0x1000);
        w.set_window(0x100, 0x100).unwrap();
        assert!(w.contains_range(0x100, 0x100));
        assert!(!w.contains_range(0x100, 0x101));
    }

    #[test]
    fn clamp() {
        let mut w = MemoryWindow::new(0x100);
        w.clamp(0x80, 0x100);
        assert_eq!(w.base(), 0x80);
        assert_eq!(w.size(), 0x80);
    }

    #[test]
    fn slide_forward() {
        let mut w = MemoryWindow::new(0x1000);
        w.set_window(0x100, 0x100).unwrap();
        w.slide(0x50).unwrap();
        assert_eq!(w.base(), 0x150);
    }

    #[test]
    fn slide_overflow() {
        let mut w = MemoryWindow::new(0x100);
        w.set_window(0x80, 0x40).unwrap();
        let err = w.slide(0x100).unwrap_err();
        assert!(matches!(err, WindowError::OutOfBounds { .. }));
    }

    #[test]
    fn slide_backward() {
        let mut w = MemoryWindow::new(0x1000);
        w.set_window(0x100, 0x100).unwrap();
        w.slide(-0x50).unwrap();
        assert_eq!(w.base(), 0xB0);
    }

    #[test]
    fn reset() {
        let mut w = MemoryWindow::new(0x1000);
        w.set_window(0x500, 0x100).unwrap();
        w.reset();
        assert_eq!(w.base(), 0);
        assert_eq!(w.size(), 0x1000);
    }

    #[test]
    fn error_display() {
        assert!(WindowError::OutOfBounds { offset: 1, size: 2, region: 3 }.to_string().contains("0x1"));
        assert!(WindowError::WindowEmpty.to_string().contains("empty"));
        assert!(WindowError::InvalidRange { start: 5, end: 3 }.to_string().contains("invalid"));
    }
}

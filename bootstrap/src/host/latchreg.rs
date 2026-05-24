use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatchState {
    Clean,
    Dirty,
    Committed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LatchError {
    ReadOnly { reg: usize },
    NotDirty { reg: usize },
    Unknown { reg: usize },
    WriteProtected,
}

impl std::fmt::Display for LatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LatchError::ReadOnly { reg } => write!(f, "reg {reg} is read-only"),
            LatchError::NotDirty { reg } => write!(f, "reg {reg} not dirty"),
            LatchError::Unknown { reg } => write!(f, "unknown reg {reg}"),
            LatchError::WriteProtected => write!(f, "write protected"),
        }
    }
}

impl std::error::Error for LatchError {}

#[derive(Debug, Clone)]
struct Reg {
    value: u64,
    shadow: u64,
    state: LatchState,
    writable: bool,
}

#[derive(Debug, Clone)]
pub struct LatchRegisterFile {
    regs: BTreeMap<usize, Reg>,
    write_protect: bool,
    total_commits: u64,
    total_discards: u64,
}

impl LatchRegisterFile {
    pub fn new() -> Self {
        Self { regs: BTreeMap::new(), write_protect: false, total_commits: 0, total_discards: 0 }
    }

    pub fn add_reg(&mut self, idx: usize, initial: u64, writable: bool) {
        self.regs.insert(idx, Reg { value: initial, shadow: initial, state: LatchState::Clean, writable });
    }

    pub fn read(&self, idx: usize) -> Result<u64, LatchError> {
        let reg = self.regs.get(&idx).ok_or(LatchError::Unknown { reg: idx })?;
        Ok(reg.value)
    }

    pub fn write(&mut self, idx: usize, value: u64) -> Result<(), LatchError> {
        if self.write_protect { return Err(LatchError::WriteProtected); }
        let reg = self.regs.get_mut(&idx).ok_or(LatchError::Unknown { reg: idx })?;
        if !reg.writable { return Err(LatchError::ReadOnly { reg: idx }); }
        reg.value = value;
        if reg.value != reg.shadow {
            reg.state = LatchState::Dirty;
        } else {
            reg.state = LatchState::Clean;
        }
        Ok(())
    }

    pub fn commit(&mut self) -> usize {
        let mut count = 0;
        for reg in self.regs.values_mut() {
            if reg.state == LatchState::Dirty {
                reg.shadow = reg.value;
                reg.state = LatchState::Committed;
                count += 1;
            }
        }
        self.total_commits += count as u64;
        count
    }

    pub fn commit_one(&mut self, idx: usize) -> Result<(), LatchError> {
        let reg = self.regs.get_mut(&idx).ok_or(LatchError::Unknown { reg: idx })?;
        if reg.state != LatchState::Dirty {
            return Err(LatchError::NotDirty { reg: idx });
        }
        reg.shadow = reg.value;
        reg.state = LatchState::Committed;
        self.total_commits += 1;
        Ok(())
    }

    pub fn discard(&mut self) -> usize {
        let mut count = 0;
        for reg in self.regs.values_mut() {
            if reg.state == LatchState::Dirty {
                reg.value = reg.shadow;
                reg.state = LatchState::Clean;
                count += 1;
            }
        }
        self.total_discards += count as u64;
        count
    }

    pub fn discard_one(&mut self, idx: usize) -> Result<(), LatchError> {
        let reg = self.regs.get_mut(&idx).ok_or(LatchError::Unknown { reg: idx })?;
        if reg.state != LatchState::Dirty {
            return Err(LatchError::NotDirty { reg: idx });
        }
        reg.value = reg.shadow;
        reg.state = LatchState::Clean;
        self.total_discards += 1;
        Ok(())
    }

    pub fn state(&self, idx: usize) -> Option<LatchState> {
        self.regs.get(&idx).map(|r| r.state)
    }

    pub fn shadow(&self, idx: usize) -> Option<u64> {
        self.regs.get(&idx).map(|r| r.shadow)
    }

    pub fn dirty_count(&self) -> usize {
        self.regs.values().filter(|r| r.state == LatchState::Dirty).count()
    }

    pub fn reg_count(&self) -> usize {
        self.regs.len()
    }

    pub fn set_write_protect(&mut self, enabled: bool) {
        self.write_protect = enabled;
    }

    pub fn is_write_protected(&self) -> bool {
        self.write_protect
    }

    pub fn total_commits(&self) -> u64 {
        self.total_commits
    }

    pub fn total_discards(&self) -> u64 {
        self.total_discards
    }

    pub fn dirty_list(&self) -> Vec<usize> {
        self.regs.iter()
            .filter(|(_, r)| r.state == LatchState::Dirty)
            .map(|(&idx, _)| idx)
            .collect()
    }
}

impl Default for LatchRegisterFile {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_file() {
        let rf = LatchRegisterFile::new();
        assert_eq!(rf.reg_count(), 0);
    }

    #[test]
    fn add_read_write() {
        let mut rf = LatchRegisterFile::new();
        rf.add_reg(0, 0x100, true);
        assert_eq!(rf.read(0), Ok(0x100));
        rf.write(0, 0x200).unwrap();
        assert_eq!(rf.read(0), Ok(0x200));
    }

    #[test]
    fn dirty_state() {
        let mut rf = LatchRegisterFile::new();
        rf.add_reg(0, 0, true);
        assert_eq!(rf.state(0), Some(LatchState::Clean));
        rf.write(0, 42).unwrap();
        assert_eq!(rf.state(0), Some(LatchState::Dirty));
        assert_eq!(rf.dirty_count(), 1);
    }

    #[test]
    fn commit() {
        let mut rf = LatchRegisterFile::new();
        rf.add_reg(0, 0, true);
        rf.add_reg(1, 0, true);
        rf.write(0, 10).unwrap();
        rf.write(1, 20).unwrap();
        let count = rf.commit();
        assert_eq!(count, 2);
        assert_eq!(rf.state(0), Some(LatchState::Committed));
        assert_eq!(rf.shadow(0), Some(10));
    }

    #[test]
    fn discard() {
        let mut rf = LatchRegisterFile::new();
        rf.add_reg(0, 0xFF, true);
        rf.write(0, 0).unwrap();
        let count = rf.discard();
        assert_eq!(count, 1);
        assert_eq!(rf.read(0), Ok(0xFF));
        assert_eq!(rf.state(0), Some(LatchState::Clean));
    }

    #[test]
    fn read_only() {
        let mut rf = LatchRegisterFile::new();
        rf.add_reg(0, 42, false);
        let err = rf.write(0, 99).unwrap_err();
        assert!(matches!(err, LatchError::ReadOnly { reg: 0 }));
    }

    #[test]
    fn write_protect() {
        let mut rf = LatchRegisterFile::new();
        rf.add_reg(0, 0, true);
        rf.set_write_protect(true);
        let err = rf.write(0, 1).unwrap_err();
        assert!(matches!(err, LatchError::WriteProtected));
    }

    #[test]
    fn commit_one() {
        let mut rf = LatchRegisterFile::new();
        rf.add_reg(0, 0, true);
        rf.write(0, 55).unwrap();
        rf.commit_one(0).unwrap();
        assert_eq!(rf.total_commits(), 1);
    }

    #[test]
    fn discard_one_not_dirty() {
        let mut rf = LatchRegisterFile::new();
        rf.add_reg(0, 0, true);
        let err = rf.discard_one(0).unwrap_err();
        assert!(matches!(err, LatchError::NotDirty { .. }));
    }

    #[test]
    fn dirty_list() {
        let mut rf = LatchRegisterFile::new();
        rf.add_reg(0, 0, true);
        rf.add_reg(1, 0, true);
        rf.add_reg(2, 0, true);
        rf.write(1, 1).unwrap();
        rf.write(2, 2).unwrap();
        assert_eq!(rf.dirty_list(), vec![1, 2]);
    }

    #[test]
    fn same_value_not_dirty() {
        let mut rf = LatchRegisterFile::new();
        rf.add_reg(0, 42, true);
        rf.write(0, 42).unwrap();
        assert_eq!(rf.state(0), Some(LatchState::Clean));
    }

    #[test]
    fn error_display() {
        assert!(LatchError::Unknown { reg: 3 }.to_string().contains("3"));
    }
}

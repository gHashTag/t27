use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegAccess {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

#[derive(Debug, Clone)]
pub struct RegDef {
    pub name: String,
    pub offset: u32,
    pub reset_value: u32,
    pub access: RegAccess,
}

impl RegDef {
    pub fn rw(name: &str, offset: u32, reset: u32) -> Self {
        Self {
            name: name.to_string(),
            offset,
            reset_value: reset,
            access: RegAccess::ReadWrite,
        }
    }

    pub fn ro(name: &str, offset: u32, reset: u32) -> Self {
        Self {
            name: name.to_string(),
            offset,
            reset_value: reset,
            access: RegAccess::ReadOnly,
        }
    }

    pub fn wo(name: &str, offset: u32) -> Self {
        Self {
            name: name.to_string(),
            offset,
            reset_value: 0,
            access: RegAccess::WriteOnly,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegFileError {
    NotFound { offset: u32 },
    WriteToReadOnly { name: String },
    ReadFromWriteOnly { name: String },
}

impl std::fmt::Display for RegFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegFileError::NotFound { offset } => write!(f, "register not found: 0x{offset:X}"),
            RegFileError::WriteToReadOnly { name } => write!(f, "write to read-only: {name}"),
            RegFileError::ReadFromWriteOnly { name } => write!(f, "read from write-only: {name}"),
        }
    }
}

impl std::error::Error for RegFileError {}

#[derive(Debug, Clone)]
pub struct RegisterFile {
    defs: BTreeMap<u32, RegDef>,
    values: BTreeMap<u32, u32>,
    dirty: std::collections::HashSet<u32>,
    total_reads: u64,
    total_writes: u64,
}

impl RegisterFile {
    pub fn new() -> Self {
        Self {
            defs: BTreeMap::new(),
            values: BTreeMap::new(),
            dirty: std::collections::HashSet::new(),
            total_reads: 0,
            total_writes: 0,
        }
    }

    pub fn add(&mut self, def: RegDef) {
        self.values.insert(def.offset, def.reset_value);
        self.defs.insert(def.offset, def);
    }

    pub fn read(&mut self, offset: u32) -> Result<u32, RegFileError> {
        let def = self.defs.get(&offset).ok_or(RegFileError::NotFound { offset })?;
        if def.access == RegAccess::WriteOnly {
            return Err(RegFileError::ReadFromWriteOnly { name: def.name.clone() });
        }
        self.total_reads += 1;
        Ok(self.values[&offset])
    }

    pub fn write(&mut self, offset: u32, value: u32) -> Result<(), RegFileError> {
        let def = self.defs.get(&offset).ok_or(RegFileError::NotFound { offset })?;
        if def.access == RegAccess::ReadOnly {
            return Err(RegFileError::WriteToReadOnly { name: def.name.clone() });
        }
        self.values.insert(offset, value);
        self.dirty.insert(offset);
        self.total_writes += 1;
        Ok(())
    }

    pub fn get_def(&self, offset: u32) -> Option<&RegDef> {
        self.defs.get(&offset)
    }

    pub fn is_dirty(&self, offset: u32) -> bool {
        self.dirty.contains(&offset)
    }

    pub fn dirty_offsets(&self) -> Vec<u32> {
        let mut offsets: Vec<u32> = self.dirty.iter().copied().collect();
        offsets.sort();
        offsets
    }

    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    pub fn reset_all(&mut self) {
        for (offset, def) in &self.defs {
            self.values.insert(*offset, def.reset_value);
        }
        self.dirty.clear();
    }

    pub fn reg_count(&self) -> usize {
        self.defs.len()
    }

    pub fn total_reads(&self) -> u64 {
        self.total_reads
    }

    pub fn total_writes(&self) -> u64 {
        self.total_writes
    }

    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.defs.values().map(|d| d.name.as_str()).collect();
        names.sort();
        names
    }
}

impl Default for RegisterFile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_read() {
        let mut rf = RegisterFile::new();
        rf.add(RegDef::rw("ctrl", 0x00, 0x42));
        assert_eq!(rf.read(0x00).unwrap(), 0x42);
    }

    #[test]
    fn write_and_read() {
        let mut rf = RegisterFile::new();
        rf.add(RegDef::rw("ctrl", 0x00, 0));
        rf.write(0x00, 0xFF).unwrap();
        assert_eq!(rf.read(0x00).unwrap(), 0xFF);
        assert!(rf.is_dirty(0x00));
    }

    #[test]
    fn read_only_rejects_write() {
        let mut rf = RegisterFile::new();
        rf.add(RegDef::ro("id", 0x04, 0x1234));
        let err = rf.write(0x04, 0).unwrap_err();
        assert!(matches!(err, RegFileError::WriteToReadOnly { .. }));
    }

    #[test]
    fn write_only_rejects_read() {
        let mut rf = RegisterFile::new();
        rf.add(RegDef::wo("pulse", 0x08));
        let err = rf.read(0x08).unwrap_err();
        assert!(matches!(err, RegFileError::ReadFromWriteOnly { .. }));
    }

    #[test]
    fn not_found() {
        let mut rf = RegisterFile::new();
        let err = rf.read(0xFF).unwrap_err();
        assert!(matches!(err, RegFileError::NotFound { .. }));
    }

    #[test]
    fn dirty_tracking() {
        let mut rf = RegisterFile::new();
        rf.add(RegDef::rw("a", 0x00, 0));
        rf.add(RegDef::rw("b", 0x04, 0));
        rf.write(0x00, 1).unwrap();
        rf.write(0x04, 2).unwrap();
        assert_eq!(rf.dirty_offsets(), vec![0x00, 0x04]);
        rf.clear_dirty();
        assert!(rf.dirty_offsets().is_empty());
    }

    #[test]
    fn reset_all() {
        let mut rf = RegisterFile::new();
        rf.add(RegDef::rw("ctrl", 0x00, 0x42));
        rf.write(0x00, 0xFF).unwrap();
        rf.reset_all();
        assert_eq!(rf.read(0x00).unwrap(), 0x42);
        assert!(!rf.is_dirty(0x00));
    }

    #[test]
    fn get_def() {
        let mut rf = RegisterFile::new();
        rf.add(RegDef::rw("ctrl", 0x00, 0));
        let def = rf.get_def(0x00).unwrap();
        assert_eq!(def.name, "ctrl");
        assert_eq!(def.access, RegAccess::ReadWrite);
    }

    #[test]
    fn stats() {
        let mut rf = RegisterFile::new();
        rf.add(RegDef::rw("x", 0x00, 0));
        rf.read(0x00).unwrap();
        rf.read(0x00).unwrap();
        rf.write(0x00, 1).unwrap();
        assert_eq!(rf.total_reads(), 2);
        assert_eq!(rf.total_writes(), 1);
    }

    #[test]
    fn names_sorted() {
        let mut rf = RegisterFile::new();
        rf.add(RegDef::rw("bravo", 0x04, 0));
        rf.add(RegDef::rw("alpha", 0x00, 0));
        assert_eq!(rf.names(), vec!["alpha", "bravo"]);
    }

    #[test]
    fn error_display() {
        assert!(RegFileError::NotFound { offset: 0xFF }.to_string().contains("0xFF"));
        assert!(RegFileError::WriteToReadOnly { name: "x".into() }.to_string().contains("read-only"));
        assert!(RegFileError::ReadFromWriteOnly { name: "y".into() }.to_string().contains("write-only"));
    }
}

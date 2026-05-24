#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapError {
    BufferFull { capacity: usize },
    NoCheckpoints,
    CheckpointNotFound { id: u64 },
}

impl std::fmt::Display for SnapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapError::BufferFull { capacity } => write!(f, "buffer full ({capacity})"),
            SnapError::NoCheckpoints => write!(f, "no checkpoints"),
            SnapError::CheckpointNotFound { id } => write!(f, "checkpoint {id} not found"),
        }
    }
}

impl std::error::Error for SnapError {}

#[derive(Debug, Clone)]
struct Checkpoint {
    id: u64,
    offset: usize,
}

#[derive(Debug, Clone)]
pub struct SnapshotBuffer {
    data: Vec<u8>,
    capacity: usize,
    checkpoints: Vec<Checkpoint>,
    next_cp_id: u64,
    total_appends: u64,
    total_replays: u64,
    total_bytes_written: u64,
}

impl SnapshotBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
            checkpoints: Vec::new(),
            next_cp_id: 1,
            total_appends: 0,
            total_replays: 0,
            total_bytes_written: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn remaining(&self) -> usize {
        self.capacity - self.data.len()
    }

    pub fn checkpoint(&mut self) -> u64 {
        let id = self.next_cp_id;
        self.next_cp_id += 1;
        self.checkpoints.push(Checkpoint { id, offset: self.data.len() });
        id
    }

    pub fn append(&mut self, bytes: &[u8]) -> Result<(), SnapError> {
        if self.data.len() + bytes.len() > self.capacity {
            return Err(SnapError::BufferFull { capacity: self.capacity });
        }
        self.data.extend_from_slice(bytes);
        self.total_appends += 1;
        self.total_bytes_written += bytes.len() as u64;
        Ok(())
    }

    pub fn replay_from(&mut self, checkpoint_id: u64) -> Result<Vec<u8>, SnapError> {
        let cp = self.checkpoints.iter().find(|c| c.id == checkpoint_id)
            .ok_or(SnapError::CheckpointNotFound { id: checkpoint_id })?;
        let result = self.data[cp.offset..].to_vec();
        self.total_replays += 1;
        Ok(result)
    }

    pub fn replay_last(&self) -> Result<Vec<u8>, SnapError> {
        let cp = self.checkpoints.last().ok_or(SnapError::NoCheckpoints)?;
        Ok(self.data[cp.offset..].to_vec())
    }

    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }

    pub fn last_checkpoint_id(&self) -> Option<u64> {
        self.checkpoints.last().map(|c| c.id)
    }

    pub fn checkpoint_offset(&self, id: u64) -> Option<usize> {
        self.checkpoints.iter().find(|c| c.id == id).map(|c| c.offset)
    }

    pub fn rollback(&mut self, checkpoint_id: u64) -> Result<usize, SnapError> {
        let cp = self.checkpoints.iter().find(|c| c.id == checkpoint_id)
            .ok_or(SnapError::CheckpointNotFound { id: checkpoint_id })?;
        let removed = self.data.len() - cp.offset;
        self.data.truncate(cp.offset);
        self.checkpoints.retain(|c| c.id <= checkpoint_id);
        Ok(removed)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn total_appends(&self) -> u64 { self.total_appends }
    pub fn total_replays(&self) -> u64 { self.total_replays }
    pub fn total_bytes_written(&self) -> u64 { self.total_bytes_written }

    pub fn clear(&mut self) {
        self.data.clear();
        self.checkpoints.clear();
        self.next_cp_id = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer() {
        let sb = SnapshotBuffer::new(1024);
        assert_eq!(sb.capacity(), 1024);
        assert!(sb.is_empty());
    }

    #[test]
    fn append_and_len() {
        let mut sb = SnapshotBuffer::new(1024);
        sb.append(&[1, 2, 3]).unwrap();
        sb.append(&[4, 5, 6]).unwrap();
        assert_eq!(sb.len(), 6);
        assert_eq!(sb.total_appends(), 2);
    }

    #[test]
    fn buffer_full() {
        let mut sb = SnapshotBuffer::new(4);
        sb.append(&[1, 2, 3]).unwrap();
        let err = sb.append(&[4, 5]).unwrap_err();
        assert!(matches!(err, SnapError::BufferFull { .. }));
    }

    #[test]
    fn checkpoint_and_replay() {
        let mut sb = SnapshotBuffer::new(1024);
        sb.append(&[1, 2, 3]).unwrap();
        let cp = sb.checkpoint();
        sb.append(&[4, 5, 6]).unwrap();
        let data = sb.replay_from(cp).unwrap();
        assert_eq!(data, vec![4, 5, 6]);
    }

    #[test]
    fn replay_last() {
        let mut sb = SnapshotBuffer::new(1024);
        sb.checkpoint();
        sb.append(&[10, 20]).unwrap();
        let data = sb.replay_last().unwrap();
        assert_eq!(data, vec![10, 20]);
    }

    #[test]
    fn no_checkpoints() {
        let sb: SnapshotBuffer = SnapshotBuffer::new(1024);
        let err = sb.replay_last().unwrap_err();
        assert!(matches!(err, SnapError::NoCheckpoints));
    }

    #[test]
    fn checkpoint_not_found() {
        let mut sb = SnapshotBuffer::new(1024);
        sb.checkpoint();
        let err = sb.replay_from(99).unwrap_err();
        assert!(matches!(err, SnapError::CheckpointNotFound { .. }));
    }

    #[test]
    fn rollback() {
        let mut sb = SnapshotBuffer::new(1024);
        sb.append(&[1, 2, 3]).unwrap();
        let cp = sb.checkpoint();
        sb.append(&[4, 5, 6]).unwrap();
        let removed = sb.rollback(cp).unwrap();
        assert_eq!(removed, 3);
        assert_eq!(sb.len(), 3);
    }

    #[test]
    fn multiple_checkpoints() {
        let mut sb = SnapshotBuffer::new(1024);
        sb.append(&[1]).unwrap();
        let cp1 = sb.checkpoint();
        sb.append(&[2]).unwrap();
        let cp2 = sb.checkpoint();
        sb.append(&[3]).unwrap();
        assert_eq!(sb.replay_from(cp1).unwrap(), vec![2, 3]);
        assert_eq!(sb.replay_from(cp2).unwrap(), vec![3]);
        assert_eq!(sb.checkpoint_count(), 2);
    }

    #[test]
    fn as_slice() {
        let mut sb = SnapshotBuffer::new(1024);
        sb.append(&[0xDE, 0xAD]).unwrap();
        assert_eq!(sb.as_slice(), &[0xDE, 0xAD]);
    }

    #[test]
    fn clear() {
        let mut sb = SnapshotBuffer::new(1024);
        sb.append(&[1]).unwrap();
        sb.checkpoint();
        sb.clear();
        assert!(sb.is_empty());
        assert_eq!(sb.checkpoint_count(), 0);
    }

    #[test]
    fn error_display() {
        assert!(SnapError::BufferFull { capacity: 64 }.to_string().contains("64"));
    }
}

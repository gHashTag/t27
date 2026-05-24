use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum IfError {
    RequestExists { id: u64 },
    RequestNotFound { id: u64 },
    AlreadyCompleted { id: u64 },
}

impl std::fmt::Display for IfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IfError::RequestExists { id } => write!(f, "request {id} exists"),
            IfError::RequestNotFound { id } => write!(f, "request {id} not found"),
            IfError::AlreadyCompleted { id } => write!(f, "request {id} completed"),
        }
    }
}

impl std::error::Error for IfError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ReqState { Inflight, Completed, TimedOut }

struct Request {
    id: u64,
    payload: Vec<u8>,
    state: ReqState,
    sent_at: u64,
    deadline: u64,
    retry_count: u32,
    max_retries: u32,
    response: Option<Vec<u8>>,
    dedup_key: Option<Vec<u8>>,
}

pub struct InflightMap {
    requests: BTreeMap<u64, Request>,
    by_dedup: BTreeMap<Vec<u8>, u64>,
    next_id: u64,
    total_sent: u64,
    total_completed: u64,
    total_timed_out: u64,
    total_retries: u64,
    total_dedup_hits: u64,
}

impl InflightMap {
    pub fn new() -> Self { Self { requests: BTreeMap::new(), by_dedup: BTreeMap::new(), next_id: 1, total_sent: 0, total_completed: 0, total_timed_out: 0, total_retries: 0, total_dedup_hits: 0 } }

    pub fn send(&mut self, payload: Vec<u8>, sent_at: u64, timeout: u64, max_retries: u32, dedup_key: Option<Vec<u8>>) -> Result<u64, IfError> {
        if let Some(ref dk) = dedup_key {
            if let Some(&existing_id) = self.by_dedup.get(dk) {
                if let Some(req) = self.requests.get(&existing_id) {
                    if req.state == ReqState::Inflight {
                        self.total_dedup_hits += 1;
                        return Ok(existing_id);
                    }
                }
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        let deadline = sent_at + timeout;
        self.requests.insert(id, Request { id, payload, state: ReqState::Inflight, sent_at, deadline, retry_count: 0, max_retries, response: None, dedup_key: dedup_key.clone() });
        if let Some(dk) = dedup_key { self.by_dedup.insert(dk, id); }
        self.total_sent += 1;
        Ok(id)
    }

    pub fn complete(&mut self, id: u64, response: Vec<u8>) -> Result<(), IfError> {
        let req = self.requests.get_mut(&id).ok_or(IfError::RequestNotFound { id })?;
        if req.state != ReqState::Inflight { return Err(IfError::AlreadyCompleted { id }); }
        req.state = ReqState::Completed;
        req.response = Some(response);
        self.total_completed += 1;
        Ok(())
    }

    pub fn tick(&mut self, now: u64) -> Vec<u64> {
        let mut timed_out = Vec::new();
        for (&id, req) in &self.requests {
            if req.state == ReqState::Inflight && now >= req.deadline {
                timed_out.push(id);
            }
        }
        for id in &timed_out {
            let req = self.requests.get_mut(id).unwrap();
            if req.retry_count < req.max_retries {
                req.retry_count += 1;
                req.deadline = now + (req.deadline - req.sent_at);
                self.total_retries += 1;
            } else {
                req.state = ReqState::TimedOut;
                self.total_timed_out += 1;
            }
        }
        timed_out
    }

    pub fn state(&self, id: u64) -> Option<&ReqState> { self.requests.get(&id).map(|r| &r.state) }
    pub fn response(&self, id: u64) -> Option<&Vec<u8>> { self.requests.get(&id).and_then(|r| r.response.as_ref()) }
    pub fn retry_count(&self, id: u64) -> Option<u32> { self.requests.get(&id).map(|r| r.retry_count) }
    pub fn inflight_count(&self) -> usize { self.requests.values().filter(|r| r.state == ReqState::Inflight).count() }
    pub fn request_count(&self) -> usize { self.requests.len() }
    pub fn total_sent(&self) -> u64 { self.total_sent }
    pub fn total_completed(&self) -> u64 { self.total_completed }
    pub fn total_timed_out(&self) -> u64 { self.total_timed_out }
    pub fn total_retries(&self) -> u64 { self.total_retries }
    pub fn total_dedup_hits(&self) -> u64 { self.total_dedup_hits }
}

impl Default for InflightMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() { assert_eq!(InflightMap::new().request_count(), 0); }

    #[test]
    fn send_complete() {
        let mut im = InflightMap::new();
        let id = im.send(b"req".to_vec(), 0, 100, 0, None).unwrap();
        im.complete(id, b"resp".to_vec()).unwrap();
        assert_eq!(im.state(id), Some(&ReqState::Completed));
        assert_eq!(im.response(id), Some(&b"resp".to_vec()));
    }

    #[test]
    fn timeout() {
        let mut im = InflightMap::new();
        im.send(b"req".to_vec(), 0, 10, 0, None).unwrap();
        let timed = im.tick(20);
        assert_eq!(timed.len(), 1);
        assert_eq!(im.state(1), Some(&ReqState::TimedOut));
    }

    #[test]
    fn retry_before_timeout() {
        let mut im = InflightMap::new();
        im.send(b"req".to_vec(), 0, 10, 2, None).unwrap();
        let timed = im.tick(15);
        assert_eq!(timed.len(), 1);
        assert_eq!(im.state(1), Some(&ReqState::Inflight));
        assert_eq!(im.retry_count(1), Some(1));
    }

    #[test]
    fn max_retries_exhausted() {
        let mut im = InflightMap::new();
        im.send(b"req".to_vec(), 0, 10, 1, None).unwrap();
        im.tick(15);
        im.tick(30);
        assert_eq!(im.state(1), Some(&ReqState::TimedOut));
    }

    #[test]
    fn dedup() {
        let mut im = InflightMap::new();
        let id1 = im.send(b"req".to_vec(), 0, 100, 0, Some(b"key1".to_vec())).unwrap();
        let id2 = im.send(b"req".to_vec(), 0, 100, 0, Some(b"key1".to_vec())).unwrap();
        assert_eq!(id1, id2);
        assert!(im.total_dedup_hits() > 0);
    }

    #[test]
    fn complete_removes_dedup() {
        let mut im = InflightMap::new();
        let id1 = im.send(b"req".to_vec(), 0, 100, 0, Some(b"k".to_vec())).unwrap();
        im.complete(id1, b"r".to_vec()).unwrap();
        let id2 = im.send(b"req".to_vec(), 0, 100, 0, Some(b"k".to_vec())).unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn not_found() {
        let mut im = InflightMap::new();
        let err = im.complete(99, b"r".to_vec()).unwrap_err();
        assert!(matches!(err, IfError::RequestNotFound { .. }));
    }

    #[test]
    fn double_complete() {
        let mut im = InflightMap::new();
        let id = im.send(b"r".to_vec(), 0, 100, 0, None).unwrap();
        im.complete(id, b"r".to_vec()).unwrap();
        let err = im.complete(id, b"r".to_vec()).unwrap_err();
        assert!(matches!(err, IfError::AlreadyCompleted { .. }));
    }

    #[test]
    fn stats() {
        let mut im = InflightMap::new();
        let id = im.send(b"r".to_vec(), 0, 100, 0, None).unwrap();
        im.complete(id, b"r".to_vec()).unwrap();
        assert_eq!(im.total_sent(), 1);
        assert_eq!(im.total_completed(), 1);
    }

    #[test]
    fn error_display() { assert!(IfError::RequestNotFound { id: 3 }.to_string().contains("3")); }
}

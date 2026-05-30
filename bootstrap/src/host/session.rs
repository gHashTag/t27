use super::protocol::{Cmd, CmdPacket, RespCode, RespPacket};
use super::transport::TransportFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionError {
    SeqOverflow,
    MaxRetriesExceeded { retries: u8 },
    UnexpectedSeq { expected: u8, got: u8 },
    TransportError(super::transport::TransportError),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::SeqOverflow => write!(f, "sequence number overflow"),
            SessionError::MaxRetriesExceeded { retries } => {
                write!(f, "max retries exceeded: {retries}")
            }
            SessionError::UnexpectedSeq { expected, got } => {
                write!(f, "unexpected seq: expected {expected}, got {got}")
            }
            SessionError::TransportError(e) => write!(f, "transport error: {e}"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<super::transport::TransportError> for SessionError {
    fn from(e: super::transport::TransportError) -> Self {
        SessionError::TransportError(e)
    }
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub max_retries: u8,
    pub cmd_timeout_ms: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            cmd_timeout_ms: 1000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub seq: u8,
    pub cmd: Cmd,
    pub frame: Vec<u8>,
    pub retries: u8,
}

#[derive(Debug, Clone)]
pub struct Session {
    config: SessionConfig,
    next_seq: u8,
    pending: Option<PendingRequest>,
}

impl Session {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            next_seq: 0,
            pending: None,
        }
    }

    pub fn next_seq(&self) -> u8 {
        self.next_seq
    }

    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub fn pending_cmd(&self) -> Option<Cmd> {
        self.pending.as_ref().map(|p| p.cmd)
    }

    pub fn build_request(&mut self, cmd: Cmd, payload: &[u8]) -> Result<Vec<u8>, SessionError> {
        let seq = self.next_seq;
        let pkt = CmdPacket::new(cmd)
            .with_seq(seq)
            .with_payload_len(payload.len() as u16);
        let frame = TransportFrame::new(pkt, payload)?;
        let encoded = frame.encode();
        self.pending = Some(PendingRequest {
            seq,
            cmd,
            frame: encoded.clone(),
            retries: 0,
        });
        self.advance_seq()?;
        Ok(encoded)
    }

    fn advance_seq(&mut self) -> Result<(), SessionError> {
        let (next, overflowed) = self.next_seq.overflowing_add(1);
        if overflowed {
            return Err(SessionError::SeqOverflow);
        }
        self.next_seq = next;
        Ok(())
    }

    pub fn handle_response(&mut self, resp_frame: &[u8]) -> Result<RespPacket, SessionError> {
        let decoded = super::transport::RespFrame::decode(resp_frame)?;
        let resp = decoded.header;
        if let Some(ref pending) = self.pending {
            if resp.seq != pending.seq {
                return Err(SessionError::UnexpectedSeq {
                    expected: pending.seq,
                    got: resp.seq,
                });
            }
        }
        if resp.code == RespCode::ErrBusy {
            if let Some(ref mut p) = self.pending {
                p.retries += 1;
            }
        }
        if resp.code.is_ok() || resp.code != RespCode::ErrBusy {
            self.pending = None;
        }
        Ok(resp)
    }

    pub fn should_retry(&self) -> bool {
        match &self.pending {
            Some(p) => p.retries < self.config.max_retries,
            None => false,
        }
    }

    pub fn retry_frame(&self) -> Option<&[u8]> {
        self.pending.as_ref().map(|p| p.frame.as_slice())
    }

    pub fn reset(&mut self) {
        self.next_seq = 0;
        self.pending = None;
    }

    pub fn stats(&self) -> SessionStats {
        SessionStats {
            next_seq: self.next_seq,
            has_pending: self.pending.is_some(),
            pending_retries: self.pending.as_ref().map_or(0, |p| p.retries),
            max_retries: self.config.max_retries,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionStats {
    pub next_seq: u8,
    pub has_pending: bool,
    pub pending_retries: u8,
    pub max_retries: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::transport::RespFrame;

    fn make_session() -> Session {
        Session::new(SessionConfig::default())
    }

    fn make_resp(seq: u8, code: RespCode) -> Vec<u8> {
        RespFrame::new(RespPacket::new(code).with_seq(seq), &[]).encode()
    }

    #[test]
    fn new_session_seq_zero() {
        let s = make_session();
        assert_eq!(s.next_seq(), 0);
        assert!(!s.has_pending());
    }

    #[test]
    fn build_request_advances_seq() {
        let mut s = make_session();
        s.build_request(Cmd::Reset, &[]).unwrap();
        assert_eq!(s.next_seq(), 1);
        assert!(s.has_pending());
    }

    #[test]
    fn build_request_with_payload() {
        let mut s = make_session();
        let frame = s.build_request(Cmd::LoadWeights, &[0xAA, 0xBB]).unwrap();
        assert!(!frame.is_empty());
        assert_eq!(s.pending_cmd(), Some(Cmd::LoadWeights));
    }

    #[test]
    fn handle_response_ok_clears_pending() {
        let mut s = make_session();
        s.build_request(Cmd::Reset, &[]).unwrap();
        let resp = make_resp(0, RespCode::Ok);
        let pkt = s.handle_response(&resp).unwrap();
        assert_eq!(pkt.code, RespCode::Ok);
        assert!(!s.has_pending());
    }

    #[test]
    fn handle_response_wrong_seq() {
        let mut s = make_session();
        s.build_request(Cmd::Reset, &[]).unwrap();
        let resp = make_resp(99, RespCode::Ok);
        let err = s.handle_response(&resp).unwrap_err();
        assert!(matches!(
            err,
            SessionError::UnexpectedSeq {
                expected: 0,
                got: 99
            }
        ));
    }

    #[test]
    fn handle_response_busy_increments_retry() {
        let mut s = make_session();
        s.build_request(Cmd::RunInference, &[]).unwrap();
        let resp = make_resp(0, RespCode::ErrBusy);
        s.handle_response(&resp).unwrap();
        assert!(s.has_pending());
        assert!(s.should_retry());
        assert_eq!(s.stats().pending_retries, 1);
    }

    #[test]
    fn should_retry_respects_max() {
        let mut s = make_session();
        s.build_request(Cmd::RunInference, &[]).unwrap();
        for _ in 0..3 {
            let resp = make_resp(0, RespCode::ErrBusy);
            s.handle_response(&resp).unwrap();
        }
        assert!(!s.should_retry());
    }

    #[test]
    fn retry_frame_returns_encoded() {
        let mut s = make_session();
        let frame = s.build_request(Cmd::Reset, &[]).unwrap();
        assert_eq!(s.retry_frame(), Some(frame.as_slice()));
    }

    #[test]
    fn reset_clears_state() {
        let mut s = make_session();
        s.build_request(Cmd::Reset, &[]).unwrap();
        s.reset();
        assert_eq!(s.next_seq(), 0);
        assert!(!s.has_pending());
    }

    #[test]
    fn stats_reflects_state() {
        let mut s = make_session();
        let stats = s.stats();
        assert_eq!(stats.next_seq, 0);
        assert!(!stats.has_pending);
        assert_eq!(stats.pending_retries, 0);
        assert_eq!(stats.max_retries, 3);
    }

    #[test]
    fn multiple_requests_sequential_seq() {
        let mut s = make_session();
        s.build_request(Cmd::Reset, &[]).unwrap();
        let resp = make_resp(0, RespCode::Ok);
        s.handle_response(&resp).unwrap();
        s.build_request(Cmd::ReadStatus, &[]).unwrap();
        assert_eq!(s.next_seq(), 2);
        let resp = make_resp(1, RespCode::Ok);
        s.handle_response(&resp).unwrap();
    }

    #[test]
    fn custom_max_retries() {
        let s = Session::new(SessionConfig {
            max_retries: 5,
            cmd_timeout_ms: 2000,
        });
        assert_eq!(s.config.max_retries, 5);
    }

    #[test]
    fn error_display() {
        let e = SessionError::SeqOverflow;
        assert!(e.to_string().contains("overflow"));
        let e = SessionError::MaxRetriesExceeded { retries: 3 };
        assert!(e.to_string().contains("3"));
        let e = SessionError::UnexpectedSeq { expected: 1, got: 2 };
        assert!(e.to_string().contains("expected"));
    }

    #[test]
    fn handle_response_err_crc_clears_pending() {
        let mut s = make_session();
        s.build_request(Cmd::LoadWeights, &[]).unwrap();
        let resp = make_resp(0, RespCode::ErrCrc);
        let pkt = s.handle_response(&resp).unwrap();
        assert_eq!(pkt.code, RespCode::ErrCrc);
        assert!(!s.has_pending());
    }
}

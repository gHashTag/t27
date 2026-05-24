#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RespStatus {
    Ok = 0x00,
    Busy = 0x01,
    Error = 0x02,
    InvalidCommand = 0x03,
    Timeout = 0x04,
    CrcMismatch = 0x05,
}

impl RespStatus {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(RespStatus::Ok),
            0x01 => Some(RespStatus::Busy),
            0x02 => Some(RespStatus::Error),
            0x03 => Some(RespStatus::InvalidCommand),
            0x04 => Some(RespStatus::Timeout),
            0x05 => Some(RespStatus::CrcMismatch),
            _ => None,
        }
    }

    pub fn is_ok(&self) -> bool {
        *self == RespStatus::Ok
    }

    pub fn is_error(&self) -> bool {
        *self != RespStatus::Ok
    }
}

impl std::fmt::Display for RespStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RespStatus::Ok => "ok",
            RespStatus::Busy => "busy",
            RespStatus::Error => "error",
            RespStatus::InvalidCommand => "invalid_command",
            RespStatus::Timeout => "timeout",
            RespStatus::CrcMismatch => "crc_mismatch",
        };
        write!(f, "{s}")
    }
}

const HEADER_SIZE: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: RespStatus,
    pub seq: u16,
    pub payload: Vec<u8>,
}

impl Response {
    pub fn new(status: RespStatus) -> Self {
        Self {
            status,
            seq: 0,
            payload: Vec::new(),
        }
    }

    pub fn ok() -> Self {
        Self::new(RespStatus::Ok)
    }

    pub fn error() -> Self {
        Self::new(RespStatus::Error)
    }

    pub fn with_seq(mut self, seq: u16) -> Self {
        self.seq = seq;
        self
    }

    pub fn with_payload(mut self, data: Vec<u8>) -> Self {
        self.payload = data;
        self
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(HEADER_SIZE + self.payload.len());
        buf.push(0x5A);
        buf.push(self.status as u8);
        buf.extend_from_slice(&self.seq.to_le_bytes());
        buf.extend_from_slice(&(self.payload.len() as u16).to_le_bytes());
        buf.extend_from_slice(&self.payload);
        let mut crc: u16 = 0xFFFF;
        for &b in &buf {
            crc ^= b as u16;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0x8408;
                } else {
                    crc >>= 1;
                }
            }
        }
        crc = !crc;
        buf.extend_from_slice(&crc.to_le_bytes());
        buf
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RespParseError {
    TooShort { got: usize, need: usize },
    BadMagic { got: u8 },
    UnknownStatus { got: u8 },
    CrcMismatch { expected: u16, got: u16 },
}

impl std::fmt::Display for RespParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RespParseError::TooShort { got, need } => write!(f, "too short: {got}/{need}"),
            RespParseError::BadMagic { got } => write!(f, "bad magic: 0x{got:02X}"),
            RespParseError::UnknownStatus { got } => write!(f, "unknown status: 0x{got:02X}"),
            RespParseError::CrcMismatch { expected, got } => {
                write!(f, "crc: expected 0x{expected:04X}, got 0x{got:04X}")
            }
        }
    }
}

impl std::error::Error for RespParseError {}

pub fn parse_response(data: &[u8]) -> Result<Response, RespParseError> {
    if data.len() < HEADER_SIZE + 2 {
        return Err(RespParseError::TooShort { got: data.len(), need: HEADER_SIZE + 2 });
    }
    if data[0] != 0x5A {
        return Err(RespParseError::BadMagic { got: data[0] });
    }
    let status = RespStatus::from_byte(data[1]).ok_or(RespParseError::UnknownStatus { got: data[1] })?;
    let seq = u16::from_le_bytes([data[2], data[3]]);
    let payload_len = u16::from_le_bytes([data[4], data[5]]) as usize;
    if data.len() < HEADER_SIZE + payload_len + 2 {
        return Err(RespParseError::TooShort { got: data.len(), need: HEADER_SIZE + payload_len + 2 });
    }
    let payload = data[HEADER_SIZE..HEADER_SIZE + payload_len].to_vec();
    let got_crc = u16::from_le_bytes([data[HEADER_SIZE + payload_len], data[HEADER_SIZE + payload_len + 1]]);
    let mut crc: u16 = 0xFFFF;
    for &b in &data[..HEADER_SIZE + payload_len] {
        crc ^= b as u16;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0x8408;
            } else {
                crc >>= 1;
            }
        }
    }
    let expected_crc = !crc;
    if got_crc != expected_crc {
        return Err(RespParseError::CrcMismatch { expected: expected_crc, got: got_crc });
    }
    Ok(Response { status, seq, payload })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_from_byte() {
        assert_eq!(RespStatus::from_byte(0x00), Some(RespStatus::Ok));
        assert_eq!(RespStatus::from_byte(0xFF), None);
    }

    #[test]
    fn status_predicates() {
        assert!(RespStatus::Ok.is_ok());
        assert!(!RespStatus::Ok.is_error());
        assert!(RespStatus::Error.is_error());
    }

    #[test]
    fn status_display() {
        assert_eq!(RespStatus::Ok.to_string(), "ok");
        assert_eq!(RespStatus::InvalidCommand.to_string(), "invalid_command");
    }

    #[test]
    fn response_builder() {
        let r = Response::ok().with_seq(42).with_payload(vec![1, 2, 3]);
        assert_eq!(r.status, RespStatus::Ok);
        assert_eq!(r.seq, 42);
        assert_eq!(r.payload, vec![1, 2, 3]);
    }

    #[test]
    fn encode_decode_roundtrip() {
        let r = Response::ok().with_seq(10).with_payload(vec![0xAA, 0xBB]);
        let encoded = r.encode();
        let decoded = parse_response(&encoded).unwrap();
        assert_eq!(decoded.status, RespStatus::Ok);
        assert_eq!(decoded.seq, 10);
        assert_eq!(decoded.payload, vec![0xAA, 0xBB]);
    }

    #[test]
    fn encode_decode_empty_payload() {
        let r = Response::ok().with_seq(0);
        let encoded = r.encode();
        let decoded = parse_response(&encoded).unwrap();
        assert!(decoded.payload.is_empty());
    }

    #[test]
    fn parse_bad_magic() {
        let mut buf = vec![0; HEADER_SIZE + 2];
        buf[0] = 0xFF;
        let err = parse_response(&buf).unwrap_err();
        assert!(matches!(err, RespParseError::BadMagic { .. }));
    }

    #[test]
    fn parse_too_short() {
        let err = parse_response(&[0x5A]).unwrap_err();
        assert!(matches!(err, RespParseError::TooShort { .. }));
    }

    #[test]
    fn parse_crc_mismatch() {
        let r = Response::ok();
        let mut encoded = r.encode();
        let len = encoded.len();
        encoded[len - 1] ^= 0xFF;
        let err = parse_response(&encoded).unwrap_err();
        assert!(matches!(err, RespParseError::CrcMismatch { .. }));
    }

    #[test]
    fn error_response() {
        let r = Response::error().with_seq(1);
        let encoded = r.encode();
        let decoded = parse_response(&encoded).unwrap();
        assert_eq!(decoded.status, RespStatus::Error);
        assert!(decoded.status.is_error());
    }
}

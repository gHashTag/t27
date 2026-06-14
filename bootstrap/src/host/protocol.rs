// Variant V (#969 dead-code audit, host/protocol.rs layer). This module is a
// self-contained, intentional public API: the host/device wire protocol (Cmd
// and RespCode enums with from_u8/has_payload/expects_response/is_ok, the
// CMD_HEADER_SIZE / RESP_HEADER_SIZE / PROTOCOL_VERSION constants, CmdPacket and
// RespPacket with their encode/decode/builder methods, and ProtocolError). It
// is fully exercised by this module's own test suite (opcode round-trip,
// header sizing, packet encode/decode, error paths) but is not yet wired into
// production host code, so every symbol emits a `dead_code` warning (12 in
// total) in the non-test build. These are deliberate public surface, not dead
// code -- a single module-scoped allow documents that without removing or
// weakening any symbol. Scoped to `not(test)` so the test build still flags
// genuinely unused items, exactly as in the #1105 / #1111 / #1129 slices of
// this audit (same pattern as the host/errors.rs slice in #1125).
#![cfg_attr(not(test), allow(dead_code))]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Cmd {
    Nop = 0x00,
    Reset = 0x01,
    LoadWeights = 0x10,
    RunInference = 0x20,
    ReadStatus = 0x30,
    ReadResult = 0x31,
    SetConfig = 0x40,
    SelfTest = 0xF0,
}

impl Cmd {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x00 => Some(Cmd::Nop),
            0x01 => Some(Cmd::Reset),
            0x10 => Some(Cmd::LoadWeights),
            0x20 => Some(Cmd::RunInference),
            0x30 => Some(Cmd::ReadStatus),
            0x31 => Some(Cmd::ReadResult),
            0x40 => Some(Cmd::SetConfig),
            0xF0 => Some(Cmd::SelfTest),
            _ => None,
        }
    }

    pub fn has_payload(self) -> bool {
        matches!(self, Cmd::LoadWeights | Cmd::SetConfig)
    }

    pub fn expects_response(self) -> bool {
        !matches!(self, Cmd::Nop)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RespCode {
    Ok = 0x00,
    ErrCrc = 0x01,
    ErrTimeout = 0x02,
    ErrBusy = 0x03,
    ErrInvalid = 0x04,
    ErrOverflow = 0x05,
}

impl RespCode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x00 => Some(RespCode::Ok),
            0x01 => Some(RespCode::ErrCrc),
            0x02 => Some(RespCode::ErrTimeout),
            0x03 => Some(RespCode::ErrBusy),
            0x04 => Some(RespCode::ErrInvalid),
            0x05 => Some(RespCode::ErrOverflow),
            _ => None,
        }
    }

    pub fn is_ok(self) -> bool {
        self == RespCode::Ok
    }
}

pub const CMD_HEADER_SIZE: usize = 8;
pub const RESP_HEADER_SIZE: usize = 4;
pub const PROTOCOL_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdPacket {
    pub cmd: Cmd,
    pub seq: u8,
    pub payload_len: u16,
    pub tag: u32,
}

impl CmdPacket {
    pub fn new(cmd: Cmd) -> Self {
        Self {
            cmd,
            seq: 0,
            payload_len: 0,
            tag: 0,
        }
    }

    pub fn with_seq(mut self, seq: u8) -> Self {
        self.seq = seq;
        self
    }

    pub fn with_payload_len(mut self, len: u16) -> Self {
        self.payload_len = len;
        self
    }

    pub fn with_tag(mut self, tag: u32) -> Self {
        self.tag = tag;
        self
    }

    pub fn encode(&self) -> [u8; CMD_HEADER_SIZE] {
        let mut buf = [0u8; CMD_HEADER_SIZE];
        buf[0] = PROTOCOL_VERSION;
        buf[1] = self.cmd as u8;
        buf[2] = self.seq;
        buf[3] = 0;
        buf[4..6].copy_from_slice(&self.payload_len.to_le_bytes());
        buf[6..8].copy_from_slice(&(self.tag as u16).to_le_bytes());
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < CMD_HEADER_SIZE {
            return Err(ProtocolError::HeaderTooShort {
                have: data.len(),
                need: CMD_HEADER_SIZE,
            });
        }
        let version = data[0];
        if version != PROTOCOL_VERSION {
            return Err(ProtocolError::BadVersion(version));
        }
        let cmd = Cmd::from_u8(data[1]).ok_or(ProtocolError::BadCommand(data[1]))?;
        let seq = data[2];
        let payload_len = u16::from_le_bytes([data[4], data[5]]);
        let tag = u16::from_le_bytes([data[6], data[7]]) as u32;
        Ok(Self {
            cmd,
            seq,
            payload_len,
            tag,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespPacket {
    pub code: RespCode,
    pub seq: u8,
    pub payload_len: u16,
}

impl RespPacket {
    pub fn new(code: RespCode) -> Self {
        Self {
            code,
            seq: 0,
            payload_len: 0,
        }
    }

    pub fn with_seq(mut self, seq: u8) -> Self {
        self.seq = seq;
        self
    }

    pub fn with_payload_len(mut self, len: u16) -> Self {
        self.payload_len = len;
        self
    }

    pub fn encode(&self) -> [u8; RESP_HEADER_SIZE] {
        let mut buf = [0u8; RESP_HEADER_SIZE];
        buf[0] = self.code as u8;
        buf[1] = self.seq;
        buf[2..4].copy_from_slice(&self.payload_len.to_le_bytes());
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.len() < RESP_HEADER_SIZE {
            return Err(ProtocolError::HeaderTooShort {
                have: data.len(),
                need: RESP_HEADER_SIZE,
            });
        }
        let code = RespCode::from_u8(data[0]).ok_or(ProtocolError::BadResponse(data[0]))?;
        let seq = data[1];
        let payload_len = u16::from_le_bytes([data[2], data[3]]);
        Ok(Self {
            code,
            seq,
            payload_len,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolError {
    HeaderTooShort { have: usize, need: usize },
    BadVersion(u8),
    BadCommand(u8),
    BadResponse(u8),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::HeaderTooShort { have, need } => {
                write!(f, "header too short: {have} < {need}")
            }
            ProtocolError::BadVersion(v) => write!(f, "bad protocol version: {v}"),
            ProtocolError::BadCommand(c) => write!(f, "bad command: 0x{c:02X}"),
            ProtocolError::BadResponse(c) => write!(f, "bad response code: 0x{c:02X}"),
        }
    }
}

impl std::error::Error for ProtocolError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_from_u8_roundtrip() {
        let cmds = [
            Cmd::Nop,
            Cmd::Reset,
            Cmd::LoadWeights,
            Cmd::RunInference,
            Cmd::ReadStatus,
            Cmd::ReadResult,
            Cmd::SetConfig,
            Cmd::SelfTest,
        ];
        for c in cmds {
            assert_eq!(Cmd::from_u8(c as u8), Some(c));
        }
    }

    #[test]
    fn cmd_from_u8_unknown() {
        assert_eq!(Cmd::from_u8(0xFF), None);
    }

    #[test]
    fn cmd_has_payload() {
        assert!(Cmd::LoadWeights.has_payload());
        assert!(Cmd::SetConfig.has_payload());
        assert!(!Cmd::Nop.has_payload());
        assert!(!Cmd::RunInference.has_payload());
    }

    #[test]
    fn cmd_expects_response() {
        assert!(!Cmd::Nop.expects_response());
        assert!(Cmd::Reset.expects_response());
        assert!(Cmd::RunInference.expects_response());
    }

    #[test]
    fn resp_code_is_ok() {
        assert!(RespCode::Ok.is_ok());
        assert!(!RespCode::ErrCrc.is_ok());
    }

    #[test]
    fn resp_code_from_u8_roundtrip() {
        let codes = [
            RespCode::Ok,
            RespCode::ErrCrc,
            RespCode::ErrTimeout,
            RespCode::ErrBusy,
            RespCode::ErrInvalid,
            RespCode::ErrOverflow,
        ];
        for c in codes {
            assert_eq!(RespCode::from_u8(c as u8), Some(c));
        }
    }

    #[test]
    fn cmd_packet_encode_decode() {
        let p = CmdPacket::new(Cmd::RunInference)
            .with_seq(42)
            .with_payload_len(1024)
            .with_tag(0xBEEF);
        let encoded = p.encode();
        let decoded = CmdPacket::decode(&encoded).unwrap();
        assert_eq!(decoded, p);
    }

    #[test]
    fn cmd_packet_decode_short() {
        let buf = [0u8; 4];
        let err = CmdPacket::decode(&buf).unwrap_err();
        assert!(matches!(
            err,
            ProtocolError::HeaderTooShort {
                have: 4,
                need: CMD_HEADER_SIZE
            }
        ));
    }

    #[test]
    fn cmd_packet_decode_bad_version() {
        let mut buf = CmdPacket::new(Cmd::Nop).encode();
        buf[0] = 99;
        assert!(matches!(CmdPacket::decode(&buf), Err(ProtocolError::BadVersion(99))));
    }

    #[test]
    fn cmd_packet_decode_bad_command() {
        let mut buf = CmdPacket::new(Cmd::Nop).encode();
        buf[1] = 0xFE;
        assert!(matches!(CmdPacket::decode(&buf), Err(ProtocolError::BadCommand(0xFE))));
    }

    #[test]
    fn resp_packet_encode_decode() {
        let p = RespPacket::new(RespCode::Ok)
            .with_seq(7)
            .with_payload_len(256);
        let encoded = p.encode();
        let decoded = RespPacket::decode(&encoded).unwrap();
        assert_eq!(decoded, p);
    }

    #[test]
    fn resp_packet_decode_short() {
        let buf = [0u8; 2];
        let err = RespPacket::decode(&buf).unwrap_err();
        assert!(matches!(
            err,
            ProtocolError::HeaderTooShort {
                have: 2,
                need: RESP_HEADER_SIZE
            }
        ));
    }

    #[test]
    fn resp_packet_decode_bad_code() {
        let mut buf = RespPacket::new(RespCode::Ok).encode();
        buf[0] = 0xFF;
        assert!(matches!(RespPacket::decode(&buf), Err(ProtocolError::BadResponse(0xFF))));
    }

    #[test]
    fn protocol_error_display() {
        let e = ProtocolError::HeaderTooShort { have: 2, need: 8 };
        assert!(e.to_string().contains("short"));
        let e = ProtocolError::BadVersion(5);
        assert!(e.to_string().contains("version"));
        let e = ProtocolError::BadCommand(0xFF);
        assert!(e.to_string().contains("command"));
    }

    #[test]
    fn cmd_packet_header_size() {
        let p = CmdPacket::new(Cmd::Nop);
        assert_eq!(p.encode().len(), CMD_HEADER_SIZE);
    }

    #[test]
    fn resp_packet_header_size() {
        let p = RespPacket::new(RespCode::Ok);
        assert_eq!(p.encode().len(), RESP_HEADER_SIZE);
    }
}

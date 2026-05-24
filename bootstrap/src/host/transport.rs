use super::protocol::{CmdPacket, RespPacket, CMD_HEADER_SIZE, RESP_HEADER_SIZE};

pub const CRC_SIZE: usize = 4;
pub const MAX_PAYLOAD: usize = 4096;
pub const FRAME_OVERHEAD: usize = 4 + CMD_HEADER_SIZE + CRC_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportError {
    PayloadTooLarge { have: usize, max: usize },
    CrcMismatch { expected: u32, got: u32 },
    FrameTooShort { have: usize, need: usize },
    BadHeader,
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::PayloadTooLarge { have, max } => {
                write!(f, "payload too large: {have} > {max}")
            }
            TransportError::CrcMismatch { expected, got } => {
                write!(f, "CRC mismatch: expected 0x{expected:08X}, got 0x{got:08X}")
            }
            TransportError::FrameTooShort { have, need } => {
                write!(f, "frame too short: {have} < {need}")
            }
            TransportError::BadHeader => write!(f, "bad frame header"),
        }
    }
}

impl std::error::Error for TransportError {}

fn crc32_table() -> &'static [u32; 256] {
    const TABLE: [u32; 256] = {
        let mut t = [0u32; 256];
        let mut i = 0;
        while i < 256 {
            let mut c = i as u32;
            let mut j = 0;
            while j < 8 {
                if c & 1 != 0 {
                    c = (c >> 1) ^ 0xEDB88320;
                } else {
                    c >>= 1;
                }
                j += 1;
            }
            t[i] = c;
            i += 1;
        }
        t
    };
    &TABLE
}

pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        let idx = ((crc ^ b as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ crc32_table()[idx];
    }
    !crc
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportFrame {
    pub header: CmdPacket,
    pub payload: Vec<u8>,
}

impl TransportFrame {
    pub fn new(header: CmdPacket, payload: &[u8]) -> Result<Self, TransportError> {
        if payload.len() > MAX_PAYLOAD {
            return Err(TransportError::PayloadTooLarge {
                have: payload.len(),
                max: MAX_PAYLOAD,
            });
        }
        Ok(Self {
            header,
            payload: payload.to_vec(),
        })
    }

    pub fn encoded_len(&self) -> usize {
        4 + CMD_HEADER_SIZE + self.payload.len() + CRC_SIZE
    }

    pub fn encode(&self) -> Vec<u8> {
        let total = self.encoded_len();
        let mut buf = Vec::with_capacity(total);
        let frame_len = (CMD_HEADER_SIZE + self.payload.len()) as u32;
        buf.extend_from_slice(&frame_len.to_le_bytes());
        buf.extend_from_slice(&self.header.encode());
        buf.extend_from_slice(&self.payload);
        let crc = crc32(&buf);
        buf.extend_from_slice(&crc.to_le_bytes());
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self, TransportError> {
        let min_len = 4 + CMD_HEADER_SIZE + CRC_SIZE;
        if data.len() < min_len {
            return Err(TransportError::FrameTooShort {
                have: data.len(),
                need: min_len,
            });
        }
        let frame_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let total_needed = 4 + frame_len + CRC_SIZE;
        if data.len() < total_needed {
            return Err(TransportError::FrameTooShort {
                have: data.len(),
                need: total_needed,
            });
        }
        let header_end = 4 + CMD_HEADER_SIZE;
        let header = CmdPacket::decode(&data[4..header_end]).map_err(|_| TransportError::BadHeader)?;
        let payload_end = 4 + frame_len;
        let payload = data[header_end..payload_end].to_vec();
        let crc_offset = payload_end;
        let stored_crc = u32::from_le_bytes([
            data[crc_offset],
            data[crc_offset + 1],
            data[crc_offset + 2],
            data[crc_offset + 3],
        ]);
        let computed_crc = crc32(&data[..payload_end]);
        if stored_crc != computed_crc {
            return Err(TransportError::CrcMismatch {
                expected: stored_crc,
                got: computed_crc,
            });
        }
        Ok(Self { header, payload })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespFrame {
    pub header: RespPacket,
    pub payload: Vec<u8>,
}

impl RespFrame {
    pub fn new(header: RespPacket, payload: &[u8]) -> Self {
        Self {
            header,
            payload: payload.to_vec(),
        }
    }

    pub fn encoded_len(&self) -> usize {
        4 + RESP_HEADER_SIZE + self.payload.len() + CRC_SIZE
    }

    pub fn encode(&self) -> Vec<u8> {
        let total = self.encoded_len();
        let mut buf = Vec::with_capacity(total);
        let frame_len = (RESP_HEADER_SIZE + self.payload.len()) as u32;
        buf.extend_from_slice(&frame_len.to_le_bytes());
        buf.extend_from_slice(&self.header.encode());
        buf.extend_from_slice(&self.payload);
        let crc = crc32(&buf);
        buf.extend_from_slice(&crc.to_le_bytes());
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self, TransportError> {
        let min_len = 4 + RESP_HEADER_SIZE + CRC_SIZE;
        if data.len() < min_len {
            return Err(TransportError::FrameTooShort {
                have: data.len(),
                need: min_len,
            });
        }
        let frame_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let total_needed = 4 + frame_len + CRC_SIZE;
        if data.len() < total_needed {
            return Err(TransportError::FrameTooShort {
                have: data.len(),
                need: total_needed,
            });
        }
        let header_end = 4 + RESP_HEADER_SIZE;
        let header =
            RespPacket::decode(&data[4..header_end]).map_err(|_| TransportError::BadHeader)?;
        let payload_end = 4 + frame_len;
        let payload = data[header_end..payload_end].to_vec();
        let crc_offset = payload_end;
        let stored_crc = u32::from_le_bytes([
            data[crc_offset],
            data[crc_offset + 1],
            data[crc_offset + 2],
            data[crc_offset + 3],
        ]);
        let computed_crc = crc32(&data[..payload_end]);
        if stored_crc != computed_crc {
            return Err(TransportError::CrcMismatch {
                expected: stored_crc,
                got: computed_crc,
            });
        }
        Ok(Self { header, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::protocol::{Cmd, RespCode};

    #[test]
    fn crc32_empty() {
        assert_eq!(crc32(&[]), 0x0000_0000);
    }

    #[test]
    fn crc32_known() {
        let c = crc32(b"123456789");
        assert_eq!(c, 0xCBF4_3926);
    }

    #[test]
    fn transport_frame_encode_decode_no_payload() {
        let cmd = CmdPacket::new(Cmd::Reset).with_seq(1);
        let frame = TransportFrame::new(cmd, &[]).unwrap();
        let encoded = frame.encode();
        let decoded = TransportFrame::decode(&encoded).unwrap();
        assert_eq!(decoded.header, frame.header);
        assert!(decoded.payload.is_empty());
    }

    #[test]
    fn transport_frame_encode_decode_with_payload() {
        let cmd = CmdPacket::new(Cmd::LoadWeights)
            .with_seq(5)
            .with_payload_len(4);
        let payload = [0xDE, 0xAD, 0xBE, 0xEF];
        let frame = TransportFrame::new(cmd, &payload).unwrap();
        let encoded = frame.encode();
        let decoded = TransportFrame::decode(&encoded).unwrap();
        assert_eq!(decoded.header, frame.header);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn transport_frame_payload_too_large() {
        let cmd = CmdPacket::new(Cmd::LoadWeights);
        let big = vec![0u8; MAX_PAYLOAD + 1];
        let err = TransportFrame::new(cmd, &big).unwrap_err();
        match err {
            TransportError::PayloadTooLarge { have, max } => {
                assert_eq!(have, MAX_PAYLOAD + 1);
                assert_eq!(max, MAX_PAYLOAD);
            }
            e => panic!("wrong error: {e}"),
        }
    }

    #[test]
    fn transport_frame_decode_crc_mismatch() {
        let cmd = CmdPacket::new(Cmd::Reset);
        let frame = TransportFrame::new(cmd, &[]).unwrap();
        let mut encoded = frame.encode();
        let last = encoded.len() - 1;
        encoded[last] ^= 0xFF;
        let err = TransportFrame::decode(&encoded).unwrap_err();
        assert!(matches!(err, TransportError::CrcMismatch { .. }));
    }

    #[test]
    fn transport_frame_decode_too_short() {
        let buf = [0u8; 4];
        let err = TransportFrame::decode(&buf).unwrap_err();
        assert!(matches!(err, TransportError::FrameTooShort { .. }));
    }

    #[test]
    fn transport_frame_encoded_len() {
        let cmd = CmdPacket::new(Cmd::Reset);
        let frame = TransportFrame::new(cmd, &[0xAA, 0xBB]).unwrap();
        assert_eq!(frame.encoded_len(), 4 + CMD_HEADER_SIZE + 2 + CRC_SIZE);
    }

    #[test]
    fn resp_frame_encode_decode() {
        let resp = RespPacket::new(RespCode::Ok).with_seq(3);
        let payload = [0x01, 0x02, 0x03];
        let frame = RespFrame::new(resp, &payload);
        let encoded = frame.encode();
        let decoded = RespFrame::decode(&encoded).unwrap();
        assert_eq!(decoded.header, frame.header);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn resp_frame_decode_crc_mismatch() {
        let resp = RespPacket::new(RespCode::Ok);
        let frame = RespFrame::new(resp, &[]);
        let mut encoded = frame.encode();
        let last = encoded.len() - 1;
        encoded[last] ^= 0xFF;
        let err = RespFrame::decode(&encoded).unwrap_err();
        assert!(matches!(err, TransportError::CrcMismatch { .. }));
    }

    #[test]
    fn resp_frame_decode_too_short() {
        let buf = [0u8; 2];
        let err = RespFrame::decode(&buf).unwrap_err();
        assert!(matches!(err, TransportError::FrameTooShort { .. }));
    }

    #[test]
    fn resp_frame_encoded_len() {
        let resp = RespPacket::new(RespCode::Ok);
        let frame = RespFrame::new(resp, &[]);
        assert_eq!(frame.encoded_len(), 4 + RESP_HEADER_SIZE + CRC_SIZE);
    }

    #[test]
    fn error_display() {
        let e = TransportError::PayloadTooLarge { have: 5000, max: 4096 };
        assert!(e.to_string().contains("5000"));
        let e = TransportError::CrcMismatch { expected: 1, got: 2 };
        assert!(e.to_string().contains("CRC"));
        let e = TransportError::FrameTooShort { have: 2, need: 16 };
        assert!(e.to_string().contains("short"));
    }

    #[test]
    fn crc_deterministic() {
        let a = crc32(b"hello");
        let b = crc32(b"hello");
        assert_eq!(a, b);
    }

    #[test]
    fn crc_different_inputs() {
        let a = crc32(b"hello");
        let b = crc32(b"world");
        assert_ne!(a, b);
    }
}

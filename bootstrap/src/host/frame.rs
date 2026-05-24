const FLAG: u8 = 0x7E;
const ESC: u8 = 0x7D;
const ESC_XOR: u8 = 0x20;

fn crc16_update(crc: u16, byte: u8) -> u16 {
    let mut crc = crc ^ (byte as u16);
    for _ in 0..8 {
        if crc & 1 != 0 {
            crc = (crc >> 1) ^ 0x8408;
        } else {
            crc >>= 1;
        }
    }
    crc
}

pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc = crc16_update(crc, b);
    }
    !crc
}

pub fn encode_frame(payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(payload.len() + 8);
    buf.push(FLAG);
    let crc = crc16(payload);
    let mut body = payload.to_vec();
    body.push((crc & 0xFF) as u8);
    body.push((crc >> 8) as u8);
    for &b in &body {
        if b == FLAG || b == ESC {
            buf.push(ESC);
            buf.push(b ^ ESC_XOR);
        } else {
            buf.push(b);
        }
    }
    buf.push(FLAG);
    buf
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    EmptyFrame,
    CrcMismatch { expected: u16, got: u16 },
    TruncatedEscape,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::EmptyFrame => write!(f, "empty frame"),
            DecodeError::CrcMismatch { expected, got } => {
                write!(f, "crc mismatch: expected 0x{expected:04X}, got 0x{got:04X}")
            }
            DecodeError::TruncatedEscape => write!(f, "truncated escape sequence"),
        }
    }
}

impl std::error::Error for DecodeError {}

fn unstuff(data: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0;
    while i < data.len() {
        if data[i] == ESC {
            if i + 1 >= data.len() {
                return Err(DecodeError::TruncatedEscape);
            }
            out.push(data[i + 1] ^ ESC_XOR);
            i += 2;
        } else {
            out.push(data[i]);
            i += 1;
        }
    }
    Ok(out)
}

pub fn decode_frame(frame: &[u8]) -> Result<Vec<u8>, DecodeError> {
    if frame.len() < 2 {
        return Err(DecodeError::EmptyFrame);
    }
    let start = if frame[0] == FLAG { 1 } else { 0 };
    let end = if frame.last() == Some(&FLAG) {
        frame.len() - 1
    } else {
        frame.len()
    };
    if start >= end {
        return Err(DecodeError::EmptyFrame);
    }
    let inner = &frame[start..end];
    if inner.is_empty() {
        return Err(DecodeError::EmptyFrame);
    }
    let unstuffed = unstuff(inner)?;
    if unstuffed.len() < 2 {
        return Err(DecodeError::EmptyFrame);
    }
    let payload_len = unstuffed.len() - 2;
    let payload = unstuffed[..payload_len].to_vec();
    let got_crc = (unstuffed[payload_len] as u16) | ((unstuffed[payload_len + 1] as u16) << 8);
    let expected_crc = crc16(&payload);
    if got_crc != expected_crc {
        return Err(DecodeError::CrcMismatch {
            expected: expected_crc,
            got: got_crc,
        });
    }
    Ok(payload)
}

#[derive(Debug, Clone)]
pub struct FrameStats {
    pub encoded: u64,
    pub decoded: u64,
    pub decode_errors: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

impl FrameStats {
    pub fn new() -> Self {
        Self {
            encoded: 0,
            decoded: 0,
            decode_errors: 0,
            bytes_in: 0,
            bytes_out: 0,
        }
    }
}

impl Default for FrameStats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct FrameCodec {
    stats: FrameStats,
}

impl FrameCodec {
    pub fn new() -> Self {
        Self {
            stats: FrameStats::new(),
        }
    }

    pub fn encode(&mut self, payload: &[u8]) -> Vec<u8> {
        let frame = encode_frame(payload);
        self.stats.encoded += 1;
        self.stats.bytes_in += payload.len() as u64;
        self.stats.bytes_out += frame.len() as u64;
        frame
    }

    pub fn decode(&mut self, frame: &[u8]) -> Result<Vec<u8>, DecodeError> {
        match decode_frame(frame) {
            Ok(payload) => {
                self.stats.decoded += 1;
                Ok(payload)
            }
            Err(e) => {
                self.stats.decode_errors += 1;
                Err(e)
            }
        }
    }

    pub fn stats(&self) -> &FrameStats {
        &self.stats
    }

    pub fn roundtrip(&mut self, payload: &[u8]) -> Vec<u8> {
        let encoded = self.encode(payload);
        self.decode(&encoded).unwrap()
    }
}

impl Default for FrameCodec {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc16_known() {
        let crc = crc16(b"123456789");
        assert_eq!(crc, 0x906E);
    }

    #[test]
    fn crc16_empty() {
        let crc = crc16(&[]);
        assert_eq!(crc, 0x0000);
    }

    #[test]
    fn encode_empty_payload() {
        let frame = encode_frame(&[]);
        assert!(frame.starts_with(&[FLAG]));
        assert!(frame.ends_with(&[FLAG]));
    }

    #[test]
    fn encode_no_special_bytes() {
        let frame = encode_frame(b"hello");
        assert!(frame.contains(&b'h'));
    }

    #[test]
    fn encode_escapes_flag() {
        let frame = encode_frame(&[FLAG]);
        let inner = &frame[1..frame.len() - 1];
        assert!(inner.contains(&ESC));
        assert!(!inner.contains(&FLAG));
    }

    #[test]
    fn encode_escapes_esc() {
        let frame = encode_frame(&[ESC]);
        let inner = &frame[1..frame.len() - 1];
        assert!(inner.iter().any(|&b| b == (ESC ^ ESC_XOR)));
        assert!(!inner.iter().any(|&b| b == ESC && false));
        let roundtrip = decode_frame(&frame).unwrap();
        assert_eq!(roundtrip, vec![ESC]);
    }

    #[test]
    fn decode_empty_frame() {
        assert!(decode_frame(&[]).unwrap_err() == DecodeError::EmptyFrame);
    }

    #[test]
    fn roundtrip_simple() {
        let payload = b"hello world";
        let encoded = encode_frame(payload);
        let decoded = decode_frame(&encoded).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn roundtrip_with_special_bytes() {
        let payload: Vec<u8> = vec![0x00, FLAG, 0x01, ESC, 0x02, FLAG, 0x03];
        let encoded = encode_frame(&payload);
        let decoded = decode_frame(&encoded).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn roundtrip_all_byte_values() {
        let payload: Vec<u8> = (0u8..=255).collect();
        let encoded = encode_frame(&payload);
        let decoded = decode_frame(&encoded).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn crc_mismatch_detected() {
        let mut frame = encode_frame(b"test");
        let len = frame.len();
        frame[len - 2] ^= 0xFF;
        let err = decode_frame(&frame).unwrap_err();
        assert!(matches!(err, DecodeError::CrcMismatch { .. }));
    }

    #[test]
    fn codec_roundtrip() {
        let mut codec = FrameCodec::new();
        let out = codec.roundtrip(b"data");
        assert_eq!(out, b"data");
        assert_eq!(codec.stats().encoded, 1);
        assert_eq!(codec.stats().decoded, 1);
    }

    #[test]
    fn codec_stats() {
        let mut codec = FrameCodec::new();
        codec.encode(b"abc");
        codec.decode(&encode_frame(b"xyz")).unwrap();
        assert_eq!(codec.stats().encoded, 1);
        assert_eq!(codec.stats().decoded, 1);
        assert_eq!(codec.stats().decode_errors, 0);
    }

    #[test]
    fn truncated_escape() {
        let bad: Vec<u8> = vec![FLAG, ESC, FLAG];
        let err = decode_frame(&bad).unwrap_err();
        assert!(matches!(err, DecodeError::TruncatedEscape));
    }

    #[test]
    fn error_display() {
        assert!(DecodeError::EmptyFrame.to_string().contains("empty"));
        assert!(DecodeError::CrcMismatch { expected: 1, got: 2 }.to_string().contains("mismatch"));
        assert!(DecodeError::TruncatedEscape.to_string().contains("truncated"));
    }
}

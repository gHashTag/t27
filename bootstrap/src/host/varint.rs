use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum VarintError {
    Overflow,
    UnexpectedEnd,
    InvalidByte { byte: u8, pos: usize },
}

impl std::fmt::Display for VarintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarintError::Overflow => write!(f, "varint overflow"),
            VarintError::UnexpectedEnd => write!(f, "unexpected end of stream"),
            VarintError::InvalidByte { byte, pos } => write!(f, "invalid byte {byte:#x} at pos {pos}"),
        }
    }
}

impl std::error::Error for VarintError {}

pub fn encode_u64(mut val: u64) -> Vec<u8> {
    if val == 0 { return vec![0]; }
    let mut buf = Vec::new();
    while val > 0 {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if val > 0 { byte |= 0x80; }
        buf.push(byte);
    }
    buf
}

pub fn decode_u64(bytes: &[u8]) -> Result<(u64, usize), VarintError> {
    let mut val: u64 = 0;
    let mut shift: u32 = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        if shift >= 70 { return Err(VarintError::Overflow); }
        val |= ((byte & 0x7F) as u64) << shift;
        shift += 7;
        if byte & 0x80 == 0 { return Ok((val, i + 1)); }
    }
    Err(VarintError::UnexpectedEnd)
}

pub fn zigzag_encode(val: i64) -> u64 {
    ((val << 1) ^ (val >> 63)) as u64
}

pub fn zigzag_decode(val: u64) -> i64 {
    ((val >> 1) as i64) ^ -((val & 1) as i64)
}

pub fn encode_i64(val: i64) -> Vec<u8> { encode_u64(zigzag_encode(val)) }

pub fn decode_i64(bytes: &[u8]) -> Result<(i64, usize), VarintError> {
    let (v, n) = decode_u64(bytes)?;
    Ok((zigzag_decode(v), n))
}

pub struct VarintReader {
    buffer: VecDeque<u8>,
    total_decoded: u64,
    total_bytes: u64,
}

impl VarintReader {
    pub fn new() -> Self { Self { buffer: VecDeque::new(), total_decoded: 0, total_bytes: 0 } }

    pub fn push(&mut self, data: &[u8]) {
        self.total_bytes += data.len() as u64;
        for &b in data { self.buffer.push_back(b); }
    }

    pub fn read_u64(&mut self) -> Result<u64, VarintError> {
        let mut val: u64 = 0;
        let mut shift: u32 = 0;
        let mut count = 0;
        loop {
            let byte = self.buffer.pop_front().ok_or(VarintError::UnexpectedEnd)?;
            count += 1;
            if shift >= 70 { return Err(VarintError::Overflow); }
            val |= ((byte & 0x7F) as u64) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                self.total_decoded += 1;
                return Ok(val);
            }
        }
    }

    pub fn read_i64(&mut self) -> Result<i64, VarintError> {
        let v = self.read_u64()?;
        Ok(zigzag_decode(v))
    }

    pub fn remaining(&self) -> usize { self.buffer.len() }
    pub fn total_decoded(&self) -> u64 { self.total_decoded }
    pub fn total_bytes(&self) -> u64 { self.total_bytes }
}

impl Default for VarintReader {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_zero() { assert_eq!(encode_u64(0), vec![0]); }

    #[test]
    fn encode_small() { assert_eq!(encode_u64(127), vec![127]); }

    #[test]
    fn encode_two_byte() { assert_eq!(encode_u64(128), vec![0x80, 0x01]); }

    #[test]
    fn encode_large() {
        let encoded = encode_u64(300);
        assert_eq!(decode_u64(&encoded), Ok((300, encoded.len())));
    }

    #[test]
    fn roundtrip_u64() {
        for val in [0u64, 1, 127, 128, 255, 256, 16383, 16384, u32::MAX as u64, u64::MAX] {
            let enc = encode_u64(val);
            let (dec, n) = decode_u64(&enc).unwrap();
            assert_eq!(dec, val);
            assert_eq!(n, enc.len());
        }
    }

    #[test]
    fn zigzag_roundtrip() {
        for val in [0i64, -1, 1, -2, 2, i64::MIN, i64::MAX] {
            assert_eq!(zigzag_decode(zigzag_encode(val)), val);
        }
    }

    #[test]
    fn i64_roundtrip() {
        for val in [0i64, -1, 127, -128, i64::MIN / 2, i64::MAX] {
            let enc = encode_i64(val);
            let (dec, _) = decode_i64(&enc).unwrap();
            assert_eq!(dec, val);
        }
    }

    #[test]
    fn unexpected_end() {
        assert!(matches!(decode_u64(&[0x80]), Err(VarintError::UnexpectedEnd)));
    }

    #[test]
    fn stream_reader() {
        let mut r = VarintReader::new();
        r.push(&[0x80, 0x01, 0x7F]);
        assert_eq!(r.read_u64(), Ok(128));
        assert_eq!(r.read_u64(), Ok(127));
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn stream_i64() {
        let mut r = VarintReader::new();
        let enc = encode_i64(-42);
        r.push(&enc);
        assert_eq!(r.read_i64(), Ok(-42));
    }

    #[test]
    fn stats() {
        let mut r = VarintReader::new();
        r.push(&encode_u64(1));
        r.push(&encode_u64(2));
        r.read_u64().unwrap();
        r.read_u64().unwrap();
        assert_eq!(r.total_decoded(), 2);
    }

    #[test]
    fn error_display() { assert!(VarintError::Overflow.to_string().contains("overflow")); }
}

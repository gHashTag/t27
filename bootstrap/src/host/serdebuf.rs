const CRC_POLY: u64 = 0x42F0E1EBA9EA3693;

fn crc64(data: &[u8]) -> u64 {
    let mut crc: u64 = 0;
    for &b in data {
        crc ^= (b as u64) << 56;
        for _ in 0..8 {
            if crc & (1 << 63) != 0 { crc = (crc << 1) ^ CRC_POLY; } else { crc <<= 1; }
        }
    }
    crc
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sb3Error {
    BufferOverflow { needed: usize, available: usize },
    BufferUnderflow { needed: usize, available: usize },
    CrcMismatch { expected: u64, got: u64 },
    InvalidTag { tag: u8 },
}

impl std::fmt::Display for Sb3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sb3Error::BufferOverflow { needed, available } => write!(f, "overflow: need {needed}, have {available}"),
            Sb3Error::BufferUnderflow { needed, available } => write!(f, "underflow: need {needed}, have {available}"),
            Sb3Error::CrcMismatch { expected, got } => write!(f, "crc mismatch: expected {expected:#x}, got {got:#x}"),
            Sb3Error::InvalidTag { tag } => write!(f, "invalid tag {tag:#x}"),
        }
    }
}

impl std::error::Error for Sb3Error {}

pub struct SerdeBuf {
    buf: Vec<u8>,
    pos: usize,
    total_serialized: u64,
    total_deserialized: u64,
}

impl SerdeBuf {
    pub fn new(capacity: usize) -> Self { Self { buf: Vec::with_capacity(capacity), pos: 0, total_serialized: 0, total_deserialized: 0 } }

    pub fn write_u8(&mut self, val: u8) -> Result<(), Sb3Error> {
        self.buf.push(val);
        Ok(())
    }

    pub fn write_u16(&mut self, val: u16) -> Result<(), Sb3Error> {
        self.buf.extend_from_slice(&val.to_le_bytes());
        Ok(())
    }

    pub fn write_u32(&mut self, val: u32) -> Result<(), Sb3Error> {
        self.buf.extend_from_slice(&val.to_le_bytes());
        Ok(())
    }

    pub fn write_u64(&mut self, val: u64) -> Result<(), Sb3Error> {
        self.buf.extend_from_slice(&val.to_le_bytes());
        Ok(())
    }

    pub fn write_bytes(&mut self, data: &[u8]) -> Result<(), Sb3Error> {
        self.write_u16(data.len() as u16)?;
        self.buf.extend_from_slice(data);
        Ok(())
    }

    pub fn write_tag(&mut self, tag: u8, data: &[u8]) -> Result<(), Sb3Error> {
        self.write_u8(tag)?;
        self.write_bytes(data)?;
        self.total_serialized += 1;
        Ok(())
    }

    pub fn finalize(&mut self) -> &[u8] {
        let checksum = crc64(&self.buf);
        self.buf.extend_from_slice(&checksum.to_le_bytes());
        &self.buf
    }

    pub fn read_u8(&mut self) -> Result<u8, Sb3Error> {
        if self.pos >= self.buf.len() { return Err(Sb3Error::BufferUnderflow { needed: 1, available: self.buf.len() - self.pos }); }
        let v = self.buf[self.pos];
        self.pos += 1;
        Ok(v)
    }

    pub fn read_u16(&mut self) -> Result<u16, Sb3Error> {
        if self.pos + 2 > self.buf.len() { return Err(Sb3Error::BufferUnderflow { needed: 2, available: self.buf.len() - self.pos }); }
        let v = u16::from_le_bytes([self.buf[self.pos], self.buf[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    pub fn read_u32(&mut self) -> Result<u32, Sb3Error> {
        if self.pos + 4 > self.buf.len() { return Err(Sb3Error::BufferUnderflow { needed: 4, available: self.buf.len() - self.pos }); }
        let v = u32::from_le_bytes(self.buf[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(v)
    }

    pub fn read_u64(&mut self) -> Result<u64, Sb3Error> {
        if self.pos + 8 > self.buf.len() { return Err(Sb3Error::BufferUnderflow { needed: 8, available: self.buf.len() - self.pos }); }
        let v = u64::from_le_bytes(self.buf[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(v)
    }

    pub fn read_bytes(&mut self) -> Result<Vec<u8>, Sb3Error> {
        let len = self.read_u16()? as usize;
        if self.pos + len > self.buf.len() { return Err(Sb3Error::BufferUnderflow { needed: len, available: self.buf.len() - self.pos }); }
        let data = self.buf[self.pos..self.pos + len].to_vec();
        self.pos += len;
        Ok(data)
    }

    pub fn read_tag(&mut self, expected: u8) -> Result<Vec<u8>, Sb3Error> {
        let tag = self.read_u8()?;
        if tag != expected { return Err(Sb3Error::InvalidTag { tag }); }
        let data = self.read_bytes()?;
        self.total_deserialized += 1;
        Ok(data)
    }

    pub fn verify_crc(&mut self) -> Result<(), Sb3Error> {
        if self.buf.len() < 8 { return Err(Sb3Error::BufferUnderflow { needed: 8, available: self.buf.len() }); }
        let payload_end = self.buf.len() - 8;
        let expected = u64::from_le_bytes(self.buf[payload_end..].try_into().unwrap());
        let got = crc64(&self.buf[..payload_end]);
        if expected != got { return Err(Sb3Error::CrcMismatch { expected, got }); }
        Ok(())
    }

    pub fn from_bytes(data: Vec<u8>) -> Self { Self { buf: data, pos: 0, total_serialized: 0, total_deserialized: 0 } }
    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
    pub fn total_serialized(&self) -> u64 { self.total_serialized }
    pub fn total_deserialized(&self) -> u64 { self.total_deserialized }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buf() { let sb = SerdeBuf::new(64); assert!(sb.is_empty()); }

    #[test]
    fn write_read_u8() {
        let mut sb = SerdeBuf::new(64);
        sb.write_u8(42).unwrap();
        let mut r = SerdeBuf::from_bytes(sb.buf.clone());
        assert_eq!(r.read_u8(), Ok(42));
    }

    #[test]
    fn write_read_u64() {
        let mut sb = SerdeBuf::new(64);
        sb.write_u64(0xDEADBEEF).unwrap();
        let mut r = SerdeBuf::from_bytes(sb.buf.clone());
        assert_eq!(r.read_u64(), Ok(0xDEADBEEF));
    }

    #[test]
    fn write_read_bytes() {
        let mut sb = SerdeBuf::new(64);
        sb.write_bytes(b"hello").unwrap();
        let mut r = SerdeBuf::from_bytes(sb.buf.clone());
        assert_eq!(r.read_bytes(), Ok(b"hello".to_vec()));
    }

    #[test]
    fn tag_roundtrip() {
        let mut sb = SerdeBuf::new(64);
        sb.write_tag(0x01, b"data").unwrap();
        let mut r = SerdeBuf::from_bytes(sb.buf.clone());
        assert_eq!(r.read_tag(0x01), Ok(b"data".to_vec()));
    }

    #[test]
    fn crc_verify() {
        let mut sb = SerdeBuf::new(64);
        sb.write_u32(123).unwrap();
        sb.finalize();
        let mut r = SerdeBuf::from_bytes(sb.buf.clone());
        assert!(r.verify_crc().is_ok());
    }

    #[test]
    fn crc_tampered() {
        let mut sb = SerdeBuf::new(64);
        sb.write_u32(123).unwrap();
        sb.finalize();
        sb.buf[0] ^= 0xFF;
        let mut r = SerdeBuf::from_bytes(sb.buf.clone());
        let err = r.verify_crc().unwrap_err();
        assert!(matches!(err, Sb3Error::CrcMismatch { .. }));
    }

    #[test]
    fn underflow() {
        let mut r = SerdeBuf::from_bytes(vec![1]);
        let err = r.read_u32().unwrap_err();
        assert!(matches!(err, Sb3Error::BufferUnderflow { .. }));
    }

    #[test]
    fn invalid_tag() {
        let mut sb = SerdeBuf::new(64);
        sb.write_tag(0x01, b"x").unwrap();
        let mut r = SerdeBuf::from_bytes(sb.buf.clone());
        let err = r.read_tag(0x02).unwrap_err();
        assert!(matches!(err, Sb3Error::InvalidTag { .. }));
    }

    #[test]
    fn error_display() { assert!(Sb3Error::CrcMismatch { expected: 1, got: 2 }.to_string().contains("mismatch")); }
}

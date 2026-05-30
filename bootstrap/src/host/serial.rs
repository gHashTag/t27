#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialError {
    EndOfBuffer { need: usize, have: usize },
    Overflow { capacity: usize, requested: usize },
}

impl std::fmt::Display for SerialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerialError::EndOfBuffer { need, have } => {
                write!(f, "end of buffer: need {need}, have {have}")
            }
            SerialError::Overflow { capacity, requested } => {
                write!(f, "overflow: capacity {capacity}, requested {requested}")
            }
        }
    }
}

impl std::error::Error for SerialError {}

#[derive(Debug, Clone)]
pub struct Serializer {
    buf: Vec<u8>,
}

impl Serializer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    pub fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn write_u16_le(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_u32_le(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_u64_le(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    pub fn write_cstr(&mut self, s: &str) {
        self.buf.extend_from_slice(s.as_bytes());
        self.buf.push(0);
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }
}

#[derive(Debug, Clone)]
pub struct Deserializer<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Deserializer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub fn read_u8(&mut self) -> Result<u8, SerialError> {
        if self.remaining() < 1 {
            return Err(SerialError::EndOfBuffer { need: 1, have: self.remaining() });
        }
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    pub fn read_u16_le(&mut self) -> Result<u16, SerialError> {
        if self.remaining() < 2 {
            return Err(SerialError::EndOfBuffer { need: 2, have: self.remaining() });
        }
        let v = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    pub fn read_u32_le(&mut self) -> Result<u32, SerialError> {
        if self.remaining() < 4 {
            return Err(SerialError::EndOfBuffer { need: 4, have: self.remaining() });
        }
        let v = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    pub fn read_u64_le(&mut self) -> Result<u64, SerialError> {
        if self.remaining() < 8 {
            return Err(SerialError::EndOfBuffer { need: 8, have: self.remaining() });
        }
        let bytes: [u8; 8] = self.data[self.pos..self.pos + 8].try_into().unwrap();
        let v = u64::from_le_bytes(bytes);
        self.pos += 8;
        Ok(v)
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], SerialError> {
        if self.remaining() < len {
            return Err(SerialError::EndOfBuffer { need: len, have: self.remaining() });
        }
        let slice = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(slice)
    }

    pub fn read_cstr(&mut self) -> Result<&'a str, SerialError> {
        let start = self.pos;
        while self.pos < self.data.len() && self.data[self.pos] != 0 {
            self.pos += 1;
        }
        if self.pos >= self.data.len() {
            return Err(SerialError::EndOfBuffer { need: 1, have: 0 });
        }
        let s = std::str::from_utf8(&self.data[start..self.pos]).unwrap_or("");
        self.pos += 1;
        Ok(s)
    }

    pub fn skip(&mut self, len: usize) -> Result<(), SerialError> {
        if self.remaining() < len {
            return Err(SerialError::EndOfBuffer { need: len, have: self.remaining() });
        }
        self.pos += len;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializer_u8() {
        let mut s = Serializer::new(16);
        s.write_u8(0xAB);
        assert_eq!(s.as_slice(), &[0xAB]);
    }

    #[test]
    fn serializer_u16_le() {
        let mut s = Serializer::new(16);
        s.write_u16_le(0x1234);
        assert_eq!(s.as_slice(), &[0x34, 0x12]);
    }

    #[test]
    fn serializer_u32_le() {
        let mut s = Serializer::new(16);
        s.write_u32_le(0x12345678);
        assert_eq!(s.as_slice(), &[0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn serializer_u64_le() {
        let mut s = Serializer::new(16);
        s.write_u64_le(0x0102030405060708);
        assert_eq!(s.as_slice(), &[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
    }

    #[test]
    fn serializer_cstr() {
        let mut s = Serializer::new(16);
        s.write_cstr("hello");
        assert_eq!(s.as_slice(), b"hello\0");
    }

    #[test]
    fn deserializer_u8() {
        let mut d = Deserializer::new(&[0xAB]);
        assert_eq!(d.read_u8().unwrap(), 0xAB);
    }

    #[test]
    fn deserializer_u16_le() {
        let mut d = Deserializer::new(&[0x34, 0x12]);
        assert_eq!(d.read_u16_le().unwrap(), 0x1234);
    }

    #[test]
    fn deserializer_u32_le() {
        let mut d = Deserializer::new(&[0x78, 0x56, 0x34, 0x12]);
        assert_eq!(d.read_u32_le().unwrap(), 0x12345678);
    }

    #[test]
    fn deserializer_u64_le() {
        let mut d = Deserializer::new(&[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
        assert_eq!(d.read_u64_le().unwrap(), 0x0102030405060708);
    }

    #[test]
    fn roundtrip() {
        let mut s = Serializer::new(32);
        s.write_u8(1);
        s.write_u16_le(0x1234);
        s.write_u32_le(0xDEADBEEF);
        s.write_u64_le(0xCAFEBABE_DEADBEEF);
        let bytes = s.into_bytes();
        let mut d = Deserializer::new(&bytes);
        assert_eq!(d.read_u8().unwrap(), 1);
        assert_eq!(d.read_u16_le().unwrap(), 0x1234);
        assert_eq!(d.read_u32_le().unwrap(), 0xDEADBEEF);
        assert_eq!(d.read_u64_le().unwrap(), 0xCAFEBABE_DEADBEEF);
    }

    #[test]
    fn read_bytes() {
        let data = b"hello";
        let mut d = Deserializer::new(data);
        assert_eq!(d.read_bytes(3).unwrap(), b"hel");
        assert_eq!(d.remaining(), 2);
    }

    #[test]
    fn read_cstr() {
        let data = b"hello\0world";
        let mut d = Deserializer::new(data);
        assert_eq!(d.read_cstr().unwrap(), "hello");
        assert_eq!(d.remaining(), 5);
    }

    #[test]
    fn end_of_buffer() {
        let data = [0x01];
        let mut d = Deserializer::new(&data);
        d.read_u8().unwrap();
        let err = d.read_u8().unwrap_err();
        assert!(matches!(err, SerialError::EndOfBuffer { .. }));
    }

    #[test]
    fn skip() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut d = Deserializer::new(&data);
        d.skip(2).unwrap();
        assert_eq!(d.read_u8().unwrap(), 0x03);
    }

    #[test]
    fn error_display() {
        let e = SerialError::EndOfBuffer { need: 4, have: 2 };
        assert!(e.to_string().contains("4"));
    }
}

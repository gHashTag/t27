#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitError {
    EndOfStream,
}

impl std::fmt::Display for BitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitError::EndOfStream => write!(f, "end of bitstream"),
        }
    }
}

impl std::error::Error for BitError {}

#[derive(Debug, Clone)]
pub struct BitstreamReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitstreamReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    pub fn bits_remaining(&self) -> usize {
        let bytes_left = self.data.len().saturating_sub(self.byte_pos);
        bytes_left * 8 - self.bit_pos as usize
    }

    pub fn bytes_consumed(&self) -> usize {
        if self.bit_pos > 0 {
            self.byte_pos + 1
        } else {
            self.byte_pos
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bits_remaining() == 0
    }

    pub fn read_bit(&mut self) -> Result<bool, BitError> {
        if self.byte_pos >= self.data.len() {
            return Err(BitError::EndOfStream);
        }
        let bit = (self.data[self.byte_pos] >> (7 - self.bit_pos)) & 1;
        self.bit_pos += 1;
        if self.bit_pos == 8 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }
        Ok(bit != 0)
    }

    pub fn read_bits(&mut self, count: u8) -> Result<u64, BitError> {
        if count as usize > self.bits_remaining() {
            return Err(BitError::EndOfStream);
        }
        if count == 0 {
            return Ok(0);
        }
        let mut result: u64 = 0;
        for _ in 0..count {
            result = (result << 1) | if self.read_bit()? { 1 } else { 0 };
        }
        Ok(result)
    }

    pub fn read_u8(&mut self) -> Result<u8, BitError> {
        Ok(self.read_bits(8)? as u8)
    }

    pub fn read_u16_le(&mut self) -> Result<u16, BitError> {
        let lo = self.read_u8()?;
        let hi = self.read_u8()?;
        Ok((hi as u16) << 8 | lo as u16)
    }

    pub fn read_u32_le(&mut self) -> Result<u32, BitError> {
        let b0 = self.read_u8()?;
        let b1 = self.read_u8()?;
        let b2 = self.read_u8()?;
        let b3 = self.read_u8()?;
        Ok((b3 as u32) << 24 | (b2 as u32) << 16 | (b1 as u32) << 8 | b0 as u32)
    }

    pub fn align_to_byte(&mut self) {
        if self.bit_pos > 0 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }
    }

    pub fn skip_bits(&mut self, count: usize) -> Result<(), BitError> {
        for _ in 0..count {
            self.read_bit()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_bits(bits: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut byte: u8 = 0;
        for (i, &b) in bits.iter().enumerate() {
            byte = (byte << 1) | (b & 1);
            if (i + 1) % 8 == 0 {
                bytes.push(byte);
                byte = 0;
            }
        }
        if bits.len() % 8 != 0 {
            byte <<= 8 - (bits.len() % 8);
            bytes.push(byte);
        }
        bytes
    }

    #[test]
    fn empty_reader() {
        let r = BitstreamReader::new(&[]);
        assert!(r.is_empty());
        assert_eq!(r.bits_remaining(), 0);
    }

    #[test]
    fn read_single_bits() {
        let data = [0b10110100];
        let mut r = BitstreamReader::new(&data);
        assert!(r.read_bit().unwrap());
        assert!(!r.read_bit().unwrap());
        assert!(r.read_bit().unwrap());
        assert!(r.read_bit().unwrap());
        assert!(!r.read_bit().unwrap());
        assert!(r.read_bit().unwrap());
        assert!(!r.read_bit().unwrap());
        assert!(!r.read_bit().unwrap());
    }

    #[test]
    fn read_bits_3() {
        let data = [0b11100010];
        let mut r = BitstreamReader::new(&data);
        assert_eq!(r.read_bits(3).unwrap(), 0b111);
        assert_eq!(r.read_bits(3).unwrap(), 0b000);
        assert_eq!(r.read_bits(2).unwrap(), 0b10);
    }

    #[test]
    fn read_bits_zero() {
        let data = [0xFF];
        let mut r = BitstreamReader::new(&data);
        assert_eq!(r.read_bits(0).unwrap(), 0);
        assert_eq!(r.bits_remaining(), 8);
    }

    #[test]
    fn read_u8() {
        let data = [0xAB, 0xCD];
        let mut r = BitstreamReader::new(&data);
        assert_eq!(r.read_u8().unwrap(), 0xAB);
        assert_eq!(r.read_u8().unwrap(), 0xCD);
    }

    #[test]
    fn read_u16_le() {
        let data = [0x34, 0x12];
        let mut r = BitstreamReader::new(&data);
        assert_eq!(r.read_u16_le().unwrap(), 0x1234);
    }

    #[test]
    fn read_u32_le() {
        let data = [0x78, 0x56, 0x34, 0x12];
        let mut r = BitstreamReader::new(&data);
        assert_eq!(r.read_u32_le().unwrap(), 0x12345678);
    }

    #[test]
    fn end_of_stream_bit() {
        let data = [0xFF];
        let mut r = BitstreamReader::new(&data);
        for _ in 0..8 {
            r.read_bit().unwrap();
        }
        assert!(matches!(r.read_bit(), Err(BitError::EndOfStream)));
    }

    #[test]
    fn end_of_stream_bits() {
        let data = [0xFF];
        let mut r = BitstreamReader::new(&data);
        assert!(matches!(r.read_bits(9), Err(BitError::EndOfStream)));
    }

    #[test]
    fn bits_remaining() {
        let data = [0xFF, 0x00];
        let mut r = BitstreamReader::new(&data);
        assert_eq!(r.bits_remaining(), 16);
        r.read_bits(3).unwrap();
        assert_eq!(r.bits_remaining(), 13);
    }

    #[test]
    fn bytes_consumed() {
        let data = [0xFF, 0x00, 0xAA];
        let mut r = BitstreamReader::new(&data);
        assert_eq!(r.bytes_consumed(), 0);
        r.read_bits(4).unwrap();
        assert_eq!(r.bytes_consumed(), 1);
        r.read_bits(4).unwrap();
        assert_eq!(r.bytes_consumed(), 1);
        r.read_bits(8).unwrap();
        assert_eq!(r.bytes_consumed(), 2);
    }

    #[test]
    fn align_to_byte() {
        let data = [0xFF, 0x00];
        let mut r = BitstreamReader::new(&data);
        r.read_bits(3).unwrap();
        r.align_to_byte();
        assert_eq!(r.bits_remaining(), 8);
        assert_eq!(r.read_u8().unwrap(), 0x00);
    }

    #[test]
    fn skip_bits() {
        let data = [0b10101010, 0b01010101];
        let mut r = BitstreamReader::new(&data);
        r.skip_bits(8).unwrap();
        assert_eq!(r.read_bits(8).unwrap(), 0b01010101);
    }

    #[test]
    fn cross_byte_boundary() {
        let data = [0b11001100, 0b10101010];
        let mut r = BitstreamReader::new(&data);
        assert_eq!(r.read_bits(4).unwrap(), 0b1100);
        assert_eq!(r.read_bits(8).unwrap(), 0b11001010);
        assert_eq!(r.read_bits(4).unwrap(), 0b1010);
    }

    #[test]
    fn error_display() {
        assert!(BitError::EndOfStream.to_string().contains("end"));
    }
}

pub struct VarCodec {
    total_encoded: u64,
    total_decoded: u64,
    bytes_encoded: u64,
    bytes_decoded: u64,
}

impl VarCodec {
    pub fn new() -> Self { Self { total_encoded: 0, total_decoded: 0, bytes_encoded: 0, bytes_decoded: 0 } }

    pub fn zigzag_encode(n: i64) -> u64 { ((n << 1) ^ (n >> 63)) as u64 }

    pub fn zigzag_decode(n: u64) -> i64 { ((n >> 1) as i64) ^ -((n & 1) as i64) }

    pub fn encode_varint(&mut self, value: u64) -> Vec<u8> {
        self.total_encoded += 1;
        let mut out = Vec::new();
        let mut v = value;
        loop {
            let mut byte = (v & 0x7F) as u8;
            v >>= 7;
            if v != 0 { byte |= 0x80; }
            out.push(byte);
            if v == 0 { break; }
        }
        self.bytes_encoded += out.len() as u64;
        out
    }

    pub fn decode_varint(&mut self, data: &[u8]) -> Result<(u64, usize), String> {
        self.total_decoded += 1;
        let mut value = 0u64;
        let mut shift = 0u32;
        for (i, &byte) in data.iter().enumerate() {
            value |= ((byte & 0x7F) as u64) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                self.bytes_decoded += (i + 1) as u64;
                return Ok((value, i + 1));
            }
            if shift >= 70 { return Err("varint too long".to_string()); }
        }
        Err("incomplete varint".to_string())
    }

    pub fn encode_signed(&mut self, value: i64) -> Vec<u8> { self.encode_varint(Self::zigzag_encode(value)) }

    pub fn decode_signed(&mut self, data: &[u8]) -> Result<(i64, usize), String> {
        let (v, n) = self.decode_varint(data)?;
        Ok((Self::zigzag_decode(v), n))
    }

    pub fn batch_encode(&mut self, values: &[u64]) -> Vec<u8> {
        let mut out = Vec::new();
        for &v in values { out.extend(self.encode_varint(v)); }
        out
    }

    pub fn batch_decode(&mut self, data: &[u8], count: usize) -> Result<Vec<u64>, String> {
        let mut result = Vec::with_capacity(count);
        let mut offset = 0usize;
        for _ in 0..count {
            let (v, n) = self.decode_varint(&data[offset..])?;
            result.push(v);
            offset += n;
        }
        Ok(result)
    }

    pub fn total_encoded(&self) -> u64 { self.total_encoded }
    pub fn total_decoded(&self) -> u64 { self.total_decoded }
    pub fn bytes_encoded(&self) -> u64 { self.bytes_encoded }
    pub fn bytes_decoded(&self) -> u64 { self.bytes_decoded }
    pub fn compression_ratio(&self) -> f64 { if self.bytes_decoded == 0 { 1.0 } else { self.bytes_encoded as f64 / self.bytes_decoded as f64 } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zigzag_roundtrip() {
        assert_eq!(VarCodec::zigzag_decode(VarCodec::zigzag_encode(-1)), -1);
        assert_eq!(VarCodec::zigzag_decode(VarCodec::zigzag_encode(42)), 42);
    }

    #[test]
    fn varint_roundtrip() {
        let mut vc = VarCodec::new();
        for &v in &[0u64, 127, 128, 300, 16384, u64::MAX] {
            let enc = vc.encode_varint(v);
            let (dec, n) = vc.decode_varint(&enc).unwrap();
            assert_eq!(dec, v);
            assert_eq!(n, enc.len());
        }
    }

    #[test]
    fn signed_roundtrip() {
        let mut vc = VarCodec::new();
        for &v in &[0i64, -1, 100, -100, i64::MIN, i64::MAX] {
            let enc = vc.encode_signed(v);
            let (dec, _) = vc.decode_signed(&enc).unwrap();
            assert_eq!(dec, v);
        }
    }

    #[test]
    fn batch_roundtrip() {
        let mut vc = VarCodec::new();
        let vals = vec![1u64, 100, 10000, 1000000];
        let enc = vc.batch_encode(&vals);
        let dec = vc.batch_decode(&enc, 4).unwrap();
        assert_eq!(dec, vals);
    }

    #[test]
    fn small_values_short() {
        let mut vc = VarCodec::new();
        assert_eq!(vc.encode_varint(0).len(), 1);
        assert_eq!(vc.encode_varint(127).len(), 1);
    }

    #[test]
    fn compression_ratio() {
        let mut vc = VarCodec::new();
        vc.batch_encode(&[1, 2, 3, 4, 5]);
        vc.batch_decode(&[0x01, 0x02, 0x03, 0x04, 0x05], 5).unwrap();
        assert!(vc.compression_ratio() <= 1.0);
    }

    #[test]
    fn stats() {
        let mut vc = VarCodec::new();
        vc.encode_varint(42); vc.decode_varint(&[42]).unwrap();
        assert_eq!(vc.total_encoded(), 1);
        assert_eq!(vc.total_decoded(), 1);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CobsErr {
    DecodeError { pos: usize },
    BufferTooSmall { needed: usize, have: usize },
}

impl std::fmt::Display for CobsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CobsErr::DecodeError { pos } => write!(f, "decode error at {pos}"),
            CobsErr::BufferTooSmall { needed, have } => write!(f, "need {needed} bytes, have {have}"),
        }
    }
}

impl std::error::Error for CobsErr {}

pub struct CobsBuf {
    total_encodes: u64,
    total_decodes: u64,
    total_bytes_in: u64,
    total_bytes_out: u64,
}

impl CobsBuf {
    pub fn new() -> Self { Self { total_encodes: 0, total_decodes: 0, total_bytes_in: 0, total_bytes_out: 0 } }

    pub fn encode(&mut self, data: &[u8]) -> Vec<u8> {
        self.total_encodes += 1;
        self.total_bytes_in += data.len() as u64;
        let mut out = Vec::with_capacity(data.len() + 2);
        let mut chunk_start = 0usize;
        let mut zero_pos = 0usize;
        out.push(0);
        for (i, &b) in data.iter().enumerate() {
            if b == 0 {
                out[zero_pos] = (i - chunk_start + 1) as u8;
                zero_pos = out.len();
                out.push(0);
                chunk_start = i + 1;
            } else {
                out.push(b);
                if i - chunk_start == 253 {
                    out[zero_pos] = 255;
                    zero_pos = out.len();
                    out.push(0);
                    chunk_start = i + 1;
                }
            }
        }
        out[zero_pos] = (data.len() - chunk_start + 1) as u8;
        out.push(0);
        self.total_bytes_out += out.len() as u64;
        out
    }

    pub fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>, CobsErr> {
        self.total_decodes += 1;
        if data.len() < 2 { return Err(CobsErr::DecodeError { pos: 0 }); }
        let mut out = Vec::new();
        let mut i = 0usize;
        while i < data.len() - 1 {
            let code = data[i] as usize;
            if code == 0 { return Err(CobsErr::DecodeError { pos: i }); }
            if i + code > data.len() { return Err(CobsErr::DecodeError { pos: i }); }
            for j in 1..code {
                if data[i + j] == 0 { return Err(CobsErr::DecodeError { pos: i + j }); }
                out.push(data[i + j]);
            }
            i += code;
            if code < 255 && i < data.len() - 1 { out.push(0); }
        }
        Ok(out)
    }

    pub fn encode_frame(&mut self, data: &[u8]) -> Vec<u8> { self.encode(data) }

    pub fn total_encodes(&self) -> u64 { self.total_encodes }
    pub fn total_decodes(&self) -> u64 { self.total_decodes }
    pub fn total_bytes_in(&self) -> u64 { self.total_bytes_in }
    pub fn total_bytes_out(&self) -> u64 { self.total_bytes_out }
    pub fn overhead(&self) -> f64 {
        if self.total_bytes_in == 0 { return 0.0; }
        (self.total_bytes_out as f64 - self.total_bytes_in as f64) / self.total_bytes_in as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_no_zeros() {
        let mut cb = CobsBuf::new();
        let enc = cb.encode(b"hello");
        let dec = cb.decode(&enc).unwrap();
        assert_eq!(dec, b"hello");
    }

    #[test]
    fn roundtrip_with_zeros() {
        let mut cb = CobsBuf::new();
        let enc = cb.encode(b"ab\x00cd\x00ef");
        let dec = cb.decode(&enc).unwrap();
        assert_eq!(dec, b"ab\x00cd\x00ef");
    }

    #[test]
    fn roundtrip_empty() {
        let mut cb = CobsBuf::new();
        let enc = cb.encode(b"");
        let dec = cb.decode(&enc).unwrap();
        assert_eq!(dec, b"");
    }

    #[test]
    fn roundtrip_all_zeros() {
        let mut cb = CobsBuf::new();
        let enc = cb.encode(b"\x00\x00\x00");
        let dec = cb.decode(&enc).unwrap();
        assert_eq!(dec, b"\x00\x00\x00");
    }

    #[test]
    fn roundtrip_long() {
        let mut cb = CobsBuf::new();
        let data: Vec<u8> = (0..300).map(|i| (i % 256) as u8).collect();
        let enc = cb.encode(&data);
        let dec = cb.decode(&enc).unwrap();
        assert_eq!(dec, data);
    }

    #[test]
    fn decode_error() {
        let mut cb = CobsBuf::new();
        assert!(cb.decode(&[0]).is_err());
    }

    #[test]
    fn overhead() {
        let mut cb = CobsBuf::new();
        cb.encode(b"hello");
        assert!(cb.overhead() > 0.0);
    }

    #[test]
    fn stats() {
        let mut cb = CobsBuf::new();
        let enc = cb.encode(b"test");
        cb.decode(&enc).unwrap();
        assert_eq!(cb.total_encodes(), 1);
        assert_eq!(cb.total_decodes(), 1);
    }

    #[test]
    fn error_display() { assert!(CobsErr::DecodeError { pos: 3 }.to_string().contains("decode")); }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumKind {
    Crc32,
    Sum8,
    Xor8,
    Fletcher16,
}

impl ChecksumKind {
    pub fn name(self) -> &'static str {
        match self {
            ChecksumKind::Crc32 => "crc32",
            ChecksumKind::Sum8 => "sum8",
            ChecksumKind::Xor8 => "xor8",
            ChecksumKind::Fletcher16 => "fletcher16",
        }
    }

    pub fn digest_size(self) -> usize {
        match self {
            ChecksumKind::Crc32 => 4,
            ChecksumKind::Sum8 => 1,
            ChecksumKind::Xor8 => 1,
            ChecksumKind::Fletcher16 => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumError {
    DigestSizeMismatch { expected: usize, got: usize },
}

impl std::fmt::Display for ChecksumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChecksumError::DigestSizeMismatch { expected, got } => {
                write!(f, "digest size mismatch: expected {expected}, got {got}")
            }
        }
    }
}

impl std::error::Error for ChecksumError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest {
    pub kind: ChecksumKind,
    pub bytes: Vec<u8>,
}

impl Digest {
    pub fn new(kind: ChecksumKind, bytes: Vec<u8>) -> Result<Self, ChecksumError> {
        if bytes.len() != kind.digest_size() {
            return Err(ChecksumError::DigestSizeMismatch {
                expected: kind.digest_size(),
                got: bytes.len(),
            });
        }
        Ok(Self { kind, bytes })
    }

    pub fn as_u32(&self) -> u32 {
        match self.bytes.len() {
            1 => self.bytes[0] as u32,
            2 => u16::from_be_bytes([self.bytes[0], self.bytes[1]]) as u32,
            4 => u32::from_be_bytes([self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3]]),
            _ => 0,
        }
    }
}

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

fn compute_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        let idx = ((crc ^ b as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ crc32_table()[idx];
    }
    !crc
}

fn compute_sum8(data: &[u8]) -> u8 {
    let mut sum: u8 = 0;
    for &b in data {
        sum = sum.wrapping_add(b);
    }
    sum
}

fn compute_xor8(data: &[u8]) -> u8 {
    let mut x: u8 = 0;
    for &b in data {
        x ^= b;
    }
    x
}

fn compute_fletcher16(data: &[u8]) -> u16 {
    let mut sum1: u16 = 0;
    let mut sum2: u16 = 0;
    for &b in data {
        sum1 = (sum1 + b as u16) % 255;
        sum2 = (sum2 + sum1) % 255;
    }
    (sum2 << 8) | sum1
}

pub fn checksum(kind: ChecksumKind, data: &[u8]) -> Digest {
    match kind {
        ChecksumKind::Crc32 => {
            let v = compute_crc32(data);
            Digest::new(kind, v.to_be_bytes().to_vec()).unwrap()
        }
        ChecksumKind::Sum8 => {
            let v = compute_sum8(data);
            Digest::new(kind, vec![v]).unwrap()
        }
        ChecksumKind::Xor8 => {
            let v = compute_xor8(data);
            Digest::new(kind, vec![v]).unwrap()
        }
        ChecksumKind::Fletcher16 => {
            let v = compute_fletcher16(data);
            Digest::new(kind, v.to_be_bytes().to_vec()).unwrap()
        }
    }
}

pub fn verify(expected: &Digest, data: &[u8]) -> bool {
    let actual = checksum(expected.kind, data);
    actual == *expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_known() {
        let d = checksum(ChecksumKind::Crc32, b"123456789");
        assert_eq!(d.as_u32(), 0xCBF43926);
    }

    #[test]
    fn crc32_empty() {
        let d = checksum(ChecksumKind::Crc32, b"");
        assert_eq!(d.as_u32(), 0x00000000);
    }

    #[test]
    fn sum8_known() {
        let d = checksum(ChecksumKind::Sum8, b"\x01\x02\x03");
        assert_eq!(d.as_u32(), 6);
    }

    #[test]
    fn sum8_wraps() {
        let d = checksum(ChecksumKind::Sum8, b"\xFF\x02");
        assert_eq!(d.as_u32(), 1);
    }

    #[test]
    fn xor8_known() {
        let d = checksum(ChecksumKind::Xor8, b"\xFF\x0F");
        assert_eq!(d.as_u32(), 0xF0);
    }

    #[test]
    fn xor8_empty() {
        let d = checksum(ChecksumKind::Xor8, b"");
        assert_eq!(d.as_u32(), 0);
    }

    #[test]
    fn fletcher16_known() {
        let d = checksum(ChecksumKind::Fletcher16, b"abcde");
        assert_ne!(d.as_u32(), 0);
        assert_eq!(d.bytes.len(), 2);
    }

    #[test]
    fn kind_name() {
        assert_eq!(ChecksumKind::Crc32.name(), "crc32");
        assert_eq!(ChecksumKind::Fletcher16.name(), "fletcher16");
    }

    #[test]
    fn digest_size() {
        assert_eq!(ChecksumKind::Crc32.digest_size(), 4);
        assert_eq!(ChecksumKind::Sum8.digest_size(), 1);
        assert_eq!(ChecksumKind::Xor8.digest_size(), 1);
        assert_eq!(ChecksumKind::Fletcher16.digest_size(), 2);
    }

    #[test]
    fn verify_matches() {
        let d = checksum(ChecksumKind::Crc32, b"hello");
        assert!(verify(&d, b"hello"));
    }

    #[test]
    fn verify_mismatch() {
        let d = checksum(ChecksumKind::Crc32, b"hello");
        assert!(!verify(&d, b"world"));
    }

    #[test]
    fn digest_size_mismatch() {
        let err = Digest::new(ChecksumKind::Crc32, vec![0x00]).unwrap_err();
        assert!(matches!(err, ChecksumError::DigestSizeMismatch { .. }));
    }

    #[test]
    fn error_display() {
        let e = ChecksumError::DigestSizeMismatch { expected: 4, got: 1 };
        assert!(e.to_string().contains("mismatch"));
    }
}

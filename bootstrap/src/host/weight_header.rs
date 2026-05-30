use super::bitstream::{BitError, BitstreamReader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderError {
    BadMagic,
    UnsupportedVersion(u8),
    BitstreamError(BitError),
    InvalidDimension { field: &'static str, value: u32 },
}

impl std::fmt::Display for HeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderError::BadMagic => write!(f, "bad magic number"),
            HeaderError::UnsupportedVersion(v) => write!(f, "unsupported version: {v}"),
            HeaderError::BitstreamError(e) => write!(f, "bitstream error: {e}"),
            HeaderError::InvalidDimension { field, value } => {
                write!(f, "invalid {field}: {value}")
            }
        }
    }
}

impl std::error::Error for HeaderError {}

impl From<BitError> for HeaderError {
    fn from(e: BitError) -> Self {
        HeaderError::BitstreamError(e)
    }
}

pub const MAGIC: u32 = 0x54325700;
pub const VERSION: u8 = 1;
pub const HEADER_SIZE: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightHeader {
    pub version: u8,
    pub layers: u16,
    pub neurons_per_layer: u16,
    pub bits_per_weight: u8,
    pub flags: u8,
    pub checksum_seed: u32,
}

impl WeightHeader {
    pub fn new(layers: u16, neurons: u16) -> Self {
        Self {
            version: VERSION,
            layers,
            neurons_per_layer: neurons,
            bits_per_weight: 2,
            flags: 0,
            checksum_seed: 0,
        }
    }

    pub fn with_checksum_seed(mut self, seed: u32) -> Self {
        self.checksum_seed = seed;
        self
    }

    pub fn with_bits_per_weight(mut self, bits: u8) -> Self {
        self.bits_per_weight = bits;
        self
    }

    pub fn total_weights(&self) -> u64 {
        self.layers as u64 * self.neurons_per_layer as u64
    }

    pub fn total_weight_bytes(&self) -> u64 {
        let total_bits = self.total_weights() * self.bits_per_weight as u64;
        (total_bits + 7) / 8
    }

    pub fn has_crc(&self) -> bool {
        (self.flags & 0x01) != 0
    }

    pub fn is_ternary(&self) -> bool {
        self.bits_per_weight == 2
    }

    pub fn encode(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&MAGIC.to_le_bytes());
        buf[4] = self.version;
        buf[5] = self.bits_per_weight;
        buf[6] = self.flags;
        buf[7] = 0;
        buf[8..10].copy_from_slice(&self.layers.to_le_bytes());
        buf[10..12].copy_from_slice(&self.neurons_per_layer.to_le_bytes());
        buf[12..16].copy_from_slice(&self.checksum_seed.to_le_bytes());
        buf
    }

    pub fn decode(data: &[u8]) -> Result<Self, HeaderError> {
        if data.len() < HEADER_SIZE {
            return Err(HeaderError::BitstreamError(BitError::EndOfStream));
        }
        let mut r = BitstreamReader::new(data);
        let magic = r.read_u32_le()?;
        if magic != MAGIC {
            return Err(HeaderError::BadMagic);
        }
        let version = r.read_u8()?;
        if version != VERSION {
            return Err(HeaderError::UnsupportedVersion(version));
        }
        let bits_per_weight = r.read_u8()?;
        let flags = r.read_u8()?;
        let _reserved = r.read_u8()?;
        let layers = r.read_u16_le()?;
        let neurons = r.read_u16_le()?;
        let checksum_seed = r.read_u32_le()?;
        if layers == 0 {
            return Err(HeaderError::InvalidDimension {
                field: "layers",
                value: 0,
            });
        }
        if neurons == 0 {
            return Err(HeaderError::InvalidDimension {
                field: "neurons_per_layer",
                value: 0,
            });
        }
        if bits_per_weight == 0 || bits_per_weight > 32 {
            return Err(HeaderError::InvalidDimension {
                field: "bits_per_weight",
                value: bits_per_weight as u32,
            });
        }
        Ok(Self {
            version,
            layers,
            neurons_per_layer: neurons,
            bits_per_weight,
            flags,
            checksum_seed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_header() {
        let h = WeightHeader::new(4, 128);
        assert_eq!(h.version, 1);
        assert_eq!(h.layers, 4);
        assert_eq!(h.neurons_per_layer, 128);
        assert_eq!(h.bits_per_weight, 2);
    }

    #[test]
    fn total_weights() {
        let h = WeightHeader::new(3, 256);
        assert_eq!(h.total_weights(), 768);
    }

    #[test]
    fn total_weight_bytes_ternary() {
        let h = WeightHeader::new(1, 32);
        assert_eq!(h.total_weight_bytes(), 8);
    }

    #[test]
    fn total_weight_bytes_4bit() {
        let h = WeightHeader::new(1, 16).with_bits_per_weight(4);
        assert_eq!(h.total_weight_bytes(), 8);
    }

    #[test]
    fn has_crc_flag() {
        let mut h = WeightHeader::new(1, 1);
        assert!(!h.has_crc());
        h.flags = 0x01;
        assert!(h.has_crc());
    }

    #[test]
    fn is_ternary() {
        let h = WeightHeader::new(1, 1);
        assert!(h.is_ternary());
        let h2 = WeightHeader::new(1, 1).with_bits_per_weight(4);
        assert!(!h2.is_ternary());
    }

    #[test]
    fn encode_decode_roundtrip() {
        let h = WeightHeader::new(8, 512).with_checksum_seed(0xDEADBEEF);
        let encoded = h.encode();
        let decoded = WeightHeader::decode(&encoded).unwrap();
        assert_eq!(decoded, h);
    }

    #[test]
    fn decode_bad_magic() {
        let mut buf = [0u8; 16];
        buf[0..4].copy_from_slice(&0x00000000u32.to_be_bytes());
        assert!(matches!(WeightHeader::decode(&buf), Err(HeaderError::BadMagic)));
    }

    #[test]
    fn decode_bad_version() {
        let h = WeightHeader::new(1, 1);
        let mut buf = h.encode();
        buf[4] = 99;
        assert!(matches!(WeightHeader::decode(&buf), Err(HeaderError::UnsupportedVersion(99))));
    }

    #[test]
    fn decode_zero_layers() {
        let h = WeightHeader::new(1, 1);
        let mut buf = h.encode();
        buf[8..10].copy_from_slice(&0u16.to_le_bytes());
        assert!(matches!(WeightHeader::decode(&buf), Err(HeaderError::InvalidDimension { field: "layers", .. })));
    }

    #[test]
    fn decode_zero_neurons() {
        let h = WeightHeader::new(1, 1);
        let mut buf = h.encode();
        buf[10..12].copy_from_slice(&0u16.to_le_bytes());
        assert!(matches!(WeightHeader::decode(&buf), Err(HeaderError::InvalidDimension { field: "neurons_per_layer", .. })));
    }

    #[test]
    fn decode_too_short() {
        let buf = [0u8; 8];
        assert!(matches!(WeightHeader::decode(&buf), Err(HeaderError::BitstreamError(_))));
    }

    #[test]
    fn encode_header_size() {
        let h = WeightHeader::new(2, 64);
        assert_eq!(h.encode().len(), HEADER_SIZE);
    }

    #[test]
    fn builder_chain() {
        let h = WeightHeader::new(4, 256)
            .with_bits_per_weight(4)
            .with_checksum_seed(0x12345678);
        assert_eq!(h.bits_per_weight, 4);
        assert_eq!(h.checksum_seed, 0x12345678);
        let encoded = h.encode();
        let decoded = WeightHeader::decode(&encoded).unwrap();
        assert_eq!(decoded.bits_per_weight, 4);
    }

    #[test]
    fn error_display() {
        let e = HeaderError::BadMagic;
        assert!(e.to_string().contains("magic"));
        let e = HeaderError::UnsupportedVersion(5);
        assert!(e.to_string().contains("version"));
        let e = HeaderError::InvalidDimension { field: "layers", value: 0 };
        assert!(e.to_string().contains("layers"));
    }
}

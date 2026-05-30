#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageError {
    BadMagic,
    UnsupportedVersion(u8),
    BadHeaderSize { have: u16, expected: u16 },
    SectionTableOverflow { count: u8, max: u8 },
    SectionOutOfBounds { index: usize },
    TruncatedSection { index: usize, need: usize, have: usize },
    CrcMismatch { expected: u32, got: u32 },
    MissingSection,
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::BadMagic => write!(f, "bad firmware magic"),
            ImageError::UnsupportedVersion(v) => write!(f, "unsupported version: {v}"),
            ImageError::BadHeaderSize { have, expected } => {
                write!(f, "bad header size: {have} != {expected}")
            }
            ImageError::SectionTableOverflow { count, max } => {
                write!(f, "too many sections: {count} > {max}")
            }
            ImageError::SectionOutOfBounds { index } => {
                write!(f, "section out of bounds: {index}")
            }
            ImageError::TruncatedSection { index, need, have } => {
                write!(f, "truncated section {index}: need {need}, have {have}")
            }
            ImageError::CrcMismatch { expected, got } => {
                write!(f, "CRC mismatch: expected 0x{expected:08X}, got 0x{got:08X}")
            }
            ImageError::MissingSection => write!(f, "missing required section"),
        }
    }
}

impl std::error::Error for ImageError {}

pub const MAGIC: u32 = 0x54465700;
pub const VERSION: u8 = 1;
pub const HEADER_SIZE: u16 = 128;
pub const MAX_SECTIONS: u8 = 8;

pub const SECTION_CODE: &[u8; 4] = b"CODE";
pub const SECTION_DATA: &[u8; 4] = b"DATA";
pub const SECTION_CFG: &[u8; 4] = b"CFG\0";
pub const SECTION_WGT: &[u8; 4] = b"WGT\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionHeader {
    pub tag: [u8; 4],
    pub offset: u32,
    pub size: u32,
    pub crc32: u32,
}

impl SectionHeader {
    pub fn new(tag: &[u8; 4], offset: u32, size: u32, crc32: u32) -> Self {
        Self {
            tag: *tag,
            offset,
            size,
            crc32,
        }
    }

    pub fn tag_str(&self) -> &str {
        std::str::from_utf8(&self.tag).unwrap_or("????")
    }

    pub fn end(&self) -> u32 {
        self.offset + self.size
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmwareHeader {
    pub version: u8,
    pub num_sections: u8,
    pub target_chip: u32,
    pub build_timestamp: u64,
    pub sections: Vec<SectionHeader>,
}

impl FirmwareHeader {
    pub fn find_section(&self, tag: &[u8; 4]) -> Option<&SectionHeader> {
        self.sections.iter().find(|s| &s.tag == tag)
    }

    pub fn require_section(&self, tag: &[u8; 4]) -> Result<&SectionHeader, ImageError> {
        let name = std::str::from_utf8(tag).unwrap_or("????");
        self.find_section(tag)
            .ok_or(ImageError::MissingSection)
    }
}

#[derive(Debug, Clone)]
pub struct FirmwareImage {
    pub header: FirmwareHeader,
    pub raw: Vec<u8>,
}

impl FirmwareImage {
    pub fn parse(data: &[u8]) -> Result<Self, ImageError> {
        if data.len() < HEADER_SIZE as usize {
            return Err(ImageError::BadHeaderSize {
                have: data.len() as u16,
                expected: HEADER_SIZE,
            });
        }
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != MAGIC {
            return Err(ImageError::BadMagic);
        }
        let version = data[4];
        if version != VERSION {
            return Err(ImageError::UnsupportedVersion(version));
        }
        let hdr_size = u16::from_le_bytes([data[6], data[7]]);
        if hdr_size != HEADER_SIZE {
            return Err(ImageError::BadHeaderSize {
                have: hdr_size,
                expected: HEADER_SIZE,
            });
        }
        let num_sections = data[5];
        if num_sections > MAX_SECTIONS {
            return Err(ImageError::SectionTableOverflow {
                count: num_sections,
                max: MAX_SECTIONS,
            });
        }
        let target_chip = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let build_timestamp = u64::from_le_bytes([
            data[12], data[13], data[14], data[15],
            data[16], data[17], data[18], data[19],
        ]);
        let mut sections = Vec::with_capacity(num_sections as usize);
        for i in 0..num_sections as usize {
            let base = 20 + i * 12;
            if base + 12 > data.len() {
                return Err(ImageError::TruncatedSection {
                    index: i,
                    need: 12,
                    have: data.len().saturating_sub(base),
                });
            }
            let tag: [u8; 4] = [data[base], data[base + 1], data[base + 2], data[base + 3]];
            let offset = u32::from_le_bytes([
                data[base + 4],
                data[base + 5],
                data[base + 6],
                data[base + 7],
            ]);
            let size = u32::from_le_bytes([
                data[base + 8],
                data[base + 9],
                data[base + 10],
                data[base + 11],
            ]);
            let end = (offset as usize) + (size as usize);
            if end > data.len() {
                return Err(ImageError::TruncatedSection {
                    index: i,
                    need: end,
                    have: data.len(),
                });
            }
            let section_data = &data[offset as usize..end];
            let crc = crc32(section_data);
            sections.push(SectionHeader {
                tag,
                offset,
                size,
                crc32: crc,
            });
        }
        let header = FirmwareHeader {
            version,
            num_sections,
            target_chip,
            build_timestamp,
            sections,
        };
        Ok(Self {
            header,
            raw: data.to_vec(),
        })
    }

    pub fn section_data(&self, section: &SectionHeader) -> &[u8] {
        let start = section.offset as usize;
        let end = start + section.size as usize;
        &self.raw[start..end]
    }

    pub fn verify_crc(&self, section: &SectionHeader) -> bool {
        let data = self.section_data(section);
        crc32(data) == section.crc32
    }

    pub fn total_image_size(&self) -> usize {
        self.raw.len()
    }

    pub fn code_size(&self) -> Option<u32> {
        self.header.find_section(SECTION_CODE).map(|s| s.size)
    }
}

fn crc32(data: &[u8]) -> u32 {
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
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        let idx = ((crc ^ b as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ TABLE[idx];
    }
    !crc
}

pub fn build_image(
    target_chip: u32,
    build_timestamp: u64,
    sections: &[( &[u8; 4], &[u8] )],
) -> Vec<u8> {
    let num_sections = sections.len() as u8;
    let header_len = 20 + (num_sections as usize) * 12;
    let padding = HEADER_SIZE as usize - header_len;
    let data_start = header_len + padding;
    let mut offsets = Vec::new();
    let mut cur_offset = data_start as u32;
    for (_, sdata) in sections {
        offsets.push(cur_offset);
        cur_offset += sdata.len() as u32;
    }
    let mut buf = Vec::new();
    buf.extend_from_slice(&MAGIC.to_le_bytes());
    buf.push(VERSION);
    buf.push(num_sections);
    buf.extend_from_slice(&HEADER_SIZE.to_le_bytes());
    buf.extend_from_slice(&target_chip.to_le_bytes());
    buf.extend_from_slice(&build_timestamp.to_le_bytes());
    for (i, (tag, sdata)) in sections.iter().enumerate() {
        buf.extend_from_slice(*tag);
        buf.extend_from_slice(&offsets[i].to_le_bytes());
        buf.extend_from_slice(&(sdata.len() as u32).to_le_bytes());
    }
    while buf.len() < data_start {
        buf.push(0);
    }
    for (_, sdata) in sections {
        buf.extend_from_slice(sdata);
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_image() -> Vec<u8> {
        build_image(
            0x10000000,
            12345,
            &[
                (SECTION_CODE, b"code bytes here"),
                (SECTION_DATA, b"data bytes here"),
            ],
        )
    }

    #[test]
    fn parse_valid_image() {
        let img_data = make_test_image();
        let img = FirmwareImage::parse(&img_data).unwrap();
        assert_eq!(img.header.version, 1);
        assert_eq!(img.header.num_sections, 2);
        assert_eq!(img.header.sections.len(), 2);
    }

    #[test]
    fn parse_bad_magic() {
        let mut img_data = make_test_image();
        img_data[0] = 0xFF;
        assert!(matches!(FirmwareImage::parse(&img_data), Err(ImageError::BadMagic)));
    }

    #[test]
    fn parse_bad_version() {
        let mut img_data = make_test_image();
        img_data[4] = 99;
        assert!(matches!(
            FirmwareImage::parse(&img_data),
            Err(ImageError::UnsupportedVersion(99))
        ));
    }

    #[test]
    fn parse_too_short() {
        let buf = [0u8; 10];
        assert!(matches!(
            FirmwareImage::parse(&buf),
            Err(ImageError::BadHeaderSize { .. })
        ));
    }

    #[test]
    fn section_data_access() {
        let img_data = make_test_image();
        let img = FirmwareImage::parse(&img_data).unwrap();
        let code_sec = img.header.find_section(SECTION_CODE).unwrap();
        let data = img.section_data(code_sec);
        assert_eq!(data, b"code bytes here");
    }

    #[test]
    fn find_section() {
        let img_data = make_test_image();
        let img = FirmwareImage::parse(&img_data).unwrap();
        assert!(img.header.find_section(SECTION_CODE).is_some());
        assert!(img.header.find_section(SECTION_WGT).is_none());
    }

    #[test]
    fn require_section_present() {
        let img_data = make_test_image();
        let img = FirmwareImage::parse(&img_data).unwrap();
        assert!(img.header.require_section(SECTION_CODE).is_ok());
    }

    #[test]
    fn require_section_missing() {
        let img_data = make_test_image();
        let img = FirmwareImage::parse(&img_data).unwrap();
        assert!(matches!(
            img.header.require_section(SECTION_WGT),
            Err(ImageError::MissingSection)
        ));
    }

    #[test]
    fn verify_crc_valid() {
        let img_data = make_test_image();
        let img = FirmwareImage::parse(&img_data).unwrap();
        for sec in &img.header.sections {
            assert!(img.verify_crc(sec));
        }
    }

    #[test]
    fn code_size() {
        let img_data = make_test_image();
        let img = FirmwareImage::parse(&img_data).unwrap();
        assert_eq!(img.code_size(), Some(15));
    }

    #[test]
    fn total_image_size() {
        let img_data = make_test_image();
        let img = FirmwareImage::parse(&img_data).unwrap();
        assert_eq!(img.total_image_size(), img_data.len());
    }

    #[test]
    fn section_header_tag_str() {
        let sh = SectionHeader::new(SECTION_CODE, 0, 0, 0);
        assert_eq!(sh.tag_str(), "CODE");
    }

    #[test]
    fn section_header_end() {
        let sh = SectionHeader::new(SECTION_CODE, 100, 50, 0);
        assert_eq!(sh.end(), 150);
    }

    #[test]
    fn error_display() {
        assert!(ImageError::BadMagic.to_string().contains("magic"));
        assert!(ImageError::CrcMismatch { expected: 1, got: 2 }.to_string().contains("CRC"));
        assert!(ImageError::SectionTableOverflow { count: 10, max: 8 }.to_string().contains("10"));
        assert!(ImageError::MissingSection.to_string().contains("missing"));
    }
}

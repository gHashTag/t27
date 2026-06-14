use std::io::Read;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    IoError(String),
    InvalidSize { expected: usize, actual: usize },
    ChecksumFailed { word_index: usize, expected: u32, actual: u32 },
    InvalidWord { word_index: usize, reason: String },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::IoError(s) => write!(f, "IO error: {s}"),
            LoadError::InvalidSize { expected, actual } => {
                write!(f, "size mismatch: expected {expected} words, got {actual}")
            }
            LoadError::ChecksumFailed { word_index, expected, actual } => {
                write!(f, "checksum failed at word {word_index}: expected {expected:#010x}, got {actual:#010x}")
            }
            LoadError::InvalidWord { word_index, reason } => {
                write!(f, "invalid word at index {word_index}: {reason}")
            }
        }
    }
}

impl std::error::Error for LoadError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordFormat {
    RawBinary,
    PackedTernary,
    Crc32Trailer,
}

#[derive(Debug, Clone)]
pub struct LoadConfig {
    pub format: WordFormat,
    pub max_words: usize,
    pub verify_checksum: bool,
    pub validate_ternary: bool,
}

impl Default for LoadConfig {
    fn default() -> Self {
        Self {
            format: WordFormat::RawBinary,
            max_words: 4096,
            verify_checksum: false,
            validate_ternary: false,
        }
    }
}

impl LoadConfig {
    pub fn raw() -> Self {
        Self::default()
    }

    pub fn with_checksum(mut self) -> Self {
        self.verify_checksum = true;
        self.format = WordFormat::Crc32Trailer;
        self
    }

    pub fn ternary(mut self) -> Self {
        self.validate_ternary = true;
        self.format = WordFormat::PackedTernary;
        self
    }
}

#[derive(Debug, Clone)]
pub struct LoadReport {
    pub words_loaded: usize,
    pub bytes_read: usize,
    pub checksum: Option<u32>,
    pub format: WordFormat,
    pub valid: bool,
}

pub fn load_words(data: &[u8], config: &LoadConfig) -> Result<(Vec<u64>, LoadReport), LoadError> {
    if data.len() % 8 != 0 {
        return Err(LoadError::IoError(format!(
            "data length {} is not a multiple of 8",
            data.len()
        )));
    }

    let total_words = data.len() / 8;
    if total_words > config.max_words {
        return Err(LoadError::InvalidSize {
            expected: config.max_words,
            actual: total_words,
        });
    }

    let mut words: Vec<u64> = Vec::with_capacity(total_words);
    for chunk in data.chunks_exact(8) {
        let word = u64::from_le_bytes([
            chunk[0], chunk[1], chunk[2], chunk[3],
            chunk[4], chunk[5], chunk[6], chunk[7],
        ]);
        words.push(word);
    }

    if config.validate_ternary {
        for (i, &word) in words.iter().enumerate() {
            let upper = word >> 54;
            if upper != 0 {
                return Err(LoadError::InvalidWord {
                    word_index: i,
                    reason: format!("reserved bits non-zero: {upper:#010x}"),
                });
            }
            for trit_idx in 0..27 {
                let trit = ((word >> (trit_idx * 2)) & 0x3) as u8;
                if trit == 0b11 {
                    return Err(LoadError::InvalidWord {
                        word_index: i,
                        reason: format!("invalid trit {trit_idx}: 0b11"),
                    });
                }
            }
        }
    }

    let checksum = if config.verify_checksum && words.len() >= 2 {
        let data_words = &words[..words.len() - 1];
        let expected = crc32_checksum(data_words);
        let actual = words[words.len() - 1] as u32;
        if expected != actual {
            return Err(LoadError::ChecksumFailed {
                word_index: words.len() - 1,
                expected,
                actual,
            });
        }
        words = words[..words.len() - 1].to_vec();
        Some(expected)
    } else {
        None
    };

    let words_loaded = words.len();
    let report = LoadReport {
        words_loaded,
        bytes_read: data.len(),
        checksum,
        format: config.format,
        valid: true,
    };

    Ok((words, report))
}

pub fn load_from_reader<R: Read>(
    mut reader: R,
    config: &LoadConfig,
) -> Result<(Vec<u64>, LoadReport), LoadError> {
    let mut buf = Vec::new();
    reader
        .read_to_end(&mut buf)
        .map_err(|e| LoadError::IoError(e.to_string()))?;
    load_words(&buf, config)
}

fn crc32_checksum(words: &[u64]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &word in words {
        for &byte in &word.to_le_bytes() {
            let idx = ((crc ^ byte as u32) & 0xFF) as usize;
            crc = (crc >> 8) ^ TABLE[idx];
        }
    }
    !crc
}

const TABLE: [u32; 256] = make_table();

const fn make_table() -> [u32; 256] {
    let mut t = [0u32; 256];
    let mut i = 0usize;
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
}

/// Serialize host words to the little-endian byte stream the loader consumes.
///
/// Intentional public API and the exact inverse of [`load_words`]: it is the
/// encode half of the loader's round-trip contract and is exercised by the
/// round-trip tests below. Production only ever loads pre-built weight blobs,
/// so the encoder is dead in non-test builds; the annotation documents that
/// without removing the symbol or breaking the round-trip tests.
#[cfg_attr(not(test), allow(dead_code))]
pub fn encode_words(words: &[u64]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(words.len() * 8);
    for &word in words {
        buf.extend_from_slice(&word.to_le_bytes());
    }
    buf
}

/// Serialize host words with a trailing CRC32, matching the loader's checksum
/// format. Intentional public API: the CRC-aware inverse of [`load_words`]
/// under [`LoadConfig::with_checksum`], exercised by the round-trip tests.
/// Dead in non-test builds (production only loads); annotated, not removed.
#[cfg_attr(not(test), allow(dead_code))]
pub fn encode_with_crc(words: &[u64]) -> Vec<u8> {
    let crc = crc32_checksum(words);
    let mut buf = encode_words(words);
    buf.extend_from_slice(&(crc as u64).to_le_bytes());
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_empty_data() {
        let (words, report) = load_words(&[], &LoadConfig::default()).unwrap();
        assert!(words.is_empty());
        assert_eq!(report.words_loaded, 0);
    }

    #[test]
    fn load_single_word() {
        let data = 42u64.to_le_bytes();
        let (words, report) = load_words(&data, &LoadConfig::default()).unwrap();
        assert_eq!(words.len(), 1);
        assert_eq!(words[0], 42);
        assert_eq!(report.bytes_read, 8);
    }

    #[test]
    fn load_multiple_words() {
        let mut data = Vec::new();
        for v in [1u64, 2, 3, 4, 5] {
            data.extend_from_slice(&v.to_le_bytes());
        }
        let (words, report) = load_words(&data, &LoadConfig::default()).unwrap();
        assert_eq!(words.len(), 5);
        assert_eq!(report.words_loaded, 5);
        assert_eq!(words[0], 1);
        assert_eq!(words[4], 5);
    }

    #[test]
    fn load_non_aligned_errors() {
        let data = [1, 2, 3];
        let result = load_words(&data, &LoadConfig::default());
        assert!(matches!(result, Err(LoadError::IoError(_))));
    }

    #[test]
    fn load_exceeds_max_errors() {
        let data = vec![0u8; 8 * 5000];
        let config = LoadConfig {
            max_words: 100,
            ..Default::default()
        };
        assert!(matches!(load_words(&data, &config), Err(LoadError::InvalidSize { .. })));
    }

    #[test]
    fn load_with_crc_pass() {
        let words = vec![0xAA00AA00AA00AA00u64, 0xBB00BB00BB00BB00u64];
        let encoded = encode_with_crc(&words);
        let config = LoadConfig::default().with_checksum();
        let (loaded, report) = load_words(&encoded, &config).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0], words[0]);
        assert!(report.checksum.is_some());
    }

    #[test]
    fn load_with_crc_tampered() {
        let words = vec![0xDEADu64, 0xBEEFu64];
        let mut encoded = encode_with_crc(&words);
        encoded[0] ^= 0xFF;
        let config = LoadConfig::default().with_checksum();
        let result = load_words(&encoded, &config);
        assert!(matches!(result, Err(LoadError::ChecksumFailed { .. })));
    }

    #[test]
    fn load_ternary_valid() {
        let mut words = Vec::new();
        for _ in 0..4 {
            let mut w: u64 = 0;
            for t in 0..27u64 {
                let val = (t % 3) as u64;
                w |= val << (t * 2);
            }
            words.push(w);
        }
        let data = encode_words(&words);
        let config = LoadConfig::default().ternary();
        let (loaded, _) = load_words(&data, &config).unwrap();
        assert_eq!(loaded.len(), 4);
    }

    #[test]
    fn load_ternary_reserved_bits() {
        let bad_word: u64 = (1u64 << 55) | 0xFF;
        let data = encode_words(&[bad_word]);
        let config = LoadConfig::default().ternary();
        let result = load_words(&data, &config);
        assert!(matches!(result, Err(LoadError::InvalidWord { .. })));
    }

    #[test]
    fn load_ternary_invalid_trit() {
        let mut w: u64 = 0;
        w |= 0b11u64 << 0;
        let data = encode_words(&[w]);
        let config = LoadConfig::default().ternary();
        let result = load_words(&data, &config);
        match result {
            Err(LoadError::InvalidWord { reason, .. }) => assert!(reason.contains("0b11")),
            other => panic!("expected InvalidWord, got {:?}", other),
        }
    }

    #[test]
    fn encode_round_trip() {
        let original = vec![100u64, 200, 300];
        let encoded = encode_words(&original);
        let (decoded, _) = load_words(&encoded, &LoadConfig::default()).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn encode_crc_round_trip() {
        let original = vec![0x1234u64, 0x5678u64];
        let encoded = encode_with_crc(&original);
        assert_eq!(encoded.len(), 24);
        let config = LoadConfig::default().with_checksum();
        let (decoded, report) = load_words(&encoded, &config).unwrap();
        assert_eq!(decoded, original);
        assert!(report.checksum.is_some());
    }

    #[test]
    fn config_builders() {
        let raw = LoadConfig::raw();
        assert_eq!(raw.format, WordFormat::RawBinary);
        assert!(!raw.verify_checksum);
        assert!(!raw.validate_ternary);

        let crc = LoadConfig::default().with_checksum();
        assert!(crc.verify_checksum);
        assert_eq!(crc.format, WordFormat::Crc32Trailer);

        let tern = LoadConfig::default().ternary();
        assert!(tern.validate_ternary);
        assert_eq!(tern.format, WordFormat::PackedTernary);
    }

    #[test]
    fn error_display() {
        let e = LoadError::IoError("bad".into());
        assert!(e.to_string().contains("IO"));
        let e = LoadError::InvalidSize { expected: 10, actual: 20 };
        assert!(e.to_string().contains("20"));
        let e = LoadError::ChecksumFailed { word_index: 5, expected: 1, actual: 2 };
        assert!(e.to_string().contains("5"));
        let e = LoadError::InvalidWord { word_index: 3, reason: "bad trit".into() };
        assert!(e.to_string().contains("bad trit"));
    }

    #[test]
    fn load_from_reader_test() {
        let words = vec![0xABCDu64, 0xEF01u64];
        let data = encode_words(&words);
        let cursor = std::io::Cursor::new(data);
        let (loaded, report) = load_from_reader(cursor, &LoadConfig::default()).unwrap();
        assert_eq!(loaded, words);
        assert_eq!(report.words_loaded, 2);
    }
}
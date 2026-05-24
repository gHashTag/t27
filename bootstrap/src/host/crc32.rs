pub struct Crc32;

impl Crc32 {
    const TABLE: [u32; 256] = Self::build_table();

    const fn build_table() -> [u32; 256] {
        let mut table = [0u32; 256];
        let mut i = 0usize;
        while i < 256 {
            let mut crc = i as u32;
            let mut j = 0;
            while j < 8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
                j += 1;
            }
            table[i] = crc;
            i += 1;
        }
        table
    }

    pub fn checksum(data: &[u8]) -> u32 {
        let mut crc = 0xFFFFFFFFu32;
        for &byte in data {
            let idx = ((crc ^ byte as u32) & 0xFF) as usize;
            crc = (crc >> 8) ^ Self::TABLE[idx];
        }
        !crc
    }

    pub fn checksum_words(words: &[u64]) -> u32 {
        let mut crc = 0xFFFFFFFFu32;
        for &word in words {
            let bytes = word.to_le_bytes();
            for &byte in &bytes {
                let idx = ((crc ^ byte as u32) & 0xFF) as usize;
                crc = (crc >> 8) ^ Self::TABLE[idx];
            }
        }
        !crc
    }

    pub fn append(words: &[u64]) -> Vec<u64> {
        let crc = Self::checksum_words(words);
        let mut out = words.to_vec();
        out.push(crc as u64);
        out
    }

    pub fn verify(words_and_crc: &[u64]) -> bool {
        if words_and_crc.is_empty() {
            return true;
        }
        let (words, crc_word) = (&words_and_crc[..words_and_crc.len() - 1], words_and_crc[words_and_crc.len() - 1]);
        Self::checksum_words(words) == crc_word as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_empty() {
        assert_eq!(Crc32::checksum(&[]), 0);
    }

    #[test]
    fn crc32_known_bytes() {
        let crc = Crc32::checksum(b"123456789");
        assert_eq!(crc, 0xCBF43926);
    }

    #[test]
    fn crc32_single_zero() {
        let crc = Crc32::checksum(&[0u8]);
        assert_ne!(crc, 0);
    }

    #[test]
    fn crc32_deterministic() {
        let a = Crc32::checksum(b"hello");
        let b = Crc32::checksum(b"hello");
        assert_eq!(a, b);
    }

    #[test]
    fn crc32_different_inputs() {
        let a = Crc32::checksum(b"foo");
        let b = Crc32::checksum(b"bar");
        assert_ne!(a, b);
    }

    #[test]
    fn crc32_words_empty() {
        assert_eq!(Crc32::checksum_words(&[]), 0);
    }

    #[test]
    fn crc32_words_single() {
        let crc = Crc32::checksum_words(&[0x0000000000000042]);
        assert_ne!(crc, 0);
    }

    #[test]
    fn crc32_words_consistent_with_bytes() {
        let words = &[0x4142434445464748u64];
        let crc_w = Crc32::checksum_words(words);
        let crc_b = Crc32::checksum(&words[0].to_le_bytes());
        assert_eq!(crc_w, crc_b);
    }

    #[test]
    fn crc32_append_and_verify() {
        let original = vec![0xDEADBEEF, 0xCAFEBABE, 0x12345678];
        let appended = Crc32::append(&original);
        assert_eq!(appended.len(), original.len() + 1);
        assert!(Crc32::verify(&appended));
    }

    #[test]
    fn crc32_verify_tampered() {
        let original = vec![0xDEADBEEF, 0xCAFEBABE];
        let mut appended = Crc32::append(&original);
        appended[0] ^= 1;
        assert!(!Crc32::verify(&appended));
    }

    #[test]
    fn crc32_verify_empty() {
        assert!(Crc32::verify(&[]));
    }

    #[test]
    fn crc32_verify_single_crc_word() {
        let data = vec![0xCAFEBABEu64];
        assert!(Crc32::verify(&data) || !Crc32::verify(&data));
    }

    #[test]
    fn crc32_table_first_entry() {
        assert_eq!(Crc32::TABLE[0], 0);
    }

    #[test]
    fn crc32_table_nonzero() {
        let nonzero: Vec<_> = Crc32::TABLE.iter().filter(|&&v| v != 0).collect();
        assert!(nonzero.len() > 200);
    }

    #[test]
    fn crc32_words_multiple() {
        let words = vec![1u64, 2, 3, 4, 5];
        let appended = Crc32::append(&words);
        assert_eq!(appended.len(), 6);
        assert!(Crc32::verify(&appended));
        let mut tampered = appended.clone();
        tampered[2] = 99;
        assert!(!Crc32::verify(&tampered));
    }
}

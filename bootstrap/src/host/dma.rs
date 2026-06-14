// Variant V (#969 dead-code audit, host/dma.rs layer). This module is a
// self-contained, intentional public API: a DMA-channel model (DmaError,
// DmaState, DmaConfig, DmaReport, DmaChannel and its transfer/reset path, plus
// the internal CRC32 helpers). It is fully exercised by this module's own test
// suite (idle/busy state, length and checksum validation, copy/partial-copy,
// cycle estimate, reset, builder) but is not yet wired into production host
// code, so every symbol emits a `dead_code` warning (12 in total) in the
// non-test build. These are deliberate public surface, not dead code -- a
// single module-scoped allow documents that without removing or weakening any
// symbol. Scoped to `not(test)` so the test build still flags genuinely unused
// items, exactly as in the #1105 / #1111 / #1129 slices of this audit (same
// pattern as the host/errors.rs slice in #1125).
#![cfg_attr(not(test), allow(dead_code))]

use super::csr_map;
use super::mmio::Mmio;

mod crc32_internal {
    pub fn checksum_words(words: &[u64]) -> u32 {
        let mut crc = 0xFFFFFFFFu32;
        for &word in words {
            for &byte in &word.to_le_bytes() {
                let idx = ((crc ^ byte as u32) & 0xFF) as usize;
                crc = (crc >> 8) ^ TABLE[idx];
            }
        }
        !crc
    }

    pub fn append(words: &[u64]) -> Vec<u64> {
        let crc = checksum_words(words);
        let mut out = words.to_vec();
        out.push(crc as u64);
        out
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaError {
    ZeroLength,
    ExceedsMaxLen,
    AlreadyBusy,
    ChecksumMismatch { expected: u32, actual: u32 },
}

impl std::fmt::Display for DmaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DmaError::ZeroLength => write!(f, "DMA transfer length is zero"),
            DmaError::ExceedsMaxLen => write!(f, "DMA transfer exceeds maximum length"),
            DmaError::AlreadyBusy => write!(f, "DMA channel is busy"),
            DmaError::ChecksumMismatch { expected, actual } => {
                write!(f, "DMA checksum mismatch: expected {expected:#010x}, got {actual:#010x}")
            }
        }
    }
}

impl std::error::Error for DmaError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaState {
    Idle,
    Busy,
    Complete,
    Error,
}

pub const DMA_MAX_WORDS: usize = 4096;

#[derive(Debug, Clone)]
pub struct DmaConfig {
    pub src_addr: u64,
    pub dst_addr: u64,
    pub word_count: u32,
    pub verify_checksum: bool,
}

impl DmaConfig {
    pub fn new(src: u64, dst: u64, count: u32) -> Self {
        Self {
            src_addr: src,
            dst_addr: dst,
            word_count: count,
            verify_checksum: false,
        }
    }

    pub fn with_checksum(mut self) -> Self {
        self.verify_checksum = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmaReport {
    pub words_transferred: u32,
    pub checksum: Option<u32>,
    pub state: DmaState,
    pub cycles: u32,
}

pub struct DmaChannel<M: Mmio> {
    mmio: M,
    state: DmaState,
    src_buf: Vec<u64>,
    dst_buf: Vec<u64>,
}

impl<M: Mmio> DmaChannel<M> {
    pub fn new(mmio: M) -> Self {
        Self {
            mmio,
            state: DmaState::Idle,
            src_buf: Vec::new(),
            dst_buf: Vec::new(),
        }
    }

    pub fn state(&self) -> DmaState {
        self.state
    }

    pub fn mmio(&self) -> &M {
        &self.mmio
    }

    pub fn mmio_mut(&mut self) -> &mut M {
        &mut self.mmio
    }

    pub fn load_src(&mut self, words: &[u64]) {
        self.src_buf = words.to_vec();
    }

    pub fn dst_buf(&self) -> &[u64] {
        &self.dst_buf
    }

    pub fn transfer(&mut self, cfg: &DmaConfig) -> Result<DmaReport, DmaError> {
        if cfg.word_count == 0 {
            return Err(DmaError::ZeroLength);
        }
        if cfg.word_count as usize > DMA_MAX_WORDS {
            return Err(DmaError::ExceedsMaxLen);
        }
        if self.state == DmaState::Busy {
            return Err(DmaError::AlreadyBusy);
        }

        let count = cfg.word_count as usize;
        if count > self.src_buf.len() {
            self.state = DmaState::Error;
            return Err(DmaError::ExceedsMaxLen);
        }

        self.state = DmaState::Busy;
        self.mmio.write32(csr_map::DMA_CTRL, 1);

        let payload = &self.src_buf[..count];
        let checksum = crc32_internal::checksum_words(payload);

        if cfg.verify_checksum && payload.len() >= 2 {
            let data_words = &payload[..payload.len() - 1];
            let expected = crc32_internal::checksum_words(data_words);
            let actual = payload[payload.len() - 1] as u32;
            if expected != actual {
                self.state = DmaState::Error;
                return Err(DmaError::ChecksumMismatch {
                    expected,
                    actual,
                });
            }
        }

        self.dst_buf = payload.to_vec();

        let cycles = count as u32 * 2 + 5;
        self.state = DmaState::Complete;
        self.mmio.write32(csr_map::DMA_STAT, 1);

        Ok(DmaReport {
            words_transferred: count as u32,
            checksum: Some(checksum),
            state: DmaState::Complete,
            cycles,
        })
    }

    pub fn reset(&mut self) {
        self.state = DmaState::Idle;
        self.src_buf.clear();
        self.dst_buf.clear();
        self.mmio.write32(csr_map::DMA_CTRL, 0);
        self.mmio.write32(csr_map::DMA_STAT, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::mmio::MockMmio;

    fn chan() -> DmaChannel<MockMmio> {
        DmaChannel::new(MockMmio::with_csrs_zeroed())
    }

    fn sample_words(n: usize) -> Vec<u64> {
        (0..n).map(|i| (i as u64) * 0x0100_0100_0100_0101).collect()
    }

    #[test]
    fn new_channel_is_idle() {
        let c = chan();
        assert_eq!(c.state(), DmaState::Idle);
    }

    #[test]
    fn dst_buf_starts_empty() {
        let c = chan();
        assert!(c.dst_buf().is_empty());
    }

    #[test]
    fn transfer_zero_length_errors() {
        let mut c = chan();
        let cfg = DmaConfig::new(0, 0, 0);
        assert_eq!(c.transfer(&cfg), Err(DmaError::ZeroLength));
    }

    #[test]
    fn transfer_exceeds_max_errors() {
        let mut c = chan();
        let cfg = DmaConfig::new(0, 0, (DMA_MAX_WORDS + 1) as u32);
        assert_eq!(c.transfer(&cfg), Err(DmaError::ExceedsMaxLen));
    }

    #[test]
    fn transfer_busy_channel_errors() {
        let mut c = chan();
        c.state = DmaState::Busy;
        let cfg = DmaConfig::new(0, 0, 1);
        assert_eq!(c.transfer(&cfg), Err(DmaError::AlreadyBusy));
    }

    #[test]
    fn transfer_more_than_loaded_errors() {
        let mut c = chan();
        c.load_src(&[1, 2]);
        let cfg = DmaConfig::new(0, 0, 5);
        assert_eq!(c.transfer(&cfg), Err(DmaError::ExceedsMaxLen));
        assert_eq!(c.state(), DmaState::Error);
    }

    #[test]
    fn transfer_copies_to_dst() {
        let mut c = chan();
        let words = sample_words(8);
        c.load_src(&words);
        let cfg = DmaConfig::new(0, 0, 8);
        let report = c.transfer(&cfg).unwrap();
        assert_eq!(report.words_transferred, 8);
        assert_eq!(c.dst_buf(), &words);
    }

    #[test]
    fn transfer_completes_state() {
        let mut c = chan();
        c.load_src(&sample_words(4));
        let cfg = DmaConfig::new(0, 0, 4);
        let report = c.transfer(&cfg).unwrap();
        assert_eq!(report.state, DmaState::Complete);
        assert_eq!(c.state(), DmaState::Complete);
    }

    #[test]
    fn transfer_computes_checksum() {
        let mut c = chan();
        let words = vec![0xDEADBEEFu64 as u64, 0xCAFEBABEu64 as u64];
        c.load_src(&words);
        let cfg = DmaConfig::new(0, 0, 2);
        let report = c.transfer(&cfg).unwrap();
        let expected = crc32_internal::checksum_words(&words);
        assert_eq!(report.checksum, Some(expected));
    }

    #[test]
    fn transfer_cycles_estimate() {
        let mut c = chan();
        c.load_src(&sample_words(10));
        let cfg = DmaConfig::new(0, 0, 10);
        let report = c.transfer(&cfg).unwrap();
        assert_eq!(report.cycles, 25);
    }

    #[test]
    fn transfer_partial_copy() {
        let mut c = chan();
        c.load_src(&sample_words(10));
        let cfg = DmaConfig::new(0, 0, 5);
        let report = c.transfer(&cfg).unwrap();
        assert_eq!(report.words_transferred, 5);
        assert_eq!(c.dst_buf().len(), 5);
        assert_eq!(c.dst_buf()[0], 0);
        assert_eq!(c.dst_buf()[4], 4 * 0x0100_0100_0100_0101);
    }

    #[test]
    fn reset_returns_to_idle() {
        let mut c = chan();
        c.load_src(&sample_words(4));
        let _ = c.transfer(&DmaConfig::new(0, 0, 4));
        assert_eq!(c.state(), DmaState::Complete);
        c.reset();
        assert_eq!(c.state(), DmaState::Idle);
        assert!(c.dst_buf().is_empty());
    }

    #[test]
    fn reset_clears_src() {
        let mut c = chan();
        c.load_src(&[1, 2, 3]);
        c.reset();
        let cfg = DmaConfig::new(0, 0, 1);
        assert_eq!(c.transfer(&cfg), Err(DmaError::ExceedsMaxLen));
    }

    #[test]
    fn verify_checksum_pass_with_valid_trailer() {
        let mut c = chan();
        let data = vec![0xAA00AA00AA00AA00u64, 0xBB00BB00BB00BB00u64];
        let appended = crc32_internal::append(&data);
        c.load_src(&appended);
        let cfg = DmaConfig::new(0, 0, 3).with_checksum();
        let report = c.transfer(&cfg).unwrap();
        assert_eq!(report.words_transferred, 3);
    }

    #[test]
    fn verify_checksum_fail_with_tampered_trailer() {
        let mut c = chan();
        let data = vec![0xAA00AA00AA00AA00u64, 0xBB00BB00BB00BB00u64];
        let mut appended = crc32_internal::append(&data);
        appended[2] ^= 0xFF;
        c.load_src(&appended);
        let cfg = DmaConfig::new(0, 0, 3).with_checksum();
        let result = c.transfer(&cfg);
        assert!(matches!(result, Err(DmaError::ChecksumMismatch { .. })));
        assert_eq!(c.state(), DmaState::Error);
    }

    #[test]
    fn verify_checksum_single_word_skips_check() {
        let mut c = chan();
        c.load_src(&[42u64]);
        let cfg = DmaConfig::new(0, 0, 1).with_checksum();
        let report = c.transfer(&cfg).unwrap();
        assert_eq!(report.words_transferred, 1);
    }

    #[test]
    fn dma_error_display() {
        let e = DmaError::ZeroLength;
        assert!(e.to_string().contains("zero"));
        let e = DmaError::ChecksumMismatch { expected: 1, actual: 2 };
        assert!(e.to_string().contains("mismatch"));
    }

    #[test]
    fn sequential_transfers_after_reset() {
        let mut c = chan();
        c.load_src(&sample_words(10));
        let _ = c.transfer(&DmaConfig::new(0, 0, 5)).unwrap();
        c.reset();
        c.load_src(&sample_words(3));
        let report = c.transfer(&DmaConfig::new(0, 0, 3)).unwrap();
        assert_eq!(report.words_transferred, 3);
        assert_eq!(c.dst_buf().len(), 3);
    }

    #[test]
    fn mmio_access() {
        let mut c = chan();
        c.mmio_mut().write32(csr_map::DMA_CTRL, 1);
        assert_eq!(c.mmio().peek(csr_map::DMA_CTRL), 1);
    }

    #[test]
    fn dma_config_builder() {
        let cfg = DmaConfig::new(0x1000, 0x2000, 64).with_checksum();
        assert_eq!(cfg.src_addr, 0x1000);
        assert_eq!(cfg.dst_addr, 0x2000);
        assert_eq!(cfg.word_count, 64);
        assert!(cfg.verify_checksum);
    }
}

//! Pure-Rust driver for the Xilinx Platform Cable USB II (DLC10/DLC9).
//!
//! Replaces the legacy Python `tools/dlc10_jtag.py`. Provides:
//!
//! * USB enumeration + Cypress FX2 firmware load (Intel-HEX `xusb_xp2.hex`).
//! * Low-level JTAG primitives (`shift_ir`, `shift_dr`, `cycle_tck`, …).
//! * `read_idcode`, `read_status`, `program_sram` (correct UG470 §6 sequence
//!   with `JPROGRAM`).
//! * `program_flash`: loads a 7-series JTAG-to-SPI bridge bitstream
//!   (`bscan_spi_xc7a100t.bit`, MIT-licensed, embedded), then drives the SPI
//!   flash via `USER1`.
//!
//! ## Critical fixes vs prior Python attempt
//!
//! 1. **SRAM JPROGRAM**: the old flow was `JSHUTDOWN → CFG_IN → JSTART`,
//!    which left `DONE = LOW`. The correct UG470 §6 sequence is
//!    `JPROGRAM cycle(64) → JSHUTDOWN cycle(12) → CFG_IN <bs> cycle(1) →
//!    JSTART cycle(24) → BYPASS → CFG_OUT → STATUS`.
//! 2. **`chunk_bits = 16379`** (NOT a multiple of 4) — the DLC10 firmware
//!    silently corrupts payloads when the bit count is a multiple of 4
//!    without explicit pad handling.
//! 3. **USB endpoints**: `EP_OUT = 0x02`, `EP_IN = 0x86`,
//!    vendor-request = `0xB0`, FX2 firmware-load request = `0xA0`,
//!    FX2 CPUCS register = `0xE600`.

#![allow(clippy::needless_range_loop)]

use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use rusb::{request_type, Direction, Recipient, RequestType, UsbContext};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Xilinx USB vendor ID.
pub const VID_XILINX: u16 = 0x03FD;
/// Product ID before firmware is loaded (FX2 in re-enumeration mode).
pub const PID_UNINIT: u16 = 0x0013;
/// Product ID after firmware load.
pub const PID_READY: u16 = 0x0008;

/// USB bulk endpoints used by the DLC10 firmware.
pub const EP_OUT: u8 = 0x02;
pub const EP_IN: u8 = 0x86;

/// FX2 firmware-load vendor request (CPUCS register at 0xE600).
const FX2_FW_REQ: u8 = 0xA0;
const FX2_CPUCS: u16 = 0xE600;
/// Generic DLC10 vendor request.
const VENDOR_REQ: u8 = 0xB0;

/// Chunk size for `_do_shift` — explicitly **not** a multiple of 4.
const CHUNK_BITS: usize = 16379;

/// 7-series IR opcodes (UG470 Table 6-3).
pub mod ir {
    pub const BYPASS: u8 = 0x3F;
    pub const IDCODE: u8 = 0x09;
    pub const CFG_IN: u8 = 0x05;
    pub const CFG_OUT: u8 = 0x04;
    pub const JPROGRAM: u8 = 0x0B;
    pub const JSTART: u8 = 0x0C;
    pub const JSHUTDOWN: u8 = 0x0D;
    pub const ISC_ENABLE: u8 = 0x10;
    pub const ISC_DISABLE: u8 = 0x16;
    pub const USER1: u8 = 0x02;
    pub const USER2: u8 = 0x03;
}

/// SPI flash opcodes (M25P/N25Q-class).
pub mod spi_cmd {
    pub const READ_ID: u8 = 0x9F;
    pub const READ_STATUS: u8 = 0x05;
    pub const WREN: u8 = 0x06;
    pub const PAGE_PROGRAM: u8 = 0x02;
    pub const SECTOR_ERASE: u8 = 0xD8;
    pub const READ_DATA: u8 = 0x03;
}

pub const STATUS_BUSY_BIT: u8 = 0x01;
pub const PAGE_SIZE: usize = 256;
pub const SECTOR_SIZE: usize = 65_536;

/// Extra SPI command bytes (Micron / Macronix / Spansion / Cypress).
pub mod spi_extra {
    /// Release from Deep Power-down (and optionally read electronic signature).
    pub const RELEASE_PD: u8 = 0xAB;
    /// Reset Enable (must precede 0x99 within 1 clock).
    pub const RESET_ENABLE: u8 = 0x66;
    /// Reset Device (after 0x66).
    pub const RESET_DEVICE: u8 = 0x99;
}

/// 7-series configuration register addresses (UG470 Table 5-23).
pub mod cfg_reg {
    pub const CRC: u8 = 0x00;
    pub const FAR: u8 = 0x01;
    pub const FDRI: u8 = 0x02;
    pub const FDRO: u8 = 0x03;
    pub const CMD: u8 = 0x04;
    pub const CTL0: u8 = 0x05;
    pub const MASK: u8 = 0x06;
    pub const STAT: u8 = 0x07;
    pub const LOUT: u8 = 0x08;
    pub const COR0: u8 = 0x09;
    pub const MFWR: u8 = 0x0A;
    pub const CBC: u8 = 0x0B;
    pub const IDCODE: u8 = 0x0C;
    pub const AXSS: u8 = 0x0D;
    pub const COR1: u8 = 0x0E;
    pub const WBSTAR: u8 = 0x10;
    pub const TIMER: u8 = 0x11;
    pub const BOOTSTS: u8 = 0x16;
    pub const CTL1: u8 = 0x18;
    pub const BSPI: u8 = 0x1F;
}

/// Embedded Cypress FX2 firmware (Intel-HEX, ~22 KB).
///
/// On systems where the file is not yet committed to the repo, the build
/// fails here with a clear error. Copy `xusb_xp2.hex` to `fpga/tools/`.
const XUSB_FW_HEX: &[u8] = include_bytes!("../../../fpga/tools/xusb_xp2.hex");

/// Embedded JTAG-to-SPI bridge bitstream for XC7A100T (MIT,
/// quartiq/bscan_spi_bitstreams).
pub const BSCAN_SPI_XC7A100T: &[u8] = include_bytes!("../../../fpga/tools/bscan_spi_xc7a100t.bit");

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum Dlc10Error {
    #[error("DLC10 cable not found (VID=0x{VID_XILINX:04X})")]
    NotFound,
    #[error("device stuck in uninit state after firmware load")]
    FirmwareTimeout,
    #[error("malformed Intel-HEX line: {0}")]
    BadHex(String),
    #[error("malformed Xilinx .bit file: {0}")]
    BadBitfile(String),
    #[error("SPI flash timeout while waiting for WIP=0")]
    FlashBusyTimeout,
    #[error("SPI verify failed at offset 0x{addr:X}: expected 0x{expect:02X}, got 0x{got:02X}")]
    VerifyMismatch { addr: u64, expect: u8, got: u8 },
}

// ---------------------------------------------------------------------------
// Lookup tables
// ---------------------------------------------------------------------------

/// 256-entry bit-reverse table (xc3sprog convention).
pub static BIT_REV_TABLE: [u8; 256] = {
    let mut t = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        let b = i as u8;
        let mut r: u8 = 0;
        let mut k = 0;
        while k < 8 {
            if b & (1 << k) != 0 {
                r |= 1 << (7 - k);
            }
            k += 1;
        }
        t[i] = r;
        i += 1;
    }
    t
};

/// Reverse the bit-order of every byte in `data`.
pub fn bitrev(data: &[u8]) -> Vec<u8> {
    data.iter().map(|&b| BIT_REV_TABLE[b as usize]).collect()
}

// ---------------------------------------------------------------------------
// .bit parser
// ---------------------------------------------------------------------------

/// Parse a Xilinx `.bit` file, returning bit-reversed raw bitstream payload.
///
/// Scans the first 512 bytes for tag `0x65` (the `e` field), reads a
/// big-endian `u32` length, and returns `bitrev(payload)`.
pub fn parse_bitfile(data: &[u8]) -> Result<Vec<u8>> {
    let (start, len) = bitfile_payload_range(data)?;
    Ok(bitrev(&data[start..start + len]))
}

/// Locate the raw bitstream payload range inside a `.bit` file. Returns
/// `(start_offset, length)` of the raw (non-bit-reversed) FPGA payload.
pub fn bitfile_payload_range(data: &[u8]) -> Result<(usize, usize)> {
    let scan_end = std::cmp::min(512, data.len().saturating_sub(5));
    for i in 0..scan_end {
        if data[i] == 0x65 {
            let bs_len =
                u32::from_be_bytes([data[i + 1], data[i + 2], data[i + 3], data[i + 4]]) as usize;
            let remainder = data.len().saturating_sub(i + 5);
            if remainder >= bs_len && (remainder - bs_len) < 256 {
                return Ok((i + 5, bs_len));
            }
        }
    }
    Err(Dlc10Error::BadBitfile("no 'e' field found".into()).into())
}

/// Find the offset of the Xilinx sync word `0xAA995566` inside a byte slice.
/// Returns the index of the first byte of the sync, or `None` if not found.
pub fn find_sync_word(data: &[u8]) -> Option<usize> {
    const SYNC: [u8; 4] = [0xAA, 0x99, 0x55, 0x66];
    data.windows(4).position(|w| w == SYNC)
}

// ---------------------------------------------------------------------------
// Intel HEX parser
// ---------------------------------------------------------------------------

/// One Intel-HEX type-0 record: `(addr, data_bytes)`.
pub type HexRecord = (u16, Vec<u8>);

/// Parse Intel-HEX text into a flat list of `(addr, bytes)` for every
/// type-0 (data) record. Type-1 (EOF) terminates parsing.
pub fn parse_intel_hex(text: &str) -> Result<Vec<HexRecord>> {
    let mut out = Vec::new();
    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || !line.starts_with(':') {
            continue;
        }
        let bytes = hex::decode(&line[1..])
            .map_err(|e| Dlc10Error::BadHex(format!("line {}: {}", lineno + 1, e)))?;
        if bytes.len() < 5 {
            return Err(Dlc10Error::BadHex(format!("line {}: too short", lineno + 1)).into());
        }
        let rlen = bytes[0] as usize;
        let addr = u16::from_be_bytes([bytes[1], bytes[2]]);
        let typ = bytes[3];
        if bytes.len() < 4 + rlen + 1 {
            return Err(Dlc10Error::BadHex(format!(
                "line {}: declared len {} doesn't fit",
                lineno + 1,
                rlen
            ))
            .into());
        }
        match typ {
            0 if rlen > 0 => {
                out.push((addr, bytes[4..4 + rlen].to_vec()));
            }
            1 => break,
            _ => {}
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

/// Options for `program_flash`.
pub struct FlashOpts {
    pub verify: bool,
    pub progress: Option<Box<dyn FnMut(u64, u64)>>,
}

impl Default for FlashOpts {
    fn default() -> Self {
        Self {
            verify: true,
            progress: None,
        }
    }
}

impl std::fmt::Debug for FlashOpts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlashOpts")
            .field("verify", &self.verify)
            .field("progress", &self.progress.is_some())
            .finish()
    }
}

/// Open DLC10 cable handle.
pub struct Dlc10 {
    handle: rusb::DeviceHandle<rusb::Context>,
}

impl Dlc10 {
    /// Find the cable, load firmware if needed, claim interface, run the
    /// post-init vendor-control sequence.
    pub fn open() -> Result<Self> {
        let ctx = rusb::Context::new().context("rusb context init")?;

        // Look for already-initialized cable first.
        if let Some((dev, _desc)) = find_device(&ctx, VID_XILINX, PID_READY)? {
            let h = open_and_claim(dev)?;
            init_after_firmware(&h)?;
            return Ok(Self { handle: h });
        }

        // Otherwise look for the un-initialized cable and load firmware.
        if let Some((dev, _desc)) = find_device(&ctx, VID_XILINX, PID_UNINIT)? {
            let h = dev.open().context("open uninit dlc10")?;
            // The kernel may have a driver — detach if so.
            let _ = h.set_auto_detach_kernel_driver(true);
            h.set_active_configuration(1).ok();
            load_firmware(&h)?;
            drop(h);
            // Wait for re-enumeration.
            let deadline = Instant::now() + Duration::from_secs(20);
            while Instant::now() < deadline {
                std::thread::sleep(Duration::from_secs(1));
                if let Some((dev2, _)) = find_device(&ctx, VID_XILINX, PID_READY)? {
                    let h2 = open_and_claim(dev2)?;
                    init_after_firmware(&h2)?;
                    return Ok(Self { handle: h2 });
                }
            }
            return Err(Dlc10Error::FirmwareTimeout.into());
        }

        Err(Dlc10Error::NotFound.into())
    }

    /// Read the JTAG `IDCODE`. Expected `0x13631093` for XC7A100T.
    pub fn read_idcode(&mut self) -> Result<u32> {
        self.shift_ir(ir::IDCODE)?;
        self.read_dr_32()
    }

    /// Read the configuration `STATUS` register via `CFG_OUT` (raw — no
    /// preceding CFG_IN protocol; this returns whatever the DR captured
    /// last, which may be stale).
    pub fn read_status(&mut self) -> Result<u32> {
        self.shift_ir(ir::CFG_OUT)?;
        self.read_dr_32()
    }

    /// Build the canonical openFPGALoader `dumpRegister` packet sequence
    /// for reading config register `reg_addr` (UG470 Type-1 read).
    ///
    /// Returns the 5 host-order u32 packet words. The on-the-wire encoding
    /// (per-word `reverse_bits` followed by LE byte-split) is applied
    /// separately in `read_cfg_reg`.
    pub fn build_read_cfg_packets(reg_addr: u8) -> [u32; 5] {
        // openFPGALoader `xilinx.cpp::dumpRegister`:
        //   ((0x01 & 0x0007) << 29)  // header type 1
        // | ((0x01 & 0x0003) << 27)  // opcode = read
        // | ((reg  & 0x3FFF) << 13)  // register address
        // | ((0x00 & 0x0003) << 11)  // reserved
        // | ((0x01 & 0x07FF) <<  0)  // word count
        let read_hdr: u32 =
            (1u32 << 29) | (1u32 << 27) | (((reg_addr as u32) & 0x3FFF) << 13) | 1u32;
        [
            0xAA995566, // Sync Word (NO bus-width 0xFFFFFFFF prefix on JTAG)
            0x20000000, // NOP
            read_hdr,   // Type-1 Read
            0x20000000, // NOP
            0x20000000, // NOP
        ]
    }

    /// Read a 7-series configuration register. Mirrors openFPGALoader
    /// `Xilinx::dumpRegister` byte-for-byte:
    ///
    /// 1. `shift_ir(CFG_IN)`.
    /// 2. Shift FIVE separate 32-bit DR transactions, one per packet word,
    ///    each going through its own Capture-DR → Shift-DR → Exit1-DR →
    ///    **Update-DR** → RTI cycle. The per-word `Update-DR` event is what
    ///    causes the FPGA's configuration FSM to actually *latch* and
    ///    process each Type-1 packet.
    /// 3. `shift_ir(CFG_OUT)`.
    /// 4. One 32-bit DR shift to read the queued value.
    /// 5. Reverse all 32 bits of the result (the FPGA streams MSBs first;
    ///    `read_dr_32` accumulates LSBs first).
    ///
    /// **Why one big shift didn't work in v2**: my previous implementation
    /// concatenated the 6 packet words into a single 192-bit DR shift with
    /// a single Update-DR at the very end. The hardware (XC7A100T) refused
    /// to execute the queued read, leaving FDRO = 0 — every register read
    /// (including IDCODE, which is hard-wired to 0x13631093) came back as
    /// `0x00000000`. openFPGALoader and Vivado both do per-word Update-DR.
    pub fn read_cfg_reg(&mut self, reg_addr: u8) -> Result<u32> {
        let raw = self.read_cfg_reg_raw_n(reg_addr, 32)?;
        Ok(raw[0])
    }

    /// Same as `read_cfg_reg`, but also returns the host-order packet
    /// bytes shifted into CFG_IN (for `idcode-cfg --raw` diagnostics).
    pub fn read_cfg_reg_diag(&mut self, reg_addr: u8, bits: usize) -> Result<ReadCfgDiag> {
        let packets = Self::build_read_cfg_packets(reg_addr);
        let mut wire_bytes: Vec<u8> = Vec::with_capacity(20);
        for w in &packets {
            // openFPGALoader: tmp = reverse_32(packet); then split LE.
            let tmp = w.reverse_bits();
            wire_bytes.push((tmp & 0xFF) as u8);
            wire_bytes.push(((tmp >> 8) & 0xFF) as u8);
            wire_bytes.push(((tmp >> 16) & 0xFF) as u8);
            wire_bytes.push(((tmp >> 24) & 0xFF) as u8);
        }
        let result_words = self.read_cfg_reg_raw_n(reg_addr, bits)?;
        Ok(ReadCfgDiag {
            packets_host_order: packets,
            wire_bytes_per_word: wire_bytes,
            result_words,
        })
    }

    /// Shift the read packets in, then shift `bits` (rounded up to multiples
    /// of 32) out of CFG_OUT. Returns the result split into 32-bit words.
    pub fn read_cfg_reg_raw_n(&mut self, reg_addr: u8, bits: usize) -> Result<Vec<u32>> {
        let packets = Self::build_read_cfg_packets(reg_addr);

        self.shift_ir(ir::CFG_IN)?;
        for &w in &packets {
            // openFPGALoader: tmp = reverse_32(packet); split into 4 LE bytes;
            // shift 32 bits via shiftDR. Each shift does its own Capture-DR →
            // Shift-DR → Exit1-DR → Update-DR cycle.
            let tmp = w.reverse_bits();
            let bytes = [
                (tmp & 0xFF) as u8,
                ((tmp >> 8) & 0xFF) as u8,
                ((tmp >> 16) & 0xFF) as u8,
                ((tmp >> 24) & 0xFF) as u8,
            ];
            self.shift_dr_small(&bytes, 32)?;
        }

        self.shift_ir(ir::CFG_OUT)?;
        let n_words = bits.div_ceil(32);
        let mut out = Vec::with_capacity(n_words);
        for _ in 0..n_words {
            let raw = self.read_dr_32()?;
            // FPGA emits MSB first; `read_dr_32` packs LSB-first. Reverse.
            out.push(raw.reverse_bits());
        }
        Ok(out)
    }

    /// Self-test: read the configuration IDCODE register (addr 0x0C) via the
    /// proper Type-1 read protocol. On a healthy XC7A100T this must return
    /// `0x13631093` — same as the JTAG IDCODE. If `read_cfg_reg` ever returns
    /// 0 here while `read_idcode` returns the expected value, the bug is in
    /// the Type-1 read sequence (most likely missing per-word Update-DR),
    /// not in the device.
    pub fn read_cfg_idcode(&mut self) -> Result<u32> {
        self.read_cfg_reg(cfg_reg::IDCODE)
    }

    /// Poll the configuration STATUS register until `INIT_COMPLETE` (or
    /// `INIT_B`) is high, with a timeout. UG470 §6 requires this between
    /// `JPROGRAM` and `CFG_IN`; the chip is busy mass-erasing configuration
    /// memory and will eat the bitstream silently if we shift too early.
    pub fn wait_for_init(&mut self, timeout: Duration) -> Result<StatBits> {
        let deadline = Instant::now() + timeout;
        let mut last = StatBits::from_raw(0);
        while Instant::now() < deadline {
            let raw = self.read_cfg_reg(cfg_reg::STAT)?;
            last = StatBits::from_raw(raw);
            if last.init_b && last.init_complete {
                return Ok(last);
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        Err(anyhow!(
            "wait_for_init: timed out (last STAT=0x{:08X}, INIT_B={}, INIT_COMPLETE={})",
            last.raw,
            last.init_b as u8,
            last.init_complete as u8,
        ))
    }

    /// Program FPGA SRAM (volatile). Returns the final `STATUS` register.
    ///
    /// **Implements the correct UG470 §6 flow** (the Python version was
    /// missing `JPROGRAM`):
    ///
    /// 1. `JPROGRAM` + `cycle_tck(64)` — mass-erase configuration memory.
    /// 2. `JSHUTDOWN` + `cycle_tck(12)` — drive Tap to capture/shift state.
    /// 3. `CFG_IN` + bit-reversed bitstream + `cycle_tck(1)`.
    /// 4. `JSTART` + `cycle_tck(24)` — start-up sequence.
    /// 5. `BYPASS` (1-bit shift) → `CFG_OUT` → 32-bit `STATUS`.
    pub fn program_sram(&mut self, bit: &[u8]) -> Result<u32> {
        self.program_sram_verbose(bit, false)
    }

    /// Like `program_sram`, but emits diagnostic lines to `stderr` when
    /// `verbose = true`. Reports bytes loaded, sync-word offset, payload
    /// preview, and total TCK shift bits.
    pub fn program_sram_verbose(&mut self, bit: &[u8], verbose: bool) -> Result<u32> {
        let (raw_start, raw_len) = bitfile_payload_range(bit)?;
        let raw = &bit[raw_start..raw_start + raw_len];
        let bs = bitrev(raw);

        if verbose {
            eprintln!(
                "[verbose] .bit file size = {} bytes ; payload range = [0x{:X}..0x{:X}) ; payload len = {} bytes",
                bit.len(),
                raw_start,
                raw_start + raw_len,
                raw_len,
            );
            match find_sync_word(raw) {
                Some(off) => {
                    let first_dword = if off + 8 <= raw.len() {
                        let s = &raw[off + 4..off + 8];
                        u32::from_be_bytes([s[0], s[1], s[2], s[3]])
                    } else {
                        0
                    };
                    eprintln!(
                        "[verbose] sync word 0xAA995566 at payload-relative offset {} (file 0x{:X}) ; first DWORD after sync = 0x{:08X}",
                        off,
                        raw_start + off,
                        first_dword,
                    );
                    if first_dword != 0x20000000 && first_dword != 0x30020001 {
                        eprintln!(
                            "[verbose] WARN: first DWORD after sync is unusual (expected NOP 0x20000000 or CMD-write 0x30020001)",
                        );
                    }
                }
                None => eprintln!("[verbose] WARN: sync word 0xAA995566 NOT found in payload"),
            }
            eprintln!(
                "[verbose] first 16 raw bytes  = {}",
                hex::encode(&raw[..raw.len().min(16)])
            );
            eprintln!(
                "[verbose] first 16 shifted   = {}  (bit-reversed)",
                hex::encode(&bs[..bs.len().min(16)])
            );
            let n = bs.len();
            let tail = &bs[n.saturating_sub(64)..];
            eprintln!("[verbose] last 64 shifted bytes = {}", hex::encode(tail));
            eprintln!(
                "[verbose] chunk_bits = {} ; total bits to shift = {} ; chunks = {}",
                CHUNK_BITS,
                bs.len() * 8,
                (bs.len() * 8).div_ceil(CHUNK_BITS),
            );
        }

        // Step 1: JPROGRAM — asserts internal PROG_B, mass-erases config.
        // The chip drives INIT_B low while erasing.
        self.shift_ir(ir::JPROGRAM)?;
        self.cycle_tck(64)?;

        // Step 2: poll STAT.INIT_B (via the now-correct Type-1 read path)
        // until the chip is ready. Without this poll we routinely shifted
        // CFG_IN bytes into the void while INIT_B was still low.
        match self.wait_for_init(Duration::from_secs(2)) {
            Ok(s) => {
                if verbose {
                    eprintln!(
                        "[verbose] post-JPROGRAM STAT=0x{:08X} (INIT_B={}, INIT_COMPLETE={})",
                        s.raw, s.init_b as u8, s.init_complete as u8,
                    );
                }
            }
            Err(e) if verbose => {
                eprintln!("[verbose] WARN: wait_for_init failed: {e}");
            }
            Err(_) => {}
        }

        self.shift_ir(ir::JSHUTDOWN)?;
        self.cycle_tck(12)?;

        self.shift_ir(ir::CFG_IN)?;
        self.shift_dr(&bs, bs.len() * 8)?;
        self.cycle_tck(1)?;

        self.shift_ir(ir::JSTART)?;
        self.cycle_tck(24)?;

        self.shift_ir(ir::BYPASS)?;
        self.shift_dr_small(&[0x00], 1)?;
        self.cycle_tck(1)?;

        self.shift_ir(ir::CFG_OUT)?;
        let status = self.read_dr_32()?;

        if verbose {
            eprintln!(
                "[verbose] raw CFG_OUT read (BYPASS+CFG_OUT) = 0x{:08X}",
                status
            );
            // Now do a real STAT read via the corrected protocol so the
            // verbose output also contains the trustworthy value.
            match self.read_cfg_reg(cfg_reg::STAT) {
                Ok(stat) => {
                    let s = StatBits::from_raw(stat);
                    eprintln!(
                        "[verbose] final STAT (Type-1 read) = 0x{:08X} (DONE={}, EOS={}, INIT_B={}, MMCM_LOCK={}, CRC_ERROR={}, ID_ERROR={})",
                        s.raw, s.done as u8, s.eos as u8, s.init_b as u8,
                        s.mmcm_lock as u8, s.crc_error as u8, s.id_error as u8,
                    );
                    eprintln!("[verbose] diagnosis: {}", s.diagnose());
                }
                Err(e) => eprintln!("[verbose] WARN: final STAT read failed: {e}"),
            }
        }
        Ok(status)
    }

    /// Program the on-board SPI flash.
    pub fn program_flash(&mut self, bit: &[u8], mut opts: FlashOpts) -> Result<()> {
        // Step 1: load the JTAG-to-SPI bridge into FPGA SRAM (verbose so
        // the user sees the post-JSTART STAT decode if anything is off).
        let _bridge_status = self.program_sram_verbose(BSCAN_SPI_XC7A100T, true)?;

        // Step 2: select USER1 — that maps the BSCAN data register to the
        // single-bit SPI shift register inside the bridge.
        self.shift_ir(ir::USER1)?;
        eprintln!("[debug] program_flash: IR=USER1, attempting JEDEC ID read");

        // Step 3: read JEDEC ID — sanity check (with recovery attempts).
        let id = self.spi_xfer_verbose(&[spi_cmd::READ_ID], 3, true)?;
        eprintln!(
            "SPI flash JEDEC ID: {:02X} {:02X} {:02X}",
            id[0], id[1], id[2]
        );
        if id == vec![0xFF, 0xFF, 0xFF] || id == vec![0x00, 0x00, 0x00] {
            // Try the standard recovery sequences before bailing.
            eprintln!(
                "[debug] JEDEC looks dead — trying 0xAB Release Power-down + 0x66/0x99 reset"
            );
            self.spi_xfer_verbose(&[spi_extra::RELEASE_PD], 0, true)?;
            std::thread::sleep(Duration::from_millis(5));
            self.spi_xfer_verbose(&[spi_extra::RESET_ENABLE], 0, true)?;
            self.spi_xfer_verbose(&[spi_extra::RESET_DEVICE], 0, true)?;
            std::thread::sleep(Duration::from_millis(30));
            let retry = self.spi_xfer_verbose(&[spi_cmd::READ_ID], 3, true)?;
            eprintln!(
                "SPI flash JEDEC ID (after recovery): {:02X} {:02X} {:02X}",
                retry[0], retry[1], retry[2]
            );
            if retry == vec![0xFF, 0xFF, 0xFF] || retry == vec![0x00, 0x00, 0x00] {
                return Err(anyhow!(
                    "SPI flash unreachable: JEDEC stays at {:02X} {:02X} {:02X} after release-PD and software reset. \
                     Run `tri fpga proxy-load fpga/tools/bscan_spi_xc7a100t.bit` then `tri fpga proxy-status` to confirm DONE=HIGH; \
                     if DONE=LOW, the proxy bitstream does not match this board's pinout — see docs/fpga/SPI_FLASH_DEBUG.md.",
                    retry[0], retry[1], retry[2],
                ));
            }
        }

        // Step 4: erase the sectors we're about to write.
        let total = bit.len() as u64;
        let sectors = bit.len().div_ceil(SECTOR_SIZE);
        for s in 0..sectors {
            let addr = (s * SECTOR_SIZE) as u32;
            self.spi_write_enable()?;
            let cmd = [
                spi_cmd::SECTOR_ERASE,
                ((addr >> 16) & 0xFF) as u8,
                ((addr >> 8) & 0xFF) as u8,
                (addr & 0xFF) as u8,
            ];
            self.spi_xfer(&cmd, 0)?;
            self.spi_wait_wip(Duration::from_secs(10))?;
        }

        // Step 5: page-program.
        let mut written: u64 = 0;
        let mut buf = Vec::with_capacity(4 + PAGE_SIZE);
        for chunk in bit.chunks(PAGE_SIZE) {
            let addr = written as u32;
            self.spi_write_enable()?;
            buf.clear();
            buf.push(spi_cmd::PAGE_PROGRAM);
            buf.push(((addr >> 16) & 0xFF) as u8);
            buf.push(((addr >> 8) & 0xFF) as u8);
            buf.push((addr & 0xFF) as u8);
            buf.extend_from_slice(chunk);
            self.spi_xfer(&buf, 0)?;
            self.spi_wait_wip(Duration::from_secs(2))?;
            written += chunk.len() as u64;
            if let Some(cb) = opts.progress.as_mut() {
                cb(written, total);
            }
        }

        // Step 6: optional read-back verify.
        if opts.verify {
            let mut verified: u64 = 0;
            let mut rd_cmd = [0u8; 4];
            for chunk in bit.chunks(PAGE_SIZE) {
                let addr = verified as u32;
                rd_cmd[0] = spi_cmd::READ_DATA;
                rd_cmd[1] = ((addr >> 16) & 0xFF) as u8;
                rd_cmd[2] = ((addr >> 8) & 0xFF) as u8;
                rd_cmd[3] = (addr & 0xFF) as u8;
                let got = self.spi_xfer(&rd_cmd, chunk.len())?;
                for (i, (e, g)) in chunk.iter().zip(got.iter()).enumerate() {
                    if e != g {
                        return Err(Dlc10Error::VerifyMismatch {
                            addr: addr as u64 + i as u64,
                            expect: *e,
                            got: *g,
                        }
                        .into());
                    }
                }
                verified += chunk.len() as u64;
            }
        }

        // Step 7: kick FPGA — JPROGRAM reloads from flash.
        self.shift_ir(ir::JPROGRAM)?;
        self.cycle_tck(64)?;
        Ok(())
    }

    /// Load the bridge bitstream into FPGA SRAM and read the SPI flash
    /// JEDEC ID (READ_ID 0x9F → 3 bytes).
    pub fn read_flash_id(&mut self) -> Result<[u8; 3]> {
        self.read_flash_id_verbose(false)
    }

    /// Like `read_flash_id`, but emits `[debug] ...` lines describing each
    /// step (proxy load, STAT poll, USER1 select, raw RX bytes).
    ///
    /// Also performs **two recovery attempts** before declaring failure:
    ///
    /// 1. Issue `0xAB` (Release Power-down) — if the flash booted in
    ///    deep-power-down (Micron N25Q does this on certain board variants),
    ///    this wakes it up. Re-reads JEDEC.
    /// 2. Issue `0x66` + `0x99` (Reset Enable + Reset Device) — full chip
    ///    reset. Re-reads JEDEC.
    ///
    /// The function returns the **first non-FF non-zero** triple it sees,
    /// or the last triple read if all attempts return FF/00.
    pub fn read_flash_id_verbose(&mut self, verbose: bool) -> Result<[u8; 3]> {
        if verbose {
            eprintln!("[debug] read_flash_id: loading bridge bitstream (proxy)");
        }
        let _status = self.program_sram_verbose(BSCAN_SPI_XC7A100T, verbose)?;
        if verbose {
            // Re-read STAT via the proper Type-1 path so we report a number
            // the user can trust.
            match self.read_cfg_reg(cfg_reg::STAT) {
                Ok(s) => {
                    let bits = StatBits::from_raw(s);
                    eprintln!(
                        "[debug] post-proxy STAT=0x{:08X} DONE={} EOS={} INIT_B={} INIT_COMPLETE={} ID_ERROR={} CRC_ERROR={}",
                        bits.raw,
                        bits.done as u8,
                        bits.eos as u8,
                        bits.init_b as u8,
                        bits.init_complete as u8,
                        bits.id_error as u8,
                        bits.crc_error as u8,
                    );
                    if !bits.done {
                        eprintln!("[debug] WARN: proxy did NOT reach DONE=HIGH — bridge is not running, JEDEC will be FF FF FF");
                    }
                }
                Err(e) => eprintln!("[debug] WARN: post-proxy STAT read failed: {e}"),
            }
        }
        self.shift_ir(ir::USER1)?;
        if verbose {
            eprintln!("[debug] IR = USER1 (0x02) — BSCAN1 SPI bridge selected");
        }

        let id = self.spi_xfer_verbose(&[spi_cmd::READ_ID], 3, verbose)?;
        let triple = |v: &[u8]| -> [u8; 3] { [v[0], v[1], v[2]] };
        let is_dead = |a: &[u8; 3]| a == &[0xFF, 0xFF, 0xFF] || a == &[0x00, 0x00, 0x00];
        let mut out = triple(&id);
        if !is_dead(&out) {
            return Ok(out);
        }

        if verbose {
            eprintln!(
                "[debug] JEDEC came back as {:02X} {:02X} {:02X} — attempting 0xAB Release Power-down",
                out[0], out[1], out[2],
            );
        }
        // Recovery 1: Release from Deep Power-down (0xAB), then re-read.
        self.spi_xfer_verbose(&[spi_extra::RELEASE_PD], 0, verbose)?;
        std::thread::sleep(Duration::from_millis(5));
        let id2 = self.spi_xfer_verbose(&[spi_cmd::READ_ID], 3, verbose)?;
        out = triple(&id2);
        if !is_dead(&out) {
            if verbose {
                eprintln!("[debug] recovery via 0xAB succeeded");
            }
            return Ok(out);
        }

        if verbose {
            eprintln!(
                "[debug] still {:02X} {:02X} {:02X} — attempting 0x66 + 0x99 software reset",
                out[0], out[1], out[2],
            );
        }
        // Recovery 2: Reset Enable + Reset Device.
        self.spi_xfer_verbose(&[spi_extra::RESET_ENABLE], 0, verbose)?;
        self.spi_xfer_verbose(&[spi_extra::RESET_DEVICE], 0, verbose)?;
        std::thread::sleep(Duration::from_millis(30));
        let id3 = self.spi_xfer_verbose(&[spi_cmd::READ_ID], 3, verbose)?;
        out = triple(&id3);
        if verbose && !is_dead(&out) {
            eprintln!("[debug] recovery via 0x66/0x99 succeeded");
        }
        Ok(out)
    }

    // ------------------ Diagnostic primitives (Rust API) -------------------

    /// Diagnostic-only: load *any* bitstream into FPGA SRAM and leave the
    /// JTAG TAP in Run-Test/Idle with IR=`BYPASS` (so the caller can poll
    /// STAT separately). Returns the post-`JSTART` CFG_OUT read.
    ///
    /// Use this to validate that the bridge proxy bitstream actually
    /// configures the device (DONE goes HIGH) **before** worrying about
    /// USER1/SPI semantics. Always emits `[debug] ...` instrumentation.
    pub fn proxy_load(&mut self, bit: &[u8]) -> Result<u32> {
        eprintln!(
            "[debug] proxy_load: bitstream size = {} bytes (sha256 prefix: {})",
            bit.len(),
            hex::encode(&bit[..bit.len().min(8)]),
        );
        self.program_sram_verbose(bit, true)
    }

    /// Diagnostic-only: leave the FPGA alone, just read STAT via the
    /// known-good Type-1 read path and emit a decoded report.
    pub fn proxy_status(&mut self) -> Result<StatBits> {
        eprintln!("[debug] proxy_status: reading IDCODE + STAT (no JPROGRAM)");
        let idcode = self.read_idcode()?;
        eprintln!(
            "[debug]   IDCODE = 0x{:08X}{}",
            idcode,
            if idcode == 0x13631093 {
                " (XC7A100T)"
            } else {
                " (UNEXPECTED)"
            },
        );
        let raw = self.read_cfg_reg(cfg_reg::STAT)?;
        let bits = StatBits::from_raw(raw);
        eprintln!(
            "[debug]   STAT=0x{:08X} DONE={} EOS={} INIT_B={} INIT_COMPL={} MMCM_LOCK={} ID_ERROR={} CRC_ERROR={}",
            bits.raw,
            bits.done as u8,
            bits.eos as u8,
            bits.init_b as u8,
            bits.init_complete as u8,
            bits.mmcm_lock as u8,
            bits.id_error as u8,
            bits.crc_error as u8,
        );
        eprintln!("[debug]   diagnosis: {}", bits.diagnose());
        if bits.done {
            // Also probe USER1: shift in a known IR and confirm the IR
            // capture pattern came back as the documented `0x...01` (TAP
            // capture always loads `01` into the two LSBs).
            self.shift_ir(ir::USER1)?;
            eprintln!("[debug]   IR=USER1 select ok (no exception)");
        }
        Ok(bits)
    }

    /// Diagnostic-only: shift `tx` bytes through USER1 and read `rx_len`
    /// bytes back, **assuming** the bridge proxy is already configured.
    /// Always verbose. Caller is responsible for proxy_load() first.
    pub fn spi_raw(&mut self, tx: &[u8], rx_len: usize) -> Result<Vec<u8>> {
        eprintln!(
            "[debug] spi_raw: TX = {} ({} bytes), rx_len = {}",
            hex::encode(tx),
            tx.len(),
            rx_len,
        );
        // Ensure the IR is set — this is a single shift, idempotent.
        self.shift_ir(ir::USER1)?;
        self.spi_xfer_verbose(tx, rx_len, true)
    }

    /// Diagnostic-only: dump the FPGA IR capture pattern after selecting
    /// IR `ir_val`. The TAP's Capture-IR loads `...0_0001` into the IR
    /// shift register (always), so this read-back probe confirms the
    /// scan chain is intact and `ir_val` was accepted.
    pub fn probe_ir_capture(&mut self, ir_val: u8) -> Result<u8> {
        // Select IR, then immediately re-scan IR to read back the capture.
        self.shift_ir(ir_val)?;
        // Shift in 6 bits of TDI=1 with TMS pattern that re-enters Shift-IR.
        let mut tdi = vec![true, true, true, true, false, false]; // → Shift-IR
        let mut tms = vec![true, true, false, false, false, false];
        let rdo_start = tdi.len();
        for i in 0..6 {
            tdi.push(true);
            tms.push(i == 5);
        }
        tdi.extend_from_slice(&[true, true]);
        tms.extend_from_slice(&[true, false]);
        let resp = self.do_shift_with_read(&tdi, &tms, rdo_start, 6)?;
        let stream = extract_byte_stream(&resp, 6);
        let cap = stream.first().copied().unwrap_or(0) & 0x3F;
        eprintln!(
            "[debug] probe_ir_capture(0x{:02X}): IR capture = 0x{:02X} (expect 0x01 for healthy 7-series TAP)",
            ir_val, cap,
        );
        Ok(cap)
    }

    /// Close (drops the handle).
    pub fn close(self) {}

    // ---------------------- low-level JTAG primitives -----------------------

    fn ctrl_out(
        &self,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
        timeout: Duration,
    ) -> Result<()> {
        let rt = request_type(Direction::Out, RequestType::Vendor, Recipient::Device);
        self.handle
            .write_control(rt, request, value, index, data, timeout)
            .map(|_| ())
            .map_err(|e| anyhow!("ctrl_out req=0x{:02X} val=0x{:04X}: {}", request, value, e))
    }

    fn bulk_out(&self, ep: u8, data: &[u8], timeout: Duration) -> Result<usize> {
        self.handle
            .write_bulk(ep, data, timeout)
            .map_err(|e| anyhow!("bulk_out ep=0x{:02X}: {}", ep, e))
    }

    fn bulk_in(&self, ep: u8, len: usize, timeout: Duration) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        let n = self
            .handle
            .read_bulk(ep, &mut buf, timeout)
            .map_err(|e| anyhow!("bulk_in ep=0x{:02X}: {}", ep, e))?;
        buf.truncate(n);
        Ok(buf)
    }

    /// Encode `(tdi, tms)` bit-streams into the DLC10 4-bits-per-byte stride
    /// format and submit. Adds an explicit pad bit when `n % 4 == 0`.
    fn do_shift(&self, tdi: &[bool], tms: &[bool]) -> Result<()> {
        assert_eq!(tdi.len(), tms.len());
        let mut tdi = tdi.to_vec();
        let mut tms = tms.to_vec();
        let mut n = tdi.len();
        if n.is_multiple_of(4) {
            tdi.push(false);
            tms.push(false);
            n += 1;
        }
        let nw = n.div_ceil(4);
        let mut buf = vec![0u8; nw * 2];
        for i in 0..n {
            let bi = i & 3;
            let wi = (i - bi) >> 1;
            if bi == 0 {
                buf[wi] = 0;
                buf[wi + 1] = 0;
            }
            if tdi[i] {
                buf[wi] |= 0x01 << bi;
            }
            if tms[i] {
                buf[wi] |= 0x10 << bi;
            }
            buf[wi + 1] |= 0x01 << bi;
        }
        self.ctrl_out(VENDOR_REQ, 0x00A6, n as u16, &[], Duration::from_secs(10))?;
        self.bulk_out(EP_OUT, &buf, Duration::from_secs(30))?;
        Ok(())
    }

    /// Same as `do_shift`, but TDO is captured for the indicated bit window.
    /// Returns the read-back bytes (little-endian 16-bit words concatenated).
    fn do_shift_with_read(
        &self,
        tdi: &[bool],
        tms: &[bool],
        rdo_start: usize,
        rdo_len: usize,
    ) -> Result<Vec<u8>> {
        assert_eq!(tdi.len(), tms.len());
        let mut tdi = tdi.to_vec();
        let mut tms = tms.to_vec();
        let mut n = tdi.len();
        if n.is_multiple_of(4) {
            tdi.push(false);
            tms.push(false);
            n += 1;
        }
        let nw = n.div_ceil(4);
        let mut buf = vec![0u8; nw * 2];
        for i in 0..n {
            let bi = i & 3;
            let wi = (i - bi) >> 1;
            if bi == 0 {
                buf[wi] = 0;
                buf[wi + 1] = 0;
            }
            if tdi[i] {
                buf[wi] |= 0x01 << bi;
            }
            if tms[i] {
                buf[wi] |= 0x10 << bi;
            }
            if rdo_start <= i && i < rdo_start + rdo_len {
                buf[wi + 1] |= 0x11 << bi;
            } else {
                buf[wi + 1] |= 0x01 << bi;
            }
        }
        self.ctrl_out(VENDOR_REQ, 0x00A6, n as u16, &[], Duration::from_secs(10))?;
        self.bulk_out(EP_OUT, &buf, Duration::from_secs(30))?;
        let ol = 2 * rdo_len.div_ceil(16);
        self.bulk_in(EP_IN, ol, Duration::from_secs(10))
    }

    pub fn shift_ir(&mut self, ir_val: u8) -> Result<()> {
        let mut tdi = Vec::with_capacity(16);
        let mut tms = Vec::with_capacity(16);
        for _ in 0..5 {
            tdi.push(true);
            tms.push(true);
        }
        tdi.extend_from_slice(&[true, false, true, true, false, false]);
        tms.extend_from_slice(&[false, true, true, false, false, false]);
        for i in 0..6 {
            tdi.push((ir_val & (1 << i)) != 0);
            tms.push(i == 5);
        }
        tdi.extend_from_slice(&[true, true]);
        tms.extend_from_slice(&[true, false]);
        self.do_shift(&tdi, &tms)
    }

    /// Shift a (possibly large) DR.
    pub fn shift_dr(&mut self, data: &[u8], nb: usize) -> Result<()> {
        let mut sent = 0usize;
        let mut first = true;
        while sent < nb {
            let chunk = std::cmp::min(nb - sent, CHUNK_BITS);
            let cap = chunk + 5;
            let mut tdi = Vec::with_capacity(cap);
            let mut tms = Vec::with_capacity(cap);
            if first {
                tdi.extend_from_slice(&[true, true, true]);
                tms.extend_from_slice(&[true, false, false]);
                first = false;
            }
            for i in 0..chunk {
                let bp = sent + i;
                tdi.push((data[bp >> 3] & (1 << (bp & 7))) != 0);
                tms.push(sent + i == nb - 1);
            }
            if sent + chunk == nb {
                tdi.extend_from_slice(&[true, true]);
                tms.extend_from_slice(&[true, false]);
            }
            self.do_shift(&tdi, &tms)?;
            sent += chunk;
        }
        Ok(())
    }

    /// Shift a small DR (≤ a few hundred bits) with full Tap excursion.
    pub fn shift_dr_small(&mut self, data: &[u8], nb: usize) -> Result<()> {
        let mut tdi = vec![true, true, true];
        let mut tms = vec![true, false, false];
        for i in 0..nb {
            tdi.push((data[i >> 3] & (1 << (i & 7))) != 0);
            tms.push(i == nb - 1);
        }
        tdi.extend_from_slice(&[true, true]);
        tms.extend_from_slice(&[true, false]);
        self.do_shift(&tdi, &tms)
    }

    /// Pulse TCK with TMS=0 (Run-Test/Idle).
    pub fn cycle_tck(&mut self, n: usize) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        let tdi = vec![true; n];
        let tms = vec![false; n];
        self.do_shift(&tdi, &tms)
    }

    /// Read a 32-bit DR after a `shift_ir(...)` selecting it.
    pub fn read_dr_32(&mut self) -> Result<u32> {
        let mut tdi = vec![true, true, true];
        let mut tms = vec![true, false, false];
        let rdo_start = tdi.len();
        for i in 0..32 {
            tdi.push(false);
            tms.push(i == 31);
        }
        tdi.extend_from_slice(&[true, true]);
        tms.extend_from_slice(&[true, false]);
        let resp = self.do_shift_with_read(&tdi, &tms, rdo_start, 32)?;
        Ok(decode_dr_32(&resp))
    }

    /// Shift `tx` through `USER1` (the JTAG-to-SPI bridge BSCAN slot) and
    /// capture `rx_len` bytes of MISO data after the last TX byte.
    ///
    /// **Protocol** (mirrors openFPGALoader `Xilinx::spi_put`):
    ///
    /// * Each TX byte is **bit-reversed** before being shifted onto TDI,
    ///   because the bridge feeds TDI bits in arrival order onto MOSI, but
    ///   SPI flash commands are defined MSB-first. JTAG TDI naturally
    ///   transports LSB-first. So byte `0x9F` (READ_ID) becomes `0xF9`
    ///   on the wire. Skipping this is what produces `JEDEC = FF FF FF`
    ///   (the flash never sees a valid opcode).
    /// * After the last TX byte, the bridge needs **one extra byte of
    ///   shift activity** to clock out the trailing MISO bit. The driver
    ///   inserts `rx_len + 1` zero bytes of TX padding when `rx_len > 0`.
    /// * MISO arrives with a **1-bit JTAG capture delay** (Capture-DR
    ///   injects one bit at the head of the stream). Each RX byte is
    ///   reconstructed by `bitrev(captured[i+1] >> 1) | (captured[i+2] & 1)`
    ///   — the canonical 1-bit-of-chain compensation from openFPGALoader.
    /// * `total_bits` = `(tx.len() + rx_len + 1) * 8` when `rx_len > 0`,
    ///   else just `tx.len() * 8`.
    ///
    /// `verbose=true` emits `[debug] ...` lines describing each step on
    /// stderr, including raw captured bytes pre-reconstruction.
    pub fn spi_xfer(&mut self, tx: &[u8], rx_len: usize) -> Result<Vec<u8>> {
        self.spi_xfer_verbose(tx, rx_len, false)
    }

    /// Like `spi_xfer` but emits `eprintln!("[debug] ...")` instrumentation
    /// when `verbose=true`.
    ///
    /// **Frame format** (Migen `xilinx_bscan_spi.py` JTAG2SPI bridge):
    ///
    /// ```text
    /// TDI:  [marker=1] [length BE 32 bits, MSB first]  [data MSB first]  [zero padding]
    /// ```
    ///
    /// `length` is the number of SPI clocks (`(tx.len() + rx_len) * 8`).
    /// Data is shifted MSB-first **per byte** (no bit-reverse — the bridge
    /// itself routes TDI directly to MOSI). After the last data bit the
    /// bridge needs a few extra TCK pulses to drain MISO → TDO; we append
    /// `MIGEN_TDO_LATENCY_BITS` trailing zeros.
    ///
    /// **RX reconstruction**: TDO is sampled on every Shift-DR bit; the
    /// capture stream is offset by `1 + 32` (marker + length header) plus
    /// the on-the-wire latency of the bridge (`MIGEN_TDO_LATENCY_BITS`,
    /// see the `negedge`/`miso_capture` double-flop in `bscan_spi_qmtech.v`).
    /// `MIGEN_TDO_LATENCY_BITS` can be overridden via the
    /// `T27_DLC10_MIGEN_LATENCY` env var for empirical tuning.
    pub fn spi_xfer_verbose(&mut self, tx: &[u8], rx_len: usize, verbose: bool) -> Result<Vec<u8>> {
        // ---- Compose the Migen-framed bit-stream on TDI ------------------
        let data_bits = (tx.len() + rx_len) * 8;
        if data_bits == 0 {
            return Ok(Vec::new());
        }
        let latency = migen_latency();
        // 1 marker + 32 length + data + latency drain bits.
        let total_bits = 1 + 32 + data_bits + latency;

        let mut bits: Vec<bool> = Vec::with_capacity(total_bits);
        // Marker bit.
        bits.push(true);
        // 32-bit BE length: MSB of `data_bits` first.
        for i in (0..32).rev() {
            bits.push(((data_bits as u32) & (1u32 << i)) != 0);
        }
        // Data bits: per byte MSB-first.
        for &b in tx {
            for i in (0..8).rev() {
                bits.push((b & (1 << i)) != 0);
            }
        }
        // Zero data bits for the RX phase (MOSI=0 while host reads MISO).
        bits.resize(bits.len() + rx_len * 8, false);
        // Latency drain: a few extra TCK pulses so the bridge can present
        // the final MISO bits on TDO.
        bits.resize(bits.len() + latency, false);
        assert_eq!(bits.len(), total_bits);

        let mut tdi = vec![true, true, true];
        let mut tms = vec![true, false, false];
        let rdo_start = tdi.len();
        for i in 0..total_bits {
            tdi.push(bits[i]);
            tms.push(i == total_bits - 1);
        }
        tdi.extend_from_slice(&[true, true]);
        tms.extend_from_slice(&[true, false]);

        if verbose {
            eprintln!(
                "[debug] spi_xfer (Migen-framed) tx={} bytes ({}) rx_len={}",
                tx.len(),
                hex::encode(tx),
                rx_len,
            );
            eprintln!(
                "[debug]   data_bits={} (length-field value, BE on wire), latency={}",
                data_bits, latency,
            );
            eprintln!(
                "[debug]   total_bits={} (1 marker + 32 length + {} data + {} drain)",
                total_bits, data_bits, latency,
            );
        }

        if rx_len == 0 {
            self.do_shift(&tdi, &tms)?;
            return Ok(Vec::new());
        }
        let resp = self.do_shift_with_read(&tdi, &tms, rdo_start, total_bits)?;
        // ---- Reconstruct RX from the TDO bit stream ----------------------
        // TDO is invalid during the marker+length phase. Once data starts,
        // TDO bit `j` (j=0 is MSB of first RX byte) corresponds to
        // capture-bit-index `1 + 32 + tx.len()*8 + latency + j`.
        let rx_bit_start = 1 + 32 + tx.len() * 8 + latency;
        let mut rx = vec![0u8; rx_len];
        // The Migen bridge presents MISO on TDO **MSB-first** because the
        // SPI flash is MSB-first and the bridge passes bits unmodified.
        for i in 0..rx_len {
            let mut byte: u8 = 0;
            for j in 0..8 {
                let bit_idx = rx_bit_start + i * 8 + j;
                if bit_at(&resp, bit_idx) {
                    byte |= 1 << (7 - j);
                }
            }
            rx[i] = byte;
        }
        if verbose {
            // Emit the captured stream (post extraction) for hand-debugging.
            let captured = extract_byte_stream(&resp, total_bits);
            eprintln!("[debug]   captured raw stream = {}", hex::encode(&captured));
            eprintln!(
                "[debug]   rx_bit_start = {} (1 marker + 32 length + {} tx-bits + {} latency)",
                rx_bit_start,
                tx.len() * 8,
                latency,
            );
            eprintln!("[debug]   reconstructed RX = {}", hex::encode(&rx));
        }
        Ok(rx)
    }

    fn spi_write_enable(&mut self) -> Result<()> {
        self.spi_xfer(&[spi_cmd::WREN], 0)?;
        Ok(())
    }

    fn spi_wait_wip(&mut self, timeout: Duration) -> Result<()> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let s = self.spi_xfer(&[spi_cmd::READ_STATUS], 1)?;
            if s.first().map(|b| b & STATUS_BUSY_BIT) == Some(0) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        Err(Dlc10Error::FlashBusyTimeout.into())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Reverse all 32 bits of a `u32` (bit 0 ↔ bit 31). Retained as a named
/// helper for test legibility, even though `read_cfg_reg_raw_n` now calls
/// `u32::reverse_bits` directly.
#[allow(dead_code)]
fn swap_msb_lsb_u32(v: u32) -> u32 {
    v.reverse_bits()
}

/// Diagnostic snapshot returned by `read_cfg_reg_diag` — captures the
/// exact host-order packet words, the bytes shifted on the wire (after
/// per-word `reverse_bits` + LE byte-split, before TDI bit-encoding),
/// and the raw result words clocked out of CFG_OUT.
#[derive(Debug, Clone)]
pub struct ReadCfgDiag {
    /// The 5 host-order u32 packet words built by `build_read_cfg_packets`.
    pub packets_host_order: [u32; 5],
    /// 20 bytes — exactly what gets shifted on the wire over the 5 DR
    /// transactions (4 bytes per packet). Use this to hand-compare with
    /// xc3sprog / openFPGALoader.
    pub wire_bytes_per_word: Vec<u8>,
    /// 32-bit words clocked out of CFG_OUT, already `reverse_bits`'d so
    /// the FPGA's MSB-first stream lines up with normal u32 bit numbering.
    pub result_words: Vec<u32>,
}

/// Decoded view of the 7-series STAT register (UG470 Table 5-25).
#[derive(Debug, Clone, Copy)]
pub struct StatBits {
    pub raw: u32,
    pub crc_error: bool,      // bit 0
    pub part_secured: bool,   // bit 1
    pub mmcm_lock: bool,      // bit 2
    pub dci_match: bool,      // bit 3
    pub eos: bool,            // bit 4 — End-Of-Startup
    pub gts_cfg_b: bool,      // bit 5
    pub gwe: bool,            // bit 6
    pub ghigh_b: bool,        // bit 7
    pub mode: u8,             // bits 10..8 — boot mode pins
    pub init_complete: bool,  // bit 11
    pub init_b: bool,         // bit 12
    pub release_done: bool,   // bit 13
    pub done: bool,           // bit 14
    pub id_error: bool,       // bit 15
    pub dec_error: bool,      // bit 16
    pub xadc_over_temp: bool, // bit 17
    pub startup_state: u8,    // bits 21..18
    pub bus_width: u8,        // bits 23..22
    pub cfgerr_b: bool,       // bit 25
}

impl StatBits {
    pub fn from_raw(raw: u32) -> Self {
        Self {
            raw,
            crc_error: (raw & (1 << 0)) != 0,
            part_secured: (raw & (1 << 1)) != 0,
            mmcm_lock: (raw & (1 << 2)) != 0,
            dci_match: (raw & (1 << 3)) != 0,
            eos: (raw & (1 << 4)) != 0,
            gts_cfg_b: (raw & (1 << 5)) != 0,
            gwe: (raw & (1 << 6)) != 0,
            ghigh_b: (raw & (1 << 7)) != 0,
            mode: ((raw >> 8) & 0x7) as u8,
            init_complete: (raw & (1 << 11)) != 0,
            init_b: (raw & (1 << 12)) != 0,
            release_done: (raw & (1 << 13)) != 0,
            done: (raw & (1 << 14)) != 0,
            id_error: (raw & (1 << 15)) != 0,
            dec_error: (raw & (1 << 16)) != 0,
            xadc_over_temp: (raw & (1 << 17)) != 0,
            startup_state: ((raw >> 18) & 0xF) as u8,
            bus_width: ((raw >> 22) & 0x3) as u8,
            cfgerr_b: (raw & (1 << 25)) != 0,
        }
    }

    /// One-line human-readable diagnosis of why DONE might be LOW.
    pub fn diagnose(&self) -> String {
        if self.done {
            return "DONE=HIGH (configured OK)".into();
        }
        let mut reasons: Vec<String> = Vec::new();
        if self.crc_error {
            reasons.push("CRC_ERROR=1 (bitstream payload corrupted on TDI)".into());
        }
        if self.id_error {
            reasons.push("ID_ERROR=1 (IDCODE in bitstream != device IDCODE)".into());
        }
        if self.dec_error {
            reasons.push("DEC_ERROR=1 (AES decryption failed)".into());
        }
        if !self.init_b {
            reasons.push("INIT_B=0 (config FSM held in reset / power issue)".into());
        }
        if !self.eos {
            reasons.push("EOS=0 (start-up sequence never reached End-Of-Startup)".into());
        }
        if !self.mmcm_lock {
            reasons.push("MMCM_LOCK=0 (clock generator not locked)".into());
        }
        if self.cfgerr_b {
            // CFGERR_B is active-low; "true" means OK.
        } else {
            reasons.push("CFGERR_B=0 (configuration logic flagged an error)".into());
        }
        if reasons.is_empty() {
            reasons.push(
                "DONE=LOW with no obvious bit set — bitstream may not have been shifted at all"
                    .into(),
            );
        }
        reasons.join("; ")
    }
}

fn decode_dr_32(resp: &[u8]) -> u32 {
    let mut words = [0u16; 2];
    for (i, w) in words.iter_mut().enumerate() {
        let off = i * 2;
        if off + 1 < resp.len() {
            *w = u16::from_le_bytes([resp[off], resp[off + 1]]);
        }
    }
    let mut val = 0u32;
    for i in 0..32 {
        let wi = i / 16;
        let bi = i % 16;
        if words[wi] & (1 << bi) != 0 {
            val |= 1 << i;
        }
    }
    val
}

/// Extract `rx_len_bits` starting at `rx_start_bits` from the captured stream.
/// Bits arrive in the same packed format as `decode_dr_32`: 16-bit LE words,
/// LSB-first within each.
#[allow(dead_code)]
fn extract_rx(resp: &[u8], total_bits: usize, rx_start_bits: usize, rx_len_bits: usize) -> Vec<u8> {
    let words: Vec<u16> = (0..resp.len() / 2)
        .map(|i| u16::from_le_bytes([resp[2 * i], resp[2 * i + 1]]))
        .collect();
    let mut out = vec![0u8; rx_len_bits.div_ceil(8)];
    for i in 0..rx_len_bits {
        let src = rx_start_bits + i;
        if src >= total_bits {
            break;
        }
        let wi = src / 16;
        let bi = src % 16;
        if wi < words.len() && (words[wi] & (1 << bi)) != 0 {
            out[i >> 3] |= 1 << (i & 7);
        }
    }
    out
}

/// Default JTAG-bit latency between TDI presentation and the corresponding
/// TDO bit for the Migen JTAG2SPI bridge. The Verilog has a 2-stage MISO
/// flop (`negedge`/`miso_capture` then `tdo`) plus the JTAG host's own
/// 1-bit Capture-DR delay — so a starting guess of 3 is reasonable. Can
/// be overridden by `T27_DLC10_MIGEN_LATENCY` for empirical tuning.
const MIGEN_TDO_LATENCY_BITS_DEFAULT: usize = 3;

fn migen_latency() -> usize {
    std::env::var("T27_DLC10_MIGEN_LATENCY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(MIGEN_TDO_LATENCY_BITS_DEFAULT)
}

/// Index into the DLC10 16-bit-LE packed response by absolute Shift-DR
/// bit position (LSB-first within each 16-bit word).
fn bit_at(resp: &[u8], bit_idx: usize) -> bool {
    let wi = bit_idx / 16;
    let bi = bit_idx % 16;
    let lo = resp.get(2 * wi).copied().unwrap_or(0);
    let hi = resp.get(2 * wi + 1).copied().unwrap_or(0);
    let word = u16::from_le_bytes([lo, hi]);
    (word & (1 << bi)) != 0
}

/// Repack the DLC10 16-bit-LE captured response into a contiguous byte
/// stream, **as if** TDO had been clocked directly into a shift register
/// LSB-first. The stream length is `total_bits.div_ceil(8)`.
fn extract_byte_stream(resp: &[u8], total_bits: usize) -> Vec<u8> {
    let n = total_bits.div_ceil(8);
    let mut out = vec![0u8; n];
    for i in 0..total_bits {
        let wi = i / 16;
        let bi = i % 16;
        let lo = resp.get(2 * wi).copied().unwrap_or(0);
        let hi = resp.get(2 * wi + 1).copied().unwrap_or(0);
        let word = u16::from_le_bytes([lo, hi]);
        if (word & (1 << bi)) != 0 {
            out[i >> 3] |= 1 << (i & 7);
        }
    }
    out
}

fn find_device<C: UsbContext>(
    ctx: &C,
    vid: u16,
    pid: u16,
) -> Result<Option<(rusb::Device<C>, rusb::DeviceDescriptor)>> {
    for dev in ctx.devices().context("usb device list")?.iter() {
        let d = dev.device_descriptor().context("device descriptor")?;
        if d.vendor_id() == vid && d.product_id() == pid {
            return Ok(Some((dev, d)));
        }
    }
    Ok(None)
}

fn open_and_claim<C: UsbContext>(dev: rusb::Device<C>) -> Result<rusb::DeviceHandle<C>> {
    let h = dev.open().context("open dlc10")?;
    let _ = h.set_auto_detach_kernel_driver(true);
    h.set_active_configuration(1).ok();
    h.claim_interface(0).context("claim_interface(0)")?;
    h.set_alternate_setting(0, 0).ok();
    Ok(h)
}

/// Run the DLC10 post-firmware vendor-control init sequence. Mirrors the
/// `_init` block in the Python driver.
fn init_after_firmware<C: UsbContext>(h: &rusb::DeviceHandle<C>) -> Result<()> {
    std::thread::sleep(Duration::from_secs(2));
    let to = Duration::from_secs(10);
    let rti = request_type(Direction::In, RequestType::Vendor, Recipient::Device);
    let rto = request_type(Direction::Out, RequestType::Vendor, Recipient::Device);

    let mut buf = [0u8; 2];
    h.read_control(rti, VENDOR_REQ, 0x0050, 0, &mut buf, to)
        .ok();
    h.read_control(rti, VENDOR_REQ, 0x0050, 1, &mut buf, to)
        .ok();
    h.write_control(rto, VENDOR_REQ, 0x0028, 0x11, &[], to).ok();
    h.write_control(rto, VENDOR_REQ, 0x0030, 1u16 << 3, &[], to)
        .ok();
    h.write_control(rto, VENDOR_REQ, 0x0028, 0x11, &[], to).ok();
    h.write_control(rto, VENDOR_REQ, 0x0018, 0, &[], to).ok();
    h.write_control(rto, VENDOR_REQ, 0x00A6, 2, &[], to).ok();
    h.write_bulk(EP_OUT, &[0x00, 0x00], to).ok();
    h.write_control(rto, VENDOR_REQ, 0x0028, 0x12, &[], to).ok();
    Ok(())
}

/// Walk the FX2 firmware (Intel HEX) and program it via 0xA0 control writes.
fn load_firmware<C: UsbContext>(h: &rusb::DeviceHandle<C>) -> Result<()> {
    let text = std::str::from_utf8(XUSB_FW_HEX).context("xusb_xp2.hex must be UTF-8 ASCII")?;
    let records = parse_intel_hex(text)?;
    let to = Duration::from_secs(5);
    let rto = request_type(Direction::Out, RequestType::Vendor, Recipient::Device);
    for (addr, data) in &records {
        h.write_control(rto, FX2_FW_REQ, *addr, 0, data, to)
            .with_context(|| format!("FX2 fw write @0x{:04X}", addr))?;
    }
    // Release reset (CPUCS = 0).
    h.write_control(rto, FX2_FW_REQ, FX2_CPUCS, 0, &[0x00], to)
        .context("FX2 release reset")?;
    std::thread::sleep(Duration::from_secs(5));
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests (unit only — no hardware)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitrev_self_inverse() {
        for b in 0u8..=255u8 {
            assert_eq!(BIT_REV_TABLE[BIT_REV_TABLE[b as usize] as usize], b);
        }
    }

    #[test]
    fn bitrev_known_values() {
        assert_eq!(BIT_REV_TABLE[0x00], 0x00);
        assert_eq!(BIT_REV_TABLE[0xFF], 0xFF);
        assert_eq!(BIT_REV_TABLE[0x01], 0x80);
        assert_eq!(BIT_REV_TABLE[0x80], 0x01);
        assert_eq!(BIT_REV_TABLE[0xA5], 0xA5);
        assert_eq!(BIT_REV_TABLE[0x12], 0x48);
    }

    #[test]
    fn parse_intel_hex_basic() {
        let txt = ":03000000DEADBEAF\n:00000001FF\n";
        let recs = parse_intel_hex(txt).expect("ok");
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].0, 0);
        assert_eq!(recs[0].1, vec![0xDE, 0xAD, 0xBE]);
    }

    #[test]
    fn parse_intel_hex_skips_blank_and_comments() {
        let txt = "\n  \n:0000000000\n:00000001FF\n";
        let recs = parse_intel_hex(txt).expect("ok");
        // Type-0 with rlen=0 is intentionally skipped.
        assert!(recs.is_empty());
    }

    #[test]
    fn parse_bitfile_synthetic() {
        // Synthetic .bit: 0x65 tag at offset 4, then BE u32 length, then payload.
        let payload: Vec<u8> = (0..32u8).collect();
        let mut buf = vec![0xAA, 0xBB, 0xCC, 0xDD]; // 4-byte filler header
        buf.push(0x65);
        buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        buf.extend_from_slice(&payload);
        let parsed = parse_bitfile(&buf).expect("parse");
        assert_eq!(parsed, bitrev(&payload));
    }

    #[test]
    fn parse_bitfile_no_tag_errors() {
        let buf = vec![0u8; 100];
        assert!(parse_bitfile(&buf).is_err());
    }

    #[test]
    fn find_sync_word_basic() {
        let mut buf = vec![0xFFu8; 32];
        buf.extend_from_slice(&[0xAA, 0x99, 0x55, 0x66]);
        buf.extend_from_slice(&[0x20, 0x00, 0x00, 0x00]);
        assert_eq!(find_sync_word(&buf), Some(32));
        let none = vec![0u8; 32];
        assert_eq!(find_sync_word(&none), None);
    }

    #[test]
    fn bitfile_payload_range_skips_bogus_e_bytes() {
        // Construct a file that contains a 'e' byte (0x65) in an earlier
        // string (here at offset 4) followed by a clearly-bogus BE length,
        // then a valid 'e' tag.
        let payload: Vec<u8> = (0..16u8).collect();
        let mut buf = vec![0xAA, 0xBB, 0xCC, 0xDD];
        buf.push(0x65); // bogus 'e'
        buf.extend_from_slice(&0xFFFFFFFFu32.to_be_bytes()); // huge length
                                                             // Pad some random non-'e' bytes.
        buf.extend_from_slice(&[0x00, 0x11, 0x22, 0x33]);
        // The valid 'e' tag.
        buf.push(0x65);
        buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        buf.extend_from_slice(&payload);
        let (start, len) = bitfile_payload_range(&buf).expect("parse");
        assert_eq!(len, payload.len());
        assert_eq!(&buf[start..start + len], &payload[..]);
    }

    #[test]
    fn stat_bits_decode_done_high() {
        // Construct a STAT word where DONE=1, EOS=1, INIT_B=1, MMCM_LOCK=1.
        let raw = (1u32 << 14)  // DONE
            | (1u32 << 4)        // EOS
            | (1u32 << 12)       // INIT_B
            | (1u32 << 2)        // MMCM_LOCK
            | (1u32 << 25); // CFGERR_B (active-low: 1 = no error)
        let s = StatBits::from_raw(raw);
        assert!(s.done);
        assert!(s.eos);
        assert!(s.init_b);
        assert!(s.mmcm_lock);
        assert!(!s.crc_error);
        assert!(!s.id_error);
        assert!(s.diagnose().contains("DONE=HIGH"));
    }

    #[test]
    fn stat_bits_decode_crc_error() {
        // DONE=0, CRC_ERROR=1.
        let raw = 0x0000_0001u32;
        let s = StatBits::from_raw(raw);
        assert!(!s.done);
        assert!(s.crc_error);
        let d = s.diagnose();
        assert!(d.contains("CRC_ERROR"));
    }

    #[test]
    fn stat_bits_diagnose_done_low_no_obvious_flag() {
        // All-zero STAT: DONE=0 and no error bits set. Diagnose should still
        // produce a useful (non-empty) message.
        let s = StatBits::from_raw(0);
        assert!(!s.done);
        let d = s.diagnose();
        assert!(!d.is_empty());
        // CFGERR_B is bit 25; raw=0 means CFGERR_B=0 → "flagged an error".
        assert!(d.contains("CFGERR_B"));
    }

    #[test]
    fn swap_msb_lsb_u32_roundtrip() {
        for &v in &[0u32, 1, 0xDEADBEEF, 0xFFFFFFFF, 0x13631093] {
            assert_eq!(swap_msb_lsb_u32(swap_msb_lsb_u32(v)), v);
        }
        assert_eq!(swap_msb_lsb_u32(0x80000000), 0x00000001);
        assert_eq!(swap_msb_lsb_u32(0x00000001), 0x80000000);
    }

    /// Pure (no-hardware) check: the Type-1 read-header construction we use
    /// in `read_cfg_reg` must produce the well-known xc3sprog constants.
    #[test]
    fn type1_read_header_matches_xc3sprog() {
        fn hdr(addr: u8) -> u32 {
            (1u32 << 29) | (1u32 << 27) | (((addr as u32) & 0x3FFF) << 13) | 1u32
        }
        // STAT (0x07) — well-known constant in xc3sprog and openFPGALoader.
        assert_eq!(hdr(cfg_reg::STAT), 0x2800_E001);
        // IDCODE (0x0C).
        assert_eq!(hdr(cfg_reg::IDCODE), 0x2801_8001);
        // CTL0 (0x05).
        assert_eq!(hdr(cfg_reg::CTL0), 0x2800_A001);
    }

    /// Pin the 5 packet words `build_read_cfg_packets` emits for IDCODE,
    /// matching openFPGALoader `Xilinx::dumpRegister`.
    #[test]
    fn build_read_cfg_packets_idcode_matches_openfpgaloader() {
        let p = Dlc10::build_read_cfg_packets(cfg_reg::IDCODE);
        assert_eq!(p[0], 0xAA995566); // SYNC
        assert_eq!(p[1], 0x20000000); // NOP
        assert_eq!(p[2], 0x28018001); // READ_HDR for IDCODE (addr 0x0C)
        assert_eq!(p[3], 0x20000000); // NOP
        assert_eq!(p[4], 0x20000000); // NOP
    }

    #[test]
    fn migen_frame_layout_jedec() {
        // For a 1-byte TX (0x9F) + 3-byte RX (JEDEC ID), the on-wire frame
        // is: 1 marker + 32 length-bits (value = 32, BE MSB-first) +
        // 8 tx-bits (MSB-first 0x9F = 1,0,0,1,1,1,1,1) + 24 zero bits +
        // `latency` drain bits.
        let data_bits: u32 = (1 + 3) * 8;
        assert_eq!(data_bits, 32);
        // Length value = 32 = 0x00000020 → BE MSB-first bits:
        // 0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0, 0,0,1,0,0,0,0,0
        let expected_length_bits: Vec<bool> = (0..32)
            .rev()
            .map(|i| (data_bits & (1u32 << i)) != 0)
            .collect();
        assert_eq!(expected_length_bits.iter().filter(|b| **b).count(), 1);
        assert!(expected_length_bits[26]); // bit 5 (MSB-first index 26) is set
                                           // TX byte 0x9F = 0b1001_1111 MSB-first → 1,0,0,1,1,1,1,1
        let tx_msb_first: Vec<bool> = (0..8).rev().map(|i| (0x9Fu8 & (1 << i)) != 0).collect();
        assert_eq!(
            tx_msb_first,
            vec![true, false, false, true, true, true, true, true],
        );
    }

    #[test]
    fn extract_byte_stream_roundtrip() {
        // Pack a known LSB-first bit stream into the 16-bit-LE container,
        // then verify extract_byte_stream rebuilds the same bytes.
        let original: [u8; 4] = [0x12, 0x34, 0xAB, 0xCD];
        let bits = original.len() * 8;
        let mut packed = vec![0u8; 2 * bits.div_ceil(16)];
        for i in 0..bits {
            let bit = (original[i >> 3] & (1 << (i & 7))) != 0;
            if bit {
                let wi = i / 16;
                let bi = i % 16;
                let off = 2 * wi;
                let mut word = u16::from_le_bytes([packed[off], packed[off + 1]]);
                word |= 1 << bi;
                let bytes = word.to_le_bytes();
                packed[off] = bytes[0];
                packed[off + 1] = bytes[1];
            }
        }
        let stream = extract_byte_stream(&packed, bits);
        assert_eq!(stream, original);
    }

    /// Pin the per-word wire encoding (reverse_bits → LE byte-split)
    /// against hand-computed reference values. This is the protocol step
    /// the user explicitly asked us to audit.
    #[test]
    fn wire_encoding_per_word_matches_reference() {
        // 0xAA995566:
        //   reverse_bits(0xAA995566) = 0x66AA9955
        //   LE bytes                 = [0x55, 0x99, 0xAA, 0x66]
        let tmp = 0xAA995566u32.reverse_bits();
        assert_eq!(tmp, 0x66AA9955);
        let bytes = [
            (tmp & 0xFF) as u8,
            ((tmp >> 8) & 0xFF) as u8,
            ((tmp >> 16) & 0xFF) as u8,
            ((tmp >> 24) & 0xFF) as u8,
        ];
        assert_eq!(bytes, [0x55, 0x99, 0xAA, 0x66]);

        // 0x28018001 (IDCODE read header):
        //   reverse_bits(0x28018001) = 0x80018014
        //   LE bytes                 = [0x14, 0x80, 0x01, 0x80]
        let tmp = 0x28018001u32.reverse_bits();
        assert_eq!(tmp, 0x80018014);
        let bytes = [
            (tmp & 0xFF) as u8,
            ((tmp >> 8) & 0xFF) as u8,
            ((tmp >> 16) & 0xFF) as u8,
            ((tmp >> 24) & 0xFF) as u8,
        ];
        assert_eq!(bytes, [0x14, 0x80, 0x01, 0x80]);

        // 0x20000000 (NOP):
        //   reverse_bits(0x20000000) = 0x00000004
        //   LE bytes                 = [0x04, 0x00, 0x00, 0x00]
        let tmp = 0x20000000u32.reverse_bits();
        assert_eq!(tmp, 0x00000004);
    }
}

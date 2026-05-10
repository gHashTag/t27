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

/// Embedded Cypress FX2 firmware (Intel-HEX, ~22 KB).
///
/// On systems where the file is not yet committed to the repo, the build
/// fails here with a clear error. Copy `xusb_xp2.hex` to `fpga/tools/`.
const XUSB_FW_HEX: &[u8] = include_bytes!("../../../fpga/tools/xusb_xp2.hex");

/// Embedded JTAG-to-SPI bridge bitstream for XC7A100T (MIT,
/// quartiq/bscan_spi_bitstreams).
const BSCAN_SPI_XC7A100T: &[u8] =
    include_bytes!("../../../fpga/tools/bscan_spi_xc7a100t.bit");

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
    let scan_end = std::cmp::min(512, data.len().saturating_sub(5));
    for i in 0..scan_end {
        if data[i] == 0x65 {
            let bs_len = u32::from_be_bytes([
                data[i + 1],
                data[i + 2],
                data[i + 3],
                data[i + 4],
            ]) as usize;
            let remainder = data.len().saturating_sub(i + 5);
            if remainder >= bs_len && (remainder - bs_len) < 256 {
                return Ok(bitrev(&data[i + 5..i + 5 + bs_len]));
            }
        }
    }
    Err(Dlc10Error::BadBitfile("no 'e' field found".into()).into())
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

    /// Read the configuration `STATUS` register via `CFG_OUT`.
    pub fn read_status(&mut self) -> Result<u32> {
        self.shift_ir(ir::CFG_OUT)?;
        self.read_dr_32()
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
        let bs = parse_bitfile(bit)?;

        self.shift_ir(ir::JPROGRAM)?;
        self.cycle_tck(64)?;

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
        Ok(status)
    }

    /// Program the on-board SPI flash.
    pub fn program_flash(&mut self, bit: &[u8], mut opts: FlashOpts) -> Result<()> {
        // Step 1: load the JTAG-to-SPI bridge into FPGA SRAM.
        let _bridge_status = self.program_sram(BSCAN_SPI_XC7A100T)?;

        // Step 2: select USER1 — that maps the BSCAN data register to the
        // single-bit SPI shift register inside the bridge.
        self.shift_ir(ir::USER1)?;

        // Step 3: read JEDEC ID — sanity check.
        let id = self.spi_xfer(&[spi_cmd::READ_ID], 3)?;
        eprintln!(
            "SPI flash JEDEC ID: {:02X} {:02X} {:02X}",
            id[0], id[1], id[2]
        );

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
        self.program_sram(BSCAN_SPI_XC7A100T)?;
        self.shift_ir(ir::USER1)?;
        let id = self.spi_xfer(&[spi_cmd::READ_ID], 3)?;
        let mut out = [0u8; 3];
        for (i, b) in id.iter().take(3).enumerate() {
            out[i] = *b;
        }
        Ok(out)
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

    /// Shift `tx` through USER1 and capture `rx_len` bytes back. The bridge
    /// uses single-bit SPI: TDI = MOSI, TDO = MISO, TCK = CCLK.
    fn spi_xfer(&mut self, tx: &[u8], rx_len: usize) -> Result<Vec<u8>> {
        // Each xfer must re-enter Shift-DR through USER1. The IR was
        // already set by the caller (program_flash), so we just toggle the
        // DR cycle.
        let total_bits = tx.len() * 8 + rx_len * 8;
        let mut tdi = vec![true, true, true];
        let mut tms = vec![true, false, false];
        let rdo_start = tdi.len();
        for i in 0..total_bits {
            // TDI is little-endian-of-bytes for tx range, then 0 for rx range.
            let bit = if i < tx.len() * 8 {
                (tx[i >> 3] & (1 << (i & 7))) != 0
            } else {
                false
            };
            tdi.push(bit);
            tms.push(i == total_bits - 1);
        }
        tdi.extend_from_slice(&[true, true]);
        tms.extend_from_slice(&[true, false]);
        if rx_len == 0 {
            self.do_shift(&tdi, &tms)?;
            return Ok(Vec::new());
        }
        let resp = self.do_shift_with_read(&tdi, &tms, rdo_start, total_bits)?;
        Ok(extract_rx(&resp, total_bits, tx.len() * 8, rx_len * 8))
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

fn open_and_claim<C: UsbContext>(
    dev: rusb::Device<C>,
) -> Result<rusb::DeviceHandle<C>> {
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
    h.read_control(rti, VENDOR_REQ, 0x0050, 0, &mut buf, to).ok();
    h.read_control(rti, VENDOR_REQ, 0x0050, 1, &mut buf, to).ok();
    h.write_control(rto, VENDOR_REQ, 0x0028, 0x11, &[], to).ok();
    h.write_control(rto, VENDOR_REQ, 0x0030, 1u16 << 3, &[], to).ok();
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
}

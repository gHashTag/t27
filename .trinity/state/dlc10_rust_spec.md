# Spec: Rust DLC10 driver + SPI flash proxy

## Goal

Replace Python `tools/dlc10_jtag.py` with a pure-Rust implementation
**and add SPI flash programming** so the FPGA boots from M25P/N25Q on every
power-up. After this, JTAG cable can be removed.

Repository: `/Users/playom/t27`, branch `feat/trios-bridge`.
Workspace: Cargo, members already include `cli/tri`, `cli/trios-bridge`,
`cli/flash-spi`. Rust-only policy enforced by pre-commit Gate 4 (`No new
.sh files`).

## Hardware (already verified working)

- DLC10 (Xilinx Platform Cable USB II) plugged into Mac USB hub.
- XILINX VID `0x03FD`, PID READY `0x0008`, PID UNINIT `0x0013`.
- Confirmed via `tools/dlc10_jtag.py` reading `IDCODE = 0x13631093`
  (XC7A100T) on QMTech Wukong V1.
- Firmware blob: `fpga/tools/xusb_xp2.hex` (22956 bytes, Intel HEX format).
- Mode pins M[2:0] = `0b001` (Master SPI, Wukong V1 default).
- SPI flash chip: M25P/N25Q-class on the board, 16 MiB typical.

## What to build

A new Cargo crate `cli/dlc10` (library + binary) plus integration into
existing `cli/flash-spi`.

### Crate `cli/dlc10` — library

`Cargo.toml`:
```toml
[package]
name = "dlc10"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Pure-Rust driver for Xilinx Platform Cable USB II (DLC10/DLC9), supports JTAG + SPI flash via 7-series proxy"

[lib]
path = "src/lib.rs"

[[bin]]
name = "dlc10"
path = "src/bin/dlc10.rs"

[dependencies]
rusb = "0.9"
anyhow = "1"
clap = { version = "4", features = ["derive"] }
thiserror = "1"
hex = "0.4"
```

### Public API (lib.rs)

```rust
pub struct Dlc10 { /* ... */ }

impl Dlc10 {
    /// Open the cable; load firmware if PID is 0x0013.
    pub fn open() -> anyhow::Result<Self>;
    pub fn read_idcode(&mut self) -> anyhow::Result<u32>;

    /// Program the FPGA SRAM (volatile). Returns final STATUS register.
    pub fn program_sram(&mut self, bit: &[u8]) -> anyhow::Result<u32>;

    /// Program the on-board SPI flash (non-volatile).
    /// Loads a 7-series JTAG-to-SPI bridge bitstream into SRAM, then writes
    /// `bit` to flash via the bridge, with read-back verify.
    pub fn program_flash(&mut self, bit: &[u8], opts: FlashOpts) -> anyhow::Result<()>;

    pub fn read_status(&mut self) -> anyhow::Result<u32>;
    pub fn close(self);
}

pub struct FlashOpts {
    pub verify: bool,
    pub progress: Option<Box<dyn FnMut(u64, u64)>>,
}
```

### Port from Python (SRAM path)

Direct translation of `tools/dlc10_jtag.py`. Key constants and routines:

- `VID = 0x03FD`, `PID_UNINIT = 0x0013`, `PID_READY = 0x0008`.
- `XC7_IR { BYPASS=0x3F, IDCODE=0x09, CFG_IN=0x05, CFG_OUT=0x04,
  JPROGRAM=0x0B, JSTART=0x0C, JSHUTDOWN=0x0D, ISC_ENABLE=0x10,
  ISC_DISABLE=0x16, USER1=0x02, USER2=0x03 }`.
- `BIT_REV_TABLE`: precomputed 256-entry bit-reverse (xc3sprog convention).
- `parse_bitfile(bytes)`: scan first 512 bytes, find tag `0x65`, read big-endian
  `u32` length, return bit-reversed payload.
- `_load_firmware`: parse Intel HEX (`fpga/tools/xusb_xp2.hex`), for each
  type-0 record do `ctrl_transfer(0x40, 0xA0, addr, 0, data, 5000)`,
  finally `ctrl_transfer(0x40, 0xA0, 0xE600, 0, [0x00], 5000)` and sleep 5 s.
- `open()` init sequence (after firmware): set_config, claim interface
  (0,0), then a series of vendor ctrl_transfers (see lines 81–90 of py).
- `_do_shift(tdi, tms)`: encode bits 4-per-byte into a 2-byte stride buffer,
  pad if `n % 4 == 0`, send via `ctrl_transfer(0x40, 0xB0, 0xA6, n)` then
  bulk-write to EP `0x02`.
- `shift_ir`, `shift_dr`, `shift_dr_small`, `read_dr_32`, `cycle_tck`,
  `read_idcode`: direct ports.
- **BUG to fix**: current Python `program_xc7` returns `STATUS = 0x00000000`
  and `DONE = LOW` on real hardware. Likely missing `JPROGRAM` pulse before
  `CFG_IN` and a proper `cycle_tck(12)` after `JSTART`. Reference: xc3sprog
  `ProgAlgXC7.cpp::flow_program()` and openOCD `xilinx_pld_xc7.c`.
  Fix the flow in the Rust port.

### NEW: SPI flash path (`program_flash`)

The trick: load a small **JTAG-to-SPI bridge bitstream** that maps SPI
pins (CS_N/CCLK/MOSI/MISO) to JTAG TDO/TDI/TCK via USER1/USER2 instructions.
Then issue SPI commands through the bridge.

Two implementation options:
1. Use a pre-built bridge from xc3sprog/openFPGALoader. We don't have these
   for XC7A100T at hand — would need to source/synthesize one (~15 KB
   bitstream embedded as `include_bytes!`).
2. **Recommended**: skip the bridge and use Xilinx's built-in BSCANE2 +
   STARTUPE2 to drive SPI pins directly via JTAG vendor-specific command
   sequence. This is what Vivado does internally. Reference:
   - Xilinx UG470 §11 "Indirect SPI Configuration"
   - openFPGALoader's `spiFlash.cpp` (used for FT2232-based cables) shows
     the exact SPI command framing (READ_ID 0x9F, READ 0x03, WRITE_EN 0x06,
     PAGE_PROGRAM 0x02, SECTOR_ERASE 0xD8, READ_STATUS 0x05, WRITE_STATUS 0x01).
   - For 7-series, the canonical bridge bitstream is published in xc3sprog's
     `bscan_spi/` directory: `bscan_spi_xc7a100t.bit` (~340 KB).

**Pragmatic plan**: download `bscan_spi_xc7a100t.bit` from
https://github.com/quartiq/bscan_spi_bitstreams (MIT licensed), embed it via
`include_bytes!`. Then:

1. `program_sram(bscan_spi_xc7a100t)` — bridge boots in FPGA SRAM.
2. Read SPI flash JEDEC ID via `READ_ID 0x9F`. Confirm M25P/N25Q.
3. For each 64 KiB sector overlapping `bit`:
   - `WRITE_EN 0x06` → `SECTOR_ERASE 0xD8 <addr>`.
   - Wait for status `WIP=0` (busy bit).
4. For each 256 B page in `bit`:
   - `WRITE_EN 0x06` → `PAGE_PROGRAM 0x02 <addr> <256B>`.
   - Wait for `WIP=0`.
5. If `verify`: read back via `READ 0x03` and `assert_eq!`.
6. Issue `JPROGRAM` → FPGA reconfigures from flash.

For SPI command transport: USER1 selects a 32-bit shift register that
clocks SPI bytes; full sequence is in xc3sprog `progalgxc7.cpp` (~150
lines). Port to Rust as `Dlc10::spi_xfer(cmd: &[u8], rx_len: usize)`.

### Binary `cli/dlc10/src/bin/dlc10.rs`

```text
dlc10 idcode                          # read and print IDCODE
dlc10 sram <file.bit>                 # SRAM program (volatile)
dlc10 flash <file.bit> [--verify]     # SPI flash program (permanent)
dlc10 flash-id                        # read SPI flash JEDEC ID
```

### Integration into `cli/flash-spi`

Replace the current `openFPGALoader` shell-out with a direct call into
`dlc10::Dlc10::program_flash`. Keep CLI flags compatible. Add cargo
dependency `dlc10 = { path = "../dlc10" }`.

## Tests

`cli/dlc10/tests/`:

- `parse_bitfile.rs` — feed a tiny synthetic .bit, confirm offset/length.
- `bitrev.rs` — table consistency.
- `intel_hex.rs` — parse a small HEX, expected (addr,bytes) pairs.
- `idcode.rs` — `#[ignore]` integration test that requires DLC10 plugged in,
  asserts `IDCODE == 0x13631093`.
- `flash_id.rs` — `#[ignore]` integration test, prints JEDEC ID.

## Done = `cargo test -p dlc10` green + on real hardware:

```
cargo run --release -p dlc10 -- idcode
# IDCODE: 0x13631093

cargo run --release -p dlc10 -- flash fpga/vsa/gf16_heartbeat_top.bit --verify
# ... ~60 s ... write OK, verify OK

# unplug DLC10, power-cycle FPGA, D5/D6 must blink the 3-phase heartbeat
```

## Constraints

- Rust 2021 edition. No `unsafe` outside `rusb` calls. No `unwrap()` in
  hot paths. Errors via `anyhow::Result`/`thiserror`.
- Embed firmware HEX and bscan bitstream via `include_bytes!`. Both are
  already in `fpga/tools/`.
- All work on branch `feat/trios-bridge`. Squash-friendly commits.
- Keep `tools/dlc10_jtag.py` for now (don't break SRAM smoke), mark as
  legacy in commit message.
- Pre-commit Gate 4 forbids `.sh` files — ensure none are added.

## What to commit at the end

1. `cli/dlc10/Cargo.toml`, `cli/dlc10/src/lib.rs`, `cli/dlc10/src/bin/dlc10.rs`,
   tests, `cli/dlc10/README.md`.
2. `Cargo.toml` workspace member updated.
3. `fpga/tools/bscan_spi_xc7a100t.bit` (downloaded from upstream, MIT,
   include LICENSE notice in `cli/dlc10/README.md`).
4. `cli/flash-spi/src/main.rs` updated to use `dlc10` crate.
5. `docs/fpga/PERSISTENT_FLASH.md` updated with "now via pure-Rust dlc10".
6. PR comment summarizing what changed and the test results.

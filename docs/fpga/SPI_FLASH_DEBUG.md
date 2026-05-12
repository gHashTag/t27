# SPI flash debug — `JEDEC = FF FF FF` on QMTech XC7A100T

> Refs #592 (DLC10 pure-Rust driver) · Refs #590 (DSLogic diagnostics)
> Last updated: 2026-05-12

## Symptom

```
$ tri fpga flash-id
JEDEC ID: FF FF FF
$ tri fpga program <bit>
SPI flash timeout while waiting for WIP=0
```

`tri fpga idcode` returns `0x13631093` (correct XC7A100T) and `tri fpga sram <bit>`
configures the device successfully (LEDs blink). Only SPI-flash access fails.

## Root-cause hypotheses ranked

### H1 — TX bytes not bit-reversed (now fixed, verified by unit test)

JTAG TDI shifts bits in **arrival order = LSB first**. SPI flash commands are
defined **MSB first**. The JTAG-to-SPI bridge from `quartiq/bscan_spi_bitstreams`
forwards TDI bits straight to MOSI, so each byte must be bit-reversed before
shifting. Without this:

- `READ_ID = 0x9F = 0b1001_1111` arrives as `0b1111_1001 = 0xF9` on MOSI.
- The flash sees an unknown opcode, never drives MISO, the line floats high
  through the pull-up → `FF FF FF`.

openFPGALoader does this in `Xilinx::spi_put`:
```c
jtx[0] = McsParser::reverseByte(cmd);
```

Our previous Rust path skipped the reversal. **`cli/dlc10/src/lib.rs::spi_xfer_verbose`
now uses `BIT_REV_TABLE[b]` on every TX byte.** Pinned by the
`spi_jedec_command_bitrev` unit test.

### H2 — RX bytes not re-aligned for the 1-bit JTAG capture skew (now fixed)

The TAP's Capture-DR cycle injects one bit at the head of the TDO stream
before the chain's TDO bits start flowing. The bridge in turn introduces
one bit of chain delay. So MISO bit `i` appears as bit `i+1` of the captured
stream. Each RX byte must be reconstructed as
`bitrev(captured[i+1] >> 1) | (captured[i+2] & 1)`.

openFPGALoader (`Xilinx::spi_put`, single-chain case):
```c
rx[i] = McsParser::reverseByte(jrx[i+1] >> 1) | (jrx[i+2] & 0x01);
```

This requires **one extra padding byte** in the on-wire TX stream so the
final MISO bit gets clocked out. `spi_xfer_verbose` now appends `rx_len + 1`
zero bytes of padding when `rx_len > 0`.

### H3 — proxy bitstream never reaches `DONE=HIGH`

If the embedded `bscan_spi_xc7a100t.bit` does not configure cleanly (DRC
fail, IDCODE mismatch, CRC error) the bridge is not running and **any**
SPI command will return `FF FF FF`. Diagnose with:

```
tri fpga proxy-load              # load embedded proxy only
tri fpga proxy-status             # read STAT — needs DONE=HIGH
```

Expected output after `proxy-load`:
```
[verbose] post-JPROGRAM STAT=0x0000...  INIT_B=1 INIT_COMPLETE=1
[verbose] final STAT (Type-1 read) = 0x.... (DONE=1, EOS=1, ...)
```

If `DONE=0` after `proxy-load`, the proxy **does not match this board's
pinout** — see H5.

### H4 — flash in deep power-down at JTAG entry

Some board designs (and some bootloaders) leave the flash in deep
power-down (`0xB9`). The first `0x9F` then returns junk until a wake-up
`0xAB` is sent. Newer Micron N25Q parts also support a software reset
sequence `0x66` + `0x99`.

`read_flash_id_verbose` (called by `tri fpga flash-id-debug`) now tries
this recovery automatically:

1. Read JEDEC. If non-FF → done.
2. Issue `0xAB` (Release Power-down). Re-read JEDEC. If non-FF → done.
3. Issue `0x66` + `0x99` (Reset Enable + Reset Device). Re-read JEDEC.

### H5 — proxy pinout mismatch (QMTech-specific)

The `quartiq` proxy is built for the *generic* XC7A100T STARTUPE2 / BSCAN
pinout. QMTech XC7A100T core boards have been observed to wire `CCLK` and
`CS_B` to non-default pins. If `tri fpga proxy-status` shows `DONE=HIGH`
but `tri fpga spi-raw 9F --rx 3` still returns `FF FF FF`, the bridge is
running but its `SS_B` / `CCLK` outputs don't reach the flash.

Mitigations (in increasing order of effort):

1. Build a QMTech-specific proxy from
   [quartiq/bscan_spi_bitstreams](https://github.com/quartiq/bscan_spi_bitstreams)
   with an XDC patch (requires Vivado — out of scope for this PR).
2. Use `openFPGALoader --board qmtech_xc7a100t` to generate a proxy
   (their `spiOverJtag` set has board-specific variants).
3. Confirm with a logic analyser that `CCLK` toggles during
   `tri fpga spi-raw 9F --rx 3`. If it doesn't, the bridge is broken.
4. Fall back to direct configuration-FSM flash programming over CFG_IN
   (UG470 §6 — `WBSTAR` warm-boot), which bypasses the bridge entirely.

## Diagnostic command reference

All commands assume the DLC10 cable is plugged in and the FPGA is powered.

```
# Sanity — TAP intact?
tri fpga idcode                          # → 0x13631093
tri fpga ir-probe 02                     # IR=USER1 capture; expect 0x01

# Load the embedded JTAG-to-SPI proxy and confirm it configured.
tri fpga proxy-load                      # uses fpga/tools/bscan_spi_xc7a100t.bit
tri fpga proxy-status                    # must show DONE=1

# Single-shot SPI transactions (proxy must already be loaded).
tri fpga spi-raw 9F --rx 3               # JEDEC ID
tri fpga spi-raw AB                      # Release Power-down
tri fpga spi-raw 66                      # Reset Enable
tri fpga spi-raw 99                      # Reset Device
tri fpga spi-raw 05 --rx 1               # Status register
tri fpga spi-raw 9F --rx 20              # extended electronic signature

# End-to-end with automatic recovery + maximum logging.
tri fpga flash-id-debug
```

## Decision matrix from real output

| `proxy-status` | `spi-raw 9F --rx 3` | Conclusion |
| --- | --- | --- |
| `DONE=0` | `FF FF FF` | Proxy did not configure. Check `STAT.diagnose()`; rebuild proxy for this board (H5). |
| `DONE=1` | `FF FF FF` | Bridge runs but flash unreachable. Try `tri fpga spi-raw AB` then re-read (H4). If still FF: pinout mismatch (H5). |
| `DONE=1` | `00 00 00` | MISO stuck low — wrong pin or chip in reset. Probe CS/SO with logic analyser. |
| `DONE=1` | `20 BA 18` (or similar) | **Flash alive.** Micron MT25Q128 (0x20=Micron, 0x18=128 Mbit). `tri fpga program <bit>` should now work. |
| `DONE=1` | `EF 40 18` | Winbond W25Q128. Same `program` path. |
| `DONE=1` | `C2 20 18` | Macronix MX25L128. Same `program` path. |

## Code changes in this PR

- `cli/dlc10/src/lib.rs`:
  - `spi_xfer_verbose`: bit-reverse TX; pad on-wire stream by `rx_len + 1` bytes;
    reconstruct RX with the 1-bit shift compensation from openFPGALoader.
  - `read_flash_id_verbose`: auto-recovery via `0xAB`, then `0x66` + `0x99`.
  - `proxy_load`, `proxy_status`, `spi_raw`, `probe_ir_capture`: pure
    diagnostic Rust APIs, used by the new `tri fpga` subcommands.
  - `BSCAN_SPI_XC7A100T`: now `pub` so `tri fpga proxy-load` (no arg) can
    use the embedded variant.
  - `spi_extra` module: `RELEASE_PD = 0xAB`, `RESET_ENABLE = 0x66`,
    `RESET_DEVICE = 0x99`.
  - `program_flash`: verbose by default; on `FF FF FF` JEDEC retries the
    recovery sequence and emits an actionable error if it still fails.

- `cli/tri/src/fpga.rs`: new subcommands `ProxyLoad`, `ProxyStatus`,
  `SpiRaw`, `IrProbe`, `FlashIdDebug`.

- Unit tests added (pure, no hardware):
  - `spi_jedec_command_bitrev` — pins the bit-reversal of `0x9F`, `0x06`,
    `0xAB`, `0x66`, `0x99`.
  - `extract_byte_stream_roundtrip` — pins the LSB-first reconstruction.

# dlc10 — pure-Rust driver for the Xilinx DLC10/DLC9

Replaces the legacy Python `tools/dlc10_jtag.py` with a Rust crate providing:

- USB enumeration + Cypress FX2 firmware load (Intel-HEX `xusb_xp2.hex`)
- Low-level JTAG primitives (`shift_ir`, `shift_dr`, `cycle_tck`, …)
- `read_idcode`, `read_status`, `program_sram` (correct UG470 §6 sequence)
- `program_flash`: loads a JTAG-to-SPI bridge bitstream into SRAM, then
  drives the on-board SPI flash (M25P/N25Q-class) via `USER1`

## Critical fixes vs the prior Python attempt

1. **SRAM `JPROGRAM` was missing.** The old flow `JSHUTDOWN → CFG_IN →
   JSTART` left `DONE = LOW`. The correct UG470 §6 sequence is now
   implemented:
   ```
   JPROGRAM   cycle_tck(64)
   JSHUTDOWN  cycle_tck(12)
   CFG_IN     <bitstream>  cycle_tck(1)
   JSTART     cycle_tck(24)
   BYPASS  →  CFG_OUT  →  STATUS
   ```
2. **`chunk_bits = 16379`** for `_do_shift` — explicitly **not** a multiple
   of 4. The DLC10 firmware silently corrupts payloads with multiple-of-4
   bit counts unless padded.
3. **USB endpoints**: `EP_OUT = 0x02`, `EP_IN = 0x86`, vendor-request
   `0xB0`, FX2 firmware-load request `0xA0`, FX2 CPUCS register `0xE600`.

## CLI

```text
dlc10 idcode                       # read and print IDCODE
dlc10 sram   <file.bit>            # SRAM program (volatile)
dlc10 flash  <file.bit> [--verify] # SPI flash program (permanent)
dlc10 flash-id                     # JEDEC ID via JTAG-to-SPI bridge
dlc10 status                       # CFG_OUT STATUS register
```

## Embedded blobs

- `fpga/tools/xusb_xp2.hex` — Cypress FX2 firmware for the DLC10 cable.
  **The file currently committed is a placeholder EOF record.** Copy the
  real 22 956-byte HEX (from a working Vivado / xc3sprog install) onto the
  build host before producing a release binary. Build will succeed with
  the placeholder, but `Dlc10::open()` will fail to bring up the cable.

- `fpga/tools/bscan_spi_xc7a100t.bit` — JTAG-to-SPI bridge bitstream for
  the XC7A100T, **404 986 bytes**, SHA-256
  `6e8cef49958fbab96a217c209782be67f4943ff80ae9c81e51425da41fc975e0`.
  Sourced from
  <https://github.com/quartiq/bscan_spi_bitstreams>, **MIT-licensed**:

  > Copyright © Robert Jördens et al.
  > Permission is hereby granted, free of charge, to any person obtaining
  > a copy of this software and associated documentation files…

  See the upstream repo for the full MIT notice; we redistribute the
  bitstream unmodified.

## Tests

```sh
cargo test -p dlc10                      # unit tests (no hardware needed)
cargo test -p dlc10 -- --ignored         # hardware integration (DLC10 + Wukong)
cargo clippy -p dlc10 -- -D warnings
```

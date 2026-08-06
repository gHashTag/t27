# FPGA Evidence — Wave Loop 388

**Date:** 2026-07-01  
**Board:** QMTech Wukong V1 / **XC7A200T-FGG676-1** (IDCODE `0x03636093`) per `fpga/HARDWARE_SSOT.md`  
**Cable:** Digilent FTDI JTAG cable (`0x0403:0x6014`)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top_200t.bit` (9.3 MB, 200T-compatible)  
**Toolchain:** OpenXC7 + `openFPGALoader` (`-c digilent_hs2`)  

---

## What was attempted

Wave Loop 388 did not perform new hardware work. It inherited the successful bring-up from Wave Loop 385 and left the bitstream / SSOT state unchanged.

## Current status

- `fpga/HARDWARE_SSOT.md` correctly records the connected board as XC7A200T-FGG676 (IDCODE `0x03636093`) and the Digilent FTDI cable.
- The canonical bring-up command remains:
  ```bash
  openFPGALoader -c digilent_hs2 fpga/verilog/ternary_mac_demo_top_200t.bit
  ```
- The last successful SRAM load (W385) reported:
  ```text
  Shift IR 35
  ir: 1 isc_done 1 isc_ena 0 init 1 done 1
  ```
- The 200T-compatible bitstream is present and tracked in the repository.

## Interpretation

- Hardware is not a blocker for the ternary MAC demo.
- W388 focused on proof-lattice expansion and completing 2D function-local array initialization; no FPGA evidence changed.
- The next hardware milestone remains non-volatile SPI flash programming.

## Next step

- When ready, attempt SPI flash programming via `openFPGALoader -f` with a 200T `bscan_spi` proxy, or document the blocker.

---

*φ² + 1/φ² = 3 | TRINITY*

# Wave Loop 398 — Close-Out Report

**Issue:** #1296  
**Branch:** `wave-loop-398`  
**Merged to:** `trinity-rust-rings`  
**Goal:** Make the QMTech Wukong V1 / XC7A200T-FGG676-1 boot-from-flash H2
hypothesis (CCLK/SPI-startup timing) actionable with board-less tooling, and
harden the cold-POR protocol for the next physical session.

## What was implemented

1. **`tri fpga patch-cor0 <in.bit> <out.bit> --oscfsel N`**
   - Rewrites `COR0[22:17]` in a 7-series `.bit` file in place.
   - Emits warnings about the undocumented OSCFSEL-to-MHz mapping and CRC risk.
   - Verified on the default 200T bitstream: `COR0` changed from `0x02003FE5` to
     `0x02063FE5` for `--oscfsel 3`.

2. **`tri fpga cclk-variants <in.bit>`**
   - Generates a sweep directory with one variant per requested raw OSCFSEL value.
   - Default output directory: `build/fpga/cclk_variants`.
   - Verified: produced `ternary_mac_demo_top_200t_oscfsel00.bit` through
     `oscfsel03.bit` from the default bitstream.

3. **Extended `tri fpga bit-config` / `scripts/dump_bit_config.py`**
   - Decodes `CTL0` (security, fallback, over-temp, ICAP, efuse key) and `BSPI`
     (read command, dummy cycles).
   - Warns when `OSCFSEL=0` (default) and when CRC register writes are present.
   - Added `--assert-idcode`, `--assert-spi-x1`, `--assert-cclk-startup` flags.

4. **Hardened `tri fpga smoke-gate`**
   - Now asserts `IDCODE=0x03636093`, `SPI_BUSWIDTH=x1`, and `STARTUPCLK=CCLK`.
   - Continues the existing yosys synthesis smoke.
   - Verified green on the default 200T demo bitstream.

5. **Hardened `tri fpga boot-log`**
   - Instructs the user to **disconnect the JTAG cable before POR** and reconnect
     it only after the board is stable (per AR66954 / XAPP1188).
   - Writes a JSON log entry to `build/fpga/boot-log-<timestamp>.json` for
     comparison across CCLK variants.
   - Decision tree now distinguishes `MODE` mismatch from H2 CCLK timing.

6. **Updated `fpga/HARDWARE_SSOT.md`**
   - Added §3.3 H2 CCLK/SPI-startup timing decision tree.
   - Added §3.4 CCLK variant generation protocol.
   - Updated §3.1 cold-POR protocol with JTAG-cable-disconnect and JSON logging.
   - Added §9 documenting `patch-cor0` / `cclk-variants` and the OSCFSEL
     uncertainty.

## What was *not* completed

A true user-assisted cold power-cycle with the CCLK variants was **not run** in
this wave. The next physical session (W399 Variant A) will:
- generate variants,
- program each to flash,
- disconnect the JTAG cable during POR,
- reconnect and capture `STAT`,
- identify the first `OSCFSEL` value that reaches `DONE=1`.

## Verification

- `tri fpga patch-cor0` and `tri fpga cclk-variants` produce valid `.bit` files
  whose `bit-config` output shows the requested `OSCFSEL` values.
- `tri fpga bit-config --assert-idcode 0x03636093 --assert-spi-x1
  --assert-cclk-startup` passes.
- `tri fpga smoke-gate` passes.
- Conformance suite: **575/575 PASS**.

## Documents produced

- `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-08.md` — W398 evidence.
- `docs/reports/FPGA_LOOP_COOPERATION_2026-07-09.md` — W399 variants.
- `docs/reports/WAVE_LOOP_398_REPORT.md` — this close-out.
- `fpga/HARDWARE_SSOT.md` — updated H2 decision tree and tooling docs.

## Cooperation variants for W399

- **A (default):** run the cold-POR CCLK sweep, measure actual CCLK, and commit a
  working default bitstream.
- **B (fallback):** board-less CI hardening and reproducible openXC7 toolchain
  recipe.
- **C (insurance):** Vivado-in-Docker controlled comparison to isolate openXC7 vs
  vendor behavior.

---

*phi^2 + phi^-2 = 3 | TRINITY*

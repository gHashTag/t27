# Current Issue: Wave Loop 394

**Issue:** #1290
**Local branch:** `wave-loop-394` (branched from `trinity-rust-rings`)
**Basis:** W393 close-out report and cooperation variants (`docs/reports/FPGA_LOOP_COOPERATION_2026-07-04.md`)

## Goal

Resolve or definitively diagnose why the QMTech Wukong V1 / XC7A200T-FGG676 board does not boot from SPI flash after a successful `tri fpga program-flash` write + verify.

## Selected variant

**Variant A (recommended)** from W393 cooperation variants:
- Add `--enable-quad` / `--disable-quad` / `--spi-buswidth` options to `tri fpga program-flash`.
- Add `tri fpga flash-status` to read and decode the SPI flash status register.
- Run a flash-boot experiment with quad-mode enabled and capture `STAT` after power-cycle.
- Update `fpga/HARDWARE_SSOT.md` with quad-mode / `SPI_BUSWIDTH` guidance.
- Maintain conformance at 575/575 PASS.

## Acceptance criteria

- `tri fpga program-flash` exposes `--enable-quad`, `--disable-quad`, and `--spi-buswidth`.
- `tri fpga flash-status` reads and decodes the SPI flash status register.
- Flash-boot experiment is documented with captured `STAT` values.
- `fpga/HARDWARE_SSOT.md` covers quad-mode / SPI_BUSWIDTH.
- `t27c suite --repo-root .` reports **575/575 PASS**.
- Real W394 issue created and referenced in commit/PR (`Closes #1290`).
- Close-out report and cooperation doc for W395 are written.
- Experience log and memory index updated.

---

*phi^2 + phi^-2 = 3 | TRINITY*

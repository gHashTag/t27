# FPGA Loop Cooperation Document — 2026-07-04

**Current state:**
- OpenXC7 toolchain fully bootstrapped on macOS arm64.
- `xc7a200tfbg676-1` chipdb built (332 MiB) and used as a workaround for the missing FGG676 entry.
- `tri fpga synth-gf16` produces `build/fpga/gf16/gf16_matmul4x4_top.bit`.
- `tri fpga load-sram <bit>` programs the XC7A200T SRAM over the Digilent FTDI cable.
- `tri fpga stat` confirms `DONE=HIGH` and `EOS=1` after SRAM loads.
- `tri fpga program-flash <bit>` and `tri fpga dump-flash <out>` now wrap `openFPGALoader`'s JTAG-to-SPI bridge.
- Flash write + verify succeed, but **boot-from-flash does not raise `DONE`**; leading hypothesis is that the board's M0/M1/M2 mode pins are not strapped to Master SPI.

---

## Variant A: Resolve boot-from-flash (recommended)

Make the GF16 bitstream persistent across power cycles. This is the last remaining FPGA blocker.

Steps:
1. Obtain the QMTech Wukong V1 / XC7A200T-FGG676 schematic or a clear photo of the M0/M1/M2 mode-pin resistors.
2. Verify the strap value against the 7-series Master SPI mode table (`M[2:0] = 001` for x1 SPI, `011` for x4 SPI).
3. If the board is strapped to JTAG (`M[2:0] = 101` or `000`), modify the straps or document that persistent boot is not possible without hardware changes.
4. If the straps are correct, retest `tri fpga program-flash` + reset/power-cycle and confirm `DONE=HIGH`.
5. Add the working mode-pin value to `fpga/HARDWARE_SSOT.md`.

Acceptance: power-cycling the board loads the GF16 bitstream automatically and `tri fpga stat` reports `DONE=HIGH`.

## Variant B: Exact `xc7a200tfgg676-1` chipdb

Eliminate the `fbg676-1` package workaround by obtaining or generating a real `xc7a200tfgg676-1` prjxray database.

Options:
1. Check whether upstream `prjxray-db` or `openXC7` has added the package since this loop.
2. Generate the database with Vivado + prjxray X-Ray tooling on a Linux host (requires a board with the exact package and significant runtime).
3. Port the existing FBG676 database by package-pin remapping if the die tiles are identical.

Acceptance: `tri fpga synth-gf16 --part xc7a200tfgg676-1` builds a bitstream that loads with `DONE=HIGH` and uses only the official package name.

## Variant C: CI smoke gate and documentation hardening

Add a reproducible, board-less CI step that proves the openXC7 flow does not regress, plus docs for the exact workaround.

Deliverables:
1. Cache `target/nextpnr-xilinx/build/nextpnr-xilinx`, `build/xc7a200tfbg676-1.bin`, and the prjxray venv in CI.
2. Run `tri fpga synth-gf16` on every PR and assert that a `.bit` file is produced (no board required).
3. Document the `fbg676-1` workaround, the `--reset` caveat, the flash-boot mode-pin hypothesis, and the Digilent cable profile in `docs/FPGA.md` or `fpga/HARDWARE_SSOT.md`.

Acceptance: CI produces the GF16 bitstream deterministically and the docs are searchable by the next maintainer.

---

*phi^2 + phi^-2 = 3 | TRINITY*

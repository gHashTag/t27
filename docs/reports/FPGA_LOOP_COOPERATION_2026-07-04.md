# FPGA Loop Cooperation Document — 2026-07-04

**Current state:**
- OpenXC7 toolchain fully bootstrapped on macOS arm64.
- `xc7a200tfbg676-1` chipdb built (332 MiB) and used as a workaround for the missing FGG676 entry.
- `tri fpga synth-gf16` produces `build/fpga/gf16/gf16_matmul4x4_top.bit`.
- `tri fpga load-sram <bit>` programs the XC7A200T SRAM over the Digilent FTDI cable.
- `tri fpga stat` confirms `DONE=HIGH` and `EOS=1`.
- SPI flash / non-volatile programming remains blocked.

---

## Variant A: SPI flash persistence (recommended)

Make the GF16 bitstream persistent across power cycles. The XC7A200T is on the board and the SPI flash is wired to the same JTAG chain, but the in-tree `dlc10` driver only supports Xilinx DLC10 cables, not the attached Digilent FTDI probe.

Options:
1. Build or install a working `bscan_spi` proxy for the XC7A200T through Vivado-in-Docker (currently blocked by installer access) and wire it into `tri fpga program`.
2. Repackage the generic `spiOverJtag_xc7a200t.bit.gz` from openFPGALoader as `spiOverJtag_xc7a200tfgg676.bit.gz` and test flash erase/write with `tri fpga` or `openFPGALoader` directly.
3. Use the OpenXC7 proxy flow if/when `nextpnr-himbaechel` supports the 200T chipdb.

Acceptance: `tri fpga program <bit>` writes the bitstream to SPI flash, survives power-cycle, and reloads automatically on the next board boot.

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
3. Document the `fbg676-1` workaround, the `--reset` caveat, and the Digilent cable profile in `docs/FPGA.md` or `fpga/HARDWARE_SSOT.md`.

Acceptance: CI produces the GF16 bitstream deterministically and the docs are searchable by the next maintainer.

---

*phi^2 + phi^-2 = 3 | TRINITY*

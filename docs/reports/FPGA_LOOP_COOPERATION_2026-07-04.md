# FPGA Loop Cooperation Document — 2026-07-04

**Current state:**
- OpenXC7 toolchain bootstrapped on macOS arm64.
- `gf16_matmul4x4_top.bit` synthesized and successfully loaded into XC7A200T-FGG676 SRAM (`DONE=1`).
- `fpga/tools/load_sram.sh` and `fpga/tools/synth_gf16_matrix.sh` added.
- SPI flash / non-volatile programming remains blocked.

---

## Variant A: Integrate into `tri` CLI (recommended)

Add a new `tri fpga load-sram <bit>` subcommand that wraps `openFPGALoader` for the Digilent FTDI cable and XC7A200T board, plus a `tri fpga synth-gf16` subcommand that drives `synth_gf16_matrix.sh`.

- Low risk: reuses working external tools.
- Keeps the repo-first philosophy (no ad-hoc shell scripts on critical path per L7).
- Acceptance: `tri fpga synth-gf16` produces a bitstream; `tri fpga load-sram <bit>` loads it with `done 1`.

## Variant B: Build a real `xc7a200tfgg676-1` chipdb and retarget

The current workaround uses `xc7a200tfbg676-1` because `prjxray-db` lacks the exact FGG676 package. Build or obtain the `xc7a200tfgg676-1` database and regenerate the chipdb for it. This is the clean long-term fix but requires either:
- Finding an upstream `prjxray-db` entry, or
- Generating it from Vivado / X-Ray data (non-trivial).

## Variant C: Attack SPI flash persistence

Resume the W389 goal of making non-volatile flash reproducible. Options:
1. Build `bscan_spi` proxy for XC7A200T through Vivado-in-Docker (blocked by installer/token).
2. Try openFPGALoader's generic `spiOverJtag_xc7a200t.bit.gz` copied to `spiOverJtag_xc7a200tfgg676.bit.gz` and verify whether flash works on this board.
3. Build the proxy with OpenXC7 if `nextpnr-himbaechel` + 200T chipdb becomes available.

Highest risk / longest path; only choose if persistence is required.

---

*phi^2 + phi^-2 = 3 | TRINITY*

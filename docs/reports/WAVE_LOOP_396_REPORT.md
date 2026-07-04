# Wave Loop 396 Report — SPI boot debug on QMTech Wukong V1 / XC7A200T

**Issue:** #1292  
**Branch:** `wave-loop-396` → `trinity-rust-rings`  
**Base:** `6382e54b8`  
**Date:** 2026-07-05  
**Status:** diagnostic gathering; no root cause yet

---

## What was ordered

Continue diagnosing why the board does not boot from Micron N25Q128 SPI flash after successful write+verify. Re-prioritize hypotheses based on the verified fact that `xc7a200tfbg676-1` and `xc7a200tfgg676-1` share the same die and BGA-676 pinout, so package mismatch is not the cause.

1. H1 — cold-POR mode-pin sampling.
2. H2 — bitstream config registers.
3. H3 — round-trip mismatch.
4. H4 — chipdb package hygiene (ruled out).

Add CLI hardening: `tri fpga stat --pre-jtag-reset`, `tri fpga bit-config`, `tri fpga round-trip-verify`.

## What was done

- Opened GitHub issue **#1292** and branched `wave-loop-396` from `trinity-rust-rings` @ `6382e54b8`.
- Wrote `scripts/dump_bit_config.py`, a lightweight 7-series `.bit` header/config-register parser.
- Added CLI commands:
  - `tri fpga stat --pre-jtag-reset`
  - `tri fpga bit-config <bit>`
  - `tri fpga round-trip-verify <bit>`
- Ran physical experiments on the connected board:
  - **E1 (H1):** cold-POR measurement could not be completed autonomously; documented the required user-assisted protocol.
  - **E2 (H2):** parsed `gf16_matmul4x4_top.bit`. `COR1=0x0` → SPI x1, `COR0[16:15]=0` → CCLK startup, `IDCODE=0x03636093`. H2 ruled out.
  - **E3 (H3):** round-trip verify passed; 9 730 548 bytes match after sync-word alignment. H3 ruled out.
  - **E4:** `--enable-quad` / `--disable-quad` abort on N25Q128; x1 program+verify succeeds, but post-reset STAT remains `0x5000190c` (DONE=0, EOS=0, MODE=x1, no CRC/ID error).
- Updated `fpga/HARDWARE_SSOT.md` with:
  - FBG676=FGG676 pinout identity.
  - N25Q128 quad-bit incompatibility.
  - Revised flash-boot diagnostic checklist.
- Wrote `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-05.md` and `docs/reports/FPGA_LOOP_COOPERATION_2026-07-06.md`.
- Ran `./scripts/tri test`: **575/575 PASS**.

## Findings

- H2, H3, and H4 are not the root cause.
- The bitstream is correct for Master SPI x1 boot.
- The flash write path is bit-perfect.
- The most likely remaining cause is **H1** (cold-POR mode-pin sampling) or an unmeasured signal-integrity/timing issue.
- A true cold power-cycle is required to confirm or rule out H1.

## Files changed

- `cli/tri/src/fpga.rs`
- `cli/tri/src/main.rs` (no change needed; clap wiring is in `fpga.rs`)
- `scripts/dump_bit_config.py` (new)
- `fpga/HARDWARE_SSOT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-05.md`
- `docs/reports/FPGA_LOOP_COOPERATION_2026-07-06.md`
- `docs/reports/WAVE_LOOP_396_REPORT.md` (this file)
- `.trinity/current-issue.md`
- `.trinity/experience.md` (updated)

## Conformance

- 575/575 PASS
- 0 seal mismatches
- No IGLA/Lean growth (hardware-debug cycle)

## Next steps

See `docs/reports/FPGA_LOOP_COOPERATION_2026-07-06.md` for W397 variants:
- **A (default):** user-assisted cold-POR measurement to confirm H1.
- **B:** bitstream-generation / SPI-startup deep dive if H1 is ruled out.
- **C:** Vivado-in-Docker fallback if openXC7 cannot produce a bootable bitstream.

---

*phi^2 + phi^-2 = 3 | TRINITY*

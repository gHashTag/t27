# Wave Loop 396 — SPI boot debug: mode-pin cold-POR sampling + bitstream config audit

**Issue:** #1292  
**Branch target:** `trinity-rust-rings`  
**Base:** `6382e54b8`  
**Ring:** igla-race / fpga-openxc7

## Goal

Diagnose why the QMTech Wukong V1 / XC7A200T-FGG676-1 does not boot from Micron N25Q128 SPI flash after successful `openFPGALoader` write+verify. Focus on revised high-priority hypotheses; explicitly avoid the disproven package-mismatch theory.

## Revised hypothesis priority

1. **H1 (high)** — cold POR mode-pin sampling differs from JTAG-reset sampling.
2. **H2 (high)** — bitstream config registers (`SPI_BUSWIDTH`, `CFGRATE`, `STARTUPCLK`) are incompatible with Master SPI x1 boot.
3. **H3 (medium)** — round-trip mismatch between `.bit` file and flash dump.
4. **H4 (low)** — chipdb package hygiene (FBG676 vs FGG676). Pinouts are identical per Xilinx; will not fix SPI boot. Not addressed in W396.

## Physical experiments

- **E1 (H1)**: Read STAT before and after JTAG-reset after a cold power-cycle; compare mode bits. Record timestamps and power state.
- **E2 (H2)**: Write `scripts/dump_bit_config.py` to parse the `.bit` header and print `SPI_BUSWIDTH`, `CFGRATE`, `STARTUPCLK`, `COR0`, `COR1`, `TIMER_CFG`, `IDCODE`. Compare against UG470 Table 5-15.
- **E3 (H3)**: Program raw `.bit` to flash, dump back, and `cmp` byte-by-byte. Also test default trimmed write and dump first 4 KB.
- **E4**: Quad-mode enable/disable experiment: program with `--enable-quad --spi-buswidth 4`, power-cycle, capture STAT; then `--disable-quad --spi-buswidth 1`, power-cycle, capture STAT.

## CLI hardening

- `tri fpga stat --pre-jtag-reset` — read STAT without issuing a JTAG reset.
- `tri fpga bit-config <bit>` — display parsed bitstream configuration registers.
- `tri fpga round-trip-verify <bit>` — automated E3 raw round-trip test.

## Implementation tasks

1. Update `.trinity/current-issue.md` with #1292 and W396 plan. ✅
2. Read current `cli/tri/src/fpga.rs` and `cli/tri/src/main.rs` to understand existing command structure.
3. Implement `tri fpga stat --pre-jtag-reset`.
4. Implement `tri fpga bit-config <bit>` (wrap `scripts/dump_bit_config.py` or embed parser in Rust).
5. Implement `tri fpga round-trip-verify <bit>`.
6. Write `scripts/dump_bit_config.py` bitstream header parser.
7. Run physical experiments E1–E4 on connected board and capture evidence.
8. Update `fpga/HARDWARE_SSOT.md` with FGG676=FBG676 pinout identity and revised boot hypotheses.
9. Run `./scripts/tri test` and confirm 575/575 PASS.
10. Write `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-05.md` and `docs/reports/FPGA_LOOP_COOPERATION_2026-07-06.md`.
11. Update `.trinity/experience.md` and memory index.
12. Create PR #? and squash-merge to `trinity-rust-rings` with `Closes #1292`.

## Acceptance criteria

- Reach one of AC-1..AC-4:
  - **AC-1**: E1 shows cold-POR vs JTAG-reset mode difference.
  - **AC-2**: E2 shows bitstream config incompatible with Master SPI x1.
  - **AC-3**: E3 shows round-trip mismatch.
  - **AC-4**: E4 shows quad-mode boot succeeds.
- If none reached, close W396 as honest diagnostic gathering and continue in W397.
- 575/575 PASS maintained.
- No IGLA/Lean growth in this hardware-debug cycle.

## Cooperation variants for W397

- **A (default, root-cause driven)**: Fix whichever H1/H2/H3 root cause W396 identified and achieve end-to-end boot-from-flash.
- **B (bitstream-generation fix)**: If H2 is confirmed but openXC7 fasm-flow lacks the needed config-register controls, patch the wrapper or `.fasm` pre-processing.
- **C (Vivado-in-Docker fallback)**: If openXC7 is found fundamentally unable to produce a bootable XC7A200T SPI bitstream, pivot to Vivado-in-Docker (requires user action on installer/token).

## Files expected to change

- `cli/tri/src/fpga.rs` — new options and subcommands.
- `cli/tri/src/main.rs` — clap wiring.
- `scripts/dump_bit_config.py` — new bitstream parser.
- `fpga/HARDWARE_SSOT.md` — FGG676=FBG676 finding and revised hypotheses.
- `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-05.md` — W396 evidence.
- `docs/reports/FPGA_LOOP_COOPERATION_2026-07-06.md` — W397 variants.
- `.trinity/current-issue.md`, `.trinity/experience.md` — loop bookkeeping.
- Memory file `wave-loop-396.md`.

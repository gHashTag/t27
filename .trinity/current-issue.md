# Current Issue: Wave Loop 396

**Issue:** #1292
**Local branch:** `wave-loop-396` (branched from `trinity-rust-rings` @ `6382e54b8`)
**Basis:** W395 close-out report and revised hypothesis priority (H1 cold-POR mode sampling, H2 bitstream config registers, H3 round-trip, H4 chipdb hygiene low priority)

## Goal

Diagnose why the QMTech Wukong V1 / XC7A200T-FGG676-1 does not boot from Micron N25Q128 SPI flash after successful write+verify, focusing on the revised high-priority hypotheses and avoiding the disproven package-mismatch theory.

## Selected variant

**Variant A (root-cause driven)** from W395 cooperation variants, updated per W396 order:
- Add `tri fpga stat --pre-jtag-reset` to read STAT without issuing a JTAG reset.
- Add `tri fpga bit-config <bit>` to parse and display bitstream config registers.
- Add `tri fpga round-trip-verify <bit>` to automate flash dump round-trip verification.
- Run physical experiments E1–E4 and capture timestamped STAT values in multiple power states.
- Update `fpga/HARDWARE_SSOT.md` with the FBG676 vs FGG676 pinout identity finding.
- Maintain conformance at 575/575 PASS; no IGLA/Lean growth in this hardware-debug cycle.

## Acceptance criteria

- One of AC-1..AC-4 reached:
  - **AC-1**: E1 shows cold-POR STAT mode bits differ from post-JTAG-reset mode bits.
  - **AC-2**: E2 shows bitstream config registers incompatible with Master SPI x1 boot.
  - **AC-3**: E3 shows round-trip mismatch between .bit and flash dump.
  - **AC-4**: E4 shows quad-mode boot succeeds (DONE=1).
- If none reached, W396 closes as honest diagnostic gathering and W397 continues.
- `tri fpga stat --pre-jtag-reset` implemented.
- `tri fpga bit-config <bit>` implemented.
- `tri fpga round-trip-verify <bit>` implemented.
- `t27c suite --repo-root .` reports **575/575 PASS**.
- Real W396 issue created and referenced in commit/PR (`Closes #1292`).
- Close-out report and cooperation doc for W397 are written.
- Experience log and memory index updated.

---

*phi^2 + phi^-2 = 3 | TRINITY*

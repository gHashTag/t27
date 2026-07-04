# Current Issue: Wave Loop 398

**Issue:** #1296
**Local branch:** `wave-loop-398` (branched from `trinity-rust-rings` @ `d370b27ab`)
**Basis:** W397 close-out report; H1 cold-POR mode sampling is likely ruled out,
so H2 (CCLK/SPI-startup timing) is the leading hypothesis.

## Goal

Make the QMTech Wukong V1 / XC7A200T-FGG676-1 boot-from-flash H2 hypothesis
(CCLK/SPI-startup timing or flash state after reset) actionable and testable
with board-less tooling, while documenting the user-assisted cold-POR/CCLK-sweep
protocol for the next physical session.

## Selected variant

**Variant A (root-cause driven)** from W397 cooperation variants, adapted for
board-less tooling + user-assisted physical closure:
- Add `tri fpga patch-cor0 <in.bit> <out.bit> --oscfsel N` to create CCLK-variants.
- Add `tri fpga cclk-variants <in.bit>` to generate a sweep directory.
- Extend `tri fpga bit-config` decoding to CTL0 and BSPI; add assertion flags.
- Harden `tri fpga smoke-gate` to fail on IDCODE/SPI-width/startup-clock regressions.
- Harden `tri fpga boot-log` with JSON logging and JTAG-cable-disconnect instructions
  (per AR66954 / XAPP1188).
- Update `fpga/HARDWARE_SSOT.md` with the H2 decision tree and `patch-cor0` usage.
- Maintain conformance at 575/575 PASS; no IGLA/Lean growth.

## Acceptance criteria

- `tri fpga patch-cor0` produces a `.bit` with the requested raw `OSCFSEL` value
  and warns about the undocumented MHz mapping and CRC risk.
- `tri fpga cclk-variants` produces a sweep directory with named variants.
- `tri fpga bit-config` decodes CTL0/BSPI and warns on `OSCFSEL=0` / enabled CRC.
- `tri fpga smoke-gate` fails CI if the demo bitstream has wrong IDCODE, SPI width,
  or startup clock.
- `tri fpga boot-log` writes a JSON log and tells the user to disconnect the JTAG
  cable before power-cycle.
- `t27c suite --repo-root .` reports **575/575 PASS**.
- Real W398 issue created and referenced in commit/PR (`Closes #1296`).
- Close-out report and cooperation doc for W399 are written.
- Experience log and memory index updated.

---

*phi^2 + phi^-2 = 3 | TRINITY*

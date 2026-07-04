# Current Issue: Wave Loop 397

**Issue:** #1294
**Local branch:** `wave-loop-397` (branched from `trinity-rust-rings` @ `4aed560b`)
**Basis:** W396 close-out report; H1 cold-POR mode sampling was the remaining
high-priority hypothesis.

## Goal

Confirm or rule out H1 cold-POR mode sampling for the QMTech Wukong V1 /
XC7A200T-FGG676-1 boot-from-flash failure. If H1 is ruled out, pivot to H2
(CCLK/SPI-startup timing) in W398.

## Selected variant

**Variant A (root-cause driven)** from W397 cooperation variants:
- Add `tri fpga boot-log <bit>` to guide the cold-POR experiment.
- Add `--repeat N` to `tri fpga stat --pre-jtag-reset` for multiple power-on samples.
- Add board-less `tri fpga smoke-gate` and integrate it into the conformance suite.
- Harden `fpga/HARDWARE_SSOT.md` cold-POR protocol and decision tree.
- Update/deprecate stale `fpga/diagnostics/jtag_wiring.md`.
- Maintain conformance at 575/575 PASS; no IGLA/Lean growth.

## Acceptance criteria

- One of AC-1..AC-4 reached:
  - **AC-1**: H1 confirmed and fix path documented.
  - **AC-2**: H1 ruled out and H2 scoped for W398.
  - **AC-3**: board boots from flash (DONE=1 after cold-POR).
  - **AC-4**: CLI smoke gate implemented and green even without physical board.
- `tri fpga boot-log <bit>` implemented.
- `tri fpga stat --pre-jtag-reset --repeat N` implemented.
- `tri fpga smoke-gate` implemented.
- `t27c suite --repo-root .` reports **575/575 PASS**.
- Real W397 issue created and referenced in commit/PR (`Closes #1294`).
- Close-out report and cooperation doc for W398 are written.
- Experience log and memory index updated.

---

*phi^2 + phi^-2 = 3 | TRINITY*

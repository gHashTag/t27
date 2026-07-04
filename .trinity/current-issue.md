# Current Issue: Wave Loop 399

**Issue:** #1298
**Local branch:** `wave-loop-399` (branched from `trinity-rust-rings` @ `d0bedd818`)
**Basis:** W398 close-out report; H2 CCLK/SPI-startup tooling is complete, but the
physical cold-POR CCLK sweep has not yet been run.

## Goal

Automate the W398 CCLK-sweep workflow so a single user-assisted cold-POR session
on the QMTech Wukong V1 / XC7A200T-FGG676-1 can generate, program, test, and
report on multiple `OSCFSEL` variants, and produce a machine-readable evidence
report.

## Selected variant

**Variant A (root-cause driven)** from W398 cooperation variants, with tooling
automation so the only manual step is the physical power-cycle / cable handling:
- Add `tri fpga cclk-sweep <in.bit>` to generate OSCFSEL variants, program each
to flash, run the interactive cold-POR protocol, capture STAT, write JSON logs,
and print a summary table.
- Add `tri fpga sweep-report` to read all `build/fpga/boot-log-*.json` files and
produce a markdown report identifying the first working variant.
- Add `tri fpga measure-cclk` helper for DSLogic / oscilloscope CCLK capture and
optional CSV parsing.
- Update `fpga/HARDWARE_SSOT.md` with the W399 sweep protocol.
- Maintain conformance at 575/575 PASS; no IGLA/Lean growth.

## Acceptance criteria

- `tri fpga cclk-sweep --dry-run` produces expected synthetic logs without a board.
- `tri fpga sweep-report` reads existing logs and produces a markdown table.
- `tri fpga measure-cclk` prints correct CCLK pin and DSLogic settings.
- `fpga/HARDWARE_SSOT.md` documents the sweep protocol.
- `t27c suite --repo-root .` reports **575/575 PASS**.
- Real W399 issue created and referenced in commit/PR (`Closes #1298`).
- Close-out report and cooperation doc for W400 are written.
- Experience log and memory index updated.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*

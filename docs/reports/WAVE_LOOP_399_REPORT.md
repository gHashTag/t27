# Wave Loop 399 — Close-Out Report

**Issue:** #1298  
**Branch:** `wave-loop-399`  
**Merged to:** `trinity-rust-rings`  
**Goal:** Automate the W398 CCLK-sweep workflow so a single user-assisted cold-POR
session on the QMTech Wukong V1 / XC7A200T-FGG676-1 can generate, program, test,
and report on multiple `OSCFSEL` variants, and produce a machine-readable evidence
report.

## What was implemented

1. **`tri fpga cclk-sweep <in.bit>`**
   - Generates `OSCFSEL` variants (default 0..5, user-overrideable).
   - For each variant: patches `COR0[22:17]`, programs flash, prints the
     JTAG-cable-disconnect + cold-POR protocol, waits for ENTER, captures STAT
     with `--pre-jtag-reset`, and writes a JSON log entry.
   - Continues on failure unless `--stop-on-fail`.
   - Supports `--dry-run` to generate synthetic logs and exercise the report path
     without a physical board.
   - Prints a per-variant summary table and identifies the first working variant.

2. **`tri fpga sweep-report`**
   - Reads all `build/fpga/boot-log-*.json` files.
   - Produces a markdown report with a per-variant table (OSCFSEL, DONE, MODE,
     CRC, ID error, conclusion).
   - Identifies the first variant that reached `DONE=HIGH` and lists next steps.

3. **`tri fpga measure-cclk`**
   - Prints DSLogic Plus / oscilloscope capture instructions for the CCLK pin
     (P12, CFGCLK / CCLK_0, bank 0, 3.3 V).
   - Optionally parses a DSView CSV export to estimate frequency and duty cycle
     via zero-crossing detection.

4. **Updated `fpga/HARDWARE_SSOT.md`**
   - Replaced the manual §3.4 variant protocol with the automated `cclk-sweep`
     command.
   - Added §3.5 CCLK measurement protocol.
   - Expanded §9 to document `cclk-sweep`, `sweep-report`, and `measure-cclk`.

## What was *not* completed

A true user-assisted cold power-cycle with the automated sweep was **not run**
in this wave. The next physical session (W400 Variant A) will run
`tri fpga cclk-sweep` on the board, measure actual CCLK, and commit a working
default bitstream.

## Verification

- `tri fpga cclk-sweep fpga/verilog/ternary_mac_demo_top_200t.bit --dry-run`
  produced 6 synthetic JSON logs and a summary table identifying OSCFSEL=0 as the
  first working variant (expected deterministic dry-run behaviour).
- `tri fpga sweep-report` produced a markdown report from the dry-run logs.
- `tri fpga measure-cclk --csv /tmp/fake_cclk.csv` correctly estimated a 25 MHz
  synthetic signal at 50 % duty cycle.
- `cargo build --release -p tri` succeeds.
- Conformance suite: **575/575 PASS**.

## Documents produced

- `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-04.md` — W399 evidence.
- `docs/reports/FPGA_LOOP_COOPERATION_2026-07-05.md` — W400 variants.
- `docs/reports/WAVE_LOOP_399_REPORT.md` — this close-out.
- `fpga/HARDWARE_SSOT.md` — updated sweep and measurement protocol.

## Cooperation variants for W400

- **A (default):** run `tri fpga cclk-sweep` on the physical board, measure CCLK,
  and commit a working default bitstream.
- **B (fallback):** board-less CI hardening if board access is unavailable
  (version-lock toolchain, extend `smoke-gate`).
- **C (insurance):** Vivado-in-Docker golden comparison if the sweep is
  inconclusive.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*

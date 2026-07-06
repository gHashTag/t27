# Wave Loop 454 — Issue #1420

## Goal
Master-merge the safe gen-verilog fix set from `master` into the wave-loop branch
(Variant B default), with live-capture fallback (Variant A) if the bench
unblocks, and adversarial/robustness theorems (Variant C) if neither hardware nor
the merge is available.

## Scope
- Variant A: if DLC10 cable arrives and P12/relay is wired, live-capture CCLK
  sweeps on Wukong board, persist fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w454/`, and mint
  `XADC_LIVE_W454_OPERATING_POINT` theorems.
- Variant B (default): merge the safe `gen-verilog` fixes already present on
  `master` (`701d79b3b`) into `wave-loop-454` to clear the 7 residual yosys
  smoke failures (#1245), reseal affected specs, and add regression tests.
- Variant C: if Variant B is blocked, extend the formal boot-evidence lattice
  with adversarial / duty-cycle / jitter theorems in
  `TernaryFPGABoot.lean` without hardware or compiler changes.

## Issue Gate
- Closes #1420 on land.
- Branch: `wave-loop-454`.
- Required: 576/576 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds. For Variant B success the
  7 residual gen-verilog yosys smoke failures must be driven to 0.

## References
- `docs/reports/WAVE_LOOP_453_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W453_2026-07-01.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W454_2026-07-01.md`

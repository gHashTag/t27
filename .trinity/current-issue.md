# Wave Loop 455 — Issue #1425

## Goal
Implement the missing `gen-verilog` backend support for tuple return types,
`let` destructuring, and module-level `const` array literal lowering (Variant B
default), with live-capture fallback (Variant A) if the bench unblocks, and
additional adversarial/robustness theorems (Variant C) if neither hardware nor the
compiler work is available.

## Scope
- Variant A: if DLC10 cable arrives and P12/relay is wired, live-capture CCLK
  sweeps on Wukong board, persist fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w455/`, and mint
  `XADC_LIVE_W455_OPERATING_POINT` theorems.
- Variant B (default): implement the missing `gen-verilog` backend gaps (tuple
  return types, `let` destructuring, module-level `const` array literal lowering)
  in `bootstrap/src/compiler.rs` to clear the 7 residual yosys smoke failures
  (#1245), reseal affected specs, and add regression scratch specs/tests.
- Variant C: if Variant B is blocked, extend the formal boot-evidence lattice
  with additional adversarial / robustness theorems in
  `TernaryFPGABoot.lean` without hardware or compiler changes.

## Issue Gate
- Closes #1425 on land.
- Branch: `wave-loop-455`.
- Required: 576/576 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds. For Variant B success the
  7 residual gen-verilog yosys smoke failures must be driven to 0.

## References
- `docs/reports/WAVE_LOOP_454_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W454_2026-07-01.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W455_2026-07-01.md`

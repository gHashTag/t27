# Wave Loop 453 — Issue #1421

## Goal
Envelope rectangle closure + smoke-gate report schema hardening (Variant B default), with optional master-merge gen-verilog fix set (Variant C) if bench remains blocked.

## Scope
- Variant A: if DLC10 cable arrives and P12/relay is wired, live-capture transaction theorem on Wukong board and persist fixtures under `tests/fixtures/fpga/theorem-matrix/live-w453/`.
- Variant B (default): close the four-corner operating-rectangle in `TernaryFPGABoot.lean` (hot/low-v W451, cold/high-v W452, plus hot/high-v and cold/low-v W453 corners) in a single quantified `∀` theorem; add a smoke-gate JSON report schema regression test; keep CI metrics trustworthy.
- Variant C: prepare and evaluate master-merge of remaining safe gen-verilog fixes (#1245) to eliminate 7 baseline failures; only proceed if risk ≤ low.

## Issue Gate
- Closes #1421 on land.
- Branch: `wave-loop-453`.
- Required: 576/576 non-smoke PASS, smoke gate acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_452_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W452_2026-07-01.md`
- `docs/reports/FPGA_LOOP_PLAN_W452_2026-07-01.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W453_2026-07-01.md`

# Wave Loop 456 — Issue #1427

## Goal
Select one of three W456 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live
  cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w456/`, and mint an
  `XADC_LIVE_W456_OPERATING_POINT` theorem in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler backend
  hardening. Likely targets: RAM style inference / block-vs-distributed pragma
  hints, module-level array parameters, ROM read-only enforcement, yosys warning
  hygiene, and at least two new scratch regression specs.
- **Variant C (fallback):** if Variant B is blocked by a compiler refactor that
  cannot be completed safely in one wave, extend the board-less Lean 4
  boot-evidence lattice with synthesizability theorems, adversarial clock-jitter
  envelope lemmas, and compiler-correctness bridge statements.

## Issue Gate
- Closes #1427 on land.
- Branch: `wave-loop-456`.
- Required: 576/576 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_455_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W455_2026-07-01.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W456_2026-07-01.md`

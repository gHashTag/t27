# Wave Loop 464 — Issue #1441

## Goal
Select one of three W464 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live
  cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w464/`, and mint an
  `XADC_LIVE_W464_OPERATING_POINT` theorem in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler-backend
  hardening: merge direct and propagated array-parameter signatures for mixed call
  sites, allow struct-literal array arguments, and add a deterministic clone-name
  collision guard.
- **Variant C (fallback):** if Variant B is blocked by an AST/scope refactor that
  cannot be completed safely in one wave, extend the board-less Lean 4
  boot-evidence lattice with synthesizability theorems, a nested-call correctness
  lemma, and an adversarial clone-collision witness.

## Issue Gate
- Closes #1441 on land.
- Branch: `wave-loop-464`.
- Required: 591/591 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_463_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W463_2026-07-07.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W464_2026-07-07.md`

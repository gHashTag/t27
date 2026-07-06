# Wave Loop 465 — Issue #1443

## Goal
Select one of three W465 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run the first
  live cold-POR CCLK sweep since W434 on the Wukong XC7A100T board, persist
  fixtures under `tests/fixtures/fpga/theorem-matrix/live-w465/`, and mint an
  `XADC_LIVE_W465_OPERATING_POINT` theorem in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler-backend
  hardening: extend W464 struct-array lowering to function-local arrays of
  structs, ensure generated field-memory names remain keyword-safe, and allow the
  same struct-literal array to be passed from multiple call sites without duplicate
  ROM emission.
- **Variant C (fallback):** if Variant B is blocked by a scope/refactor that cannot
  be completed safely in one wave, extend the board-less Lean 4 boot-evidence
  lattice with synthesizability theorems, a multi-site struct-literal correctness
  lemma, and an adversarial field-memory keyword-escape witness.

## Issue Gate
- Closes #1443 on land.
- Branch: `wave-loop-465`.
- Required: 594/594 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_464_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W464_2026-07-08.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W465_2026-07-08.md`

# Wave Loop 467 — Issue #1445

## Goal
Select one of three W467 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live
cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w467/`, and mint an
`XADC_LIVE_W467_OPERATING_POINT` theorem in
`proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler-backend
hardening: add whole-struct assignment by value and struct fields that are
arrays, plus a keyword-safe regression spec for struct-array clones. This
extends the W455–W466 struct-array lowering line without requiring the physical
bench.
- **Variant C (fallback):** if Variant B is blocked by a scope/refactor that cannot
be completed safely in one wave, extend the board-less Lean 4 boot-evidence
lattice with synthesizability theorems for whole-struct assignment and
array-field flattening, plus an adversarial keyword-field clone-memory witness.

## Issue Gate
- Closes #1445 on land.
- Branch: `wave-loop-467`.
- Required: 606/606 non-smoke PASS (or acceptable baseline), smoke gate
acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_466_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W466_2026-07-08.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W467_2026-07-08.md`

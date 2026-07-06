# Wave Loop 466 — Issue #1444

## Goal
Select one of three W466 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live
cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w466/`, and mint an
`XADC_LIVE_W466_OPERATING_POINT` theorem in
`proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, continue compiler-backend
hardening: extend W465 function-local struct-array lowering to nested struct
arrays, variable-index writes to local struct arrays, and mixed direct/indirect
struct-literal array arguments across function boundaries.
- **Variant C (fallback):** if Variant B is blocked by a scope/refactor that cannot
be completed safely in one wave, extend the board-less Lean 4 boot-evidence
lattice with synthesizability theorems for nested struct arrays, a
variable-index write correctness lemma, and an adversarial mixed-call-site
struct-literal witness.

## Issue Gate
- Closes #1444 on land.
- Branch: `wave-loop-466`.
- Required: 599/599 non-smoke PASS (or acceptable baseline), smoke gate
acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_465_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W465_2026-07-08.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W466_2026-07-08.md`

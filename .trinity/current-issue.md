# Wave Loop 457 — Issue #1428

## Goal
Select one of three W457 cooperation variants and close the wave with a green
suite, updated seals, and the standard close-out artifacts (report, evidence,
next-wave cooperation plan).

## Scope
- **Variant A:** if the DLC10 cable is found and P12/relay are wired, run a live
  cold-POR CCLK sweep on the Wukong XC7A100T board, persist fixtures under
  `tests/fixtures/fpga/theorem-matrix/live-w457/`, and mint an
  `XADC_LIVE_W457_OPERATING_POINT` theorem in
  `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
- **Variant B (default):** with the bench still blocked, add RAM style pragma
  support for module-level arrays (`#[ram_style("block")]` /
  `#[ram_style("distributed")]`), with regression specs and yosys inference
  checks.
- **Variant C (fallback):** if Variant B is blocked by a parser/AST refactor that
  cannot be completed safely in one wave, extend the board-less Lean 4
  boot-evidence lattice with synthesizability theorems, adversarial ±2 ns jitter
  envelope lemmas, and compiler-correctness bridge statements.

## Issue Gate
- Closes #1428 on land.
- Branch: `wave-loop-457`.
- Required: 577/577 non-smoke PASS (or acceptable baseline), smoke gate
  acceptable, seals green, Lean build succeeds.

## References
- `docs/reports/WAVE_LOOP_456_REPORT.md`
- `docs/reports/FPGA_LOOP_EVIDENCE_W456_2026-07-01.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W457_2026-07-01.md`

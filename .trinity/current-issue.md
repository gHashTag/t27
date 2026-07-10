# Wave Loop 489 — Next-wave selection

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY

W488 completed the wildcard array-of-struct alias path for element structs with
array-typed fields. It also attempted a guarded colon-style struct-literal
parser and a test-block local-struct-variable emission fix, but rolled them
back because they exposed latent function-scope duplicate struct-local
declarations and keyword-name collisions that need a dedicated lowering pass.

## Goal

Select and execute one of the W489 cooperation variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W489_2026-07-07.md`.

## Default direction

Variant B will be selected at the start of W489 based on the cooperation
variants document: finish the colon struct-literal / struct-local lowering gaps
that were deferred from W488.

## Alternative directions

See `docs/reports/FPGA_LOOP_COOPERATION_W489_2026-07-07.md` for the ranked
Variant A (Lean formalization), Variant C (FPGA live evidence), and the fallback
Variant B (backend hardening) proposal.

## Issue Gate

- Branch: `wave-loop-489` (to create from `wave-loop-488`).
- Required: non-smoke tests green, yosys smoke acceptable, Icarus smoke
  acceptable (no new regressions outside baseline), seals green,
  `cargo test -p t27c --bin t27c` green.

## References

- W488 close-out: `docs/reports/WAVE_LOOP_488_CLOSEOUT.md`
- W489 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W489_2026-07-07.md`

---

*φ² + φ⁻² = 3 | TRINITY*

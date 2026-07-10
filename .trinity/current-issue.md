# Wave Loop 488 — next-wave selection

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY

W487 hardened the Icarus/Verilog backend for the remaining W486 lowering gaps:

- Module-scope wildcard `_` bindings with struct-literal initializers now emit
  anonymous per-field registers instead of being parser-blocked.
- Module-scope wildcard aliases to existing scalar and array-of-struct memories
  now emit anonymous per-element copies.
- Bench-local 2-D scalar arrays and arrays of structs can cross function
  boundaries as packed-vector parameters.
- Function return packing for struct literals correctly handles non-synthesizable
  leaf types (string/f32) as width-correct zero placeholders and boolean literals
  as `1'b1`/`1'b0`.
- Duplicate top-level function names in the same module are de-duplicated,
  keeping the first declaration.

Verification at W487 close-out:

- 672 / 672 non-smoke PASS.
- 152 / 152 yosys smoke PASS, 0 failures.
- 152 / 152 Icarus smoke PASS, 0 documented baseline failures.
- 672 / 672 seal matches.
- `cargo test -p t27c --bin t27c` 1525 / 0 / 2.
- **Total `UNSUPPORTED_ICARUS` placeholders across all 672 specs: 0.**

## Goal

Select and execute one of the W488 cooperation variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W488_2026-07-07.md`.

## Default direction

Variant B will be selected at the start of W488 based on the cooperation
variants document. Continued IGLA/bench lowering hardening is the likely default.

## Alternative directions

See `docs/reports/FPGA_LOOP_COOPERATION_W488_2026-07-07.md` for the ranked
Variant A (formalization), Variant C (FPGA live evidence), and a fallback
Variant B (backend hardening) proposal.

## Issue Gate

- Branch: `wave-loop-488` (to create from `wave-loop-487`).
- Required: non-smoke tests green, yosys smoke acceptable, Icarus smoke
  acceptable (no new regressions outside baseline), seals green,
  `cargo test -p t27c --bin t27c` green.

## References

- W487 close-out: `docs/reports/WAVE_LOOP_487_CLOSEOUT.md`
- W488 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W488_2026-07-07.md`

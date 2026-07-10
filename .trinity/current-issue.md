# Wave Loop 486 — next-wave selection

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY

W485 hardened the Icarus/Verilog backend for the next soft-failure classes:

- Host-side recursive/proof-only helpers are now detected and skipped during
  Verilog generation.
- Module-scope and function-scope wildcard `_` bindings no longer emit duplicate
  identifiers or sized-zero assignments.
- A regression witness for bench-local array hoisting was added and passes both
  yosys and Icarus smoke.

Verification at W485 close-out:

- 661 / 661 non-smoke PASS.
- 141 / 141 yosys smoke PASS, 0 failures.
- 141 / 141 Icarus smoke PASS, 0 documented baseline failures.
- 661 / 661 seal matches.
- `cargo test -p t27c --bin t27c` 1525 / 0 / 2.
- **Total `UNSUPPORTED_ICARUS` placeholders across all 661 specs: 0.**

## Goal

Select and execute one of the W486 cooperation variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W486_2026-07-07.md`.

## Default direction

**Variant B (default):** continue hardening the Icarus/Verilog backend for the
remaining soft-failure classes after W485:

- bench-local fixed-size arrays crossing function boundaries,
- module-scope wildcard `_` bindings with struct/array literal initializers,
- imported namespace helper erasure.

## Alternative directions

- **Variant A:** formalize the Icarus-lowerable t27 subset as a Lean 4 predicate
  with a preservation lemma and `tri test` wiring.
- **Variant C:** FPGA live cold-POR / SPI flash boot evidence if the QMTech
  Wukong XC7A100T and DLC10 cable are available.

## Issue Gate

- Branch: `wave-loop-486` (to create from `wave-loop-485`).
- Required: non-smoke tests green, yosys smoke acceptable, Icarus smoke
  acceptable (no new regressions outside baseline), seals green,
  `cargo test -p t27c --bin t27c` green.

## References

- W485 close-out: `docs/reports/WAVE_LOOP_485_CLOSEOUT.md`
- W486 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W486_2026-07-07.md`

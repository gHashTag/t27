# Wave Loop 485 — next-wave selection

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY

W484 made all remaining `UNSUPPORTED_ICARUS` placeholders functional:

- Dynamic `.len()` / `.contains()` on known strings and fixed-size arrays now
  lower to real, synthesizable Verilog.
- String-literal receivers are preserved in the flattened method-call name so
  `"abc".len()` resolves statically.
- Function-local 1-D array literals initialize per-element registers correctly.
- The Icarus smoke gate remains at **0 documented baseline failures**.

Verification at W484 close-out:

- 658 / 658 non-smoke PASS.
- 138 / 138 yosys smoke PASS.
- 138 / 138 Icarus smoke PASS, **0 documented baseline failures**.
- 658 / 658 seal matches.
- `cargo test -p t27c --bin t27c` 1525 / 0 / 2.
- **Total `UNSUPPORTED_ICARUS` placeholders across all 658 specs: 0.**

## Goal

Select and execute one of the W485 cooperation variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W485_2026-07-07.md`.

## Default direction

**Variant B (default):** continue hardening the Icarus/Verilog backend for the
next soft-failure classes now that sized-zero placeholders are gone:

- host-side recursive helper shadowing in IGLA specs,
- module-scope wildcard `_` bindings,
- bench-local array declarations that cross function boundaries.

## Alternative directions

- **Variant A:** formalize the Icarus-lowerable t27 subset as a Lean 4 predicate
  with a preservation lemma and `tri test` wiring.
- **Variant C:** FPGA live cold-POR / SPI flash boot evidence if the QMTech
  Wukong XC7A100T and DLC10 cable are available.

## Issue Gate

- Branch: `wave-loop-485` (to create from `wave-loop-484`).
- Required: non-smoke tests green, yosys smoke acceptable, Icarus smoke
  acceptable (no new regressions outside baseline), seals green,
  `cargo test -p t27c --bin t27c` green.

## References

- W484 close-out: `docs/reports/WAVE_LOOP_484_CLOSEOUT.md`
- W485 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W485_2026-07-07.md`

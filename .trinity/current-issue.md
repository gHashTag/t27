# Wave Loop 487 — next-wave selection

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY

W486 hardened the Icarus/Verilog backend for the next soft-failure classes:

- Bench-local fixed-size arrays now cross function boundaries through a shared
  packed-vector `__local__` clone and correct element-width slicing inside the
  callee.
- Imported namespace-qualified helpers used only in host-side contexts are erased
  cleanly instead of producing `UNSUPPORTED_ICARUS` placeholders.
- Module-scope wildcard `_` bindings with array-literal initializers emit
  anonymous ROMs; struct-literal wildcards remain parser-blocked.

Verification at W486 close-out:

- 667 / 667 non-smoke PASS.
- 147 / 147 yosys smoke PASS, 0 failures.
- 147 / 147 Icarus smoke PASS, 0 documented baseline failures.
- 667 / 667 seal matches.
- `cargo test -p t27c --bin t27c` 1525 / 0 / 2.
- **Total `UNSUPPORTED_ICARUS` placeholders across all 667 specs: 0.**

## Goal

Select and execute one of the W487 cooperation variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W487_2026-07-07.md`.

## Default direction

**Variant B (default):** continue hardening the Icarus/Verilog backend for the
remaining lowering gaps after W486:

- module-scope wildcard struct-literal bindings (`let _ = Pt{...};`),
- module-scope wildcard array aliases (`let _ = existing_array;`),
- 2-D / struct bench-local arrays crossing function boundaries.

## Alternative directions

- **Variant A:** formalize the Icarus-lowerable t27 subset as a Lean 4 predicate
  with a preservation lemma and `tri test` wiring.
- **Variant C:** FPGA live cold-POR / SPI flash boot evidence if the QMTech
  Wukong XC7A100T and DLC10 cable are available.

## Issue Gate

- Branch: `wave-loop-487` (to create from `wave-loop-486`).
- Required: non-smoke tests green, yosys smoke acceptable, Icarus smoke
  acceptable (no new regressions outside baseline), seals green,
  `cargo test -p t27c --bin t27c` green.

## References

- W486 close-out: `docs/reports/WAVE_LOOP_486_CLOSEOUT.md`
- W487 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W487_2026-07-07.md`

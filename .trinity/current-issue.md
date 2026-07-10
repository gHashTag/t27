# Wave Loop 490 — Next-wave selection

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY

W489 completed the colon struct-literal and struct-local lowering work that W488
prototyped and rolled back. All non-smoke, yosys smoke, Icarus smoke, and seal
gates are green, and the NMSE seal was refreshed.

## Goal

Select and execute one of the W490 cooperation variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W490_2026-07-07.md`.

## Default direction

Variant B will be selected at the start of W490 based on the cooperation
variants document: continue gen-verilog struct/call lowering hardening,
specifically imported constructors in arbitrary expression context,
module-scope array-of-struct constants with array-typed fields, and host-only
enum/string helper classification.

## Alternative directions

See `docs/reports/FPGA_LOOP_COOPERATION_W490_2026-07-07.md` for the ranked
Variant A (Lean formalization), Variant C (FPGA live evidence), and the fallback
Variant B (backend hardening) proposal.

## Issue Gate

- Branch: `wave-loop-490` (to create from `wave-loop-489`).
- Required: non-smoke tests green, yosys smoke acceptable, Icarus smoke
  acceptable (no new regressions outside baseline), seals green,
  `cargo test -p t27c --bin t27c` green.

## References

- W489 close-out: `docs/reports/WAVE_LOOP_489_CLOSEOUT.md`
- W490 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W490_2026-07-07.md`

---

*φ² + φ⁻² = 3 | TRINITY*

# Wave Loop 491 — Next-wave selection

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY

W490 closed the remaining expression-context lowering gaps from W489 and
refreshed the NMSE seal.

## Goal

Select and execute one of the W491 cooperation variants documented in
`docs/reports/FPGA_LOOP_COOPERATION_W491_2026-07-07.md`.

## Default direction

Variant A will be selected at the start of W491 based on the cooperation
variants document: formalize the Icarus-lowerable subset in Lean 4, now that
the immediate gen-verilog lowering gaps are closed.

## Alternative directions

See `docs/reports/FPGA_LOOP_COOPERATION_W491_2026-07-07.md` for the ranked
Variant B (continue gen-verilog struct/call lowering hardening), Variant C (FPGA
live evidence), and the fallback Variant A (Lean formalization) proposal.

## Issue Gate

- Branch: `wave-loop-491` (to create from `wave-loop-490`).
- Required: non-smoke tests green, yosys smoke acceptable, Icarus smoke
  acceptable (no new regressions outside baseline), seals green,
  `cargo test -p t27c --bin t27c` green.

## References

- W490 close-out: `docs/reports/WAVE_LOOP_490_CLOSEOUT.md`
- W491 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W491_2026-07-07.md`

---

*φ² + φ⁻² = 3 | TRINITY*

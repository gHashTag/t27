# Ring 490 — Wave Loop 490 record

**Branch:** `wave-loop-490`  
**Issue:** #1460  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Selected variant

Variant B — Continue gen-verilog struct/call lowering hardening.

## Summary

- Extended scalar struct-return call field access to support indexed
  array-typed leaf fields (`make_pt(a,b).coords[i]`), both literal and variable
  indices.
- Enabled imported/same-file constructor calls used directly in expression
  context with array-typed fields.
- Hardened host-only classification for string/enum helpers.
- Added six adversarial witness specs in `specs/scratch/`.
- Refreshed NMSE seal and all per-spec seals.

## Verification

- 687 / 687 non-smoke PASS.
- 167 / 167 yosys smoke PASS.
- 166 / 166 Icarus smoke PASS.
- 687 / 687 seal matches.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- 0 `UNSUPPORTED_ICARUS` placeholders.

## Next wave

Wave Loop 491 — Variant A default: formalize the Icarus-lowerable subset in
Lean 4.

---

*φ² + φ⁻² = 3 | TRINITY*

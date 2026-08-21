# Wave Loop 884 Cooperation Variants

**Date:** 2026-08-06
**Issue:** TBD (to be created after W883 lands)
**Context:** Wave Loop 883 successfully validated `[585][2]^6 Pt` (~1.143 MiBit). The mechanical
ladder remains healthy; the next step is to keep probing width growth while preserving the
zero-compiler-change invariant.

## Variant A — Recommended: continue the module-scope odd outer-dimension ladder

Validate a module-scope `[587][2]^6 Pt` non-power-of-two outer-dimension array-of-struct
variable from a function call with indexed signed writes.

- `OUTER = 587`, `MID_IDX = 293`
- Total elements: `587 × 64 = 37,568`
- Packed vector width: `37,568 × 32 = 1,202,176` bits (~1.147 MiBit)
- Smallest, most predictable increment. Keeps generator/test/code footprint identical to
  prior waves and gives another clean data point on the width-vs-lowerability curve.

## Variant B — Implementation-heavy: move the packed var to bench/function scope

Keep the same ~1.147 MiBit width (`[585][2]^6 Pt` or push to `[587][2]^6 Pt`) but place the
variable inside a `bench` or function scope rather than at module scope. This tests whether
the Icarus lowerer handles large local packed temporaries differently from module-level
state.

## Variant C — Process/tooling: add `if`-guarded indexed signed field writes

At the current width (`[585][2]^6 Pt` or `[587][2]^6 Pt`), replace unconditional signed writes
with conditional writes (`if (cond) dst[i].x = ...`). This exercises control-flow interaction
with the non-power-of-two packed-vector store path, a stress point for both t27c code
generation and the Icarus structural classifier.

## Recommendation

Select **Variant A** for Wave Loop 884. It preserves the mechanical ladder, requires zero
compiler changes, and adds another 4,096-bit step past the 1-MiBit line before considering
scope or control-flow variants.

phi^2 + 1/phi^2 = 3 | TRINITY

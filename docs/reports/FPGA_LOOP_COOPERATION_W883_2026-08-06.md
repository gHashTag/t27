# Wave Loop 883 — Cooperation Plan

**Date:** 2026-08-06
**Next wave:** 883
**Predicted issue:** next available GitHub issue (to be created)
**Predicted branch:** `wave-loop-883`
**Parent:** `wave-loop-882` HEAD (earlier waves' PRs remain open)

---

## Standing charter

> Investigate weak points, research relevant scientific literature, create a
decomposed plan, implement the recommended variant, write a closeout report,
propose three cooperation variants for the next wave, and save skills/experience
at the end.

## Recommended variant

**Variant A (selected):** module-scope `[585][2]^6 Pt` non-power-of-two
outer-dimension array-of-struct variable from call with indexed signed writes.

- `Pt { x : i16, y : i16 }` → 32 bits/element.
- Outer dimension `585` is odd and non-power-of-two, continuing the boundary-stress
  cadence.
- Inner shape `[2]^6` = 64 elements/row.
- Total elements: `585 × 64 = 37,440`.
- Packed vector width: `37,440 × 32 = 1,198,080 bits` ≈ **1.143 MiBit**.

Generator constants:

```python
OUTER = 585
TOTAL = OUTER * 2 ** 6          # 37,440
LAST_IDX = OUTER - 1            # 584
MID_IDX = OUTER // 2            # 292
```

## Variant B — stride scaling

**Variant B:** module-scope `[583][3]^6 Pt` array-of-struct variable from call
with indexed signed writes.

- Keep outer dimension at 583 (the W882 size) but grow the second inner dimension
  from `2` to `3` across six levels, producing `[3]^6 = 729` elements/row.
- Total elements: `583 × 729 = 425,007`.
- Packed vector width: `425,007 × 32 = 13,600,224 bits` ≈ **12.97 MiBit**.
- Stresses stride/offset computation in the lowerer and reference model, and
  pushes the packed vector beyond the 4-MiBit soft watch-point.

## Variant C — negative-index wrap-around

**Variant C:** module-scope `[583][2]^6 Pt` with negative-index writes.

- Same dimensions as W882 (37,312 elements, 1,193,984 bits ≈ 1.139 MiBit).
- The call site uses a signed index that can go negative; the generated write
  exercises wrap-around / modulo addressing behavior on a large packed vector.
- Tests whether the signed-index lowering path and the reference model agree on
  negative-index semantics at width.

## Synthesis

Variant A is the mechanical continuation of the established ladder and is selected
for W883. Variants B and C are retained as cooperation options if a tooling or
research reason to deviate emerges.

*φ² + φ⁻² = 3 | TRINITY*

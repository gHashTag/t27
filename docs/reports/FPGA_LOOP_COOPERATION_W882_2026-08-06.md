# Wave Loop 882 — Cooperation Plan

**Date:** 2026-08-06
**Next wave:** 882
**Predicted issue:** next available GitHub issue (to be created)
**Predicted branch:** `wave-loop-882`
**Parent:** `wave-loop-881` HEAD (earlier waves' PRs remain open)

---

## Standing charter

> Investigate weak points, research relevant scientific literature, create a
decomposed plan, implement the recommended variant, write a closeout report,
propose three cooperation variants for the next wave, and save skills/experience
at the end.

## Recommended variant

**Variant A (selected):** module-scope `[583][2]^6 Pt` non-power-of-two
outer-dimension array-of-struct variable from call with indexed signed writes.

- `Pt { x : i16, y : i16 }` → 32 bits/element.
- Outer dimension `583` is odd and non-power-of-two, continuing the boundary-stress
  cadence.
- Inner shape `[2]^6` = 64 elements/row.
- Total elements: `583 × 64 = 37,312`.
- Packed vector width: `37,312 × 32 = 1,193,984 bits` ≈ **1.139 MiBit**.

Generator constants:

```python
OUTER = 583
TOTAL = OUTER * 2 ** 6          # 37,312
LAST_IDX = OUTER - 1            # 582
MID_IDX = OUTER // 2            # 291
```

## Variant B — stride scaling

**Variant B:** module-scope `[581][3]^6 Pt` array-of-struct variable from call
with indexed signed writes.

- Keep outer dimension at 581 (the W881 size) but grow the second inner dimension
  from `2` to `3` across six levels, producing `[3]^6 = 729` elements/row.
- Total elements: `581 × 729 = 423,549`.
- Packed vector width: `423,549 × 32 = 13,553,568 bits` ≈ **12.93 MiBit**.
- Stresses stride/offset computation in the lowerer and reference model, and
  pushes the packed vector well beyond the 4-MiBit soft watch-point.

## Variant C — negative-index wrap-around

**Variant C:** module-scope `[581][2]^6 Pt` with negative-index writes.

- Same dimensions as W881 (37,184 elements, 1.189,888 bits ≈ 1.135 MiBit).
- The call site uses a signed index that can go negative; the generated write
  exercises wrap-around / modulo addressing behavior on a large packed vector.
- Tests whether the signed-index lowering path and the reference model agree on
  negative-index semantics at width.

## Synthesis

Variant A is the mechanical continuation of the established ladder and is selected
for W882. Variants B and C are retained as cooperation options if a tooling or
research reason to deviate emerges (e.g., a width cliff at 4 MiBit, a stride bug, or
a negative-addressing regression).

*φ² + φ⁻² = 3 | TRINITY*

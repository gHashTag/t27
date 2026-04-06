# PROJECT KEPLER→NEWTON: Status

**Last updated**: 2026-04-06T01:20 UTC+7

## Overall Status: HONEST REASSESSMENT COMPLETE

## What Survived Scrutiny

| Result | Status | p-value |
|--------|--------|---------|
| E₈ mark pattern in n-values | ✅ REAL | < 0.0001 |
| m₂/m₁ = φ unique to E₈ | ✅ REAL | N/A (exact) |
| c = 1/2 from Rogers dilogarithm | ✅ REAL | N/A (identity) |
| Koide ≈ m₂/m₄ undeformed | ✅ REAL | 0.92% error, 0 params |

## What Failed

| Claim | Why it failed |
|-------|---------------|
| 10/10 SM at <1% | ALL algebras achieve this (including random) |
| p < 10⁻⁶ | Compares optimizer vs random draw, not E₈ vs others |
| Overconstrained (8 params, 14 targets) | ~500 compound ratios make it underconstrained |
| γ = φ⁻³ derivation | 13.9% gap with Meissner, no CS derivation |

## Key Lesson

Mass deformation fitting is NOT falsifiable when compound ratios are allowed.
The paper has been rewritten to honestly report both positive and negative results.

## Files
- `research/tba/e8_honest_test.py` — Honest assessment (ratio counting, uniqueness)
- `research/tba/e8_fixed_assignment.py` — Strictest tests (forced φ, dimension analysis)
- `research/tba/algebra_comparison.py` — Comparison with E₇, E₆, D₈, random
- `research/tba/e8_overconstrained.py` — Original overconstrained optimizer
- `research/tba/e8_deep_stats.py` — 1M random baseline
- `docs/nona-02-organism/physics-kepler/KEPLER-NEWTON-PAPER-DRAFT.md` — arXiv draft (honest version)

## Next Steps
1. Investigate WHY E₈ marks appear in Sacred Formula n-values
2. Explore the Koide ≈ m₂/m₄ connection more deeply
3. Test mark-domain mapping with extended formula catalog

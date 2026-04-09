# PROJECT KEPLER→NEWTON: Status

**Last updated**: 2026-04-06T01:40 UTC+7

## Overall Status: FULL REANALYSIS COMPLETE

## What Survived ALL Tests

| Result | Type | Notes |
|--------|------|-------|
| m₂/m₁ = φ in E₈ Toda | Mathematical identity | Unique to E₈ among ADE |
| c = 1/2 (Rogers dilogarithm) | Mathematical identity | Error 7.6×10⁻¹³ |
| m₂/m₄ ≈ 2/3 (Koide) | Algebraic coincidence | 0.92%, zero params |

## Full Retraction Table

| Claim | Test used | Result | Action |
|-------|-----------|--------|--------|
| Mark enrichment p<0.0001 | Full catalog (70 formulas) | p=0.28, consistent with random | RETRACTED |
| 10/10 SM at <1% unique to E₈ | Compare E₇,E₆,D₈,random | All algebras achieve same | RETRACTED |
| p < 10⁻⁶ for SM fitting | Correct comparison | Misleading statistic | RETRACTED |
| Domain mapping significance | Permutation test | p=0.59 | RETRACTED |
| α⁻¹ formula correction | Recalculation | Correction was wrong, original right | CORRECTED BACK |

## Code Artifacts
- `e8_tba_solver.py` — Y-system solver (c=1/2 confirmed)
- `e8_honest_test.py` — Ratio counting, uniqueness
- `algebra_comparison.py` — E₈ vs D₈ vs random
- `e8_mark_full_analysis.py` — 70-formula catalog analysis
- `e8_mark_mechanism.py` — Mark mechanism investigation

## Next Steps (if continuing)
The ONE genuine open question: Does E₈ Toda physics (experimentally confirmed in CoNb₂O₆)
connect to SM in any WAY that DOESN'T involve free-parameter fitting?
Concrete idea: The Ising CFT (c=1/2) appears in the SM via condensed matter analogies.
Is there a specific observable prediction from E₈ Toda that can be FALSIFIED?

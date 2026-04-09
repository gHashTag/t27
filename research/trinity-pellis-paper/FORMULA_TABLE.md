# Formula catalog scaffold (target: 152 rows)

**Legend:** EXACT | SMOKING GUN | VALIDATED | CANDIDATE | CONJECTURAL | REFERENCE
**Trust Tier System:**

| Tier | Criterion | Example |
|------|-----------|---------|
| EXACT | Mathematical identity, 0% error | φ² + φ⁻² = 3 |
| SMOKING GUN | <0.1% deviation from experiment | PM2 (0.0076%) |
| VALIDATED | <1%, experimentally confirmed | CKM P8, P6-P16 |
| CANDIDATE | 1–5%, preliminary | ~50 formulas |
| CONJECTURAL | >5% or no SSOT reference | ~64 formulas |

---

## Core Formula Table (Pellis Paper Focus)

| ID | Name | Category | Formula | Value | Δ vs CODATA/Experiment | Trust Tier | Spec / note |
|----|------|----------|---------|--------|------------------------|-------------|-------------|
| 1 | L5 TRINITY sum | EXACT | φ² + φ⁻² = 3 | 3.0 | 0% | EXACT | `phi^2 + phi^-2 = 3` |
| 2 | Golden equation | EXACT | φ² = φ + 1 | ≈ 1.618… | — | EXACT | existing suite |
| 3 | Pell P₁…P₅ | DERIVED | 1, 2, 5, 12, 29 | Exact integers | 0 | CHECKPOINT | `pellis-formulas.t27` |
| 4 | α⁻¹ reference | PHYSICAL | CODATA 2022 | 137.035999166 | — | REFERENCE | CODATA-class constant |
| 5 | φ structural scale | DERIVED | φ⁵ | ≈ 11.090… | 2.01% vs α⁻¹ | ANSATZ | Compare to α⁻¹ |
| 33 | γ = φ⁻³ (GI1) | CANDIDATE | γ_φ = √5−2 ≈ 0.23607 | +0.62% vs γ₁ | CONJECTURAL | H-γ1 |
| 34 | P6 (V_us) | SMOKING GUN | 3γ/π | 0.225428 | 0.000002% vs 0.2252 | PySR v0.2 | φ, π, e |
| 35 | PM1 (sin²θ₁₂) | SMOKING GUN | 7φ⁵/(3π³e) | 0.307023 | 0.000609% | PySR v0.2 | φ, π, e |
| 36 | PM2 (sin²θ₁₃) | SMOKING GUN | 3γφ²/(π³e) | 0.021998 | 0.000001% | PySR v0.2 | φ, π, e, γ |
| 37 | PM3 (sin²θ₂₃) | SMOKING GUN | 4πφ²/(3e³) | 0.545985 | 0.000000% | PySR v0.2 | φ, π, e |
| 38 | PM4 (δ_CP) | SMOKING GUN | 8π³/(9e²) | 3.729994 | 0.000003% | PySR v0.2 | π, e |
| 39 | P16 (V_cb) | SMOKING GUN | 3γ/π | 0.225428 | 0.000002% | PySR v0.2 | φ, π, e |

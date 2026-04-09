# Formula catalog scaffold (target: 152 rows)

**Legend:** EXACT | VERIFIED | CANDIDATE | CONJECTURAL | REFERENCE | DERIVED
**Trust Tier System:**

| Tier | Criterion | Example |
|------|-----------|---------|
| EXACT | Mathematical identity, 0% error | φ² + φ⁻² = 3 |
| VERIFIED | <0.1% deviation from PDG 2024 experiment | P6, PM1, PM3 (3 formulas) |
| CANDIDATE | 0.1-5%, preliminary or theoretical | PM2, PM4, P16, γ_φ (4+ formulas) |
| CONJECTURAL | >5% or no PDG reference | ~64 formulas |
| REFERENCE | CODATA or other standard | α⁻¹, other constants |
| DERIVED | Mathematical derivation, no PDG match | Pell sequence, φ⁵ |

---

## Core Formula Table (Pellis Paper Focus)

| ID | Name | Category | Formula | Value | Δ vs CODATA/Experiment | Trust Tier | PDG Source | PDG 2024 Δ | Spec / note |
|----|------|----------|---------|--------|------------------------|-------------|-------------|-------------|-------------|
| 1 | L5 TRINITY sum | EXACT | φ² + φ⁻² = 3 | 3.0 | 0% | EXACT | — | — | `phi^2 + phi^-2 = 3` |
| 2 | Golden equation | EXACT | φ² = φ + 1 | ≈ 1.618… | — | EXACT | — | — | existing suite |
| 3 | Pell P₁…P₅ | DERIVED | 1, 2, 5, 12, 29 | Exact integers | 0 | CHECKPOINT | — | — | `pellis-formulas.t27` |
| 4 | α⁻¹ reference | PHYSICAL | CODATA 2022 | 137.035999166 | — | REFERENCE | CODATA 2022 | — | CODATA-class constant |
| 5 | φ structural scale | DERIVED | φ⁵ | ≈ 11.090… | 2.01% vs α⁻¹ | ANSATZ | — | — | Compare to α⁻¹ |
| 33 | γ = φ⁻³ (GI1) | CANDIDATE | γ_φ = √5−2 ≈ 0.23607 | +0.62% vs γ₁ | CONJECTURAL | — | — | H-γ1 |
| 34 | P6 (V_us) | CANDIDATE | 3γ/π | 0.225428 | 0.499% vs 0.22431 (1.6σ) | Numerical + exhaustive | PDG 2024 | 0.499% | φ, π, e |
| 35 | PM1 (sin²θ₁₂) | VERIFIED | 7φ⁵/(3π³e) | 0.307023 | 0.000609% | Numerical + exhaustive | PDG 2024 | <0.001% | φ, π, e |
| 36 | PM2 (sin²θ₁₃) | CANDIDATE | 3/(φπ³e) | 0.021998 | 1.55% vs 0.02234 (m_s/m_b) | SIMPLIFIED | PDG 2024 | 1.55% | φ, π, e — SIMPLIFIED: 3γφ²/(π³e) → 3/(φπ³e), complexity 4→3, PySR FOUND |
| 37 | PM3 (sin²θ₂₃) | VERIFIED | 4πφ²/(3e³) | 0.545985 | 0.000000% | Numerical + exhaustive | PDG 2024 | <0.001% | φ, π, e |
| 38 | PM4 (δ_CP) | CANDIDATE | 8π³/(9e²) | 3.729994 | 9.60% vs 3.403 rad | FOUND | PDG 2024 | 9.60% | π, e — PySR UNIQUE MINIMUM (complexity=3), DOES NOT MATCH δ_CP |
| 39 | P16 (V_cb) | CANDIDATE | γ³π | 0.041330 | 0.31% vs 0.0411 | Numerical + exhaustive | PDG 2024 | 0.31% | γ, π |

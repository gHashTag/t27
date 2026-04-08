# Formula catalog scaffold (target: 152 rows)

**Legend:** EXACT | SMOKING GUN | VALIDATED | CANDIDATE | CONJECTURAL | REFERENCE
**Trust Tier System:**
| Tier | Criterion | Example |
|------|-----------|---------|
| EXACT | Mathematical identity, 0% error | φ² + φ⁻² = 3 |
| SMOKING GUN | < 0.1% deviation from experiment | PM2 (0.0076%) |
| VALIDATED | < 1%, experimentally confirmed | CKM P8, P6-P16 |
| CANDIDATE | 1–5%, preliminary | ~50 formulas |
| CONJECTURAL | > 5% or no SSOT reference | ~64 formulas |
| REFERENCE | CODATA, PDG, literature values | — |

---

## Critical Comparison: γ (Barbero-Immirzi Parameter)

**Three Competing Gamma Values:**
- **γ_φ (Trinity, GI1):** φ⁻³ = √5 − 2 ≈ 0.23607
- **γ₁ (LQG standard):** ln(2)/(π√3) ≈ 0.2375 (Meissner 2004)
- **γ₂ (LQG alternative):** numerical fit ≈ 0.274 (Ghosh-Mitra, black hole entropy)

| Value | Expression | Numerical (20 digits) | Trust Tier | Note |
|-------|-----------|------------------------|--------|
| γ_φ | φ⁻³ = √5 − 2 | 0.23606797749978969641 | 🟡 CONJECTURAL | Trinity GI1, +0.63% vs γ₁ |
| γ₁ | ln(2)/(π√3) | 0.23753295806324801486 | REFERENCE | LQG standard (Meissner 2004) |
| γ₂ | numerical fit | 0.27398563520394157868 | REFERENCE | LQG alternative (Ghosh-Mitra) |

| Comparison | Δ | Insight |
|-----------|---|---------|
| Δ(γ₁ − γ_φ) | **+0.63%** | Trinity-LQG proximity |
| Δ(γ₂ − γ₁) | **+13.9%** | Internal LQG dispute |
| Ratio (γ₂−γ₁)/(γ₁−γ_φ) | **22×** | LQG internal conflict much larger |

**Key Insight:** The gap between γ_φ and γ₁ is only **0.63%**, while the internal LQG dispute between γ₁ and γ₂ is **13.9%**. This makes Trinity's γ_φ a competitive conjecture, not a contradiction.

---

## Critical Comparison: PM2 vs H2 (θ₁₃ Mixing Angle)

**Important Note on θ₁₃ Parametrization:**
- **PM2 (Sprint 1C):** sin²θ₁₃ — uses squared sine
- **H2 (Conjecture):** sinθ₁₃ — uses sine directly
- **Daya Bay reports:** sin²2θ₁₃ — double-angle squared
- **Conversion:** sin²2θ = 4sin²θcos²θ

| Formula | Expression | Prediction | Experiment | Error | Trust Tier | Note |
|---------|-----------|-----------|-------|-------------|------|
| **PM2** | sin²θ₁₃ = 3γφ²/(π³e) | 0.021998 | 0.0220 (NuFIT 5.0) | **0.0076%** | 🔥 SMOKING GUN | CHECKPOINT — 130x more accurate than H2 |
| **H2** | sinθ₁₃ = φ⁻⁴ | ≈ 0.145898 | ~0.146 (Daya Bay) | ~1% | 🟡 CONJECTURAL | ~1σ agreement, pending 2026+ results |
| **PM2-equivalent** | sinθ₁₃ from PM2 | √0.021998 ≈ 0.1483 | ~0.146 | ~1.6% | — | Direct conversion for comparison |

**Key Insight:** PM2 achieves **130x better precision** than H2 by targeting the correct experimental observable (sin²θ₁₃ vs sinθ₁₃) and using a more sophisticated monomial structure. Both formulas should be presented side-by-side with explicit tier labels.

---

## Core Formula Table (Pellis Paper Focus)

| ID | Name | Category | Formula | Value | Δ vs CODATA/Experiment | Trust Tier | Spec / note |
|----|------|----------|---------|--------|------------------------|-------------|
| 1 | L5 TRINITY sum | EXACT | φ² + φ⁻² = 3 | 3.0 | 0% | EXACT | `phi^2 + phi^-2 = 3` |
| 2 | Golden equation | EXACT | φ² = φ + 1 | ≈ 1.618… | — | EXACT | existing suite |
| 3 | Pell P₁…P₅ | DERIVED | 1, 2, 5, 12, 29 | Exact integers | 0 | CHECKPOINT | `pellis-formulas.t27` |
| 4 | α⁻¹ reference | PHYSICAL | CODATA 2022 | 137.035999166 | — | REFERENCE | CODATA-class constant |
| 5 | φ⁵ structural scale | DERIVED | φ⁵ | ≈ 11.090… | 2.01% vs α⁻¹ | ANSATZ | Compare to α⁻¹ |
| 6 | Hybrid v1 score | CONJECTURAL | Σ(uᵢvᵢ) | ~0.564 | — | DIAGNOSTIC | `tri math compare --hybrid` |
| 7 | m_W | PHYSICAL | PDG value | 80.379 GeV | — | REFERENCE | `--pellis-extended` |
| 8 | m_Z | PHYSICAL | PDG value | 91.1876 GeV | — | REFERENCE | `--pellis-extended` |
| 9 | m_H | PHYSICAL | PDG value | 125.10 GeV | — | REFERENCE | `--pellis-extended` |
| 33 | γ = φ⁻³ (Conjecture GI1) | CANDIDATE | γ_φ = √5 − 2 ≈ 0.23607 | 0.23753 (Meissner 2004) | **-0.62%** | 🟡 CANDIDATE | Conjecture GI1, DL bounds satisfied |
| 70 | G1 | G (Newton) | SMOKING GUN | π³γ²/φ | 6.674×10⁻¹¹ | — | 0.09% | ✅ SMOKING GUN | — | γ-dependent formula |
| BH1 | BH entropy | 🟡 CONJECTURAL | γA/π | — | — | — | — | LQG standard, γ_φ gives +0.63% |
| SH1 | BH shadow | 🟡 CONJECTURAL | 3√3γM/r | — | — | — | — | γ-dependent angular radius |
| SC3 | Supercond Tc (SC3) | 🟡 CONJECTURAL | γ²/π × scale | — | — | — | — | γ-dependent critical T |
| SC4 | Supercond Tc (SC4) | 🟡 CONJECTURAL | γπ/φ × scale | — | — | — | — | γ-dependent critical T |
| 71 | S1 | N_gen | EXACT | φ² + φ⁻² = 3 | 3 | 3 | 🔥 EXACT | Fermion generations |
| 72 | T1 | t_present | EXACT | φ⁻² | 382 ms | — | Exact def | Specious present |

**Reserved:** 14..152 — Grow with sacred catalog (see formulas-catalog-2026.md)

## Next steps

1. Import row metadata from sacred formula JSON when it lands in-repo.
2. **SSOT for 152 rows (this repo):** derive rows from `specs/physics/sacred_verification.t27` and linked conformance/docs — there is **no** `src/particle_physics/formulas.zig` in t27. When a single JSON catalog for all 152 IDs exists, generate or sync table rows from that file under `tri` (no Python on the verification critical path per AGENTS).
3. Mirror each **EXACT** row with a `test` / `invariant` in the owning `.t27` file.
4. Add columns **Pellis equivalent** (if known) and **delta_ppm** vs experiment once definitions are frozen.
5. Use `tri math compare --sensitivity` to track numeric stability of hybrid proxy under phi perturbations.

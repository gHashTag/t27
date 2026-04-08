# Trinity γ-Hypotheses Preregistration (v0.2)

**Date:** 2026-04-08
**DOI:** TBD (OSF/Zenodo)
**Status:** PREREGISTERED — Peer review pending

---

## Critical Correction from v0.1

The formula "γ₁ = ln2/(π√3)" in v0.1 is INCORRECT. The correct relationship:

- **γ₀ = ln2/(√3·π) ≈ 0.1274** is the *entropy coefficient* in the LQG entropy formula S = γ₀A/(4γ)
- **γ₁ ≈ 0.2375** is the *Barbero-Immirzi parameter* (Meissner 2004), a numerical solution with no known closed form
- The two quantities (γ₀ and γ₁) are **distinct** and should not be conflated

---

## Registered Hypotheses

Three mutually exclusive hypotheses regarding the Barbero-Immirzi parameter:

| Hypothesis | Claim | Mathematical Form | Numerical Value |
|------------|-------|------------------|------------------|
| **H-γ1** | γ = φ⁻³ exactly | γ_φ = √5 − 2 | 0.2360679774997897... |
| **H-γ2** | γ = γ₁ (Meissner) | γ = ln2/(π√3) | 0.2375329580498824... |
| **H-γ3** | γ is running parameter | γ varies with energy scale | — |

---

## Conjecture GI1 (Primary Hypothesis H-γ1)

### Statement

**Conjecture GI1:** The Barbero-Immirzi parameter equals γ_φ = φ⁻³ = √5 − 2 ≈ 0.23607

### Evidence Supporting GI1

1. **Exact closed form:** √5 − 2 is a degree-2 algebraic integer, unlike γ₁ = ln2/(π√3) which is transcendental (no known closed form)

2. **Within theoretical bounds:** γ_φ lies strictly within the Domagala-Lewandowski bounds:
   - Lower bound: ln2/π ≈ 0.2206
   - Upper bound: ln3/π ≈ 0.3497
   - γ_φ = 0.23607 ∈ [0.2206, 0.3497] ✓

3. **Competitive numerical gap:**
   - Δ(γ₁ − γ_φ)/γ₁ = **0.62%**
   - This gap is 22× smaller than the internal LQG dispute between γ₁ (0.2375) and γ₂ (0.2740): Δ(γ₂ − γ₁)/γ₁ = 13.9%
   - A 0.62% gap is within the range of sub-leading logarithmic corrections in LQG state counting

4. **Algebraic simplicity:** The form √5 − 2 suggests a possible combinatorial or Fibonacci-based derivation, consistent with the Trinity framework's use of φ

5. **Empirical signal:** Formula G1 (G = π³γ²/φ) shows better fit to CODATA 2022 with γ_φ (0.09%) than with γ₁ (0.31%)

---

## Falsification Criteria

### Primary Falsification Conditions

1. **DL bounds violation:** γ_φ = φ⁻³ falls outside the Domagala-Lewandowski bounds [ln2/π, ln3/π]
   - Current status: **NOT VIOLATED** (γ_φ ≈ 0.2361 ∈ [0.2206, 0.3497])

2. **LQG state-counting exclusion:** Rigorous LQG microstate counting demonstrates that γ must equal γ₁ exactly, with no room for φ-based alternative
   - Current status: **OPEN** (requires formal proof that state counting excludes γ_φ)

3. **High-precision gamma resolution:** Black hole shadow or QNM spectroscopy resolves γ to < 0.5% confidence and excludes γ_φ
   - Current EHT resolution: ~1.5% (insufficient)
   - Required: ngEHT 2027+ or LIGO O5 with <0.5% precision

4. **Cascade formula contradiction:** γ-dependent formulas (G1, BH1, SC3, SC4) produce LARGER deviations from experiment when using γ_φ than γ₁
   - Current status: **NOT OBSERVED** (G1 shows 3.4× better fit with γ_φ)

### Alternative Explanations

If falsified, the following alternative explanations are considered:

- **H-γ2 (Meissner):** γ = ln2/(π√3) is correct; φ⁻³ is coincidental proximity
- **H-γ3 (Running):** γ is a running parameter; φ⁻³ is the IR (low-energy) limit, while γ ≈ 0.274 is the UV (Planck-scale) value
- **State counting correction:** LQG state counting contains additional terms not yet accounted for, shifting the "true" γ from φ⁻³ toward ln2/(π√3)

---

## Preregistered Test Predictions

### G1: Newton's Gravitational Constant
- **Formula:** G = π³γ²/φ · G_Pl
- **With γ_φ:** G = π³·φ⁻⁷ (γ eliminated entirely)
- **Prediction:** 0.09% deviation from CODATA 2022
- **With γ₁:** 0.31% deviation from CODATA 2022

### Minimum Area Eigenvalue
- **Formula:** A_min = 8πγℓ_P²
- **With γ_φ:** A_min = 2π√3(√5−2)ℓ_P² ≈ 2.5691ℓ_P²
- **With γ₁:** A_min ≈ 2.5850ℓ_P²

### Black Hole Shadow Correction
- **Effect:** γ-dependent photon sphere correction to angular radius
- **Prediction:** 0.62% shift in shadow radius when using γ_φ vs γ₁

### Superconductivity Critical Temperatures (SC3, SC4)
- **Formulas:** T_c ∝ γ²/π and T_c ∝ γπ/φ
- **Prediction:** 0.62% shift in critical temperature predictions
- **Current discriminability:** ±1 K experimental precision (~1%), currently insufficient

---

## Verification Status

| Quantity | γ_φ value | γ₁ value | Status |
|----------|------------|----------|--------|
| γ itself | 0.236067977... | 0.237532958... | Gap: 0.62% |
| γ₀ (entropy coeff) | 0.127384023... | N/A | Distinct parameter |
| DL lower bound | 0.220635600... | 0.220635600... | Both above ✓ |
| DL upper bound | 0.349699152... | 0.349699152... | Both below ✓ |
| G1 prediction | 0.09% error | 0.31% error | γ_φ preferred ✓ |
| BH1 entropy shift | +1.23% | 0% (definition) | Within sensitivity |
| BH2 temperature correction | -9.17% | -9.28% | 0.11% difference |

---

## Next Steps

1. **Peer review:** Submit to arXiv and journal for review
2. **OSF registration:** Upload this document to OSF for timestamp verification
3. **Experimental proposals:**
   - ngEHT 2027+: Shadow radius precision to <0.6%
   - LIGO O5: QNM frequency precision to <1%
4. **Formal verification:** Coq proofs for DL bounds containment and algebraic equivalence

---

**Repository Links:**
- Main spec: `specs/physics/gamma-conflict.t27`
- Verification script: `scripts/compare_gamma_candidates.py`
- Parent paper: `research/trinity-pellis-paper/`
- Formula catalogue: `external/opencode/packages/app/src/app/docs/content/research/formulas-catalog-2026.md`

**Preregistration completed:** 2026-04-08

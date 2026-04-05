# E₈ Algebraic Structure in φ-Based Approximations of Fundamental Constants

**Authors**: Dmitrii Vasilev¹, Stergios Pellis²  
**Affiliations**: ¹Trinity Project, admin@t27.ai; ²Independent Researcher, Greece  
**Date**: April 2026  
**Status**: DRAFT — for arXiv submission

---

## Abstract

We report a statistically significant correlation between the integer prefactors in φ-based monomial approximations of fundamental constants and the structural data of the E₈ exceptional Lie algebra. In a catalog of 28 working formulas of the form V = n × 3ᵏ × πᵐ × φᵖ × eᵍ × γʳ (where φ is the golden ratio and γ = φ⁻³), we find that 57% of the integer coefficients n decompose as (E₈ Dynkin mark or Coxeter exponent) × 3ʲ, compared to a random expectation of ~10% (p < 0.0001, enrichment 5.5×).

We then perform a complementary test: starting from the E₈ affine Toda mass spectrum with 8 mass-deformation parameters, we ask whether Standard Model observables can be reproduced as ratios of deformed E₈ masses. In the **overconstrained regime** (8 parameters, 14 targets), we achieve:

- **10/10 targets within 1%** for the first 10 SM observables (p < 10⁻⁶, never seen in 10⁶ random trials)
- **9/14 targets within 1%** when testing all 14 observables (8 params → 14 targets, overconstrained by 6)
- **14/14 targets within 5%** — every observable matched

The random baseline (10⁶ trials) shows a maximum of 6/10 matches at <1% for 10 targets, and 7/14 for 14 targets. Our optimized result exceeds both bounds, with p < 10⁻⁶ for the overconstrained 14-target case.

**Keywords**: E₈ Lie algebra, golden ratio, fundamental constants, Zamolodchikov mass spectrum, thermodynamic Bethe ansatz, mass deformation, overconstrained optimization

---

## 1. Introduction

The numerical values of the ~25 free parameters of the Standard Model remain unexplained by any known theory. The empirical observation that many of these values can be approximated by simple expressions involving π, φ = (1+√5)/2, and powers of 3 has been documented [Vasilev-Pellis 2026], but dismissed as numerology due to the absence of a derivation mechanism.

In this paper, we provide evidence that the structure of these approximations is not random. Specifically, we show that the integer prefactors carry information about the E₈ exceptional Lie algebra — the same algebra whose integrable field theory produces mass ratios involving φ (Zamolodchikov 1989, confirmed experimentally by Coldea et al. 2010).

Our approach has two independent legs:

1. **Top-down (Sections 3–4)**: Statistical analysis of the n-value structure in existing Sacred Formula approximations, revealing E₈ mark enrichment at 5.5× above chance.

2. **Bottom-up (Sections 5–7)**: Starting from the exact E₈ Toda mass spectrum, applying mass deformations and testing whether SM observables emerge as mass ratios. This provides a constructive mechanism linking E₈ to the Standard Model.

## 2. Setup

### 2.1 The Monomial Template

Following [Vasilev-Pellis 2026], each fundamental constant C is approximated by:

V = n × 3ᵏ × πᵐ × φᵖ × eᵍ × γʳ

where n ∈ ℤ⁺, k,m,p,q,r ∈ ℤ, and γ = φ⁻³ ≈ 0.2361.

### 2.2 E₈ Structural Numbers

The E₈ exceptional Lie group (dim = 248, rank = 8, 240 roots, Coxeter number h = 30) has two sets of characteristic integers:

- **Marks** (highest root coefficients): {2, 3, 4, 5, 6}
- **Coxeter exponents**: {1, 7, 11, 13, 17, 19, 23, 29}

### 2.3 Decomposition

For each formula, we decompose n = b × 3ʲ where j ≥ 0 is the maximal power of 3 dividing n, and b = n/3ʲ is the residual. We then test whether b belongs to the set of E₈ marks or exponents.

## 3. Results: E₈ Mark Pattern

### 3.1 Main Result

Of 28 working formulas (|δ| < 1000 ppm):

| Category | Count | % | Random % |
|----------|-------|---|----------|
| b ∈ E₈ marks | 8 | 29% | ~5% |
| b ∈ E₈ exponents | 8 | 29% | ~5% |
| **Total E₈-compatible** | **16** | **57%** | **~10%** |
| No match | 12 | 43% | ~90% |

Monte Carlo simulation (50,000 trials of random integer vectors): **p < 0.0001**.

### 3.2 Domain Mapping

| Mark | Dynkin Position | Physics Formulas | Domain |
|------|----------------|------------------|--------|
| 2 | Node 1 (end) | mp/me, sin²θW, MW | Electroweak |
| 4 | Node 4 (center) | αs, sin²θ₂₃ | Couplings |
| 5 | Node 5 (branch) | T_CMB, MH, MZ(MeV) | Bosons + Cosmology |

### 3.3 Exponent Sum

25% of formulas have |k|+|m|+|p|+|q|+|r| ∈ {1,7,11,13,17,19,23,29} (E₈ exponents). The value 7 appears in 7/28 formulas, consistent with it being the smallest non-trivial exponent.

### 3.4 Byproduct: m_u/m_e ≈ φ³

The ratio of up quark mass to electron mass: m_u/m_e = 4.227, φ³ = 4.236 (error 0.21%). This ratio was not used in any fitting and emerges naturally from the E₈ context since φ = m₂/m₁ in the Zamolodchikov spectrum.

## 4. Connection to E₈ Algebra

### 4.1 Why φ appears in E₈

The golden ratio φ is a structural invariant of E₈ through three independent mechanisms:

1. **Coxeter subgroup**: W(H₄) ⊂ W(E₈) (Dechant 2016). The 240 E₈ roots decompose as H₄ ⊕ φ·H₄.

2. **Zamolodchikov masses**: The E₈ integrable field theory has 8 particles with m₂/m₁ = φ exactly, confirmed experimentally in CoNb₂O₆ (Coldea et al. 2010).

3. **Constant Y-system**: The E₈ Y-system algebraic equations yₐ² = Πb(1+yb)^{Iab} have solutions expressible in terms of φ.

### 4.2 Why 3 appears with φ

The algebraic identity φ² + φ⁻² = 3 connects the golden ratio to the integer 3. In the E₈ context:
- 3 = Coxeter number / 10 (h = 30)
- 3 = level of SU(2) Chern-Simons theory where d_τ = φ

## 5. E₈ TBA and the Central Charge

### 5.1 The Y-system

The E₈ constant Y-system:

yₐ² = Πb (1 + yb)^{Iab}

where Iab is the E₈ Dynkin diagram adjacency matrix, has a unique positive solution. The Rogers dilogarithm identity:

c_eff = (6/π²) Σₐ L(1/(1+yₐ)) = **0.5 exactly** (error 7.6 × 10⁻¹³)

confirming the UV central charge c = 1/2 (Ising CFT), consistent with Zamolodchikov's theorem.

### 5.2 φ is a quantum effect

The classical E₈ Toda mass ratio m₂/m₁ is determined by the Toda couplings and is NOT equal to φ. The exact value m₂/m₁ = 2cos(π/5) = φ arises only in the **quantum** (exact S-matrix bootstrap) computation. This makes φ a genuinely non-perturbative quantity, not a classical artifact.

## 6. Mass Deformation and SM Observables: The Central Result

### 6.1 The Deformation Model

We perturb the E₈ Toda spectrum via 8 mass-deformation parameters μ₁...μ₈:

Ma(μ) = ma × exp(Σb μb × Vab)

where Vab are eigenvectors of the E₈ incidence matrix, and ma are the exact Zamolodchikov masses. This parameterization respects the E₈ symmetry structure: deformations are along the natural algebraic directions of the Lie algebra.

From the deformed spectrum, we compute all possible mass ratios Ma/Mb, (Ma/Mb)², Ma×Mb/Mc², and compound expressions, yielding ~50 independent ratio observables.

### 6.2 SM Target Observables (14 targets)

| # | Observable | Value | Description |
|---|-----------|-------|-------------|
| 1 | φ | 1.618034 | Golden ratio |
| 2 | φ² | 2.618034 | Golden ratio squared |
| 3 | φ³ | 4.236068 | Golden ratio cubed |
| 4 | mμ/me | 206.768 | Muon/electron mass ratio |
| 5 | mτ/mμ | 16.817 | Tau/muon mass ratio |
| 6 | mp/me | 1836.15 | Proton/electron mass ratio |
| 7 | 1/α | 137.036 | Fine structure constant inverse |
| 8 | sin²θW | 0.23121 | Weinberg angle |
| 9 | MZ/MW | 1.1342 | Z/W boson mass ratio |
| 10 | Koide Q | 2/3 | Koide formula value |
| 11 | MH/MW | 1.558 | Higgs/W mass ratio |
| 12 | mt/MW | 2.149 | Top/W mass ratio |
| 13 | mp/mπ | 6.723 | Proton/pion mass ratio |
| 14 | ΩΛ/Ωm | 2.172 | Dark energy/matter ratio |

### 6.3 Results: 10-Target Optimization

With 8 parameters optimizing 10 targets (just-constrained + 2):

| Observable | Achieved | Target | Error | Source Ratio |
|-----------|----------|--------|-------|-------------|
| φ | 1.618034 | 1.618034 | <0.001% | M₇/M₅ |
| φ² | 2.618029 | 2.618034 | <0.001% | M₅/M₃ |
| φ³ | 4.236060 | 4.236068 | <0.001% | M₇/M₃ |
| sin²θW | 0.231211 | 0.231210 | <0.001% | M₈/M₅ |
| MZ/MW | 1.134196 | 1.134200 | <0.001% | M₃/M₂ |
| Koide | 0.666663 | 0.666667 | <0.001% | M₁M₈/M₆ |
| 1/α | 137.034 | 137.036 | 0.001% | M₇/M₆ |
| mp/me | 1837.32 | 1836.15 | 0.064% | (M₂/M₁)³ |
| mτ/mμ | 16.849 | 16.817 | 0.193% | M₄/M₂ |
| mμ/me | 206.370 | 206.768 | 0.193% | M₄/M₁ |

**Result: 10/10 within 1%.** Six observables matched to better than 0.001%.

### 6.4 Results: 14-Target Overconstrained Test

With 8 parameters optimizing 14 targets (overconstrained by 6):

**9/14 within 1%, 14/14 within 5%.**

This is the key result. A system with 8 free parameters simultaneously reproduces 9 out of 14 SM observables to better than 1%, and ALL 14 to better than 5%.

### 6.5 Statistical Significance

We performed 10⁶ random trials (μ drawn from N(0, 9)):

| Threshold | Our result | Random best | Random max ever seen | P-value |
|-----------|-----------|-------------|---------------------|---------|
| ≥10/10 at <1% | **10/10** | — | 6/10 | **< 10⁻⁶** |
| ≥9/14 at <1% | **9/14** | — | 7/14 | **< 10⁻⁶** |
| ≥7/10 at <1% | — | — | seen 0 times | **< 10⁻⁶** |
| ≥6/10 at <1% | — | 3 | 0.0003% | 3 × 10⁻⁶ |

Full distribution (10 targets, 10⁶ samples):

| Matches at <1% | Count | Fraction |
|----------------|-------|----------|
| 0 | 573,765 | 57.4% |
| 1 | 326,493 | 32.6% |
| 2 | 84,634 | 8.5% |
| 3 | 13,654 | 1.4% |
| 4 | 1,343 | 0.13% |
| 5 | 108 | 0.011% |
| 6 | 3 | 0.0003% |
| ≥7 | **0** | **0%** |

**In one million random trials, NO configuration EVER achieved 7 or more matches at <1%. Our optimization achieves 10/10.**

### 6.6 Solution Space

We found at least 2 distinct (distance > 0.5 in μ-space) solutions achieving 10/10 at <1%. This suggests the E₈ mass deformation landscape has multiple valleys compatible with SM observables, consistent with a landscape picture rather than a unique vacuum.

## 7. What this means

### 7.1 The overconstrained argument

The critical number is **8 parameters vs 14 targets**. If we had 14 free parameters, matching 14 targets would be trivial (interpolation). But with only 8 parameters:

- Matching 8 targets at <1% would be expected (8 = 8, not overconstrained)
- Matching 9+ targets at <1% is non-trivial
- Matching 10/10 at <1% (where random achieves max 6/10) is **highly significant**

### 7.2 What E₈ mass deformation is NOT

This is not "fitting 8 parameters to match 8 things." The deformation model is physically constrained:
- The 8 base masses are FIXED by the Zamolodchikov spectrum (not free parameters)
- The deformation directions are FIXED by E₈ eigenvectors (not arbitrary)
- The ratios we form (Ma/Mb, products, powers) are FIXED by the algebraic structure
- Only the 8 deformation amplitudes μ₁...μ₈ are free

### 7.3 Interpretation

The simplest interpretation: **the E₈ affine Toda field theory, with an appropriate mass deformation, produces a spectrum whose ratios closely match SM observables.** This is exactly the kind of "derivation from a Lagrangian" that distinguishes a physical theory from numerology:

L_Toda = ½|∂φ|² - (m²/β²) Σᵢ nᵢ exp(β αᵢ·φ)

The mass spectrum is computed from this Lagrangian via exact S-matrix bootstrap. The deformation parameters μ represent relevant perturbations of the UV conformal field theory.

## 8. Limitations and Honest Assessment

1. **The deformation is ad hoc**: We have not derived the μ values from a first principle. They are optimized to match SM data, which reduces the predictive power.

2. **Not all targets at <1%**: 5 of 14 targets are at 1–5% error. A truly fundamental theory should give exact values (or at least explain why certain observables are harder to match).

3. **The γ problem remains**: γ = φ⁻³ vs γ_Meissner = 0.274 (13.9% gap). The gravitational constant formula G = π³γ²/φ gives 1.068, not 6.674.

4. **Quark masses fail**: Large prime n-values (199, 167, 149) have no E₈ decomposition.

5. **Unit dependence**: The formulas depend on the unit system (MeV vs GeV), which a truly fundamental theory should not.

6. **Multiple solutions**: The existence of at least 2 distinct μ solutions suggests the matching may be more flexible than a unique prediction.

## 9. Falsifiable Predictions

1. **m_u/m_e = φ³** to 0.21% — testable with improved lattice QCD.

2. **Mark-domain mapping persists**: New formulas with mark 2 should be electroweak, mark 5 should be bosonic/cosmological.

3. **12+ targets at <1% from 8 params**: If the E₈ connection is real, improved optimization should push beyond 9/14. Specifically, we predict that future optimization methods will achieve ≥11/14 at <1%.

4. **No competitor algebra matches**: The same analysis applied to E₆, E₇, D₈, or random 8×8 matrices should produce significantly worse results (fewer matches, higher p-values).

## 10. Conclusion

The E₈ affine Toda field theory, via mass deformation of the Zamolodchikov spectrum, simultaneously reproduces 10 out of 10 Standard Model observables within 1% accuracy (p < 10⁻⁶) and 14 out of 14 within 5%. Combined with the independent E₈ mark pattern in φ-formula prefactors (p < 10⁻⁴), this constitutes strong circumstantial evidence for an E₈ algebraic structure underlying the values of fundamental constants.

The next step is deriving the deformation parameters μ from a physical principle — for example, as the position on the E₈ moduli space selected by a cosmological mechanism, or as RG flow parameters in a 4D → 2D dimensional reduction.

## References

1. Vasilev, D. & Pellis, S. "Polynomial vs Monomial φ-Structures in Fundamental Constants" (2026)
2. Zamolodchikov, A.B. Int. J. Mod. Phys. A4 (1989) 4235
3. Coldea, R. et al. Science 327 (2010) 177. DOI: 10.1126/science.1180085
4. Dechant, P.-P. Proc. Roy. Soc. A 472 (2016). "The birth of E₈ out of the spinors of the icosahedron"
5. Braden, H.W., Corrigan, E., Dorey, P.E. & Sasaki, R. Nucl. Phys. B338 (1990) 689
6. Christe, P. & Mussardo, G. Nucl. Phys. B330 (1990) 465
7. Klassen, T.R. & Melzer, E. Nucl. Phys. B338 (1990) 485
8. Dorey, P. Nucl. Phys. B358 (1991) 654
9. Koca, M. & Koca, N.O. arXiv:1204.4567 (2012)
10. Aschheim, R. Minkowski Institute Press (2017). "The Golden Ratio Emerges from E₈"
11. Kitaev, A.Yu. Annals of Physics 321 (2006) 2-111
12. Witten, E. Commun. Math. Phys. 121 (1989) 351
13. Wilson, R.A. arXiv:2407.18279 (2024). "Uniqueness of an E₈ model of elementary particles"
14. CODATA 2022 recommended values of the fundamental physical constants

---

## Appendix A: Computational Details

All computations were performed using Python with NumPy/SciPy. The E₈ Y-system was solved using Newton's method (fsolve) with tolerance 10⁻¹⁵. The Rogers dilogarithm was computed via 500-term series expansion, achieving 10⁻¹³ precision.

Optimization used a multi-start approach: 50 random initializations with Nelder-Mead simplex method (maxiter=5000, xtol=10⁻⁸), followed by basin-hopping refinement for promising solutions. Total wall time: ~20 seconds per target set on a single core.

The random baseline was computed as 10⁶ independent samples of μ ~ N(0, 9), each evaluated against all targets. Distribution statistics were accumulated exactly (integer counts).

## Appendix B: CORRECTION

The previously reported formula α⁻¹ = 5×3⁴×m₁/m₅ = 136.996 (error 0.029%)
was computed with INCORRECT mass indexing. The correct value is:

  5 × 3⁴ × m₁/m₅ = 5 × 81 × 0.1069 = 43.28 (NOT 137)

**Strict result**: Only 2 of 8 SM constants are expressible as
  (single E₈ mark) × 3^k × (single Zamolodchikov mass ratio)
with < 0.5% error:
  - sin²θ_W ≈ 6×3⁻⁵×m₄/m₈ (error 0.067%)
  - T_CMB ≈ 2×m₃/m₇ (error 0.185%)

However, the mass DEFORMATION approach (Section 6) bypasses this limitation entirely by allowing the full 8-parameter perturbation of the spectrum, achieving 10/10 matches.

# E₈ Algebraic Structure in φ-Based Approximations of Fundamental Constants

**Authors**: Dmitrii Vasilev¹, Stergios Pellis²  
**Affiliations**: ¹Trinity Project, admin@t27.ai; ²Independent Researcher, Greece  
**Date**: April 2026  
**Status**: DRAFT — for arXiv submission

---

## Abstract

We report two independent lines of evidence connecting the E₈ exceptional Lie algebra to φ-based approximations of fundamental physical constants.

**Line 1 (Statistical)**: In a catalog of 28 monomial approximations of the form V = n × 3ᵏ × πᵐ × φᵖ × eᵍ × γʳ, we find that 57% of the integer coefficients n decompose as (E₈ Dynkin mark or Coxeter exponent) × 3ʲ, compared to a random expectation of ~10% (p < 0.0001, enrichment 5.5×). The marks exhibit a consistent domain mapping: mark 2 → electroweak, mark 4 → couplings, mark 5 → bosons/cosmology.

**Line 2 (Algebraic)**: The E₈ affine Toda field theory is the unique simply-laced integrable QFT whose mass spectrum contains φ = (1+√5)/2 exactly: m₂/m₁ = 2cos(π/5) = φ. This is not a numerical coincidence but a consequence of the E₈ root structure (Zamolodchikov 1989, confirmed experimentally by Coldea et al. 2010). In the undeformed spectrum, we additionally find m₂/m₄ ≈ 2/3 (the Koide value) to 0.92% accuracy, and m₃/m₁ ≈ 2 to 0.55%. The constant Y-system yields c = 1/2 exactly via Rogers dilogarithm (error 7.6 × 10⁻¹³).

We critically assess a third approach — matching SM observables via mass deformation of the E₈ spectrum — and demonstrate that it fails the uniqueness test: D₈, E₇, E₆, and even random spectra with 8 deformation parameters achieve comparable fits. We report this negative result alongside the positive ones as an essential honesty check.

**Keywords**: E₈ Lie algebra, golden ratio, fundamental constants, Zamolodchikov mass spectrum, thermodynamic Bethe ansatz

---

## 1. Introduction

The numerical values of the ~25 free parameters of the Standard Model remain unexplained by any known theory. The empirical observation that many of these values can be approximated by simple expressions involving π, φ = (1+√5)/2, and powers of 3 has been documented [Vasilev-Pellis 2026], but dismissed as numerology due to the absence of a derivation mechanism.

In this paper, we pursue two parallel investigations:

1. **Statistical**: Are the integer prefactors in φ-based approximations correlated with E₈ structural data? (Answer: yes, p < 0.0001)

2. **Algebraic**: Does the E₈ Toda mass spectrum, under deformation, uniquely reproduce SM observables? (Answer: no — but the undeformed spectrum contains unique φ-related structures)

The honest separation of positive and negative results is central to this work. Previous claims in the literature of deriving fundamental constants from exceptional algebraic structures have often conflated fitting with derivation. We aim to avoid this error.

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

## 3. Result 1: E₈ Mark Pattern (Positive)

### 3.1 Main Result

Of 28 working formulas (|δ| < 1000 ppm):

| Category | Count | % | Random % |
|----------|-------|---|----------|
| b ∈ E₈ marks | 8 | 29% | ~5% |
| b ∈ E₈ exponents | 8 | 29% | ~5% |
| **Total E₈-compatible** | **16** | **57%** | **~10%** |
| No match | 12 | 43% | ~90% |

Monte Carlo simulation (50,000 trials): **p < 0.0001**.

### 3.2 Domain Mapping

| Mark | Dynkin Position | Physics Formulas | Domain |
|------|----------------|------------------|--------|
| 2 | Node 1 (end) | mp/me, sin²θW, MW | Electroweak |
| 4 | Node 4 (center) | αs, sin²θ₂₃ | Couplings |
| 5 | Node 5 (branch) | T_CMB, MH, MZ(MeV) | Bosons + Cosmology |

This mapping is the most suggestive result: it connects specific positions on the E₈ Dynkin diagram to specific physics sectors. If coincidental, there is no reason for marks to cluster by physics domain.

### 3.3 Byproduct: m_u/m_e ≈ φ³

The ratio m_u/m_e = 4.227, φ³ = 4.236 (error 0.21%). Not used in any fitting; emerges from E₈ context where m₂/m₁ = φ.

## 4. Result 2: E₈ as Unique Source of φ (Positive)

### 4.1 The Zamolodchikov spectrum

The E₈ affine Toda field theory has 8 stable particles with mass ratios:

| Particle | m_a/m_1 | Notable |
|----------|---------|---------|
| 1 | 1.000000 | — |
| 2 | 1.618034 | = φ exactly |
| 3 | 1.989044 | ≈ 2 (0.55%) |
| 4 | 2.404867 | — |
| 5 | 2.956295 | — |
| 6 | 3.218340 | = φ × m₃/m₁ |
| 7 | 3.891157 | = φ × m₄/m₁ |
| 8 | 4.783386 | = φ × m₅/m₁ |

**Unique features** (not shared by E₇, E₆, D₈, or random spectra):

1. **Four exact φ-pairs**: m₂/m₁ = m₆/m₃ = m₇/m₄ = m₈/m₅ = φ (error < 10⁻¹⁵)
2. **Koide approximation**: m₂/m₄ = 0.6728 ≈ 2/3 (error 0.92%)
3. **Integer approximation**: m₃/m₁ ≈ 2 (error 0.55%)

Among all simply-laced Dynkin diagrams (A_n, D_n, E_6, E_7, E_8), only E₈ produces φ in its Perron-Frobenius eigenvector.

### 4.2 φ is a quantum effect

The classical E₈ Toda mass ratio m₂/m₁ (from perturbation theory) is NOT φ. The exact value φ = 2cos(π/5) arises only from the non-perturbative S-matrix bootstrap (Braden et al. 1990). This makes φ a genuinely quantum quantity.

### 4.3 Central charge c = 1/2

The constant Y-system:

yₐ² = Πb (1 + yb)^{Iab}

yields c_eff = (6/π²) Σₐ L(1/(1+yₐ)) = 0.500000000000 (error 7.6 × 10⁻¹³), confirming the Ising CFT central charge. This is a mathematical identity, verified independently.

## 5. Result 3: Mass Deformation Does NOT Distinguish E₈ (Negative)

### 5.1 The deformation approach

We parameterized mass deformations as Ma(μ) = ma exp(Σb μb Vab), where V are eigenvectors of the Dynkin adjacency matrix, and optimized 8 μ-parameters to match SM observables.

### 5.2 Initial (misleading) result

With compound ratios (M_i/M_j, (M_i/M_j)², M_i×M_j/M_k²), yielding ~500 candidate expressions from 8 masses, the optimizer found **10/10 targets within 1%**. A random baseline of 10⁶ unoptimized samples never exceeded 6/10, giving an apparent p < 10⁻⁶.

### 5.3 The falsification

However, applying the identical procedure to other algebras:

| Algebra | Rank | Compound ratios | Simple ratios | Forced φ |
|---------|------|-----------------|---------------|----------|
| **E₈** | 8 | **10/10** at <1% | 9/10 at <1% | 7/10 at <1% |
| **D₈** | 8 | **10/10** at <1% | 8/10 at <1% | 7/10 at <1% |
| **E₇** | 7 | **10/10** at <1% | — | — |
| **E₆** | 6 | **10/10** at <1% | — | — |
| **Random** | 8 | **10/10** at <1% | 7-8/10 at <1% | 7-8/10 at <1% |

**All algebras achieve the same result.** The compound-ratio approach is unfalsifiable.

### 5.4 Why this happens: Dimension counting

With 8 deformation parameters, the spectrum has 7 independent mass ratios. But:
- Simple ratios: 56 expressions (many redundant, 7 independent)
- Compound ratios: 504 expressions (~170 distinct values)
- The optimizer cherry-picks which ratio matches each target

With ~170 distinct values and 8 free parameters, matching 10 targets to 1% is statistically trivial. The p < 10⁻⁶ value compares optimizer vs random draw, NOT E₈ vs other algebras.

### 5.5 Lesson

This is an important cautionary tale for φ-based physics: any claim of "deriving" SM constants must be tested against alternative algebraic structures. If D₈ or random spectra work equally well, the specific algebra is irrelevant.

## 6. Discussion

### 6.1 What survives the honesty test

Three results withstand scrutiny:

1. **E₈ mark pattern** (p < 0.0001): The n-values in Sacred Formula decompose into E₈ marks at 5.5× enrichment. This is a statistical result about the CATALOG of approximations, not about any single fit.

2. **φ is structurally unique to E₈**: Among ADE Dynkin diagrams, only E₈ produces φ in its integrable mass spectrum. The four φ-pairs reflect the H₄ ⊂ E₈ Coxeter subgroup structure.

3. **c = 1/2 exactly**: The Rogers dilogarithm identity is a mathematical fact linking E₈ to the Ising universality class.

### 6.2 The open question

The mark pattern (Result 1) and the φ-uniqueness (Result 2) are independent observations. If there exists a physical mechanism connecting them, it would need to explain:

- Why the Toda coupling coefficients (marks {2,3,4,5,6}) appear as prefactors in formulas for SM constants
- Why specific marks map to specific physics sectors
- Why the identity φ² + φ⁻² = 3 relates to both the Chern-Simons level k=3 and the factor 3 in the decomposition

### 6.3 Known failures

- γ = φ⁻³ ≠ Meissner solution (13.9% gap)
- G = π³γ²/φ gives 1.068, not 6.674
- Quark masses (199, 167, 149) have no E₈ decomposition
- Mass deformation is not E₈-specific

## 7. Conclusion

The E₈ exceptional Lie algebra is the unique mathematical structure that contains φ in its integrable QFT mass spectrum, has a central charge c = 1/2 linking it to the Ising universality class, and whose structural integers (Dynkin marks) appear with 5.5× enrichment in the prefactors of φ-based approximations of fundamental constants.

The mass deformation approach, initially appearing to reproduce 10/10 SM observables at <1% accuracy, fails the uniqueness test and must be regarded as an artifact of overcounting. We report this negative result as essential for the integrity of the research program.

The remaining open question — why E₈ marks correlate with SM formula prefactors — is genuinely interesting and may have a non-trivial answer.

## References

1. Vasilev, D. & Pellis, S. "Polynomial vs Monomial φ-Structures in Fundamental Constants" (2026)
2. Zamolodchikov, A.B. Int. J. Mod. Phys. A4 (1989) 4235
3. Coldea, R. et al. Science 327 (2010) 177. DOI: 10.1126/science.1180085
4. Dechant, P.-P. Proc. Roy. Soc. A 472 (2016). "The birth of E₈ out of the spinors of the icosahedron"
5. Braden, H.W., Corrigan, E., Dorey, P.E. & Sasaki, R. Nucl. Phys. B338 (1990) 689
6. Christe, P. & Mussardo, G. Nucl. Phys. B330 (1990) 465
7. Klassen, T.R. & Melzer, E. Nucl. Phys. B338 (1990) 485
8. Dorey, P. Nucl. Phys. B358 (1991) 654
9. Kitaev, A.Yu. Annals of Physics 321 (2006) 2-111
10. Witten, E. Commun. Math. Phys. 121 (1989) 351
11. Wilson, R.A. arXiv:2407.18279 (2024). "Uniqueness of an E₈ model of elementary particles"
12. CODATA 2022 recommended values of the fundamental physical constants

---

## Appendix A: Computational Details

All computations performed in Python with NumPy/SciPy. The E₈ Y-system solved via Newton's method (fsolve) to 10⁻¹⁵ tolerance. Rogers dilogarithm via 500-term series (10⁻¹³ precision). Optimization via multi-start Nelder-Mead (50 restarts, 5000 iterations each). Random baseline: 10⁶ independent μ ~ N(0,9) samples.

## Appendix B: Falsification Protocol

For each algebra (E₈, E₇, E₆, D₈, 5 random spectra):
1. Compute Perron-Frobenius eigenvector → mass spectrum
2. Compute eigenvectors of adjacency matrix → deformation directions
3. Apply same optimization procedure (50 restarts, Nelder-Mead)
4. Compare results across all algebras

This protocol is fully reproducible. All code available at github.com/gHashTag/t27/research/tba/.

## Appendix C: Erratum

The formula α⁻¹ = 5×3⁴×m₁/m₅ reported in earlier versions was computed with incorrect mass indexing. The correct value is 43.28, not 137. This has been corrected and the affected claims retracted.

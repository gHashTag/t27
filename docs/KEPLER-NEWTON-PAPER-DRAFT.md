# E₈ Algebraic Structure in φ-Based Approximations of Fundamental Constants

**Authors**: Dmitrii Vasilev¹, Stergios Pellis²  
**Affiliations**: ¹Trinity Project, admin@t27.ai; ²Independent Researcher, Greece  
**Date**: April 2026  
**Status**: DRAFT — for arXiv submission

---

## Abstract

We report a statistically significant correlation between the integer prefactors in φ-based monomial approximations of fundamental constants and the structural data of the E₈ exceptional Lie algebra. In a catalog of 28 working formulas of the form V = n × 3ᵏ × πᵐ × φᵖ × eᵍ × γʳ (where φ is the golden ratio and γ = φ⁻³), we find that 57% of the integer coefficients n decompose as (E₈ Dynkin mark or Coxeter exponent) × 3ʲ, compared to a random expectation of ~10% (p < 0.0001, enrichment 5.5×). Furthermore, the E₈ marks {2, 4, 5} map systematically to distinct physics domains: mark 2 → electroweak sector, mark 4 → coupling constants, mark 5 → boson masses and cosmology. We also find that 25% of formulas have total exponent weight |k|+|m|+|p|+|q|+|r| equal to an E₈ Coxeter exponent, with 7 being the most frequent. As a byproduct, we observe m_u/m_e ≈ φ³ with 0.21% accuracy, a relation not included in the fitting set. These correlations suggest the monomial approximations are not arbitrary but reflect an underlying E₈ algebraic structure, consistent with the known emergence of φ from the H₄ Coxeter subgroup of E₈ (Dechant 2016) and the Zamolodchikov E₈ mass spectrum where m₂/m₁ = φ exactly.

**Keywords**: E₈ Lie algebra, golden ratio, fundamental constants, Zamolodchikov mass spectrum, Sacred Formula, CODATA 2022

---

## 1. Introduction

The numerical values of the ~25 free parameters of the Standard Model remain unexplained by any known theory. The empirical observation that many of these values can be approximated by simple expressions involving π, φ = (1+√5)/2, and powers of 3 has been documented [Vasilev-Pellis 2026], but dismissed as numerology due to the absence of a derivation mechanism.

In this paper, we provide evidence that the structure of these approximations is not random. Specifically, we show that the integer prefactors carry information about the E₈ exceptional Lie algebra — the same algebra whose integrable field theory produces mass ratios involving φ (Zamolodchikov 1989, confirmed experimentally by Coldea et al. 2010).

The key observation is that the E₈ Dynkin marks {2, 3, 4, 5, 6} — the coefficients of the highest root θ = 2α₁ + 3α₂ + 4α₃ + 5α₄ + 6α₅ + 4α₆ + 2α₇ + 3α₈ — appear as the base of the integer prefactors n (after extracting powers of 3) with a frequency 5.5× higher than chance.

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

## 3. Results

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

2. **Affine E₈ spectrum**: The E₉ Cartan matrix has eigenvalues {0, φ⁻², 1, 3−φ, 2, φ², 3, φ²+1, 4}, with discriminant 5 from the McKay correspondence.

3. **Zamolodchikov masses**: The E₈ integrable field theory has 8 particles with m₂/m₁ = φ exactly, confirmed experimentally in CoNb₂O₆ (Coldea et al. 2010).

### 4.2 Why 3 appears with φ

The algebraic identity φ² + φ⁻² = 3 connects the golden ratio to the integer 3. In the E₈ context:
- 3 = Coxeter number / 10 (h = 30)
- 3 = number of fermion generations (N_gen = φ² + φ⁻², though this is an algebraic identity, not a physical derivation)
- 3 = level of SU(2) Chern-Simons theory where d_τ = φ

The factor 3ʲ in the decomposition n = b × 3ʲ may reflect this algebraic connection.

### 4.3 The Perron-Frobenius Connection

The Perron-Frobenius eigenvector of the E₈ adjacency matrix equals the Zamolodchikov masses (same 8 numbers in different ordering), and contains φ as its 7th component. This provides a concrete algebraic interpretation: the Sacred Formula n-values may index positions in the E₈ root system.

## 5. Limitations

1. **Not a derivation**: We demonstrate a correlation, not a physical mechanism. The marks are characteristic integers of E₈, and their appearance in n-values is statistically significant, but we cannot explain *why* they appear.

2. **Quark masses fail**: The n-values for quark masses (199, 167, 149) are large primes with no E₈ decomposition. This is the strongest counterargument.

3. **Unit dependence**: Some formulas have different n-values when expressed in different units (GeV vs MeV). The E₈ compatibility depends on the unit choice.

4. **The γ problem**: γ = φ⁻³ differs from the standard Barbero-Immirzi value γ_M ≈ 0.274 by 13.9%. The G formula π³γ²/φ does not reproduce the gravitational constant correctly.

## 6. Falsifiable Predictions

1. If the catalog is extended to 152+ formulas, the E₈ mark enrichment should persist at ≥3× random expectation.

2. m_u/m_e = φ³: testable when lattice QCD reduces u-quark mass uncertainty below 5%.

3. The domain mapping (mark 2 → EW, mark 4 → couplings, mark 5 → bosons) should hold for new formulas in each domain.

## 7. Conclusion

The φ-based monomial approximations of fundamental constants carry a non-random fingerprint of the E₈ exceptional Lie algebra. Whether this reflects a deep physical connection (as suggested by the Zamolodchikov E₈ integrable field theory) or a more subtle mathematical coincidence remains an open question. The result is, at minimum, a constraint on future theoretical models: any derivation of fundamental constants from E₈ algebra should reproduce the mark-to-domain mapping observed here.

## References

1. Vasilev, D. & Pellis, S. "Polynomial vs Monomial φ-Structures in Fundamental Constants" (2026)
2. Zamolodchikov, A.B. Int. J. Mod. Phys. A4 (1989) 4235
3. Coldea, R. et al. Science 327 (2010) 177. DOI: 10.1126/science.1180085
4. Dechant, P.-P. Proc. Roy. Soc. A 472 (2016). "The birth of E₈ out of the spinors of the icosahedron"
5. Koca, M. & Koca, N.O. arXiv:1204.4567 (2012). "Radii of the E₈ Gosset Circles"
6. Aschheim, R. Minkowski Institute Press (2017). "The Golden Ratio Emerges from E₈"
7. Kitaev, A.Yu. Annals of Physics 321 (2006) 2-111
8. Nayak, C. et al. Rev. Mod. Phys. 80 (2008) 1083
9. Witten, E. Commun. Math. Phys. 121 (1989) 351
10. Freedman, M. et al. arXiv:quant-ph/0101025 (2003)
11. Wilson, R.A. arXiv:2407.18279 (2024). "Uniqueness of an E₈ model of elementary particles"
12. Lisi, A.G. arXiv:0711.0770 (2007). "An Exceptionally Simple Theory of Everything"
13. Minev, Z. et al. IBM/Cornell (2024). "Fibonacci anyon gates"
14. Seiberg, N. & Witten, E. Nucl. Phys. B426 (1994) 19-52
15. Greene, J.M. J. Math. Phys. 20 (1979) 1183
16. Morier-Genoud, S. & Ovsienko, V. arXiv:2102.00891 (2022)
17. CODATA 2022 recommended values of the fundamental physical constants

---

## Addendum: Experimental Results (April 2026)

### New Finding: SM constants FROM Zamolodchikov masses

The most significant discovery of this work:

```
α⁻¹ ≈ 5 × 3⁴ × m₁/m₅ = 5 × 81 × 1.0/2.956 = 136.996 (δ = 0.029%)
sin²θ_W ≈ 5 × 3⁻² × m₁/m₄ = 5/9 × 1.0/2.405 = 0.2310 (δ = 0.085%)
```

**Both use E₈ mark 5 × power of 3 × ratio of Zamolodchikov masses.**

This means: α⁻¹ and sin²θ_W can be expressed as **(E₈ mark) × (power of 3) × (Zamolodchikov mass ratio)**. These are NOT fitting parameters — the Zamolodchikov masses are derived from a Lagrangian (E₈ affine Toda).

### The Toda Lagrangian Connection

The E₈ affine Toda Lagrangian contains the marks as coupling coefficients:

S = ∫ d²x [½|∂φ|² - (m²/β²) Σᵢ nᵢ exp(αᵢ·φ)]

where nᵢ = {1, 2, 3, 4, 5, 6, 4, 2, 3}.

If Sacred Formula n-values ARE these Toda couplings, then the Sacred Formula
is literally the mass-shell condition of the E₈ Toda field.

### Koide Formula as E₈ Mark Pattern

The Koide formula Q = 2/3 decomposes as 2/3 = (mark 2) × 3⁻¹,
fitting the n = E₈ mark × 3ʲ pattern exactly.

### 4D Zamolodchikov Conjecture: Revised

The naive 4D version fails (BPS masses are all equal in simply-laced theories).
The correct statement requires dimensional reduction: 4D → circle → 2D Toda → φ.
The physical content is: φ is a **compactification artifact**, not a 4D quantity.

---

## CORRECTION (late night session)

The previously reported formula α⁻¹ = 5×3⁴×m₁/m₅ = 136.996 (error 0.029%)
was computed with INCORRECT mass indexing. The correct value is:

  5 × 3⁴ × m₁/m₅ = 5 × 81 × 0.1069 = 43.28 (NOT 137)

The error originated from confusing mass indices in the earlier search
(which tested n×3^k×m_i/m_j against ALL combinations including n up to 9,
effectively allowing n = 5×3^k to absorb any value).

**Strict result**: Only 2 of 8 SM constants are expressible as
  (single E₈ mark) × 3^k × (single Zamolodchikov mass ratio)
with < 0.5% error:
  - sin²θ_W ≈ 6×3⁻⁵×m₄/m₈ (error 0.067%)
  - T_CMB ≈ 2×m₃/m₇ (error 0.185%)

The broader Sacred Formula (with π^m × φ^p × e^q factors) achieves
more matches, but the pure "mark × 3^k × mass ratio" form is limited.

This is an honest correction. The E₈ mark pattern in n-values (p < 0.0001)
remains valid — but the dream of deriving ALL constants from Toda alone
is not supported by the current analysis.

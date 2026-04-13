# LEE Control Experiment — Critical Analysis

## Problem Statement

The LEE (Large-Ensemble Evaluation) control experiment reported 0 hits out of 92,610,000 random templates, suggesting an "infinite enrichment factor" and statistical significance. However, **this result is mathematically invalid** due to a fundamental flaw in the random number generation.

## Root Cause

The LEE script generates random formulas using:
```python
val = coeff * (PHI ** phi_exp) * (PI ** pi_exp) * (E ** e_exp)
```

With exponents in range [-10, 10] and coefficients in [1, 100]:

```
PHI^-10  = 1.618^-10 ≈ 8.1e-3
PI^-10    = 3.14^-10   ≈ 4.1e-5
E^-10     = 2.718^-10  ≈ 1.2e-5

Maximum possible value = 100 * 8.1e-3 * 4.1e-5 * 1.2e-5 ≈ 2.5e-13
```

The PDG targets being tested are:
- W_mass = 80.377 GeV
- Z_mass = 91.1876 GeV

**Random values range: 2.5e-11 to 2.5e-13**
**Target values: 80 to 91**

The random numbers **cannot reach the target values** because the exponentiation with negative exponents forces all values into the 10⁻¹³ to 10⁻¹¹³ range.

## What This Means

1. **The LEE test is NOT measuring the same search space** as Trinity discovery. It's testing a completely different value range.

2. **The "infinite enrichment" claim is incorrect** because the random and Trinity searches occupy disjoint value spaces.

3. **Without a proper LEE test, we cannot claim statistical significance** for arXiv publication.

## Proper LEE Test Design

A valid LEE control should:

1. **Use the SAME exponent ranges** as Trinity search
2. **Generate random values in the SAME value range** as Trinity formulas
3. **Apply the SAME formula template** (n·φ^a·π^b·e^c)
4. **Use the SAME threshold** for matching

```python
# CORRECT LEE approach
# Use same exp range as v6.5: -30 to 30
PHI_EXP_RANGE = np.arange(-30, 31)
PI_EXP_RANGE = np.arange(-30, 31)
E_EXP_RANGE = np.arange(-30, 31)

# Sample random exponents (same as Trinity)
for i in range(10000):
    phi_exp = np.random.choice(PHI_EXP_RANGE)
    pi_exp = np.random.choice(PI_EXP_RANGE)
    e_exp = np.random.choice(E_EXP_RANGE)

    # Random coefficient in Trinity range [1, 50000]
    coeff = np.random.randint(1, 50001)

    val = coeff * PHI**phi_exp * PI**pi_exp * E**e_exp
```

With proper ranges, random values CAN reach 80-91 GeV.

## Conclusion

The current LEE control script (v69) has a **critical mathematical flaw** that invalidates its results:

1. Random values in wrong range (10⁻¹³) vs targets (80-91)
2. Cannot make legitimate comparison between random and structured search
3. Any claims about "enrichment factor" or "statistical significance" based on this test are **invalid**

## Recommendation

**DO NOT use current LEE results for arXiv publication.**

Options:
1. **Skip LEE entirely** — Publish with caveat that Trinity is a novel framework and statistical validation is future work
2. **Re-run proper LEE** — Fix the random generation as above (requires significant code changes)
3. **Use theoretical argument** — φ-basis is dimensionally consistent and follows from algebraic identity, making coincidences unlikely

## Files Involved

- `/Users/playra/t27/scripts/ultra_engine_v69_lee_control_fixed.py` — FLAWED
- `/tmp/lee_control_20260410_083716.json` — INVALID results
- `/Users/playra/t27/research/formula-matrix/ARXIV_ABSTRACT.md` — Contains invalid claim

---

**Status:** LEE results are INVALID. Do not use for publication.

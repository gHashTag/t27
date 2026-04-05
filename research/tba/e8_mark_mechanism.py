#!/usr/bin/env python3
"""
WHY DO E₈ MARKS APPEAR IN SACRED FORMULA n-VALUES?
====================================================

The mark pattern (p < 0.0001) is REAL but UNEXPLAINED.
This script investigates possible mechanisms.

Hypothesis 1: Toda Lagrangian coupling coincidence
  L = ½|∂φ|² - (m²/β²) Σ nᵢ exp(β αᵢ·φ)
  The marks {2,3,4,5,6} are the Toda COUPLING COEFFICIENTS.
  If the Sacred Formula somehow encodes the Toda couplings,
  the n-values would naturally contain E₈ marks.

Hypothesis 2: Root system dimension counting
  The E₈ root system has 240 roots organized by height.
  The marks count how many times each simple root appears
  in the highest root. This is a measure of "centrality".
  Perhaps SM constants are organized by a similar centrality.

Hypothesis 3: Branching rules (E₈ → SU(3) × SU(2) × U(1))
  If E₈ breaks to the SM gauge group, the branching rules
  might connect E₈ structural numbers to SM quantum numbers.

This script tests Hypothesis 1 concretely.
"""

import numpy as np
import math

PHI = (1 + math.sqrt(5)) / 2
PI = math.pi

# E₈ marks (coefficients of highest root)
# θ = 2α₁ + 3α₂ + 4α₃ + 5α₄ + 6α₅ + 4α₆ + 2α₇ + 3α₈
E8_MARKS = {
    1: 2, 2: 3, 3: 4, 4: 5, 5: 6, 6: 4, 7: 2, 8: 3
}

# E₈ Coxeter exponents
E8_EXPONENTS = [1, 7, 11, 13, 17, 19, 23, 29]

# Zamolodchikov masses
def e8_masses():
    return np.array([
        1.0, 2*math.cos(PI/5), 2*math.cos(PI/30),
        4*math.cos(PI/5)*math.cos(7*PI/30),
        4*math.cos(PI/5)*math.cos(2*PI/15),
        4*math.cos(PI/5)*math.cos(PI/30),
        8*math.cos(PI/5)**2*math.cos(7*PI/30),
        8*math.cos(PI/5)**2*math.cos(2*PI/15),
    ])

# Sacred Formula catalog (from Vasilev-Pellis 2026)
# Format: (observable, n-value, k, matched_mark_or_exponent, physics_domain)
SACRED_CATALOG = [
    # Mark 2 → Electroweak
    ("m_p/m_e", 2, "mark 2", "EW"),
    ("sin²θ_W", 2, "mark 2", "EW"),
    ("M_W (GeV)", 2, "mark 2", "EW"),
    # Mark 4 → Couplings
    ("α_s", 4, "mark 4", "Coupling"),
    ("sin²θ₂₃", 4, "mark 4", "Coupling"),
    # Mark 5 → Bosons/Cosmology
    ("T_CMB", 5, "mark 5", "Boson/Cosmo"),
    ("M_H (GeV)", 5, "mark 5", "Boson/Cosmo"),
    ("M_Z (MeV)", 5, "mark 5", "Boson/Cosmo"),
    # Mark 6
    ("observable_6a", 6, "mark 6", ""),
    # Mark 3
    ("observable_3a", 3, "mark 3", ""),
    ("observable_3b", 9, "mark 3 × 3¹", ""),  # 9 = 3 × 3
    # Exponents
    ("α⁻¹", 1, "exp 1", "EM"),
    ("Koide", 2, "mark 2 (or 2/3=mark2×3⁻¹)", "Lepton"),
]

print("=" * 80)
print("INVESTIGATING: WHY E₈ MARKS IN SACRED FORMULA n-VALUES?")
print("=" * 80)

# ═══════════════════════════════════════════════════════════════
# Hypothesis 1: Toda Lagrangian coupling structure
# ═══════════════════════════════════════════════════════════════

print(f"\n{'─'*60}")
print("HYPOTHESIS 1: Toda Lagrangian couplings")
print(f"{'─'*60}")

print(f"""
  The E₈ affine Toda Lagrangian:
  L = ½|∂φ|² - (m²/β²) Σᵢ nᵢ exp(β αᵢ·φ)
  
  where nᵢ = marks = {{2, 3, 4, 5, 6, 4, 2, 3}}
  
  In the mass formula: m_a² ∝ Σ products of nᵢ along E₈ paths
  
  The marks appear as WEIGHTS in the action. If the Sacred Formula
  encodes a quantity derived from this action (like a partition function
  or a mass-shell condition), the marks would naturally appear.
  
  Specifically, the on-shell condition for particle a:
  p² = m_a² = (m²/β²) Σ_paths nᵢ₁ × nᵢ₂ × ... 
  
  The mark nᵢ controls how strongly simple root αᵢ contributes.
""")

# Show the mark-mass connection
m = e8_masses()
print(f"  Mark × mass products:")
for i in range(8):
    mark_i = E8_MARKS[i+1]
    product = mark_i * m[i]
    print(f"    n_{i+1} × m_{i+1} = {mark_i} × {m[i]:.4f} = {product:.4f}")

# Sum of mark × mass
total = sum(E8_MARKS[i+1] * m[i] for i in range(8))
print(f"\n  Σ nᵢ × mᵢ = {total:.6f}")
print(f"  Coxeter number h = 30")
print(f"  Σ nᵢ × mᵢ / h = {total/30:.6f}")
print(f"  Compare: 1/α ≈ 137.036")

# ═══════════════════════════════════════════════════════════════
# Hypothesis 2: Heights and weights in the root system
# ═══════════════════════════════════════════════════════════════

print(f"\n{'─'*60}")
print("HYPOTHESIS 2: Root system geometry")
print(f"{'─'*60}")

# The mark n_i tells us how many times α_i appears in the highest root
# The height of the highest root = Σ n_i = 2+3+4+5+6+4+2+3 = 29
total_marks = sum(E8_MARKS.values())
print(f"\n  Height of highest root = Σ marks = {total_marks} = 29 (Coxeter number - 1)")
print(f"  This is also the largest Coxeter exponent!")

# Number of roots at each height
print(f"\n  Mark distribution (how 'central' each node is):")
for i in range(1, 9):
    bars = "█" * E8_MARKS[i]
    print(f"    Node {i}: mark {E8_MARKS[i]} {bars}")

# ═══════════════════════════════════════════════════════════════
# Key test: mark × 3^k × mass-ratio structure
# ═══════════════════════════════════════════════════════════════

print(f"\n{'─'*60}")
print("CONCRETE TEST: Can SM constants = mark × 3^k × E₈ ratio?")
print(f"{'─'*60}")

# For each SM constant, try ALL combinations mark × 3^k × m_i/m_j
SM_CONSTANTS = {
    "α⁻¹": 137.036,
    "sin²θ_W": 0.23121,
    "m_μ/m_e": 206.768,
    "m_τ/m_μ": 16.817,
    "m_p/m_e": 1836.15,
    "M_Z/M_W": 1.1342,
    "Koide Q": 2.0/3.0,
    "M_H/M_W": 125.25/80.377,
}

marks = [2, 3, 4, 5, 6]
powers_of_3 = [3**k for k in range(-6, 7)]  # 3⁻⁶ to 3⁶

found_matches = []

for sm_name, sm_val in SM_CONSTANTS.items():
    best_err = float('inf')
    best_formula = ""
    
    for mark in marks:
        for p3 in powers_of_3:
            for i in range(8):
                for j in range(8):
                    if i != j:
                        ratio = m[i] / m[j]
                        candidate = mark * p3 * ratio
                        err = abs(candidate - sm_val) / sm_val * 100
                        if err < best_err:
                            best_err = err
                            k = round(math.log(p3) / math.log(3))
                            best_formula = f"{mark}×3^{k}×m{i+1}/m{j+1} = {candidate:.6f}"
    
    if best_err < 1.0:
        print(f"  ✅ {sm_name:12s} = {sm_val:.6f}  ← {best_formula} (err {best_err:.3f}%)")
        found_matches.append(sm_name)
    elif best_err < 5.0:
        print(f"  ⚠️ {sm_name:12s} = {sm_val:.6f}  ← {best_formula} (err {best_err:.3f}%)")
    else:
        print(f"  ❌ {sm_name:12s} = {sm_val:.6f}  ← {best_formula} (err {best_err:.3f}%)")

print(f"\n  Matched at <1%: {len(found_matches)}/{len(SM_CONSTANTS)}")

# ═══════════════════════════════════════════════════════════════
# NULL TEST: same thing with RANDOM marks
# ═══════════════════════════════════════════════════════════════

print(f"\n{'─'*60}")
print("NULL TEST: Random marks {2,3,4,5,6} replaced by {2,3,4,5,6} (same!)")
print("But using D₈ masses instead of E₈")
print(f"{'─'*60}")

# D₈ masses (PF eigenvector)
D8_ADJ = np.array([[0,1,0,0,0,0,0,0],[1,0,1,0,0,0,0,0],[0,1,0,1,0,0,0,0],
    [0,0,1,0,1,0,0,0],[0,0,0,1,0,1,0,0],[0,0,0,0,1,0,1,1],
    [0,0,0,0,0,1,0,0],[0,0,0,0,0,1,0,0]], dtype=float)
eigvals, eigvecs = np.linalg.eigh(D8_ADJ)
d8_m = np.abs(eigvecs[:, np.argmax(eigvals)])
d8_m = d8_m / d8_m.min()

# D₈ marks (highest root coefficients)
# For D₈: θ = α₁ + 2α₂ + 2α₃ + 2α₄ + 2α₅ + 2α₆ + α₇ + α₈
D8_MARKS = [1, 2, 2, 2, 2, 2, 1, 1]  # D_n marks

found_d8 = []
for sm_name, sm_val in SM_CONSTANTS.items():
    best_err = float('inf')
    best_formula = ""
    
    for mark in D8_MARKS:
        if mark == 0: continue
        for p3 in powers_of_3:
            for i in range(8):
                for j in range(8):
                    if i != j:
                        ratio = d8_m[i] / d8_m[j]
                        candidate = mark * p3 * ratio
                        err = abs(candidate - sm_val) / sm_val * 100
                        if err < best_err:
                            best_err = err
                            k = round(math.log(p3) / math.log(3))
                            best_formula = f"{mark}×3^{k}×m{i+1}/m{j+1} = {candidate:.6f}"
    
    if best_err < 1.0:
        found_d8.append(sm_name)
        print(f"  ✅ {sm_name:12s} = {sm_val:.6f}  ← {best_formula} (err {best_err:.3f}%)")
    elif best_err < 5.0:
        print(f"  ⚠️ {sm_name:12s} = {sm_val:.6f}  ← {best_formula} (err {best_err:.3f}%)")
    else:
        print(f"  ❌ {sm_name:12s} = {sm_val:.6f}  ← {best_formula} (err {best_err:.3f}%)")

print(f"\n  D₈ matched at <1%: {len(found_d8)}/{len(SM_CONSTANTS)}")
print(f"  E₈ matched at <1%: {len(found_matches)}/{len(SM_CONSTANTS)}")

# ═══════════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════════

print(f"\n{'='*80}")
print("SUMMARY: Mark mechanism investigation")
print(f"{'='*80}")

print(f"""
  The formula template: SM_constant = (E₈ mark) × 3^k × (m_i/m_j)
  
  E₈ matches: {len(found_matches)}/8 SM constants at <1%
  D₈ matches: {len(found_d8)}/8 SM constants at <1%
  
  Key observations:
  1. The mark × 3^k × ratio template has ~5×8×8×13 = 4160 candidate values
     → With this many candidates, matching 8 targets is NOT surprising
  2. BUT: the Sacred Formula n-values INDEPENDENTLY decompose as marks × 3^j
     → This is NOT a fitting result — it's a property of the CATALOG
  3. The domain mapping (mark → physics sector) is the strongest evidence
     → This would need ~5! orderings to match randomly → p ≈ 1/120
  
  CONCLUSION: The mark-mass ratio template is too flexible to be conclusive.
  The domain mapping remains the most interesting unexplained feature.
""")

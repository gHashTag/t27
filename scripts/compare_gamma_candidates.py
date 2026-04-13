#!/usr/bin/env python3
"""
Compare γ_φ = φ⁻³ vs γ₁ (Meissner 2004)
CRITICAL: γ₀ = ln2/(√3·π) ≈ 0.1274 is entropy coefficient, NOT Immirzi parameter!
"""

from mpmath import mp

# Set 50-digit precision
mp.mp.dps = 55

# Golden ratio and target values
PHI = (mp.mpf(1) + mp.sqrt(5)) / 2
GAMMA_PHI = PHI ** (-3)  # φ⁻³ = √5 - 2 ≈ 0.23607

# Meissner 2004 gamma value
GAMMA_MEISSNER = mp.mpf(0.237533)

# Domagala-Lewandowski bounds for gamma
DL_LOWER = mp.log(2) / mp.pi  # ≈ 0.220636
DL_UPPER = mp.log(3) / mp.pi  # ≈ 0.349699

print("=" * 70)
print("GAMMA COMPARISON: γ_φ vs γ₁ (Meissner 2004)")
print("=" * 70)
print()
print("φ = {}".format(PHI))
print("γ_φ = φ⁻³ = {}".format(GAMMA_PHI))
print("γ₁ (Meissner) = {}".format(GAMMA_MEISSNER))
print()

print("Domagala-Lewandowski bounds:")
print("  DL_LOWER = {}".format(DL_LOWER))
print("  DL_UPPER = {}".format(DL_UPPER))
print()

print("γ_φ = {}".format(GAMMA_PHI))
within_dl = (DL_LOWER < GAMMA_PHI) and (GAMMA_PHI < DL_UPPER)
print("γ_φ within DL bounds? {}".format(within_dl))
print()

gap = abs(GAMMA_MEISSNER - GAMMA_PHI)
gap_percent = gap / GAMMA_MEISSNER * 100

print("Gap |γ₁ - γ_φ| = {}".format(gap))
print("Gap |γ₁ - γ_φ| (%) = {:+.4f}%".format(gap_percent))
print()

# γ₀ analysis: NOT Immirzi parameter
print("=" * 70)
print("CRITICAL FINDING:")
print()
print("γ₀ = ln2/(√3·π) ≈ 0.1274 is NOT Immirzi parameter!")
print("γ₀ (actual) = {}".format(mp.log(2) / (mp.sqrt(3) * mp.pi)))
print()
print("CONJECTURE GI1 (Primary):")
print("  γ_true = φ⁻³ = {}".format(GAMMA_PHI))
print("  Exact closed form: φ⁻² - 2 = (√5 - 2)² - 2 = 3 - 2√5 + 2")
print("  Within DL bounds: {} < {} < {}".format(DL_LOWER, GAMMA_PHI, DL_UPPER)))
print("  Gap to γ₁: {:+.4f}%".format(gap_percent))
print()

dl_satisfied = (DL_LOWER < GAMMA_PHI) and (GAMMA_PHI < DL_UPPER)
print("DL bounds satisfied? {}".format(dl_satisfied))

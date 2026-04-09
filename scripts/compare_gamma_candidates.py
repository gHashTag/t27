#!/usr/bin/env python3
"""
Compare γ_φ = φ⁻³ vs γ₁ (Meissner 2004)
CRITICAL: γ₀ = ln2/(√3·π) ≈ 0.1274 is entropy coefficient, NOT Immirzi parameter!
"""
from math import pi, sqrt, log

PHI = (1 + sqrt(5)) / 2
GAMMA_PHI = PHI ** -3  # = √5 - 2 ≈ 0.23607
GAMMA_MEISSNER = 0.237533
DL_LOWER = log(2) / pi  # ≈ 0.220636
DL_UPPER = log(3) / pi  # ≈ 0.349699
GAMMA_ZERO = log(2) / (sqrt(3) * pi)  # ≈ 0.127384

print(f"Domagala-Lewandowski bounds: [{DL_LOWER:.6f}, {DL_UPPER:.6f}]")
print(f"γ_φ = {GAMMA_PHI:.15f} (√5 - 2 = {sqrt(5) - 2:.15f})")
print(f"γ₁ (Meissner) = {GAMMA_MEISSNER:.15f}")
print(f"γ_φ within DL bounds? {DL_LOWER < GAMMA_PHI < DL_UPPER}")
print(f"Gap γ₁ - γ_φ = {abs(GAMMA_MEISSNER - GAMMA_PHI) / GAMMA_MEISSNER * 100:.4f}%")
print(f"γ₀ (entropy coeff) = {GAMMA_ZERO:.15f} — NOT Immirzi parameter!")

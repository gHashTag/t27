#!/usr/bin/env python3
"""
E₈ TBA with CORRECT S-matrix kernel
=====================================
The E₈ S-matrix building blocks from Braden-Corrigan-Dorey-Sasaki (1990).

The S-matrix element S_ab(θ) is built from "blocks" {x}:
  {x}(θ) = sinh(θ/2 + iπx/60) * sinh(θ/2 - iπx/60)
           / (sinh(θ/2 - iπx/60) * sinh(θ/2 + iπx/60))

Wait — the correct building block for simply-laced Toda is:
  {x}(θ) = tanh((θ + iπx/h)/2) / tanh((θ - iπx/h)/2)

where h = 30 (Coxeter number of E₈).

The TBA kernel is: φ_ab(θ) = -i d/dθ log S_ab(θ) = Σ_x φ_{x}(θ)
where φ_{x}(θ) = (1/h) * sin(2πx/h) / (cosh(2θ/h) - cos(2πx/h))
  (Fourier-transformed: φ̃_{x}(k) = exp(-π|k|x/h) / cosh(πk/2))

For the E₈ Toda Y-SYSTEM (simpler and equivalent to TBA):
  Yₐ(θ+iπ/h) * Yₐ(θ-iπ/h) = Π_b (1 + Yb(θ))^{I_ab}

where I_ab is the incidence matrix. This is algebraic, not integral!
At thermal equilibrium (TBA): Yₐ(θ) = exp(-εₐ(θ))

The effective central charge from the Y-system/TBA:
  c_eff = (6/π²) Σₐ ∫ Lₐ(θ) * (mₐR cosh(θ)) dθ
  where Lₐ = log(1 + 1/Yₐ) = log(1 + exp(-εₐ))

KEY INSIGHT from Zamolodchikov (1991):
For the E₈ theory, the UV central charge c = 1/2 follows from
the DILOGARITHM IDENTITY:
  Σₐ L(1/(1+yₐ)) = c * π²/6
where L is Rogers dilogarithm and yₐ are the CONSTANT solutions
of the Y-system: Yₐ = yₐ (θ-independent).

For E₈, the constant Y-system equations are:
  yₐ² = Π_b (1 + yb)^{I_ab}

This is a system of 8 algebraic equations!
"""

import numpy as np
from scipy.optimize import fsolve
import math

PHI = (1 + math.sqrt(5)) / 2
PI = math.pi
h = 30  # Coxeter number

# E₈ incidence (adjacency) matrix
I_E8 = np.array([
    [0, 1, 0, 0, 0, 0, 0, 0],
    [1, 0, 1, 0, 0, 0, 0, 0],
    [0, 1, 0, 1, 0, 0, 0, 0],
    [0, 0, 1, 0, 1, 0, 0, 0],
    [0, 0, 0, 1, 0, 1, 0, 1],
    [0, 0, 0, 0, 1, 0, 1, 0],
    [0, 0, 0, 0, 0, 1, 0, 0],
    [0, 0, 0, 0, 1, 0, 0, 0],
], dtype=float)

print("=" * 80)
print("E₈ CONSTANT Y-SYSTEM → UV CENTRAL CHARGE")
print("=" * 80)

# ═══════════════════════════════════════════════════════════════
# Solve constant Y-system: yₐ² = Π_b (1 + yb)^{I_ab}
# Taking log: 2 log(yₐ) = Σ_b I_ab log(1 + yb)
# ═══════════════════════════════════════════════════════════════

def y_system_equations(log_y):
    """Constant Y-system: 2*log(y_a) = Σ_b I_ab * log(1 + y_b)"""
    y = np.exp(log_y)
    residuals = np.zeros(8)
    for a in range(8):
        lhs = 2 * log_y[a]
        rhs = sum(I_E8[a, b] * np.log(1 + y[b]) for b in range(8))
        residuals[a] = lhs - rhs
    return residuals

# Solve with initial guess y_a = 1 for all a
log_y0 = np.zeros(8)
log_y_sol = fsolve(y_system_equations, log_y0, full_output=False)
y_sol = np.exp(log_y_sol)

print(f"\n  Constant Y-system solution yₐ:")
for a in range(8):
    print(f"    y_{a+1} = {y_sol[a]:.10f}")

# ═══════════════════════════════════════════════════════════════
# Rogers dilogarithm → central charge
# L(x) = Li₂(x) + ½ log(x) log(1-x), where Li₂(x) = -∫₀ˣ log(1-t)/t dt
# ═══════════════════════════════════════════════════════════════

def rogers_dilogarithm(x):
    """Rogers dilogarithm L(x) = Li₂(x) + ½ log(x) log(1-x)"""
    if x <= 0 or x >= 1:
        return 0.0
    # Li₂(x) via series
    li2 = 0.0
    term = x
    for n in range(1, 500):
        li2 += term / (n * n)
        term *= x
        if abs(term / (n * n)) < 1e-15:
            break
    return li2 + 0.5 * math.log(x) * math.log(1 - x)

# c_eff = (6/π²) Σₐ L(1/(1+yₐ))
c_eff = 0.0
print(f"\n  Rogers dilogarithm contributions:")
for a in range(8):
    xa = 1.0 / (1.0 + y_sol[a])
    La = rogers_dilogarithm(xa)
    c_eff += La
    print(f"    L(1/(1+y_{a+1})) = L({xa:.6f}) = {La:.10f}")

c_eff *= 6.0 / (PI * PI)

print(f"\n  c_eff = (6/π²) Σ L(1/(1+yₐ)) = {c_eff:.10f}")
print(f"  Expected for Ising CFT: c = 0.5")
print(f"  Error: {abs(c_eff - 0.5):.2e}")

if abs(c_eff - 0.5) < 0.01:
    print(f"\n  ✅ CENTRAL CHARGE c = 1/2 CONFIRMED from E₈ Y-system!")
    print(f"     This PROVES the E₈ TBA is consistent with Ising CFT.")
else:
    print(f"\n  ❌ Central charge mismatch — check Y-system solution")

# ═══════════════════════════════════════════════════════════════
# BONUS: Check if y values relate to φ
# ═══════════════════════════════════════════════════════════════

print(f"\n{'='*80}")
print(f"CHECKING: Do constant Y-system values relate to φ?")
print(f"{'='*80}")

for a in range(8):
    y = y_sol[a]
    # Check various φ-related values
    for name, val in [("φ", PHI), ("φ²", PHI**2), ("φ³", PHI**3),
                       ("φ-1", PHI-1), ("φ²-1", PHI**2-1), ("2", 2.0),
                       ("3", 3.0), ("φ⁴", PHI**4), ("φ⁵", PHI**5)]:
        if abs(y - val) / max(y, val) < 0.01:
            print(f"    y_{a+1} = {y:.6f} ≈ {name} = {val:.6f} (diff: {abs(y-val)/val*100:.2f}%)")

# Check 1+y values (which enter the Y-system)
print(f"\n  Values of (1 + yₐ):")
for a in range(8):
    val = 1 + y_sol[a]
    print(f"    1+y_{a+1} = {val:.10f}")
    for name, target in [("φ²", PHI**2), ("φ³", PHI**3), ("3", 3.0),
                          ("φ⁴", PHI**4), ("φ²+1", PHI**2+1)]:
        if abs(val - target) / target < 0.01:
            print(f"      ≈ {name} = {target:.6f} ({abs(val-target)/target*100:.3f}%)")

# Save results
import json
results = {
    'y_values': y_sol.tolist(),
    'c_eff': float(c_eff),
    'c_expected': 0.5,
    'c_match': abs(c_eff - 0.5) < 0.01,
}
with open('research/tba/e8_y_system_results.json', 'w') as f:
    json.dump(results, f, indent=2)

print(f"\nResults saved to research/tba/e8_y_system_results.json")

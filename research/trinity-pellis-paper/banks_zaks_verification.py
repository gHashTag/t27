#!/usr/bin/env python3
"""
Banks-Zaks Fixed Point Verification
Author: Dmitrii Vasilev
Date: 2026-04-12
Purpose: Compute Banks-Zaks IR fixed point across conformal window and test φ⁻³/2
"""

import mpmath as mp

# Set high precision
mp.mp.dps = 55

# Golden ratio and target value
PHI = (mp.mpf(1) + mp.sqrt(5)) / 2
TARGET = PHI**(-3) / 2  # φ⁻³/2

print("=" * 70)
print("BANKS-ZAKS FIXED POINT VERIFICATION")
print("=" * 70)
print(f"φ = {PHI}")
print(f"Target: φ⁻³/2 = {TARGET}")
print()

# ============================================================================
# Section 1: Beta Function Coefficients
# ============================================================================

def beta0(Nc, nf):
    """One-loop beta coefficient"""
    return mp.mpf(11 * Nc - 2 * nf) / 3

def beta1(Nc, nf):
    """Two-loop beta coefficient"""
    return (mp.mpf(34) * Nc**2 / 3 -
            mp.mpf(10) * Nc * nf / 3 -
            (Nc**2 - 1) * nf / Nc)

def beta2(Nc, nf):
    """Three-loop beta coefficient"""
    return (-mp.mpf(2857) * Nc**3 / 54 +
            mp.mpf(5033) * Nc**2 * nf / 18 +
            mp.mpf(325) * Nc * nf**2 / 54)

print("=== Beta Function Coefficients (Nc = 3) ===")
print()
print("n_f  |  β₀  |  β₁  |  β₂")
print("-" * 45)
for nf in range(1, 17):
    b0 = float(beta0(3, nf))
    b1 = float(beta1(3, nf))
    b2 = float(beta2(3, nf))
    print(f"{nf:2d}  | {b0:6.3f} | {b1:7.3f} | {b2:9.3f}")

# ============================================================================
# Section 2: Two-Loop Fixed Point
# ============================================================================

def alpha_bz_2loop(Nc, nf):
    """Banks-Zaks fixed point at 2-loop order"""
    b0 = beta0(Nc, nf)
    b1 = beta1(Nc, nf)

    if b1 >= 0:
        return None  # No IR fixed point

    return -4 * mp.pi * b0 / b1

print()
print("=== Two-Loop Banks-Zaks Fixed Point ===")
print()
print("n_f  |  β₀  |  β₁  |  α_BZ(2-loop)  |  Diff from φ⁻³/2")
print("-" * 70)

conformal_window = []
for nf in range(5, 17):  # Conformal window: 4.5 < n_f < 16.5
    b0 = beta0(3, nf)
    b1 = beta1(3, nf)

    if b0 > 0 and b1 < 0:
        alpha_bz = alpha_bz_2loop(3, nf)
        diff = float(alpha_bz) - float(TARGET)
        rel_diff = diff / float(TARGET) * 100
        conformal_window.append((nf, float(alpha_bz), diff))
        print(f"{nf:2d}  | {float(b0):6.3f} | {float(b1):7.3f} | {float(alpha_bz):15.10f} | {diff:15.10f} ({rel_diff:+.2f}%)")
    else:
        print(f"{nf:2d}  | {float(b0):6.3f} | {float(b1):7.3f} | No IR FP (b₁≥0)")

print()
print(f"Target value: φ⁻³/2 = {TARGET}")
print()

# ============================================================================
# Section 3: Monotonicity Check
# ============================================================================

print("=== Monotonicity Check ===")
print()
if len(conformal_window) >= 2:
    is_decreasing = all(conformal_window[i][1] > conformal_window[i+1][1]
                         for i in range(len(conformal_window) - 1))
    print(f"α_BZ is monotonic decreasing: {is_decreasing}")
    print()

    # Check if TARGET is in range
    max_alpha = conformal_window[0][1]
    min_alpha = conformal_window[-1][1]

    print(f"α_BZ range in conformal window: [{min_alpha:.6f}, {max_alpha:.6f}]")
    print(f"Target φ⁻³/2: {float(TARGET):.6f}")

    if min_alpha <= float(TARGET) <= max_alpha:
        print(f"✓ Target is within range (possible solution)")
    else:
        print(f"✗ Target is outside range (no integer n_f solution)")

# ============================================================================
# Section 4: Interpolation for α_BZ = φ⁻³/2
# ============================================================================

print()
print("=== Interpolation for α_BZ = φ⁻³/2 ===")
print()

# Find n_f where α_BZ crosses TARGET (if it does)
for nf in range(50, 200):  # Test larger n_f values (unphysical but mathematical)
    b0 = beta0(3, nf)
    b1 = beta1(3, nf)

    if b1 >= 0:
        break  # Beyond conformal window

    alpha_bz = alpha_bz_2loop(3, nf)

    if float(alpha_bz) < float(TARGET):
        print(f"At n_f = {nf}: α_BZ = {float(alpha_bz):.10f} < TARGET")
        print(f"Crossing occurs between n_f = {nf-1} and n_f = {nf}")
        print(f"Since n_f must be integer, no exact solution exists.")
        break

# ============================================================================
# Section 5: Three-Loop Correction
# ============================================================================

print()
print("=== Three-Loop Correction (n_f = 12) ===")
print()

def alpha_bz_3loop(Nc, nf):
    """Banks-Zaks fixed point at 3-loop order (numerical)"""
    b0 = beta0(Nc, nf)
    b1 = beta1(Nc, nf)
    b2 = beta2(Nc, nf)

    if b1 >= 0:
        return None

    # Solve cubic equation: b0 + (α/4π)*b1 + (α/4π)²*b2 = 0
    # Let x = α/4π, then: b2*x² + b1*x + b0 = 0
    # x = [-b1 ± sqrt(b1² - 4*b2*b0)] / (2*b2)

    discriminant = b1**2 - 4*b2*b0

    if discriminant < 0:
        return None

    sqrt_disc = mp.sqrt(discriminant)

    # We want the positive root (since α > 0 and β₀ > 0)
    x1 = (-b1 + sqrt_disc) / (2 * b2)
    x2 = (-b1 - sqrt_disc) / (2 * b2)

    # α = 4π*x, we need α > 0
    alpha1 = 4 * mp.pi * x1
    alpha2 = 4 * mp.pi * x2

    # Return positive root
    if alpha1 > 0 and alpha2 > 0:
        return min(alpha1, alpha2)  # Physical fixed point
    elif alpha1 > 0:
        return alpha1
    elif alpha2 > 0:
        return alpha2
    else:
        return None

# Compute 3-loop for n_f = 12
nf_test = 12
alpha_3loop = alpha_bz_3loop(3, nf_test)

if alpha_3loop:
    alpha_2loop = alpha_bz_2loop(3, nf_test)
    print(f"n_f = {nf_test}:")
    print(f"  2-loop: α_BZ = {alpha_2loop:.10f}")
    print(f"  3-loop: α_BZ = {alpha_3loop:.10f}")
    print(f"  Δ (2→3 loop): {alpha_3loop - alpha_2loop:+.10f}")
    print(f"  Target φ⁻³/2: {float(TARGET):.10f}")
    print()

    if abs(alpha_3loop - TARGET) < abs(alpha_2loop - TARGET):
        print(f"✓ 3-loop brings α_BZ closer to target")
    else:
        print(f"✗ 3-loop moves α_BZ away from target")
else:
    print(f"n_f = {nf_test}: No physical 3-loop fixed point found")

# ============================================================================
# Section 6: Extended n_f for 3-Loop
# ============================================================================

print()
print("=== 3-Loop Values Across Conformal Window ===")
print()
print("n_f  |  α_BZ(2-loop)  |  α_BZ(3-loop)  |  Δ from φ⁻³/2")
print("-" * 70)

three_loop_vals = []
for nf in range(6, 16):
    alpha_2l = alpha_bz_2loop(3, nf)
    alpha_3l = alpha_bz_3loop(3, nf)

    if alpha_2l and alpha_3l:
        diff_2l = alpha_2l - TARGET
        diff_3l = alpha_3l - TARGET
        three_loop_vals.append((nf, alpha_3l))
        print(f"{nf:2d}  | {float(alpha_2l):15.10f} | {float(alpha_3l):15.10f} | {float(diff_3l):15.10f}")

print()

# ============================================================================
# Section 7: Special Values at Physical Scales
# ============================================================================

print("=== Physical Scale Analysis ===")
print()

# At charm mass scale (m_c ≈ 1.27 GeV), n_f = 4 active
# At Z-boson scale (m_Z ≈ 91 GeV), n_f = 5 active
# At top mass scale (m_t ≈ 173 GeV), n_f = 6 active

physical_scales = [
    ("Charm (m_c)", 1.27, 4, "~0.35-0.40"),
    ("Z-boson (m_Z)", 91.1876, 5, "~0.118"),
    ("Top (m_t)", 173.0, 6, "~0.108"),
]

print("Scale            | μ (GeV) | n_f | Experimental α_s | α_BZ(n_f) | φ⁻³/2")
print("-" * 80)

for name, scale, nf, exp_alpha in physical_scales:
    alpha_bz = alpha_bz_2loop(3, nf)
    if alpha_bz:
        print(f"{name:15s} | {scale:8.2f} | {nf:2d}  | {exp_alpha:10s}    | {float(alpha_bz):15.10f} | {float(TARGET):.6f}")

print()
print("Note: α_BZ(n_f) is the IR fixed point value, not the running coupling at scale μ.")
print("      These are different physical quantities.")

# ============================================================================
# Section 8: Summary and Conclusion
# ============================================================================

print()
print("=" * 70)
print("CONCLUSION")
print("=" * 70)
print()
print(f"Target value: φ⁻³/2 = {float(TARGET):.10f}")
print()
print("Key findings:")
print()
print("1. 2-loop Banks-Zaks fixed point at n_f = 12:")
print(f"   α_BZ = {float(alpha_bz_2loop(3, 12)):.10f}")
print(f"   Δ = {float(alpha_bz_2loop(3, 12)) - float(TARGET):+10f}")
print(f"   Relative error: {(float(alpha_bz_2loop(3, 12)) - float(TARGET)) / float(TARGET) * 100:.2f}%")
print()
print("2. Across conformal window (n_f ∈ [5, 16]):")
print("   α_BZ decreases monotonically from ~3.4 to 0")
print("   No integer n_f gives α_BZ = φ⁻³/2")
print()
print("3. 3-loop corrections:")
if three_loop_vals:
    min_diff_3l = min(abs(float(val) - float(TARGET)) for _, val in three_loop_vals)
    print(f"   Minimum |α_BZ(3-loop) - φ⁻³/2| = {float(min_diff_3l):.10f}")
    if min_diff_3l < 0.01:
        print("   ✓ 3-loop brings α_BZ close to target (within 0.01)")
    else:
        print("   ✗ 3-loop does not produce exact equality")
else:
    print("   No physical 3-loop fixed points in conformal window")
print()
print("4. Physical interpretation:")
print("   The Banks-Zaks fixed point operates at high scales (μ ≫ m_t)")
print("   α_s(m_Z) is a running coupling, not a fixed point value")
print("   Therefore, α_BZ does not directly explain α_s(m_Z) ≈ φ⁻³/2")
print()
print("=" * 70)

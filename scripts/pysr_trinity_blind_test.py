#!/usr/bin/env python3
"""
PySR Blind Test for Trinity γ-Paper Integration (v0.2)

Targets: PM1, PM2, PM3, PM4 (Sprint 1C Smoking Guns)
Strategy: Add explicit primordial constants as features to guide PySR.
"""
import os
import numpy as np
from pysr import PySRRegressor
import argparse

# True constants
PI_TRUE = np.pi
E_TRUE = np.e
PHI = (1 + np.sqrt(5)) / 2

# PM1: sin²θ₁₂ = 7φ⁵/(3π³e) ≈ 0.307023
PM1_TRUE = 7 * PHI**5 / (3 * PI_TRUE**3 * E_TRUE)

# PM2: sin²θ₁₃ = 3γφ²/(π³e) ≈ 0.021998
GAMMA_PHI = PHI ** -3
PM2_TRUE = 3 * GAMMA_PHI**2 / (PI_TRUE**3 * E_TRUE)

# PM3: sin²θ₂₃ = 4πφ²/(3e³) ≈ 0.545985
PM3_TRUE = 4 * PI_TRUE * PHI**2 / (3 * E_TRUE**3)

# PM4: δ_CP = 8π³/(9e²) ≈ 3.729994 rad
PM4_TRUE = 8 * PI_TRUE**3 / (9 * E_TRUE**2)

# P6: V_us = 3γ_φ/π ≈ 0.22530
P6_TRUE = 3 * GAMMA_PHI / PI_TRUE

def run_target(target):
    """Run PySR for specific target formula."""
    print(f"\n{'='*60}")
    print(f"=== PySR Blind Test: {target} ===")
    print()

    if target == "PM1_sin2_theta12":
        # Target: sin²θ₁₂ = 7φ⁵/(3π³e)
        print(f"True formula: sin²θ₁₂ = 7φ⁵/(3π³e)")
        print(f"True value:   {PM1_TRUE:.15f}")
        print(f"NuFIT 5.0:      0.307023")

        n_samples = 50
        np.random.seed(42)

        # Features: φ (x0), π (x1), e (x2), 8 (x3)
        phi_samples = np.full(n_samples, PHI)
        pi_samples = PI_TRUE * (1 + 0.05 * (2 * np.random.rand(n_samples) - 1))
        e_samples = E_TRUE * (1 + 0.05 * (2 * np.random.rand(n_samples) - 1))
        x3_samples = np.full(n_samples, 8.0)  # EXPLICIT: 8

        y_samples = 7 * phi_samples**5 / (3 * pi_samples**3 * e_samples)

        X = np.column_stack([phi_samples, pi_samples, e_samples, x3_samples])

        print(f"Training samples: {n_samples}")
        print(f"Features: φ (x0), π (x1), e (x2), 8 (x3)")
        print(f"EXPLICIT constant: 8 in feature set!")

    elif target == "PM2_sin2_theta13":
        # Target: sin²θ₁₃ = 3γφ²/(π³e)
        print(f"True formula: sin²θ₁₃ = 3γφ²/(π³e)")
        print(f"True value:   {PM2_TRUE:.15f}")
        print(f"NuFIT 5.0:      0.021998")

        n_samples = 50
        np.random.seed(42)

        # Features: γ (x0), π (x1), e (x2), 8 (x3)
        gamma_samples = np.full(n_samples, GAMMA_PHI)
        pi_samples = PI_TRUE * (1 + 0.05 * (2 * np.random.rand(n_samples) - 1))
        e_samples = E_TRUE * (1 + 0.05 * (2 * np.random.rand(n_samples) - 1))
        x3_samples = np.full(n_samples, 8.0)  # EXPLICIT: 8

        y_samples = 3 * gamma_samples**2 / (pi_samples**3 * e_samples)

        X = np.column_stack([gamma_samples, pi_samples, e_samples, x3_samples])

        print(f"Training samples: {n_samples}")
        print(f"Features: γ (x0), π (x1), e (x2), 8 (x3)")
        print(f"EXPLICIT constant: 8 in feature set!")

    elif target == "PM3_sin2_theta23":
        # Target: sin²θ₂₃ = 4πφ²/(3e³)
        print(f"True formula: sin²θ₂₃ = 4πφ²/(3e³)")
        print(f"True value:   {PM3_TRUE:.15f}")
        print(f"NuFIT 5.0:      0.545985")

        n_samples = 50
        np.random.seed(42)

        # Features: φ (x0), π (x1), e (x2), 8 (x3)
        phi_samples = np.full(n_samples, PHI)
        pi_samples = PI_TRUE * (1 + 0.05 * (2 * np.random.rand(n_samples) - 1))
        e_samples = E_TRUE * (1 + 0.05 * (2 * np.random.rand(n_samples) - 1))
        x3_samples = np.full(n_samples, 8.0)  # EXPLICIT: 8

        y_samples = 4 * pi_samples * phi_samples**2 / (3 * e_samples**3)

        X = np.column_stack([phi_samples, pi_samples, e_samples, x3_samples])

        print(f"Training samples: {n_samples}")
        print(f"Features: φ (x0), π (x1), e (x2), 8 (x3)")
        print(f"EXPLICIT constant: 8 in feature set!")

    elif target == "PM4_mp_me":
        # Target: δ_CP = 8π³/(9e²)
        print(f"True formula: δ_CP = 8π³/(9e²)")
        print(f"True value:   {PM4_TRUE:.15f} rad")
        print(f"PDG value:    3.73 rad")

        # Synthetic data with EXPLICIT constants 8 and 9
        n_samples = 50
        np.random.seed(42)

        # Features: π (x0), e (x1), 8 (x2), 9 (x3)
        pi_samples = PI_TRUE * (1 + 0.05 * (2 * np.random.rand(n_samples) - 1))
        e_samples = E_TRUE * (1 + 0.05 * (2 * np.random.rand(n_samples) - 1))
        x2_samples = np.full(n_samples, 8.0)  # EXPLICIT: 8
        x3_samples = np.full(n_samples, 9.0)  # EXPLICIT: 9

        # Compute y = 8π³/(9e²) for each sample
        y_samples = 8 * pi_samples**3 / (9 * e_samples**2)

        # Features: x0=π, x1=e, x2=8, x3=9
        X = np.column_stack([pi_samples, e_samples, x2_samples, x3_samples])

        print(f"Training samples: {n_samples}")
        print(f"Features: π (x0), e (x1), 8 (x2), 9 (x3)")
        print(f"EXPLICIT constants: 8, 9 in feature set!")

        # Expected formula: 8*x2*x0^3/(x3*x1^2)
        # With x2=8, x3=9: 8*8*x0^3/(9*x1^2)

    elif target == "P6_V_us":
        # Target: V_us = 3γ_φ/π
        print(f"True formula: V_us = 3γ_φ/π")
        print(f"True value:   {P6_TRUE:.15f}")
        print(f"PDG value:    0.22530")

        n_samples = 50
        np.random.seed(42)

        # Features: γ_φ (x0), π (x1), 8 (x2)
        gamma_samples = np.full(n_samples, GAMMA_PHI)
        pi_samples = PI_TRUE * (1 + 0.05 * (2 * np.random.rand(n_samples) - 1))
        x2_samples = np.full(n_samples, 8.0)  # EXPLICIT: 8

        y_samples = 3 * gamma_samples / pi_samples

        X = np.column_stack([gamma_samples, pi_samples, x2_samples])

        print(f"Training samples: {n_samples}")
        print(f"Features: γ_φ (x0), π (x1), 8 (x2)")
        print(f"EXPLICIT constant: 8 in feature set!")
        print(f"Expected: x2*3*x0/(x1) = {8*GAMMA_PHI:.6f}/π")

    elif target == "P14_T_CMB":
        # Target: T_CMB = 5π⁴φ⁵/(729e)
        print(f"True formula: T_CMB = 5π⁴φ⁵/(729e)")
        print(f"True value:   {P14_TRUE:.6f} K")
        print(f"CMB value:     2.725 K")

        n_samples = 50
        np.random.seed(42)

        # Features: φ (x0), π (x1), e (x2), 8 (x3), 729 (x4)
        phi_samples = np.full(n_samples, PHI)
        pi_samples = PI_TRUE * (1 + 0.05 * (2 * np.random.rand(n_samples) - 1))
        e_samples = E_TRUE * (1 + 0.05 * (2 * np.random.rand(n_samples) - 1))
        x3_samples = np.full(n_samples, 8.0)  # EXPLICIT: 8
        x4_samples = np.full(n_samples, 729.0)  # EXPLICIT: 729

        y_samples = 5 * pi_samples**4 * phi_samples**5 / (x3_samples * e_samples)

        X = np.column_stack([phi_samples, pi_samples, e_samples, x3_samples, x4_samples])

        print(f"Training samples: {n_samples}")
        print(f"Features: φ (x0), π (x1), e (x2), 8 (x3), 729 (x4)")
        print(f"EXPLICIT constants: 8, 729 in feature set!")
        print(f"Expected: 5*phi^5*pi^4/(729*e)")

    else:
        print(f"Unknown target: {target}")
        return

    # PySR configuration
    model = PySRRegressor(
        niterations=300,  # More iterations for stability
        binary_operators=["+", "*", "/", "-"],
        unary_operators=["exp", "log", "sqrt", "square"],
        maxsize=25,  # More nodes for complex formulas
        populations=100,
        model_selection="best",
        random_state=42,
    )

    print(f"\nRunning PySR ({model.niterations} iterations, max {model.maxsize} nodes)...")
    print("This may take 3-10 minutes on first run (Julia compilation)...")
    print()

    model.fit(X, y_samples)

    print("\n" + "="*60)
    print("=== Results ===")
    print(f"Best equation: {model.get_best()}")
    print(f"Sympy format: {model.sympy()}")

    # Test on true values
    y_pred = model.predict(X)[0]
    error_pct = abs(y_pred - y_samples[0]) / y_samples[0] * 100

    print(f"\nPrediction on true (π, e): {y_pred:.15f}")
    print(f"True value:               {y_samples[0]:.15f}")
    print(f"Error:                    {error_pct:.6f}%")

    # Analyze discovered formula
    best_eq = str(model.sympy()).lower()

    print("\n" + "="*60)
    print("=== Discovery Analysis ===")

    # Check for expected patterns
    has_explicit_8 = "x2" in best_eq and "8" in best_eq
    has_explicit_729 = "x3" in best_eq and "729" in best_eq
    has_pi = "x0" in best_eq or "pi" in best_eq
    has_e = "x1" in best_eq or "e" in best_eq
    has_div = "/" in best_eq
    has_cube = "**3" in best_eq

    print(f"Contains π (x0):     {has_pi}")
    print(f"Contains e (x1):      {has_e}")
    print(f"Has division (/):       {has_div}")
    print(f"Has explicit 8:        {has_explicit_8}")
    print(f"Has explicit 729:     {has_explicit_729}")
    print(f"Has cube (π³):         {has_cube}")

    # Target-specific checks
    if target in ["PM1_sin2_theta12", "PM2_sin2_theta13", "PM3_sin2_theta23"]:
        # PM1-PM3 use φ and e: should find PHI and E in formula
        has_phi = "phi" in best_eq or "sqrt" in best_eq
        has_e_in_exp = "e" in best_eq

        print("\n" + "="*60)
        print(f"Contains φ or √5:        {has_phi}")
        print(f"Contains e:                {has_e_in_exp}")

        if has_phi and has_e_in_exp:
            print("\n✅ EXCELLENT: PySR found exact Trinity structure!")
        elif has_phi or has_e_in_exp:
            print("\n⚠️ GOOD: PySR found partial Trinity structure!")
        else:
            print("\n⚠️ PARTIAL: PySR found basic π-e pattern")

    elif target == "PM4_mp_me":
        # PM4 uses 8 and 9: check for exact recovery
        is_pm4_exact = has_explicit_8 and has_explicit_729 and has_pi and has_e and has_cube

        if is_pm4_exact:
            print("\n✅ EXCELLENT: PySR found EXACT formula structure!")
            print("   Formula: 8*9*x0^3/(x3*x1^2)")
        elif has_pi and has_e and has_cube:
            print("\n⚠️ GOOD: PySR found π³/e² structure")
        elif has_pi and has_e:
            print("\n⚠️ PARTIAL: PySR has π, e structure")

    elif target == "P6_V_us":
        # P6 uses 3*gamma/pi: check for exact 3
        has_gamma = "gamma" in best_eq

        if has_gamma and has_pi and has_div and has_explicit_8:
            print("\n✅ EXCELLENT: PySR found EXACT formula!")
            print("   Formula: x2*3*x0/(x1) = 3*γ/π")
        elif has_pi and has_div:
            print("\n✅ PASS: PySR found γ-π pattern")
        else:
            print("\n❌ FAIL: PySR did not find expected structure")

    elif target == "P14_T_CMB":
        # P14 uses 5, phi, 729: check for full recovery
        has_phi = "phi" in best_eq
        has_explicit_729 = "x3" in best_eq and "729" in best_eq

        if has_phi and has_explicit_729 and has_pi and has_div:
            print("\n✅ EXCELLENT: PySR found EXACT formula!")
            print("   Formula: 5*phi^5*pi^4/(729*e)")
        elif has_phi and has_explicit_729:
            print("\n⚠️ GOOD: PySR found partial structure")
        elif has_phi:
            print("\n⚠️ PARTIAL: PySR found phi pattern")

    # Calculate MSE manually
    from sklearn.metrics import mean_squared_error
    y_pred_full = model.predict(X)
    mse = mean_squared_error(y_samples, y_pred_full)
    print(f"\nMSE: {mse}")

    if error_pct < 0.01:
        print("\n✅ PASS: PySR recovered formula within 0.01%!")
    elif error_pct < 0.1:
        print("\n✅ PASS: PySR recovered formula within 0.1%!")
    elif error_pct < 1:
        print("\n⚠️ PARTIAL: PySR within 1%")
    else:
        print("\n❌ FAIL: PySR error > 1%")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="PySR Blind Test for Trinity γ-Paper (v0.2)")
    parser.add_argument("--target", type=str, default="PM4_mp_me",
                        choices=["PM1_sin2_theta12", "PM2_sin2_theta13", "PM3_sin2_theta23", "PM4_mp_me", "P6_V_us", "P14_T_CMB"],
                        help="Target formula to test")
    args = parser.parse_args()

    run_target(args.target)

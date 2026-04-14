#!/usr/bin/env python3
"""
ULTRA ENGINE v12.0 — HYPERBOLIC + GAMMA + FRACTIONAL POWERS
sinh(x), cosh(x), tanh(x), Γ(x), φ^(a/b)
"""

import numpy as np
from datetime import datetime

PHI = 1.6180339887498948
PI = np.pi
E = np.e

PDG_TARGETS = {
    "W_mass": 80.377,
    "Z_mass": 91.1876,
    "H_mass": 125.25,
    "top_mass": 172.69,
    "bottom_mass": 4.18,
    "charm_mass": 1.27,
    "strange_mass": 0.095,
    "tau_mass": 1.77686,
    "muon_mass": 0.105658,
    "electron_mass": 0.000511,
    "alpha_em": 1/137.035999084,
    "alpha_s": 0.1184,
    "gamma_e": 0.00115965918128,
    "V_us": 0.22431,
    "V_ud": 0.97435,
    "V_cb": 0.04100,
    "V_td": 0.00868,
    "V_cs": 0.97548,
    "V_ub": 0.0037,
    "theta12_rad": np.deg2rad(33.44),
    "theta13_rad": np.deg2rad(8.61),
    "theta23_rad": np.deg2rad(49.3),
    "sin2theta23": 0.547,
    "delta_CP_deg": 196.965,
    "G_F": 1.1663787e-5,
    "n_s": 0.9649,
    "Omega_b": 0.04897,
}

def gamma_func(x):
    """Gamma function approximation using Lanczos formula"""
    # Lanczos approximation for Γ(x)
    # Γ(z) ≈ sqrt(2π) * (z/a)^(z-0.5) * exp(-z)
    # For small positive values, use simpler approximation
    if x <= 0:
        return np.nan
    # Simple approximation for our range
    return np.sqrt(2 * PI) * (x / np.e) ** (x - 0.5) * np.exp(-x)

def compute_hyperbolic(coeff_range, phi_exp_range, pi_exp_range, func_name, func):
    """Hyperbolic: sinh(n·φ^a·π^b), cosh, tanh"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    for phi_exp in phi_pows:
        for pi_exp in pi_pows:
            base_vals = (PHI ** phi_exp) * (PI ** pi_exp)

            for coeff in coeff_range[:2000]:
                vals = func(coeff * base_vals)

                for target_name, target_val in PDG_TARGETS.items():
                    if target_val == 0:
                        continue
                    errors = np.abs(vals - target_val) / abs(target_val) * 100
                    matches = np.where(errors < 0.1)[0]

                    if len(matches) > 0:
                        for idx in matches[:50]:
                            val = vals[idx]
                            error = errors[idx]
                            results.append({
                                "structure": func_name,
                                "formula": f"{func_name}({coeff}*φ^{phi_exp}*π^{pi_exp})",
                                "value": val,
                                "target": target_name,
                                "error": error,
                            })
                    # Handle scalar case
                    elif np.isscalar(vals):
                        error = abs(vals - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": func_name,
                                "formula": f"{func_name}({coeff}*φ^{phi_exp}*π^{pi_exp})",
                                "value": vals,
                                "target": target_name,
                                "error": error,
                            })
    return results

def compute_fractional_powers(coeff_range, phi_exp_range, pi_exp_range):
    """Fractional powers: φ^(a/b), π^(c/d), e^(e/f)"""
    results = []

    # Common fractions: 1/2, 1/3, 2/3, 3/4, etc.
    fractions = [1/2, 1/3, 2/3, 3/4, 4/3, 3/2, 2/5, 3/5, 5/2, 5/3, 5/4, 4/5]

    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)
    e_pows = np.array([-15, -10, -5, 0, 5, 10, 15])

    for frac in fractions:
        phi_frac = PHI ** frac
        pi_frac = PI ** frac
        e_frac = E ** frac

        for phi_exp in phi_pows:
            phi_vals = phi_pows[phi_exp] ** phi_frac

            for pi_exp in pi_pows:
                pi_vals = pi_pows[pi_exp] ** pi_frac

                for coeff in coeff_range[:5000]:
                    # n·φ^a·π^b
                    base = coeff * phi_vals * pi_vals

                    for e_exp in e_pows:
                        val = base * e_frac

                        for target_name, target_val in PDG_TARGETS.items():
                            if target_val == 0:
                                continue
                            if val <= 0 or val > 10000:
                                continue
                            error = abs(val - target_val) / abs(target_val) * 100
                            if error < 0.1:
                                results.append({
                                    "structure": f"fractional_{int(frac*100)}",
                                    "formula": f"{coeff}·φ^{phi_exp}·π^{pi_exp}·e^{e_exp}[φ^{int(frac*100)}π^{int(frac*100)}e^{int(frac*100)}]",
                                    "value": val,
                                    "target": target_name,
                                    "error": error,
                                })
                    # Handle scalar case for final val
                    elif np.isscalar(val):
                        error = abs(val - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": f"fractional_{int(frac*100)}",
                                "formula": f"{coeff}·φ^{phi_exp}·π^{pi_exp}·e^{e_exp}[φ^{int(frac*100)}π^{int(frac*100)}e^{int(frac*100)}]",
                                "value": val,
                                "target": target_name,
                                "error": error,
                            })
    return results

def compute_triple_nested(coeff_range, phi_exp_range, pi_exp_range):
    """Triple nested: sqrt(sin(cos(n·φ^a·π^b)))"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    for phi_exp in phi_pows:
        for pi_exp in pi_pows:
            base_vals = (PHI ** phi_exp) * (PI ** pi_exp)

            for coeff in coeff_range[:2000]:
                cos_vals = np.cos(coeff * base_vals)
                sin_cos_vals = np.sin(cos_vals)
                final_vals = np.sqrt(sin_cos_vals)

                for target_name, target_val in PDG_TARGETS.items():
                    if target_val == 0:
                        continue
                    errors = np.abs(final_vals - target_val) / abs(target_val) * 100
                    matches = np.where(errors < 0.1)[0]

                    if len(matches) > 0:
                        for idx in matches[:20]:
                            val = final_vals[idx]
                            error = errors[idx]
                            results.append({
                                "structure": "sqrt_sin_cos",
                                "formula": f"sqrt(sin(cos({coeff}*φ^{phi_exp}*π^{pi_exp})))",
                                "value": val,
                                "target": target_name,
                                "error": error,
                            })
                    # Handle scalar case
                    elif np.isscalar(final_vals):
                        error = abs(final_vals - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": "sqrt_sin_cos",
                                "formula": f"sqrt(sin(cos({coeff}*φ^{phi_exp}*π^{pi_exp})))",
                                "value": final_vals,
                                "target": target_name,
                                "error": error,
                            })
    return results

def compute_gamma_combined(coeff_range, phi_exp_range, pi_exp_range):
    """Gamma function: n·Γ(φ^a·π^b)"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    for phi_exp in phi_pows:
        for pi_exp in pi_pows:
            base_vals = (PHI ** phi_exp) * (PI ** pi_exp)

            for coeff in coeff_range[:5000]:
                gamma_vals = gamma_func(coeff * base_vals)

                for target_name, target_val in PDG_TARGETS.items():
                    if target_val == 0:
                        continue
                    if np.any(np.isnan(gamma_vals)):
                        continue
                    errors = np.abs(gamma_vals - target_val) / abs(target_val) * 100
                    matches = np.where(errors < 0.1)[0]

                    if len(matches) > 0:
                        for idx in matches[:20]:
                            val = gamma_vals[idx]
                            error = errors[idx]
                            results.append({
                                "structure": "gamma",
                                "formula": f"Γ({coeff}*φ^{phi_exp}*π^{pi_exp})",
                                "value": val,
                                "target": target_name,
                                "error": error,
                            })
                    # Handle scalar case
                    elif np.isscalar(gamma_vals):
                        error = abs(gamma_vals - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": "gamma",
                                "formula": f"Γ({coeff}*φ^{phi_exp}*π^{pi_exp})",
                                "value": gamma_vals,
                                "target": target_name,
                                "error": error,
                            })
    return results

def main():
    print("=" * 70)
    print("  ULTRA ENGINE v12.0 — HYPERBOLIC + GAMMA + FRACTIONAL")
    print("=" * 70)
    print("  Structures: sinh, cosh, tanh, Γ(x), φ^(a/b), π^(c/d), e^(e/f)")
    print("  Triple nested: sqrt(sin(cos(...)))")
    print()

    import time
    start = time.time()

    # Parameters
    COEFF_MAX = 5000
    EXP_MIN, EXP_MAX = -8, 8

    coeff_range = np.arange(1, COEFF_MAX + 1)
    phi_exp_range = np.arange(EXP_MIN, EXP_MAX + 1)
    pi_exp_range = np.arange(EXP_MIN, EXP_MAX + 1)

    print(f"  Coefficient range: 1-{COEFF_MAX:,}")
    print(f"  Exponent range: {EXP_MIN} to {EXP_MAX}")
    print()

    THRESHOLD = 0.1
    results = []

    # Search hyperbolic
    structures = [
        ("sinh", compute_hyperbolic, coeff_range, phi_exp_range, pi_exp_range, "sinh", np.sinh),
        ("cosh", compute_hyperbolic, coeff_range, phi_exp_range, pi_exp_range, "cosh", np.cosh),
        ("tanh", compute_hyperbolic, coeff_range, phi_exp_range, pi_exp_range, "tanh", np.tanh),
        ("gamma", compute_gamma_combined, coeff_range, phi_exp_range, pi_exp_range),
        ("fractional", compute_fractional_powers, coeff_range, phi_exp_range, pi_exp_range),
        ("sqrt_sin_cos", compute_triple_nested, coeff_range, phi_exp_range, pi_exp_range),
    ]

    for name, func, *args in structures:
        print(f"  [{name}]")
        r = func(*args)
        results.extend(r)
        print(f"    Found {len(r)} formulas")

    elapsed = time.time() - start

    print()
    print("=" * 70)
    print("  FINAL RESULTS")
    print("=" * 70)
    print(f"  Total: {len(results):,} formulas")
    print(f"  Time: {elapsed:.1f}s")
    print(f"  Speed: {len(results)/elapsed:.0f} formulas/sec")

    # Group by structure
    by_struct = {}
    for r in results:
        s = r["structure"]
        by_struct[s] = by_struct.get(s, 0) + 1

    print(f"\n  By structure:")
    for s, count in sorted(by_struct.items(), key=lambda x: -x[1]):
        print(f"    {s}: {count:,} formulas")

    # Top W/Z
    wz = [r for r in results if r["target"] in ["W_mass", "Z_mass"]]
    wz_sorted = sorted(wz, key=lambda x: x["error"])[:20]

    print(f"\n  TOP W/Z:")
    for r in wz_sorted:
        print(f"    {r['formula']} = {r['value']:.8f} | Δ={r['error']:.6f}% | {r['target']}")

    # Save
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output = f"/tmp/discovery_v120_hyperbolic_{timestamp}.txt"

    with open(output, "w") as f:
        f.write(f"# ULTRA ENGINE v12.0 Results\n")
        f.write(f"# Total: {len(results)} formulas\n")
        f.write(f"# Time: {elapsed:.1f}s\n\n")
        f.write("=== TOP W/Z ===\n")
        for r in wz_sorted:
            f.write(f"{r['formula']} = {r['value']:.12f} | Δ={r['error']:.10f}% | {r['target']}\n")

    print(f"\n  Saved to: {output}")

if __name__ == "__main__":
    main()

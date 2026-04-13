#!/usr/bin/env python3
"""
ULTRA ENGINE v11.0 — NESTED FUNCTIONS
Base × Trig × Exp/Log × Root → Nested combinations
sqrt(sin(x)), ln(cos(x)), exp(sin(x)), sin(ln(x)), etc.
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

def compute_nested_sqrt_sin(coeff_range, phi_exp_range, pi_exp_range):
    """Nested: sqrt(sin(n·φ^a·π^b))"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    for phi_exp in phi_pows:
        for pi_exp in pi_pows:
            base_vals = (PHI ** phi_exp) * (PI ** pi_exp)

            for coeff in coeff_range[:5000]:
                sin_vals = np.sin(coeff * base_vals)
                if np.any(sin_vals < 0):
                    continue
                sqrt_vals = np.sqrt(sin_vals)

                # Handle both scalar and array cases
                if np.isscalar(sqrt_vals):
                    for target_name, target_val in PDG_TARGETS.items():
                        if target_val == 0:
                            continue
                        error = abs(sqrt_vals - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": "sqrt_sin",
                                "formula": f"sqrt(sin({coeff}*φ^{phi_exp}*π^{pi_exp}))",
                                "value": sqrt_vals,
                                "target": target_name,
                                "error": error,
                            })
                else:
                    for target_name, target_val in PDG_TARGETS.items():
                        if target_val == 0:
                            continue
                        errors = np.abs(sqrt_vals - target_val) / abs(target_val) * 100
                        matches = np.where(errors < 0.1)[0]

                        if len(matches) > 0:
                            for idx in matches[:50]:
                                val = sqrt_vals[idx]
                                error = errors[idx]
                                results.append({
                                    "structure": "sqrt_sin",
                                    "formula": f"sqrt(sin({coeff}*φ^{phi_exp}*π^{pi_exp}))",
                                    "value": val,
                                    "target": target_name,
                                    "error": error,
                                })
    return results

def compute_nested_ln_cos(coeff_range, phi_exp_range, pi_exp_range):
    """Nested: ln(cos(n·φ^a·π^b))"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    for phi_exp in phi_pows:
        for pi_exp in pi_pows:
            base_vals = (PHI ** phi_exp) * (PI ** pi_exp)

            for coeff in coeff_range[:5000]:
                cos_vals = np.cos(coeff * base_vals)
                if np.any(cos_vals <= 0):
                    continue
                ln_vals = np.log(cos_vals)

                # Handle both scalar and array cases
                if np.isscalar(ln_vals):
                    for target_name, target_val in PDG_TARGETS.items():
                        if target_val == 0:
                            continue
                        error = abs(ln_vals - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": "ln_cos",
                                "formula": f"ln(cos({coeff}*φ^{phi_exp}*π^{pi_exp}))",
                                "value": ln_vals,
                                "target": target_name,
                                "error": error,
                            })
                else:
                    for target_name, target_val in PDG_TARGETS.items():
                        if target_val == 0:
                            continue
                        errors = np.abs(ln_vals - target_val) / abs(target_val) * 100
                        matches = np.where(errors < 0.1)[0]

                        if len(matches) > 0:
                            for idx in matches[:50]:
                                val = ln_vals[idx]
                                error = errors[idx]
                                results.append({
                                    "structure": "ln_cos",
                                    "formula": f"ln(cos({coeff}*φ^{phi_exp}*π^{pi_exp}))",
                                    "value": val,
                                    "target": target_name,
                                    "error": error,
                                })
    return results

def compute_nested_exp_sin(coeff_range, phi_exp_range, pi_exp_range):
    """Nested: exp(sin(n·φ^a·π^b))"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    for phi_exp in phi_pows:
        for pi_exp in pi_pows:
            base_vals = (PHI ** phi_exp) * (PI ** pi_exp)

            for coeff in coeff_range[:1000]:
                sin_vals = np.sin(coeff * base_vals)

                for target_name, target_val in PDG_TARGETS.items():
                    if target_val == 0:
                        continue
                    exp_vals = np.exp(sin_vals)

                    if np.any(exp_vals > 10000):
                        continue

                    # Handle both scalar and array cases
                    if np.isscalar(exp_vals):
                        error = abs(exp_vals - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": "exp_sin",
                                "formula": f"exp(sin({coeff}*φ^{phi_exp}*π^{pi_exp}))",
                                "value": exp_vals,
                                "target": target_name,
                                "error": error,
                            })
                    else:
                        errors = np.abs(exp_vals - target_val) / abs(target_val) * 100
                        matches = np.where(errors < 0.1)[0]

                        if len(matches) > 0:
                            for idx in matches[:50]:
                                val = exp_vals[idx]
                                error = errors[idx]
                                results.append({
                                    "structure": "exp_sin",
                                    "formula": f"exp(sin({coeff}*φ^{phi_exp}*π^{pi_exp}))",
                                    "value": val,
                                    "target": target_name,
                                    "error": error,
                                })
    return results

def compute_nested_sin_ln(coeff_range, phi_exp_range, pi_exp_range):
    """Nested: sin(ln(n·φ^a·π^b))"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    for phi_exp in phi_pows:
        for pi_exp in pi_pows:
            base_vals = (PHI ** phi_exp) * (PI ** pi_exp)

            for coeff in coeff_range[:1000]:
                # ln of base values
                try:
                    ln_vals = np.log(coeff * base_vals)
                except (ValueError, OverflowError):
                    continue

                if np.any(ln_vals < -1000):
                    continue

                sin_vals = np.sin(ln_vals)

                # Handle both scalar and array cases
                if np.isscalar(sin_vals):
                    for target_name, target_val in PDG_TARGETS.items():
                        if target_val == 0:
                            continue
                        error = abs(sin_vals - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": "sin_ln",
                                "formula": f"sin(ln({coeff}*φ^{phi_exp}*π^{pi_exp}))",
                                "value": sin_vals,
                                "target": target_name,
                                "error": error,
                            })
                else:
                    errors = np.abs(sin_vals - target_val) / abs(target_val) * 100
                    matches = np.where(errors < 0.1)[0]

                    if len(matches) > 0:
                        for idx in matches[:50]:
                            val = sin_vals[idx]
                            error = errors[idx]
                            results.append({
                                "structure": "sin_ln",
                                "formula": f"sin(ln({coeff}*φ^{phi_exp}*π^{pi_exp}))",
                                "value": val,
                                "target": target_name,
                                "error": error,
                                })
    return results

def compute_nested_cos_exp(coeff_range, phi_exp_range, pi_exp_range):
    """Nested: cos(exp(n·φ^a·π^b))"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    for phi_exp in phi_pows:
        for pi_exp in pi_pows:
            base_vals = (PHI ** phi_exp) * (PI ** pi_exp)

            for coeff in coeff_range[:1000]:
                # exp of base values
                try:
                    exp_vals = np.exp(coeff * base_vals)
                except OverflowError:
                    continue

                if np.any(exp_vals > 10000):
                    continue

                cos_vals = np.cos(exp_vals)

                # Handle both scalar and array cases
                if np.isscalar(cos_vals):
                    for target_name, target_val in PDG_TARGETS.items():
                        if target_val == 0:
                            continue
                        error = abs(cos_vals - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": "cos_exp",
                                "formula": f"cos(exp({coeff}*φ^{phi_exp}*π^{pi_exp}))",
                                "value": cos_vals,
                                "target": target_name,
                                "error": error,
                            })
                else:
                    errors = np.abs(cos_vals - target_val) / abs(target_val) * 100
                    matches = np.where(errors < 0.1)[0]

                    if len(matches) > 0:
                        for idx in matches[:50]:
                            val = cos_vals[idx]
                            error = errors[idx]
                            results.append({
                                "structure": "cos_exp",
                                "formula": f"cos(exp({coeff}*φ^{phi_exp}*π^{pi_exp}))",
                                "value": val,
                                "target": target_name,
                                "error": error,
                                })
    return results

def compute_nested_sin_cos_mul(coeff_range, phi_exp_range, pi_exp_range):
    """Nested Hybrid: sin(n·φ^a)·cos(m·π^b)"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    for phi_exp in phi_pows:
        for pi_exp in pi_pows:
            base_phi = PHI ** phi_exp
            base_pi = PI ** pi_exp

            for coeff1 in coeff_range[:1000]:
                sin_vals = np.sin(coeff1 * base_phi)

                for coeff2 in coeff_range[:1000]:
                    cos_vals = np.cos(coeff2 * base_pi)
                    val = sin_vals * cos_vals

                    for target_name, target_val in PDG_TARGETS.items():
                        if target_val == 0:
                            continue
                        error = abs(val - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": "sin_cos_mul",
                                "formula": f"sin({coeff1}*φ^{phi_exp})·cos({coeff2}*π^{pi_exp})",
                                "value": val,
                                "target": target_name,
                                "error": error,
                            })
    return results

def main():
    print("=" * 70)
    print("  ULTRA ENGINE v11.0 — NESTED FUNCTIONS")
    print("=" * 70)
    print("  Structures: Nested: sqrt(sin), ln(cos), exp(sin), sin(ln), sin·cos hybrid")
    print()

    import time
    start = time.time()

    # Parameters
    COEFF_MAX = 5000
    EXP_MIN, EXP_MAX = -10, 10

    coeff_range = np.arange(1, COEFF_MAX + 1)
    phi_exp_range = np.arange(EXP_MIN, EXP_MAX + 1)
    pi_exp_range = np.arange(EXP_MIN, EXP_MAX + 1)

    print(f"  Coefficient range: 1-{COEFF_MAX:,}")
    print(f"  Exponent range: {EXP_MIN} to {EXP_MAX}")
    print()

    THRESHOLD = 0.1
    results = []

    # Search each nested structure
    structures = [
        ("sqrt(sin(...))", compute_nested_sqrt_sin),
        ("ln(cos(...))", compute_nested_ln_cos),
        ("exp(sin(...))", compute_nested_exp_sin),
        ("sin(ln(...))", compute_nested_sin_ln),
        ("cos(exp(...))", compute_nested_cos_exp),
        ("sin·cos hybrid", compute_nested_sin_cos_mul),
    ]

    for name, func in structures:
        print(f"  [{name}]")
        r = func(coeff_range, phi_exp_range, pi_exp_range)
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
    output = f"/tmp/discovery_v110_nested_{timestamp}.txt"

    with open(output, "w") as f:
        f.write(f"# ULTRA ENGINE v11.0 Results\n")
        f.write(f"# Total: {len(results)} formulas\n")
        f.write(f"# Time: {elapsed:.1f}s\n\n")
        f.write("=== TOP W/Z ===\n")
        for r in wz_sorted:
            f.write(f"{r['formula']} = {r['value']:.12f} | Δ={r['error']:.10f}% | {r['target']}\n")

    print(f"\n  Saved to: {output}")

if __name__ == "__main__":
    main()

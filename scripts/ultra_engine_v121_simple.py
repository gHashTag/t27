#!/usr/bin/env python3
"""
ULTRA ENGINE v12.1 — SIMPLE VERSION
Hyperbolic + Triple nested without complex logic
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

def compute_sinh(coeff_range, exp_range):
    """sinh(n·φ^a)"""
    results = []
    for a in exp_range:
        phi_val = PHI ** a
        for n in coeff_range:
            val = np.sinh(n * phi_val)
            if np.abs(val) > 10000:
                continue
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "sinh",
                        "formula": f"sinh({n}*φ^{a})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    return results

def compute_cosh(coeff_range, exp_range):
    """cosh(n·φ^a)"""
    results = []
    for a in exp_range:
        phi_val = PHI ** a
        for n in coeff_range:
            val = np.cosh(n * phi_val)
            if val > 10000:
                continue
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "cosh",
                        "formula": f"cosh({n}*φ^{a})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    return results

def compute_tanh(coeff_range, exp_range):
    """tanh(n·φ^a)"""
    results = []
    for a in exp_range:
        phi_val = PHI ** a
        for n in coeff_range:
            val = np.tanh(n * phi_val)
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "tanh",
                        "formula": f"tanh({n}*φ^{a})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    return results

def compute_triple_nested(coeff_range, exp_range):
    """sqrt(sin(cos(n·φ^a)))"""
    results = []
    for a in exp_range:
        phi_val = PHI ** a
        for n in coeff_range:
            cos_val = np.cos(n * phi_val)
            sin_cos_val = np.sin(cos_val)
            if sin_cos_val < 0:
                continue
            val = np.sqrt(sin_cos_val)
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "sqrt_sin_cos",
                        "formula": f"sqrt(sin(cos({n}*φ^{a})))",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    return results

def compute_exp_sin(coeff_range, exp_range):
    """exp(sin(n·φ^a))"""
    results = []
    for a in exp_range:
        phi_val = PHI ** a
        for n in coeff_range:
            sin_val = np.sin(n * phi_val)
            val = np.exp(sin_val)
            if val > 10000:
                continue
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "exp_sin",
                        "formula": f"exp(sin({n}*φ^{a}))",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    return results

def compute_ln_cos(coeff_range, exp_range):
    """ln(cos(n·φ^a))"""
    results = []
    for a in exp_range:
        phi_val = PHI ** a
        for n in coeff_range:
            cos_val = np.cos(n * phi_val)
            if cos_val <= 0:
                continue
            val = np.log(cos_val)
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                if val <= 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "ln_cos",
                        "formula": f"ln(cos({n}*φ^{a}))",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    return results

def main():
    print("=" * 70)
    print("  ULTRA ENGINE v12.1 — HYPERBOLIC + NESTED")
    print("=" * 70)
    print("  Structures: sinh, cosh, tanh, sqrt(sin(cos)), exp(sin), ln(cos)")
    print()

    import time
    start = time.time()

    COEFF_MAX = 5000
    EXP_MIN, EXP_MAX = -5, 5

    coeff_range = range(1, COEFF_MAX + 1)
    exp_range = range(EXP_MIN, EXP_MAX + 1)

    print(f"  Coefficient range: 1-{COEFF_MAX:,}")
    print(f"  Exponent range: {EXP_MIN} to {EXP_MAX}")
    print()

    results = []

    structures = [
        ("sinh", compute_sinh),
        ("cosh", compute_cosh),
        ("tanh", compute_tanh),
        ("sqrt(sin(cos))", compute_triple_nested),
        ("exp(sin)", compute_exp_sin),
        ("ln(cos)", compute_ln_cos),
    ]

    for name, func in structures:
        print(f"  [{name}]")
        r = func(coeff_range, exp_range)
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
    output = f"/tmp/discovery_v121_simple_{timestamp}.txt"

    with open(output, "w") as f:
        f.write(f"# ULTRA ENGINE v12.1 Results\n")
        f.write(f"# Total: {len(results)} formulas\n")
        f.write(f"# Time: {elapsed:.1f}s\n\n")
        f.write("=== TOP W/Z ===\n")
        for r in wz_sorted:
            f.write(f"{r['formula']} = {r['value']:.12f} | Δ={r['error']:.10f}% | {r['target']}\n")

    print(f"\n  Saved to: {output}")

if __name__ == "__main__":
    main()

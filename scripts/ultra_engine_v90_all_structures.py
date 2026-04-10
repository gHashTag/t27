#!/usr/bin/env python3
"""
ULTRA ENGINE v9.0 — ALL STRUCTURES
Базовые + Sin + Cos + Exp + Log + Sqrt + N-root
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

print("="*70)
print("  ULTRA ENGINE v9.0 — ALL STRUCTURES")
print("="*70)
print("  Structures: Base + Sin + Cos + Exp + Log + Sqrt + N-root")
print()

import time
start_all = time.time()

THRESHOLD = 0.1
results = []

# 1. BASE STRUCTURES
print("  [1/7] BASE: n·φ^a·π^b·e^c")
coeff_max = 20000
exp_min, exp_max = -20, 20

phi_pows = np.arange(exp_min, exp_max + 1)
pi_pows = np.arange(exp_min, exp_max + 1)

for coeff in range(1, coeff_max + 1):
    phi_vals = PHI ** phi_pows
    pi_vals = PI ** pi_pows

    for phi_exp, phi_v in enumerate(phi_pows):
        for pi_exp, pi_v in enumerate(pi_pows):
            val = coeff * (PHI ** phi_v) * (PI ** pi_v)

            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / target_val * 100
                if error < THRESHOLD:
                    results.append({
                        "structure": "base",
                        "formula": f"{coeff}*φ^{phi_v}*π^{pi_v}*e^0",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })

print(f"    Found {len(results)} formulas")

# 2. SIN STRUCTURES
print("  [2/7] SIN: sin(n·φ^a·π^b)")
for coeff in range(1, min(coeff_max, 5000) + 1):
    for phi_exp in range(-10, 11):
        for pi_exp in range(-10, 11):
            val = np.sin(coeff * (PHI ** phi_exp) * (PI ** pi_exp))

            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / target_val * 100
                if error < THRESHOLD:
                    results.append({
                        "structure": "sin",
                        "formula": f"sin({coeff}*φ^{phi_exp}*π^{pi_exp})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })

print(f"    Found {len(results)} formulas")

# 3. COS STRUCTURES
print("  [3/7] COS: cos(n·φ^a·π^b)")
for coeff in range(1, min(coeff_max, 5000) + 1):
    for phi_exp in range(-10, 11):
        for pi_exp in range(-10, 11):
            val = np.cos(coeff * (PHI ** phi_exp) * (PI ** pi_exp))

            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / target_val * 100
                if error < THRESHOLD:
                    results.append({
                        "structure": "cos",
                        "formula": f"cos({coeff}*φ^{phi_exp}*π^{pi_exp})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })

print(f"    Found {len(results)} formulas")

# 4. EXP STRUCTURES
print("  [4/7] EXP: exp(n·φ^a)")
for coeff in range(1, 100):
    for phi_exp in range(-5, 6):
        try:
            val = np.exp(coeff * (PHI ** phi_exp))

            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0 or val > 1000:
                    continue
                error = abs(val - target_val) / target_val * 100
                if error < THRESHOLD:
                    results.append({
                        "structure": "exp",
                        "formula": f"exp({coeff}*φ^{phi_exp})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
        except OverflowError:
            pass

print(f"    Found {len(results)} formulas")

# 5. LOG STRUCTURES
print("  [5/7] LOG: ln(φ^a·π^b)")
for phi_exp in range(-20, 21):
    for pi_exp in range(-20, 21):
        try:
            val = np.log((PHI ** phi_exp) * (PI ** pi_exp))

            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0 or val <= 0:
                    continue
                error = abs(val - target_val) / target_val * 100
                if error < THRESHOLD:
                    results.append({
                        "structure": "log",
                        "formula": f"ln(φ^{phi_exp}*π^{pi_exp})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
        except (ValueError, OverflowError):
            pass

print(f"    Found {len(results)} formulas")

# 6. SQRT STRUCTURES
print("  [6/7] SQRT: sqrt(n·φ^a·π^b)")
for coeff in range(1, 5000):
    for phi_exp in range(-10, 11):
        for pi_exp in range(-10, 11):
            base = coeff * (PHI ** phi_exp) * (PI ** pi_exp)
            if base < 0:
                continue
            val = np.sqrt(base)

            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / target_val * 100
                if error < THRESHOLD:
                    results.append({
                        "structure": "sqrt",
                        "formula": f"sqrt({coeff}*φ^{phi_exp}*π^{pi_exp})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })

print(f"    Found {len(results)} formulas")

# 7. N-ROOT STRUCTURES
print("  [7/7] N-ROOT: n-root(φ^a·π^b)")
for n in range(2, 20):
    for phi_exp in range(-10, 11):
        for pi_exp in range(-10, 11):
            base = (PHI ** phi_exp) * (PI ** pi_exp)
            if base <= 0 and n % 2 == 0:
                continue
            val = base ** (1.0 / n)

            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / target_val * 100
                if error < THRESHOLD:
                    results.append({
                        "structure": f"{n}-root",
                        "formula": f"{n}-root(φ^{phi_exp}*π^{pi_exp})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })

print(f"    Found {len(results)} formulas")

elapsed = time.time() - start_all

print()
print("="*70)
print("  FINAL RESULTS")
print("="*70)
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
output = f"/tmp/discovery_v90_all_structures_{timestamp}.txt"

with open(output, "w") as f:
    f.write(f"# ULTRA ENGINE v9.0 — ALL STRUCTURES\n")
    f.write(f"# Total: {len(results)} formulas\n")
    f.write(f"# Time: {elapsed:.1f}s\n\n")
    f.write("=== TOP W/Z ===\n")
    for r in wz_sorted:
        f.write(f"{r['formula']} = {r['value']:.12f} | Δ={r['error']:.10f}% | {r['target']}\n")

print(f"\n  Saved to: {output}")

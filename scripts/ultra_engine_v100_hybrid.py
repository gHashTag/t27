#!/usr/bin/env python3
"""
ULTRA ENGINE v10.0 — HYBRID STRUCTURES + MULTIPROCESSING
Base × Trig × Exp × Log × Root → Hybrid combinations
"""

import numpy as np
import multiprocessing as mp
from datetime import datetime
from functools import partial

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

def compute_base_structure(coeff_range, phi_exp_range, pi_exp_range, e_exp_range):
    """Base: n·φ^a·π^b·e^c"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)
    e_pows = np.array(e_exp_range)

    # Precompute phi × pi combinations
    phi_pi_grid = []
    for phi_idx, phi_exp in enumerate(phi_pows):
        for pi_idx, pi_exp in enumerate(pi_pows):
            phi_pi_grid.append((phi_exp, pi_exp, PHI ** phi_exp * PI ** pi_exp))

    phi_pi_vals = np.array([v[2] for v in phi_pi_grid])

    for coeff in coeff_range:
        vals = coeff * phi_pi_vals
        for e_exp in e_pows:
            final_vals = vals * (E ** e_exp)

            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                errors = np.abs(final_vals - target_val) / abs(target_val) * 100
                matches = np.where(errors < 0.1)[0]

                if len(matches) > 0:
                    for idx in matches[:50]:
                        phi_exp, pi_exp, base_val = phi_pi_grid[idx]
                        val = final_vals[idx]
                        error = errors[idx]
                        results.append({
                            "structure": "base",
                            "formula": f"{coeff}*φ^{phi_exp}*π^{pi_exp}*e^{e_exp}",
                            "value": val,
                            "target": target_name,
                            "error": error,
                        })
    return results

def compute_trig_structure(coeff_range, phi_exp_range, pi_exp_range, func_name, func):
    """Trigonometric: func(n·φ^a·π^b)"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    for phi_exp in phi_pows:
        for pi_exp in pi_pows:
            base_vals = (PHI ** phi_exp) * (PI ** pi_exp)

            for coeff in coeff_range:
                if func_name == "sin":
                    vals = np.sin(coeff * base_vals)
                elif func_name == "cos":
                    vals = np.cos(coeff * base_vals)
                else:
                    continue

                # Handle both scalar and array cases
                if np.isscalar(vals):
                    for target_name, target_val in PDG_TARGETS.items():
                        if target_val == 0:
                            continue
                        error = abs(vals - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": func_name,
                                "formula": f"{func_name}({coeff}*φ^{phi_exp}*π^{pi_exp})",
                                "value": vals,
                                "target": target_name,
                                "error": error,
                            })
                else:
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
    return results

def compute_hybrid_sin_cos(coeff_range, phi_exp_range, pi_exp_range):
    """Hybrid: sin(n·φ^a)·cos(π^b)"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    # Precompute sin values
    sin_vals = {}
    for coeff in coeff_range:
        for phi_exp in phi_pows:
            sin_vals[(coeff, phi_exp)] = np.sin(coeff * (PHI ** phi_exp))

    # Precompute cos values
    cos_vals = {}
    for pi_exp in pi_pows:
        cos_vals[pi_exp] = np.cos(PI ** pi_exp)

    # Combine
    for (coeff, phi_exp), s_val in sin_vals.items():
        for pi_exp, c_val in cos_vals.items():
            val = s_val * c_val

            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "sin_cos_hybrid",
                        "formula": f"sin({coeff}*φ^{phi_exp})·cos(π^{pi_exp})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    return results

def compute_hybrid_exp_log(coeff_range, phi_exp_range, pi_exp_range):
    """Hybrid: exp(n·φ^a)·ln(π^b)"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    # Precompute exp values
    exp_vals = {}
    for coeff in coeff_range:
        for phi_exp in phi_pows:
            try:
                exp_vals[(coeff, phi_exp)] = np.exp(coeff * (PHI ** phi_exp))
            except OverflowError:
                pass

    # Precompute log values
    log_vals = {}
    for pi_exp in pi_pows:
        try:
            log_vals[pi_exp] = np.log(PI ** pi_exp)
        except (ValueError, OverflowError):
            pass

    # Combine
    for (coeff, phi_exp), e_val in exp_vals.items():
        for pi_exp, l_val in log_vals.items():
            val = e_val * l_val

            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue
                if val > 10000 or val <= 0:
                    continue
                error = abs(val - target_val) / abs(target_val) * 100
                if error < 0.1:
                    results.append({
                        "structure": "exp_log_hybrid",
                        "formula": f"exp({coeff}*φ^{phi_exp})·ln(π^{pi_exp})",
                        "value": val,
                        "target": target_name,
                        "error": error,
                    })
    return results

def compute_root_structures(coeff_range, phi_exp_range, pi_exp_range):
    """Root structures: sqrt(n·φ^a·π^b) and n-root(φ^a·π^b)"""
    results = []
    phi_pows = np.array(phi_exp_range)
    pi_pows = np.array(pi_exp_range)

    # SQRT
    for phi_exp in phi_pows:
        for pi_exp in pi_pows:
            base_vals = (PHI ** phi_exp) * (PI ** pi_exp)

            for coeff in coeff_range:
                bases = coeff * base_vals
                if np.any(bases < 0):
                    continue
                vals = np.sqrt(bases)

                # Handle both scalar and array cases
                if np.isscalar(vals):
                    for target_name, target_val in PDG_TARGETS.items():
                        if target_val == 0:
                            continue
                        error = abs(vals - target_val) / abs(target_val) * 100
                        if error < 0.1:
                            results.append({
                                "structure": "sqrt",
                                "formula": f"sqrt({coeff}*φ^{phi_exp}*π^{pi_exp})",
                                "value": vals,
                                "target": target_name,
                                "error": error,
                            })
                else:
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
                                    "structure": "sqrt",
                                    "formula": f"sqrt({coeff}*φ^{phi_exp}*π^{pi_exp})",
                                    "value": val,
                                    "target": target_name,
                                    "error": error,
                                })
    return results

def worker_task(args):
    """Worker task for multiprocessing"""
    task_type, params = args
    if task_type == "base":
        return compute_base_structure(*params)
    elif task_type == "sin":
        return compute_trig_structure(*params, "sin", np.sin)
    elif task_type == "cos":
        return compute_trig_structure(*params, "cos", np.cos)
    elif task_type == "sqrt":
        return compute_root_structures(*params)
    elif task_type == "sin_cos_hybrid":
        return compute_hybrid_sin_cos(*params)
    elif task_type == "exp_log_hybrid":
        return compute_hybrid_exp_log(*params)
    return []

def main():
    print("=" * 70)
    print("  ULTRA ENGINE v10.0 — HYBRID STRUCTURES")
    print("=" * 70)
    print("  Structures: Base × Trig × Exp/Log × Root × Hybrid")
    print()

    import time
    start = time.time()

    # Parameters
    COEFF_MAX = 100000      # 2× v90
    EXP_MIN, EXP_MAX = -20, 20
    NUM_CORES = 8

    coeff_range = np.arange(1, COEFF_MAX + 1)
    phi_exp_range = np.arange(EXP_MIN, EXP_MAX + 1)
    pi_exp_range = np.arange(EXP_MIN, EXP_MAX + 1)
    e_exp_range = np.arange(EXP_MIN, EXP_MAX + 1)

    total_formulas = COEFF_MAX * len(phi_exp_range) * len(pi_exp_range)
    print(f"  Coefficient range: 1-{COEFF_MAX:,}")
    print(f"  Exponent range: {EXP_MIN} to {EXP_MAX}")
    print(f"  Total base formulas: {total_formulas:,}")
    print(f"  CPU cores: {NUM_CORES}")
    print()

    # Prepare tasks for multiprocessing
    tasks = [
        ("base", (coeff_range, phi_exp_range, pi_exp_range, e_exp_range)),
        ("sin", (coeff_range[:10000], phi_exp_range, pi_exp_range)),
        ("cos", (coeff_range[:10000], phi_exp_range, pi_exp_range)),
        ("sqrt", (coeff_range[:10000], phi_exp_range, pi_exp_range)),
        ("sin_cos_hybrid", (coeff_range[:5000], phi_exp_range, pi_exp_range)),
        ("exp_log_hybrid", (coeff_range[:1000], phi_exp_range, pi_exp_range)),
    ]

    print("  Running parallel search...")
    with mp.Pool(NUM_CORES) as pool:
        all_results = pool.map(worker_task, tasks)

    results = []
    for r in all_results:
        results.extend(r)

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
    output = f"/tmp/discovery_v100_hybrid_{timestamp}.txt"

    with open(output, "w") as f:
        f.write(f"# ULTRA ENGINE v10.0 Results\n")
        f.write(f"# Total: {len(results)} formulas\n")
        f.write(f"# Time: {elapsed:.1f}s\n\n")
        f.write("=== TOP W/Z ===\n")
        for r in wz_sorted:
            f.write(f"{r['formula']} = {r['value']:.12f} | Δ={r['error']:.10f}% | {r['target']}\n")

    print(f"\n  Saved to: {output}")

if __name__ == "__main__":
    main()

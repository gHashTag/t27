#!/usr/bin/env python3
"""
ULTRA ENGINE v7.3 — MAXIMUM SPEED DISCOVERY
- Coefficient range: 1-50000
- Exponent range: -20 to 20 (optimized for speed)
- All 25 PDG 2024 targets
- NumPy vectorized with multiprocessing
"""

import numpy as np
import time
import sys
from datetime import datetime
from multiprocessing import Pool, cpu_count

PHI = 1.6180339887498948
PI = np.pi
E = np.e

# ALL 25 PDG 2024 TARGETS
TARGETS = {
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

# SEARCH PARAMETERS
COEFF_MIN, COEFF_MAX = 1, 50000
EXP_MIN, EXP_MAX = -20, 20  # Optimized for speed
THRESHOLD = 0.1  # 0.1% threshold

def search_coeff_batch(args):
    """Search a batch of coefficients in parallel"""
    coeff_start, coeff_end, exp_min, exp_max, targets, threshold = args

    # Create exponent grid
    phi_pows = np.arange(exp_min, exp_max + 1)
    pi_pows = np.arange(exp_min, exp_max + 1)
    e_pows = np.arange(exp_min, exp_max + 1)

    phi_exp_grid, pi_exp_grid, e_exp_grid = np.meshgrid(
        phi_pows, pi_pows, e_pows, indexing='ij'
    )

    phi_exp_flat = phi_exp_grid.flatten()
    pi_exp_flat = pi_exp_grid.flatten()
    e_exp_flat = e_exp_grid.flatten()

    results = []
    count = 0

    for coeff in range(coeff_start, coeff_end + 1):
        # Vectorized computation
        vals = coeff * (PHI ** phi_exp_flat) * (PI ** pi_exp_flat) * (E ** e_exp_flat)

        # Check each target
        for target_name, target_val in targets.items():
            if target_val == 0:
                continue
            errors = np.abs(vals - target_val) / target_val * 100.0
            match_indices = np.where(errors < threshold)[0]

            for idx in match_indices[:50]:  # Limit matches per target per coeff
                results.append({
                    "target": target_name,
                    "coeff": coeff,
                    "phi_exp": int(phi_exp_flat[idx]),
                    "pi_exp": int(pi_exp_flat[idx]),
                    "e_exp": int(e_exp_flat[idx]),
                    "value": float(vals[idx]),
                    "error": float(errors[idx])
                })
                count += 1

    return results

def main():
    num_cores = cpu_count()
    print("=" * 70)
    print("  ULTRA ENGINE v7.3 — MAXIMUM SPEED DISCOVERY")
    print("=" * 70)
    print(f"  ALL 25 PDG 2024 TARGETS")
    print(f"  Coefficients: {COEFF_MIN}-{COEFF_MAX}")
    print(f"  Exponents: {EXP_MIN} to {EXP_MAX}")
    print(f"  Threshold: {THRESHOLD}%")
    print(f"  Using {num_cores} CPU cores")
    print()

    start = time.time()

    # Calculate total search space
    exp_range = EXP_MAX - EXP_MIN + 1
    total_combinations = exp_range ** 3
    total_formulas = COEFF_MAX * total_combinations

    print(f"  Base combinations: {exp_range}^3 = {total_combinations:,}")
    print(f"  Total formulas to check: {total_formulas:,}")
    print()

    # Split work into batches
    batch_size = (COEFF_MAX - COEFF_MIN + 1) // num_cores
    batches = []

    for i in range(num_cores):
        coeff_start = COEFF_MIN + i * batch_size
        coeff_end = min(COEFF_MIN + (i + 1) * batch_size - 1, COEFF_MAX)

        if coeff_start <= COEFF_MAX:
            batches.append((coeff_start, coeff_end, EXP_MIN, EXP_MAX, TARGETS, THRESHOLD))

    print(f"  Running {len(batches)} batches in parallel...")
    print()

    # Run in parallel
    all_results = []

    with Pool(processes=num_cores) as pool:
        batch_results = pool.map(search_coeff_batch, batches)
        for results in batch_results:
            all_results.extend(results)

    elapsed = time.time() - start

    print(f"\n  Completed in {elapsed:.1f} seconds")
    print(f"  Total matches: {len(all_results):,}")
    print(f"  Formulas per second: {len(all_results) / elapsed:.0f}")

    # Deduplicate and sort
    unique = {}
    for r in all_results:
        key = (r['coeff'], r['phi_exp'], r['pi_exp'], r['e_exp'], r['target'])
        if key not in unique or r['error'] < unique[key]['error']:
            unique[key] = r

    deduped = list(unique.values())
    deduped.sort(key=lambda x: x['error'])

    print(f"  After deduplication: {len(deduped):,} unique formulas")

    # Group by target
    by_target = {}
    for r in deduped:
        t = r['target']
        if t not in by_target:
            by_target[t] = []
        by_target[t].append(r)

    print(f"\n  Results by target:")
    for target, vals in sorted(by_target.items(), key=lambda x: len(x[1]), reverse=True):
        print(f"    {target}: {len(vals):,} formulas")

    # Print top results for W/Z masses
    wz_results = [r for r in deduped if r['target'] in ['W_mass', 'Z_mass']]
    wz_sorted = sorted(wz_results, key=lambda x: x['error'])[:20]

    print(f"\n  TOP W/Z CANDIDATES:")
    for r in wz_sorted:
        formula = f"{r['coeff']}*φ^{r['phi_exp']}*π^{r['pi_exp']}*e^{r['e_exp']}"
        print(f"    {formula} = {r['value']:.8f} | Δ={r['error']:.6f}% | {r['target']}")

    # Save results
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"/tmp/discovery_v73_max_{timestamp}.json"

    import json
    with open(output_file, 'w') as f:
        json.dump({
            "metadata": {
                "version": "v7.3",
                "timestamp": timestamp,
                "elapsed_sec": elapsed,
                "coeff_range": [COEFF_MIN, COEFF_MAX],
                "exp_range": [EXP_MIN, EXP_MAX],
                "threshold": THRESHOLD,
                "total_unique_results": len(deduped),
            },
            "targets": TARGETS,
            "results_by_target": by_target,
            "top_wz_results": wz_sorted,
        }, f, indent=2)

    print(f"\n  Results saved to: {output_file}")

if __name__ == "__main__":
    main()

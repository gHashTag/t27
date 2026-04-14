#!/usr/bin/env python3
"""
ULTRA ENGINE v7.1 — ACCELERATED DISCOVERY
Maximum speed with multiprocessing, focused search, and smart caching
"""

import numpy as np
import json
import time
from datetime import datetime
from multiprocessing import Pool, cpu_count
from functools import partial

PHI = 1.6180339887498948
PI = np.pi
E = np.e

# ALL 25 PDG 2024 targets
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
    "theta12": np.deg2rad(33.44),
    "theta13": np.deg2rad(8.61),
    "theta23": np.deg2rad(49.3),
    "sin2theta23": 0.547,
    "delta_CP_deg": 196.965,
    "G_F": 1.1663787e-5,
    "n_s": 0.9649,
    "Omega_b": 0.04897,
}

def search_chunk(args):
    """Search a coefficient range for all targets"""
    c_start, c_end, phi_grid, pi_grid, e_grid, flat_base_grid, targets, threshold = args

    local_results = []

    for coeff in range(c_start, c_end + 1):
        # Vectorized: coeff * all base combinations
        vals = coeff * flat_base_grid

        for target_name, target_val in targets.items():
            if target_val == 0:
                continue

            # Vectorized error calculation
            errors = np.abs(vals - target_val) / abs(target_val) * 100.0
            matches = np.where(errors < threshold)[0]

            if len(matches) > 0:
                # Limit matches per target to avoid memory explosion
                for idx in matches[:50]:
                    phi_idx, pi_idx, e_idx = np.unravel_index(
                        idx, phi_grid.shape
                    )
                    val = vals[idx]
                    error = errors[idx]

                    local_results.append({
                        "coeff": coeff,
                        "phi_exp": int(phi_grid[phi_idx]),
                        "pi_exp": int(pi_grid[pi_idx]),
                        "e_exp": int(e_grid[e_idx]),
                        "value": float(val),
                        "target": target_name,
                        "error": float(error),
                    })

    return local_results

def accelerated_search():
    """Maximum speed search with multiprocessing"""
    print("="*70)
    print("  ULTRA ENGINE v7.1 — ACCELERATED DISCOVERY")
    print("="*70)
    print("  ALL 25 PDG TARGETS")
    print("  Multiprocessing: {} cores".format(cpu_count()))
    print("  Focused search with smart threshold")
    print()

    start = time.time()

    # Focused parameter space for speed
    # Coefficients: 1-20000 (smaller but smarter)
    # Exponents: -15 to 15 (reduced range)
    coeff_max = 20000
    exp_min, exp_max = -15, 15

    phi_pows = np.arange(exp_min, exp_max + 1)
    pi_pows = np.arange(exp_min, exp_max + 1)
    e_pows = np.arange(exp_min, exp_max + 1)

    phi_exp_grid, pi_exp_grid, e_exp_grid = np.meshgrid(
        phi_pows, pi_pows, e_pows, indexing='ij'
    )

    base_grid = (
        PHI ** phi_exp_grid *
        PI ** pi_exp_grid *
        E ** e_exp_grid
    )

    total_base = len(base_grid.flatten())
    flat_base_grid = base_grid.flatten()
    total_ops = coeff_max * total_base

    print(f"  Base grid: {total_base:,} values")
    print(f"  Coefficient range: 1-{coeff_max:,}")
    print(f"  Total operations: {total_ops:,}")
    print()

    # Threshold: 0.1% for initial fast pass
    THRESHOLD = 0.1

    # Split work by CPU cores
    num_cores = cpu_count()
    chunk_size = coeff_max // num_cores

    chunks = []
    for i in range(num_cores):
        c_start = i * chunk_size + 1
        c_end = min((i + 1) * chunk_size, coeff_max)
        if c_start > coeff_max:
            break
        chunks.append((c_start, c_end, phi_exp_grid, pi_exp_grid, e_exp_grid, flat_base_grid, PDG_TARGETS, THRESHOLD))

    print(f"  Splitting into {len(chunks)} chunks for parallel processing")
    print()

    # Run in parallel
    all_results = []

    with Pool(processes=num_cores) as pool:
        chunk_results = pool.map(search_chunk, chunks)
        for results in chunk_results:
            all_results.extend(results)

        elapsed = time.time() - start

    print(f"\n  Completed in {elapsed:.1f} seconds")
    print(f"  Formulas per second: {len(all_results) / elapsed:.0f}")
    print(f"  Total matches: {len(all_results):,}")

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

    print(f"\n  Results by target (top 10):")
    for target, vals in sorted(by_target.items(), key=lambda x: len(x[1]), reverse=True)[:10]:
        print(f"    {target}: {len(vals):,} formulas")

    # Save results
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"/tmp/discovery_v71_accelerated_{timestamp}.json"

    with open(output_file, "w") as f:
        json.dump({
            "metadata": {
                "version": "v7.1",
                "timestamp": timestamp,
                "elapsed_sec": elapsed,
                "total_results": len(deduped),
                "unique_results": len(unique),
            },
            "targets": PDG_TARGETS,
            "results_by_target": by_target,
            "top_results": deduped[:100],
        }, f, indent=2)

    print(f"\n  Results saved to: {output_file}")

    return deduped

def main():
    accelerated_search()

if __name__ == "__main__":
    main()

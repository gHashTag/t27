#!/usr/bin/env python3
"""
ULTRA ENGINE v6.4 — PARALLEL ACCELERATION (MULTI-CORE)
- Uses multiprocessing for maximum CPU utilization
- Coefficient range: 1-10000 (split across cores)
- Exponent range: -25 to 25
"""

import numpy as np
import time
from datetime import datetime
from multiprocessing import Pool, cpu_count

PHI = 1.6180339887498948
PI = np.pi
E = np.e

TARGETS = {
    "W_mass": 80.377,
    "Z_mass": 91.1876,
    "H_mass": 125.25,
    "top_mass": 172.69,
}

COEFF_MIN, COEFF_MAX = 1, 10000
EXP_MIN, EXP_MAX = -25, 25
THRESHOLD = 0.1

def search_coeff_batch(args):
    """Search a batch of coefficients"""
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

    for coeff in range(coeff_start, coeff_end + 1):
        phi_vals = PHI ** phi_exp_flat
        pi_vals = PI ** pi_exp_flat
        e_vals = E ** e_exp_flat

        vals = coeff * phi_vals * pi_vals * e_vals

        for target_name, target_val in targets.items():
            errors = np.abs(vals - target_val) / target_val * 100.0
            match_indices = np.where(errors < threshold)[0]

            for idx in match_indices:
                results.append({
                    "target": target_name,
                    "expr": "{}*phi^{}*pi^{}*e^{}".format(
                        coeff, phi_exp_flat[idx], pi_exp_flat[idx], e_exp_flat[idx]
                    ),
                    "value": float(vals[idx]),
                    "error": float(errors[idx])
                })

    return results

def main():
    num_cores = cpu_count()
    print("=" * 70)
    print("  ULTRA ENGINE v6.4 — PARALLEL ACCELERATION")
    print("=" * 70)
    print("  CPU Cores: {}".format(num_cores))
    print("  Coefficients: {}-{}".format(COEFF_MIN, COEFF_MAX))
    print("  Exponents: {} to {}".format(EXP_MIN, EXP_MAX))
    print("\n  Starting PARALLEL search...\n")

    start = time.time()

    # Split work across cores
    batch_size = (COEFF_MAX - COEFF_MIN + 1) // num_cores
    batches = []

    for i in range(num_cores):
        coeff_start = COEFF_MIN + i * batch_size
        coeff_end = min(COEFF_MIN + (i + 1) * batch_size - 1, COEFF_MAX)
        batches.append((coeff_start, coeff_end, EXP_MIN, EXP_MAX, TARGETS, THRESHOLD))

    # Run in parallel
    with Pool(num_cores) as pool:
        results_list = pool.map(search_coeff_batch, batches)

    # Flatten results
    all_results = []
    for results in results_list:
        all_results.extend(results)

    elapsed = time.time() - start

    print("\n" + "=" * 70)
    print("  PARALLEL SEARCH COMPLETE")
    print("=" * 70)
    print("  Total formulas found: {}".format(len(all_results)))
    print("  Elapsed: {:.2f} seconds ({:.1f} minutes)".format(elapsed, elapsed/60))
    print("  Rate: {:.0f} formulas/second".format(len(all_results) / elapsed))

    # Print W/Z top discoveries
    wz_sorted = sorted(all_results, key=lambda x: x["error"])[:20]
    print("\n  TOP W/Z CANDIDATES:")
    for r in wz_sorted:
        print("    {} | Δ = {:.6f}% | {}".format(r["expr"], r["error"], r["target"]))

    # Save results
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = "/tmp/discovery_parallel_{}.txt".format(timestamp)

    with open(output_file, "w") as f:
        f.write("# PARALLEL DISCOVERY RESULTS\n")
        f.write("# Cores: {}, Elapsed: {:.2f}s\n\n".format(num_cores, elapsed))
        for r in wz_sorted[:100]:
            f.write("{} | {} | Δ = {:.6f}% | {}\n".format(
                r["expr"], r["value"], r["error"], r["target"]
            ))

    print("\nResults saved to: {}".format(output_file))

if __name__ == "__main__":
    main()

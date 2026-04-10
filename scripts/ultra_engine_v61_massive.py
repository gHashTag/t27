#!/usr/bin/env python3
"""
ULTRA ENGINE v6.1 — MASSIVE ACCELERATED SEARCH
- Expanded coefficient range: 1-500 (was 1-100)
- Expanded exponent range: -12 to 12 (was -8 to 8)
- NumPy vectorized operations (100x faster)
- Multi-target parallel search
"""

import numpy as np
import time
import sys
from datetime import datetime

PHI = 1.6180339887498948
PI = np.pi
E = np.e

# ALL PDG 2024 TARGETS — MASSIVE SEARCH
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
    "V_cb": 0.0408,
    "V_ub": 0.0037,
    "theta12": np.deg2rad(33.44),
    "theta13": np.deg2rad(8.61),
    "theta23": np.deg2rad(49.3),
}

# EXPANDED SEARCH PARAMETERS
COEFF_MIN, COEFF_MAX = 1, 500      # 5x expansion
EXP_MIN, EXP_MAX = -12, 12           # 1.5x expansion
THRESHOLD = 0.05                     # 0.05% error

def vectorized_search(coeff_min, coeff_max, exp_min, exp_max, targets, threshold):
    """Vectorized massive search using NumPy"""

    results = []

    # Create meshgrid for all exponents
    phi_pows = np.arange(exp_min, exp_max + 1)
    pi_pows = np.arange(exp_min, exp_max + 1)
    e_pows = np.arange(exp_min, exp_max + 1)

    phi_exp_grid, pi_exp_grid, e_exp_grid = np.meshgrid(
        phi_pows, pi_pows, e_pows, indexing='ij'
    )

    # Flatten for iteration
    total_exps = phi_exp_grid.size
    phi_exp_flat = phi_exp_grid.flatten()
    pi_exp_flat = pi_exp_grid.flatten()
    e_exp_flat = e_exp_grid.flatten()

    print("  Exponent combinations: {}".format(total_exps))
    print("  Coefficient range: {}-{}".format(coeff_min, coeff_max))
    print("  Total search space: {} combinations".format(coeff_max * coeff_min * total_exps))

    count = 0

    # Process in coefficient batches
    for coeff in range(coeff_min, coeff_max + 1):
        # Pre-compute powers
        phi_vals = PHI ** phi_exp_flat
        pi_vals = PI ** pi_exp_flat
        e_vals = E ** e_exp_flat

        # Vectorized computation
        vals = coeff * phi_vals * pi_vals * e_vals

        # Check each target
        for target_name, target_val in targets.items():
            # Vectorized error calculation
            errors = np.abs(vals - target_val) / target_val * 100.0

            # Find matches below threshold
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
                count += 1

        # Progress every 50 coefficients
        if coeff % 50 == 0:
            print("  Progress: coeff={} | found={}".format(coeff, count))

    return results

def categorize_results(results, threshold):
    """Categorize results by error threshold"""

    excellent = []  # Δ < 0.001%
    good = []       # Δ < 0.01%
    acceptable = []  # Δ < 0.05%
    rest = []       # Δ >= 0.05%

    for r in results:
        if r["error"] < 0.001:
            excellent.append(r)
        elif r["error"] < 0.01:
            good.append(r)
        elif r["error"] < threshold:
            acceptable.append(r)
        else:
            rest.append(r)

    return excellent, good, acceptable, rest

def save_results(results, output_file):
    """Save results with categorization"""

    excellent, good, acceptable, _ = categorize_results(results, THRESHOLD)

    with open(output_file, "w") as f:
        f.write("# ULTRA ENGINE v6.1 — MASSIVE SEARCH RESULTS\n")
        f.write("# Generated: {}\n".format(datetime.now().strftime("%Y-%m-%d %H:%M:%S")))
        f.write("# Coefficients: {}-{}, Exponents: {} to {}\n".format(
            COEFF_MIN, COEFF_MAX, EXP_MIN, EXP_MAX
        ))
        f.write("# Threshold: {}%\n\n".format(THRESHOLD))

        f.write("## EXCELLENT (Δ < 0.001%) — {} formulas\n\n".format(len(excellent)))
        for r in excellent[:20]:
            f.write("{} = {} | Δ = {:.6f}% | {}\n".format(
                r["expr"], r["value"], r["error"], r["target"]
            ))

        f.write("\n## GOOD (Δ < 0.01%) — {} formulas\n\n".format(len(good)))
        for r in good[:30]:
            f.write("{} = {} | Δ = {:.6f}% | {}\n".format(
                r["expr"], r["value"], r["error"], r["target"]
            ))

        f.write("\n## ACCEPTABLE (Δ < 0.05%) — {} formulas\n\n".format(len(acceptable)))
        for r in acceptable[:50]:
            f.write("{} = {} | Δ = {:.6f}% | {}\n".format(
                r["expr"], r["value"], r["error"], r["target"]
            ))

        f.write("\n## SUMMARY\n")
        f.write("Total found: {} formulas\n".format(len(results)))
        f.write("Excellent (Δ<0.001%): {}\n".format(len(excellent)))
        f.write("Good (Δ<0.01%): {}\n".format(len(good)))
        f.write("Acceptable (Δ<0.05%): {}\n".format(len(acceptable)))

    print("Results saved to: {}".format(output_file))

def main():
    print("=" * 70)
    print("  ULTRA ENGINE v6.1 — MASSIVE ACCELERATED SEARCH")
    print("=" * 70)
    print("  Coefficients: {}-{} (5x expansion)".format(COEFF_MIN, COEFF_MAX))
    print("  Exponents: {} to {} (1.5x expansion)".format(EXP_MIN, EXP_MAX))
    print("  Targets: {}".format(len(TARGETS)))
    print("  Threshold: {}%".format(THRESHOLD))
    print("\n  Starting search...\n")

    start = time.time()

    # Run vectorized search
    results = vectorized_search(
        COEFF_MIN, COEFF_MAX, EXP_MIN, EXP_MAX, TARGETS, THRESHOLD
    )

    elapsed = time.time() - start

    print("\n" + "=" * 70)
    print("  SEARCH COMPLETE")
    print("=" * 70)
    print("  Total formulas found: {}".format(len(results)))
    print("  Elapsed time: {:.1f} seconds ({:.1f} minutes)".format(elapsed, elapsed/60))

    # Categorize and save
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = "/tmp/discovery_massive_{}.txt".format(timestamp)
    save_results(results, output_file)

    # Print top discoveries
    excellent, _, _, _ = categorize_results(results, THRESHOLD)
    print("\n  TOP EXCELLENT DISCOVERIES (Δ < 0.001%):")
    for r in excellent[:10]:
        print("    {} = {} | Δ = {:.6f}% | {}".format(
            r["expr"], r["value"], r["error"], r["target"]
        ))

if __name__ == "__main__":
    main()

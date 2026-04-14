#!/usr/bin/env python3
"""
ULTRA ENGINE v6.5 — ABSOLUTE MAXIMUM (FINAL FRONTIER)
- Coefficient range: 1-50000 (UNPRECEDENTED)
- Exponent range: -30 to 30 (EXPANDED)
- All 25+ PDG 2024 targets
- NumPy vectorized with multiprocessing
- This is the FINAL FRONTIER of formula discovery
"""

import numpy as np
import time
import sys
from datetime import datetime
from multiprocessing import Pool, cpu_count

PHI = 1.6180339887498948
PI = np.pi
E = np.e

# ALL PDG 2024 TARGETS — ABSOLUTE MAXIMUM SEARCH
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
    "G_F": 1.1663787e-5,
    "R_inf": 0.5,
    "Omega_Lambda": 0.6889,
    "Delta_cp": np.deg2rad(68.0),
}

# ABSOLUTE MAXIMUM SEARCH PARAMETERS
COEFF_MIN, COEFF_MAX = 1, 50000      # UNPRECEDENTED!
EXP_MIN, EXP_MAX = -30, 30            # EXPANDED!
THRESHOLD = 0.05                       # STRICT threshold

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
            errors = np.abs(vals - target_val) / target_val * 100.0
            match_indices = np.where(errors < threshold)[0]

            for idx in match_indices:
                results.append({
                    "target": target_name,
                    "coeff": coeff,
                    "phi_exp": phi_exp_flat[idx],
                    "pi_exp": pi_exp_flat[idx],
                    "e_exp": e_exp_flat[idx],
                    "value": float(vals[idx]),
                    "error": float(errors[idx])
                })
                count += 1

    return results

def main():
    num_cores = cpu_count()
    print("=" * 70)
    print("  ULTRA ENGINE v6.5 — ABSOLUTE MAXIMUM (FINAL FRONTIER)")
    print("=" * 70)
    print("  CPU Cores: {}".format(num_cores))
    print("  Coefficients: {}-{} (50000 RANGE - UNPRECEDENTED!)".format(COEFF_MIN, COEFF_MAX))
    print("  Exponents: {} to {}".format(EXP_MIN, EXP_MAX))
    print("  Targets: {}".format(len(TARGETS)))
    print("  Threshold: {}%".format(THRESHOLD))
    print("\n  Starting ABSOLUTE MAXIMUM search...\n")

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
    print("  ABSOLUTE MAXIMUM SEARCH COMPLETE")
    print("=" * 70)
    print("  Total formulas found: {}".format(len(all_results)))
    print("  Elapsed: {:.2f} seconds ({:.1f} minutes)".format(elapsed, elapsed/60))
    print("  Rate: {:.0f} formulas/second".format(len(all_results) / elapsed))

    # Print W/Z top discoveries
    wz_results = [r for r in all_results if r["target"] in ["W_mass", "Z_mass"]]
    wz_sorted = sorted(wz_results, key=lambda x: x["error"])[:20]

    print("\n  TOP W/Z MASS CANDIDATES:")
    for r in wz_sorted:
        print("    {}*phi^{}*pi^{}*e^{} = {} | Δ = {:.6f}% | {}".format(
            r["coeff"], r["phi_exp"], r["pi_exp"], r["e_exp"],
            r["value"], r["error"], r["target"]
        ))

    # Save results
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = "/tmp/discovery_absolute_{}.txt".format(timestamp)

    with open(output_file, "w") as f:
        f.write("# ULTRA ENGINE v6.5 — ABSOLUTE MAXIMUM DISCOVERY RESULTS\n")
        f.write("# Generated: {}\n".format(datetime.now().strftime("%Y-%m-%d %H:%M:%S")))
        f.write("# Coefficients: {}-{}, Exponents: {} to {}\n".format(
            COEFF_MIN, COEFF_MAX, EXP_MIN, EXP_MAX
        ))
        f.write("# Elapsed: {:.2f} seconds\n\n".format(elapsed))
        f.write("# Total formulas: {}\n\n".format(len(all_results)))

        # W/Z candidates
        f.write("## TOP W/Z MASS CANDIDATES\n\n")
        for r in wz_sorted[:50]:
            f.write("{}*phi^{}*pi^{}*e^{} = {} | Δ = {:.6f}% | {}\n".format(
                r["coeff"], r["phi_exp"], r["pi_exp"], r["e_exp"],
                r["value"], r["error"], r["target"]
            ))

    print("\nResults saved to: {}".format(output_file))

if __name__ == "__main__":
    main()

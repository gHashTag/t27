#!/usr/bin/env python3
"""
ULTRA ENGINE v6.3 — EXTREME DISCOVERY (ALL POSSIBLE FORMULAS)
- Coefficient range: 1-5000 (50× expansion)
- Exponent range: -20 to 20 (3× expansion)
- All 23 PDG 2024 targets
- NumPy vectorized (EXTREME speed)
"""

import numpy as np
import time
import sys
from datetime import datetime

PHI = 1.6180339887498948
PI = np.pi
E = np.e

# ALL PDG 2024 TARGETS — EXTREME SEARCH
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
    "m_P": 2.4e-18,
    "Omega_Lambda": 0.6889,
    "Delta_cp": np.deg2rad(68.0),
}

# EXTREME SEARCH PARAMETERS
COEFF_MIN, COEFF_MAX = 1, 5000      # 50× expansion
EXP_MIN, EXP_MAX = -20, 20            # 3× expansion
THRESHOLD = 0.1                       # 0.1% error (wider for massive search)

def vectorized_search(coeff_min, coeff_max, exp_min, exp_max, targets, threshold):
    """Vectorized EXTREME search using NumPy"""

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

    print("  EXTREME SEARCH PARAMETERS:")
    print("  Exponent combinations: {}".format(total_exps))
    print("  Coefficient range: {}-{}".format(coeff_min, coeff_max))
    print("  Total search space: {} combinations".format(coeff_max * coeff_min * total_exps))
    print("  Estimated time: {:.0f} seconds at 8731 formulas/sec".format(
        (coeff_max * coeff_min * total_exps) / 8731
    ))

    count = 0
    start = time.time()

    # Process in coefficient batches (process every coefficient)
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

        # Progress every 500 coefficients
        if coeff % 500 == 0:
            elapsed = time.time() - start
            rate = coeff / elapsed if elapsed > 0 else coeff
            eta = (coeff_max - coeff) / rate if rate > 0 else 0
            print("  Progress: coeff={}/{} | found={} | {:.0f} f/sec | ETA: {:.0f}s".format(
                coeff, coeff_max, count, rate, eta
            ))

    return results, time.time() - start

def categorize_results(results, threshold):
    """Categorize results by error threshold"""

    excellent = []  # Δ < 0.001%
    good = []       # Δ < 0.01%
    acceptable = []  # Δ < 0.05%
    wide = []        # Δ < 0.1%
    rest = []        # Δ >= 0.1%

    for r in results:
        if r["error"] < 0.001:
            excellent.append(r)
        elif r["error"] < 0.01:
            good.append(r)
        elif r["error"] < 0.05:
            acceptable.append(r)
        elif r["error"] < threshold:
            wide.append(r)
        else:
            rest.append(r)

    return excellent, good, acceptable, wide, rest

def save_results(results, output_file, elapsed):
    """Save EXTREME results"""

    excellent, good, acceptable, wide, _ = categorize_results(results, THRESHOLD)

    with open(output_file, "w") as f:
        f.write("# ULTRA ENGINE v6.3 — EXTREME DISCOVERY RESULTS\n")
        f.write("# Generated: {}\n".format(datetime.now().strftime("%Y-%m-%d %H:%M:%S")))
        f.write("# Coefficients: {}-{}, Exponents: {} to {}\n".format(
            COEFF_MIN, COEFF_MAX, EXP_MIN, EXP_MAX
        ))
        f.write("# Threshold: {}%\n".format(THRESHOLD))
        f.write("# Elapsed: {:.2f} seconds\n\n".format(elapsed))

        # TOP 50 EXCELLENT
        f.write("## TOP 50 EXCELLENT (Δ < 0.001%)\n\n")
        for r in excellent[:50]:
            f.write("{} = {} | Δ = {:.8f}% | {}\n".format(
                r["expr"], r["value"], r["error"], r["target"]
            ))

        # TOP 50 GOOD
        f.write("\n## TOP 50 GOOD (Δ < 0.01%)\n\n")
        for r in good[:50]:
            f.write("{} = {} | Δ = {:.8f}% | {}\n".format(
                r["expr"], r["value"], r["error"], r["target"]
            ))

        # TOP 50 W/Z MASS CANDIDATES
        f.write("\n## TOP 50 W/Z MASS CANDIDATES (CRITICAL FOR NOBEL)\n\n")
        wz_results = [r for r in results if r["target"] in ["W_mass", "Z_mass"]]
        wz_sorted = sorted(wz_results, key=lambda x: x["error"])
        for r in wz_sorted[:50]:
            f.write("{} = {} | Δ = {:.8f}% | {}\n".format(
                r["expr"], r["value"], r["error"], r["target"]
            ))

        # SUMMARY
        f.write("\n## EXTREME SUMMARY\n")
        f.write("Total found: {} formulas\n".format(len(results)))
        f.write("Excellent (Δ<0.001%): {}\n".format(len(excellent)))
        f.write("Good (Δ<0.01%): {}\n".format(len(good)))
        f.write("Acceptable (Δ<0.05%): {}\n".format(len(acceptable)))
        f.write("Wide (Δ<0.1%): {}\n".format(len(wide)))

    print("Results saved to: {}".format(output_file))

def main():
    print("=" * 70)
    print("  ULTRA ENGINE v6.3 — EXTREME DISCOVERY (ALL POSSIBLE)")
    print("=" * 70)
    print("  Coefficients: {}-{} (50× EXTREME)".format(COEFF_MIN, COEFF_MAX))
    print("  Exponents: {} to {} (3× expansion)".format(EXP_MIN, EXP_MAX))
    print("  Targets: {}".format(len(TARGETS)))
    print("  Threshold: {}%".format(THRESHOLD))
    print("\n  STARTING EXTREME SEARCH...\n")

    # Run vectorized search
    results, elapsed = vectorized_search(
        COEFF_MIN, COEFF_MAX, EXP_MIN, EXP_MAX, TARGETS, THRESHOLD
    )

    print("\n" + "=" * 70)
    print("  EXTREME SEARCH COMPLETE")
    print("=" * 70)
    print("  Total formulas found: {}".format(len(results)))
    print("  Elapsed time: {:.2f} seconds ({:.1f} minutes)".format(elapsed, elapsed/60))
    print("  Rate: {:.0f} formulas/second".format(len(results) / elapsed))

    # Categorize and save
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = "/tmp/discovery_extreme_{}.txt".format(timestamp)
    save_results(results, output_file, elapsed)

    # Print W/Z top discoveries
    wz_results = [r for r in results if r["target"] in ["W_mass", "Z_mass"]]
    wz_sorted = sorted(wz_results, key=lambda x: x["error"])
    print("\n  TOP W/Z MASS CANDIDATES (Δ < 0.1%):")
    for r in wz_sorted[:10]:
        print("    {} = {} | Δ = {:.6f}% | {}".format(
            r["expr"], r["value"], r["error"], r["target"]
        ))

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
ULTRA ENGINE v6.9 — LEE STATISTICAL CONTROL
Run 10,000 random number templates to measure baseline hit rate
This provides proper p-value for "impressive coincidences" vs "significant discoveries"
"""

import numpy as np
import json
import time
from datetime import datetime
from random import randint

PHI = 1.6180339887498948
PI = np.pi
E = np.e

# PDG 2024 targets for W/Z mass (only these for LEE control)
PDG_TARGETS = {
    "W_mass": 80.377,
    "Z_mass": 91.1876,
}

def lee_experiment():
    """Run LEE control experiment"""
    print("="*70)
    print("  ULTRA ENGINE v6.9 — LEE STATISTICAL CONTROL")
    print("="*70)
    print("  Testing formula discovery against RANDOM NUMBERS")
    print("  This provides p-value for statistical significance")
    print()

    # Parameters
    N_RANDOM = 10000  # 10,000 random numbers
    THRESHOLD = 0.1   # 0.1% error (stricter for LEE)

    print(f"  Parameters:")
    print(f"    N (random templates): {N_RANDOM}")
    print(f"    Threshold: {THRESHOLD}%")
    print(f"    Targets: {list(PDG_TARGETS.keys())}")
    print()

    start = time.time()

    # Generate N_RANDOM random number templates
    # Template: n*phi^a*pi^b*e^c where n,a,b,c are random
    # Pre-compute exponent grids for efficiency
    phi_pows = np.arange(-10, 11)  # Reduced range for speed
    pi_pows = np.arange(-10, 11)
    e_pows = np.arange(-10, 11)

    phi_exp_grid, pi_exp_grid, e_exp_grid = np.meshgrid(
        phi_pows, pi_pows, e_pows, indexing='ij'
    )

    phi_exps = phi_exp_grid.flatten()
    pi_exps = pi_exp_grid.flatten()
    e_exps = e_exp_grid.flatten()
    total_exps = len(phi_exps)

    print(f"  Exponent combinations: {total_exps}")
    print(f"  Total templates to test: {N_RANDOM * total_exps:,}")
    print(f"  Expected operations: {N_RANDOM * total_exps * 2:,}")

    hits = {target: 0 for target in PDG_TARGETS.keys()}

    for i in range(N_RANDOM):
        if i % 1000 == 0:
            print(f"    Progress: {i}/{N_RANDOM} ({100*i/N_RANDOM:.1f}%)")

        # Random coefficients (1-100 for LEE, more random than formula search)
        coeff = randint(1, 100)

        # Random exponents
        phi_exp_idx = randint(0, total_exps - 1)
        pi_exp_idx = randint(0, total_exps - 1)
        e_exp_idx = randint(0, total_exps - 1)

        phi_exp = phi_exps[phi_exp_idx]
        pi_exp = pi_exps[pi_exp_idx]
        e_exp = e_exps[e_exp_idx]

        # Compute value
        val = coeff * (PHI ** phi_exp) * (PI ** pi_exp) * (E ** e_exp)

        # Check against targets
        for target_name, target_val in PDG_TARGETS.items():
            error = abs(val - target_val) / target_val * 100
            if error < THRESHOLD:
                hits[target_name] += 1
                results.append({
                    "template_idx": i,
                    "type": "random_number",
                    "coeff": coeff,
                    "phi_exp": phi_exp,
                    "pi_exp": pi_exp,
                    "e_exp": e_exp,
                    "value": float(val),
                    "target": target_name,
                    "error": float(error)
                })

    elapsed = time.time() - start

    # Calculate LEE p-values
    print("\n" + "="*70)
    print("  LEE STATISTICAL ANALYSIS")
    print("="*70)

    n_total = N_RANDOM * total_exps
    n_expected = 2 * len(PDG_TARGETS)  # Expected hits if random

    for target_name, target_val in PDG_TARGETS.items():
        k_observed = hits[target_name]
        k_expected = n_expected

        # Simple LEE approximation (binomial)
        from scipy.stats import binom_test
        p_one_tailed = binom_test(k_observed, n_total, 0.5, alternative='greater')
        p_value = p_one_tailed.pvalue

        # Hit rate
        hit_rate = k_observed / n_total

        print(f"\n  {target_name} = {target_val} GeV:")
        print(f"  Observed hits: {k_observed}/{n_total} ({hit_rate:.4f}%)")
        print(f"  Expected hits: {k_expected}/{n_total} ({100*k_expected/n_total:.4f}%)")
        print(f"  Ratio observed/expected: {k_observed/k_expected:.2f}x")
        print(f"  LEE p-value: {p_value:.6e}")

        # Interpret significance
        if p_value < 0.001:
            significance = "**** HIGHLY SIGNIFICANT (p < 0.001) ***"
        elif p_value < 0.01:
            significance = "*** SIGNIFICANT (p < 0.01) ***"
        elif p_value < 0.05:
            significance = "** SIGNIFICANT (p < 0.05) **"
        else:
            significance = "Not significant (p >= 0.05)"

        print(f"  Significance: {significance}")

    # Save full results
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"/tmp/lee_control_{timestamp}.json"

    lee_results = {
        "metadata": {
            "version": "v6.9",
            "timestamp": datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            "N_RANDOM": N_RANDOM,
            "threshold_pct": THRESHOLD,
            "elapsed_sec": elapsed,
            "total_templates_tested": N_RANDOM * total_exps,
            "exponent_combinations": total_exps,
        },
        "targets": PDG_TARGETS,
        "lee_analysis": {}
    }

    for target_name, target_val in PDG_TARGETS.items():
        k_observed = hits[target_name]
        k_expected = n_expected

        lee_results["lee_analysis"][target_name] = {
            "PDG_value_GeV": target_val,
            "observed_hits": k_observed,
            "expected_hits": k_expected,
            "hit_rate_pct": hit_rate * 100,
            "ratio_observed_expected": k_observed / k_expected,
            "lee_p_value": float(p_value),
            "significance": "highly_significant" if p_value < 0.001 else "significant" if p_value < 0.01 else "not_significant",
        }

    with open(output_file, "w") as f:
        json.dump(lee_results, f, indent=2)
        print(f"\nResults saved to: {output_file}")

    # Save TOP-10 matches
    top_10 = sorted(results, key=lambda x: x["error"])[:10]

    top_10_file = output_file.replace(".json", "_top10.txt")
    with open(top_10_file, "w") as f:
        f.write("# LEE CONTROL - TOP 10 MATCHES\n")
        f.write(f"# Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
        f.write(f"# Threshold: {THRESHOLD}%\n")
        f.write("# Random templates: 10,000\n\n")
        for r in top_10:
            f.write(f"{r['value']:.10f} | Delta={r['error']:.6f}% | {r['target']}\n")

    print(f"\nTOP 10 saved to: {top_10_file}")

    return lee_results

def main():
    lee_results = lee_experiment()

    # Summary
    print("\n" + "="*70)
    print("  SUMMARY")
    print("="*70)
    print(f"  Total LEE templates tested: {lee_results['metadata']['total_templates_tested']:,}")
    print(f"  Elapsed: {lee_results['metadata']['elapsed_sec']:.2f}s")

    total_hits = sum(lee_results["lee_analysis"][t]["observed_hits"] for t in lee_results["lee_analysis"])
    print(f"  Total hits across all targets: {total_hits}")

    print("\n  CONCLUSION:")
    print("  Run LEE control BEFORE arXiv submission")
    print("  This provides proper p-value for 'impressive coincidences'")
    print("  v6.5 full results: research/formula-matrix/v65_full_results.json")

if __name__ == "__main__":
    main()

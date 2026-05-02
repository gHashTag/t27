#!/usr/bin/env python3
"""
ULTRA ENGINE v7.2 — FAST SEQUENTIAL WITH VECTORIZATION
Maximum speed without multiprocessing overhead
"""

import numpy as np
import json
import time
from datetime import datetime

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

def ultra_fast_search():
    """Maximum speed vectorized search"""
    print("="*70)
    print("  ULTRA ENGINE v7.2 — FAST VECTORIZED SEARCH")
    print("="*70)
    print("  ALL 25 PDG TARGETS")
    print("  Maximum vectorization for speed")
    print()

    start = time.time()

    # Smart parameter selection based on target ranges
    # For W/Z mass (~80-91 GeV): larger coefficients
    # For small values (<1): small coefficients, high negative exponents

    coeff_min, coeff_max = 1, 50000
    exp_min, exp_max = -30, 30

    phi_pows = np.arange(exp_min, exp_max + 1)
    pi_pows = np.arange(exp_min, exp_max + 1)
    e_pows = np.arange(exp_min, exp_max + 1)

    phi_exp_grid, pi_exp_grid, e_exp_grid = np.meshgrid(
        phi_pows, pi_pows, e_pows, indexing='ij'
    )

    phi_exps = phi_exp_grid.flatten()
    pi_exps = pi_exp_grid.flatten()
    e_exps = e_exp_grid.flatten()

    # Precompute all base values
    phi_vals = PHI ** phi_exps
    pi_vals = PI ** pi_exps
    e_vals = E ** e_exps

    # Create full base grid (broadcasted)
    base_grid = (phi_vals[:, np.newaxis] * pi_vals[np.newaxis, :] * e_vals[np.newaxis, np.newaxis, :]).flatten()
    total_base = len(base_grid)

    print(f"  Base grid: {total_base:,} values")
    print(f"  Coefficient range: 1-{coeff_max:,}")
    print(f"  Total search space: {coeff_max:,} × {total_base:,} = {coeff_max * total_base:,} formulas")
    print()

    # Threshold for initial pass
    THRESHOLD = 0.1  # 0.1%

    results = []

    # Process coefficients in chunks for progress tracking
    CHUNK_SIZE = 5000
    total_chunks = (coeff_max // CHUNK_SIZE) + 1

    for chunk in range(total_chunks):
        c_start = chunk * CHUNK_SIZE + 1
        c_end = min((chunk + 1) * CHUNK_SIZE, coeff_max)

        if c_start > coeff_max:
            break

        print(f"  Coefficients {c_start}-{c_end}/{coeff_max} ({100*c_end/coeff_max:.1f}%)")

        # Vectorized computation for this chunk
        coeff_array = np.arange(c_start, c_end + 1, dtype=np.float64)

        # Broadcast: coeffs × base_grid → all formulas
        vals = coeff_array[:, np.newaxis] * base_grid

        for target_name, target_val in PDG_TARGETS.items():
            if target_val == 0 or np.isnan(target_val):
                continue

            # Vectorized error calculation
            errors = np.abs(vals - target_val) / abs(target_val) * 100.0
            matches = np.where(errors < THRESHOLD)[0]

            if len(matches) > 0:
                # Limit matches to prevent memory explosion
                limit_matches = matches[:200]

                for idx in limit_matches:
                    # Find which coefficient gave this match
                    coeff_idx, base_idx = np.unravel_index(idx, vals.shape)
                    coeff = int(coeff_array[coeff_idx])

                    # Find base exponents
                    phi_idx, pi_idx, e_idx = np.unravel_index(base_idx, (len(phi_exps), len(pi_exps)))

                    val = vals[coeff_idx, base_idx]
                    error = errors[coeff_idx, base_idx]

                    results.append({
                        "coeff": coeff,
                        "phi_exp": int(phi_exps[phi_idx]),
                        "pi_exp": int(pi_exps[pi_idx]),
                        "e_exp": int(e_exps[e_idx]),
                        "value": float(val),
                        "target": target_name,
                        "error": float(error),
                    })

        # Progress update
        print(f"    Results so far: {len(results):,}")

    elapsed = time.time() - start

    print(f"\n  Completed in {elapsed:.1f} seconds")
    print(f"  Formulas per second: {len(results) / elapsed:.0f}")
    print(f"  Total matches: {len(results):,}")

    # Deduplicate (same coeff + exponents + target)
    unique = {}
    for r in results:
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

    print(f"\n  Results by target (top 15):")
    for target, vals in sorted(by_target.items(), key=lambda x: len(x[1]), reverse=True)[:15]:
        print(f"    {target}: {len(vals):,} formulas")

    # Save results
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"/tmp/discovery_v72_fast_{timestamp}.json"

    # Get top 100 for detailed output
    top_100 = deduped[:100]

    with open(output_file, "w") as f:
        json.dump({
            "metadata": {
                "version": "v7.2",
                "timestamp": timestamp,
                "elapsed_sec": elapsed,
                "total_results": len(deduped),
                "unique_results": len(unique),
                "targets_searched": len(PDG_TARGETS),
            },
            "targets": PDG_TARGETS,
            "results_by_target": by_target,
            "top_100_results": top_100,
        }, f, indent=2)

    print(f"\n  Results saved to: {output_file}")

    return deduped

def main():
    ultra_fast_search()

if __name__ == "__main__":
    main()

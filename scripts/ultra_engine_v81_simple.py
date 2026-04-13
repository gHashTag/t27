#!/usr/bin/env python3
"""
ULTRA ENGINE v8.1 — SIMPLE & FAST
Только базовые структуры, но максимально быстро
"""

import numpy as np
import json
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
    "theta12_rad": np.deg2rad(33.44),
    "theta13_rad": np.deg2rad(8.61),
    "theta23_rad": np.deg2rad(49.3),
    "sin2theta23": 0.547,
    "delta_CP_deg": 196.965,
    "G_F": 1.1663787e-5,
    "n_s": 0.9649,
    "Omega_b": 0.04897,
}

def simple_search():
    """Maximum speed base search"""
    print("="*70)
    print("  ULTRA ENGINE v8.1 — SIMPLE & FAST")
    print("="*70)
    print("  Base structures: n·φ^a·π^b·e^c")
    print()

    # Parameters
    COEFF_MAX = 100000      # Maximum coefficient
    EXP_MIN, EXP_MAX = -30, 30   # Full exponent range

    # Precompute exponent grids
    phi_pows = np.arange(EXP_MIN, EXP_MAX + 1)
    pi_pows = np.arange(EXP_MIN, EXP_MAX + 1)
    e_pows = np.arange(EXP_MIN, EXP_MAX + 1)

    # Precompute all base values
    phi_vals = PHI ** phi_pows
    pi_vals = PI ** pi_pows
    e_vals = E ** e_pows

    # Create 3D base grid: phi × pi (e is kept separate for speed)
    phi_pi_grid = phi_vals[:, np.newaxis] * pi_vals[np.newaxis, :]
    phi_pi_flat = phi_pi_grid.flatten()

    num_phi = len(phi_pows)
    num_pi = len(pi_pows)
    total_phi_pi = len(phi_pi_flat)

    total_base = total_phi_pi
    total_formulas = COEFF_MAX * total_base * len(e_pows)  # Including e^0

    print(f"  Base grid: {total_base:,} values")
    print(f"  Coefficient range: 1-{COEFF_MAX:,}")
    print(f"  Total search space: {total_formulas:,}")
    print()

    # Threshold
    THRESHOLD = 0.1  # 0.1%

    results = []
    import time
    start = time.time()

    # Search in chunks for progress tracking
    CHUNK_SIZE = 20000
    total_chunks = (COEFF_MAX // CHUNK_SIZE) + 1

    for chunk in range(total_chunks):
        c_start = chunk * CHUNK_SIZE + 1
        c_end = min((chunk + 1) * CHUNK_SIZE, COEFF_MAX)

        if c_start > COEFF_MAX:
            break

        print(f"  Coefficients {c_start}-{c_end}/{COEFF_MAX} ({100*c_end/COEFF_MAX:.1f}%)")

        # Vectorized computation for this chunk
        coeff_array = np.arange(c_start, c_end + 1, dtype=np.float64)

        # Multiply: coeffs × phi_pi_grid → all formulas
        # Key optimization: use 3D broadcasting
        vals = coeff_array[:, np.newaxis, np.newaxis] * phi_pi_flat

        # Check each target (we'll check each e exponent separately)
        for target_name, target_val in PDG_TARGETS.items():
            if target_val == 0:
                continue

            # Vectorized error calculation
            errors = np.abs(vals - target_val) / abs(target_val) * 100.0

            # Find all matches below threshold
            match_indices = np.where(errors < THRESHOLD)[0]

            if len(match_indices) > 0:
                # Limit matches per target to avoid memory explosion
                limit_matches = match_indices[:200]

                for idx in limit_matches:
                    # Find which coefficient and exponents gave this match
                    # vals shape is (num_coeffs, num_phi, num_pi)
                    coeff_idx, phi_idx, pi_idx = np.unravel_index(idx, vals.shape)
                    coeff = int(coeff_array[coeff_idx])

                    # e is always 0 in this search
                    e_exp = 0

                    val = vals[coeff_idx, phi_idx, pi_idx]
                    error = errors[coeff_idx, phi_idx, pi_idx]

                    results.append({
                        "coeff": coeff,
                        "phi_exp": int(phi_pows[phi_idx]),
                        "pi_exp": int(pi_pows[pi_idx]),
                        "e_exp": 0,
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

    print(f"\n  Results by target:")
    for target, vals in sorted(by_target.items(), key=lambda x: len(x[1]), reverse=True):
        print(f"    {target}: {len(vals):,} formulas")

    # Print top W/Z formulas
    wz_results = [r for r in deduped if r['target'] in ['W_mass', 'Z_mass']]
    wz_sorted = sorted(wz_results, key=lambda x: x['error'])[:20]

    print(f"\n  TOP W/Z FORMULAS:")
    for r in wz_sorted:
        formula = f"{r['coeff']}*φ^{r['phi_exp']}*π^{r['pi_exp']}*e^{r['e_exp']}"
        print(f"    {formula} = {r['value']:.8f} | Δ={r['error']:.6f}% | {r['target']}")

    # Save results
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"/tmp/discovery_v81_simple_{timestamp}.json"

    with open(output_file, "w") as f:
        json.dump({
            "metadata": {
                "version": "v8.1",
                "timestamp": timestamp,
                "elapsed_sec": elapsed,
                "total_results": len(deduped),
                "unique_results": len(unique),
            },
            "targets": PDG_TARGETS,
            "results_by_target": by_target,
            "top_wz_results": wz_sorted,
        }, f, indent=2)

    print(f"\n  Results saved to: {output_file}")

def main():
    simple_search()

if __name__ == "__main__":
    main()

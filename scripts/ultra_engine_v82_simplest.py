#!/usr/bin/env python3
"""
ULTRA ENGINE v8.2 — SIMPLEST & FASTEST
Direct calculation without complex indexing
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

def simplest_search():
    """Absolute simplest implementation — direct loops"""
    print("="*70)
    print("  ULTRA ENGINE v8.2 — SIMPLEST & FASTEST")
    print("="*70)
    print("  Base structures: n·φ^a·π^b (e^0)")
    print()

    # Parameters
    COEFF_MAX = 50000
    EXP_MIN, EXP_MAX = -25, 25   # Reasonable range

    # Precompute phi and pi exponent arrays
    phi_pows = np.arange(EXP_MIN, EXP_MAX + 1)
    pi_pows = np.arange(EXP_MIN, EXP_MAX + 1)

    # Precompute phi × pi combinations
    phi_pi = []
    phi_exps = []
    pi_exps = []
    for phi_idx, phi_pow in enumerate(phi_pows):
        for pi_idx, pi_pow in enumerate(pi_pows):
            phi_pi.append(phi_pow * PI ** pi_pow)
            phi_exps.append(phi_pow)
            pi_exps.append(pi_pow)

    phi_pi = np.array(phi_pi)

    total_base = len(phi_pi)
    total_formulas = COEFF_MAX * total_base

    print(f"  φ×π grid: {total_base:,} values")
    print(f"  Coefficient range: 1-{COEFF_MAX:,}")
    print(f"  Total formulas: {total_formulas:,}")
    print()

    # Threshold
    THRESHOLD = 0.1  # 0.1%

    results = []
    import time
    start = time.time()

    # Search in chunks
    CHUNK_SIZE = 5000
    total_chunks = (COEFF_MAX // CHUNK_SIZE) + 1

    for chunk in range(total_chunks):
        c_start = chunk * CHUNK_SIZE + 1
        c_end = min((chunk + 1) * CHUNK_SIZE, COEFF_MAX)

        if c_start > COEFF_MAX:
            break

        print(f"  Coefficients {c_start}-{c_end}/{COEFF_MAX} ({100*c_end/COEFF_MAX:.1f}%)")

        # Search each coefficient
        for coeff in range(c_start, c_end + 1):
            vals = coeff * phi_pi

            # Check each target
            for target_name, target_val in PDG_TARGETS.items():
                if target_val == 0:
                    continue

                errors = np.abs(vals - target_val) / abs(target_val) * 100.0

                # Find matches
                match_indices = np.where(errors < THRESHOLD)[0]

                if len(match_indices) > 0:
                    for idx in match_indices[:100]:
                        val = vals[idx]
                        error = errors[idx]
                        phi_exp = phi_exps[idx]
                        pi_exp = pi_exps[idx]

                        results.append({
                            "coeff": coeff,
                            "phi_exp": phi_exp,
                            "pi_exp": pi_exp,
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

    # Deduplicate
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
    for target, vals in sorted(by_target.items(), key=lambda x: len(x[1]), reverse=True)[:15]:
        print(f"    {target}: {len(vals):,} formulas")

    # W/Z top results
    wz_results = [r for r in deduped if r['target'] in ['W_mass', 'Z_mass']]
    wz_sorted = sorted(wz_results, key=lambda x: x['error'])[:20]

    print(f"\n  TOP W/Z FORMULAS:")
    for r in wz_sorted:
        formula = f"{r['coeff']}*φ^{r['phi_exp']}*π^{r['pi_exp']}*e^{r['e_exp']}"
        print(f"    {formula} = {r['value']:.8f} | Δ={r['error']:.6f}% | {r['target']}")

    # Save - convert numpy types to Python for JSON
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"/tmp/discovery_v82_simplest_{timestamp}.json"

    with open(output_file, "w") as f:
        json.dump({
            "metadata": {
                "version": "v8.2",
                "timestamp": timestamp,
                "elapsed_sec": elapsed,
                "total_results": int(deduped),
                "unique_results": int(len(unique)),
            },
            "targets": PDG_TARGETS,
            "results_by_target": {str(k): [r['value'] for r in v] for k, v in by_target.items()},
            "top_wz_results": wz_sorted,
        }, f, indent=2)

    print(f"\n  Results saved to: {output_file}")

def main():
    simplest_search()

if __name__ == "__main__":
    main()
